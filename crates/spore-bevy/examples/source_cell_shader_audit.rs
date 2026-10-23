//! Resolve original compiled programs for an explicit static inspection context.
//! This is not evidence of the retail game's later material/state overrides.
use spore_core::{
    gmdl, gmdl_skin, prop, shader_binding, shader_pack,
    shader_selection::{self, SelectionContext},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("usage: source_cell_shader_audit <install> [GMDL packed instance hex]".into());
    }
    let instance = args
        .get(1)
        .map(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16))
        .transpose()?
        .unwrap_or(0x40617100780A9EAF);
    let root = Path::new(&args[0]).join("Data");
    let mut libraries = Vec::new();
    for name in ["PatchData", "Spore_Game", "Spore_Content", "Spore_Graphics"] {
        libraries.push(dbpf::PackageIndex::open(
            root.join(format!("{name}.package")),
        )?);
    }
    let read = |typ, instance| {
        libraries
            .iter()
            .find_map(|p| p.find_instance(typ, instance).map(|e| p.read(e)))
            .ok_or_else(|| format!("source resource missing {typ:08X}:{instance:016X}"))?
            .map_err(|e| e.to_string())
    };
    let bytes = read(0x00e6bce5, instance)?;
    let model = gmdl::parse(&bytes)?;
    let tail = gmdl_skin::parse(&bytes, &model)?.ok_or("no material inputs")?;
    let pack = shader_pack::parse(&read(0x0469a3f7, 0x4021200400000003)?)?;
    // 70E580 constructs the manager's fallback state with generic_static.
    // This audit explicitly selects that static context, with no game overrides.
    let builder = pack
        .builders
        .iter()
        .find(|b| b.header.shader_id() == 0x80000002 && b.header.name == "generic_static")
        .ok_or("missing measured static builder")?;
    let variant = builder
        .variants
        .iter()
        .find(|v| v.render_type == 0)
        .ok_or("missing type0 static builder")?;
    let lighting = prop::parse(&read(0x00b1b104, 0x40200100fced73a2)?)?;
    let coefficients = lighting
        .items
        .iter()
        .find(|i| i.id == 0x0100eac5)
        .ok_or("missing source light coefficients")?;
    if !lighting.exact || coefficients.prop_type != 0x32 || coefficients.values.len() != 25
        || !coefficients.values.iter().all(|v| matches!(v, prop::PropValue::ColorRGB(r,g,b) if r.is_finite() && g.is_finite() && b.is_finite()))
    { return Err("unestablished source lighting input".into()); }
    println!("CONTROLLED_STATIC_CONTEXT model={instance:016X} profile=3 builder=generic_static render_type=0 source_light=40200100FCED73A2 cell_stage=absent unpack_enabled=true retail_draw_claim=false");
    for (i, mesh) in model.meshes.iter().enumerate() {
        let material = tail.materials.get(i).ok_or("missing material slot")?;
        let mut data = BTreeMap::new();
        let mut samplers = BTreeSet::new();
        for item in &material.shader_data {
            data.insert(u16::try_from(item.id)?, item.payload.clone());
        }
        // Only presence is inspected for 219 in this builder; the actual
        // coefficient packing/rotation remains the next rendering gate.
        data.insert(0x219, vec![]);
        if let Some(textures) = data.get(&0x20d) {
            for texture in shader_binding::texture_bindings(textures)? {
                if texture.availability().is_some_and(|v| v != 0) {
                    samplers.insert(u16::try_from(texture.sampler)?);
                }
            }
        }
        let vb = model
            .vertex_buffers
            .get(mesh.vertex_index as usize)
            .ok_or("missing vertex buffer")?;
        let declaration = model
            .vertex_formats
            .get(vb.format_index as usize)
            .ok_or("missing declaration")?;
        // Native 760FB0 and 73EDF0 derive marker 22C from the actual saved
        // normal storage. Both measured default enable bytes are one.
        if declaration.native_normal_unpack_required(true) {
            data.insert(0x22c, vec![]);
        }
        let context = SelectionContext {
            declaration_features: declaration.native_shader_features()?,
            shader_data: &data,
            available_samplers: &samplers,
        };
        let vertex = shader_selection::select(&variant.vertex, &context)?;
        let pixel = shader_selection::select(&variant.pixel, &context)?;
        for (name, selected, table) in [
            ("vertex", vertex, &pack.compiled_vertex),
            ("pixel", pixel, &pack.compiled_pixel),
        ] {
            let program = table.lookup(&selected.fragment_ids)?;
            println!("SOURCE_STATIC_SELECTION mesh={i} stage={name} features={:08X} recipe={:02X?} cached={} bytecode_bytes={} uniforms={}",
                context.declaration_features, selected.fragment_ids, program.is_some(), program.map_or(0,|p|p.bytecode.len()), program.map_or(0,|p|p.uniforms.len()));
        }
    }
    Ok(())
}
