//! Compare the complete bounded E55FB0 application with source-order CPU goldens.
use spore_core::cell_movement_application::{apply_movement_delta, CellMovementDelta};
use std::io::Read;

fn word(input: &mut impl Read) -> std::io::Result<u32> {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn vector(input: &mut impl Read) -> std::io::Result<[f32; 3]> {
    Ok([
        f32::from_bits(word(input)?),
        f32::from_bits(word(input)?),
        f32::from_bits(word(input)?),
    ])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_movement_application_verify <RSCELLA1 fixture>")?;
    #[cfg(target_arch = "x86_64")]
    {
        let mut mxcsr = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{address}]", address = in(reg) &mut mxcsr,
                options(nostack, preserves_flags));
        }
        if mxcsr & 0xffc0 != 0x1f80 {
            return Err(format!("unsupported MXCSR control {:08X}", mxcsr & 0xffc0).into());
        }
    }
    let mut input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut magic = [0; 8];
    input.read_exact(&mut magic)?;
    if &magic != b"RSCELLA1" {
        return Err("wrong movement application fixture magic".into());
    }
    let count = word(&mut input)?;
    if count == 0 || count > 1_000_000 {
        return Err("invalid application fixture count".into());
    }
    for ordinal in 0..count {
        let actor_word_248 = word(&mut input)?;
        let part_count = word(&mut input)? as i32;
        let scale = f32::from_bits(word(&mut input)?);
        let delta = vector(&mut input)?;
        let mut actor = vector(&mut input)?;
        let index_count = word(&mut input)?;
        if index_count > 1_000_000 {
            return Err("invalid application index count".into());
        }
        let indices = (0..index_count)
            .map(|_| word(&mut input).map(|value| value as i32))
            .collect::<std::io::Result<Vec<_>>>()?;
        let node_count = word(&mut input)?;
        if node_count > 1_000_000 {
            return Err("invalid application node count".into());
        }
        let mut nodes = (0..node_count)
            .map(|_| vector(&mut input))
            .collect::<std::io::Result<Vec<_>>>()?;
        let expected_actor = vector(&mut input)?.map(f32::to_bits);
        let expected_nodes = (0..node_count)
            .map(|_| vector(&mut input).map(|value| value.map(f32::to_bits)))
            .collect::<std::io::Result<Vec<_>>>()?;
        apply_movement_delta(
            CellMovementDelta {
                actor_word_248,
                part_count,
                part_indices: &indices,
                scale,
                delta,
            },
            &mut actor,
            &mut nodes,
        )?;
        let actual_nodes: Vec<_> = nodes
            .into_iter()
            .map(|value| value.map(f32::to_bits))
            .collect();
        if actor.map(f32::to_bits) != expected_actor || actual_nodes != expected_nodes {
            let node_difference = actual_nodes
                .iter()
                .zip(&expected_nodes)
                .enumerate()
                .find(|(_, (actual, expected))| actual != expected);
            return Err(format!(
                "movement application case {ordinal}: actor {:08X?} != {:08X?}; first node difference {:08X?}",
                actor.map(f32::to_bits), expected_actor, node_difference
            ).into());
        }
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing movement application fixture bytes".into());
    }
    println!(
        "CELL_MOVEMENT_APPLICATION_VERIFY cases={count} exact_bits=true mxcsr_control=00001F80"
    );
    Ok(())
}
