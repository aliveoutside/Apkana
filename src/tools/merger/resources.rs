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

    let mut base_table = parse_res_table(base_arsc)
        .map_err(|e| format!("Merge: cannot safely merge base resources.arsc: {e}"))?;
    validate_table(&base_table)?;
    for (index, bytes) in split_arsc_tables.iter().enumerate() {
        let split = parse_res_table(bytes)
            .map_err(|e| format!("Merge: invalid split resources.arsc #{}: {e}", index + 1))?;
        validate_table(&split)?;
        merge_single_table(&mut base_table, split)?;
        push_info(
            output,
            format!("Merged resources.arsc from split #{}", index + 1),
        );
    }
    let bytes = Vec::<u8>::try_from(base_table)
        .map_err(|e| format!("Merge: cannot serialize resources.arsc without loss: {e}"))?;
    parse_res_table(&bytes)
        .map_err(|e| format!("Merge: serialized resources.arsc failed validation: {e}"))?;
    Ok(bytes)
}

fn validate_table(table: &ResTable) -> Result<(), String> {
    if !table.string_pool.string_pool.styles.is_empty() {
        return Err("Merge: styled ARSC string pools cannot yet be remapped without loss".into());
    }
    for pkg in table.packages.as_packages() {
        if pkg.id == 0 || pkg.id > 255 || pkg.type_id_offset > 255 {
            return Err("Merge: invalid ARSC package/type offset".into());
        }
        for chunk in &pkg.chunks {
            match &chunk.data {
                ResTypeValue::TableType(t) => {
                    if t.id == 0
                        || (ResStringPoolRef {
                            index: t.id as u32 - 1,
                        })
                        .resolve(&pkg.string_pool_type)
                        .is_none()
                    {
                        return Err("Merge: invalid ARSC type string reference".into());
                    }
                    let value = |v: &ResValue| -> Result<(), String> {
                        if let ResValueType::String(r) = v.data
                            && r.resolve(&table.string_pool).is_none()
                        {
                            return Err("Merge: invalid ARSC value string reference".into());
                        }
                        Ok(())
                    };
                    for (index, e) in &t.entries {
                        if *index > 65535 {
                            return Err("Merge: ARSC entry ID exceeds 16 bits".into());
                        }
                        if let Some(e) = e {
                            let key = match &e.data {
                                ResTableEntryValue::ResValue(v) => {
                                    value(&v.data)?;
                                    v.key
                                }
                                ResTableEntryValue::Map(v) => {
                                    for m in &v.map {
                                        value(&m.value)?;
                                    }
                                    v.key
                                }
                                ResTableEntryValue::Compact(_) => continue,
                            };
                            if key.resolve(&pkg.string_pool_key).is_none() {
                                return Err("Merge: invalid ARSC key string reference".into());
                            }
                        }
                    }
                    if t.entries.iter().any(|(_, e)| {
                        e.as_ref()
                            .is_some_and(|e| matches!(e.data, ResTableEntryValue::Compact(_)))
                    }) {
                        return Err("Merge: compact ARSC entries are unsupported; refusing to corrupt resources".into());
                    }
                }
                ResTypeValue::TableSpec(_) | ResTypeValue::TableLibrary(_) => {}
                _ => return Err(
                    "Merge: ARSC contains library/overlay/alias chunks that cannot safely be fused"
                        .into(),
                ),
            }
        }
    }
    Ok(())
}

fn parse_res_table(bytes: &[u8]) -> Result<ResTable, String> {
    if let Ok(table) = ResTable::try_from(bytes) {
        return Ok(table);
    }

    let mut cursor = Cursor::new(bytes);
    ResTable::read_all(&mut cursor).map_err(|e| e.to_string())
}

fn merge_single_table(base_table: &mut ResTable, split_table: ResTable) -> Result<(), String> {
    let mut package_index = 0usize;
    while let Some(split_pkg) = split_table.packages.get(package_index) {
        let Some(base_pkg_index) = find_package_index_by_id(base_table, split_pkg.id) else {
            let mut pkg = split_pkg.clone();
            let key_pool = pkg.string_pool_key.clone();
            for chunk in &mut pkg.chunks {
                remap_chunk(
                    chunk,
                    &key_pool,
                    &mut pkg.string_pool_key,
                    &split_table.string_pool,
                    &mut base_table.string_pool,
                );
            }
            base_table.packages.add_package(pkg);
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
        )?;
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
) -> Result<(), String> {
    if base_pkg.name != split_pkg.name {
        return Err("Merge: ARSC package ID collision".into());
    }
    if base_pkg.type_id_offset != 0 {
        return Err("Merge: base ARSC type ID offset is unsupported".into());
    }
    // Type IDs are positional and must never be allocated by string deduplication.
    for (i, name) in split_pkg
        .string_pool_type
        .string_pool
        .get_strings()
        .enumerate()
    {
        let used = split_pkg.chunks.iter().any(|c| matches!(&c.data, ResTypeValue::TableType(t) if t.id as usize == i + 1 && t.entries.iter().any(|(_, e)| e.is_some())));
        if !used {
            continue;
        }
        let index = i + split_pkg.type_id_offset as usize;
        let reference = ResStringPoolRef {
            index: index as u32,
        };
        if let Some(old) = reference.resolve(&base_pkg.string_pool_type)
            && !old.is_empty()
            && old != name
        {
            return Err(format!(
                "Merge: ARSC type ID {} has conflicting names: {old:?} vs {name:?}",
                index + 1
            ));
        }
        base_pkg.string_pool_type.write(name.to_owned(), reference);
    }
    for c in &mut split_pkg.chunks {
        let id = match &mut c.data {
            ResTypeValue::TableType(t) => &mut t.id,
            ResTypeValue::TableSpec(t) => &mut t.id,
            _ => continue,
        };
        *id = u8::try_from(*id as u32 + split_pkg.type_id_offset)
            .map_err(|_| "Merge: ARSC type ID overflow")?;
    }
    copy_all_strings(&mut base_pkg.string_pool_key, &split_pkg.string_pool_key);

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
        if let ResTypeValue::TableLibrary(lib) = &split_chunk.data
            && let Some(base_lib) = base_pkg.library_mut()
        {
            for entry in &lib.entries {
                if let Some(old) = base_lib
                    .entries
                    .iter()
                    .find(|e| e.package_id == entry.package_id)
                {
                    if old != entry {
                        return Err("Merge: conflicting ARSC dynamic package mappings".into());
                    }
                } else {
                    base_lib.entries.push(entry.clone());
                }
            }
            continue;
        }
        if let ResTypeValue::TableType(split_type) = &split_chunk.data
            && let Some(base_type) =
                find_table_type_mut(base_pkg, split_type.id, &split_type.config)
        {
            merge_table_type(base_type, split_type.clone())?;
            continue;
        }

        if let ResTypeValue::TableSpec(split_spec) = &split_chunk.data
            && let Some(base_spec) = find_table_spec_mut(base_pkg, split_spec.id)
        {
            merge_table_spec(base_spec, split_spec);
            continue;
        }

        base_pkg.chunks.push(split_chunk);
    }
    base_pkg.last_public_type = base_pkg
        .last_public_type
        .max(split_pkg.last_public_type + split_pkg.type_id_offset);
    base_pkg.last_public_key = base_pkg.string_pool_key.string_pool.get_strings().count() as u32;
    Ok(())
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
        ResTableEntryValue::ResValue(ResTableResValueEntry { key, data }) => {
            remap_key_ref(key, split_key_pool, base_key_pool);
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

fn find_table_spec_mut(package: &mut ResTablePackage, id: u8) -> Option<&mut ResTableTypeSpec> {
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

fn merge_table_type(base: &mut ResTableType, split: ResTableType) -> Result<(), String> {
    let sparse = base.flags.sparse();

    for (entry_index, split_entry) in split.entries {
        let Some(split_entry) = split_entry else {
            continue;
        };

        if let Some(base_slot) = base.entries.iter_mut().find(|(idx, _)| *idx == entry_index) {
            if let Some(old) = &base_slot.1
                && old != &split_entry
            {
                return Err(format!(
                    "Merge: conflicting ARSC resource at type {} entry {} for the same configuration",
                    base.id, entry_index
                ));
            }
            if base_slot.1.is_none() {
                base_slot.1 = Some(split_entry);
            }
            continue;
        }

        base.entries.push((entry_index, Some(split_entry)));
    }

    normalize_entries(&mut base.entries, sparse);
    Ok(())
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

/// Only strings referenced by live entries are paths; unused pool strings may
/// refer to resources excluded by bundletool and must not be treated as errors.
pub fn validate_file_references(
    bytes: &[u8],
    files: &std::collections::HashSet<String>,
) -> Result<(), String> {
    let table = parse_res_table(bytes).map_err(|e| format!("Merge: invalid output ARSC: {e}"))?;
    let check = |v: &ResValue| -> Result<(), String> {
        if let ResValueType::String(r) = v.data {
            let text = r
                .resolve(&table.string_pool)
                .ok_or("Merge: unresolved ARSC string reference")?;
            if text.starts_with("res/") && !files.contains(text) {
                return Err(format!("Merge: resource payload '{text}' is missing"));
            }
        }
        Ok(())
    };
    for pkg in table.packages.as_packages() {
        for c in &pkg.chunks {
            if let ResTypeValue::TableType(t) = &c.data {
                for (_, e) in &t.entries {
                    if let Some(e) = e {
                        match &e.data {
                            ResTableEntryValue::ResValue(v) => check(&v.data)?,
                            ResTableEntryValue::Map(v) => {
                                for m in &v.map {
                                    check(&m.value)?;
                                }
                            }
                            ResTableEntryValue::Compact(_) => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{merge_resource_tables, merge_table_spec};
    use crate::tools::ToolOutput;
    use resand::table::ResTableTypeSpec;

    #[test]
    fn refuses_lossy_resource_fallback() {
        assert!(
            merge_resource_tables(b"invalid", &[vec![1, 2]], &mut Vec::<ToolOutput>::new())
                .is_err()
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
