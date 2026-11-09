//! Checked single-subobject subset of 509B22..509C57.
//!
//! Faces are explicitly supplied ordinals into three equal corner streams.
//! Source caches by position only: a repeated position reuses its first triple,
//! even when later secondary or tertiary indices differ. No source subobject,
//! allocation, mutation, palette binding, or live identity is inferred.
use std::collections::BTreeMap;
use crate::creature_skin_corner_attributes_v1::{self, Attributes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { StreamLength, FaceOutOfRange, PositionOutOfRange, TooManyCorners, Attribute(creature_skin_corner_attributes_v1::Error) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triples { pub triples: Vec<[u32; 3]>, pub corners: Vec<u32> }

pub fn build(positions: &[u32], secondary: &[u32], tertiary: &[u32], faces: &[u32], position_count: usize) -> Result<Triples, Error> {
    if positions.len() != secondary.len() || positions.len() != tertiary.len() || positions.len() % 3 != 0 { return Err(Error::StreamLength); }
    let count = faces.len().checked_mul(3).ok_or(Error::TooManyCorners)?;
    if count > u32::MAX as usize { return Err(Error::TooManyCorners); }
    let mut cache = BTreeMap::new();
    let mut triples = Vec::new();
    let mut corners = Vec::with_capacity(count);
    for &face in faces {
        let first = (face as usize).checked_mul(3).ok_or(Error::FaceOutOfRange)?;
        if first.checked_add(3).filter(|&end| end <= positions.len()).is_none() { return Err(Error::FaceOutOfRange); }
        for i in first..first+3 {
            let position = positions[i];
            if position as usize >= position_count { return Err(Error::PositionOutOfRange); }
            let ordinal = if let Some(&n) = cache.get(&position) { n } else {
                let n = u32::try_from(triples.len()).map_err(|_| Error::TooManyCorners)?;
                triples.push([position, secondary[i], tertiary[i]]);
                cache.insert(position, n);
                n
            };
            corners.push(ordinal);
        }
    }
    Ok(Triples { triples, corners })
}

/// Project the first position index of each generated triple into supplied records.
pub fn project_records(triples: &Triples, records: &[[u8; 20]]) -> Result<Vec<Attributes>, Error> {
    let positions: Vec<u32> = triples.triples.iter().map(|t| t[0]).collect();
    creature_skin_corner_attributes_v1::join(records, &positions).map_err(Error::Attribute)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caches_first_triple_and_composes_records() {
        let result = build(&[0,1,0, 1,2,0], &[10,11,12,13,14,15], &[20,21,22,23,24,25], &[0,1], 3).unwrap();
        assert_eq!(result.triples, vec![[0,10,20],[1,11,21],[2,14,24]]);
        assert_eq!(result.corners, vec![0,1,0,1,2,0]);
        let mut records = [[0u8;20];3]; records[1][0] = 255;
        let attrs = project_records(&result,&records).unwrap();
        assert_eq!(attrs[1].offsets[0], 765);
        assert_eq!(project_records(&result,&records[..2]),Err(Error::Attribute(creature_skin_corner_attributes_v1::Error::IndexOutOfRange)));
        assert_eq!(build(&[0,1,0], &[1,2], &[3,4,5], &[0], 2),Err(Error::StreamLength));
        assert_eq!(build(&[0,1,0], &[1,2,3], &[3,4,5], &[1], 2),Err(Error::FaceOutOfRange));
        assert_eq!(build(&[0,1,0], &[1,2,3], &[3,4,5], &[0], 1),Err(Error::PositionOutOfRange));
    }
}
