//! Local conditional direct-draw census verifier; stdout contains asset bytes.
//! Redirect output under ignored target/ and compare with the Python comparator.
use spore_core::{gmdl, gmdl_skin, gmdl_direct_draw_influences_v1};
use std::{env, fs, io::{self, BufWriter, Write}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let path = args.next().ok_or("usage: gmdl_direct_draw_census_verify_v1 <decoded-gmdl>")?;
    if args.next().is_some() { return Err("expected exactly one decoded GMDL path".into()); }
    let data = fs::read(path)?;
    let model = gmdl::parse(&data).map_err(|e| format!("GMDL parse: {e:?}"))?;
    let skin = gmdl_skin::parse(&data, &model).map_err(|e| format!("skin parse: {e:?}"))?
        .ok_or("missing skin tail")?;
    let mut out = BufWriter::new(io::stdout().lock());
    writeln!(out, "GMDL_DIRECT_DRAW_INFLUENCES_V1 {}", model.meshes.len())?;
    for mesh in 0..model.meshes.len() {
        let draw = gmdl_direct_draw_influences_v1::resolve(&data, &model, &skin, mesh,
            gmdl_direct_draw_influences_v1::DrawPath::DirectFresh769a80Unmodified)
            .map_err(|e| format!("mesh {mesh}: {e:?}"))?;
        writeln!(out, "M {} {} {} {} {}", draw.mesh_ordinal, draw.descriptor.first_word,
            draw.descriptor.bone_count, draw.base_offset_bytes, draw.referenced.len())?;
        for row in draw.referenced {
            writeln!(out, "V {} {} {} {} {} {} {} {} {}", row.vertex_index,
                row.register_offsets[0], row.register_offsets[1], row.register_offsets[2], row.register_offsets[3],
                row.weight_bytes[0], row.weight_bytes[1], row.weight_bytes[2], row.weight_bytes[3])?;
        }
    }
    out.flush()?;
    Ok(())
}
