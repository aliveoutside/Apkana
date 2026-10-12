use std::borrow::Cow;

use resand::defs::ResourceMap;
use resand::res_value::{ResValue, ResValueType};
use resand::string_pool::StringPoolHandler;
use resand::xmltree::{ResXMLTreeAttribute, XMLTree, XMLTreeNode};

use crate::tools::{OutputStream, ToolOutput};

const SPLIT_META_NAMES: &[&str] = &[
    "com.android.vending.splits.required",
    "com.android.stamp.source",
    "com.android.stamp.type",
    "com.android.vending.splits",
    "com.android.vending.derived.apk.id",
];

pub fn sanitize_manifest(
    manifest_bytes: &[u8],
    has_abi_splits: bool,
    output: &mut Vec<ToolOutput>,
) -> Result<Vec<u8>, String> {
    let mut tree = XMLTree::try_from(manifest_bytes)
        .map_err(|e| format!("Merge: failed to parse AndroidManifest.xml: {e}"))?;

    {
        let XMLTree {
            string_pool,
            resource_map,
            root,
        } = &mut tree;

        let lookup_pool = string_pool.clone();

        if let Some(manifest_node) = root.get_element_mut(&["manifest"], &lookup_pool) {
            let removed = remove_attributes(
                manifest_node,
                &lookup_pool,
                &[
                    "split",
                    "configForSplit",
                    "isFeatureSplit",
                    "splitTypes",
                    "requiredSplitTypes",
                    "isSplitRequired",
                ],
            );
            if removed > 0 {
                push_info(
                    output,
                    format!("Removed {removed} split attribute(s) from <manifest>"),
                );
            }
        }

        root.children
            .retain(|n| n.element.name.resolve(&lookup_pool) != Some("uses-split"));

        if let Some(app_node) = root.get_element_mut(&["manifest", "application"], &lookup_pool) {
            let removed = remove_attributes(app_node, &lookup_pool, &["isSplitRequired"]);
            if removed > 0 {
                push_info(
                    output,
                    format!("Removed {removed} split attribute(s) from <application>"),
                );
            }

            let removed_meta = remove_split_metadata_nodes(app_node, &lookup_pool);
            if removed_meta > 0 {
                push_info(
                    output,
                    format!("Removed {removed_meta} split metadata node(s) from manifest"),
                );
            }

            if has_abi_splits
                && set_or_insert_extract_native_libs(
                    app_node,
                    &lookup_pool,
                    string_pool,
                    resource_map.as_mut(),
                )
            {
                push_info(
                    output,
                    String::from("Set android:extractNativeLibs=true on <application>"),
                );
            }
        }
    }

    Vec::<u8>::try_from(tree)
        .map_err(|e| format!("Merge: failed to serialize AndroidManifest.xml: {e}"))
}

fn remove_attributes(node: &mut XMLTreeNode, strings: &StringPoolHandler, names: &[&str]) -> usize {
    let before = node.element.attributes.len();
    node.element.attributes.retain(|attr| {
        let Some(name) = attr.name.resolve(strings) else {
            return true;
        };

        let normalized = normalize_attr_name(name);
        !names.contains(&normalized)
    });
    before.saturating_sub(node.element.attributes.len())
}

fn remove_split_metadata_nodes(app_node: &mut XMLTreeNode, strings: &StringPoolHandler) -> usize {
    let before = app_node.children.len();

    app_node.children.retain(|child| {
        if child.element.name.resolve(strings) != Some("meta-data") {
            return true;
        }

        let Some(name_attr) = child
            .get_attribute("name", strings)
            .or_else(|| child.get_attribute("android:name", strings))
        else {
            return true;
        };

        let Some(meta_name) = attribute_string_value(name_attr, strings) else {
            return true;
        };

        !SPLIT_META_NAMES.contains(&meta_name.as_str())
    });

    before.saturating_sub(app_node.children.len())
}

fn attribute_string_value(
    attr: &ResXMLTreeAttribute,
    strings: &StringPoolHandler,
) -> Option<String> {
    if let Some(value) = attr.raw_value.resolve(strings) {
        return Some(value.to_string());
    }

    match attr.typed_value.data {
        ResValueType::String(reference) => reference.resolve(strings).map(|v| v.to_string()),
        _ => None,
    }
}

fn set_or_insert_extract_native_libs(
    app_node: &mut XMLTreeNode,
    lookup_pool: &StringPoolHandler,
    string_pool: &mut StringPoolHandler,
    resource_map: Option<&mut ResourceMap>,
) -> bool {
    if let Some(attr) = app_node.get_attribute_mut("extractNativeLibs", lookup_pool) {
        attr.write_bool(true);
        return true;
    }

    if let Some(attr) = app_node.get_attribute_mut("android:extractNativeLibs", lookup_pool) {
        attr.write_bool(true);
        return true;
    }

    let attr = app_node.insert_attribute(
        Cow::Borrowed("extractNativeLibs"),
        ResValue::new_bool(true),
        string_pool,
        resource_map,
        Some(resand::defs::ResTableRef::new(0x010104ea)),
    );
    attr.ns = string_pool.allocate(Cow::Borrowed("http://schemas.android.com/apk/res/android"));
    true
}

fn normalize_attr_name(name: &str) -> &str {
    name.strip_prefix("android:").unwrap_or(name)
}

fn push_info(output: &mut Vec<ToolOutput>, line: impl Into<String>) {
    output.push(ToolOutput {
        line: line.into(),
        stream: OutputStream::Stdout,
    });
}

/// Import feature declarations with their string pool and Android attribute IDs.
pub fn merge_manifests(
    base: &[u8],
    splits: &[Vec<u8>],
    native: bool,
    has_code: bool,
    output: &mut Vec<ToolOutput>,
) -> Result<Vec<u8>, String> {
    let mut tree =
        XMLTree::try_from(base).map_err(|e| format!("Merge: invalid base manifest: {e}"))?;
    for bytes in splits {
        let source = XMLTree::try_from(bytes.as_slice())
            .map_err(|e| format!("Merge: invalid split manifest: {e}"))?;
        for child in &source.root.children {
            let tag = child
                .element
                .name
                .resolve(&source.string_pool)
                .ok_or("Merge: invalid manifest tag reference")?;
            if tag == "uses-split" {
                continue;
            }
            if tag == "uses-sdk" {
                let base_sdk = tree
                    .root
                    .children
                    .iter()
                    .find(|n| n.element.name.resolve(&tree.string_pool) == Some("uses-sdk"));
                for name in ["minSdkVersion", "targetSdkVersion", "maxSdkVersion"] {
                    let split_value = super::metadata::attr(&source, child, name);
                    if split_value.is_some()
                        && split_value
                            != base_sdk.and_then(|n| super::metadata::attr(&tree, n, name))
                    {
                        return Err(format!(
                            "Merge: feature has incompatible SDK constraint '{name}'"
                        ));
                    }
                }
                continue;
            }
            if tag == "application" {
                let mut imported = child.clone();
                remap_node(&mut imported, &source, &mut tree)?;
                let pool = tree.string_pool.clone();
                let target = tree
                    .root
                    .get_element_mut(&["manifest", "application"], &pool)
                    .ok_or("Merge: base has no application element")?;
                // Configuration-only attributes do not change application semantics.
                for attr in &imported.element.attributes {
                    let name = attr
                        .name
                        .resolve(&pool)
                        .ok_or("Merge: invalid application attribute")?;
                    if matches!(name, "hasCode" | "splitName" | "isSplitRequired") {
                        continue;
                    }
                    let existing = target
                        .element
                        .attributes
                        .iter()
                        .find(|a| a.name == attr.name && a.ns == attr.ns);
                    if existing != Some(attr) {
                        return Err(format!(
                            "Merge: split changes application attribute '{name}'; cannot fuse safely"
                        ));
                    }
                }
                for n in imported.children {
                    append_unique(&mut target.children, n, &pool)?;
                }
            } else {
                if !matches!(
                    tag,
                    "uses-permission"
                        | "uses-permission-sdk-23"
                        | "permission"
                        | "permission-group"
                        | "permission-tree"
                        | "uses-feature"
                        | "queries"
                ) {
                    return Err(format!("Merge: unsupported split manifest element <{tag}>"));
                }
                let mut imported = child.clone();
                remap_node(&mut imported, &source, &mut tree)?;
                append_unique(&mut tree.root.children, imported, &tree.string_pool)?;
            }
        }
    }
    if has_code {
        let pool = tree.string_pool.clone();
        if let Some(app) = tree
            .root
            .get_element_mut(&["manifest", "application"], &pool)
            && let Some(a) = app.get_attribute_mut("hasCode", &pool)
        {
            a.write_bool(true);
        }
    }
    let bytes = Vec::<u8>::try_from(tree)
        .map_err(|e| format!("Merge: cannot serialize merged manifest: {e}"))?;
    sanitize_manifest(&bytes, native, output)
}

fn append_unique(
    nodes: &mut Vec<XMLTreeNode>,
    mut node: XMLTreeNode,
    pool: &StringPoolHandler,
) -> Result<(), String> {
    // Remove split ownership now that the component lives in the base APK.
    remove_attributes(&mut node, pool, &["splitName"]);
    let name = node
        .get_attribute("name", pool)
        .and_then(|a| attribute_string_value(a, pool));
    // Unnamed nodes (especially intent-filters) are independent declarations.
    // Combining their actions/categories would change intent matching semantics.
    if name.is_none() {
        if !nodes.contains(&node) {
            nodes.push(node);
        }
        return Ok(());
    }
    let tag = node.element.name;
    if let Some(old) = nodes.iter_mut().find(|n| {
        n.element.name == tag
            && n.get_attribute("name", pool)
                .and_then(|a| attribute_string_value(a, pool))
                == name
    }) {
        if node.element.attributes != old.element.attributes {
            return Err(format!(
                "Merge: conflicting manifest declaration '{}'",
                name.unwrap_or_default()
            ));
        }
        for child in node.children {
            append_unique(&mut old.children, child, pool)?;
        }
    } else {
        nodes.push(node);
    }
    Ok(())
}

fn remap_node(
    node: &mut XMLTreeNode,
    source: &XMLTree,
    target: &mut XMLTree,
) -> Result<(), String> {
    fn reference(
        r: &mut resand::string_pool::ResStringPoolRef,
        source: &XMLTree,
        target: &mut XMLTree,
    ) -> Result<(), String> {
        if r.index == u32::MAX {
            return Ok(());
        }
        let text = r
            .resolve(&source.string_pool)
            .ok_or("Merge: invalid manifest string reference")?
            .to_owned();
        *r = target.string_pool.allocate(Cow::Owned(text));
        Ok(())
    }
    reference(&mut node.element.name, source, target)?;
    reference(&mut node.element.ns, source, target)?;
    reference(&mut node.element.node.comment, source, target)?;
    if let Some(ns) = &mut node.namespace {
        reference(&mut ns.prefix, source, target)?;
        reference(&mut ns.uri, source, target)?;
    }
    for attr in &mut node.element.attributes {
        let resource = source
            .resource_map
            .as_ref()
            .and_then(|m| m.get(attr.name))
            .copied();
        reference(&mut attr.name, source, target)?;
        reference(&mut attr.ns, source, target)?;
        reference(&mut attr.raw_value, source, target)?;
        if let ResValueType::String(r) = &mut attr.typed_value.data {
            reference(r, source, target)?;
        }
        if let Some(id) = resource {
            let map = target.resource_map.get_or_insert_with(|| ResourceMap {
                mapping: Vec::new(),
            });
            if map
                .get(attr.name)
                .is_some_and(|old| !old.is_null() && *old != id)
            {
                return Err("Merge: conflicting Android manifest attribute IDs".into());
            }
            map.insert(attr.name, id);
        }
    }
    for child in &mut node.children {
        remap_node(child, source, target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_intent_filters_are_never_combined() {
        let mut pool = StringPoolHandler::new_empty();
        let tag = pool.allocate(Cow::Borrowed("intent-filter"));
        let make = |action: &str, pool: &mut StringPoolHandler| {
            let mut filter = XMLTreeNode::new();
            filter.element.name = tag;
            let mut child = XMLTreeNode::new();
            child.element.name = pool.allocate(Cow::Borrowed(action));
            filter.children.push(child);
            filter
        };
        let first = make("first-action", &mut pool);
        let second = make("second-action", &mut pool);
        let mut nodes = vec![first];
        append_unique(&mut nodes, second, &pool).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[1].children.len(), 1);
    }
}
