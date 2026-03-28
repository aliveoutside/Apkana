use std::collections::HashSet;
use std::io::{Cursor, Read};

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;
use zip::{result::ZipError, ZipArchive};

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

    let mut out_writer = ZipWriter::new(Cursor::new(Vec::new()));
    let mut existing_entries = HashSet::new();
    let mut deferred_dex = Vec::new();
    let mut split_arsc_tables: Vec<Vec<u8>> = Vec::new();

    let mut base_manifest: Option<Vec<u8>> = None;
    let mut base_resources: Option<Vec<u8>> = None;
    let mut manifest_compression = CompressionMethod::Deflated;
    let mut resources_compression = CompressionMethod::Deflated;
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

        let bytes = read_entry_bytes(&mut entry)
            .map_err(|e| format!("Merge: failed to read base APK entry '{name}': {e}"))?;

        if is_signature_artifact(&name) {
            continue;
        }

        if name == "AndroidManifest.xml" {
            manifest_compression = compression;
            base_manifest = Some(bytes);
            continue;
        }

        if name == "resources.arsc" {
            resources_compression = compression;
            base_resources = Some(bytes);
            continue;
        }

        if name == "res/xml/splits0.xml" {
            continue;
        }

        write_entry(&mut out_writer, &name, compression, &bytes)
            .map_err(|e| format!("Merge: failed to copy base APK entry '{name}': {e}"))?;
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

        if split.split_type == SplitType::Abi {
            has_abi_splits = true;
        }

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

            if name == "AndroidManifest.xml" {
                continue;
            }

            let bytes = read_entry_bytes(&mut entry).map_err(|e| {
                format!(
                    "Merge: failed to read split entry '{}' in '{}': {e}",
                    name, split.name
                )
            })?;

            if is_signature_artifact(&name) || name == "res/xml/splits0.xml" {
                continue;
            }

            if name == "resources.arsc" {
                split_arsc_tables.push(bytes);
                continue;
            }

            if is_dex_file(&name) {
                deferred_dex.push(DeferredDex {
                    source_name: name,
                    bytes,
                    compression,
                });
                continue;
            }

            if existing_entries.contains(&name) {
                push_warn(output, format!("Skipping duplicate split entry: {name}"));
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

    if let Some(manifest_bytes) = base_manifest {
        let sanitized_manifest =
            manifest::sanitize_manifest(&manifest_bytes, has_abi_splits, output)?;
        write_entry(
            &mut out_writer,
            "AndroidManifest.xml",
            manifest_compression,
            &sanitized_manifest,
        )
        .map_err(|e| format!("Merge: failed to write sanitized AndroidManifest.xml: {e}"))?;
    } else {
        push_warn(
            output,
            String::from("Base APK did not contain AndroidManifest.xml; output may be invalid"),
        );
    }

    if let Some(base_arsc) = base_resources {
        let merged_arsc = resources::merge_resource_tables(&base_arsc, &split_arsc_tables, output)?;
        write_entry(
            &mut out_writer,
            "resources.arsc",
            resources_compression,
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
        write_entry(
            &mut out_writer,
            "resources.arsc",
            resources_compression,
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

fn read_entry_bytes(entry: &mut impl Read) -> Result<Vec<u8>, ZipError> {
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
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

    suffix.parse::<usize>().ok()
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

fn push_warn(output: &mut Vec<ToolOutput>, line: impl Into<String>) {
    output.push(ToolOutput {
        line: line.into(),
        stream: OutputStream::Stderr,
    });
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
