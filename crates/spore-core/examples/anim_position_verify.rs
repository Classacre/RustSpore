//! Compare Rust position sampling with bounded original INFO/POS x86 replies.
//! Requests and original replies stay in ignored target/. Quaternion, deform,
//! INFO side effects and the native exact-key return flag are outside this API.
use spore_core::{
    anim::{Anim, AnimBone, AnimComponent, AnimKeyframe},
    anim_curve::sample_position,
    native_x87::Context,
};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<_> = std::env::args_os().skip(1).collect();
    if paths.len() != 3 {
        return Err(
            "usage: anim_position_verify <requests1632.bin> <native124.bin> <new result TSV>"
                .into(),
        );
    }
    let request_file = std::fs::File::open(&paths[0])?;
    let reply_file = std::fs::File::open(&paths[1])?;
    let count = request_file.metadata()?.len() / 1632;
    if count == 0
        || count > 1_000_000
        || request_file.metadata()?.len() != count * 1632
        || reply_file.metadata()?.len() != count * 124
    {
        return Err("invalid request/reply stream lengths".into());
    }
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
    let mut requests = std::io::BufReader::new(request_file);
    let mut replies = std::io::BufReader::new(reply_file);
    let mut result = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&paths[2])?,
    );
    let mut contexts = BTreeSet::new();
    let mut finite = 0;
    let mut nonfinite = 0;
    for ordinal in 0..count {
        let mut request = [0; 1632];
        let mut reply = [0; 124];
        requests.read_exact(&mut request)?;
        replies.read_exact(&mut reply)?;
        let word = |at: usize| u32::from_le_bytes(request[at..at + 4].try_into().unwrap());
        let expected_word = |at: usize| u32::from_le_bytes(reply[at..at + 4].try_into().unwrap());
        let frame = f32::from_bits(word(0));
        let rows = word(8) as usize;
        let control = u16::from_le_bytes(request[20..22].try_into()?);
        let context = Context::from_control_word(control)?;
        if rows == 0 || rows > 16 || word(4) as usize > rows {
            return Err(format!("invalid request bounds at case{ordinal}").into());
        }
        let descriptors = vec![
            AnimComponent {
                flags: 0,
                tag: *b"INFO",
                offset: 0,
                stride: 100,
                slot: 0,
                reserved: [0; 3],
            },
            AnimComponent {
                flags: 0x11,
                tag: *b"POS\0",
                offset: 20,
                stride: 100,
                slot: 1,
                reserved: [0; 3],
            },
        ];
        let mut keys = Vec::with_capacity(rows);
        for index in 0..rows {
            keys.push(AnimKeyframe {
                frame: word(32 + index * 100),
                info: [0; 4],
                position: None,
                rotation: None,
            });
        }
        let animation = Anim {
            path: String::new(),
            hash: 0,
            header_flags_280: word(16),
            frame_duration: 1.,
            duration: f32::from_bits(word(12)),
            last_frame: 0,
            bone_count: 1,
            header_field_304: 0,
            anim_name: String::new(),
            bones: vec![AnimBone {
                name: String::new(),
                keyframe_count: rows as u32,
                has_position: true,
                has_rotation: false,
                channel_flags: 0,
                selector_words: [0; 19],
                components: descriptors.clone(),
                keyframe_stride: 100,
                keyframes: keys,
                raw_keyframes: request[32..32 + rows * 100].to_vec(),
            }],
        };
        let expected: [u32; 4] = std::array::from_fn(|i| expected_word(20 + 4 * i));
        let neighbors: [usize; 4] = std::array::from_fn(|i| expected_word(108 + 4 * i) as usize);
        let sample = sample_position(&animation, 0, &descriptors, frame, context);
        if expected.iter().any(|b| !f32::from_bits(*b).is_finite()) {
            if sample.is_ok() {
                return Err(
                    format!("case{ordinal}: nonfinite native result accepted: {sample:?}").into(),
                );
            }
            writeln!(result, "{ordinal}\t{control:04X}\tNONFINITE_REJECTED")?;
            nonfinite += 1;
        } else {
            let sample = sample
                .map_err(|e| format!("case{ordinal} CW{control:04X}: {e}"))?
                .ok_or("enabled POS absent")?;
            let actual = [
                sample.position[0],
                sample.position[1],
                sample.position[2],
                sample.weight,
            ]
            .map(f32::to_bits);
            if actual != expected || sample.neighbors != neighbors {
                return Err(format!("case{ordinal} CW{control:04X} frame{frame:?} upper{}: {:08X?}/{:?} != {:08X?}/{neighbors:?}", word(4), actual, sample.neighbors, expected).into());
            }
            writeln!(
                result,
                "{ordinal}\t{control:04X}\t{:08X}\t{:08X}\t{:08X}\t{:08X}\t{}\t{}\t{}\t{}",
                actual[0],
                actual[1],
                actual[2],
                actual[3],
                neighbors[0],
                neighbors[1],
                neighbors[2],
                neighbors[3]
            )?;
            finite += 1;
        }
        contexts.insert(control);
    }
    result.flush()?;
    println!("ANIM_POSITION_VERIFY cases={count} finite_exact={finite} nonfinite_rejected={nonfinite} explicit_contexts={} mxcsr=00001F80 native_active_context=UNKNOWN", contexts.len());
    Ok(())
}
