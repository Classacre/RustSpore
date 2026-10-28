//! Synthetic position diagnostic on a locally decoded GMDL; outputs contain asset bytes.
use spore_core::{gmdl, gmdl_skin, gmdl_direct_draw_influences_v1::DrawPath, gmdl_direct_draw_position_diagnostic_v1::{deform_positions, Matrix}};
use std::{fs, io::Write, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("target/source-cell/jawhead-decoded-v1.gmdl")?;
    let model = gmdl::parse(&data).map_err(|e| format!("model: {e:?}"))?;
    let skin = gmdl_skin::parse(&data, &model).map_err(|e| format!("skin: {e:?}"))?.ok_or("missing skin")?;
    let extent = skin.palettes.iter().try_fold(0usize, |max, p| {
        let end = usize::try_from(p.first_word).ok()?.checked_add(usize::try_from(p.bone_count).ok()?)?;
        Some(max.max(end))
    }).ok_or("palette extent")?;
    let id: Matrix = [[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.]];
    let output = Path::new("target/source-cell/gmdl-position-diagnostic-source-v1");
    fs::create_dir_all(output)?;
    for mode in ["identity", "translation"] {
        let mut palette = vec![id; extent];
        if mode == "translation" { for (i, m) in palette.iter_mut().enumerate() {
            m[0][3] = (i as f32 + 1.) * 0.125;
            m[1][3] = -(i as f32 + 1.) * 0.0625;
            m[2][3] = (i as f32 + 1.) * 0.03125;
        }}
        for mesh in 0..model.meshes.len() {
            let result = deform_positions(&data, &model, &skin, mesh, DrawPath::DirectFresh769a80Unmodified, &palette)
                .map_err(|e| format!("{mode} mesh {mesh}: {e:?}"))?;
            let mut file = fs::OpenOptions::new().write(true).create_new(true)
                .open(output.join(format!("{mode}-mesh-{mesh}.bin")))?;
            file.write_all(&result)?;
        }
    }
    println!("wrote synthetic diagnostic vertex-buffer copies to {}", output.display());
    Ok(())
}
