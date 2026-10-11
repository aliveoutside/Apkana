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
    Unknown,
}

impl SplitType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Abi => "abi",
            Self::Density => "density",
            Self::Language => "language",
            Self::Unknown => "unknown",
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
    // A bundletool .apks may contain incompatible standalone and split variants.
    // We cannot safely select the target variant without interpreting toc.pb.
    if format == InputFormat::Apks
        && apk_entries.len() > 1
        && apk_entries.iter().any(|(name, _)| {
            let name = name.to_ascii_lowercase();
            name.starts_with("standalones/")
                || name.starts_with("instant/")
                || name.starts_with("universal/")
                || name.starts_with("asset-slices/")
        })
    {
        return Err(String::from(
            "Merge: APKS contains standalone/instant/asset variants; select one compatible split set first",
        ));
    }

    if format == InputFormat::Xapk {
        if obb_count > 0 {
            warnings.push(format!(
                "XAPK contains {obb_count} OBB file(s); OBB extraction is not performed"
            ));
        }

        if let Some(manifest) = xapk_manifest {
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

    let base_index = choose_base_index(&apk_entries).ok_or_else(|| {
        String::from("Merge: ambiguous base APK; expected exactly one base module")
    })?;

    let (base_name, base_apk) = apk_entries.swap_remove(base_index);
    let mut splits = Vec::new();
    for (name, data) in apk_entries {
        let split_type = classify_split_type(&name);
        splits.push(SplitApk {
            name,
            data,
            split_type,
        });
    }

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

fn choose_base_index(entries: &[(String, Vec<u8>)]) -> Option<usize> {
    let bases: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, (name, _))| {
            let base = file_name_only(name).to_ascii_lowercase();
            (base == "base.apk" || base == "base-master.apk").then_some(i)
        })
        .collect();
    if bases.len() == 1 {
        return bases.first().copied();
    }
    if bases.len() > 1 {
        return None;
    }
    if entries.len() == 1 {
        return Some(0);
    }

    let candidates: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, (name, _))| (!looks_like_split_name(file_name_only(name))).then_some(i))
        .collect();
    (candidates.len() == 1).then(|| candidates[0])
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

fn looks_like_split_name(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    lower.starts_with("split_")
        || lower.starts_with("config.")
        || lower.contains("split_config")
        || lower.contains(".config.")
}

fn classify_split_type(name: &str) -> SplitType {
    let lower = name.to_ascii_lowercase();

    const ABI_TOKENS: &[&str] = &[
        "arm64_v8a",
        "armeabi_v7a",
        "armeabi",
        "x86_64",
        "x86",
        "riscv64",
        "mips64",
        "mips",
    ];

    if ABI_TOKENS.iter().any(|token| lower.contains(token)) {
        return SplitType::Abi;
    }

    const DENSITY_TOKENS: &[&str] = &[
        "ldpi", "mdpi", "tvdpi", "hdpi", "xhdpi", "xxhdpi", "xxxhdpi", "nodpi", "anydpi",
    ];

    if DENSITY_TOKENS.iter().any(|token| lower.contains(token)) {
        return SplitType::Density;
    }

    let stem = Path::new(file_name_only(name))
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    let mut language_like = false;
    for token in stem
        .split(['.', '-', '_'])
        .filter(|segment| !segment.is_empty())
    {
        if is_language_token(token) {
            language_like = true;
            break;
        }
    }

    if language_like {
        return SplitType::Language;
    }

    SplitType::Unknown
}

fn is_language_token(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();

    if lower.len() == 2 && lower.chars().all(|c| c.is_ascii_lowercase()) {
        return true;
    }

    if lower.len() == 3 && lower.chars().all(|c| c.is_ascii_lowercase()) {
        return true;
    }

    if let Some((lang, region)) = lower.split_once('r')
        && (lang.len() == 2 || lang.len() == 3)
        && lang.chars().all(|c| c.is_ascii_lowercase())
        && region.len() == 2
        && region.chars().all(|c| c.is_ascii_lowercase())
    {
        return true;
    }

    false
}

fn file_name_only(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::{InputFormat, SplitType, classify_split_type, detect_format};
    use std::path::Path;

    #[test]
    fn detect_format_accepts_supported_extensions() {
        assert_eq!(
            detect_format(Path::new("bundle.apks")).unwrap(),
            InputFormat::Apks
        );
        assert_eq!(
            detect_format(Path::new("bundle.xapk")).unwrap(),
            InputFormat::Xapk
        );
        assert_eq!(
            detect_format(Path::new("bundle.apkm")).unwrap(),
            InputFormat::Apkm
        );
        assert_eq!(
            detect_format(Path::new("BUNDLE.APKS")).unwrap(),
            InputFormat::Apks
        );
    }

    #[test]
    fn detect_format_rejects_unsupported_extensions() {
        let error = detect_format(Path::new("bundle.apk")).unwrap_err();
        assert!(error.contains("unsupported input format"));
    }

    #[test]
    fn classify_split_type_detects_common_split_kinds() {
        assert_eq!(
            classify_split_type("split_config.arm64_v8a.apk"),
            SplitType::Abi
        );
        assert_eq!(classify_split_type("config.xxhdpi.apk"), SplitType::Density);
        assert_eq!(
            classify_split_type("split_config.en.apk"),
            SplitType::Language
        );
        assert_eq!(
            classify_split_type("nested/path/split_config.fr.apk"),
            SplitType::Language
        );
    }

    #[test]
    fn classify_split_type_prefers_density_when_multiple_tokens_match() {
        assert_eq!(
            classify_split_type("split_config.hdpi.en.apk"),
            SplitType::Density
        );
    }

    #[test]
    fn classify_split_type_falls_back_to_unknown() {
        assert_eq!(
            classify_split_type("feature_camera.apk"),
            SplitType::Unknown
        );
    }
}

#[cfg(test)]
mod safety_tests {
    use super::*;

    #[test]
    fn does_not_guess_a_base_from_multiple_unknown_apks() {
        let entries = vec![("a.apk".into(), vec![]), ("b.apk".into(), vec![])];
        assert_eq!(choose_base_index(&entries), None);
    }

    #[test]
    fn rejects_duplicate_base_candidates() {
        let entries = vec![
            ("variants/a/base.apk".into(), vec![]),
            ("variants/b/base.apk".into(), vec![]),
        ];
        assert_eq!(choose_base_index(&entries), None);
    }

    #[test]
    fn bounded_reader_rejects_oversized_content() {
        let mut source = std::io::Cursor::new(vec![1u8; 33]);
        assert!(read_limited(&mut source, 32, "nested.apk").is_err());
    }
}
