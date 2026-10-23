//! Rust-only projection of the reviewed whole INFO+ROT100 native fixture.
use spore_core::anim::{Anim, AnimBone, AnimComponent, AnimKeyframe};
use spore_core::anim_curve::key_neighbors;
use spore_core::anim_quaternion::{ArithmeticContext, SsePolicy};
use spore_core::anim_rotation_curve::{sample_rotation, RotationBlendPolicy, RotationCurveError};
use spore_core::native_crt_acos::ConstantWindow;
use spore_core::native_x87::{Context, Extended};
use std::collections::BTreeMap;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: anim_rotation_curve_verify <verified-constants.bin> <inputs.bin> <new-output.bin>".into());
    }
    let bytes = std::fs::read(&args[0])?;
    let constants = ConstantWindow::new(&bytes)?;
    let input = std::fs::read(&args[1])?;
    if input.is_empty() || input.len() % 1952 != 0 {
        return Err("expected nonempty complete1952-byte input records".into());
    }
    let mut output = std::io::BufWriter::new(std::fs::File::create_new(&args[2])?);
    let mut counts = [0usize; 3];
    let mut errors = BTreeMap::<String, usize>::new();
    for (n, row) in input.chunks_exact(1952).enumerate() {
        let word = |at| u32::from_le_bytes(row[at..at + 4].try_into().unwrap());
        let frame = f32::from_bits(word(0));
        let count = word(4) as usize;
        if count > 16 || word(20) > 1 || word(24) > 1 || word(28) != 0 {
            return Err(format!("invalid request header at {n}").into());
        }
        let context = ArithmeticContext {
            x87: Context::from_control_word(word(16) as u16)?,
            sse: SsePolicy::Mxcsr1f80,
        };
        let raw = row[32..32 + count * 120].to_vec();
        let keyframes: Vec<_> = raw
            .chunks_exact(120)
            .map(|key| AnimKeyframe {
                frame: u32::from_le_bytes(key[..4].try_into().unwrap()),
                info: std::array::from_fn(|i| {
                    u32::from_le_bytes(key[4 + i * 4..8 + i * 4].try_into().unwrap())
                }),
                position: None,
                rotation: Some(std::array::from_fn(|i| {
                    f32::from_le_bytes(key[20 + i * 4..24 + i * 4].try_into().unwrap())
                })),
            })
            .collect();
        if keyframes.iter().any(|key| key.frame > 0x1000000)
            || keyframes
                .windows(2)
                .any(|keys| keys[0].frame > keys[1].frame)
        {
            return Err(format!("invalid fixture frame sequence at {n}").into());
        }
        let mut upper = 0;
        for key in &keyframes {
            if context.x87.compare(
                Extended::from_f32(frame)?,
                Extended::from_i32(key.frame as i32),
            )? == std::cmp::Ordering::Less
            {
                break;
            }
            upper += 1;
        }
        let neighbors = if count == 0 {
            [u32::MAX; 4]
        } else {
            key_neighbors(count, upper, word(12) & 8 != 0)?.map(|v| v as u32)
        };
        let animation = Anim {
            path: String::new(),
            hash: 0,
            header_flags_280: word(12),
            frame_duration: 1.,
            duration: f32::from_bits(word(8)),
            last_frame: 0,
            bone_count: 1,
            header_field_304: 0,
            anim_name: String::new(),
            bones: vec![AnimBone {
                name: String::new(),
                keyframe_count: count as u32,
                has_position: false,
                has_rotation: true,
                channel_flags: 0,
                selector_words: [0; 19],
                components: vec![
                    AnimComponent {
                        flags: 0,
                        tag: *b"INFO",
                        offset: 0,
                        stride: 120,
                        slot: 0,
                        reserved: [0; 3],
                    },
                    AnimComponent {
                        flags: if word(24) != 0 { 0x12 } else { 2 },
                        tag: *b"ROT\0",
                        offset: 20,
                        stride: 120,
                        slot: 2,
                        reserved: [0; 3],
                    },
                ],
                keyframe_stride: 120,
                keyframes,
                raw_keyframes: raw,
            }],
        };
        let mut packed = [0u32; 12];
        packed[6..10].copy_from_slice(&neighbors);
        match sample_rotation(
            &animation,
            0,
            &animation.bones[0].components,
            frame,
            context,
            if word(20) == 0 {
                RotationBlendPolicy::Spherical
            } else {
                RotationBlendPolicy::NormalizedHemisphere
            },
            constants,
        ) {
            Ok(None) => counts[0] += 1,
            Ok(Some(sample)) => {
                counts[1] += 1;
                packed[0] = 1;
                packed[1..5].copy_from_slice(&sample.rotation_xyzw.map(f32::to_bits));
                packed[5] = sample.weight.to_bits();
                packed[6..10].copy_from_slice(&sample.neighbors.map(|v| v as u32));
                packed[10] = sample.fraction.to_bits();
            }
            Err(error) => {
                counts[2] += 1;
                packed[0] = 2;
                packed[11] = match &error {
                    RotationCurveError::NonFiniteInput => 1,
                    RotationCurveError::NonFiniteIntermediate => 2,
                    RotationCurveError::Curve(_) => 3,
                    RotationCurveError::Quaternion(_) => 4,
                    _ => return Err(format!("unexpected structural error at {n}: {error}").into()),
                };
                *errors.entry(format!("{error:?}")).or_default() += 1;
            }
        }
        for value in packed {
            output.write_all(&value.to_le_bytes())?;
        }
    }
    output.flush()?;
    println!("rows={} absent={} sampled={} explicit_arithmetic_exclusions={} errors={errors:?} original_host_executed=false", input.len()/1952, counts[0], counts[1], counts[2]);
    Ok(())
}
