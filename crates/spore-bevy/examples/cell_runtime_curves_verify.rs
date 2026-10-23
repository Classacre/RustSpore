//! Pending full-channel native custom-curve comparison; parent owns admission.
//! Source cases reparse canonical original ANIM and all three PCTP copies.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{
    anim::{self, Anim, AnimBone, AnimComponent, AnimKeyframe},
    anim_curve::{key_neighbors, CurveError},
    anim_deform::{self, PreparedComponents},
    cell_runtime_curves::{sample_component, CellRuntimeCurveContext, CellRuntimeCurveError},
    native_sse::Policy,
    native_x87::{Context, Extended},
    part_capabilities::{self, PartCapabilities},
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Component, Path},
};

const INPUT: usize = 9632;
const OUTPUT: usize = 476;
const MODELED: usize = 468;
const HEADER: &str = "case\tfamily\tpackage\tlow\ttype\tscope\tsource_sha256\tchannel\tcomponent\tframe_bits\tcw\tflags\tslot\tupper\tprevious\tleft\tright\tfollowing\texpected_policy\tsource_component_ordinal";
const WITNESS_HEADER: &str = "package\tlow\ttype\tscope\tchannel\tcomponent\tendpoint\tsource_frame_u32\tnearest_f32_bits\tbefore_case\tbefore_frame_bits\tbefore_upper\tafter_case\tafter_frame_bits\tafter_upper";
type SourceComponent = (String, NativeResourceKey, usize, usize);

fn word(raw: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(raw[at..at + 4].try_into().unwrap())
}
fn put(raw: &mut [u8], at: usize, value: u32) {
    raw[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn hex(value: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(value, 16)
}
fn finite_bits(value: u32) -> bool {
    value & 0x7f80_0000 != 0x7f80_0000
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mxcsr() -> u32 {
    let mut value = 0;
    unsafe {
        std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut value,options(nostack,preserves_flags));
    }
    value
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn set_mxcsr(value: u32) {
    unsafe {
        std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&value,options(nostack,preserves_flags));
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct RestoreMxcsr(u32);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Drop for RestoreMxcsr {
    fn drop(&mut self) {
        set_mxcsr(self.0);
    }
}

fn native_unsigned(value: u32, context: Context) -> Result<Extended, CellRuntimeCurveError> {
    let signed = Extended::from_i32(value as i32);
    Ok(if value & 0x8000_0000 == 0 {
        signed
    } else {
        context.add(signed, Extended::from_f32(4294967296.)?)?
    })
}
fn upper(bone: &AnimBone, frame: f32, context: Context) -> Result<usize, CellRuntimeCurveError> {
    let frame = Extended::from_f32(frame)?;
    let mut result = 0;
    for row in &bone.keyframes {
        if context.compare(frame, native_unsigned(row.frame, context)?)? == Ordering::Less {
            break;
        }
        result += 1;
    }
    Ok(result)
}

fn evaluate(
    animation: &Anim,
    channel: usize,
    prepared: &[AnimComponent],
    component: usize,
    frame: f32,
    context: Context,
) -> Result<[u8; MODELED], CellRuntimeCurveError> {
    let sample = sample_component(
        animation,
        channel,
        prepared,
        component,
        frame,
        CellRuntimeCurveContext {
            x87: context,
            sse: Policy::Mxcsr1f80,
        },
    )?;
    let bone = &animation.bones[channel];
    let mut output = [0xa5; MODELED];
    if bone.keyframes.is_empty() {
        if sample.is_some() {
            return Err(CellRuntimeCurveError::UnsupportedDescriptor);
        }
        put(&mut output, 440, 0xdead_beef);
        put(&mut output, 444, 0);
        for at in [448, 452, 456, 460, 464] {
            put(&mut output, at, u32::MAX);
        }
        return Ok(output);
    }
    let selected_upper = upper(bone, frame, context)?;
    let selected = key_neighbors(
        bone.keyframes.len(),
        selected_upper,
        animation.header_flags_280 & 8 != 0,
    )?;
    let info = bone
        .components
        .iter()
        .find(|c| c.flags & 15 == 0 && c.tag != [0; 4])
        .ok_or(CellRuntimeCurveError::UnsupportedDescriptor)?;
    let stride = bone.keyframe_stride as usize;
    let info_at = selected[1] * stride + info.offset as usize;
    output[..20].copy_from_slice(&bone.raw_keyframes[info_at..info_at + 20]);
    let descriptor = &prepared[component];
    let destination = [0usize, 20, 100, 200, 240, 280, 320, 360, 400][descriptor.slot as usize];
    let expected_enabled = selected[1] == selected[2] || descriptor.flags & 0x10 != 0;
    match sample {
        Some(value) => {
            if !expected_enabled || value.neighbors != selected {
                return Err(CellRuntimeCurveError::UnsupportedDescriptor);
            }
            if selected[1] == selected[2] {
                let copied = value
                    .endpoint_bytes
                    .ok_or(CellRuntimeCurveError::UnsupportedDescriptor)?;
                output[destination..destination + 40].copy_from_slice(&copied);
            } else {
                if value.endpoint_bytes.is_some() {
                    return Err(CellRuntimeCurveError::UnsupportedDescriptor);
                }
                put(&mut output, destination, value.value.to_bits());
                put(&mut output, destination + 4, value.weight.to_bits());
            }
        }
        None if !expected_enabled => {}
        None => return Err(CellRuntimeCurveError::UnsupportedDescriptor),
    }
    let extended_frame = Extended::from_f32(frame)?;
    let exact = if context.compare(
        extended_frame,
        native_unsigned(bone.keyframes[selected[1]].frame, context)?,
    )? == Ordering::Equal
    {
        Some(selected[1])
    } else if context.compare(
        extended_frame,
        native_unsigned(bone.keyframes[selected[2]].frame, context)?,
    )? == Ordering::Equal
    {
        Some(selected[2])
    } else {
        None
    };
    put(&mut output, 440, exact.map_or(u32::MAX, |i| i as u32));
    put(&mut output, 444, u32::from(exact.is_some()));
    for (index, value) in selected.into_iter().enumerate() {
        put(&mut output, 448 + index * 4, value as u32);
    }
    put(&mut output, 464, selected_upper as u32);
    Ok(output)
}

fn next_float(value: f32, upward: bool) -> f32 {
    let bits = value.to_bits();
    if bits & 0x7fff_ffff == 0 {
        return f32::from_bits(if upward { 1 } else { 0x8000_0001 });
    }
    f32::from_bits(if (value > 0.) == upward {
        bits + 1
    } else {
        bits - 1
    })
}
fn schedule(bone: &AnimBone) -> BTreeMap<u32, &'static str> {
    let frames: Vec<_> = bone.keyframes.iter().map(|row| row.frame).collect();
    let mut result = BTreeMap::new();
    let Some((&first, &last)) = frames.first().zip(frames.last()) else {
        return result;
    };
    let mut before = first as f32;
    if f64::from(before) >= f64::from(first) {
        before = next_float(before, false);
    }
    let mut after = last as f32;
    if f64::from(after) <= f64::from(last) {
        after = next_float(after, true);
    }
    let mut proposed = vec![(before, "before_first"), (after, "after_last")];
    for (key, label) in [(first, "exact_first"), (last, "exact_last")] {
        if f64::from(key as f32) == f64::from(key) {
            proposed.push((key as f32, label));
        }
    }
    for pair in frames.windows(2) {
        if pair[1] <= pair[0] {
            continue;
        }
        let (left, right) = (f64::from(pair[0]), f64::from(pair[1]));
        let mut midpoint = ((left + right) / 2.) as f32;
        let mut kind = "segment_midpoint";
        if !(left < f64::from(midpoint) && f64::from(midpoint) < right) {
            midpoint = pair[0] as f32;
            kind = "segment_left_fallback";
            if f64::from(midpoint) < left {
                midpoint = next_float(midpoint, true);
            }
        }
        if left <= f64::from(midpoint) && f64::from(midpoint) < right {
            proposed.push((midpoint, kind));
        }
    }
    for (frame, kind) in proposed {
        result.entry(frame.to_bits()).or_insert(kind);
    }
    result
}

fn synthetic(request: &[u8; INPUT], flags: u32, slot: u32) -> Anim {
    let rows = word(request, 4) as usize;
    let components = vec![
        AnimComponent {
            flags: 0,
            tag: *b"INFO",
            offset: 0,
            stride: 60,
            slot: 0,
            reserved: [0; 3],
        },
        AnimComponent {
            flags,
            tag: 0x98c9_42b6u32.to_le_bytes(),
            offset: 20,
            stride: 60,
            slot,
            reserved: [0; 3],
        },
    ];
    Anim {
        path: String::new(),
        hash: 0,
        header_flags_280: word(request, 12),
        frame_duration: 1.,
        duration: f32::from_bits(word(request, 8)),
        last_frame: 0,
        bone_count: 1,
        header_field_304: 0,
        anim_name: String::new(),
        bones: vec![AnimBone {
            name: String::new(),
            keyframe_count: rows as u32,
            has_position: false,
            has_rotation: false,
            channel_flags: 0,
            selector_words: [0; 19],
            components,
            keyframe_stride: 60,
            keyframes: (0..rows)
                .map(|i| AnimKeyframe {
                    frame: word(request, 32 + 60 * i),
                    info: std::array::from_fn(|j| word(request, 36 + 60 * i + 4 * j)),
                    position: None,
                    rotation: None,
                })
                .collect(),
            raw_keyframes: request[32..32 + rows * 60].to_vec(),
        }],
    }
}

fn exact_read(
    index: &NativePackageIndex,
    key: NativeResourceKey,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut found = index.entries().iter().filter(|e| e.key == key);
    let first = found.next().ok_or("canonical source key absent")?;
    if found.next().is_some() {
        return Err("canonical source key ambiguous".into());
    }
    Ok(index.read(first)?)
}
fn dictionaries(install: &Path) -> Result<Vec<PartCapabilities>, Box<dyn Error>> {
    let mut result = vec![];
    for (package, low, scope) in [
        ("Data/PatchData.package", 0x142d_3338, 0x82d0),
        ("Data/Spore_Game.package", 0xf8cc_fe08, 0x6f14),
        ("DataEP1/Spore_EP1_Data.package", 0x82bc_2330, 0x82ea),
    ] {
        let index = NativePackageIndex::open(install.join(package))?;
        result.push(part_capabilities::parse(&exact_read(
            &index,
            NativeResourceKey {
                instance_id: low,
                type_id: 0x7c19_aa7a,
                group_id: scope,
            },
        )?)?);
    }
    Ok(result)
}
fn relative(value: &str) -> Result<&Path, Box<dyn Error>> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err("source package path must remain relative".into());
    }
    Ok(path)
}

struct Witness {
    row: Vec<String>,
    seen: u8,
}
fn witnesses(path: &Path) -> Result<Vec<Witness>, Box<dyn Error>> {
    let mut rows = BufReader::new(File::open(path)?).lines();
    if rows.next().transpose()?.as_deref() != Some(WITNESS_HEADER) {
        return Err("unexpected time witness header".into());
    }
    let mut result = vec![];
    for row in rows {
        let row: Vec<_> = row?.split('\t').map(str::to_owned).collect();
        if row.len() != 15 {
            return Err("malformed time witness".into());
        }
        result.push(Witness { row, seen: 0 });
    }
    if result.len() != 12 {
        return Err("expected twelve nonrepresentable endpoint witnesses".into());
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 7 {
        return Err("usage: cell_runtime_curves_verify install input.bin cases.tsv native.bin modeled.bin policy.tsv nonrepresentable-times.tsv".into());
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    return Err("native x86 floating policy required".into());
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let _restore = RestoreMxcsr(mxcsr());
    set_mxcsr(0x1f80);
    let install = Path::new(&args[0]);
    let dictionaries = dictionaries(install)?;
    let mut request_stream = BufReader::new(File::open(&args[1])?);
    let mut metadata = BufReader::new(File::open(&args[2])?).lines();
    let mut native_stream = BufReader::new(File::open(&args[3])?);
    if metadata.next().transpose()?.as_deref() != Some(HEADER) {
        return Err("unexpected curve metadata header".into());
    }
    let new_file = |path: &Path| OpenOptions::new().write(true).create_new(true).open(path);
    let mut output = BufWriter::new(new_file(Path::new(&args[4]))?);
    let mut projection = BufWriter::new(new_file(
        &Path::new(&args[4]).with_extension("native-projection.bin"),
    )?);
    let mut policies = BufWriter::new(new_file(Path::new(&args[5]))?);
    let mut witnesses = witnesses(Path::new(&args[6]))?;
    let mut witness_output = BufWriter::new(new_file(
        &Path::new(&args[4]).with_extension("time-witnesses.tsv"),
    )?);
    writeln!(
        policies,
        "case\tpolicy\tnative_value_bits\tnative_weight_bits\tflags"
    )?;
    writeln!(witness_output,"case\tpackage\tlow\ttype\tscope\tchannel\tcomponent\tendpoint\tsource_time\tnearest_f32\tnative_upper")?;
    let mut package: Option<(String, NativePackageIndex)> = None;
    let mut loaded: Option<(NativeResourceKey, Anim, PreparedComponents)> = None;
    let mut remaining: BTreeMap<SourceComponent, BTreeSet<u32>> = BTreeMap::new();
    let mut unique_clips = BTreeSet::new();
    let mut context_set = BTreeSet::new();
    let (mut total, mut modeled_count, mut policy_count, mut source_count, mut witness_count) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for line in metadata {
        set_mxcsr(0x1f80);
        let line = line?;
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 20 || row[0].parse::<usize>()? != total {
            return Err("malformed curve metadata row".into());
        }
        let mut request = [0; INPUT];
        request_stream.read_exact(&mut request)?;
        let mut native = [0; OUTPUT];
        native_stream.read_exact(&mut native)?;
        let frame = f32::from_bits(word(&request, 0));
        let count = word(&request, 4) as usize;
        let flags = hex(row[11])?;
        let slot: u32 = row[12].parse()?;
        let cw = word(&request, 16) as u16;
        let context = Context::from_control_word(cw)?;
        let choice = word(&request, 20) as usize;
        if count > 160
            || choice >= 24
            || word(&request, 24) != 0
            || word(&request, 28) != 0
            || !frame.is_finite()
            || !finite_bits(word(&request, 8))
            || word(&request, 12) & !8 != 0
            || word(&request, 16) != u32::from(cw)
        {
            return Err(format!("invalid curve input case{total}").into());
        }
        if [0x83, 0x93, 0xa3, 0xb3][choice / 6] != flags
            || (choice % 6 + 3) as u32 != slot
            || row[9] != format!("{:08X}", frame.to_bits())
            || row[10] != format!("{cw:04X}")
        {
            return Err("descriptor/metadata mismatch".into());
        }
        if u16::from_le_bytes(native[468..470].try_into()?) != cw
            || u16::from_le_bytes(native[470..472].try_into()?) & 0x3800 != 0
            || word(&native, 472) & !0x3f != 0x1f80
        {
            return Err("native curve floating metadata mismatch".into());
        }
        let is_source = row[1].starts_with("source_");
        let source_key = if is_source {
            Some(NativeResourceKey {
                instance_id: hex(row[3])?,
                type_id: hex(row[4])?,
                group_id: hex(row[5])?,
            })
        } else {
            None
        };
        if let Some(key) = source_key {
            if key.type_id != 0xee17_c6ad {
                return Err("source is not ANIM".into());
            }
            if package.as_ref().map(|p| p.0.as_str()) != Some(row[2]) {
                package = Some((
                    row[2].to_owned(),
                    NativePackageIndex::open(install.join(relative(row[2])?))?,
                ));
                loaded = None;
            }
            if loaded.as_ref().map(|v| v.0) != Some(key) {
                let animation = anim::parse(&exact_read(&package.as_ref().unwrap().1, key)?)?;
                let prepared = anim_deform::prepare_native_initial(&animation, &dictionaries[0])?;
                for dictionary in &dictionaries[1..] {
                    if anim_deform::prepare_native_initial(&animation, dictionary)? != prepared {
                        return Err("three source PCTP preparations differ".into());
                    }
                }
                loaded = Some((key, animation, prepared));
            }
        }
        let constructed = (!is_source).then(|| synthetic(&request, flags, slot));
        let (animation, descriptors, channel, component) =
            if let Some(animation) = constructed.as_ref() {
                (
                    animation,
                    animation.bones[0].components.as_slice(),
                    0usize,
                    1usize,
                )
            } else {
                let (_, animation, prepared) = loaded.as_ref().unwrap();
                let channel: usize = row[7].parse()?;
                let component: usize = row[8].parse()?;
                (
                    animation,
                    prepared
                        .channels
                        .get(channel)
                        .ok_or("source channel absent")?
                        .components
                        .as_slice(),
                    channel,
                    component,
                )
            };
        let bone = &animation.bones[channel];
        if bone.keyframes.len() != count || bone.keyframe_count as usize != count {
            return Err("source channel count was cropped".into());
        }
        if let Some(key) = source_key {
            let descriptor = descriptors
                .get(component)
                .ok_or("source component absent")?;
            if descriptor.flags != flags
                || descriptor.slot != slot
                || descriptor.flags & 15 != 3
                || ![0x98c942b6, 0x98c942b5, 0x98c942b4, 0x98c942b3]
                    .contains(&u32::from_le_bytes(descriptor.tag))
            {
                return Err("source descriptor preparation differs".into());
            }
            if animation.duration.to_bits() != word(&request, 8)
                || animation.header_flags_280 & 8 != word(&request, 12)
            {
                return Err("source duration/header changed".into());
            }
            let info = bone
                .components
                .iter()
                .find(|c| c.flags & 15 == 0 && c.tag != [0; 4])
                .ok_or("source INFO absent")?;
            for index in 0..count {
                let at = index * bone.keyframe_stride as usize;
                let expected_info =
                    &bone.raw_keyframes[at + info.offset as usize..at + info.offset as usize + 20];
                let expected_custom = &bone.raw_keyframes
                    [at + descriptor.offset as usize..at + descriptor.offset as usize + 40];
                if request[32 + 60 * index..52 + 60 * index] != *expected_info
                    || request[52 + 60 * index..92 + 60 * index] != *expected_custom
                {
                    return Err("source INFO/custom row bytes changed".into());
                }
            }
            let scheduled = schedule(bone);
            let kind = scheduled
                .get(&frame.to_bits())
                .ok_or("source frame outside independently recomputed schedule")?;
            if row[1] != format!("source_{kind}") {
                return Err("source frame schedule kind mismatch".into());
            }
            let identity = (row[2].to_owned(), key, channel, component);
            let frames = remaining
                .entry(identity)
                .or_insert_with(|| scheduled.keys().copied().collect());
            if !frames.remove(&frame.to_bits()) {
                return Err("source component sample repeated".into());
            }
            unique_clips.insert((row[2].to_owned(), key));
            source_count += 1;
            for witness in &mut witnesses {
                for (case_at, frame_at, upper_at, mask) in [(9, 10, 11, 1u8), (12, 13, 14, 2u8)] {
                    if witness.row[case_at].parse::<usize>()? != total {
                        continue;
                    }
                    if witness.row[..6]
                        != row[2..6]
                            .iter()
                            .map(|s| s.to_string())
                            .chain([row[7].to_owned(), row[8].to_owned()])
                            .collect::<Vec<_>>()
                    {
                        return Err("time witness source identity differs".into());
                    }
                    let endpoint = match witness.row[6].as_str() {
                        "first" => 0,
                        "last" => count - 1,
                        _ => return Err("invalid time witness endpoint".into()),
                    };
                    let value = bone.keyframes[endpoint].frame;
                    if value != witness.row[7].parse::<u32>()?
                        || f64::from(value as f32) == f64::from(value)
                        || (value as f32).to_bits() != hex(&witness.row[8])?
                        || frame.to_bits() != hex(&witness.row[frame_at])?
                        || word(&native, 464) != witness.row[upper_at].parse::<u32>()?
                    {
                        return Err("nonrepresentable source time witness failed".into());
                    }
                    witness.seen |= mask;
                    witness_count += 1;
                    writeln!(
                        witness_output,
                        "{total}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{value}\t{:08X}\t{}",
                        row[2],
                        row[3],
                        row[4],
                        row[5],
                        channel,
                        component,
                        witness.row[6],
                        (value as f32).to_bits(),
                        word(&native, 464)
                    )?;
                }
            }
        }
        let mut actual = None;
        for incoming in [0x1f80, 0xdf81, 0xbf84] {
            set_mxcsr(incoming);
            let next = evaluate(animation, channel, descriptors, component, frame, context);
            if mxcsr() != incoming {
                return Err(format!("caller MXCSR changed case{total}").into());
            }
            if actual.as_ref().is_some_and(|old| old != &next) {
                return Err(format!("caller state changes curve result case{total}").into());
            }
            actual = Some(next);
        }
        set_mxcsr(0x1f80);
        match (actual.unwrap(), row[18]) {
            (Ok(actual), "-") => {
                if actual != native[..MODELED] {
                    let byte = actual
                        .iter()
                        .zip(&native)
                        .position(|(a, b)| a != b)
                        .unwrap();
                    return Err(format!("curve case{total} differs at modeled byte{byte}, source={} channel={channel} component={component}",row[2]).into());
                }
                output.write_all(&actual)?;
                projection.write_all(&native[..MODELED])?;
                modeled_count += 1;
            }
            (Err(CellRuntimeCurveError::Curve(CurveError::NonFiniteInput)), "NonFiniteInput") => {
                let offset = [0usize, 20, 100, 200, 240, 280, 320, 360, 400][slot as usize];
                writeln!(
                    policies,
                    "{total}\tNonFiniteInput\t{:08X}\t{:08X}\t{flags:08X}",
                    word(&native, offset),
                    word(&native, offset + 4)
                )?;
                policy_count += 1;
            }
            (actual, policy) => {
                return Err(format!(
                    "curve case{total} unexpected policy {policy}: {}",
                    match actual {
                        Ok(_) => "modeled".to_owned(),
                        Err(e) => format!("{e:?}"),
                    }
                )
                .into())
            }
        }
        let selected_upper = if count == 0 {
            -1
        } else {
            upper(bone, frame, context)? as i64
        };
        if row[13].parse::<i64>()? != selected_upper || word(&native, 464) != selected_upper as u32
        {
            return Err("source global upper selection mismatch".into());
        }
        for index in 0..4 {
            if row[14 + index].parse::<u32>()? != word(&native, 448 + index * 4) {
                return Err("source neighbor metadata mismatch".into());
            }
        }
        context_set.insert(cw);
        total += 1;
    }
    let mut trailing = [0];
    if request_stream.read(&mut trailing)? != 0
        || native_stream.read(&mut trailing)? != 0
        || remaining.values().any(|set| !set.is_empty())
        || witnesses.iter().any(|w| w.seen != 3)
        || total != 84544
        || modeled_count != 84256
        || policy_count != 288
        || source_count != 50367
        || remaining.len() != 2086
        || unique_clips.len() != 818
        || context_set.len() != 12
        || witness_count != 24
    {
        return Err(
            "curve denominators, frame schedule coverage or time witnesses incomplete".into(),
        );
    }
    output.flush()?;
    projection.flush()?;
    policies.flush()?;
    witness_output.flush()?;
    println!("CELL_RUNTIME_CURVES_VERIFY cases={total} modeled={modeled_count} policy={policy_count} source_cases={source_count} clips={} components={} contexts={} dictionaries=3 caller_states=3 time_witnesses=12 witness_case_checks={witness_count} exact_bits=true",unique_clips.len(),remaining.len(),context_set.len());
    Ok(())
}
