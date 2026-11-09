//! Hash-bound direction-glyph inspection of one synthetic 0473 deformation.
//! This is not a live creature, animation, palette, material or renderer match.
use bevy::{app::AppExit, prelude::*, render::view::screenshot::ScreenshotCaptured};
use sha2::{Digest, Sha256};
use std::{fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::Instant};

const EXPECTED_SHA: &str = "3393ad82bdb8b3ed8cf556bad192d49a0d8a28dc84e4759725b4ba33d055bd7f";
const SCHEMA: &str = "rustspore-conditional-0473-partial-geometry-v1";
const LABEL: &str = "0473 CONDITIONAL INSPECTION - SYNTHETIC MATRICES\nNORMALS: GREEN  TANGENTS: ORANGE\nNOT A LIVE POSE OR MATERIAL";

#[derive(Resource)]
struct Preview { positions: Vec<Vec3>, normals: Vec<Vec3>, tangents: Vec<Vec3>, indices: Vec<usize>, center: Vec3, radius: f32 }
#[derive(Component)] struct PreviewLabel;
#[derive(Resource)]
struct Capture { file: Option<File>, result: Arc<Mutex<Option<Result<(), String>>>>, started: Instant, dispatched: bool }

fn decode_document(value: &serde_json::Value) -> Result<Preview, String> {
    if value.get("schema").and_then(|x| x.as_str()) != Some(SCHEMA) { return Err("unexpected inspection schema".into()); }
    let deformation = value.get("conditional_synthetic_cpu_deformation").ok_or("missing synthetic deformation")?;
    let limits = deformation.get("limits").and_then(|x| x.as_str()).ok_or("missing deformation limits")?;
    if !limits.contains("synthetic") || !limits.contains("Normals and tangents are not deformed") || !limits.contains("live pose") { return Err("missing synthetic-pose limitations".into()); }
    let rows = deformation.get("position_words_hex").and_then(|x| x.as_array()).ok_or("missing position words")?;
    if rows.len() != 85 { return Err("expected 85 positions".into()); }
    let mut positions = Vec::with_capacity(85);
    for row in rows {
        let words = row.as_array().ok_or("invalid position row")?;
        if words.len() != 3 { return Err("expected three position words".into()); }
        let mut xyz = [0.0; 3];
        for (dst, word) in xyz.iter_mut().zip(words) {
            let word = word.as_str().filter(|x| x.len() == 8 && x.bytes().all(|c| c.is_ascii_hexdigit())).ok_or("invalid position word")?;
            *dst = f32::from_bits(u32::from_str_radix(word, 16).map_err(|_| "invalid position word")?);
            if !dst.is_finite() { return Err("nonfinite position".into()); }
        }
        positions.push(Vec3::from_array(xyz));
    }
    let values = value.get("conditional_same_object_export").and_then(|x| x.get("global_indices")).and_then(|x| x.as_array()).ok_or("missing triangle indices")?;
    if values.len() != 252 { return Err("expected 252 triangle indices".into()); }
    let indices: Vec<usize> = values.iter().map(|v| v.as_u64().and_then(|n| usize::try_from(n).ok()).filter(|&n| n < positions.len()).ok_or_else(|| "invalid triangle index".to_string())).collect::<Result<_,_>>()?;
    let (mut lo, mut hi) = (positions[0], positions[0]);
    for p in &positions[1..] { lo = lo.min(*p); hi = hi.max(*p); }
    let center = (lo + hi) * 0.5;
    let radius = (hi - lo).max_element();
    if !center.is_finite() || !radius.is_finite() || !(1.0e-6..=1.0e4).contains(&radius) { return Err("invalid inspection extent".into()); }
    let section = value.get("conditional_synthetic_cpu_directions").ok_or("missing synthetic directions")?;
    let limits = section.get("limits").and_then(|x| x.as_str()).ok_or("missing direction limits")?;
    if !limits.contains("synthetic matrices") || !limits.contains("not a live pose") { return Err("missing direction limitations".into()); }
    let parse = |key: &str| -> Result<Vec<Vec3>, String> {
        let rows = section.get(key).and_then(|x| x.as_array()).ok_or("missing direction rows")?;
        if rows.len() != 85 { return Err("expected 85 direction rows".into()); }
        rows.iter().map(|row| {
            let words = row.as_array().filter(|r| r.len()==3).ok_or("invalid direction row")?;
            let mut xyz=[0.;3];
            for (dst, word) in xyz.iter_mut().zip(words) {
                let text=word.as_str().filter(|x| x.len()==8 && x.bytes().all(|c| c.is_ascii_hexdigit())).ok_or("invalid direction word")?;
                *dst=f32::from_bits(u32::from_str_radix(text,16).map_err(|_| "invalid direction word")?);
            }
            let v=Vec3::from_array(xyz);
            if !v.is_finite() || !(0.01..=2.0).contains(&v.length()) { return Err("invalid direction magnitude".into()); }
            Ok(v)
        }).collect()
    };
    let normals=parse("normal_words_hex")?; let tangents=parse("tangent_words_hex")?;
    Ok(Preview { positions, normals, tangents, indices, center, radius })
}
fn decode_checked(bytes: &[u8]) -> Result<Preview, String> {
    if bytes.len() > 8 * 1024 * 1024 || format!("{:x}", Sha256::digest(bytes)) != EXPECTED_SHA { return Err("inspection SHA-256 mismatch".into()); }
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| format!("inspection JSON: {e}"))?;
    decode_document(&value)
}
fn read_preview(path: &Path) -> Result<Preview, String> {
    let file = File::open(path).map_err(|e| format!("inspection file: {e}"))?;
    let mut bytes = Vec::new();
    file.take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes).map_err(|e| format!("inspection read: {e}"))?;
    decode_checked(&bytes)
}
fn capture_destination(path: &Path) -> Result<(PathBuf, File), String> {
    let root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target")).map_err(|e| format!("target directory: {e}"))?;
    let parent = fs::canonicalize(path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."))).map_err(|e| format!("capture parent: {e}"))?;
    if !parent.starts_with(root) || !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")) { return Err("capture must be a new PNG under workspace target".into()); }
    let dest = parent.join(path.file_name().ok_or("missing capture filename")?);
    let file = OpenOptions::new().write(true).create_new(true).open(&dest).map_err(|e| format!("capture create: {e}"))?;
    Ok((dest, file))
}
pub fn run(artifact: &Path, shot: &Path) -> Result<(), String> {
    let preview = read_preview(artifact)?;
    let (destination, file) = capture_destination(shot)?;
    let result = Arc::new(Mutex::new(None));
    let mut app = App::new();
    crate::offscreen::configure(&mut app, 1280, 800);
    app.insert_resource(preview).insert_resource(Capture { file: Some(file), result: result.clone(), started: Instant::now(), dispatched: false })
        .add_systems(Startup, setup).add_systems(Update, (draw, capture));
    app.run();
    let outcome = result.lock().map_err(|_| "capture result lock poisoned".to_string())?.take();
    match outcome { Some(Ok(())) => { println!("Synthetic direction-glyph inspection written: {}", destination.display()); Ok(()) }, Some(Err(e)) => Err(format!("capture failed: {e}; output may be partial")), None => Err(format!("capture did not finish; {} may be partial", destination.display())) }
}
fn setup(mut commands: Commands, preview: Res<Preview>) {
    commands.spawn((Camera3d::default(), Transform::from_translation(preview.center + Vec3::new(2.0, 1.3, 2.6) * preview.radius).looking_at(preview.center, Vec3::Y)));
    commands.spawn((Text::new(LABEL), TextFont { font_size: 22.0, ..default() }, TextColor(Color::WHITE), Node { position_type: PositionType::Absolute, left: Val::Px(20.0), top: Val::Px(20.0), ..default() }, PreviewLabel));
}
fn draw(preview: Res<Preview>, mut gizmos: Gizmos) {
    for tri in preview.indices.chunks_exact(3) {
        for (a,b) in [(0,1),(1,2),(2,0)] { gizmos.line(preview.positions[tri[a]], preview.positions[tri[b]], Color::srgb(0.3,0.85,1.0)); }
    }
    for i in (0..preview.positions.len()).step_by(4) {
        let start=preview.positions[i]; let scale=preview.radius*0.12;
        gizmos.line(start,start+preview.normals[i]*scale,Color::srgb(0.2,1.0,0.35));
        gizmos.line(start,start+preview.tangents[i]*scale,Color::srgb(1.0,0.55,0.1));
    }
}
fn capture(mut commands: Commands, mut state: ResMut<Capture>, target: Res<crate::offscreen::OffscreenTarget>, mut readiness: ResMut<crate::offscreen::OffscreenReadiness>, labels: Query<&bevy::text::TextLayoutInfo, With<PreviewLabel>>, mut exit: MessageWriter<AppExit>) {
    if let Some(error) = readiness.failure() { *state.result.lock().unwrap() = Some(Err(error)); exit.write(AppExit::error()); return; }
    if state.started.elapsed().as_secs() > 120 { *state.result.lock().unwrap() = Some(Err("capture timeout".into())); exit.write(AppExit::error()); return; }
    if state.result.lock().unwrap().is_some() { exit.write(AppExit::Success); return; }
    let text_ready = labels.iter().count() == 1 && labels.iter().all(|layout| readiness.require_text(layout));
    if !state.dispatched && text_ready && readiness.ready() {
        let mut file = state.file.take().expect("exclusive capture file"); let result = state.result.clone();
        commands.spawn(target.screenshot()).observe(move |event: On<ScreenshotCaptured>| {
            let outcome = event.image.clone().try_into_dynamic().map_err(|e| format!("image conversion: {e}"))
                .and_then(|image| image.to_rgb8().write_to(&mut file, image::ImageFormat::Png).map_err(|e| format!("PNG: {e}")))
                .and_then(|_| file.flush().map_err(|e| format!("flush: {e}")))
                .and_then(|_| file.sync_all().map_err(|e| format!("sync: {e}")));
            *result.lock().unwrap() = Some(outcome);
        });
        state.dispatched = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_unpinned_and_checks_fixture() {
        assert!(decode_checked(b"{}").is_err());
        let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/creature-skin-synthetic-directions-inspection-20261109.json");
        if path.exists() {
            let bytes=fs::read(path).unwrap(); let p=decode_checked(&bytes).unwrap();
            assert_eq!((p.positions.len(),p.normals.len(),p.tangents.len(),p.indices.len()),(85,85,85,252));
            let mut v:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
            v["conditional_synthetic_cpu_directions"]["normal_words_hex"][0][0]=serde_json::json!("7f800000"); assert!(decode_document(&v).is_err());
            v["conditional_synthetic_cpu_directions"]["normal_words_hex"][0][0]=serde_json::json!("00000000"); v["conditional_same_object_export"]["global_indices"][0]=serde_json::json!(85); assert!(decode_document(&v).is_err());
        }
    }
}
