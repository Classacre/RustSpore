//! Strict source material-link/state association over caller-supplied packages.
use spore_core::{material_links, rw4};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for e in std::fs::read_dir(path)? {
            let e = e?;
            if !e.file_type()?.is_symlink() {
                collect(&e.path(), out)?;
            }
        }
    } else if path
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("package"))
    {
        out.push(path.to_owned());
    } else if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.display().to_string(),
        ));
    }
    Ok(())
}
struct Fnv(u64);
impl Fnv {
    fn new() -> Self {
        Self(0xCBF29CE484222325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for v in b {
            self.0 = (self.0 ^ u64::from(*v)).wrapping_mul(0x100000001B3);
        }
    }
    fn word(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
}
fn binding_digest(
    links: &material_links::MaterialLinks,
    states: &material_links::MaterialStates,
) -> Result<Fnv, material_links::MaterialError> {
    let mut h = Fnv::new();
    fn pairs(h: &mut Fnv, p: &[material_links::StateValue]) {
        h.word(p.len() as u32);
        for v in p { h.word(v.state); h.word(v.value); }
    }
    h.word(states.states.len() as u32);
    for state in &states.states {
        let b = state.bindings()?;
        if b.to_bytes()? != state.payload { return Err(material_links::MaterialError::InvalidBinding); }
        h.word(state.section_index as u32);
        h.word(b.prefix.len() as u32);
        for region in &b.prefix {
            h.word(region.flag);
            h.word(region.bytes.len() as u32);
            h.bytes(&region.bytes);
        }
        h.word(u32::from(b.groups.is_some()));
        h.word(b.groups.as_ref().map_or(0, Vec::len) as u32);
        for g in b.groups.iter().flatten() { h.word(g.id); pairs(&mut h, &g.values); }
        h.word(b.texture_leading_word);
        h.word(b.textures.len() as u32);
        for t in &b.textures {
            h.word(t.stage);
            h.word(t.raster_reference);
            h.word(t.list_a_flags);
            pairs(&mut h, &t.list_a);
            h.word(t.list_b_flags);
            pairs(&mut h, &t.list_b);
        }
    }
    h.word(links.materials.len() as u32);
    for (i, link) in links.materials.iter().enumerate() {
        let slots = states.texture_slots(i, link)?;
        h.word(slots.len() as u32);
        for slot in slots { h.word(slot.state_index as u32); h.word(slot.stage); }
    }
    Ok(h)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: material_links_verify <install> <new digest TSV>".into());
    }
    let mut out = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])?;
    writeln!(
        out,
        "package\tresource_key\tmaterials\tstates\ttextures\tfnv64\tbindings_fnv64"
    )?;
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    paths.dedup();
    let (mut packages, mut files, mut materials, mut states, mut textures) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for e in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0469A3F7 && e.key.instance_id >> 32 == 0x40212000)
        {
            let bytes = index.read(e)?;
            let links = material_links::parse(&bytes)?;
            if links.to_bytes()? != bytes {
                return Err("material-link reserialization mismatch".into());
            }
            let instance = 0x4021200100000000 | (e.key.instance_id & 0xFFFFFFFF);
            let matches: Vec<_> = index
                .entries()
                .iter()
                .filter(|r| r.key.type_id == 0x2F4E681B && r.key.instance_id == instance)
                .collect();
            if matches.len() != 1 {
                return Err(format!(
                    "{} {}: expected one exact companion, found{}",
                    path.display(),
                    e.key,
                    matches.len()
                )
                .into());
            }
            let data = index.read(matches[0])?;
            let rw = rw4::parse(&data)?;
            let bound = links.bind_states(&rw, &data)?;
            let bindings_digest = binding_digest(&links, &bound)?;
            for s in &bound.states {
                if s.to_bytes() != rw.object(&data, s.section_index as u32).unwrap() {
                    return Err("compiled-state reserialization mismatch".into());
                }
            }
            let mut h = Fnv::new();
            h.word(links.materials.len() as u32);
            let mut local_textures = 0;
            for (m, r) in links.materials.iter().zip(&bound.material_ranges) {
                h.word(m.material_id);
                h.word(u32::from(m.state_count));
                h.word(m.textures.len() as u32);
                for t in &m.textures {
                    h.word(t.instance_id);
                    h.word(t.group_id);
                }
                h.word(r.start as u32);
                h.word(r.end as u32);
                local_textures += m.textures.len();
            }
            h.word(bound.states.len() as u32);
            for s in &bound.states {
                for v in [
                    s.section_index as u32,
                    s.primitive_type,
                    s.soft_dirty,
                    s.soft_delta,
                    s.hard_dirty,
                    s.hard_delta,
                    s.shader_id,
                ] {
                    h.word(v);
                }
                h.word(s.payload.len() as u32);
                h.bytes(&s.alignment_padding);
                h.bytes(&s.payload);
            }
            writeln!(
                out,
                "{}\t{}\t{}\t{}\t{}\t{:016X}\t{:016X}",
                path.display(),
                e.key,
                links.materials.len(),
                bound.states.len(),
                local_textures,
                h.0,
                bindings_digest.0
            )?;
            files += 1;
            materials += links.materials.len();
            states += bound.states.len();
            textures += local_textures;
            println!(
                "material-library={} {} records={} states={} textures={} exact=true",
                path.display(),
                e.key,
                links.materials.len(),
                bound.states.len(),
                local_textures
            );
        }
    }
    println!("MATERIAL_VERIFY packages={packages} libraries={files} materials={materials} states={states} texture_references={textures} exact=true");
    if files == 0 {
        return Err("no material libraries".into());
    }
    Ok(())
}
