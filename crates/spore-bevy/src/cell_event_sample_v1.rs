//! Restricted explicit event sample selection; no runtime selection is implied.
use std::error::Error;
// Restricted A19CC0/A19520 walk: no high-bit parent key, no guessed
// separators, only the scalar and array types needed by this bridge.
pub fn select(bytes: &[u8], row: usize) -> Result<u32, Box<dyn Error>> {
    fn word(b: &[u8], p: usize) -> Result<u32, Box<dyn Error>> {
        Ok(u32::from_be_bytes(
            b.get(p..p.checked_add(4).ok_or("offset overflow")?)
                .ok_or("truncated word")?
                .try_into()?,
        ))
    }
    let declared = word(bytes, 0)?;
    if declared & 0x8000_0000 != 0 {
        return Err("high-bit parent path unsupported".into());
    }
    if declared > 100_000 {
        return Err("too many event properties".into());
    }
    let mut p = 4usize;
    let mut selected: Option<u32> = None;
    let mut seen_samples = false;
    for _ in 0..declared {
        let id = word(bytes, p)?;
        let end = p.checked_add(8).ok_or("offset overflow")?;
        let header = bytes.get(p..end).ok_or("truncated property header")?;
        let ty = u16::from_be_bytes([header[4], header[5]]);
        let flags = u16::from_be_bytes([header[6], header[7]]);
        p = end;
        if flags == 0 && matches!(ty, 0x0009 | 0x000a | 0x000d | 0x0020) {
            if id == 0x701ed91e {
                return Err("samples property is not an array".into());
            }
            // A19520's scalar default path consumes 16 bytes for type 0020;
            // the three numeric types use dedicated 4-byte reads.
            let n = if ty == 0x20 { 16 } else { 4 };
            p = p.checked_add(n).ok_or("offset overflow")?;
            if p > bytes.len() {
                return Err("truncated scalar".into());
            }
        } else if ty == 0x20 && matches!(flags, 0x9c | 0x1c) {
            let count = word(bytes, p)? as usize;
            let stride = word(bytes, p.checked_add(4).ok_or("offset overflow")?)? as usize;
            p = p.checked_add(8).ok_or("offset overflow")?;
            if count > 1_000_000 || stride != 12 {
                return Err("unsupported array count or stride".into());
            }
            let size = count.checked_mul(stride).ok_or("array size overflow")?;
            let end = p.checked_add(size).ok_or("array end overflow")?;
            let values = bytes.get(p..end).ok_or("truncated array")?;
            if id == 0x701ed91e {
                if seen_samples {
                    return Err("duplicate samples property".into());
                }
                seen_samples = true;
                let offset = row.checked_mul(12).ok_or("row overflow")?;
                if row >= count {
                    return Err("sample row out of range".into());
                }
                let value = u32::from_le_bytes(values[offset..offset + 4].try_into()?);
                if value == 0 {
                    return Err("zero sample triggers native fallback; unsupported".into());
                }
                selected = Some(value);
            }
            p = end;
        } else {
            return Err("unsupported native property type or flags".into());
        }
    }
    if p != bytes.len() {
        return Err("trailing bytes after declared properties".into());
    }
    selected.ok_or_else(|| "missing samples property".into())
}
