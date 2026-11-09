//! Bounded projection of `6A0550` array admission and `661560` loop.
//! The descriptor and records are caller-supplied, stable, readable snapshots;
//! callbacks return normally and do not mutate them. This does not resolve a
//! package, actual service, or a live Space-stage object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub admitted_array: bool,
    pub count: usize,
    pub visited_first_words: Vec<u32>,
}

/// Model only the array form: a successful +24 callback, type 0x20, and
/// flag 0x10. Supplied records bound all reads; source recursion is scripted.
pub fn project(callback_success: bool, type_word: u16, flags: u16,
               count: usize, records: &[[u32; 3]]) -> Result<Outcome, &'static str> {
    if !callback_success || type_word != 0x20 || flags & 0x10 == 0 {
        return Ok(Outcome { admitted_array: false, count: 0, visited_first_words: Vec::new() });
    }
    if count > i32::MAX as usize || count > records.len() {
        return Err("count exceeds admitted readable records");
    }
    Ok(Outcome { admitted_array: true, count,
        visited_first_words: records[..count].iter().map(|r| r[0]).collect() })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn array_order_and_failed_admissions() {
        let rows = [[4,9,8],[7,6,5]];
        assert_eq!(project(true,0x20,0x10,2,&rows).unwrap().visited_first_words,vec![4,7]);
        assert_eq!(project(true,0x20,0x10,0,&rows).unwrap().count,0);
        for (ok,typ,flags) in [(false,0x20,0x10),(true,0x21,0x10),(true,0x20,0)] {
            assert!(!project(ok,typ,flags,usize::MAX,&rows).unwrap().admitted_array);
        }
        assert!(project(true,0x20,0x10,3,&rows).is_err());
    }
}
