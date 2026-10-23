//! Compare all modeled scalar-cache bytes and query traces; validate FP metadata.
use spore_core::{
    cell_capability_curve::{sample_capability_curve, CellCapabilityCurveError},
    cell_capability_scalar::{
        derive_cell_capability_scalars, CellCapabilityScalarContext, CellCapabilityScalarError,
        CellCapabilityScalarGlobals, CellCapabilityScalars,
    },
    cell_globals,
    native_sse::Policy,
    native_x87::Context,
    prop::{self, PropValue},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

const IDS: [u32; 9] = [
    0x7754_0c34,
    0x2c57_4746,
    0xbbab_1bf5,
    0x2ef6_8471,
    0xe501_5d0c,
    0x5a38_c4ba,
    0x03ab_589e,
    0xaee2_835e,
    0x92ba_ae19,
];
fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
fn float(data: &[u8], at: usize) -> f32 {
    f32::from_bits(word(data, at))
}
fn current_mxcsr() -> u32 {
    let mut value = 0u32;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: bounded local on the original SSE2 fixture target.
    unsafe {
        std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value,
            options(nostack, preserves_flags));
    }
    value
}
fn set_mxcsr(value: u32) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: callers use either exact saved state or defined masked controls.
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

fn modeled(request: &[u8; 1488], value: &CellCapabilityScalars) -> [u8; 232] {
    let mut complete = [0xa5u8; 240];
    complete[..140].copy_from_slice(&request[32..172]);
    value.write_native_tail((&mut complete[..140]).try_into().unwrap());
    let calls = value.queries().len() as u32;
    let counts: [u32; 13] = std::array::from_fn(|index| word(request, 32 + index * 4));
    let globals = if [8, 9, 10].into_iter().any(|class| counts[class] != 0) {
        1
    } else {
        2
    };
    let properties = if word(request, 28) == 0 { 0 } else { calls };
    for (offset, number) in [
        (140, calls),
        (144, globals),
        (148, globals),
        (216, globals),
        (220, calls),
        (224, properties),
        (228, properties),
        (232, 0),
        (236, 0),
    ] {
        complete[offset..offset + 4].copy_from_slice(&number.to_le_bytes());
    }
    for (index, query) in value.queries().iter().enumerate() {
        let at = 152 + index * 8;
        complete[at..at + 4].copy_from_slice(&query.identifier.to_le_bytes());
        complete[at + 4..at + 8].copy_from_slice(&query.value.to_bits().to_le_bytes());
    }
    let mut result = [0u8; 232];
    result[..208].copy_from_slice(&complete[..208]);
    result[208..].copy_from_slice(&complete[216..]);
    result
}

fn error_label(error: CellCapabilityScalarError) -> &'static str {
    match error {
        CellCapabilityScalarError::NonFiniteInput => "NonFiniteInput",
        CellCapabilityScalarError::NonFiniteIntermediate => "NonFiniteIntermediate",
        CellCapabilityScalarError::IntegerConversionOutsideFiniteDomain => {
            "IntegerConversionOutsideFiniteDomain"
        }
        CellCapabilityScalarError::Curve(CellCapabilityCurveError::NonFiniteInput) => {
            "CurveNonFiniteInput"
        }
        CellCapabilityScalarError::Curve(CellCapabilityCurveError::NonFiniteIntermediate) => {
            "CurveNonFiniteIntermediate"
        }
        _ => "UnexpectedError",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let nonfinite = args.get(3).is_some_and(|value| value == "--nonfinite");
    if args.len() != 3 && !(args.len() == 4 && nonfinite) {
        return Err("usage: cell_capability_scalar_verify <fixture-dir> <composition-dir> <new modeled232.bin or policy.tsv> [--nonfinite]".into());
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !std::is_x86_feature_detected!("sse2") {
        return Err("scalar fixture requires SSE2".into());
    }
    let _restore = RestoreMxcsr(current_mxcsr());
    let root = Path::new(&args[0]);
    let composition = Path::new(&args[1]);
    let prefix = if nonfinite { "nonfinite-" } else { "" };
    let input = File::open(root.join(format!("{prefix}input.bin")))?;
    let native = File::open(root.join(format!("{prefix}output.bin")))?;
    let extent = input.metadata()?.len();
    if extent % 1488 != 0 || native.metadata()?.len() != extent / 1488 * 240 {
        return Err("scalar native fixture extents differ".into());
    }
    let cases = extent / 1488;
    if cases != if nonfinite { 8 } else { 33480 } {
        return Err("scalar source/synthetic policy denominators differ".into());
    }
    let mut input = BufReader::new(input);
    let mut native = BufReader::new(native);
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    let mut contexts = BTreeSet::new();
    let policy: Vec<_> = std::fs::read_to_string(root.join("nonfinite-policy.tsv"))?
        .lines()
        .skip(1)
        .map(|line| line.split('\t').nth(1).unwrap().to_owned())
        .collect();
    if policy.len() != 8 {
        return Err("expected eight explicit checked-API policy rows".into());
    }
    if nonfinite {
        writeln!(
            output,
            "case\texpected_error\tactual_error\tnative_CVTSS2SI_word"
        )?;
    }
    for ordinal in 0..cases as usize {
        let mut request = [0u8; 1488];
        let mut expected = [0u8; 240];
        input.read_exact(&mut request)?;
        native.read_exact(&mut expected)?;
        let control = u16::from_le_bytes(request[..2].try_into()?);
        contexts.insert(control);
        let actual_cw = u16::from_le_bytes(expected[212..214].try_into()?);
        let sw = u16::from_le_bytes(expected[214..216].try_into()?);
        if actual_cw != control || sw & 0x3800 != 0 || word(&expected, 208) & !0x3f != 0x1f80 {
            return Err(format!("native floating context differs at {ordinal}").into());
        }
        let counts = std::array::from_fn(|index| word(&request, 32 + index * 4));
        let globals = CellCapabilityScalarGlobals {
            cilia_speed_as_jet: float(&request, 8),
            flagella_speed_as_jet: float(&request, 12),
            flagella_speed_as_cilia: float(&request, 16),
            cilia_speed_as_flagella: float(&request, 20),
            no_part_speed: float(&request, 24),
        };
        let mut points: [Vec<[f32; 2]>; 9] = std::array::from_fn(|_| Vec::new());
        for (slot, points) in points.iter_mut().enumerate() {
            let at = 192 + slot * 144;
            let count = word(&request, at + 4) as usize;
            let storage = word(&request, at);
            if count > 16 || storage > 1 || (storage == 1 && count > 1) {
                return Err("invalid scalar curve geometry".into());
            }
            *points = (0..count)
                .map(|index| {
                    [
                        float(&request, at + 16 + 8 * index),
                        float(&request, at + 20 + 8 * index),
                    ]
                })
                .collect();
        }
        let mut identifiers = IDS;
        identifiers[5] = word(&request, 4);
        let context = CellCapabilityScalarContext {
            x87: Context::from_control_word(control)?,
            sse: Policy::Mxcsr1f80,
        };
        let mut reference = None;
        for ambient in [0x1f80, 0xdf81, 0xbf84] {
            set_mxcsr(ambient);
            let value = derive_cell_capability_scalars(
                &counts,
                globals,
                identifiers[5],
                context,
                |identifier, query, context| {
                    let slot = identifiers
                        .iter()
                        .position(|candidate| *candidate == identifier)
                        .expect("native scalar literal ID is in the fixed map");
                    sample_capability_curve(
                        (word(&request, 28) != 0).then_some(points[slot].as_slice()),
                        query,
                        context,
                    )
                },
            );
            if current_mxcsr() != ambient {
                return Err(format!("scalar API changed caller MXCSR at case{ordinal}").into());
            }
            if nonfinite {
                let error = value
                    .err()
                    .ok_or("nonfinite-policy case unexpectedly succeeded")?;
                if error_label(error) != policy[ordinal] {
                    return Err(
                        format!("policy case{ordinal}: {error:?} != {}", policy[ordinal]).into(),
                    );
                }
                if ambient == 0x1f80 {
                    writeln!(
                        output,
                        "{ordinal}\t{}\t{}\t{:08X}",
                        policy[ordinal],
                        error_label(error),
                        word(&expected, 104)
                    )?;
                }
                continue;
            }
            let value =
                value.map_err(|error| format!("scalar case{ordinal},CW{control:04X}:{error}"))?;
            let actual = modeled(&request, &value);
            let mut normalized = [0u8; 232];
            normalized[..208].copy_from_slice(&expected[..208]);
            normalized[208..].copy_from_slice(&expected[216..]);
            if actual != normalized {
                let at = actual
                    .iter()
                    .zip(normalized)
                    .position(|(a, b)| *a != b)
                    .unwrap();
                let aligned = at / 4 * 4;
                return Err(format!("scalar case{ordinal},CW{control:04X},ambient{ambient:04X} differs at modeled byte{at}:Rust{:08X},native{:08X}", word(&actual, aligned), word(&normalized, aligned)).into());
            }
            if let Some(previous) = reference {
                if previous != actual {
                    return Err("modeled scalar bytes depend on ambient state".into());
                }
            } else {
                reference = Some(actual);
            }
        }
        if let Some(actual) = reference {
            output.write_all(&actual)?;
        }
    }
    output.flush()?;
    if nonfinite {
        println!("CELL_CAPABILITY_SCALAR_VERIFY policy_cases={cases} ambient_states=3 exact_rejections=true caller_MXCSR_restored=true");
        return Ok(());
    }
    // Reparse raw sources and independently bind each real request to the
    // measured native class-count tail, current Globals schema and actual PROP.
    let mut source_curves = BTreeMap::new();
    for name in ["curve-0.prop", "curve-1.prop"] {
        let bytes = std::fs::read(root.join(name))?;
        let tree = prop::parse(&bytes)?;
        if !tree.exact || tree.consumed != bytes.len() || tree.items.len() != 18 {
            return Err("source capability PROP is not exact named framing".into());
        }
        let mut curves = BTreeMap::new();
        let mut point_count = 0;
        for item in tree.items {
            if item.anonymous || item.prop_type != 0x30 || item.specifier != 0x9c {
                return Err("source capability property is not a named Vector2 array".into());
            }
            let mut pairs = Vec::new();
            for value in item.values {
                let PropValue::Vector2(x, y) = value else {
                    return Err("source curve value is not Vector2".into());
                };
                pairs.push([x.to_bits(), y.to_bits()]);
            }
            point_count += pairs.len();
            if curves.insert(item.id, pairs).is_some() {
                return Err("duplicate source curve identifier".into());
            }
        }
        if point_count != 104 {
            return Err("source capability point denominator differs".into());
        }
        source_curves.insert(name, curves);
    }
    let mut source_globals = BTreeMap::new();
    for name in ["globals-0.bin", "globals-2.bin"] {
        let bytes = std::fs::read(root.join(name))?;
        let globals = cell_globals::parse(&bytes)?;
        source_globals.insert(
            name,
            [
                globals.cilia_speed_as_jet(),
                globals.flagella_speed_as_jet(),
                globals.flagella_speed_as_cilia(),
                globals.cilia_speed_as_flagella(),
                globals.no_part_speed(),
            ]
            .map(f32::to_bits),
        );
    }
    if cell_globals::parse(&std::fs::read(root.join("globals-1.bin"))?).is_ok() {
        return Err("older 264-byte Globals was not positively excluded".into());
    }
    let table = std::fs::read_to_string(root.join("source-cases.tsv"))?;
    let mut lines = table.lines();
    if lines.next() != Some("case\tcreature_case\tglobals_file\tcurve_file\tcw") {
        return Err("unexpected scalar source table header".into());
    }
    let mut source_native = BufReader::new(File::open(composition.join("cache-output.bin"))?);
    let mut source_cases = 0;
    let mut used_creatures = BTreeSet::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 || fields[0].parse::<usize>()? != source_cases || fields[4] != "027F" {
            return Err("noncontiguous scalar source cases".into());
        }
        let creature = fields[1].parse::<u64>()?;
        if creature >= 1365 {
            return Err("source scalar creation out of bounds".into());
        }
        used_creatures.insert(creature);
        input.seek(SeekFrom::Start(source_cases as u64 * 1488))?;
        let mut request = [0u8; 1488];
        input.read_exact(&mut request)?;
        source_native.seek(SeekFrom::Start(creature * 144))?;
        let mut tail = [0u8; 144];
        source_native.read_exact(&mut tail)?;
        if request[32..172] != tail[4..]
            || word(&request, 4) != IDS[5]
            || word(&request, 28) != 1
            || u16::from_le_bytes(request[..2].try_into()?) != 0x27f
        {
            return Err("source scalar cache/count/ID binding differs".into());
        }
        let globals = source_globals
            .get(fields[2])
            .ok_or("unknown source Globals file")?;
        for (axis, &value) in globals.iter().enumerate() {
            if word(&request, 8 + axis * 4) != value {
                return Err("typed source Globals weight differs".into());
            }
        }
        let curves = source_curves
            .get(fields[3])
            .ok_or("unknown source curve PROP")?;
        for (slot, identifier) in IDS.into_iter().enumerate() {
            let points = curves
                .get(&identifier)
                .ok_or("source scalar curve missing")?;
            let at = 192 + slot * 144;
            if word(&request, at) != 0 || word(&request, at + 4) as usize != points.len() {
                return Err("source curve framing differs".into());
            }
            for (index, point) in points.iter().enumerate() {
                if [
                    word(&request, at + 16 + 8 * index),
                    word(&request, at + 20 + 8 * index),
                ] != *point
                {
                    return Err("typed source curve point differs".into());
                }
            }
        }
        source_cases += 1;
    }
    if source_cases != 5460 || used_creatures.len() != 1365 || contexts.len() != 12 {
        return Err("scalar complete source/context denominators differ".into());
    }
    println!("CELL_CAPABILITY_SCALAR_VERIFY cases={cases} source_cases={source_cases} source_creatures={} Globals_copies=2 PROP_copies=2 modeled_bytes_per_case=232 FP_metadata_bytes_validated=8 contexts={} ambient_states=3 caller_MXCSR_restored=true exact_modeled_bits=true", used_creatures.len(), contexts.len());
    Ok(())
}
