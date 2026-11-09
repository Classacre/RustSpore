//! Fixed explicit Cell_idle Mouth samples and provisional nonroot aggregate.
//! The source capture used a normalized sampler tag and supplied frame, duration
//! and no-wrap state. The pose index, blend, masks and zero-count snapshot are
//! supplied; no live actor, controller, selection or native aggregate join.
use sha2::{Digest, Sha256};
use spore_core::{
    anim,
    cell_runtime_contributions::{accumulate_scalar_components, CellRuntimeContributionContext, CellRuntimeScalarSlot},
    cell_runtime_curves::{sample_component, CellRuntimeCurveContext},
    cell_runtime_data::{CellRuntimeDataAccumulator, AGGREGATE_SIZE},
    native_sse::Policy,
    native_x87::{Context, Precision, Rounding},
};
use std::{env, error::Error, fs};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const CLIP: &str = "89decdc736685566a5678eecced5921b228347eeeb67efded03944fff7c4d09a";
const NATIVE: &str = "ac76fb6303ea832b8dfe917ff991be827d8f4dfef7a1c88d9bb8778370ee6c34";
const KEYS: [u32; 4] = [0x89e0_6a31, 0x114b_b90c, 0x1505_4351, 0xc229_9aa7];

fn pinned(bytes: &[u8], hash: &str) -> Result<()> {
    if format!("{:x}", Sha256::digest(bytes)) != hash { return Err("input SHA-256 mismatch".into()); }
    Ok(())
}

fn compare(clip: &[u8], native: &[u8]) -> Result<Vec<[u8; AGGREGATE_SIZE]>> {
    pinned(clip, CLIP)?;
    pinned(native, NATIVE)?;
    if native.len() != 16 * 476 { return Err("native output length".into()); }
    let mut animation = anim::parse(clip)?;
    if animation.duration.to_bits() != 74.0f32.to_bits() || animation.header_flags_280 & 8 == 0 {
        return Err("unexpected serialized duration or wrap".into());
    }
    // The host did not use the serialized header values for this fixed run.
    animation.duration = 73.0;
    animation.header_flags_280 &= !8;
    let mouth = animation.bones.get(1).ok_or("missing Mouth")?;
    if mouth.name != "Mouth" || mouth.keyframe_count != 3 || mouth.keyframe_stride != 180
        || mouth.keyframes.iter().map(|k| k.frame).collect::<Vec<_>>() != [0, 12, 73]
        || mouth.components.len() != 5 { return Err("unexpected Mouth structure".into()); }
    for (i, descriptor) in mouth.components[1..].iter().enumerate() {
        if descriptor.flags != 0x13 || descriptor.slot != i as u32 + 3
            || u32::from_le_bytes(descriptor.tag) != KEYS[i] { return Err("unexpected Mouth descriptor".into()); }
    }
    let x87 = Context { precision: Precision::Bits53, rounding: Rounding::NearestEven };
    let mut projections = Vec::new();
    for (frame_index, frame) in [0.0f32, 12.0, 42.5, 73.0].into_iter().enumerate() {
        let mut slots = [CellRuntimeScalarSlot { value_bits: 0xa5a5_a5a5, weight_bits: 0xa5a5_a5a5 }; 6];
        for component in 1..=4 {
            let sample = sample_component(&animation, 1, &mouth.components, component, frame,
                CellRuntimeCurveContext { x87, sse: Policy::Mxcsr1f80 })?.ok_or("empty sample")?;
            let slot = component - 1;
            let record = &native[(frame_index * 4 + slot) * 476..(frame_index * 4 + slot + 1) * 476];
            let off = 200 + slot * 40;
            if sample.value.to_bits() != u32::from_le_bytes(record[off..off+4].try_into()?)
                || sample.weight.to_bits() != u32::from_le_bytes(record[off+4..off+8].try_into()?) {
                return Err("source sample mismatch".into());
            }
            let cw=u16::from_le_bytes(record[468..470].try_into()?);
            let sw=u16::from_le_bytes(record[470..472].try_into()?);
            let mx=u32::from_le_bytes(record[472..476].try_into()?);
            if cw!=0x27f || sw&0x3840!=0 || mx&!0x3f!=0x1f80 { return Err("source FP metadata".into()); }
            slots[slot]=CellRuntimeScalarSlot { value_bits: sample.value.to_bits(), weight_bits: sample.weight.to_bits() };
        }
        let mut snapshot=[0xa5;AGGREGATE_SIZE];snapshot[..4].fill(0);
        let mut aggregate=CellRuntimeDataAccumulator::from_snapshot(snapshot)?;
        accumulate_scalar_components(&mut aggregate,&mouth.components,&slots,0x1e,0,0,1.0f32.to_bits(),
            CellRuntimeContributionContext { x87, sse: Policy::Mxcsr1f80 })?;
        projections.push(*aggregate.raw_bytes());
    }
    Ok(projections)
}

fn main() -> Result<()> {
    let a:Vec<_>=env::args_os().collect();
    if a.len()!=4 || a[1]!="--admit-synthetic-mouth-pose5" {
        return Err("usage: --admit-synthetic-mouth-pose5 <Cell_idle.anim> <native-stdout>".into());
    }
    let rows=compare(&fs::read(&a[2])?,&fs::read(&a[3])?)?;
    println!("matched 16 source sample pairs; {} supplied pose5 aggregate projections are provisional",rows.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unpinned_inputs_are_rejected() {
        assert!(compare(&[],&[]).is_err());
        assert!(pinned(&[0; 16],CLIP).is_err());
        assert!(pinned(&[0; 16],NATIVE).is_err());
    }
}
