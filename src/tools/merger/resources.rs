use std::borrow::Cow;
use std::io::Cursor;

use resand::defs::{ResChunk, ResTypeValue};
use resand::res_value::{ResValue, ResValueType};
use resand::string_pool::{ResStringPoolRef, StringPoolHandler};
use resand::table::{
    ResTable, ResTableConfig, ResTableEntry, ResTableEntryValue, ResTableMapEntry, ResTablePackage,
    ResTableResValueEntry, ResTableType, ResTableTypeSpec,
};

use crate::tools::{OutputStream, ToolOutput};

pub fn merge_resource_tables(
    base_arsc: &[u8],
    split_arsc_tables: &[Vec<u8>],
    output: &mut Vec<ToolOutput>,
) -> Result<Vec<u8>, String> {
    if split_arsc_tables.is_empty() {
        return Ok(base_arsc.to_vec());
    }

    let mut base_table = match parse_res_table(base_arsc) {
        Ok(table) => table,
        Err(error) => {
            push_warn(
                output,
                format!(
                    "resources.arsc merge skipped: failed to parse base table ({error}); keeping original base resources.arsc"
                ),
            );
            return Ok(base_arsc.to_vec());
        }
    };

    let mut merged_count = 0usize;

    for (index, split_bytes) in split_arsc_tables.iter().enumerate() {
        let split_table = match parse_res_table(split_bytes.as_slice()) {
            Ok(table) => table,
            Err(error) => {
                push_warn(
                    output,
                    format!(
                        "Skipping split resources.arsc #{}: parse failed ({error})",
                        index + 1
                    ),
                );
                continue;
            }
        };

        merge_single_table(&mut base_table, split_table, output)?;
        merged_count += 1;
        push_info(
            output,
            format!("Merged resources.arsc from split #{}", index + 1),
        );
    }

    if merged_count == 0 {
        push_warn(
            output,
            String::from(
                "No split resources.arsc tables were merged; keeping original base resources.arsc",
            ),
        );
        return Ok(base_arsc.to_vec());
    }

    match Vec::<u8>::try_from(base_table) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            push_warn(
                output,
                format!(
                    "Failed to serialize merged resources.arsc ({error}); keeping original base resources.arsc"
                ),
            );
            Ok(base_arsc.to_vec())
        }
    }
}

fn parse_res_table(bytes: &[u8]) -> Result<ResTable, String> {
    if let Ok(table) = ResTable::try_from(bytes) {
        return Ok(table);
    }

    let mut cursor = Cursor::new(bytes);
    ResTable::read_all(&mut cursor).map_err(|e| e.to_string())
}

fn merge_single_table(
    base_table: &mut ResTable,
    split_table: ResTable,
    output: &mut Vec<ToolOutput>,
) -> Result<(), String> {
    let mut package_index = 0usize;
    while let Some(split_pkg) = split_table.packages.get(package_index) {
        let Some(base_pkg_index) = find_package_index_by_id(base_table, split_pkg.id) else {
            push_warn(
                output,
                format!(
                    "resources.arsc: package id {} missing in base table; skipping",
                    split_pkg.id
                ),
            );
            package_index += 1;
            continue;
        };

        let base_pkg = base_table
            .packages
            .get_mut(base_pkg_index)
            .ok_or_else(|| String::from("Merge: invalid package index while merging ARSC"))?;

        merge_package(
            base_pkg,
            split_pkg.clone(),
            &split_table.string_pool,
            &mut base_table.string_pool,
        );
        package_index += 1;
    }

    Ok(())
}

fn find_package_index_by_id(table: &ResTable, id: u32) -> Option<usize> {
    let mut index = 0usize;
    while let Some(pkg) = table.packages.get(index) {
        if pkg.id == id {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn merge_package(
    base_pkg: &mut ResTablePackage,
    mut split_pkg: ResTablePackage,
    split_value_pool: &StringPoolHandler,
    base_value_pool: &mut StringPoolHandler,
) {
    copy_all_strings(&mut base_pkg.string_pool_key, &split_pkg.string_pool_key);
    copy_all_strings(&mut base_pkg.string_pool_type, &split_pkg.string_pool_type);

    for split_chunk in &mut split_pkg.chunks {
        remap_chunk(
            split_chunk,
            &split_pkg.string_pool_key,
            &mut base_pkg.string_pool_key,
            split_value_pool,
            base_value_pool,
        );
    }

    for split_chunk in split_pkg.chunks {
        if let ResTypeValue::TableType(split_type) = &split_chunk.data {
            if let Some(base_type) =
                find_table_type_mut(base_pkg, split_type.id, &split_type.config)
            {
                merge_table_type(base_type, split_type.clone());
                continue;
            }
        }

        if let ResTypeValue::TableSpec(split_spec) = &split_chunk.data {
            if let Some(base_spec) = find_table_spec_mut(base_pkg, split_spec.id) {
                merge_table_spec(base_spec, split_spec);
                continue;
            }
        }

        base_pkg.chunks.push(split_chunk);
    }
}

fn copy_all_strings(target: &mut StringPoolHandler, source: &StringPoolHandler) {
    for string in source.string_pool.get_strings() {
        target.allocate(Cow::Borrowed(string));
    }
}

fn remap_chunk(
    chunk: &mut ResChunk,
    split_key_pool: &StringPoolHandler,
    base_key_pool: &mut StringPoolHandler,
    split_value_pool: &StringPoolHandler,
    base_value_pool: &mut StringPoolHandler,
) {
    if let ResTypeValue::TableType(table_type) = &mut chunk.data {
        for (_, entry_opt) in &mut table_type.entries {
            let Some(entry) = entry_opt else {
                continue;
            };
            remap_entry(
                entry,
                split_key_pool,
                base_key_pool,
                split_value_pool,
                base_value_pool,
            );
        }
    }
}

fn remap_entry(
    entry: &mut ResTableEntry,
    split_key_pool: &StringPoolHandler,
    base_key_pool: &mut StringPoolHandler,
    split_value_pool: &StringPoolHandler,
    base_value_pool: &mut StringPoolHandler,
) {
    match &mut entry.data {
        ResTableEntryValue::ResValue(ResTableResValueEntry { data, .. }) => {
            remap_res_value(data, split_value_pool, base_value_pool)
        }
        ResTableEntryValue::Map(ResTableMapEntry { key, map, .. }) => {
            remap_key_ref(key, split_key_pool, base_key_pool);
            for table_map in map {
                remap_res_value(&mut table_map.value, split_value_pool, base_value_pool);
            }
        }
        ResTableEntryValue::Compact(_) => {}
    }
}

fn remap_key_ref(
    key_ref: &mut ResStringPoolRef,
    split_key_pool: &StringPoolHandler,
    base_key_pool: &mut StringPoolHandler,
) {
    if let Some(name) = key_ref.resolve(split_key_pool) {
        *key_ref = base_key_pool.allocate(Cow::Owned(name.to_string()));
    }
}

fn remap_res_value(
    value: &mut ResValue,
    split_value_pool: &StringPoolHandler,
    base_value_pool: &mut StringPoolHandler,
) {
    if let ResValueType::String(string_ref) = value.data
        && let Some(existing) = string_ref.resolve(split_value_pool)
    {
        value.data =
            ResValueType::String(base_value_pool.allocate(Cow::Owned(existing.to_string())));
    }
}

fn find_table_type_mut<'a>(
    package: &'a mut ResTablePackage,
    id: u8,
    config: &ResTableConfig,
) -> Option<&'a mut ResTableType> {
    for chunk in &mut package.chunks {
        if let ResTypeValue::TableType(table_type) = &mut chunk.data
            && table_type.id == id
            && table_type.config == *config
        {
            return Some(table_type);
        }
    }
    None
}

fn find_table_spec_mut<'a>(
    package: &'a mut ResTablePackage,
    id: u8,
) -> Option<&'a mut ResTableTypeSpec> {
    for chunk in &mut package.chunks {
        if let ResTypeValue::TableSpec(spec) = &mut chunk.data
            && spec.id == id
        {
            return Some(spec);
        }
    }

    None
}

fn merge_table_spec(base: &mut ResTableTypeSpec, split: &ResTableTypeSpec) {
    if split.types_count > base.types_count {
        base.types_count = split.types_count;
    }

    if base.config_masks.len() < split.config_masks.len() {
        base.config_masks.resize(split.config_masks.len(), 0);
    }

    for (index, mask) in split.config_masks.iter().enumerate() {
        if let Some(base_mask) = base.config_masks.get_mut(index) {
            *base_mask |= *mask;
        }
    }
}

fn merge_table_type(base: &mut ResTableType, split: ResTableType) {
    let sparse = base.flags.sparse();

    for (entry_index, split_entry) in split.entries {
        let Some(split_entry) = split_entry else {
            continue;
        };

        if let Some(base_slot) = base.entries.iter_mut().find(|(idx, _)| *idx == entry_index) {
            if base_slot.1.is_none() {
                base_slot.1 = Some(split_entry);
            }
            continue;
        }

        base.entries.push((entry_index, Some(split_entry)));
    }

    normalize_entries(&mut base.entries, sparse);
}

fn normalize_entries(entries: &mut Vec<(usize, Option<ResTableEntry>)>, sparse: bool) {
    if sparse {
        entries.sort_by_key(|(index, _)| *index);
        entries.dedup_by_key(|(index, _)| *index);
        return;
    }

    if entries.is_empty() {
        return;
    }

    let max_index = entries.iter().map(|(index, _)| *index).max().unwrap_or(0);
    let mut normalized = vec![(0usize, None); max_index + 1];

    for (index, slot) in normalized.iter_mut().enumerate() {
        slot.0 = index;
    }

    for (index, value) in entries.drain(..) {
        if let Some(slot) = normalized.get_mut(index) {
            slot.1 = value;
        }
    }

    *entries = normalized;
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
    use super::{merge_resource_tables, merge_table_spec};
    use resand::table::ResTableTypeSpec;
    use crate::tools::ToolOutput;

    #[test]
    fn keeps_base_resources_when_base_table_cannot_be_parsed() {
        let base = b"not-an-arsc".to_vec();
        let split = vec![1_u8, 2_u8, 3_u8];
        let mut output: Vec<ToolOutput> = Vec::new();

        let merged = merge_resource_tables(&base, &[split], &mut output)
            .expect("merge should gracefully fallback to base resources");

        assert_eq!(merged, base);
        assert!(
            output.iter().any(|line| {
                line.line
                    .contains("resources.arsc merge skipped: failed to parse base table")
            }),
            "expected fallback warning to be emitted"
        );
    }

    #[test]
    fn merge_table_spec_expands_and_combines_masks() {
        let mut base = ResTableTypeSpec {
            id: 8,
            types_count: 1,
            config_masks: vec![0x1, 0x2],
        };
        let split = ResTableTypeSpec {
            id: 8,
            types_count: 3,
            config_masks: vec![0x4, 0x8, 0x10],
        };

        merge_table_spec(&mut base, &split);

        assert_eq!(base.types_count, 3);
        assert_eq!(base.config_masks, vec![0x5, 0xA, 0x10]);
    }
}
