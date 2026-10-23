//! Verify typed RW4 animation against every RW4 resource in supplied packages.
//! Usage: tools\build.bat run -p spore-bevy --example rw4_animation_verify -- <directory>...
//! Source assets and resulting resource-level diagnostics stay local.

use spore_core::rw4_animation::{
    self as animation, AffineTransform, AnimationError, SampleValue, TrackValues,
};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn collect(path: &Path, packages: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_symlink() {
                collect(&entry.path(), packages)?;
            }
        }
    } else if path.is_file()
        && path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("package"))
    {
        packages.push(path.to_path_buf());
    } else if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.display().to_string(),
        ));
    }
    Ok(())
}

fn identity_error(matrix: AffineTransform) -> f32 {
    matrix
        .rows
        .iter()
        .flatten()
        .zip(AffineTransform::IDENTITY.rows.iter().flatten())
        .map(|(a, b)| (a - b).abs())
        .fold(0., f32::max)
}

fn compose(
    j: usize,
    local: &[AffineTransform],
    parents: &[Option<usize>],
    global: &mut [Option<AffineTransform>],
) -> AffineTransform {
    if let Some(m) = global[j] {
        return m;
    }
    let m = match parents[j] {
        Some(p) => compose(p, local, parents, global).then_local(local[j]),
        None => local[j],
    };
    global[j] = Some(m);
    m
}

#[derive(Default)]
struct Summary {
    packages: usize,
    rw4: usize,
    parsed: usize,
    problems: usize,
    skeletons: usize,
    bones: usize,
    skins: usize,
    bindings: usize,
    clips: usize,
    reserved_bytes: usize,
    scalar_keys: usize,
    transform_keys: usize,
    samples: usize,
    track_types: BTreeMap<(u32, u32), usize>,
    unsupported_tracks: usize,
    sample_errors: BTreeMap<String, usize>,
    skinned_meshes: usize,
    skinned_vertices: usize,
    skinning_errors: BTreeMap<String, usize>,
    binding_mesh_resources: usize,
    invertible_bind: usize,
    total_bind: usize,
    matrix_identities: usize,
    matrix_total: usize,
    matched_clips: usize,
    partial_clips: usize,
    compatible_clips: usize,
    compatible_id_equal: usize,
    compatible_id_different: usize,
    clip_id_examples: Vec<String>,
    nontrivial_clips: usize,
    direct_identity: usize,
    composed_identity: usize,
    composed_only: usize,
    direct_only: usize,
    composed_better: usize,
    direct_better: usize,
    tied: usize,
    direct_errors: Vec<f32>,
    composed_errors: Vec<f32>,
    comparison_examples: Vec<String>,
    parse_errors: BTreeMap<String, usize>,
}

fn examine(
    data: &[u8],
    rw: &spore_core::rw4::Rw4,
    set: &animation::AnimationSet,
    label: &str,
    s: &mut Summary,
) {
    s.skeletons += set.skeletons.len();
    s.bones += set
        .skeletons
        .iter()
        .map(|x| x.value.bones.len())
        .sum::<usize>();
    s.skins += set.animation_skins.len();
    s.bindings += set.bindings.len();
    s.clips += set.clips.len();
    for skin in &set.animation_skins {
        for m in &skin.value.inverse_bind {
            s.total_bind += 1;
            s.invertible_bind += usize::from(m.inverse().is_some());
        }
    }
    for matrices in &set.skin_matrices {
        for m in &matrices.value.matrices {
            s.matrix_total += 1;
            s.matrix_identities += usize::from(identity_error(*m) < 1e-5);
        }
    }
    for clip in &set.clips {
        s.reserved_bytes += clip.value.reserved_bytes;
        for track in &clip.value.tracks {
            *s.track_types.entry((track.kind, track.stride)).or_default() += 1;
            match &track.values {
                TrackValues::Scalar(keys) => s.scalar_keys += keys.len(),
                TrackValues::Transform(keys) => s.transform_keys += keys.len(),
                TrackValues::Unsupported(_) => s.unsupported_tracks += 1,
            }
        }
        if clip.value.is_sampleable() {
            for time in [0., clip.value.duration * 0.5, clip.value.duration] {
                match clip.value.sample(time, false) {
                    Ok(values) => {
                        s.samples += 1;
                        for value in values {
                            let finite = match value.value {
                                SampleValue::Scalar(v) => v.is_finite(),
                                SampleValue::Transform(t) => t
                                    .affine()
                                    .is_ok_and(|m| m.rows.iter().flatten().all(|v| v.is_finite())),
                            };
                            if !finite {
                                *s.sample_errors
                                    .entry("non-finite sampled value".into())
                                    .or_default() += 1;
                            }
                        }
                    }
                    Err(e) => *s.sample_errors.entry(e.to_string()).or_default() += 1,
                }
            }
        }
    }
    let meshes = rw.meshes(data);
    if !set.bindings.is_empty() && !meshes.is_empty() {
        s.binding_mesh_resources += 1;
    }
    for binding in &set.bindings {
        let skeleton = &set
            .skeletons
            .iter()
            .find(|x| x.section_index == binding.value.skeleton)
            .unwrap()
            .value;
        let skin = &set
            .animation_skins
            .iter()
            .find(|x| x.section_index == binding.value.animation_skin)
            .unwrap()
            .value;
        for mesh in &meshes {
            match animation::vertex_skinning(rw, data, mesh, skeleton.bones.len()) {
                Ok(vertices) => {
                    s.skinned_meshes += 1;
                    s.skinned_vertices += vertices.len();
                }
                Err(e) => {
                    let count = s.skinning_errors.entry(e.to_string()).or_default();
                    *count += 1;
                    if *count <= 4 {
                        eprintln!(
                            "skinning_unsupported {label} binding_section={}: {e}",
                            binding.section_index
                        );
                    }
                }
            }
        }
        if skeleton.bones.is_empty() {
            continue;
        }
        let parents: Vec<_> = skeleton.bones.iter().map(|b| b.parent).collect();
        for clip in &set.clips {
            let transform_ids: Vec<_> = clip
                .value
                .tracks
                .iter()
                .filter(|t| matches!(t.values, TrackValues::Transform(_)))
                .map(|t| t.bone_id)
                .collect();
            if !transform_ids.is_empty()
                && transform_ids
                    .iter()
                    .all(|id| skeleton.bones.iter().filter(|b| b.id == *id).count() == 1)
            {
                s.compatible_clips += 1;
                if clip.value.id == skeleton.id {
                    s.compatible_id_equal += 1;
                } else {
                    s.compatible_id_different += 1;
                    if s.clip_id_examples.len() < 5 {
                        s.clip_id_examples.push(format!(
                            "{label} skeleton={:08X} clip={:08X} transform_tracks={}",
                            skeleton.id,
                            clip.value.id,
                            transform_ids.len()
                        ));
                    }
                }
            }
            let mut local = Vec::new();
            for bone in &skeleton.bones {
                let matching: Vec<_> = clip
                    .value
                    .tracks
                    .iter()
                    .filter(|t| {
                        t.bone_id == bone.id && matches!(t.values, TrackValues::Transform(_))
                    })
                    .collect();
                if matching.len() != 1 {
                    break;
                }
                let TrackValues::Transform(keys) = &matching[0].values else {
                    unreachable!()
                };
                let Some(first) = keys.first() else {
                    break;
                };
                let Ok(matrix) = first.value.affine() else {
                    break;
                };
                local.push(matrix);
            }
            if local.len() != skeleton.bones.len() {
                s.partial_clips += 1;
                continue;
            }
            s.matched_clips += 1;
            if parents.iter().all(Option::is_none) {
                continue;
            }
            s.nontrivial_clips += 1;
            let mut global = vec![None; local.len()];
            let direct = local
                .iter()
                .zip(&skin.inverse_bind)
                .map(|(m, ib)| identity_error(m.then_local(*ib)))
                .fold(0., f32::max);
            let composed = (0..local.len())
                .map(|j| {
                    identity_error(
                        compose(j, &local, &parents, &mut global).then_local(skin.inverse_bind[j]),
                    )
                })
                .fold(0., f32::max);
            s.direct_errors.push(direct);
            s.composed_errors.push(composed);
            let di = direct < 1e-3;
            let ci = composed < 1e-3;
            s.direct_identity += usize::from(di);
            s.composed_identity += usize::from(ci);
            s.composed_only += usize::from(ci && !di);
            s.direct_only += usize::from(di && !ci);
            if composed + 1e-4 < direct {
                s.composed_better += 1;
            } else if direct + 1e-4 < composed {
                s.direct_better += 1;
            } else {
                s.tied += 1;
            }
            if (ci && !di && s.comparison_examples.len() < 8) || label.contains("406060007A070007")
            {
                s.comparison_examples.push(format!("{label} skeleton_section={} clip_section={} bones={} direct_max={direct:.8} composed_max={composed:.8}",binding.value.skeleton,clip.section_index,local.len()));
            }
        }
    }
}

fn quantiles(values: &mut [f32]) -> String {
    if values.is_empty() {
        return "none".into();
    }
    values.sort_by(f32::total_cmp);
    format!(
        "min={:.8} median={:.8} p95={:.8} max={:.8}",
        values[0],
        values[values.len() / 2],
        values[(values.len() - 1) * 95 / 100],
        values[values.len() - 1]
    )
}

struct Fnv(u64);
impl Fnv {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
    fn word(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
    fn float(&mut self, v: f32) {
        self.word(v.to_bits());
    }
}

/// The independent Python probe reconstructs this canonical typed-word stream.
fn digests(
    writer: &mut BufWriter<std::fs::File>,
    path: &Path,
    key: dbpf::ResourceKey,
    data: &[u8],
    rw: &spore_core::rw4::Rw4,
    set: &animation::AnimationSet,
) {
    let mut emit = |kind: &str, index: String, count: usize, h: Fnv| {
        writeln!(
            writer,
            "{}\t{}\t{kind}\t{index}\t{count}\t{:016X}",
            path.display(),
            key,
            h.0
        )
        .unwrap()
    };
    for item in &set.skeletons {
        let s = &item.value;
        let mut h = Fnv::new();
        h.word(s.id);
        h.word(s.flags);
        h.word(s.bones.len() as u32);
        for b in &s.bones {
            h.word(b.id);
            h.word(b.flags);
            h.word(b.parent.map_or(u32::MAX, |p| p as u32));
        }
        emit("skeleton", item.section_index.to_string(), s.bones.len(), h);
    }
    for item in &set.animation_skins {
        let mut h = Fnv::new();
        h.word(item.value.raw_columns.len() as u32);
        for (m, affine) in item.value.raw_columns.iter().zip(&item.value.inverse_bind) {
            for v in m {
                h.float(*v);
            }
            for v in affine.rows.iter().flatten() {
                h.float(*v);
            }
        }
        emit(
            "inverse_bind",
            item.section_index.to_string(),
            item.value.raw_columns.len(),
            h,
        );
    }
    for item in &set.skin_matrices {
        let mut h = Fnv::new();
        h.word(item.value.matrices.len() as u32);
        for m in &item.value.matrices {
            for v in m.rows.iter().flatten() {
                h.float(*v);
            }
        }
        emit(
            "matrix_slots",
            item.section_index.to_string(),
            item.value.matrices.len(),
            h,
        );
    }
    for item in &set.clips {
        let c = &item.value;
        let mut h = Fnv::new();
        h.word(c.id);
        h.float(c.duration);
        h.word(c.flags);
        h.word(c.reserved_bytes as u32);
        h.word(c.tracks.len() as u32);
        let mut keys = 0;
        for t in &c.tracks {
            h.word(t.bone_id);
            h.word(t.kind);
            h.word(t.stride);
            match &t.values {
                TrackValues::Scalar(values) => {
                    h.word(values.len() as u32);
                    keys += values.len();
                    for k in values {
                        h.float(k.time);
                        h.float(k.value);
                    }
                }
                TrackValues::Transform(values) => {
                    h.word(values.len() as u32);
                    keys += values.len();
                    for k in values {
                        h.float(k.time);
                        for v in k.value.rotation {
                            h.float(v);
                        }
                        for v in k.value.translation {
                            h.float(v);
                        }
                        for v in k.value.scale {
                            h.float(v);
                        }
                    }
                }
                TrackValues::Unsupported(bytes) => {
                    h.word(bytes.len() as u32);
                    for b in bytes {
                        h.word(*b as u32);
                    }
                }
            }
        }
        emit("clip", item.section_index.to_string(), keys, h);
    }
    for item in &set.bindings {
        let skeleton = &set
            .skeletons
            .iter()
            .find(|s| s.section_index == item.value.skeleton)
            .unwrap()
            .value;
        for (ordinal, mesh) in rw.meshes(data).iter().enumerate() {
            if let Ok(values) = animation::vertex_skinning(rw, data, mesh, skeleton.bones.len()) {
                let mut h = Fnv::new();
                h.word(values.len() as u32);
                for v in &values {
                    for joint in v.joints {
                        h.word(joint as u32);
                    }
                    for weight in v.weights {
                        h.float(weight);
                    }
                }
                emit(
                    "influences",
                    format!("{}:{ordinal}", item.section_index),
                    values.len(),
                    h,
                );
            }
        }
    }
}

fn main() {
    let mut args = std::env::args_os().skip(1).peekable();
    if args.peek().is_none() {
        eprintln!(
            "usage: rw4_animation_verify [--bone-inventory <local.tsv>] <package-or-directory>..."
        );
        std::process::exit(2);
    }
    let mut packages = Vec::new();
    let mut inventory = None;
    let mut digest_output = None;
    while let Some(arg) = args.next() {
        if arg == "--digests" {
            let path = args.next().expect("--digests needs a path");
            let mut writer =
                BufWriter::new(std::fs::File::create(path).expect("create digest output"));
            writeln!(writer, "package\tresource_key\tkind\tsection\tcount\tfnv64").unwrap();
            digest_output = Some(writer);
            continue;
        }
        if arg == "--bone-inventory" {
            let path = args.next().expect("--bone-inventory needs a path");
            let mut writer =
                BufWriter::new(std::fs::File::create(path).expect("create bone inventory"));
            writeln!(writer,"package\tresource_key\tskeleton_section\tskeleton_id\tskeleton_flags\tbone_index\tbone_id\tparent_index\tbone_flags").unwrap();
            inventory = Some(writer);
            continue;
        }
        if let Err(e) = collect(Path::new(&arg), &mut packages) {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
    packages.sort();
    packages.dedup();
    let mut s = Summary::default();
    for path in &packages {
        let index = match dbpf::PackageIndex::open(path) {
            Ok(index) => index,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                s.problems += 1;
                continue;
            }
        };
        s.packages += 1;
        let mut local = 0;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x2F4E_681B)
        {
            s.rw4 += 1;
            local += 1;
            let label = format!(
                "{} {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                entry.key
            );
            let bytes = match index.read(entry) {
                Ok(b) => b,
                Err(e) => {
                    s.problems += 1;
                    eprintln!("{label}: {e}");
                    continue;
                }
            };
            let rw = match spore_core::rw4::parse(&bytes) {
                Ok(r) => r,
                Err(e) => {
                    s.problems += 1;
                    eprintln!("{label}: {e}");
                    continue;
                }
            };
            match animation::parse(&rw, &bytes) {
                Ok(set) => {
                    s.parsed += 1;
                    if let Some(writer) = &mut digest_output {
                        digests(writer, path, entry.key, &bytes, &rw, &set);
                    }
                    if let Some(writer) = &mut inventory {
                        for skeleton in &set.skeletons {
                            for (i, bone) in skeleton.value.bones.iter().enumerate() {
                                writeln!(
                                    writer,
                                    "{}\t{}\t{}\t{:08X}\t{}\t{}\t{:08X}\t{}\t{}",
                                    path.display(),
                                    entry.key,
                                    skeleton.section_index,
                                    skeleton.value.id,
                                    skeleton.value.flags,
                                    i,
                                    bone.id,
                                    bone.parent.map_or(-1, |p| p as i64),
                                    bone.flags
                                )
                                .unwrap();
                            }
                        }
                    }
                    examine(&bytes, &rw, &set, &label, &mut s);
                }
                Err(e) => {
                    s.problems += 1;
                    let reason = match e {
                        AnimationError::Malformed { reason, .. } => reason.to_owned(),
                        _ => e.to_string(),
                    };
                    let count = s.parse_errors.entry(reason).or_default();
                    *count += 1;
                    if *count <= 4 {
                        eprintln!("{label}: {e}");
                    }
                }
            }
        }
        if local > 0 {
            println!("package={} rw4={local}", path.display());
        }
    }
    println!(
        "packages={}/{} rw4={}/{} problems={}",
        s.packages,
        packages.len(),
        s.parsed,
        s.rw4,
        s.problems
    );
    println!("skeletons={} bones={} animation_skins={} bindings={} clips={} scalar_keys={} transform_keys={} sample_calls={}",s.skeletons,s.bones,s.skins,s.bindings,s.clips,s.scalar_keys,s.transform_keys,s.samples);
    for ((kind, stride), count) in s.track_types {
        println!("track_type=0x{kind:X} stride={stride} tracks={count}");
    }
    println!(
        "unsupported_tracks={} parse_errors={:?} sample_errors={:?}",
        s.unsupported_tracks, s.parse_errors, s.sample_errors
    );
    println!("validated_zero_reserve_bytes={}", s.reserved_bytes);
    println!(
        "binding_mesh_resources={} skinned_meshes={} skinned_vertices={} skinning_errors={:?}",
        s.binding_mesh_resources, s.skinned_meshes, s.skinned_vertices, s.skinning_errors
    );
    println!(
        "invertible_inverse_bind={}/{} skin_matrix_identity={}/{}",
        s.invertible_bind, s.total_bind, s.matrix_identities, s.matrix_total
    );
    println!("matching_complete_clips={} partial_or_unrelated_clips={} nontrivial_hierarchy_comparisons={}",s.matched_clips,s.partial_clips,s.nontrivial_clips);
    println!(
        "transform_id_subset_compatible={} clip_skeleton_id_equal={} different={}",
        s.compatible_clips, s.compatible_id_equal, s.compatible_id_different
    );
    for example in s.clip_id_examples {
        println!("compatible_id_different {example}");
    }
    println!("first_pose_inverse_bind direct_identity={} composed_identity={} composed_only={} direct_only={} composed_better={} direct_better={} tied={}",s.direct_identity,s.composed_identity,s.composed_only,s.direct_only,s.composed_better,s.direct_better,s.tied);
    println!("direct_error {}", quantiles(&mut s.direct_errors));
    println!("composed_error {}", quantiles(&mut s.composed_errors));
    for example in s.comparison_examples {
        println!("comparison {example}");
    }
    if s.rw4 == 0 || s.problems > 0 || !s.sample_errors.is_empty() {
        std::process::exit(1);
    }
}
