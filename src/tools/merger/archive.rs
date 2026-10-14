use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_json::Value;
use zip::ZipArchive;

// Bounds for untrusted, uncompressed archive data.
const MAX_ARCHIVE_ENTRIES: usize = 10_000;
const MAX_APK_ENTRIES: usize = 128;
const MAX_APK_SIZE: u64 = 512 * 1024 * 1024;
const MAX_APKS_TOTAL: u64 = 1024 * 1024 * 1024;
const MAX_MANIFEST_JSON: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    Apks,
    Xapk,
    Apkm,
}

impl InputFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::Apks => "APKS",
            Self::Xapk => "XAPK",
            Self::Apkm => "APKM",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitType {
    Abi,
    Density,
    Language,
    Feature,
    Configuration,
}

impl SplitType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Abi => "abi",
            Self::Density => "density",
            Self::Language => "language",
            Self::Feature => "feature",
            Self::Configuration => "configuration",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SplitApk {
    pub name: String,
    pub data: Vec<u8>,
    pub split_type: SplitType,
}

#[derive(Debug, Clone)]
pub struct SplitSet {
    pub base_apk: Vec<u8>,
    pub splits: Vec<SplitApk>,
    pub warnings: Vec<String>,
}

pub fn detect_format(path: &Path) -> Result<InputFormat, String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "apks" => Ok(InputFormat::Apks),
        "xapk" => Ok(InputFormat::Xapk),
        "apkm" => Ok(InputFormat::Apkm),
        _ => Err(String::from(
            "Merge: unsupported input format. Expected .apks, .xapk, or .apkm",
        )),
    }
}

pub fn extract_split_set(path: &Path, format: InputFormat) -> Result<SplitSet, String> {
    let file = File::open(path).map_err(|e| format!("Merge: failed to open input archive: {e}"))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Merge: failed to read ZIP archive: {e}"))?;

    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!(
            "Merge: too many archive entries (>{MAX_ARCHIVE_ENTRIES})"
        ));
    }
    let mut apk_entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut seen_apk_paths = HashSet::new();
    let mut unpacked_total = 0u64;
    let mut warnings = Vec::new();
    let mut xapk_manifest: Option<Value> = None;
    let mut obb_count = 0usize;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Merge: failed to read archive entry #{i}: {e}"))?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let lower_name = name.to_ascii_lowercase();

        if lower_name.ends_with(".apk") {
            if apk_entries.len() >= MAX_APK_ENTRIES {
                return Err(format!(
                    "Merge: too many APK modules (maximum {MAX_APK_ENTRIES})"
                ));
            }
            if !seen_apk_paths.insert(lower_name.clone()) {
                return Err(format!("Merge: duplicate APK entry '{name}'"));
            }
            let declared = entry.size();
            if declared > MAX_APK_SIZE || unpacked_total.saturating_add(declared) > MAX_APKS_TOTAL {
                return Err(format!(
                    "Merge: APK entry '{name}' exceeds safe size limits"
                ));
            }
            let bytes = read_limited(&mut entry, MAX_APK_SIZE, &name)?;
            unpacked_total = unpacked_total.saturating_add(bytes.len() as u64);
            if unpacked_total > MAX_APKS_TOTAL {
                return Err(String::from("Merge: total APK data exceeds 1 GiB limit"));
            }
            apk_entries.push((name, bytes));
            continue;
        }

        if lower_name.ends_with("manifest.json") {
            if entry.size() > MAX_MANIFEST_JSON {
                return Err(String::from("Merge: manifest.json is too large"));
            }
            let bytes = read_limited(&mut entry, MAX_MANIFEST_JSON, "manifest.json")?;
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                xapk_manifest = Some(value);
            }
            continue;
        }

        if lower_name.ends_with(".obb") {
            obb_count += 1;
        }
    }

    if apk_entries.is_empty() {
        return Err(String::from("Merge: no APK entries found in archive"));
    }
    if format == InputFormat::Xapk {
        if obb_count > 0 {
            warnings.push(format!(
                "XAPK contains {obb_count} OBB file(s); OBB extraction is not performed"
            ));
        }

        if let Some(manifest) = &xapk_manifest {
            if let Some(package_name) = manifest.get("package_name").and_then(Value::as_str) {
                warnings.push(format!("XAPK package: {package_name}"));
            }

            if let Some(expansions) = manifest.get("expansions").and_then(Value::as_array)
                && !expansions.is_empty()
            {
                warnings.push(format!(
                    "XAPK manifest lists {} expansion file(s); these are not extracted",
                    expansions.len()
                ));
            }
        }
    }

    if obb_count > 0
        || xapk_manifest
            .as_ref()
            .and_then(|m| m.get("expansions"))
            .and_then(Value::as_array)
            .is_some_and(|a| !a.is_empty())
    {
        return Err("Merge: archive requires OBB expansion files; a single APK cannot preserve this application's data".into());
    }
    let metadata: Vec<_> = apk_entries
        .iter()
        .map(|(name, bytes)| super::metadata::inspect(bytes).map_err(|e| format!("{name}: {e}")))
        .collect::<Result<_, _>>()?;
    let base_index = super::metadata::validate_set(&metadata)?;
    let (base_name, base_apk) = apk_entries.remove(base_index);
    let mut metadata = metadata;
    metadata.remove(base_index);
    let splits = apk_entries
        .into_iter()
        .zip(metadata)
        .map(|((name, data), m)| SplitApk {
            name,
            data,
            split_type: m.kind,
        })
        .collect::<Vec<_>>();

    warnings.push(format!("Base APK: {base_name}"));
    if !splits.is_empty() {
        warnings.push(format!("Found {} split APK module(s)", splits.len()));
    }

    Ok(SplitSet {
        base_apk,
        splits,
        warnings,
    })
}

fn read_limited<R: Read>(reader: &mut R, limit: u64, name: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Merge: failed to read '{name}': {e}"))?;
    if bytes.len() as u64 > limit {
        return Err(format!(
            "Merge: '{name}' exceeds safe uncompressed size limit"
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supported_formats() {
        for (name, format) in [
            ("a.apks", InputFormat::Apks),
            ("a.xapk", InputFormat::Xapk),
            ("a.APKM", InputFormat::Apkm),
        ] {
            assert_eq!(detect_format(Path::new(name)).unwrap(), format);
        }
        assert!(detect_format(Path::new("a.apk")).is_err());
    }
    #[test]
    fn bounded_reader_rejects_oversized_content() {
        assert!(read_limited(&mut std::io::Cursor::new(vec![0; 33]), 32, "a").is_err());
    }
}
