//! Compare retained80, committed32, selected source row and exact DIVSS bits.
use spore_core::{
    cell_movement_scaling::{
        size_movement_scale_details, CellMovementScalingContext, ORIGINAL_SCALE_TABLE_BITS,
    },
    native_sse::Policy,
    native_x87::Context,
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
    // SAFETY: bounded same-thread local on the checked SSE2 fixture target.
    unsafe {
        std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value,
            options(nostack, preserves_flags));
    }
    value
}
fn set_mxcsr(value: u32) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: exact saved state or defined masked control/status bits.
    unsafe {
        std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &value,
            options(nostack, preserves_flags));
    }
}
fn current_x87() -> (u16, u16) {
    let (mut control, mut status) = (0u16, 0u16);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: only read the same-thread control/status into bounded locals.
    unsafe {
        std::arch::asm!("fnstcw [{control}]", "fnstsw [{status}]",
            control = in(reg) &mut control, status = in(reg) &mut status,
            options(nostack, preserves_flags));
    }
    (control, status)
}
struct RestoreMxcsr(u32);
impl Drop for RestoreMxcsr {
    fn drop(&mut self) {
        set_mxcsr(self.0);
    }
}

fn modeled(request: &[u8; 112]) -> Result<[u8; 24], Box<dyn std::error::Error>> {
    let table: [f32; 20] =
        std::array::from_fn(|index| f32::from_bits(word(request, 16 + index * 4)));
    let actor = word(request, 0) as i32;
    let progress = word(request, 4) as i32;
    let control = u16::from_le_bytes(request[8..10].try_into()?);
    let x87 = Context::from_control_word(control)?;
    let result = size_movement_scale_details(
        actor,
        progress,
        (actor != -1).then_some(&table),
        CellMovementScalingContext {
            x87,
            sse: Policy::Mxcsr1f80,
        },
    )?;
    let mut output = [0u8; 24];
    output[..10].copy_from_slice(&result.retained.to_le_bytes());
    output[12..16].copy_from_slice(&x87.store_f32(result.retained)?.to_bits().to_le_bytes());
    output[16..20].copy_from_slice(
        &result
            .selected_scale_index
            .map_or(u32::MAX, |index| index as u32)
            .to_le_bytes(),
    );
    output[20..].copy_from_slice(&result.ratio_bits.unwrap_or(0).to_le_bytes());
    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let nonfinite = args.get(2).is_some_and(|value| value == "--nonfinite");
    if args.len() != 2 && !(args.len() == 3 && nonfinite) {
        return Err(
            "usage: cell_movement_scaling_verify <fixture-dir> <new modeled24.bin> [--nonfinite]"
                .into(),
        );
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !std::is_x86_feature_detected!("sse2") {
        return Err("original scaling fixture requires SSE2".into());
    }
    let root = Path::new(&args[0]);
    let original = std::fs::read(root.join("original-scale-table.bin"))?;
    let original_rust: Vec<_> = ORIGINAL_SCALE_TABLE_BITS
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    if original != original_rust {
        return Err(
            "typed original scale table words differ from independently pinned source bytes".into(),
        );
    }
    let prefix = if nonfinite { "nonfinite-" } else { "" };
    let inp = File::open(root.join(format!("{prefix}input.bin")))?;
    let native = File::open(root.join(format!("{prefix}output.bin")))?;
    let count = if nonfinite { 3864 } else { 261960 };
    if inp.metadata()?.len() != count * 112 || native.metadata()?.len() != count * 32 {
        return Err("scaling source/native fixture extents differ".into());
    }
    let metadata = std::fs::read_to_string(root.join(format!("{prefix}cases.tsv")))?;
    let mut rows = metadata.lines().skip(1);
    let mut inp = BufReader::new(inp);
    let mut native = BufReader::new(native);
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[1])?,
    );
    let saved_mxcsr = current_mxcsr();
    let _restore = RestoreMxcsr(saved_mxcsr);
    let mut contexts = BTreeSet::new();
    let mut labels = BTreeMap::new();
    let mut original_combinations = BTreeSet::new();
    let mut nonfinite_ratio_count = 0usize;
    for ordinal in 0..count as usize {
        let mut request = [0u8; 112];
        let mut expected = [0u8; 32];
        inp.read_exact(&mut request)?;
        native.read_exact(&mut expected)?;
        let row: Vec<_> = rows
            .next()
            .ok_or("missing scaling metadata row")?
            .split('\t')
            .collect();
        if row.len() != 3 || row[0].parse::<usize>()? != ordinal {
            return Err(format!("scaling metadata row {ordinal} is malformed").into());
        }
        let cw = u16::from_le_bytes(request[8..10].try_into()?);
        let native_cw = u16::from_le_bytes(expected[24..26].try_into()?);
        let native_sw = u16::from_le_bytes(expected[26..28].try_into()?);
        if native_cw != cw || native_sw & 0x3800 != 0 || word(&expected, 28) & !0x3f != 0x1f80 {
            return Err(format!("native scaling case {ordinal} changed FP controls/TOP").into());
        }
        if u16::from_str_radix(row[2], 16)? != cw || expected[10..12] != [0; 2] {
            return Err("scaling metadata CW or reserved output differs".into());
        }
        contexts.insert(cw);
        *labels.entry(row[1].to_owned()).or_insert(0usize) += 1;
        let actor = word(&request, 0) as i32;
        let progress = word(&request, 4) as i32;
        if row[1] == "original_table_all_actor_progress" {
            if request[16..96] != original
                || !(-1..20).contains(&actor)
                || !(0..=1000).contains(&progress)
            {
                return Err("original scaling case has synthetic table/actor/progress".into());
            }
            if !original_combinations.insert((cw, actor, progress)) {
                return Err("original scaling combination repeated".into());
            }
        } else if row[1] == "original_table_early_unused_progress" {
            if actor != -1 || request[16..96] != original {
                return Err("unused-progress source scaling case differs".into());
            }
        } else if !(row[1].starts_with("synthetic_")
            || nonfinite && row[1].starts_with("nonfinite_"))
        {
            return Err("unknown scaling cohort".into());
        }
        let mut result = None;
        for ambient in [0x1f80, 0xdf81, 0xbf84] {
            let prior_x87 = current_x87();
            set_mxcsr(ambient);
            let actual = modeled(&request)?;
            let state_unchanged = current_mxcsr() == ambient && current_x87() == prior_x87;
            set_mxcsr(saved_mxcsr);
            if !state_unchanged {
                return Err(
                    format!("scaling case {ordinal} changed complete caller FP state").into(),
                );
            }
            if actual != expected[..24] {
                let byte = actual
                    .iter()
                    .zip(&expected)
                    .position(|(a, b)| a != b)
                    .unwrap();
                return Err(format!("scaling case {ordinal} differs at modeled byte {byte}: Rust {:02X}, native {:02X}", actual[byte], expected[byte]).into());
            }
            if let Some(previous) = result {
                if previous != actual {
                    return Err("scaling result depends on incoming MXCSR".into());
                }
            }
            result = Some(actual);
        }
        let ratio = word(&expected, 20);
        if actor != -1 && ratio & 0x7f80_0000 == 0x7f80_0000 {
            nonfinite_ratio_count += 1;
        }
        output.write_all(&result.unwrap())?;
    }
    if rows.next().is_some() || contexts.len() != 12 {
        return Err("scaling metadata/context count differs".into());
    }
    if !nonfinite && original_combinations.len() != 252252 {
        return Err("full original scaling table/progress denominator differs".into());
    }
    output.flush()?;
    println!("CELL_MOVEMENT_SCALING_VERIFY cases={count} original_combinations={} contexts={} caller_states=3 retained80_exact=true committed32_exact=true nonfinite_ratios={nonfinite_ratio_count} labels={labels:?}", original_combinations.len(), contexts.len());
    Ok(())
}
