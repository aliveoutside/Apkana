use sha1::{Digest, Sha1};
use std::collections::HashSet;

/// Read defined class descriptors, rejecting malformed or overlapping DEX modules.
pub fn collect_classes(bytes: &[u8], classes: &mut HashSet<Vec<u8>>) -> Result<(), String> {
    let invalid = || "Merge: malformed or unsupported DEX".to_string();
    let word = |offset: usize| -> Result<usize, String> {
        Ok(u32::from_le_bytes(
            bytes
                .get(offset..offset + 4)
                .ok_or_else(invalid)?
                .try_into()
                .unwrap(),
        ) as usize)
    };
    if bytes.len() < 112
        || bytes.get(..4) != Some(b"dex\n")
        || word(32)? != bytes.len()
        || word(36)? != 112
        || word(40)? != 0x12345678
    {
        return Err(invalid());
    }
    if !matches!(
        &bytes[4..8],
        b"035\0" | b"037\0" | b"038\0" | b"039\0" | b"040\0"
    ) {
        return Err("Merge: unsupported DEX version".into());
    }
    if word(8)? as u32 != adler2::adler32_slice(&bytes[12..])
        || Sha1::digest(&bytes[32..]).as_slice() != &bytes[12..32]
    {
        return Err("Merge: DEX checksum/signature mismatch".into());
    }
    let (strings_count, strings) = (word(56)?, word(60)?);
    let (types_count, types) = (word(64)?, word(68)?);
    let (count, defs) = (word(96)?, word(100)?);
    if strings_count > bytes.len() / 4 || types_count > bytes.len() / 4 || count > bytes.len() / 32
    {
        return Err(invalid());
    }
    for i in 0..count {
        let ty = word(defs + i * 32)?;
        if ty >= types_count {
            return Err(invalid());
        }
        let string = word(types + ty * 4)?;
        if string >= strings_count {
            return Err(invalid());
        }
        let mut pos = word(strings + string * 4)?;
        // Skip the UTF-16 length's bounded ULEB128 prefix.
        let mut terminated = false;
        for _ in 0..5 {
            let b = *bytes.get(pos).ok_or_else(invalid)?;
            pos += 1;
            if b & 0x80 == 0 {
                terminated = true;
                break;
            }
        }
        if !terminated {
            return Err(invalid());
        }
        let tail = bytes.get(pos..).ok_or_else(invalid)?;
        let end = tail.iter().position(|b| *b == 0).ok_or_else(invalid)?;
        let descriptor = &tail[..end];
        if !descriptor.starts_with(b"L") || !descriptor.ends_with(b";") {
            return Err(invalid());
        }
        if !classes.insert(descriptor.to_vec()) {
            return Err(format!(
                "Merge: duplicate DEX class {}",
                String::from_utf8_lossy(descriptor)
            ));
        }
    }
    Ok(())
}
