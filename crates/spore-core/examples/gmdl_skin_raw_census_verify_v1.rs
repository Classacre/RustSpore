//! Emit raw influence bytes from an explicitly supplied decoded GMDL payload.
//! Compare the output with tools/gmdl_skin_raw_census_compare_v1.py.
use spore_core::{gmdl, gmdl_skin_raw_influences_v1::decode_range};
use std::{env, fs, io::{self, BufWriter, Write}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let path = args.next().ok_or("usage: gmdl_skin_raw_census_verify_v1 <decoded-gmdl>")?;
    if args.next().is_some() { return Err("expected exactly one decoded GMDL path".into()); }
    let data = fs::read(path)?;
    let model = gmdl::parse(&data).map_err(|e| format!("GMDL parse failed: {e:?}"))?;
    let mut out = BufWriter::new(io::stdout().lock());
    writeln!(out, "GMDL_RAW_INFLUENCES_V1 {}", model.meshes.len())?;
    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        let vb_index = usize::try_from(mesh.vertex_index).map_err(|_| "negative vertex buffer index")?;
        let vb = model.vertex_buffers.get(vb_index).ok_or("invalid vertex buffer index")?;
        let count = usize::try_from(vb.vertex_count).map_err(|_| "negative vertex count")?;
        let rows = decode_range(&data, &model, mesh_index, 0..count)
            .map_err(|e| format!("mesh {mesh_index}: {e:?}"))?;
        writeln!(out, "M {mesh_index} {vb_index} {count}")?;
        for row in rows {
            writeln!(out, "V {} {} {} {} {} {} {} {} {}", row.vertex_index,
                row.register_offsets[0], row.register_offsets[1], row.register_offsets[2], row.register_offsets[3],
                row.weight_bytes[0], row.weight_bytes[1], row.weight_bytes[2], row.weight_bytes[3])?;
        }
    }
    out.flush()?;
    Ok(())
}
