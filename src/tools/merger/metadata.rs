//! Split identity comes from the binary manifest, never the archive filename.
use super::archive::SplitType;
use resand::res_value::ResValueType;
use resand::xmltree::{XMLTree, XMLTreeNode};
use std::collections::HashSet;
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub package: String,
    pub version: String,
    pub split: Option<String>,
    pub parent: Option<String>,
    pub dependencies: Vec<String>,
    pub feature: bool,
    pub kind: SplitType,
    pub required_types: Vec<String>,
    pub provided_types: Vec<String>,
    pub split_required: bool,
}

pub fn attr(tree: &XMLTree, node: &XMLTreeNode, name: &str) -> Option<String> {
    let a = node.get_attribute(name, &tree.string_pool)?;
    if let Some(s) = a.raw_value.resolve(&tree.string_pool) {
        return Some(s.to_owned());
    }
    Some(match a.typed_value.data {
        ResValueType::String(s) => s.resolve(&tree.string_pool)?.to_owned(),
        ResValueType::IntDec(n) | ResValueType::IntHex(n) => n.to_string(),
        ResValueType::IntBoolean(n) => bool::from(n).to_string(),
        _ => return None,
    })
}

pub fn inspect(data: &[u8]) -> Result<Metadata, String> {
    let mut zip =
        ZipArchive::new(Cursor::new(data)).map_err(|e| format!("Merge: invalid APK: {e}"))?;
    let mut bytes = Vec::new();
    zip.by_name("AndroidManifest.xml")
        .map_err(|_| "Merge: APK has no AndroidManifest.xml")?
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Merge: manifest exceeds 2 MiB limit".into());
    }
    let tree = XMLTree::try_from(bytes.as_slice())
        .map_err(|e| format!("Merge: invalid binary manifest: {e}"))?;
    let root = &tree.root;
    if root.element.name.resolve(&tree.string_pool) != Some("manifest") {
        return Err("Merge: expected manifest root".into());
    }
    if attr(&tree, root, "isolatedSplits").as_deref() == Some("true") {
        return Err("Merge: isolated split class loaders cannot be safely fused".into());
    }
    let package = attr(&tree, root, "package").ok_or("Merge: missing package name")?;
    let version = format!(
        "{}:{}",
        attr(&tree, root, "versionCodeMajor").unwrap_or_else(|| "0".into()),
        attr(&tree, root, "versionCode").ok_or("Merge: missing versionCode")?
    );
    let split = attr(&tree, root, "split").filter(|s| !s.is_empty());
    let feature = attr(&tree, root, "isFeatureSplit").as_deref() == Some("true");
    let parent = attr(&tree, root, "configForSplit").filter(|s| !s.is_empty());
    let dependencies = root
        .children
        .iter()
        .filter(|n| n.element.name.resolve(&tree.string_pool) == Some("uses-split"))
        .filter_map(|n| attr(&tree, n, "name"))
        .collect();
    let mut kind = if feature {
        SplitType::Feature
    } else {
        SplitType::Configuration
    };
    // ABI, DPI and locale target classification uses payload metadata. Qualifiers
    // remain in ARSC; multiple densities/locales can coexist in a universal APK.
    if (0..zip.len()).any(|i| zip.by_index(i).is_ok_and(|e| e.name().starts_with("lib/"))) {
        kind = SplitType::Abi;
    } else if let Ok(e) = zip.by_name("resources.arsc") {
        let mut arsc = Vec::new();
        e.take(256 * 1024 * 1024 + 1)
            .read_to_end(&mut arsc)
            .map_err(|e| e.to_string())?;
        if let Ok(table) = resand::table::ResTable::try_from(arsc.as_slice()) {
            for pkg in table.packages.as_packages() {
                for c in &pkg.chunks {
                    if let resand::defs::ResTypeValue::TableType(t) = &c.data {
                        if t.config.locale.unwrap_or(0) != 0 {
                            kind = SplitType::Language;
                        } else if t.config.screen_type.unwrap_or(0) >> 16 != 0 {
                            kind = SplitType::Density;
                        }
                    }
                }
            }
        }
    }
    if feature {
        kind = SplitType::Feature;
    }
    Ok(Metadata {
        package,
        version,
        split,
        parent,
        dependencies,
        feature,
        kind,
        required_types: attr(&tree, root, "requiredSplitTypes")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
        provided_types: attr(&tree, root, "splitTypes")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
        split_required: attr(&tree, root, "isSplitRequired").as_deref() == Some("true")
            || root.children.iter().any(|n| {
                n.element.name.resolve(&tree.string_pool) == Some("application")
                    && attr(&tree, n, "isSplitRequired").as_deref() == Some("true")
            }),
    })
}

pub fn validate_set(metadata: &[Metadata]) -> Result<usize, String> {
    let bases: Vec<_> = metadata
        .iter()
        .enumerate()
        .filter(|(_, m)| m.split.is_none())
        .collect();
    if bases.len() != 1 {
        return Err("Merge: archive contains multiple APK variants or no base; supply one compatible split set".into());
    }
    let (base_index, base) = bases[0];
    if metadata.len() == 1 && base.split_required {
        return Err("Merge: base requires splits but the archive contains none".into());
    }
    let provided: HashSet<_> = metadata
        .iter()
        .flat_map(|m| m.provided_types.iter())
        .collect();
    for m in metadata {
        for required in &m.required_types {
            if !provided.contains(required) {
                return Err(format!(
                    "Merge: archive is incomplete; required split type '{required}' is missing"
                ));
            }
        }
    }
    let mut ids = HashSet::new();
    let features: HashSet<_> = metadata
        .iter()
        .filter(|m| m.feature)
        .filter_map(|m| m.split.as_deref())
        .collect();
    for m in metadata {
        if m.package != base.package || m.version != base.version {
            return Err("Merge: APK modules have different package names or versions".into());
        }
        if let Some(id) = &m.split
            && !ids.insert(id)
        {
            return Err(format!(
                "Merge: duplicate split identity '{id}'; incompatible variants"
            ));
        }
        if let Some(parent) = &m.parent
            && parent != "base"
            && !features.contains(parent.as_str())
        {
            return Err(format!(
                "Merge: missing feature '{parent}' for configuration split"
            ));
        }
        for dep in &m.dependencies {
            if dep != "base" && !features.contains(dep.as_str()) {
                return Err(format!("Merge: missing required feature '{dep}'"));
            }
        }
    }
    Ok(base_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<Metadata> {
        let data = include_bytes!("../../../tests/fixtures/feature.apks");
        let mut zip = ZipArchive::new(Cursor::new(data)).unwrap();
        (0..zip.len())
            .map(|i| {
                let mut e = zip.by_index(i).unwrap();
                let mut bytes = Vec::new();
                e.read_to_end(&mut bytes).unwrap();
                inspect(&bytes).unwrap()
            })
            .collect()
    }
    #[test]
    fn rejects_package_and_version_mismatches() {
        let original = fixture();
        let mut metadata = original.clone();
        metadata[1].package = "another.package".into();
        assert!(
            validate_set(&metadata)
                .unwrap_err()
                .contains("different package")
        );
        metadata = original;
        metadata[1].version = "0:2".into();
        assert!(validate_set(&metadata).unwrap_err().contains("versions"));
    }
    #[test]
    fn rejects_duplicate_identity_and_missing_dependencies() {
        let mut metadata = fixture();
        metadata.push(metadata[1].clone());
        assert!(
            validate_set(&metadata)
                .unwrap_err()
                .contains("duplicate split identity")
        );
        metadata.pop();
        metadata[0].dependencies.push("missing-camera".into());
        assert!(
            validate_set(&metadata)
                .unwrap_err()
                .contains("missing required feature")
        );
    }
    #[test]
    fn refuses_missing_required_abi_even_with_other_splits_present() {
        let metadata: Vec<_> = fixture()
            .into_iter()
            .filter(|m| m.kind != SplitType::Abi)
            .collect();
        assert!(
            validate_set(&metadata)
                .unwrap_err()
                .contains("required split type 'base__abi'")
        );
    }
}
