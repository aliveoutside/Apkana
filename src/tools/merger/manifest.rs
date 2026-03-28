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
                &["splitTypes", "requiredSplitTypes"],
            );
            if removed > 0 {
                push_info(
                    output,
                    format!("Removed {removed} split attribute(s) from <manifest>"),
                );
            }
        }

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
        !names.iter().any(|target| *target == normalized)
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

    app_node.insert_attribute(
        Cow::Borrowed("extractNativeLibs"),
        ResValue::new_bool(true),
        string_pool,
        resource_map,
        None,
    );
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
