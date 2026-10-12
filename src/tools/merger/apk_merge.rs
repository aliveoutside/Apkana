use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

// Bound memory usage when unpacking nested APK ZIPs.
const MAX_NESTED_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_UNPACKED_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_NESTED_ENTRIES: usize = 10_000;

use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{ZipArchive, result::ZipError};

use crate::tools::{OutputStream, ToolOutput};

use super::archive::{SplitApk, SplitType};
use super::{manifest, resources};

pub fn merge_splits(
    base_apk: &[u8],
    splits: &[SplitApk],
    output: &mut Vec<ToolOutput>,
) -> Result<Vec<u8>, String> {
    let mut base_archive = ZipArchive::new(Cursor::new(base_apk))
        .map_err(|e| format!("Merge: invalid base APK ZIP: {e}"))?;
    if base_archive.len() > MAX_NESTED_ENTRIES {
        return Err(String::from("Merge: too many base APK ZIP entries"));
    }
    let mut unpacked_total = 0u64;

    let mut out_writer = ZipWriter::new(Cursor::new(Vec::new()));
    let mut existing_entries = HashSet::new();
    let mut payloads: HashMap<String, Vec<u8>> = HashMap::new();
    let mut split_manifests = Vec::new();
    let mut classes = HashSet::new();
    let mut base_seen = HashSet::new();
    let mut deferred_dex = Vec::new();
    let mut split_arsc_tables: Vec<Vec<u8>> = Vec::new();

    let mut base_manifest: Option<Vec<u8>> = None;
    let mut base_resources: Option<Vec<u8>> = None;
    let mut manifest_compression = CompressionMethod::Deflated;

    let mut has_abi_splits = false;

    for i in 0..base_archive.len() {
        let mut entry = base_archive
            .by_index(i)
            .map_err(|e| format!("Merge: failed to read base APK entry #{i}: {e}"))?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let compression = entry.compression();
        validate_dex_filename(&name)?;
        if !base_seen.insert(name.clone()) {
            return Err(format!("Merge: duplicate base ZIP entry {name}"));
        }

        if entry.size() > MAX_NESTED_ENTRY_BYTES {
            return Err(format!("Merge: base entry '{name}' exceeds size limit"));
        }
        let bytes = read_entry_bytes(&mut entry)
            .map_err(|e| format!("Merge: failed to read base APK entry '{name}': {e}"))?;
        add_unpacked(&mut unpacked_total, bytes.len(), &name)?;

        if is_signature_artifact(&name) {
            continue;
        }

        if name == "AndroidManifest.xml" {
            manifest_compression = compression;
            base_manifest = Some(bytes);
            continue;
        }

        if name == "resources.arsc" {
            base_resources = Some(bytes);
            continue;
        }

        if name.starts_with("lib/") {
            has_abi_splits = true;
        }
        write_entry(&mut out_writer, &name, compression, &bytes)
            .map_err(|e| format!("Merge: failed to copy base APK entry '{name}': {e}"))?;
        if is_dex_file(&name) {
            super::dex::collect_classes(&bytes, &mut classes)?;
        }
        payloads.insert(name.clone(), bytes);
        existing_entries.insert(name);
    }

    for split in splits {
        push_info(
            output,
            format!(
                "Processing split: {} ({})",
                split.name,
                split.split_type.name()
            ),
        );

        let mut split_archive = ZipArchive::new(Cursor::new(split.data.as_slice()))
            .map_err(|e| format!("Merge: invalid split APK '{}': {e}", split.name))?;
        if split_archive.len() > MAX_NESTED_ENTRIES {
            return Err(format!("Merge: too many entries in split '{}'", split.name));
        }

        if split.split_type == SplitType::Abi {
            has_abi_splits = true;
        }

        let mut seen = HashSet::new();
        for i in 0..split_archive.len() {
            let mut entry = split_archive.by_index(i).map_err(|e| {
                format!(
                    "Merge: failed to read split entry #{i} in '{}': {e}",
                    split.name
                )
            })?;

            if entry.is_dir() {
                continue;
            }

            let name = entry.name().to_string();
            let compression = entry.compression();
            validate_dex_filename(&name)?;

            if !seen.insert(name.clone()) {
                return Err(format!(
                    "Merge: duplicate ZIP entry {name} in {}",
                    split.name
                ));
            }

            if entry.size() > MAX_NESTED_ENTRY_BYTES {
                return Err(format!("Merge: split entry '{name}' exceeds size limit"));
            }
            let bytes = read_entry_bytes(&mut entry).map_err(|e| {
                format!(
                    "Merge: failed to read split entry '{}' in '{}': {e}",
                    name, split.name
                )
            })?;
            add_unpacked(&mut unpacked_total, bytes.len(), &name)?;

            if name == "AndroidManifest.xml" {
                split_manifests.push(bytes);
                continue;
            }

            if is_signature_artifact(&name) {
                continue;
            }

            if name == "resources.arsc" {
                split_arsc_tables.push(bytes);
                continue;
            }

            if is_dex_file(&name) {
                super::dex::collect_classes(&bytes, &mut classes)?;
                deferred_dex.push(DeferredDex {
                    source_name: name,
                    bytes,
                    compression,
                });
                continue;
            }

            if existing_entries.contains(&name) {
                if payloads.get(&name) != Some(&bytes) {
                    return Err(format!(
                        "Merge: conflicting ZIP payload {name} in {}; refusing data loss",
                        split.name
                    ));
                }
                continue;
            }

            if name.starts_with("lib/") {
                has_abi_splits = true;
            }

            write_entry(&mut out_writer, &name, compression, &bytes).map_err(|e| {
                format!(
                    "Merge: failed to write split entry '{}' from '{}': {e}",
                    name, split.name
                )
            })?;
            payloads.insert(name.clone(), bytes);
            existing_entries.insert(name);
        }
    }

    let mut next_dex_index = next_available_dex_index(&existing_entries);
    for dex in deferred_dex {
        let name = loop {
            let candidate = dex_name(next_dex_index);
            if !existing_entries.contains(&candidate) {
                break candidate;
            }
            next_dex_index += 1;
        };

        write_entry(&mut out_writer, &name, dex.compression, &dex.bytes).map_err(|e| {
            format!(
                "Merge: failed to add DEX '{}' to output: {e}",
                dex.source_name
            )
        })?;
        existing_entries.insert(name);
        next_dex_index += 1;
    }

    let count = existing_entries.iter().filter(|n| is_dex_file(n)).count();
    if (1..=count).any(|i| !existing_entries.contains(&dex_name(i))) {
        return Err("Merge: non-contiguous DEX filenames; Android would skip code".into());
    }
    validate_native_sets(&payloads)?;
    if let Some(manifest_bytes) = base_manifest {
        let sanitized_manifest = manifest::merge_manifests(
            &manifest_bytes,
            &split_manifests,
            has_abi_splits,
            !classes.is_empty(),
            output,
        )?;
        write_entry(
            &mut out_writer,
            "AndroidManifest.xml",
            manifest_compression,
            &sanitized_manifest,
        )
        .map_err(|e| format!("Merge: failed to write sanitized AndroidManifest.xml: {e}"))?;
    } else {
        return Err("Merge: base APK has no manifest".into());
    }

    if let Some(base_arsc) = base_resources {
        let merged_arsc = resources::merge_resource_tables(&base_arsc, &split_arsc_tables, output)?;
        resources::validate_file_references(&merged_arsc, &existing_entries)?;
        write_entry(
            &mut out_writer,
            "resources.arsc",
            CompressionMethod::Stored,
            &merged_arsc,
        )
        .map_err(|e| format!("Merge: failed to write merged resources.arsc: {e}"))?;
    } else if !split_arsc_tables.is_empty() {
        let mut iter = split_arsc_tables.into_iter();
        let first = iter
            .next()
            .ok_or_else(|| String::from("Merge: missing first split resources.arsc"))?;
        let remaining: Vec<Vec<u8>> = iter.collect();
        let merged_arsc = resources::merge_resource_tables(&first, &remaining, output)?;
        resources::validate_file_references(&merged_arsc, &existing_entries)?;
        write_entry(
            &mut out_writer,
            "resources.arsc",
            CompressionMethod::Stored,
            &merged_arsc,
        )
        .map_err(|e| format!("Merge: failed to write merged split resources.arsc: {e}"))?;
    }

    let writer = out_writer
        .finish()
        .map_err(|e| format!("Merge: failed to finalize output APK: {e}"))?;

    Ok(writer.into_inner())
}

#[derive(Debug)]
struct DeferredDex {
    source_name: String,
    bytes: Vec<u8>,
    compression: CompressionMethod,
}

fn read_entry_bytes(entry: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    entry
        .take(MAX_NESTED_ENTRY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_NESTED_ENTRY_BYTES {
        return Err(String::from("uncompressed ZIP entry exceeds 256 MiB limit"));
    }
    Ok(bytes)
}

fn add_unpacked(total: &mut u64, size: usize, name: &str) -> Result<(), String> {
    *total = total.saturating_add(size as u64);
    if *total > MAX_UNPACKED_BYTES {
        Err(format!(
            "Merge: unpacked data exceeds 1 GiB limit at '{name}'"
        ))
    } else {
        Ok(())
    }
}

fn write_entry(
    writer: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &str,
    compression: CompressionMethod,
    bytes: &[u8],
) -> Result<(), ZipError> {
    let options = SimpleFileOptions::default().compression_method(compression);
    writer.start_file(name, options)?;
    std::io::Write::write_all(writer, bytes)?;
    Ok(())
}

fn is_signature_artifact(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();

    if upper == "STAMP-CERT-SHA256" || upper == "STAMP-CERT-SHA256.CER" {
        return true;
    }

    let Some(meta_name) = upper.strip_prefix("META-INF/") else {
        return false;
    };

    if meta_name.contains('/') {
        return false;
    }

    meta_name == "MANIFEST.MF"
        || meta_name.ends_with(".RSA")
        || meta_name.ends_with(".DSA")
        || meta_name.ends_with(".EC")
        || meta_name.ends_with(".SF")
}

fn is_dex_file(name: &str) -> bool {
    parse_dex_index(name).is_some()
}

fn parse_dex_index(name: &str) -> Option<usize> {
    if name.contains('/') || !name.starts_with("classes") || !name.ends_with(".dex") {
        return None;
    }

    let suffix = &name["classes".len()..name.len() - ".dex".len()];
    if suffix.is_empty() {
        return Some(1);
    }

    suffix
        .parse::<usize>()
        .ok()
        .filter(|i| *i >= 2 && i.to_string() == suffix)
}

fn validate_dex_filename(name: &str) -> Result<(), String> {
    if !name.contains('/')
        && name.starts_with("classes")
        && name.ends_with(".dex")
        && parse_dex_index(name).is_none()
    {
        return Err(format!("Merge: noncanonical DEX filename '{name}'"));
    }
    Ok(())
}

fn next_available_dex_index(existing_entries: &HashSet<String>) -> usize {
    let mut index = 1usize;
    loop {
        let name = dex_name(index);
        if !existing_entries.contains(&name) {
            return index;
        }
        index += 1;
    }
}

fn dex_name(index: usize) -> String {
    if index == 1 {
        String::from("classes.dex")
    } else {
        format!("classes{index}.dex")
    }
}

fn push_info(output: &mut Vec<ToolOutput>, line: impl Into<String>) {
    output.push(ToolOutput {
        line: line.into(),
        stream: OutputStream::Stdout,
    });
}

fn validate_native_sets(payloads: &HashMap<String, Vec<u8>>) -> Result<(), String> {
    let mut abis: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (name, data) in payloads {
        if let Some(path) = name.strip_prefix("lib/") {
            let (abi, lib) = path
                .split_once('/')
                .ok_or("Merge: invalid native library path")?;
            let expected = match abi {
                "armeabi" | "armeabi-v7a" => (1, 40),
                "arm64-v8a" => (2, 183),
                "x86" => (1, 3),
                "x86_64" => (2, 62),
                "riscv64" => (2, 243),
                _ => return Err(format!("Merge: unsupported native ABI '{abi}'")),
            };
            if data.len() < 20
                || &data[..4] != b"\x7fELF"
                || data[4] != expected.0
                || data[5] != 1
                || u16::from_le_bytes([data[18], data[19]]) != expected.1
            {
                return Err(format!(
                    "Merge: native library '{name}' is not a valid ELF for {abi}"
                ));
            }
            abis.entry(abi).or_default().insert(lib);
        }
    }
    if let Some(first) = abis.values().next()
        && abis.values().any(|libs| libs != first)
    {
        return Err("Merge: native library sets differ between ABIs; supply a complete single-ABI split set".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_signature_artifact;

    #[test]
    fn strips_only_signature_related_meta_inf_files() {
        assert!(is_signature_artifact("META-INF/CERT.RSA"));
        assert!(is_signature_artifact("META-INF/CERT.SF"));
        assert!(is_signature_artifact("META-INF/MANIFEST.MF"));
        assert!(is_signature_artifact("STAMP-CERT-SHA256"));
    }

    #[test]
    fn preserves_non_signature_meta_inf_files() {
        assert!(!is_signature_artifact(
            "META-INF/services/kotlinx.coroutines.internal.MainDispatcherFactory"
        ));
        assert!(!is_signature_artifact(
            "META-INF/com/android/build/gradle/app-metadata.properties"
        ));
    }
}

#[cfg(test)]
mod merge_safety_tests {
    use super::*;

    #[test]
    fn refuses_incomplete_multi_abi_libraries() {
        fn elf(class: u8, machine: u8) -> Vec<u8> {
            let mut b = vec![0; 20];
            b[..4].copy_from_slice(b"\x7fELF");
            b[4] = class;
            b[5] = 1;
            b[18] = machine;
            b
        }
        let payloads = HashMap::from([
            ("lib/x86_64/liba.so".to_owned(), elf(2, 62)),
            ("lib/arm64-v8a/libb.so".to_owned(), elf(2, 183)),
        ]);
        assert!(
            validate_native_sets(&payloads)
                .unwrap_err()
                .contains("differ between ABIs")
        );
    }

    #[test]
    fn tracks_unpacked_limit() {
        let mut size = MAX_UNPACKED_BYTES - 3;
        assert!(add_unpacked(&mut size, 4, "huge.bin").is_err());
    }
}

#[cfg(test)]
mod binary_fixture_tests {
    use super::super::archive::{InputFormat, extract_split_set};
    use super::*;
    fn fixture() -> super::super::archive::SplitSet {
        extract_split_set(
            std::path::Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/feature.apks"
            )),
            InputFormat::Apks,
        )
        .unwrap()
    }
    #[test]
    fn rejects_duplicate_classes_from_real_feature_dex() {
        let mut set = fixture();
        let feature = set
            .splits
            .iter()
            .find(|s| s.split_type == SplitType::Feature)
            .unwrap()
            .clone();
        set.splits.push(feature);
        assert!(
            merge_splits(&set.base_apk, &set.splits, &mut vec![])
                .unwrap_err()
                .contains("duplicate DEX class")
        );
    }
    #[test]
    fn rejects_conflicting_assets_instead_of_skipping_them() {
        let mut set = fixture();
        let feature = set
            .splits
            .iter_mut()
            .find(|s| s.split_type == SplitType::Feature)
            .unwrap();
        let mut writer = ZipWriter::new_append(Cursor::new(feature.data.clone())).unwrap();
        write_entry(
            &mut writer,
            "assets/base.txt",
            CompressionMethod::Stored,
            b"different feature payload",
        )
        .unwrap();
        feature.data = writer.finish().unwrap().into_inner();
        assert!(
            merge_splits(&set.base_apk, &set.splits, &mut vec![])
                .unwrap_err()
                .contains("conflicting ZIP payload assets/base.txt")
        );
    }
    #[test]
    fn rejects_missing_density_payloads() {
        let mut set = fixture();
        let density = set
            .splits
            .iter_mut()
            .find(|s| s.split_type == SplitType::Density)
            .unwrap();
        let mut zip = ZipArchive::new(Cursor::new(&density.data)).unwrap();
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for i in 0..zip.len() {
            let mut e = zip.by_index(i).unwrap();
            if e.name().starts_with("res/") {
                continue;
            }
            let name = e.name().to_owned();
            let compression = e.compression();
            let mut data = Vec::new();
            e.read_to_end(&mut data).unwrap();
            write_entry(&mut writer, &name, compression, &data).unwrap();
        }
        density.data = writer.finish().unwrap().into_inner();
        assert!(
            merge_splits(&set.base_apk, &set.splits, &mut vec![])
                .unwrap_err()
                .contains("resource payload")
        );
    }
}
