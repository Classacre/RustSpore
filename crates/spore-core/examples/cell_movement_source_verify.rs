//! Original-byte AI/style/base-speed getters, including full source definitions.
use spore_core::{
    cell_definition::{self, DEFINITION_SIZE},
    cell_movement_source::{
        creation_base_speeds, resolve_definition_movement_style, CellMovementSourceError,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

fn word(raw: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(raw[at..at + 4].try_into().unwrap())
}
fn current_mxcsr() -> u32 {
    let mut value = 0u32;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: bounded local on the SSE2 fixture target.
    unsafe {
        std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value,
            options(nostack, preserves_flags));
    }
    value
}
fn set_mxcsr(value: u32) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: exact saved state or explicitly defined masked control bits.
    unsafe {
        std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &value,
            options(nostack, preserves_flags));
    }
}
struct RestoreMxcsr(u32);
impl Drop for RestoreMxcsr {
    fn drop(&mut self) {
        set_mxcsr(self.0);
    }
}
fn current_x87_control() -> u16 {
    let mut value = 0u16;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: only read the same-thread control word into a bounded local.
    unsafe {
        std::arch::asm!("fnstcw [{value}]", value = in(reg) &mut value,
            options(nostack, preserves_flags));
    }
    value
}

fn modeled(request: &[u8; 832]) -> Result<[u8; 32], CellMovementSourceError> {
    let definition = cell_definition::parse(&request[32..828]).unwrap();
    let fc = word(request, 0);
    let difficulty = word(request, 12) as i32;
    let selected = definition.selected_ai_variant(difficulty);
    let style = resolve_definition_movement_style(
        fc,
        (fc != 0).then_some(definition),
        difficulty,
        Some(word(request, 16)),
    )?;
    let speeds = creation_base_speeds(fc, Some(f32::from_bits(word(request, 20))))?;
    let calls = if fc == 0 {
        0
    } else {
        1 + u32::from(definition.ai(selected).movement_style() == 0)
    };
    let mut raw = [0u8; 32];
    for (at, value) in [(0, style), (4, selected.source_offset() as u32), (8, calls)] {
        raw[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (index, value) in speeds.into_iter().enumerate() {
        let at = 12 + index * 4;
        raw[at..at + 4].copy_from_slice(&value.to_bits().to_le_bytes());
    }
    Ok(raw)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let nonfinite = args.get(3).is_some_and(|value| value == "--nonfinite");
    if args.len() != 3 && !(args.len() == 4 && nonfinite) {
        return Err("usage: cell_movement_source_verify <fixture-dir> <scalar-fixture-dir> <new modeled32.bin or policy.tsv> [--nonfinite]".into());
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !std::is_x86_feature_detected!("sse2") {
        return Err("getter fixture caller-state check requires SSE2".into());
    }
    let root = Path::new(&args[0]);
    let scalar = Path::new(&args[1]);
    let definitions_raw = std::fs::read(root.join("definitions.bin"))?;
    if definitions_raw.len() != 489 * DEFINITION_SIZE {
        return Err("getter source definition denominator differs".into());
    }
    let definitions: Vec<_> = definitions_raw
        .chunks_exact(DEFINITION_SIZE)
        .map(cell_definition::parse)
        .collect::<Result<_, _>>()?;
    let scalar_output = std::fs::read(scalar.join("output.bin"))?;
    if scalar_output.len() != 33480 * 240 {
        return Err("full scalar-cache evidence extent differs".into());
    }
    let prefix = if nonfinite { "nonfinite-" } else { "" };
    let inp = File::open(root.join(format!("{prefix}input.bin")))?;
    let native = File::open(root.join(format!("{prefix}output.bin")))?;
    let count = if nonfinite { 192 } else { 68340 };
    if inp.metadata()?.len() != count * 832 || native.metadata()?.len() != count * 40 {
        return Err("movement source/native fixture extents differ".into());
    }
    let metadata = std::fs::read_to_string(root.join(if nonfinite {
        "nonfinite-policy.tsv"
    } else {
        "cases.tsv"
    }))?;
    let mut rows = metadata.lines().skip(1);
    let mut inp = BufReader::new(inp);
    let mut native = BufReader::new(native);
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    if nonfinite {
        writeln!(output, "case\texpected_checked_policy\tactual_checked_policy\tnative_speed0_bits\tnative_x87_status")?;
    }
    let saved_mxcsr = current_mxcsr();
    let _restore = RestoreMxcsr(saved_mxcsr);
    let mut contexts = BTreeSet::new();
    let mut labels = BTreeMap::new();
    let mut source_definitions = BTreeSet::new();
    let mut source_cache_cases = BTreeSet::new();
    let mut policy_rejections = 0usize;
    for ordinal in 0..count as usize {
        let mut request = [0u8; 832];
        let mut expected = [0u8; 40];
        inp.read_exact(&mut request)?;
        native.read_exact(&mut expected)?;
        let row: Vec<_> = rows
            .next()
            .ok_or("missing getter metadata row")?
            .split('\t')
            .collect();
        if row.len() != 5 || row[0].parse::<usize>()? != ordinal {
            return Err(format!("getter metadata row {ordinal} is malformed").into());
        }
        let cw = u16::from_le_bytes(request[24..26].try_into()?);
        let actual_cw = u16::from_le_bytes(expected[28..30].try_into()?);
        let sw = u16::from_le_bytes(expected[30..32].try_into()?);
        if cw != actual_cw || sw & 0x3800 != 0 || word(&expected, 32) != 0x1f80 {
            return Err(format!("getter {ordinal} post-call FP metadata differs").into());
        }
        contexts.insert(cw);
        let mut projection = [0u8; 32];
        projection[..28].copy_from_slice(&expected[..28]);
        projection[28..].copy_from_slice(&expected[36..]);
        let mut result = None;
        for ambient in [0x1f80, 0xdf81, 0xbf84] {
            let prior_cw = current_x87_control();
            set_mxcsr(ambient);
            let value = modeled(&request);
            let state_unchanged = current_mxcsr() == ambient && current_x87_control() == prior_cw;
            set_mxcsr(saved_mxcsr);
            if !state_unchanged {
                return Err(format!("getter {ordinal} changed caller FP control/status").into());
            }
            if let Some(previous) = &result {
                if previous != &value {
                    return Err(format!("getter {ordinal} depends on ambient MXCSR").into());
                }
            }
            result = Some(value);
        }
        let result = result.unwrap();
        if nonfinite {
            if row[1].parse::<u32>()? != word(&request, 0)
                || u32::from_str_radix(row[2], 16)? != word(&request, 20)
                || u16::from_str_radix(row[3], 16)? != cw
            {
                return Err(format!("nonfinite getter {ordinal} request metadata differs").into());
            }
            let policy = match &result {
                Err(CellMovementSourceError::NonFiniteCachedSpeed) => {
                    policy_rejections += 1;
                    "NonFiniteCachedSpeed"
                }
                Ok(_) if word(&request, 0) == 0 => "OkDefault",
                _ => return Err(format!("unexpected getter policy in case {ordinal}").into()),
            };
            if policy != row[4] {
                return Err(format!("getter policy case {ordinal} differs").into());
            }
            if let Ok(actual) = &result {
                if actual != &projection {
                    return Err(format!("unused nonfinite getter cache {ordinal} differs").into());
                }
            }
            writeln!(
                output,
                "{ordinal}\t{}\t{policy}\t{:08X}\t{sw:04X}",
                row[4],
                word(&expected, 12)
            )?;
        } else {
            if u16::from_str_radix(row[4], 16)? != cw {
                return Err("source getter CW metadata differs".into());
            }
            *labels.entry(row[1].to_owned()).or_insert(0usize) += 1;
            let definition = row[2].parse::<i32>()?;
            let cache_case = row[3].parse::<i32>()?;
            match row[1] {
                "source_definition_with_explicit_cache_words" => {
                    let source = definitions
                        .get(definition as usize)
                        .ok_or("unknown source definition")?;
                    if source.raw().as_slice() != &request[32..828] || cache_case != -1 {
                        return Err(
                            format!("source definition {definition} request bytes differ").into(),
                        );
                    }
                    source_definitions.insert(definition);
                }
                "source_scalar_cache_with_explicit_zero_style_definition" => {
                    if !(0..5460).contains(&cache_case)
                        || definition != -1
                        || request[32..828] != [0; 796]
                    {
                        return Err("source cache getter definition/ordinal differs".into());
                    }
                    let at = cache_case as usize * 240 + (0x1254 - 0x1204);
                    if request[16..24] != scalar_output[at..at + 8] {
                        return Err(format!(
                            "source scalar-cache {cache_case} getter words differ"
                        )
                        .into());
                    }
                    source_cache_cases.insert(cache_case);
                }
                "synthetic_style_selection_inheritance_product"
                | "synthetic_cache_mask_bits"
                | "synthetic_finite_speed_copies_and_keys" => {
                    if definition != -1 || cache_case != -1 {
                        return Err("synthetic getter case claims source identity".into());
                    }
                }
                _ => return Err("unknown source getter cohort".into()),
            }
            let actual = result?;
            if actual != projection {
                let field = actual
                    .iter()
                    .zip(projection)
                    .position(|(a, b)| *a != b)
                    .unwrap();
                return Err(format!("getter case {ordinal} differs at modeled byte {field}: Rust {:02X}, native {:02X}", actual[field], projection[field]).into());
            }
            output.write_all(&actual)?;
        }
    }
    if rows.next().is_some() {
        return Err("unexpected extra getter metadata row".into());
    }
    if contexts.len() != 12 {
        return Err("expected all twelve explicit native x87 policies".into());
    }
    if nonfinite {
        if policy_rejections != 96 {
            return Err(
                "expected 96 reached nonfinite-cache errors and96 unused-cache defaults".into(),
            );
        }
    } else if source_definitions.len() != 489 || source_cache_cases.len() != 5460 {
        return Err("full source getter coverage differs".into());
    }
    output.flush()?;
    println!("CELL_MOVEMENT_SOURCE_VERIFY cases={count} definitions={} source_cache_cases={} contexts={} caller_states=3 rejected_nonfinite={policy_rejections} exact_bits=true labels={labels:?}", source_definitions.len(), source_cache_cases.len(), contexts.len());
    Ok(())
}
