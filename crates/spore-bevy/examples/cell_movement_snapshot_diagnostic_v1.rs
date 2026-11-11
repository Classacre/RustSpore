//! Explicit snapshot projection of E55FB0; no live actor or producer is inferred.
use spore_core::cell_movement_application::{apply_movement_delta, CellMovementDelta};
use spore_core::native_sse::{self, Policy, SseError};
use std::{env, fs::File, io::Read};

const LIMIT: u64 = 1024 * 1024;
fn read_bounded(path: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("{path}: {e}"))?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{path}: {e}"))?;
    if bytes.len() as u64 > LIMIT {
        return Err(format!("{path}: exceeds 1 MiB limit"));
    }
    Ok(bytes)
}
fn word(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset.checked_add(4).ok_or("offset overflow")?;
    let b: [u8; 4] = bytes
        .get(offset..end)
        .ok_or_else(|| format!("missing word at +{offset:X}"))?
        .try_into()
        .unwrap();
    Ok(u32::from_le_bytes(b))
}
fn hex_word(s: &str) -> Result<u32, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 8 {
        return Err("delta words must have eight hexadecimal digits".into());
    }
    u32::from_str_radix(s, 16).map_err(|_| "invalid delta word".into())
}
#[derive(Debug)]
struct Projection {
    before: [u32; 3],
    after: [u32; 3],
    changed: Vec<(usize, [u32; 3], [u32; 3])>,
}
fn project(actor: &[u8], environment: &[u8], delta: [u32; 3]) -> Result<Projection, String> {
    if environment.len() % 12 != 0 {
        return Err("environment must contain complete 12-byte vectors".into());
    }
    let before = [word(actor, 0x90)?, word(actor, 0x94)?, word(actor, 0x98)?];
    let scale = word(actor, 0xbc)?;
    let enabled = word(actor, 0x248)?;
    let count = word(actor, 0x250)? as i32;
    let mut indices = Vec::new();
    if enabled != 0 && count >= 2 {
        let n = usize::try_from(count).map_err(|_| "invalid count")?;
        if n > (actor.len().saturating_sub(0x254)) / 4 {
            return Err("actor index array is truncated".into());
        }
        for i in 0..n {
            indices.push(word(actor, 0x254 + i * 4)? as i32);
        }
    }
    let mut parts = environment
        .chunks_exact(12)
        .map(|b| {
            [
                f32::from_bits(u32::from_le_bytes(b[0..4].try_into().unwrap())),
                f32::from_bits(u32::from_le_bytes(b[4..8].try_into().unwrap())),
                f32::from_bits(u32::from_le_bytes(b[8..12].try_into().unwrap())),
            ]
        })
        .collect::<Vec<_>>();
    let old = parts.clone();
    let mut value = before.map(f32::from_bits);
    let result = native_sse::scoped(Policy::Mxcsr1f80, || {
        Ok::<_, SseError>(apply_movement_delta(
            CellMovementDelta {
                actor_word_248: enabled,
                part_count: count,
                part_indices: &indices,
                scale: f32::from_bits(scale),
                delta: delta.map(f32::from_bits),
            },
            &mut value,
            &mut parts,
        ))
    })
    .map_err(|e| e.to_string())?;
    result.map_err(|e| e.to_string())?;
    let changed = old
        .iter()
        .zip(parts.iter())
        .enumerate()
        .filter_map(|(i, (a, b))| {
            let a = a.map(f32::to_bits);
            let b = b.map(f32::to_bits);
            (a != b).then_some((i, a, b))
        })
        .collect();
    Ok(Projection {
        before,
        after: value.map(f32::to_bits),
        changed,
    })
}
fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 5 {
        return Err("usage: cell_movement_snapshot_diagnostic_v1 ACTOR.bin ENV_VECTORS.bin DELTA_X_HEX DELTA_Y_HEX DELTA_Z_HEX".into());
    }
    let actor = read_bounded(&args[0])?;
    let environment = read_bounded(&args[1])?;
    let delta = [
        hex_word(&args[2])?,
        hex_word(&args[3])?,
        hex_word(&args[4])?,
    ];
    let p = project(&actor, &environment, delta)?;
    println!("E55FB0 supplied-snapshot projection; MXCSR=1F80; source identity, timing and producer unverified");
    println!("actor +90 before={:08X?} after={:08X?}", p.before, p.after);
    for (i, before, after) in p.changed {
        println!("environment vector {i}: before={before:08X?} after={after:08X?}");
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn set(buf: &mut [u8], offset: usize, v: u32) {
        buf[offset..offset + 4].copy_from_slice(&v.to_le_bytes());
    }
    #[test]
    fn projects_explicit_source_offsets_and_repeated_indices() {
        let mut a = vec![0; 0x25c];
        set(&mut a, 0x90, 1f32.to_bits());
        set(&mut a, 0xbc, 2f32.to_bits());
        set(&mut a, 0x248, 1);
        set(&mut a, 0x250, 2);
        set(&mut a, 0x254, 0);
        set(&mut a, 0x258, 0);
        let env = vec![0; 12];
        let p = project(&a, &env, [1f32.to_bits(), 0, 0]).unwrap();
        assert_eq!(p.after[0], 3f32.to_bits());
        assert_eq!(p.changed[0].2[0], 4f32.to_bits());
        assert_eq!(a[0x90..0x94], 1f32.to_le_bytes());
        assert_eq!(env, vec![0; 12]);
    }
    #[test]
    fn rejects_truncation_bad_indices_and_nonfinite_without_fabricating_results() {
        let mut a = vec![0; 0x258];
        set(&mut a, 0xbc, 1f32.to_bits());
        set(&mut a, 0x248, 1);
        set(&mut a, 0x250, 2);
        assert!(project(&a, &[0; 12], [0; 3])
            .unwrap_err()
            .contains("truncated"));
        a.extend_from_slice(&1u32.to_le_bytes());
        assert!(project(&a, &[0; 12], [0; 3]).is_err());
        set(&mut a, 0x248, 0);
        assert!(project(&a, &[0; 11], [0; 3]).is_err());
        assert!(project(&a, &[], [f32::NAN.to_bits(), 0, 0]).is_err());
    }
}
