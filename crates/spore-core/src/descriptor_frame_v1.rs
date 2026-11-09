//! Bounded opaque parsing of the observed 693710 descriptor frame.
//! The class and sorted descriptor keys are explicitly supplied admissions;
//! this does not interpret callbacks, identify an archive or prove live state.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError { Truncated, Length, Count, Class, Keys }

#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a> { pub key: u32, pub bytes: &'a [u8] }

fn word(bytes: &[u8], at: usize) -> Result<u32, FrameError> {
    let b = bytes.get(at..at.checked_add(4).ok_or(FrameError::Length)?)
        .ok_or(FrameError::Truncated)?;
    Ok(u32::from_be_bytes(b.try_into().unwrap()))
}

/// Parse one complete stable frame under an explicitly supplied sorted key set.
/// `max_fields` and `max_bytes` are caller-selected resource limits.
pub fn parse<'a>(bytes: &'a [u8], class: u32, keys: &[u32], max_fields: usize, max_bytes: usize) -> Result<Vec<Field<'a>>, FrameError> {
    if bytes.len() > max_bytes { return Err(FrameError::Length); }
    if keys.len() > max_fields { return Err(FrameError::Count); }
    if keys.windows(2).any(|w| w[0] >= w[1]) { return Err(FrameError::Keys); }
    if word(bytes, 0)? != class { return Err(FrameError::Class); }
    let size = word(bytes, 4)? as usize;
    if size.checked_add(8) != Some(bytes.len()) { return Err(FrameError::Length); }
    let count = word(bytes, 8)? as usize;
    if count > max_fields || count != keys.len() { return Err(FrameError::Count); }
    let head = count.checked_mul(8).and_then(|n| n.checked_add(12)).ok_or(FrameError::Length)?;
    if head > bytes.len() { return Err(FrameError::Truncated); }
    let mut at = head;
    let mut fields = Vec::with_capacity(count);
    for (i, &expected) in keys.iter().enumerate() {
        let key = word(bytes, 12 + 8*i)?;
        if key != expected { return Err(FrameError::Keys); }
        let len = word(bytes, 16 + 8*i)? as usize;
        let end = at.checked_add(len).ok_or(FrameError::Length)?;
        let value = bytes.get(at..end).ok_or(FrameError::Truncated)?;
        fields.push(Field { key, bytes: value });
        at = end;
    }
    if at != bytes.len() { return Err(FrameError::Length); }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> Vec<u8> {
        let mut b = Vec::new();
        for x in [7u32, 24, 2, 3, 1, 5, 3] { b.extend(x.to_be_bytes()); }
        b.extend([9, 8, 7, 6]); b
    }
    #[test] fn accepts_opaque_fields() {
        let b=frame(); let f=parse(&b,7,&[3,5],2,32).unwrap();
        assert_eq!(f[0].bytes, &[9]); assert_eq!(f[1].bytes, &[8,7,6]);
    }
    #[test] fn rejects_malformed_and_unadmitted_frames() {
        let b=frame();
        assert_eq!(parse(&b,8,&[3,5],2,32),Err(FrameError::Class));
        assert_eq!(parse(&b,7,&[3,5],1,32),Err(FrameError::Count));
        assert_eq!(parse(&b,7,&[3,5],2,31),Err(FrameError::Length));
        assert_eq!(parse(&b,7,&[3,3],2,32),Err(FrameError::Keys));
        let mut x=b.clone(); x[19]=4;
        assert_eq!(parse(&x,7,&[3,5],2,32),Err(FrameError::Truncated));
        let mut x=b.clone(); x[11]=3;
        assert_eq!(parse(&x,7,&[3,5],3,32),Err(FrameError::Count));
        assert_eq!(parse(&b[..10],7,&[3,5],2,32),Err(FrameError::Length));
    }
}
