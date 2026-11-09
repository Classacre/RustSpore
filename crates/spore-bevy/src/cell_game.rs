//! Playable Cell-stage vertical slice with deterministic simulation and GPU visuals.
//! Retail content, editors and cross-stage progression are separate milestone gates.

#[path = "cell_kind0_inspection.rs"]
mod cell_kind0_inspection;
#[path = "cell_source_audio_preview_v1.rs"]
mod cell_source_audio_preview_v1;
#[path = "native_source_ui_fixture_v1.rs"]
mod native_source_ui_fixture_v1;
#[path = "cell_source_ui_five_v1.rs"]
mod cell_source_ui_five_v1;
#[path = "cell_effectmap_plane_v1.rs"]
mod cell_effectmap_plane_v1;
#[path = "cell_effect_plane_grid_v1.rs"]
mod cell_effect_plane_grid_v1;
#[path = "cell_effectmap_154_v1.rs"]
mod cell_effectmap_154_v1;
#[path = "cell_startup_source_normals_v1.rs"]
mod cell_startup_source_normals_v1;

use bevy::{
    audio::AddAudioSource,
    asset::{load_internal_asset, uuid_handle, RenderAssetUsages},
    camera::{visibility::{NoFrustumCulling, RenderLayers}, ScalingMode, Viewport},
    image::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode},
    mesh::{Indices, MeshVertexBufferLayoutRef},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d, PrimitiveTopology, RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureDimension, TextureFormat},
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
    shader::{Shader, ShaderRef},
};
use spore_core::cell::{
    CellBehavior, CellInput, CellSnapshot, CellState, CellWorld, Diet, FoodKind,
};
use spore_core::editor::{Creation, CreationKind, EditorSnapshot, MAX_SAVE_BYTES};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Resource)]
struct Game {
    world: CellWorld,
    seed: u64,
    diet: Diet,
    paused: bool,
    elapsed: f64,
    notice: String,
    rebuild: bool,
    save_path: PathBuf,
    shot: Option<PathBuf>,
    demo_seconds: Option<f64>,
    capture_frame: Option<u64>,
    frames: u64,
}

#[derive(Resource)]
struct Art {
    circle: Handle<Mesh>,
    ring: Handle<Mesh>,
    leaf: Handle<Mesh>,
    line: Handle<Mesh>,
    player: Handle<ColorMaterial>,
    predator: Handle<ColorMaterial>,
    prey: Handle<ColorMaterial>,
    plant: Handle<ColorMaterial>,
    meat: Handle<ColorMaterial>,
    membrane: Handle<ColorMaterial>,
    nucleus: Handle<ColorMaterial>,
    white: Handle<ColorMaterial>,
    dark: Handle<ColorMaterial>,
    rock: Handle<ColorMaterial>,
}

#[derive(Resource, Default)]
struct SceneObjects(HashMap<(u8, u32), Entity>);
#[derive(Resource, Default)]
struct AuthoredArt {
    creation: Option<Creation>,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<ColorMaterial>>,
}
/// Set once by capture after all required render and text assets are ready.
#[derive(Resource, Default)]
struct OffscreenSimulationReady(bool);

#[derive(Resource)]
struct SourceUiFixture { rgba:Vec<u8>, rectangles:Vec<([f32;4],[f32;4])>, five:bool }
#[derive(Resource)]
struct SourceEffectPlane { art:cell_effectmap_plane_v1::PlaneArt, rect:[f32;4], is_154:bool, hidden:bool }

#[derive(Resource)]
struct SourcePlayer {
    mesh: spore_core::mesh::MeshData,
    skin: crate::assets::TextureData,
    mips: Option<crate::assets::TextureMipChain>,
    sampler: ImageSamplerDescriptor,
    center: Vec3,
    scale: f32,
    rotation: Quat,
    kind: SourceKind,
    unorm_texture: bool,
    skin_lanes: Option<cell_kind0_inspection::SkinLanes>,
    material_s1: Option<crate::assets::TextureMipChain>,
    traced_basis: Option<(Vec<[f32;3]>,Vec<[f32;4]>)>,
    inspection_camera: bool,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
enum SourceKind { Cell, Creature, StartupCell }

#[derive(Resource)]
struct SourceRival(Vec<(u32,SourcePlayer)>);

#[derive(Resource)]
struct SourceRivalRender(Vec<(u32,SourcePlayerRender)>);

#[derive(Resource)]
struct SourcePlayerRender {
    mesh: Handle<Mesh>,
    material: SourcePlayerMaterial,
    center: Vec3,
    scale: f32,
    rotation: Quat,
}

#[derive(Clone)]
enum SourcePlayerMaterial {
    Pbr(Handle<StandardMaterial>),
    SavedBlend(Handle<ExtendedMaterial<StandardMaterial, SavedBlendExtension>>),
    FragmentPreview(Handle<FragmentPreviewMaterial>),
    SyntheticPalette(Handle<SyntheticPaletteMaterial>),
    StartupMaterial(Handle<StartupMaterial>),
    TracedMaterial(Handle<TracedMaterial>),
    InspectionMaterial(Handle<InspectionMaterial>),
}

const FRAGMENT_PREVIEW_SHADER: Handle<Shader> = uuid_handle!("de1cc589-f293-4a6b-8ef6-7247b9299f90");
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct FragmentPreviewMaterial {
    #[texture(0)] #[sampler(1)] texture: Handle<Image>,
}
impl Material for FragmentPreviewMaterial {
    fn vertex_shader() -> ShaderRef { FRAGMENT_PREVIEW_SHADER.into() }
    fn fragment_shader() -> ShaderRef { FRAGMENT_PREVIEW_SHADER.into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        let component = BlendComponent { src_factor: BlendFactor::SrcAlpha, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add };
        descriptor.fragment.as_mut().and_then(|f| f.targets.first_mut()).and_then(Option::as_mut).expect("fragment preview requires a color target").blend = Some(BlendState { color: component, alpha: component });
        descriptor.depth_stencil.as_mut().expect("fragment preview requires depth").depth_write_enabled = true;
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
#[derive(Resource)]
struct FragmentPreviewDiagnostic;
const STARTUP_MATERIAL_SHADER: Handle<Shader> = uuid_handle!("b224cf91-6716-4fc2-9c9e-813835c147de");
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct StartupMaterial { #[texture(0)] #[sampler(1)] stage0: Handle<Image>, #[texture(2)] #[sampler(3)] stage1: Handle<Image> }
impl Material for StartupMaterial {
 fn vertex_shader() -> ShaderRef { STARTUP_MATERIAL_SHADER.into() }
 fn fragment_shader() -> ShaderRef { STARTUP_MATERIAL_SHADER.into() }
 fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
 fn specialize(_: &MaterialPipeline, d:&mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _:MaterialPipelineKey<Self>)->Result<(),SpecializedMeshPipelineError>{
  let c=BlendComponent{src_factor:BlendFactor::SrcAlpha,dst_factor:BlendFactor::OneMinusSrcAlpha,operation:BlendOperation::Add};
  d.fragment.as_mut().and_then(|f|f.targets.first_mut()).and_then(Option::as_mut).expect("startup material requires color").blend=Some(BlendState{color:c,alpha:c});
  d.depth_stencil.as_mut().expect("startup material requires depth").depth_write_enabled=true;d.primitive.cull_mode=None;Ok(())
 }
}
const TRACED_MATERIAL_SHADER: Handle<Shader> = uuid_handle!("1276a147-507c-4e51-96a2-2a477dc7394f");
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TracedMaterial { #[texture(0)] #[sampler(1)] stage0: Handle<Image>, #[texture(2)] #[sampler(3)] stage1: Handle<Image> }
impl Material for TracedMaterial {
 fn vertex_shader()->ShaderRef{TRACED_MATERIAL_SHADER.into()} fn fragment_shader()->ShaderRef{TRACED_MATERIAL_SHADER.into()}
 fn alpha_mode(&self)->AlphaMode{AlphaMode::Blend}
 fn specialize(_: &MaterialPipeline,d:&mut RenderPipelineDescriptor,_:&MeshVertexBufferLayoutRef,_:MaterialPipelineKey<Self>)->Result<(),SpecializedMeshPipelineError>{let c=BlendComponent{src_factor:BlendFactor::SrcAlpha,dst_factor:BlendFactor::OneMinusSrcAlpha,operation:BlendOperation::Add};d.fragment.as_mut().and_then(|f|f.targets.first_mut()).and_then(Option::as_mut).expect("traced material requires color").blend=Some(BlendState{color:c,alpha:c});d.depth_stencil.as_mut().expect("traced material requires depth").depth_write_enabled=true;d.primitive.cull_mode=None;Ok(())}
}
const INSPECTION_CAMERA_SHADER: Handle<Shader> = uuid_handle!("60028218-8323-4cbc-8c48-0380743817d3");
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct InspectionMaterial { #[texture(0)] #[sampler(1)] stage0: Handle<Image>, #[texture(2)] #[sampler(3)] stage1: Handle<Image> }
impl Material for InspectionMaterial {
 fn vertex_shader()->ShaderRef{INSPECTION_CAMERA_SHADER.into()} fn fragment_shader()->ShaderRef{INSPECTION_CAMERA_SHADER.into()}
 fn alpha_mode(&self)->AlphaMode{AlphaMode::Opaque}
 fn specialize(_: &MaterialPipeline,d:&mut RenderPipelineDescriptor,_:&MeshVertexBufferLayoutRef,_:MaterialPipelineKey<Self>)->Result<(),SpecializedMeshPipelineError>{d.depth_stencil.as_mut().expect("inspection material requires depth").depth_write_enabled=true;d.primitive.cull_mode=None;Ok(())}
}
const EFFECT_PLANE_SHADER: Handle<Shader> = uuid_handle!("80c18475-1733-4226-9e2a-a588c92d3531");
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct EffectPlaneMaterial { #[texture(0)] #[sampler(1)] texture: Handle<Image> }
impl Material for EffectPlaneMaterial {
    fn vertex_shader() -> ShaderRef { EFFECT_PLANE_SHADER.into() }
    fn fragment_shader() -> ShaderRef { EFFECT_PLANE_SHADER.into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        let component = BlendComponent { src_factor: BlendFactor::SrcAlpha, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add };
        descriptor.fragment.as_mut().and_then(|f| f.targets.first_mut()).and_then(Option::as_mut).expect("plane requires a color target").blend = Some(BlendState { color: component, alpha: component });
        descriptor.depth_stencil.as_mut().expect("plane requires depth").depth_write_enabled = true;
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
const SYNTHETIC_PALETTE_SHADER: Handle<Shader> = uuid_handle!("e028d231-8ca6-41ab-9ee0-559ed4e272de");
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SyntheticPaletteMaterial { #[texture(0)] #[sampler(1)] texture: Handle<Image> }
impl Material for SyntheticPaletteMaterial {
    fn vertex_shader() -> ShaderRef { SYNTHETIC_PALETTE_SHADER.into() }
    fn fragment_shader() -> ShaderRef { SYNTHETIC_PALETTE_SHADER.into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        let component = BlendComponent { src_factor: BlendFactor::SrcAlpha, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add };
        descriptor.fragment.as_mut().and_then(|f| f.targets.first_mut()).and_then(Option::as_mut).expect("synthetic palette requires a color target").blend = Some(BlendState { color: component, alpha: component });
        descriptor.depth_stencil.as_mut().expect("synthetic palette requires depth").depth_write_enabled = true;
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
#[derive(Resource)] struct SyntheticPaletteDiagnostic;

/// Controlled candidate-state inspection: PBR shading and Bevy ordering remain.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SavedBlendExtension {}
impl MaterialExtension for SavedBlendExtension {
    fn specialize(
        _: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let component = BlendComponent { src_factor: BlendFactor::SrcAlpha, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add };
        let target = descriptor.fragment.as_mut().and_then(|f| f.targets.first_mut()).and_then(Option::as_mut).expect("saved-blend diagnostic requires a color target");
        target.blend = Some(BlendState { color: component, alpha: component });
        descriptor.depth_stencil.as_mut().expect("saved-blend diagnostic requires depth").depth_write_enabled = true;
        Ok(())
    }
}
#[derive(Resource)]
struct SavedBlendDiagnostic;

/// Inspection-only basis: source (x,y,z) to screen-facing (y,z,x).
fn creature_inspection_rotation() -> Quat {
    Quat::from_xyzw(-0.5, -0.5, -0.5, 0.5)
}

fn inspection_mip_lod_max(filter: i32, levels: usize) -> f32 {
    if filter == 0 { 0.0 } else { levels.saturating_sub(1) as f32 }
}

fn creature_mip_chain_required(texture_type: u32, filter: i32) -> Result<bool, String> {
    match (texture_type, filter) {
        (crate::assets::RASTER, _) => Ok(true),
        (crate::assets::RW4, 0) => Ok(false),
        (crate::assets::RW4, _) => Err("source Creature RW4 mip chain unsupported when mip filtering is enabled".into()),
        _ => Err("source Creature texture type unsupported".into()),
    }
}

fn inspection_image(texture: &crate::assets::TextureData, mips: Option<&crate::assets::TextureMipChain>, sampler: &ImageSamplerDescriptor) -> Image {
    let mut image = Image::new(Extent3d { width: texture.width, height: texture.height, depth_or_array_layers: 1 }, TextureDimension::D2, texture.rgba.clone(), TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    if let Some(mips) = mips {
        image.texture_descriptor.mip_level_count = mips.levels.len() as u32;
        image.data = Some(mips.levels.iter().flat_map(|level| level.rgba.iter().copied()).collect());
    }
    image.sampler = ImageSampler::Descriptor(sampler.clone());
    image
}

fn apply_startup_unorm(image: &mut Image, kind: SourceKind, enabled: bool) -> bool {
    if kind == SourceKind::StartupCell && enabled {
        image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
        true
    } else { false }
}

fn load_source_creature(install: &Path, instance: u64, exact_content_only: bool, renderer: &str) -> Result<(spore_core::mesh::MeshData, crate::assets::TextureData, Option<crate::assets::TextureMipChain>, ImageSamplerDescriptor), String> {
    let base = install.join("Data/Spore_Content.package");
    let paths = crate::assets::PackageSet::discover(install).paths().to_vec();
    let source = crate::assets::AssetSource::open(&base)?;
    let source = if exact_content_only { source } else { source.with_set(crate::assets::PackageSet::from_paths(paths.clone())) };
    let asset = source.authored_creature_asset(instance, 0).map_err(|e| e.to_string())?;
    if exact_content_only {
        let key = |instance_id, type_id, group_id| dbpf::native_index::NativeResourceKey { instance_id, type_id, group_id };
        if asset.reference.creature != key(0xe219e5d3, 0x0f43029a, 0x40616200)
            || asset.reference.prop != key(0xe219e5d3, 0x00b1b104, 0x40616200)
            || asset.reference.model != key(0xe219e5d3, 0x00e6bce5, 0x40617100)
            || asset.texture_key != key(0xe219e5d3, 0x2f4e681c, 0x40612900) {
            return Err("startup source art resolved outside the admitted content keys".into());
        }
    }
    let tex = &asset.texture;
    if tex.width == 0 || tex.height == 0 || tex.width > 4096 || tex.height > 4096
        || (tex.width as usize).checked_mul(tex.height as usize).and_then(|n| n.checked_mul(4)) != Some(tex.rgba.len()) {
        return Err("source Creature texture dimensions or decoded extent unsupported".into());
    }
    let binding = asset.texture_binding;
    if binding.availability() != Some(1) { return Err("source sampler unavailable".into()); }
    let address = match binding.address_mode() {
        Some(1) => ImageAddressMode::Repeat, Some(2) => ImageAddressMode::MirrorRepeat,
        Some(3) => ImageAddressMode::ClampToEdge, _ => return Err("source sampler addressing unsupported".into()),
    };
    let filter = match binding.min_mag_filter() {
        Some(1) => ImageFilterMode::Nearest, Some(2) => ImageFilterMode::Linear,
        _ => return Err("source sampler filtering unsupported".into()),
    };
    let mip = match binding.mip_filter() {
        Some(0 | 1) => ImageFilterMode::Nearest, Some(2) => ImageFilterMode::Linear,
        _ => return Err("source sampler mip filtering unsupported".into()),
    };
    let mip_filter = binding.mip_filter().unwrap();
    let mips = if creature_mip_chain_required(asset.texture_key.type_id, mip_filter)? {
        let chain = source.authored_creature_raster_mips(asset.texture_key)?;
        let first = chain.levels.first().ok_or("source Creature mip chain empty")?;
        if first.width != tex.width || first.height != tex.height || first.rgba != tex.rgba {
            return Err("source Creature base mip differs from texture".into());
        }
        Some(chain)
    } else { None };
    let sampler = ImageSamplerDescriptor { address_mode_u: address, address_mode_v: address, address_mode_w: address,
        min_filter: filter, mag_filter: filter, mipmap_filter: mip, lod_min_clamp: 0.0,
        lod_max_clamp: inspection_mip_lod_max(mip_filter, mips.as_ref().map_or(1, |chain| chain.levels.len())), ..default() };
    let lookup = crate::assets::PackageSet::from_paths(paths);
    for (role, key) in [("creature", asset.reference.creature), ("prop", asset.reference.prop),
        ("model", asset.reference.model), ("texture", asset.texture_key)] {
        let path = if source.resource_keys().any(|k| k == key) { base.clone() } else {
            lookup.locate(key.type_id, key.scoped_instance()).ok_or("source resource provenance unavailable")?
        };
        println!("CELL_SOURCE_CREATURE_RESOURCE role={role} package={} key={key}", path.display());
    }
    println!("CELL_SOURCE_CREATURE_INSPECTION instance={instance:016X} assignment=controlled_prototype_player renderer={renderer} static=true original_animation=false runtime_selection=unverified overlay=inspection-order basis=source_y_z_x_inference mip_chain={} levels={}", if mips.is_some() { "declared" } else { "base_only_filter_disabled" }, mips.as_ref().map_or(1, |chain| chain.levels.len()));
    Ok((asset.mesh, asset.texture, mips, sampler))
}

/// Inspection only: exact package payload, caller admits stable files across reads.
fn load_startup_sampler1(install:&Path)->Result<crate::assets::TextureMipChain,String>{
 use dbpf::native_index::{NativePackageIndex,NativeResourceKey};use sha2::{Digest,Sha256};
 let path=install.join("Data/Spore_Content.package");
 {use std::io::Read;let mut file=std::fs::File::open(&path).map_err(|e|e.to_string())?;let mut bytes=[0;dbpf::HEADER_SIZE];file.read_exact(&mut bytes).map_err(|e|e.to_string())?;let h=dbpf::DbpfHeader::parse(&bytes).map_err(|e|e.to_string())?;if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("startup sampler1 index exceeds bound".into())}}
 let key=NativeResourceKey{instance_id:0xe219e5d3,type_id:0x2f4e681c,group_id:0x40612901};
 let read=||->Result<Vec<u8>,String>{let idx=NativePackageIndex::open(&path).map_err(|e|e.to_string())?;let ent=idx.index().find_unique(key).map_err(|e|e.to_string())?.ok_or("startup sampler1 absent")?;
  if ent.stored_size.max(ent.decompressed_size)>16*1024*1024{return Err("startup sampler1 exceeds bound".into())}let raw=idx.read(ent).map_err(|e|e.to_string())?;
  if format!("{:x}",Sha256::digest(&raw))!="ceecebb4937eef972474e222dc2069238ce171f8f0a97ab89a8981328da96150"{return Err("startup sampler1 hash mismatch".into())}Ok(raw)};
 let raw=read()?;let parsed=spore_core::raster::parse(&raw).map_err(|e|e.to_string())?;
 if parsed.width!=256||parsed.height!=256||parsed.format!=spore_core::raster::FORMAT_RAW_RGBA{return Err("startup sampler1 format unsupported".into())}
 let chain=crate::assets::decode_standalone_raster_mips(&raw)?;if chain.levels.len()!=9||chain.levels.iter().enumerate().any(|(i,l)|l.width!=(256u32>>i).max(1)||l.height!=(256u32>>i).max(1)){return Err("startup sampler1 mips unsupported".into())}
 if raw!=read()?{return Err("startup sampler1 changed during inspection".into())}Ok(chain)
}
/// The candidate is a controlled seed-1 source-data join, not an observed actor.
fn admit_startup_triple(triple: [u32; 3]) -> Result<(), String> {
    if triple != [0xe219e5d3, 0x3d97a8e4, 0x40616200] {
        return Err(format!("unsupported controlled startup triple {triple:08X?}"));
    }
    Ok(())
}

/// Admit only the exact decoded resources from the content package. The caller
/// must keep package files stable throughout inspection and subsequent loading.
fn verify_startup_art_content(install: &Path) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
    let index = NativePackageIndex::open(&install.join("Data/Spore_Content.package"))
        .map_err(|e| e.to_string())?;
    for (kind, group, expected) in [
        (0x0f43029a, 0x40616200, "dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137"),
        (0x00b1b104, 0x40616200, "563e75d585b589631dba74a5a6bfa154140cf583cad0caff4ddc7637f619eba0"),
        (0x00e6bce5, 0x40617100, "95ba0f5e5e71761ccf0f2ef919cce9bcc8d40303be4b1b7a9fa954a7c7cb8ff4"),
        (0x2f4e681c, 0x40612900, "5c7889ac480c635daca5b27f7ae38ee0fc4fac5fc47b9040809f481354c5f89c"),
    ] {
        let key = NativeResourceKey { instance_id: 0xe219e5d3, type_id: kind, group_id: group };
        let entry = index.index().find_unique(key).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("startup source resource missing {key}"))?;
        if entry.stored_size > 16*1024*1024 || entry.decompressed_size > 16*1024*1024 {
            return Err(format!("startup source resource exceeds bound {key}"));
        }
        let bytes = index.read(entry).map_err(|e| e.to_string())?;
        if bytes.len() != entry.decompressed_size as usize || format!("{:x}", Sha256::digest(&bytes)) != expected {
            return Err(format!("startup source resource hash or length mismatch {key}"));
        }
    }
    Ok(())
}

#[derive(Component)]
struct Hud;
#[derive(Component)]
struct Banner;
#[derive(Component)]
struct Meter(u8);

fn parse_source_rival_roster(text:&str)->Result<Vec<u32>,String>{
    let fields:Vec<_>=text.split(',').collect();
    if fields.is_empty()||fields.len()>8||fields.iter().any(|s|s.len()!=8||!s.bytes().all(|b|b.is_ascii_hexdigit())){return Err("source rival roster requires 1 to 8 comma-separated eight-digit hex IDs".into())}
    let ids=fields.into_iter().map(|s|u32::from_str_radix(s,16).map_err(|_|"invalid source rival ID".to_string())).collect::<Result<Vec<_>,_>>()?;
    let mut seen=HashSet::new();if ids.iter().any(|id|!seen.insert(*id)){return Err("source rival roster contains duplicate IDs".into())}Ok(ids)
}
fn parse_effect_plane_rect(text:&str)->Result<[f32;4],String>{
    let fields:Vec<_>=text.split(',').collect();if fields.len()!=4||fields.iter().any(|s|s.len()!=8||!s.bytes().all(|b|b.is_ascii_hexdigit())){return Err("effect plane rect requires four f32 hex words: center x,y,width,height".into())}
    let values:Vec<_>=fields.into_iter().map(|s|u32::from_str_radix(s,16).map(f32::from_bits).map_err(|_|"invalid effect plane rect".to_string())).collect::<Result<_,_>>()?;
    let r:[f32;4]=values.try_into().map_err(|_|"invalid effect plane rect")?;
    if r.iter().any(|v|!v.is_finite())||r[0].abs()>1000.||r[1].abs()>1000.||r[2]<=0.||r[3]<=0.||r[2]>1000.||r[3]>1000.{return Err("effect plane rect outside admitted bounds".into())}Ok(r)
}
pub fn run(args: &[String]) -> Result<(), String> {
    let mut seed = 14;
    let mut diet = Diet::Herbivore;
    let mut explicit_diet = false;
    let mut creation_path = None;
    let mut shot = None;
    let mut demo_seconds = None;
    let mut source_install: Option<PathBuf> = None;
    let mut source_id: Option<u32> = None;
    let mut rival_id: Option<u32> = None;
    let mut rival_roster: Option<Vec<u32>> = None;
    let (mut effect_plane_install,mut effect_plane_rect,mut effect_plane_grid)=(None,None,None);
    let mut effect_plane_admitted=false;
    let (mut effect_154_install,mut effect_154_rect)=(None,None);
    let (mut effect_154_admitted,mut effect_154_hidden)=(false,false);
    let mut startup_install: Option<PathBuf> = None;
    let mut startup_stable_admitted = false;
    let mut startup_source_normals = false;
    let mut startup_unorm_texture = false;
    let mut startup_material_preview = false;
    let mut startup_traced_material = false;
    let mut startup_inspection_camera = false;
    let mut creature_install: Option<PathBuf> = None;
    let mut creature_instance: Option<u64> = None;
    let mut audio_package: Option<PathBuf> = None;
    let mut audio_key: Option<u64> = None;
    let mut audio_sha: Option<String> = None;
    let mut event_package: Option<PathBuf> = None;
    let mut event_key: Option<u64> = None;
    let mut event_sha: Option<String> = None;
    let mut event_row: Option<usize> = None;
    let mut event_loader_admitted = false;
    let mut offscreen = false;
    let (mut ui_install, mut ui_layout, mut ui_atlas, mut ui_fixture, mut ui_clip, mut ui_half) = (None, None, None, None, None, None);
    let (mut five_layout,mut five_atlas,mut five_span,mut five_origin,mut five_state,mut five_mask)=(None,None,None,None,None,None);
    let mut five_cursor=None;
    let mut five_admitted=false;
    let mut saved_blend=false;
    let mut fragment_preview=false;
    let mut synthetic_palette=false;
    let mut save_path = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustSpore")
        .join("saves")
        .join("cell.rsp");
    let mut i = 0;
    while i < args.len() {
        let value = || {
            args.get(i + 1)
                .ok_or_else(|| format!("{} requires a value", args[i]))
        };
        if args[i] == "--admit-event-loader-complete-unchanged-and-stable-packages" {
            if event_loader_admitted { return Err("repeated event loader admission".into()); }
            event_loader_admitted = true; i += 1; continue;
        }
        if args[i] == "--admit-startup-stable-packages" {
            if startup_stable_admitted { return Err("repeated startup stable-package admission".into()); }
            startup_stable_admitted = true; i += 1; continue;
        }
        if args[i] == "--startup-source-normals" {
            if startup_source_normals { return Err("repeated startup source-normal option".into()); }
            startup_source_normals = true; i += 1; continue;
        }
        if args[i] == "--startup-inspection-camera" {if startup_inspection_camera{return Err("repeated startup inspection camera".into())}startup_inspection_camera=true;i+=1;continue;}
        if args[i] == "--startup-traced-material" {if startup_traced_material{return Err("repeated startup traced material".into())}startup_traced_material=true;i+=1;continue;}
        if args[i] == "--startup-material-preview" {
            if startup_material_preview { return Err("repeated startup material preview".into()); }
            startup_material_preview=true;i+=1;continue;
        }
        if args[i] == "--startup-unorm-texture" {
            if startup_unorm_texture { return Err("repeated startup unorm-texture option".into()); }
            startup_unorm_texture = true; i += 1; continue;
        }
        if args[i] == "--offscreen" {
            offscreen = true;
            i += 1;
            continue;
        }
        if args[i] == "--source-kind0-saved-blend" {
            if saved_blend { return Err("repeated source kind-0 saved-blend diagnostic".into()); }
            saved_blend=true; i+=1; continue;
        }
        if args[i] == "--source-kind0-fragment-preview" {
            if fragment_preview { return Err("repeated source kind-0 fragment preview".into()); }
            fragment_preview=true; i+=1; continue;
        }
        if args[i] == "--source-kind0-synthetic-palette" {
            if synthetic_palette { return Err("repeated source kind-0 synthetic palette".into()); }
            synthetic_palette=true; i+=1; continue;
        }
        if args[i]=="--admit-five-ui-stable-packages-one-dispatch-no-other-mutation" {
            if five_admitted{return Err("repeated five UI admission".into())}five_admitted=true;i+=1;continue;
        }
        if args[i]=="--admit-effect-plane-stable-packages" {
            if effect_plane_admitted{return Err("repeated effect-plane admission".into())}effect_plane_admitted=true;i+=1;continue;
        }
        if args[i]=="--admit-effect-154-stable-packages" {if effect_154_admitted{return Err("repeated effect-154 admission".into())}effect_154_admitted=true;i+=1;continue;}
        if args[i]=="--source-effect-154-no-effect" {if effect_154_hidden{return Err("repeated effect-154 control".into())}effect_154_hidden=true;i+=1;continue;}
        match args[i].as_str() {
            "--source-ui-five-layout" if five_layout.is_none()=>five_layout=Some(PathBuf::from(value()?)),
            "--source-ui-five-atlas" if five_atlas.is_none()=>five_atlas=Some(PathBuf::from(value()?)),
            "--source-ui-five-span" if five_span.is_none()=>five_span=Some(value()?.to_owned()),
            "--source-ui-five-origin" if five_origin.is_none()=>five_origin=Some(value()?.to_owned()),
            "--source-ui-five-state" if five_state.is_none()=>five_state=Some(value()?.to_owned()),
            "--source-ui-five-mask" if five_mask.is_none()=>five_mask=Some(value()?.to_owned()),
            "--source-ui-five-cursor" if five_cursor.is_none()=>five_cursor=Some(value()?.to_owned()),
            "--source-effect-plane-install" if effect_plane_install.is_none()=>effect_plane_install=Some(PathBuf::from(value()?)),
            "--source-effect-plane-rect" if effect_plane_rect.is_none()=>effect_plane_rect=Some(value()?.to_owned()),
            "--source-effect-plane-grid" if effect_plane_grid.is_none()=>effect_plane_grid=Some(value()?.to_owned()),
            "--source-effect-154-install" if effect_154_install.is_none()=>effect_154_install=Some(PathBuf::from(value()?)),
            "--source-effect-154-rect" if effect_154_rect.is_none()=>effect_154_rect=Some(value()?.to_owned()),
            "--source-ui-install" if ui_install.is_none() => ui_install=Some(PathBuf::from(value()?)),
            "--source-ui-layout" if ui_layout.is_none() => ui_layout=Some(PathBuf::from(value()?)),
            "--source-ui-atlas" if ui_atlas.is_none() => ui_atlas=Some(PathBuf::from(value()?)),
            "--source-ui-fixture" if ui_fixture.is_none() => ui_fixture=Some(PathBuf::from(value()?)),
            "--source-ui-clip" if ui_clip.is_none() => ui_clip=Some(value()?.to_owned()),
            "--source-ui-half-pixel" if ui_half.is_none() => ui_half=Some(value()?.to_owned()),
            "--source-event-package" if event_package.is_none() => event_package=Some(PathBuf::from(value()?)),
            "--source-event-key" if event_key.is_none() => {
                let text=value()?;
                if text.len()!=16 || !text.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("source event key must be exactly 16 hexadecimal digits".into())}
                event_key=Some(u64::from_str_radix(text,16).map_err(|_|"invalid source event key")?);
            },
            "--source-event-sha256" if event_sha.is_none() => event_sha=Some(value()?.to_owned()),
            "--source-event-row" if event_row.is_none() => event_row=Some(value()?.parse().map_err(|_|"source event row must be a decimal usize")?),
            "--source-audio-package" if audio_package.is_none() => audio_package=Some(PathBuf::from(value()?)),
            "--source-audio-key" if audio_key.is_none() => {
                let text=value()?.trim();
                if text.len()!=16 || !text.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("source audio key must be exactly 16 hexadecimal digits".into())}
                audio_key=Some(u64::from_str_radix(text,16).map_err(|_|"invalid source audio key")?);
            },
            "--source-audio-sha256" if audio_sha.is_none() => audio_sha=Some(value()?.to_owned()),
            "--source-startup-install" if startup_install.is_none() => startup_install = Some(PathBuf::from(value()?)),
            "--source-startup-install" => return Err("repeated --source-startup-install".into()),
            "--source-cell-install" => source_install = Some(PathBuf::from(value()?)),
            "--source-creature-install" => creature_install = Some(PathBuf::from(value()?)),
            "--source-creature-instance" => {
                let text = value()?.trim();
                let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
                if text.len() != 16 { return Err("source Creature instance must be exactly 16 hexadecimal digits".into()); }
                creature_instance = Some(u64::from_str_radix(text, 16).map_err(|_| "source Creature instance must be exactly 16 hexadecimal digits".to_string())?);
            }
            "--source-rival-cell-id" => {
                if rival_id.is_some(){return Err("repeated --source-rival-cell-id".into())}
                let text = value()?.trim();
                let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
                rival_id = Some(u32::from_str_radix(text, 16).map_err(|_| "source rival Cell ID must be hexadecimal u32".to_string())?);
            }
            "--source-rival-cell-roster" => {
                if rival_roster.is_some(){return Err("repeated --source-rival-cell-roster".into())}
                rival_roster=Some(parse_source_rival_roster(value()?)?);
            }
            "--source-cell-id" => {
                let text = value()?.trim();
                let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
                source_id = Some(u32::from_str_radix(text, 16).map_err(|_| "source Cell ID must be hexadecimal u32".to_string())?);
            }
            "--seed" => seed = value()?.parse().map_err(|_| "invalid seed".to_string())?,
            "--diet" => {
                explicit_diet = true;
                diet = match value()?.as_str() {
                    "herbivore" => Diet::Herbivore,
                    "carnivore" => Diet::Carnivore,
                    "omnivore" => Diet::Omnivore,
                    _ => return Err("diet must be herbivore, carnivore or omnivore".into()),
                }
            }
            "--creation" => creation_path = Some(PathBuf::from(value()?)),
            "--shot" => shot = Some(PathBuf::from(value()?)),
            "--demo" => {
                let seconds: f64 = value()?
                    .parse()
                    .map_err(|_| "invalid demo duration".to_string())?;
                if !seconds.is_finite() || seconds <= 0.0 || seconds > 600.0 {
                    return Err("demo duration must be in (0, 600] seconds".into());
                }
                demo_seconds = Some(seconds);
            }
            "--save-file" => save_path = PathBuf::from(value()?),
            arg => return Err(format!("unknown Cell option {arg}")),
        }
        i += 2;
    }
    if startup_install.is_some() != startup_stable_admitted { return Err("startup source art requires --source-startup-install and --admit-startup-stable-packages together".into()); }
    if startup_source_normals && startup_install.is_none() { return Err("--startup-source-normals requires admitted startup source art".into()); }
    if startup_inspection_camera && !startup_traced_material {return Err("startup inspection camera requires --startup-traced-material".into())}
    if startup_traced_material && !startup_material_preview {return Err("startup traced material requires --startup-material-preview".into())}
    if startup_material_preview && (!offscreen || startup_install.is_none() || saved_blend || fragment_preview || synthetic_palette || startup_source_normals || startup_unorm_texture) {return Err("startup material preview requires admitted offscreen startup art and excludes other material/normal previews".into())}
    if startup_unorm_texture && startup_install.is_none() { return Err("--startup-unorm-texture requires admitted startup source art".into()); }
    if startup_install.is_some() && (seed != 1 || source_install.is_some() || source_id.is_some()
        || creature_install.is_some() || creature_instance.is_some() || creation_path.is_some()
        || rival_id.is_some() || rival_roster.is_some()) {
        return Err("startup source art requires --seed 1 and excludes other source art, rivals and creation".into());
    }
    if source_install.is_some() != source_id.is_some() {
        return Err("--source-cell-install and --source-cell-id must be provided together".into());
    }
    if saved_blend && (!offscreen || source_id != Some(cell_kind0_inspection::CELL_ID) || source_install.is_none()) {
        return Err("source kind-0 saved-blend diagnostic requires offscreen and the explicit FD24574B source player".into());
    }
    if fragment_preview && (!offscreen || source_id != Some(cell_kind0_inspection::CELL_ID) || source_install.is_none()) {
        return Err("source kind-0 fragment preview requires offscreen and the explicit FD24574B source player".into());
    }
    if fragment_preview && saved_blend { return Err("source kind-0 preview modes are mutually exclusive".into()); }
    if synthetic_palette && !fragment_preview { return Err("source kind-0 synthetic palette requires --source-kind0-fragment-preview".into()); }
    if rival_id.is_some() && rival_roster.is_some(){return Err("source rival ID and roster are mutually exclusive".into())}
    if (rival_id.is_some()||rival_roster.is_some()) && source_install.is_none() {
        return Err("source rival selection requires --source-cell-install and --source-cell-id".into());
    }
    let event_count=usize::from(event_package.is_some())+usize::from(event_key.is_some())+usize::from(event_sha.is_some())+usize::from(event_row.is_some());
    let audio_count=usize::from(audio_package.is_some())+usize::from(audio_key.is_some())+usize::from(audio_sha.is_some());
    let event_mode=event_count!=0 || event_loader_admitted;
    if event_mode {
        if event_count!=4 || !event_loader_admitted || audio_package.is_none() || audio_sha.is_none() || audio_key.is_some() {
            return Err("event preview requires event package, key, SHA-256, row, loader admission, and source audio package and SHA-256; source audio key is incompatible".into());
        }
    } else if audio_count!=0 && audio_count!=3 {return Err("--source-audio-package, --source-audio-key and --source-audio-sha256 must be provided together".into())}
    if offscreen && (audio_count!=0 || event_mode) {return Err("source audio preview is unavailable with --offscreen".into())}
    let audio_preview=if event_mode {
        Some(cell_source_audio_preview_v1::load_event(event_package.as_ref().unwrap(),event_key.unwrap(),event_sha.as_ref().unwrap(),event_row.unwrap(),audio_package.as_ref().unwrap(),audio_sha.as_ref().unwrap())?)
    } else { match (&audio_package,audio_key,&audio_sha) {
        (Some(package),Some(key),Some(sha))=>Some(cell_source_audio_preview_v1::load(package,key,sha)?),
        _=>None,
    }};
    if creature_install.is_some() != creature_instance.is_some() {
        return Err("--source-creature-install and --source-creature-instance must be provided together".into());
    }
    if source_install.is_some() && creature_install.is_some() {
        return Err("source Cell and source Creature inspection art cannot be combined".into());
    }
    if (source_install.is_some() || creature_install.is_some()) && creation_path.is_some() {
        return Err("source inspection art cannot be combined with --creation".into());
    }
    if offscreen && shot.is_none() {
        return Err("--offscreen requires --shot <new PNG>".into());
    }
    let plane_requested=effect_plane_install.is_some()||effect_plane_rect.is_some()||effect_plane_grid.is_some()||effect_plane_admitted;
    let effect_154_requested=effect_154_install.is_some()||effect_154_rect.is_some()||effect_154_admitted||effect_154_hidden;
    if startup_inspection_camera && effect_154_requested {return Err("startup inspection camera excludes source effect 154".into())}
    if plane_requested&&(!offscreen||effect_plane_install.is_none()||(effect_plane_rect.is_some()==effect_plane_grid.is_some())||!effect_plane_admitted){return Err("effect plane requires --offscreen, install, exactly one explicit world rect or grid, and stable-package admission".into())}
    if effect_154_requested&&(!offscreen||effect_154_install.is_none()||effect_154_rect.is_none()||!effect_154_admitted||plane_requested){return Err("effect 154 requires --offscreen, install, explicit world rect and stable-package admission; excludes other plane".into())}
    let effect_plane=if plane_requested{let rect=if let Some(grid)=effect_plane_grid.as_deref(){let r=cell_effect_plane_grid_v1::parse_and_project(grid)?;println!("CELL_EFFECT_PLANE_GRID supplied={grid} identity_basis=true x87=027f mxcsr=1f80 source_state=unverified live_selection=false");r}else{parse_effect_plane_rect(effect_plane_rect.as_deref().unwrap())?};let art=cell_effectmap_plane_v1::load(effect_plane_install.as_deref().unwrap())?;Some(SourceEffectPlane{art,rect,is_154:false,hidden:false})}else if effect_154_requested{let rect=parse_effect_plane_rect(effect_154_rect.as_deref().unwrap())?;let art=cell_effectmap_154_v1::load(effect_154_install.as_deref().unwrap())?;Some(SourceEffectPlane{art,rect,is_154:true,hidden:effect_154_hidden})}else{None};
    let ui_count = [ui_install.is_some(), ui_layout.is_some(), ui_atlas.is_some(), ui_fixture.is_some(), ui_clip.is_some(), ui_half.is_some()].into_iter().filter(|v| *v).count();
    let five_count=[five_layout.is_some(),five_atlas.is_some(),five_span.is_some(),five_origin.is_some(),five_state.is_some(),five_mask.is_some()].into_iter().filter(|v|*v).count();
    if (five_count!=0||five_admitted) && (five_count!=6||!five_admitted){return Err("five source UI requires layout, atlas, span, origin, state, mask and explicit admission".into())}
    if five_cursor.is_some() && five_count!=6{return Err("source UI cursor requires the complete five-segment fixture".into())}
    if five_count!=0 && (!offscreen||ui_count!=0){return Err("five source UI requires --offscreen and excludes the single-segment fixture".into())}
    if ui_count != 0 && ui_count != 6 { return Err("source UI fixture requires install, layout, atlas, fixture, clip and half-pixel options".into()); }
    if ui_count != 0 && !offscreen { return Err("source UI fixture requires --offscreen".into()); }
    let ui = if ui_count == 6 {
        let clip = match ui_clip.as_deref() { Some("shader") => spore_core::spui_draw::UiClipPath::Shader, Some("scissor") => spore_core::spui_draw::UiClipPath::Scissor, _ => return Err("source UI clip must be shader or scissor".into()) };
        let half_pixel = match ui_half.as_deref() { Some("0") => false, Some("1") => true, _ => return Err("source UI half-pixel must be 0 or 1".into()) };
        let request = native_source_ui_fixture_v1::UiRequest { layout: ui_layout.unwrap(), atlas: ui_atlas.unwrap(), fixture: ui_fixture.unwrap(), clip, half_pixel };
        let admitted = native_source_ui_fixture_v1::admit(&ui_install.unwrap(), &request).map_err(|e| e.to_string())?;
        if admitted.rectangles.iter().any(|(_, uv)| uv[0] > uv[2] || uv[1] > uv[3]) { return Err("source UI fixture has inverted texture coordinates".into()); }
        Some(SourceUiFixture {rgba:admitted.rgba,rectangles:admitted.rectangles,five:false})
    } else if five_count==6 {
        let pair=|s:&str|->Result<[f32;2],String>{let parts:Vec<_>=s.split(',').collect();if parts.len()!=2||parts.iter().any(|x|x.len()!=8||!x.bytes().all(|b|b.is_ascii_hexdigit())){return Err("five UI coordinates require two comma-separated f32 hex words".into())}Ok([f32::from_bits(u32::from_str_radix(parts[0],16).map_err(|_|"invalid coordinate")?),f32::from_bits(u32::from_str_radix(parts[1],16).map_err(|_|"invalid coordinate")?)])};
        let parse_hex=|s:&str,n:usize|->Result<u32,String>{if s.len()!=n||!s.bytes().all(|b|b.is_ascii_hexdigit()){return Err("five UI state or mask has invalid hex width".into())}u32::from_str_radix(s,16).map_err(|_|"invalid five UI state or mask".into())};
        let cursor=five_cursor.as_deref().map(pair).transpose()?;
        let x=cell_source_ui_five_v1::load(&five_layout.unwrap(),&five_atlas.unwrap(),pair(five_span.as_deref().unwrap())?,pair(five_origin.as_deref().unwrap())?,parse_hex(five_state.as_deref().unwrap(),8)?,parse_hex(five_mask.as_deref().unwrap(),2)? as u8,cursor).map_err(|e|e.to_string())?;
        Some(SourceUiFixture {rgba:x.rgba,rectangles:x.rectangles,five:true})
    } else { None };
    let mut startup_basis=None;
    let selected_art = if let (Some(install), Some(id)) = (&source_install, source_id) {
        let (mesh, skin, mips, sampler, skin_lanes) = if id == cell_kind0_inspection::CELL_ID {
            let asset = cell_kind0_inspection::load(install, synthetic_palette)?;
            (asset.mesh, asset.texture, Some(asset.mips), asset.sampler, asset.skin_lanes)
        } else {
            let library = crate::source_cell::SourceCellLibrary::open_install(install)?;
            let asset = library.load(id)?;
            let mut sampler = crate::source_cell::source_skin_sampler(&asset)?;
            let mips = library.skin_mips(&asset)?;
            sampler.lod_max_clamp = mips.levels.len().saturating_sub(1) as f32;
            println!("CELL_SOURCE_INSPECTION_MIPS levels={} source=declared_atlas", mips.levels.len());
            println!("CELL_SOURCE_INSPECTION creation={:016X} pending_effects={} renderer=pbr static=true original_animation=false overlay=inspection-order", asset.definition.creation_instance, asset.pending_structure_entries.len());
            for (role, identity) in [("cell", &asset.definition.cell), ("structure", &asset.definition.structure),
                ("model_prop", &asset.model_prop), ("model", &asset.model), ("skin", &asset.skin_source)] {
                println!("CELL_SOURCE_RESOURCE role={role} package={:?} key={}", identity.package, identity.key);
            }
            (asset.mesh, asset.skin, Some(mips), sampler, None)
        };
        Some((mesh, skin, mips, sampler, skin_lanes, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), SourceKind::Cell))
    } else if let Some(install) = &startup_install {
        let staged = crate::cell_source_player_v1::inspect(install, 1, 0)?;
        admit_startup_triple(staged.triple)?;
        verify_startup_art_content(install)?;
        let (mut mesh, skin, mips, sampler) = load_source_creature(install, 0x40616200e219e5d3, true, if startup_inspection_camera { "inspection_camera" } else if startup_traced_material { "traced_material" } else if startup_material_preview { "synthetic_material" } else { "pbr" })?;
        if startup_source_normals || startup_traced_material {
            use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
            let index=NativePackageIndex::open(&install.join("Data/Spore_Content.package")).map_err(|e|e.to_string())?;
            let key=NativeResourceKey{instance_id:0xe219e5d3,type_id:0x00e6bce5,group_id:0x40617100};
            let entry=index.index().find_unique(key).map_err(|e|e.to_string())?.ok_or("startup normal model absent")?;
            if entry.stored_size.max(entry.decompressed_size)>1024*1024{return Err("startup normal model exceeds bound".into())}
            let raw=index.read(entry).map_err(|e|e.to_string())?;
            if startup_traced_material {startup_basis=Some(cell_startup_source_normals_v1::raw_basis(&raw,&mesh)?);} else {
                mesh.normals=cell_startup_source_normals_v1::replacement(&raw,&mesh)?;
                println!("CELL_STARTUP_NORMALS mode=source_formula_cpu_approximation scope=pinned_candidate_only gpu_shader_parity=unverified live_pose=unverified");
            }
        }
        // Recheck exact decoded resources after asset loading. Stable package
        // files throughout the read remain an explicit inspection admission.
        verify_startup_art_content(install)?;
        println!("CELL_STARTUP_ART seed=1 progress=0 triple=E219E5D3:3D97A8E4:40616200 assignment=prototype_player actor_identity=unverified animation=unverified stable_packages=admitted");
        Some((mesh, skin, mips, sampler, None, creature_inspection_rotation(), SourceKind::StartupCell))
    } else if let (Some(install), Some(instance)) = (&creature_install, creature_instance) {
        let (mesh, skin, mips, sampler) = load_source_creature(install, instance, false, "pbr")?;
        Some((mesh, skin, mips, sampler, None, creature_inspection_rotation(), SourceKind::Creature))
    } else { None };
    let material_s1=if startup_material_preview {Some(load_startup_sampler1(startup_install.as_ref().expect("admitted startup install"))?)} else {None};
    let source_player = if let Some((mesh, skin, mips, sampler, skin_lanes, rotation, kind)) = selected_art {
        let (min, max) = mesh.bounds().ok_or("source inspection mesh is empty")?;
        let center = Vec3::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5, (min.z + max.z) * 0.5);
        let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
        if !center.is_finite() || !extent.is_finite() || extent <= 1e-6 {
            return Err("source inspection mesh has invalid or degenerate bounds".into());
        }
        if kind == SourceKind::StartupCell {
            let mut projected_min = Vec3::splat(f32::INFINITY);
            let mut projected_max = Vec3::splat(f32::NEG_INFINITY);
            for point in &mesh.positions {
                let v = rotation * (Vec3::new(point.x, point.y, point.z) - center);
                projected_min = projected_min.min(v);
                projected_max = projected_max.max(v);
            }
            println!("CELL_STARTUP_ART_BOUNDS source_extent={extent:.6} screen_x={:.6} screen_y={:.6} screen_depth={:.6} normalization={:.6}", projected_max.x-projected_min.x, projected_max.y-projected_min.y, projected_max.z-projected_min.z, 2.0/extent);
        }
        Some(SourcePlayer { mesh, skin, mips, sampler, center, scale: 2.0 / extent, rotation, kind, unorm_texture: startup_unorm_texture && kind == SourceKind::StartupCell, skin_lanes, material_s1, traced_basis:startup_basis, inspection_camera:startup_inspection_camera })
    } else { None };
    let rival_ids=rival_roster.or_else(||rival_id.map(|id|vec![id]));
    let source_rival = if let (Some(install), Some(ids)) = (&source_install, rival_ids) {
        let library = crate::source_cell::SourceCellLibrary::open_install(install)?;
        let mut loaded=Vec::with_capacity(ids.len());
        for id in ids {
        let asset = library.load(id)?;
        let mut sampler = crate::source_cell::source_skin_sampler(&asset)?;
        let mips = library.skin_mips(&asset)?;
        sampler.lod_max_clamp = mips.levels.len().saturating_sub(1) as f32;
        let (min, max) = asset.mesh.bounds().ok_or("source rival mesh is empty")?;
        let center = Vec3::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5, (min.z + max.z) * 0.5);
        let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
        if !center.is_finite() || !extent.is_finite() || extent <= 1e-6 {
            return Err("source rival mesh has invalid or degenerate bounds".into());
        }
        println!("CELL_SOURCE_RIVAL id={id:08X} assignment=controlled_prototype_roster renderer=pbr static=true original_selection=unverified");
        for (role, identity) in [("cell", &asset.definition.cell), ("structure", &asset.definition.structure),
            ("model_prop", &asset.model_prop), ("model", &asset.model), ("skin", &asset.skin_source)] {
            println!("CELL_SOURCE_RIVAL_RESOURCE role={role} package={:?} key={}", identity.package, identity.key);
        }
        loaded.push((id,SourcePlayer { mesh: asset.mesh, skin: asset.skin, mips: Some(mips), sampler, center, scale: 2.0 / extent, rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), kind: SourceKind::Cell, unorm_texture: false, skin_lanes: None, material_s1: None, traced_basis:None, inspection_camera:false }));
        }
        Some(SourceRival(loaded))
    } else { None };
    let world = configured_world(seed, diet, explicit_diet, creation_path.as_deref())?;
    diet = world.player.diet;
    if shot.as_ref().is_some_and(|path| path.exists()) {
        return Err("screenshot already exists; choose a new output path".into());
    }
    if let Some(parent) = shot
        .as_ref()
        .and_then(|path| path.parent())
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut app = App::new();
    if offscreen {
        if effect_plane.as_ref().is_some_and(|p|p.is_154){crate::offscreen::configure_unorm_composite(&mut app,1280,800)}else{crate::offscreen::configure(&mut app, 1280, 800)};
        app.init_resource::<OffscreenSimulationReady>();
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | Cell".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }));
    }
    if startup_inspection_camera {
        load_internal_asset!(app, INSPECTION_CAMERA_SHADER, "cell_startup_inspection_camera.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<InspectionMaterial>::default());
    }
    if startup_traced_material {
        load_internal_asset!(app, TRACED_MATERIAL_SHADER, "cell_startup_traced_material.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<TracedMaterial>::default());
    }
    if startup_material_preview {
        load_internal_asset!(app, STARTUP_MATERIAL_SHADER, "cell_startup_material_preview.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<StartupMaterial>::default());
    }
    if saved_blend {
        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, SavedBlendExtension>>::default());
        app.insert_resource(SavedBlendDiagnostic);
    }
    if fragment_preview || effect_plane.as_ref().is_some_and(|p|p.is_154) {
        load_internal_asset!(app, FRAGMENT_PREVIEW_SHADER, "cell_kind0_fragment_preview.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<FragmentPreviewMaterial>::default());
        if fragment_preview {app.insert_resource(FragmentPreviewDiagnostic);}
    }
    if effect_plane.as_ref().is_some_and(|p|!p.is_154) {
        load_internal_asset!(app, EFFECT_PLANE_SHADER, "cell_effectmap_plane_preview.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<EffectPlaneMaterial>::default());
    }
    if synthetic_palette {
        load_internal_asset!(app, SYNTHETIC_PALETTE_SHADER, "cell_kind0_synthetic_palette.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<SyntheticPaletteMaterial>::default());
        app.insert_resource(SyntheticPaletteDiagnostic);
    }
    let source_audio_selected=audio_preview.is_some();
    if let Some(preview)=audio_preview {
        app.add_audio_source::<cell_source_audio_preview_v1::Preview>()
            .insert_resource(cell_source_audio_preview_v1::PendingPreview(preview))
            .add_systems(Startup,cell_source_audio_preview_v1::start);
    }
    if let Some(source) = source_player { app.insert_resource(source.kind); app.insert_resource(source); }
    if let Some(plane) = effect_plane {app.insert_resource(plane);}
    if let Some(source) = source_rival { app.insert_resource(source); }
    if let Some(ui) = ui { app.insert_resource(UiScale(1.0)); app.insert_resource(ui); }
    app.insert_resource(ClearColor(Color::srgb(0.012, 0.032, 0.046)))
        .insert_resource(Game {
            world,
            seed,
            diet,
            paused: false,
            elapsed: 0.0,
            notice: if source_audio_selected {"Source audio inspection preview".into()} else {String::new()},
            rebuild: true,
            save_path,
            shot,
            demo_seconds,
            capture_frame: None,
            frames: 0,
        })
        .init_resource::<SceneObjects>()
        .init_resource::<AuthoredArt>()
        .add_systems(Startup, (setup,log_gpu))
        .add_systems(Update, (drive, sync_scene, update_hud, capture).chain())
        .run();
    Ok(())
}

pub(crate) fn read_creation(path: &std::path::Path) -> Result<Creation, String> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_SAVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_SAVE_BYTES {
        return Err("creation exceeds the bounded save limit".into());
    }
    EditorSnapshot::from_bytes(&bytes)
        .map(|snapshot| snapshot.creation)
        .map_err(|error| error.to_string())
}
fn configured_world(
    seed: u64,
    diet: Diet,
    explicit_diet: bool,
    path: Option<&std::path::Path>,
) -> Result<CellWorld, String> {
    let Some(path) = path else {
        return Ok(CellWorld::new(seed, diet));
    };
    let creation = read_creation(path)?;
    if creation.kind != CreationKind::Cell {
        return Err("Cell gameplay requires a Cell creation".into());
    }
    let authored_diet = match creation
        .gameplay_loadout()
        .map_err(|error| error.to_string())?
    {
        spore_core::editor::GameplayLoadout::Cell(loadout) => loadout.diet,
        _ => return Err("Cell gameplay requires a Cell creation".into()),
    };
    let mut world = CellWorld::new(seed, authored_diet);
    world
        .apply_creation(creation)
        .map_err(|error| error.to_string())?;
    if explicit_diet && world.player.diet != diet {
        return Err(
            "--diet contradicts the authored mouth; edit the creation or omit --diet".into(),
        );
    }
    crate::editor::creation_mesh(world.creation.as_ref().unwrap(), true)?;
    Ok(world)
}

fn log_gpu(adapter:Res<RenderAdapterInfo>){println!("CELL_GPU name={:?} backend={:?} type={:?}",adapter.name,adapter.backend,adapter.device_type);}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    source: Option<Res<SourcePlayer>>,
    rival: Option<Res<SourceRival>>,
    ui: Option<Res<SourceUiFixture>>,
    effect_plane: Option<Res<SourceEffectPlane>>,
    mut source_materials: ResMut<Assets<StandardMaterial>>,
    mut saved_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, SavedBlendExtension>>>>,
    saved_blend: Option<Res<SavedBlendDiagnostic>>,
    mut preview_materials: ParamSet<(Option<ResMut<Assets<FragmentPreviewMaterial>>>, Option<ResMut<Assets<EffectPlaneMaterial>>>, Option<ResMut<Assets<StartupMaterial>>>, Option<ResMut<Assets<TracedMaterial>>>, Option<ResMut<Assets<InspectionMaterial>>>)>,
    fragment_preview: Option<Res<FragmentPreviewDiagnostic>>,
    mut synthetic_materials: Option<ResMut<Assets<SyntheticPaletteMaterial>>>,
    synthetic_palette: Option<Res<SyntheticPaletteDiagnostic>>,
    mut images: ResMut<Assets<Image>>,
    mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>,
) {
    if let Some(ui) = ui {
        if ui.rectangles.iter().any(|(d,_)|d[0]<d[2] && d[1]<d[3]) {
            let mut atlas = Image::new(Extent3d { width:512, height:512, depth_or_array_layers:1 }, TextureDimension::D2, ui.rgba.clone(), TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
            atlas.sampler = ImageSampler::linear();
            let handle = images.add(atlas);
            if let Some(readiness) = readiness.as_mut() { readiness.require_image(&handle); }
            for (dest, uv) in &ui.rectangles {
                if dest[0] == dest[2] || dest[1] == dest[3] { continue; }
                let mut node = ImageNode::new(handle.clone());
                node.rect = Some(Rect::new(uv[0]*512., uv[1]*512., uv[2]*512., uv[3]*512.));
                commands.spawn((node, Node { position_type:PositionType::Absolute, left:px(dest[0]), top:px(dest[1]), width:px(dest[2]-dest[0]), height:px(dest[3]-dest[1]), ..default() }, GlobalZIndex(100)));
            }
        }
        println!("CELL_SOURCE_UI experimental_bevy=true controlled_fixture=true five_segments={} native_shader=false bevy_shader=true clip_and_half_pixel_admission_only={} controlled_placement=true controlled_state=true native_placement=false runtime_state=false",ui.five,!ui.five);
    }
    if let Some(plane)=effect_plane.as_ref(){
        if !plane.hidden {
        let data=&plane.art.geometry;let mut mesh=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION,data.positions.iter().map(|p|[p.x,p.y,p.z]).collect::<Vec<_>>());mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,data.uvs.clone());mesh.insert_indices(Indices::U32(data.indices.clone()));
        let mut tex=Image::new(Extent3d{width:256,height:256,depth_or_array_layers:1},TextureDimension::D2,plane.art.pixels[..256*256*4].to_vec(),TextureFormat::Rgba8Unorm,RenderAssetUsages::default());tex.texture_descriptor.mip_level_count=9;tex.data=Some(plane.art.pixels.clone());tex.sampler=ImageSampler::Descriptor(ImageSamplerDescriptor{address_mode_u:ImageAddressMode::Repeat,address_mode_v:ImageAddressMode::Repeat,mag_filter:ImageFilterMode::Linear,min_filter:ImageFilterMode::Linear,mipmap_filter:ImageFilterMode::Nearest,lod_max_clamp:8.,..default()});
        let image=images.add(tex);if let Some(ready)=readiness.as_mut(){ready.require_image(&image)}
        let [x,y,w,h]=plane.rect;
        if plane.is_154 {
            let material=preview_materials.p0().as_mut().expect("154 fragment shader plugin").add(FragmentPreviewMaterial{texture:image});
            if !plane.hidden {commands.spawn((Mesh3d(meshes.add(mesh)),MeshMaterial3d(material),Transform::from_xyz(x,y,0.).with_scale(Vec3::new(w,h,1.)),RenderLayers::layer(2)));}
        } else {
            let material=preview_materials.p1().as_mut().expect("effect plane shader plugin").add(EffectPlaneMaterial{texture:image});
            commands.spawn((Mesh3d(meshes.add(mesh)),MeshMaterial3d(material),Transform::from_xyz(x,y,0.).with_scale(Vec3::new(w,h,1.)),RenderLayers::layer(2)));
        }
        }
        commands.spawn((Camera3d::default(),Camera{order:-1,..default()},Projection::Orthographic(OrthographicProjection{scaling_mode:ScalingMode::FixedVertical{viewport_height:76.0},..OrthographicProjection::default_3d()}),Transform::from_xyz(0.,0.,100.),RenderLayers::layer(2),Msaa::Off));
        let label=if plane.is_154 {"SOURCE 154: controlled placement; unorm scene diagnostic"}else{"SOURCE PLANE: caller placement; shader diagnostic"};
        commands.spawn((Text::new(label),TextFont{font_size:14.,..default()},TextColor(Color::WHITE),Node{position_type:PositionType::Absolute,left:px(28),bottom:px(if rival.is_some(){120.}else if source.is_some(){96.}else{72.}),..default()}));
        if plane.is_154 {println!("CELL_EFFECT_154 controlled_world_rect={:?} hidden_control={} source=154C862B conditional_fragment=true profile=3 data210=saved data24e=zero other_data=absent stage0_bc3=true stage1_unsampled=true source_alpha=true scene_target=unorm final_target=srgb msaa=off native_runtime_selection=false native_placement=false app_camera=bevy_reverse_z native_depth_parity=false",plane.rect,plane.hidden);}else{println!("CELL_EFFECT_PLANE controlled_world_rect={:?} source=44C489B7 source_fragment_inspired=true opaque_mips=true inverse_srgb=true cpu_bc1=true sampler=wrap_linear_nearest msaa=off app_target_srgb=true native_runtime_selection=false native_placement=false app_camera=bevy_reverse_z native_depth_parity=false",plane.rect);}
    }
    let mut camera_2d = commands.spawn((
        Camera2d,
        Camera{clear_color:if effect_plane.is_some(){ClearColorConfig::None}else{ClearColorConfig::Default},..default()},
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 76.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    if effect_plane.is_some() || source.as_ref().is_some_and(|s|s.inspection_camera) { camera_2d.insert(Msaa::Off); }
    let creature_mode = source.as_ref().is_some_and(|source| source.kind == SourceKind::Creature);
    if let Some(source) = source {
        let data = &source.mesh;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
        if let Some((normal,tangent))=&source.traced_basis {mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,normal.clone());mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT,tangent.clone());} else {mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());}
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());
        if synthetic_palette.is_some() {
            let lanes=source.skin_lanes.as_ref().expect("admitted synthetic palette lanes are required");
            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, lanes.indices.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, lanes.weights.clone());
        }
        mesh.insert_indices(Indices::U32(data.indices.clone()));
        let texture = &source.skin;
        let mut image = inspection_image(texture, source.mips.as_ref(), &source.sampler);
        if apply_startup_unorm(&mut image,source.kind,source.unorm_texture) {
            println!("CELL_STARTUP_TEXTURE format=rgba8unorm experiment=sampler_interpretation source_shader=false lighting_parity=unverified");
        }
        if fragment_preview.is_some() || source.material_s1.is_some() { image.texture_descriptor.format = TextureFormat::Rgba8Unorm; }
        let image = images.add(image);
        if let Some(readiness) = readiness.as_mut() { readiness.require_image(&image); }
        let base = StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(image.clone()), perceptual_roughness: 0.7, metallic: 0.0, cull_mode: None, double_sided: true, ..default() };
        let material = if let Some(chain)=&source.material_s1 {
            let sampler1=ImageSamplerDescriptor{address_mode_u:ImageAddressMode::ClampToEdge,address_mode_v:ImageAddressMode::ClampToEdge,address_mode_w:ImageAddressMode::ClampToEdge,min_filter:ImageFilterMode::Linear,mag_filter:ImageFilterMode::Linear,mipmap_filter:ImageFilterMode::Linear,lod_max_clamp:8.0,..default()};
            let mut second=inspection_image(&chain.levels[0],Some(chain),&sampler1);second.texture_descriptor.format=TextureFormat::Rgba8Unorm;let second=images.add(second);
            if let Some(readiness)=readiness.as_mut(){readiness.require_image(&second);}
            if source.inspection_camera {
                let material=preview_materials.p4().as_mut().expect("inspection material plugin required").add(InspectionMaterial{stage0:image,stage1:second});
                println!("CELL_STARTUP_INSPECTION_CAMERA controlled=true camera=frozen_editor_inspection viewport=768x768 reverse_z=true half_pixel=wgpu depth_format=bevy target_transfer=bevy gameplay_camera=false live_pose=unverified retail_parity=false");
                SourcePlayerMaterial::InspectionMaterial(material)
            } else if source.traced_basis.is_some(){
                let material=preview_materials.p3().as_mut().expect("traced material plugin required").add(TracedMaterial{stage0:image,stage1:second});
                println!("CELL_STARTUP_TRACED_MATERIAL controlled=true basis=source_packed shading_constants=frozen_inspection_trace clip_camera=bevy shading_space_vs_clip=unverified implicit_lod=bevy live_material=unverified retail_parity=false");
                SourcePlayerMaterial::TracedMaterial(material)
            } else {
                let material=preview_materials.p2().as_mut().expect("startup material plugin required").add(StartupMaterial{stage0:image,stage1:second});
                println!("CELL_STARTUP_MATERIAL controlled=true basis=synthetic lighting=synthetic material_c0=trace_captured camera=bevy implicit_lod=bevy live_material=unverified retail_parity=false");
                SourcePlayerMaterial::StartupMaterial(material)
            }
        } else if synthetic_palette.is_some() {
            let material=synthetic_materials.as_mut().expect("synthetic palette plugin is required").add(SyntheticPaletteMaterial { texture: image });
            println!("CELL_SOURCE_SYNTHETIC_PALETTE controlled=true formula=x:k*0.004,y:(k_mod_3)*0.004 k=register_offset/3 lanes=raw_ubyte4_weights_ubyte4n runtime_pose=unverified camera=bevy_inspection depth_compare=bevy_reverse_z transparent_order=bevy");
            println!("CELL_SOURCE_FRAGMENT_SAMPLER {:?}", source.sampler);
            SourcePlayerMaterial::SyntheticPalette(material)
        } else if fragment_preview.is_some() {
            let material=preview_materials.p0().as_mut().expect("fragment preview plugin is required").add(FragmentPreviewMaterial { texture: image });
            println!("CELL_SOURCE_FRAGMENT_PREVIEW controlled=true identity_pose=unweighted_bind_position material_tint=white texture=stage0_unorm shader=source_fragment_inspired camera=bevy_inspection depth_compare=bevy_reverse_z transparent_order=bevy runtime_state=unverified");
            println!("CELL_SOURCE_FRAGMENT_SAMPLER {:?}", source.sampler);
            SourcePlayerMaterial::FragmentPreview(material)
        } else if saved_blend.is_some() {
            let mut base=base; base.alpha_mode=AlphaMode::Blend;
            let material=saved_materials.as_mut().expect("saved-blend material plugin is required").add(ExtendedMaterial { base, extension:SavedBlendExtension {} });
            println!("CELL_SOURCE_SAVED_BLEND controlled=true saved_blend_depth_fields=true pbr_shader=true original_shader=false depth_write=true depth_compare=bevy_reverse_z transparent_order=bevy runtime_state=unverified");
            SourcePlayerMaterial::SavedBlend(material)
        } else { SourcePlayerMaterial::Pbr(source_materials.add(base)) };
        commands.insert_resource(SourcePlayerRender { mesh: meshes.add(mesh), material, center: source.center, scale: source.scale, rotation: source.rotation });
        let mut camera_3d = commands.spawn((Camera3d::default(), Camera { order: 1, clear_color: ClearColorConfig::None, viewport: if source.inspection_camera {Some(Viewport{physical_position:UVec2::new(256,16),physical_size:UVec2::splat(768),..default()})}else{None}, ..default() },
            Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: 76.0 }, ..OrthographicProjection::default_3d() }),
            Transform::from_xyz(0., 0., 100.), RenderLayers::layer(1), IsDefaultUiCamera,
            AmbientLight { color: Color::WHITE, brightness: 300., ..default() }));
        if effect_plane.is_some() || source.inspection_camera { camera_3d.insert(Msaa::Off); }
        if source.inspection_camera {
            camera_3d.remove::<IsDefaultUiCamera>();
            commands.spawn((Camera2d, Camera { order:2, clear_color:ClearColorConfig::None, ..default() }, RenderLayers::layer(30), IsDefaultUiCamera, Msaa::Off));
            println!("CELL_STARTUP_INSPECTION_ROUTING ui=full_viewport top_camera=ui_only camera_msaa=off experimental=true");
        }
        commands.spawn((DirectionalLight { illuminance: 9000., ..default() }, Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -0.35, 0.)), RenderLayers::layer(1)));
        let label = if source.inspection_camera { "SOURCE STARTUP CELL: frozen editor camera inspection; prototype gameplay" } else if source.traced_basis.is_some() { "SOURCE STARTUP CELL: frozen inspection material; prototype gameplay" } else if source.material_s1.is_some() { "SOURCE STARTUP CELL: synthetic material inspection; prototype gameplay" } else if synthetic_palette.is_some() { "SOURCE CELL: synthetic palette preview; experimental gameplay" } else if fragment_preview.is_some() { "SOURCE CELL: controlled fragment preview; experimental gameplay" } else if source.kind == SourceKind::StartupCell { "SOURCE STARTUP CELL: static candidate art; prototype gameplay" } else if source.kind == SourceKind::Creature { "SOURCE CREATURE: static PBR inspection art; experimental gameplay" } else { "SOURCE CELL: static PBR inspection art; experimental gameplay" };
        let label_bottom = if rival.is_some() { 96. } else { 72. };
        commands.spawn((Text::new(label), TextFont { font_size: 14., ..default() }, TextColor(Color::WHITE), Node { position_type: PositionType::Absolute, left: px(28), bottom: px(label_bottom), ..default() }));
    }
    if let Some(rival) = rival {
        let mut roster=Vec::with_capacity(rival.0.len());
        for (id,source) in &rival.0 {
        let data = &source.mesh;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());
        mesh.insert_indices(Indices::U32(data.indices.clone()));
        let image = images.add(inspection_image(&source.skin, source.mips.as_ref(), &source.sampler));
        if let Some(readiness) = readiness.as_mut() { readiness.require_image(&image); }
        let material = SourcePlayerMaterial::Pbr(source_materials.add(StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(image), perceptual_roughness: 0.7, metallic: 0.0, cull_mode: None, double_sided: true, ..default() }));
        roster.push((*id,SourcePlayerRender { mesh: meshes.add(mesh), material, center: source.center, scale: source.scale, rotation: source.rotation }));
        }
        commands.insert_resource(SourceRivalRender(roster));
        commands.spawn((Text::new("RIVALS: controlled source roster on prototype actors"), TextFont { font_size: 14., ..default() }, TextColor(Color::WHITE), Node { position_type: PositionType::Absolute, left: px(28), bottom: px(72), ..default() }));
    }
    let circle = meshes.add(Circle::new(1.0));
    let line = meshes.add(Rectangle::new(1.0, 1.0));
    let ring = meshes.add(Annulus::new(0.96, 1.04));
    let leaf = meshes.add(RegularPolygon::new(1.0, 3));
    let bg = materials.add(Color::srgb(0.025, 0.092, 0.108));
    let border = materials.add(Color::srgb(0.06, 0.19, 0.20));
    if effect_plane.is_none(){commands.spawn((
        Mesh2d(line.clone()),
        MeshMaterial2d(border),
        Transform::from_xyz(0.0, 0.0, -9.0).with_scale(Vec3::new(82.0, 58.0, 1.0)),
    ));}
    if effect_plane.is_none(){commands.spawn((
        Mesh2d(line.clone()),
        MeshMaterial2d(bg),
        Transform::from_xyz(0.0, 0.0, -8.0).with_scale(Vec3::new(80.0, 56.0, 1.0)),
    ));}
    let speck = materials.add(Color::srgba(0.19, 0.45, 0.43, 0.12));
    for i in 0..110 {
        let x = ((i * 139 + 17) % 791) as f32 / 10.0 - 39.5;
        let y = ((i * 83 + 31) % 551) as f32 / 10.0 - 27.5;
        commands.spawn((
            Mesh2d(circle.clone()),
            MeshMaterial2d(speck.clone()),
            Transform::from_xyz(x, y, -6.0).with_scale(Vec3::splat(0.04 + (i % 5) as f32 * 0.024)),
        ));
    }
    commands.insert_resource(Art {
        circle,
        ring,
        leaf,
        line,
        player: materials.add(Color::srgb(0.16, 0.80, 0.68)),
        predator: materials.add(Color::srgb(0.93, 0.35, 0.23)),
        prey: materials.add(Color::srgb(0.50, 0.45, 0.77)),
        plant: materials.add(Color::srgb(0.62, 0.90, 0.34)),
        meat: materials.add(Color::srgb(0.96, 0.55, 0.44)),
        membrane: materials.add(Color::srgba(0.61, 1.0, 0.88, 0.70)),
        nucleus: materials.add(Color::srgba(0.02, 0.28, 0.30, 0.76)),
        white: materials.add(Color::srgb(0.92, 0.98, 0.90)),
        dark: materials.add(Color::srgb(0.018, 0.055, 0.064)),
        rock: materials.add(Color::srgb(0.16, 0.25, 0.27)),
    });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: px(26),
                right: px(26),
                height: px(78),
                padding: UiRect::all(px(12)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(16),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.045, 0.06, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CELL"),
                TextFont {
                    font_size: 25.0,
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.95, 0.83)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.91, 0.89)),
                Node {
                    width: px(280),
                    ..default()
                },
                Hud,
            ));
            for (i, label, colour) in [
                (0, "HEALTH", Color::srgb(0.34, 0.85, 0.66)),
                (1, "ENERGY", Color::srgb(0.36, 0.65, 0.94)),
                (2, "GROWTH", Color::srgb(0.91, 0.77, 0.36)),
            ] {
                parent
                    .spawn(Node {
                        width: px(175),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        ..default()
                    })
                    .with_children(|group| {
                        group.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.56, 0.71, 0.71)),
                        ));
                        group
                            .spawn((
                                Node {
                                    width: percent(100),
                                    height: px(6),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.08, 0.17, 0.19)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: percent(100),
                                        height: percent(100),
                                        ..default()
                                    },
                                    BackgroundColor(colour),
                                    Meter(i),
                                ));
                            });
                    });
            }
        });
    let controls = if creature_mode { "W/S swim   A/D turn   Shift boost   Space bite   P pause\nR restart   1/2/3 diet   Esc exit" } else { "W/S swim   A/D turn   Shift boost   Space bite   P pause\nR restart   1/2/3 diet   F5 save   F9 load   Esc exit" };
    commands.spawn((Text::new(controls), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.61, 0.77, 0.77)), Node { position_type: PositionType::Absolute, left: px(28), bottom: px(22), ..default() }));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.92, 0.71)),
        Node {
            position_type: PositionType::Absolute,
            top: percent(40),
            left: percent(37),
            ..default()
        },
        Banner,
    ));
}

fn keyboard_input(keys: &ButtonInput<KeyCode>) -> CellInput {
    let axis =
        |positive, negative| (keys.pressed(positive) as i8 - keys.pressed(negative) as i8) as f32;
    CellInput {
        thrust: axis(KeyCode::KeyW, KeyCode::KeyS),
        turn: axis(KeyCode::KeyA, KeyCode::KeyD),
        boost: keys.pressed(KeyCode::ShiftLeft),
        bite: keys.pressed(KeyCode::Space),
    }
}

pub(crate) fn write_save(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_extension(format!("tmp-{}-{nonce}", std::process::id()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn read_save(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save exceeds 16 MiB limit",
        ));
    }
    Ok(bytes)
}

fn validate_source_snapshot(world: &CellWorld, source_enabled: bool) -> Result<(), String> {
    if source_enabled && world.creation.is_some() {
        return Err("save contains an authored creation; restart without source Cell inspection art to load it".into());
    }
    Ok(())
}

fn offscreen_step_delta(delta: f64, elapsed: f64, duration: f64, ready: bool) -> f64 {
    if !ready { return 0.0; }
    delta.min(0.1).min((duration - elapsed).max(0.0))
}

fn drive(time: Res<Time>, mut keys: ResMut<ButtonInput<KeyCode>>, mut game: ResMut<Game>, source: Option<Res<SourceKind>>, ready: Option<Res<OffscreenSimulationReady>>) {
    // Render preparation, text layout, and capture continue in their own systems.
    // Do not let pipeline compilation time advance the offscreen simulation.
    if ready.as_ref().is_some_and(|ready| !ready.0) { return; }
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        game.paused = !game.paused;
    }
    let new_diet = if keys.just_pressed(KeyCode::Digit1) {
        Some(Diet::Herbivore)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(Diet::Carnivore)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(Diet::Omnivore)
    } else {
        None
    };
    if new_diet.is_some() && game.world.creation.is_some() {
        game.notice = "Edit mouth in studio to change diet".into();
    }
    if keys.just_pressed(KeyCode::KeyR) || (new_diet.is_some() && game.world.creation.is_none()) {
        if let Some(diet) = new_diet.filter(|_| game.world.creation.is_none()) {
            game.diet = diet;
        }
        if game.world.creation.is_some() {
            game.world.restart();
        } else {
            game.world = CellWorld::new(game.seed, game.diet);
        }
        game.paused = false;
        game.rebuild = true;
        game.notice.clear();
    }
    if keys.just_pressed(KeyCode::F5) {
        if source.as_deref() == Some(&SourceKind::Creature) {
            game.notice = "Save unavailable during source Creature inspection".into();
        } else {
        let result = game
            .world
            .snapshot()
            .to_bytes()
            .map_err(|e| e.to_string())
            .and_then(|saved| write_save(&game.save_path, &saved).map_err(|e| e.to_string()));
        game.notice = match result {
            Ok(()) => "Saved".into(),
            Err(e) => format!("Save failed: {e}"),
        };
        }
    }
    if keys.just_pressed(KeyCode::F9) {
        if source.as_deref() == Some(&SourceKind::Creature) {
            game.notice = "Load unavailable during source Creature inspection".into();
        } else {
        let result = read_save(&game.save_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| CellSnapshot::from_bytes(&bytes).map_err(|e| e.to_string()))
            .and_then(|snapshot| CellWorld::from_snapshot(snapshot).map_err(|e| e.to_string()))
            .and_then(|world| {
                validate_source_snapshot(&world, source.is_some())?;
                if let Some(creation) = &world.creation {
                    crate::editor::creation_mesh(creation, true)?;
                }
                Ok(world)
            });
        match result {
            Ok(world) => {
                game.seed = world.seed();
                game.diet = world.player.diet;
                game.world = world;
                game.paused = false;
                game.rebuild = true;
                game.notice = "Loaded".into();
            }
            Err(e) => game.notice = format!("Load failed: {e}"),
        }
        }
    }
    let dt = if ready.is_some() {
        offscreen_step_delta(time.delta_secs_f64(), game.elapsed, game.demo_seconds.unwrap_or(1.5), true)
    } else {
        time.delta_secs_f64().min(0.1)
    };
    game.elapsed += dt;
    if game.demo_seconds.is_some() {
        for key in [
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::Space,
            KeyCode::ShiftLeft,
        ] {
            keys.release(key);
        }
        let player = &game.world.player;
        let desired = game
            .world
            .food
            .iter()
            .filter(|food| {
                game.world.can_perceive(food.position)
                    && matches!(
                        (game.diet, food.kind),
                        (Diet::Omnivore, _)
                            | (Diet::Herbivore, FoodKind::Plant)
                            | (Diet::Carnivore, FoodKind::Meat)
                    )
            })
            .min_by(|a, b| {
                let distance = |p: spore_core::cell::CellVec2| {
                    (p.x - player.position.x).powi(2) + (p.y - player.position.y).powi(2)
                };
                distance(a.position).total_cmp(&distance(b.position))
            })
            .map(|food| {
                (food.position.y - player.position.y).atan2(food.position.x - player.position.x)
            })
            .unwrap_or(player.heading);
        let error = (desired - player.heading + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        if error.abs() > 0.06 {
            keys.press(if error > 0.0 {
                KeyCode::KeyA
            } else {
                KeyCode::KeyD
            });
        }
        if error.abs() < 0.6 {
            keys.press(KeyCode::KeyW);
        }
        keys.press(KeyCode::Space);
    }
    if !game.paused && game.capture_frame.is_none() && dt > 0.0 {
        game.world.advance(dt, keyboard_input(&keys));
    }
}

fn add_actor(commands: &mut Commands, art: &Art, player: bool, dangerous: bool) -> Entity {
    let colour = if player {
        art.player.clone()
    } else if dangerous {
        art.predator.clone()
    } else {
        art.prey.clone()
    };
    commands
        .spawn((Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(colour),
                Transform::default().with_scale(Vec3::new(1.05, 0.85, 1.0)),
            ));
            parent.spawn((
                Mesh2d(art.ring.clone()),
                MeshMaterial2d(art.membrane.clone()),
                Transform::from_xyz(0.0, 0.0, 0.05).with_scale(Vec3::new(1.05, 0.85, 1.0)),
            ));
            parent.spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(art.nucleus.clone()),
                Transform::from_xyz(-0.10, 0.0, 0.10).with_scale(Vec3::new(0.48, 0.43, 1.0)),
            ));
            for side in [-1.0, 1.0] {
                parent.spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.white.clone()),
                    Transform::from_xyz(0.55, 0.31 * side, 0.14).with_scale(Vec3::splat(0.17)),
                ));
                parent.spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.dark.clone()),
                    Transform::from_xyz(0.63, 0.31 * side, 0.16).with_scale(Vec3::splat(0.075)),
                ));
                parent.spawn((
                    Mesh2d(art.line.clone()),
                    MeshMaterial2d(art.membrane.clone()),
                    Transform::from_xyz(-1.20, 0.22 * side, 0.02)
                        .with_rotation(Quat::from_rotation_z(-0.2 * side))
                        .with_scale(Vec3::new(0.75, 0.075, 1.0)),
                ));
            }
            parent.spawn((
                Mesh2d(art.leaf.clone()),
                MeshMaterial2d(art.white.clone()),
                Transform::from_xyz(1.0, 0.0, 0.12)
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(0.24, 0.32, 1.0)),
            ));
        })
        .id()
}

fn sync_scene(
    mut commands: Commands,
    art: Res<Art>,
    mut objects: ResMut<SceneObjects>,
    mut game: ResMut<Game>,
    mut transforms: Query<&mut Transform>,
    mut authored: ResMut<AuthoredArt>,
    source: Option<Res<SourcePlayerRender>>,
    rival: Option<Res<SourceRivalRender>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if authored.creation != game.world.creation {
        if let Some(mesh) = authored.mesh.take() {
            meshes.remove(mesh.id());
        }
        if let Some(creation) = &game.world.creation {
            match crate::editor::creation_mesh(creation, true) {
                Ok(mesh) => authored.mesh = Some(meshes.add(mesh)),
                Err(error) => {
                    game.notice = format!("Creation visual failed: {error}");
                }
            }
            if authored.material.is_none() {
                authored.material = Some(materials.add(Color::WHITE));
            }
        }
        authored.creation = game.world.creation.clone();
        game.rebuild = true;
    }
    if game.rebuild {
        for (_, entity) in objects.0.drain() {
            commands.entity(entity).despawn();
        }
        game.rebuild = false;
    }
    let mut live = HashSet::new();
    for (is_player, actor) in std::iter::once((true, &game.world.player))
        .chain(game.world.enemies.iter().map(|a| (false, a)))
    {
        if !is_player && !game.world.can_perceive(actor.position) {
            continue;
        }
        let key = (0, actor.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            if is_player {
                if let Some(source) = &source {
                    return commands.spawn((Transform::default(), Visibility::Visible)).with_children(|parent| {
                        // The selected inspection path supplies its own inferred basis;
                        // neither establishes original gameplay-facing orientation.
                        let rotation = source.rotation;
                        let transform=Transform::from_translation(-(rotation * source.center) * source.scale)
                            .with_rotation(rotation).with_scale(Vec3::splat(source.scale));
                        match &source.material {
                            SourcePlayerMaterial::Pbr(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::SavedBlend(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::FragmentPreview(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::SyntheticPalette(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::StartupMaterial(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::TracedMaterial(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::InspectionMaterial(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1),NoFrustumCulling)); }
                        }
                    }).id();
                }
                if let (Some(mesh), Some(material)) = (&authored.mesh, &authored.material) {
                    return commands
                        .spawn((
                            Mesh2d(mesh.clone()),
                            MeshMaterial2d(material.clone()),
                            Transform::default(),
                        ))
                        .id();
                }
            }
            if !is_player {
                if let Some(rival) = &rival {
                    let (id,source) = &rival.0[actor.id as usize % rival.0.len()];
                    println!("CELL_SOURCE_RIVAL_ASSIGNMENT actor_id={} source_id={id:08X} policy=prototype_id_modulo_roster controlled=true native_selection=false",actor.id);
                    return commands.spawn((Transform::default(), Visibility::Visible)).with_children(|parent| {
                        let rotation = source.rotation;
                        let transform=Transform::from_translation(-(rotation * source.center) * source.scale)
                            .with_rotation(rotation).with_scale(Vec3::splat(source.scale));
                        match &source.material {
                            SourcePlayerMaterial::Pbr(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::SavedBlend(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::FragmentPreview(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::SyntheticPalette(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::StartupMaterial(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::TracedMaterial(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1))); }
                            SourcePlayerMaterial::InspectionMaterial(material) => { parent.spawn((Mesh3d(source.mesh.clone()),MeshMaterial3d(material.clone()),transform,RenderLayers::layer(1),NoFrustumCulling)); }
                        }
                    }).id();
                }
            }
            add_actor(
                &mut commands,
                &art,
                is_player,
                actor.behavior == CellBehavior::Predator,
            )
        });
        let t = Transform::from_xyz(actor.position.x, actor.position.y, 2.0)
            .with_rotation(Quat::from_rotation_z(actor.heading))
            .with_scale(Vec3::splat(actor.radius()));
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = t;
        } else {
            commands.entity(entity).insert(t);
        }
    }
    for food in &game.world.food {
        if !game.world.can_perceive(food.position) {
            continue;
        }
        let key = (1, food.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh2d(art.leaf.clone()),
                    MeshMaterial2d(if food.kind == FoodKind::Plant {
                        art.plant.clone()
                    } else {
                        art.meat.clone()
                    }),
                    Transform::default(),
                ))
                .id()
        });
        let t = Transform::from_xyz(food.position.x, food.position.y, 1.0)
            .with_rotation(Quat::from_rotation_z(food.id as f32 * 0.73))
            .with_scale(Vec3::splat(food.radius()));
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = t;
        } else {
            commands.entity(entity).insert(t);
        }
    }
    for obstacle in &game.world.obstacles {
        let key = (2, obstacle.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.rock.clone()),
                    Transform::default(),
                ))
                .id()
        });
        let t = Transform::from_xyz(obstacle.position.x, obstacle.position.y, 0.0)
            .with_scale(Vec3::splat(obstacle.radius));
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = t;
        } else {
            commands.entity(entity).insert(t);
        }
    }
    objects.0.retain(|key, entity| {
        if live.contains(key) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
}

fn update_hud(
    game: Res<Game>,
    mut texts: Query<&mut Text, (With<Hud>, Without<Banner>)>,
    mut banners: Query<&mut Text, (With<Banner>, Without<Hud>)>,
    mut meters: Query<(&Meter, &mut Node)>,
) {
    for mut text in &mut texts {
        text.0 = format!(
            "{:?}  |  {} nutrients\nMass {:.1}  |  {} rivals\n{}",
            game.diet,
            game.world.eaten,
            game.world.player.mass,
            game.world.enemies.len(),
            if game.notice.is_empty() {
                game.world
                    .creation
                    .as_ref()
                    .map(|creation| {
                        format!(
                            "{} | awareness {:.0}m",
                            creation.name.chars().take(12).collect::<String>(),
                            game.world.perception_radius()
                        )
                    })
                    .unwrap_or_default()
            } else {
                game.notice.chars().take(38).collect()
            }
        );
    }
    for (meter, mut node) in &mut meters {
        node.width = percent(
            100.0
                * match meter.0 {
                    0 => game.world.player.health / game.world.player.max_health,
                    1 => game.world.player.energy,
                    _ => game.world.progress(),
                }
                .clamp(0.0, 1.0),
        );
    }
    for mut text in &mut banners {
        text.0 = if game.paused {
            "PAUSED\nP to return".into()
        } else {
            match game.world.state {
                CellState::Playing => String::new(),
                CellState::Evolved => "READY TO EVOLVE\nR to swim again".into(),
                CellState::Dead => "THE TIDE MOVES ON\nR to try again".into(),
            }
        };
    }
}

fn capture(mut commands: Commands, mut game: ResMut<Game>, target: Option<Res<crate::offscreen::OffscreenTarget>>, mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>, mut simulation_ready: Option<ResMut<OffscreenSimulationReady>>, layouts: Query<(&Text, Option<&bevy::text::TextLayoutInfo>)>, time: Res<Time>) {
    if let Some(readiness) = readiness.as_mut() {
        if let Some(error) = readiness.failure() { eprintln!("Cell offscreen render failed: {error}"); std::process::exit(1); }
        let text_ready = layouts.iter().all(|(text, layout)| text.0.is_empty() || layout.is_some_and(|layout| readiness.require_text(layout)));
        if !text_ready || !readiness.ready() {
            if time.elapsed_secs() > 120. { eprintln!("Cell offscreen renderer did not become ready"); std::process::exit(1); }
            return;
        }
        if let Some(simulation_ready) = simulation_ready.as_mut() {
            simulation_ready.0 = true;
        }
    }
    game.frames += 1;
    let duration = game.demo_seconds.unwrap_or(1.5);
    if game.capture_frame.is_none()
        && game.elapsed >= duration
        && (game.shot.is_some() || game.demo_seconds.is_some())
    {
        if let Some(path) = &game.shot {
            commands
                .spawn(target.as_ref().map_or_else(Screenshot::primary_window, |target| target.screenshot()))
                .observe(save_to_disk(path.clone()));
        }
        game.capture_frame = Some(game.frames);
        println!("CELL_RESULT state={:?} tick={} eaten={} kills={} progress={:.6} health={:.3} x={:.4} y={:.4}", game.world.state, game.world.tick, game.world.eaten, game.world.kills, game.world.progress(), game.world.player.health, game.world.player.position.x, game.world.player.position.y);
    }
    if let Some(frame) = game.capture_frame {
        if game.frames > frame + 10 && game.shot.as_ref().is_none_or(|path| path.exists()) {
            std::process::exit(0);
        }
        if game.frames > frame + 600 {
            eprintln!("Cell screenshot capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_art_requires_the_controlled_seed_and_exclusive_inputs() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-startup-install", "missing"])).unwrap_err().contains("together"));
        assert!(run(&args(&["--startup-inspection-camera"])).unwrap_err().contains("requires --startup-traced-material"));
        assert!(run(&args(&["--startup-inspection-camera","--startup-inspection-camera"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--offscreen","--source-startup-install","missing","--admit-startup-stable-packages","--seed","1","--startup-material-preview","--startup-traced-material","--startup-inspection-camera","--shot","unused.png","--source-effect-154-no-effect"])).unwrap_err().contains("excludes source effect 154"));
        assert!(run(&args(&["--offscreen","--source-startup-install","missing","--admit-startup-stable-packages","--seed","1","--startup-material-preview","--startup-traced-material","--startup-inspection-camera","--shot","unused.png","--source-effect-plane-install","missing"])).unwrap_err().contains("effect plane requires"));
        assert!(run(&args(&["--startup-traced-material"])).unwrap_err().contains("requires --startup-material-preview"));
        assert!(run(&args(&["--startup-traced-material","--startup-traced-material"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--startup-material-preview"])).unwrap_err().contains("requires admitted"));
        assert!(run(&args(&["--startup-material-preview","--startup-material-preview"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--offscreen","--source-startup-install","missing","--admit-startup-stable-packages","--seed","1","--startup-material-preview","--startup-unorm-texture"])).unwrap_err().contains("excludes"));
        assert!(run(&args(&["--startup-source-normals"])).unwrap_err().contains("requires admitted"));
        assert!(run(&args(&["--startup-source-normals", "--startup-source-normals"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--startup-unorm-texture"])).unwrap_err().contains("requires admitted"));
        assert!(run(&args(&["--startup-unorm-texture", "--startup-unorm-texture"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--admit-startup-stable-packages"])).unwrap_err().contains("together"));
        assert!(run(&args(&["--admit-startup-stable-packages", "--admit-startup-stable-packages"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--source-startup-install", "missing", "--admit-startup-stable-packages"])).unwrap_err().contains("requires --seed 1"));
        assert!(run(&args(&["--source-startup-install", "missing", "--admit-startup-stable-packages", "--seed", "1", "--creation", "x"])).unwrap_err().contains("excludes"));
        assert!(run(&args(&["--source-startup-install", "missing", "--source-startup-install", "missing"])).unwrap_err().contains("repeated"));
        assert!(admit_startup_triple([0xe219e5d3, 0x3d97a8e4, 0x40616200]).is_ok());
        assert!(admit_startup_triple([0xe219e5d2, 0x3d97a8e4, 0x40616200]).is_err());
    }

    #[test]
    fn effect_plane_requires_explicit_bounded_offscreen_inputs(){
        let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-effect-plane-install","missing"])).unwrap_err().contains("effect plane requires"));
        assert!(run(&args(&["--offscreen","--shot","unused.png","--source-effect-plane-install","missing","--admit-effect-plane-stable-packages","--source-effect-plane-rect","00000000,00000000,42A00000,42600000","--source-effect-plane-grid","3,0,0,0,3f0ccccd"])).unwrap_err().contains("exactly one"));
        assert_eq!(parse_effect_plane_rect("00000000,00000000,42A00000,42600000").unwrap(),[0.,0.,80.,56.]);
        for bad in ["0,0,80,56","00000000,00000000,00000000,42600000","7FC00000,00000000,42A00000,42600000"]{assert!(parse_effect_plane_rect(bad).is_err())}
    }

    #[test]
    fn effect_154_requires_complete_explicit_controls() {
        let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-effect-154-no-effect"])).unwrap_err().contains("effect 154 requires"));
        assert!(run(&args(&["--offscreen","--shot","unused.png","--source-effect-154-install","missing"])).unwrap_err().contains("effect 154 requires"));
        assert!(run(&args(&["--admit-effect-154-stable-packages","--admit-effect-154-stable-packages"])).unwrap_err().contains("repeated"));
    }

    #[test]
    fn saved_blend_requires_explicit_kind0_offscreen() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-kind0-saved-blend"]))
            .unwrap_err().contains("requires offscreen"));
        assert!(run(&args(&["--offscreen", "--shot", "unused.png", "--source-cell-install", "missing", "--source-cell-id", "7D00E384", "--source-kind0-saved-blend"]))
            .unwrap_err().contains("explicit FD24574B"));
        assert!(run(&args(&["--source-kind0-saved-blend", "--source-kind0-saved-blend"]))
            .unwrap_err().contains("repeated"));
    }

    #[test]
    fn fragment_preview_requires_explicit_kind0_and_excludes_other_preview() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-kind0-fragment-preview"]))
            .unwrap_err().contains("requires offscreen"));
        assert!(run(&args(&["--offscreen", "--shot", "unused.png", "--source-cell-install", "missing", "--source-cell-id", "7D00E384", "--source-kind0-fragment-preview"]))
            .unwrap_err().contains("explicit FD24574B"));
        assert!(run(&args(&["--offscreen", "--shot", "unused.png", "--source-cell-install", "missing", "--source-cell-id", "FD24574B", "--source-kind0-fragment-preview", "--source-kind0-saved-blend"]))
            .unwrap_err().contains("mutually exclusive"));
    }

    #[test]
    fn synthetic_palette_requires_the_explicit_fragment_preview() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-kind0-synthetic-palette"]))
            .unwrap_err().contains("requires --source-kind0-fragment-preview"));
        assert!(run(&args(&["--source-kind0-synthetic-palette", "--source-kind0-synthetic-palette"]))
            .unwrap_err().contains("repeated"));
    }

    #[test]
    fn source_ui_requires_complete_explicit_offscreen_fixture() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-ui-layout", "missing"])).unwrap_err().contains("requires install"));
        let all = ["--source-ui-install", "missing", "--source-ui-layout", "missing", "--source-ui-atlas", "missing", "--source-ui-fixture", "missing", "--source-ui-clip", "shader", "--source-ui-half-pixel", "0"];
        assert!(run(&args(&all)).unwrap_err().contains("requires --offscreen"));
        let mut bad = all.to_vec(); bad.extend(["--offscreen", "--shot", "target/source-ui/unused-test-shot.png"]); bad[9] = "unknown";
        assert!(run(&args(&bad)).unwrap_err().contains("clip must be"));
    }

    #[test]
    fn five_source_ui_requires_all_inputs_and_explicit_offscreen_admission() {
        let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-ui-five-cursor","3F000000,42A00000"])).unwrap_err().contains("complete five-segment"));
        assert!(run(&args(&["--source-ui-five-layout","missing"])).unwrap_err().contains("explicit admission"));
        let all=["--source-ui-five-layout","missing","--source-ui-five-atlas","missing","--source-ui-five-span","447A0000,42700000","--source-ui-five-origin","42200000,42200000","--source-ui-five-state","00000000","--source-ui-five-mask","01","--admit-five-ui-stable-packages-one-dispatch-no-other-mutation"];
        assert!(run(&args(&all)).unwrap_err().contains("requires --offscreen"));
        let mut bad=all.to_vec();bad.extend(["--offscreen","--shot","target/source-ui/unused-five-test.png"]);bad[5]="bad";
        assert!(run(&args(&bad)).unwrap_err().contains("coordinates require"));
    }
    #[test]
    fn source_audio_requires_explicit_selection_and_rejects_offscreen() {
        let args=|v:&[&str]|v.iter().map(|x|x.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-audio-package","missing"])).unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-audio-key","bad"])).unwrap_err().contains("16 hexadecimal"));
        assert!(run(&args(&["--offscreen","--source-audio-package","missing","--source-audio-key","021407EECD3FD77F","--source-audio-sha256","00"])).unwrap_err().contains("unavailable with --offscreen"));
    }
    #[test]
    fn event_audio_requires_all_admissions_and_excludes_direct_key() {
        let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let event=["--source-event-package","missing","--source-event-key","021407EE34C2FF0B","--source-event-sha256","00","--source-event-row","0","--source-audio-package","missing","--source-audio-sha256","00"];
        assert!(run(&args(&event)).unwrap_err().contains("loader admission"));
        let mut admitted=event.to_vec();
        admitted.push("--admit-event-loader-complete-unchanged-and-stable-packages");
        let mut mixed=admitted.clone(); mixed.extend(["--source-audio-key","021407EE12345678"]);
        assert!(run(&args(&mixed)).unwrap_err().contains("incompatible"));
        let mut offscreen=admitted.clone(); offscreen.push("--offscreen");
        assert!(run(&args(&offscreen)).unwrap_err().contains("unavailable with --offscreen"));
        let mut repeated=admitted; repeated.extend(["--source-event-row","1"]);
        assert!(run(&args(&repeated)).unwrap_err().contains("unknown Cell option"));
    }
    #[test]
    fn source_art_requires_explicit_pair_and_rejects_creation_conflict() {
        let args = |parts: &[&str]| parts.iter().map(|part| part.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-rival-cell-id", "7D00E384"]))
            .unwrap_err().contains("requires --source-cell-install"));
        assert!(run(&args(&["--source-rival-cell-id", "bad-id"]))
            .unwrap_err().contains("hexadecimal u32"));
        assert!(run(&args(&["--source-cell-install", "missing-install"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-cell-id", "7D00E384"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-cell-install", "missing-install", "--source-cell-id", "7D00E384", "--creation", "unused.rsc"]))
            .unwrap_err().contains("cannot be combined"));
        assert!(run(&args(&["--source-cell-id", "not-hex"]))
            .unwrap_err().contains("hexadecimal u32"));
        assert!(run(&args(&["--offscreen"]))
            .unwrap_err().contains("requires --shot"));
        assert!(run(&args(&["--source-creature-install", "missing-install"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-creature-instance", "40616200A1C46DF2"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-creature-instance", "A1C46DF2"]))
            .unwrap_err().contains("16 hexadecimal digits"));
        assert!(run(&args(&["--source-creature-install", "missing", "--source-creature-instance", "40616200A1C46DF2", "--creation", "unused.rsc"]))
            .unwrap_err().contains("cannot be combined"));
        assert!(run(&args(&["--source-cell-install", "missing", "--source-cell-id", "FD24574B", "--source-creature-install", "missing", "--source-creature-instance", "40616200A1C46DF2"]))
            .unwrap_err().contains("cannot be combined"));
    }

    #[test]
    fn rival_roster_is_explicit_bounded_and_unambiguous() {
        let args=|parts:&[&str]|parts.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert_eq!(parse_source_rival_roster("EFE530A1,4BC16582").unwrap(),vec![0xefe530a1,0x4bc16582]);
        for bad in ["", "123", "EFE530A1,", "EFE530A1,efe530a1", "00000000,00000001,00000002,00000003,00000004,00000005,00000006,00000007,00000008"] {assert!(parse_source_rival_roster(bad).is_err(),"{bad}");}
        assert!(run(&args(&["--source-rival-cell-roster","EFE530A1","--source-rival-cell-roster","4BC16582"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--source-rival-cell-id","EFE530A1","--source-rival-cell-id","4BC16582"])).unwrap_err().contains("repeated"));
        assert!(run(&args(&["--source-rival-cell-id","EFE530A1","--source-rival-cell-roster","4BC16582"])).unwrap_err().contains("mutually exclusive"));
        assert!(run(&args(&["--source-rival-cell-roster","EFE530A1,4BC16582"])).unwrap_err().contains("requires --source-cell-install"));
    }

    #[test]
    fn source_creature_basis_maps_axes_for_inspection() {
        let rotation = creature_inspection_rotation();
        assert!((rotation * Vec3::X - Vec3::Z).length() < 1e-6);
        assert!((rotation * Vec3::Y - Vec3::X).length() < 1e-6);
        assert!((rotation * Vec3::Z - Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn inspection_image_uploads_levels_in_source_order() {
        use crate::assets::{TextureData, TextureMipChain};
        let texture = TextureData { width: 2, height: 2, rgba: vec![7; 16] };
        let chain = TextureMipChain { levels: vec![
            TextureData { width: 2, height: 2, rgba: vec![7; 16] },
            TextureData { width: 1, height: 1, rgba: vec![9; 4] },
        ] };
        let sampler = ImageSamplerDescriptor::default();
        let image = inspection_image(&texture, Some(&chain), &sampler);
        assert_eq!(image.texture_descriptor.mip_level_count, 2);
        assert_eq!(image.data.as_ref().unwrap(), &[vec![7; 16], vec![9; 4]].concat());
        let mut controlled=inspection_image(&texture,Some(&chain),&sampler);
        assert!(apply_startup_unorm(&mut controlled,SourceKind::StartupCell,true));
        assert_eq!(controlled.texture_descriptor.format,TextureFormat::Rgba8Unorm);
        assert_eq!(controlled.texture_descriptor.mip_level_count,image.texture_descriptor.mip_level_count);
        assert_eq!(controlled.sampler,image.sampler);
        assert_eq!(controlled.data,image.data);
        for kind in [SourceKind::Cell,SourceKind::Creature] {
            let mut other=inspection_image(&texture,Some(&chain),&sampler);
            assert!(!apply_startup_unorm(&mut other,kind,true));
            assert_eq!(other.texture_descriptor.format,TextureFormat::Rgba8UnormSrgb);
            assert_eq!(other.data,controlled.data);
        }
        let mut default_startup=inspection_image(&texture,Some(&chain),&sampler);
        assert!(!apply_startup_unorm(&mut default_startup,SourceKind::StartupCell,false));
        assert_eq!(default_startup.texture_descriptor.format,TextureFormat::Rgba8UnormSrgb);
        assert_eq!(inspection_image(&texture, None, &sampler).texture_descriptor.mip_level_count, 1);
        assert_eq!(inspection_mip_lod_max(0, 2), 0.0);
        assert_eq!(inspection_mip_lod_max(1, 2), 1.0);
        assert_eq!(inspection_mip_lod_max(2, 2), 1.0);
        assert_eq!(creature_mip_chain_required(crate::assets::RASTER, 0).unwrap(), true);
        assert_eq!(creature_mip_chain_required(crate::assets::RW4, 0).unwrap(), false);
        assert!(creature_mip_chain_required(crate::assets::RW4, 1).is_err());
        assert!(creature_mip_chain_required(crate::assets::RW4, 2).is_err());
        assert!(creature_mip_chain_required(0, 0).is_err());
    }

    #[test]
    fn creature_inspection_denies_save_load_and_retains_mode_on_restart() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("rustspore-cell-creature-{}-{nonce}.rsp", std::process::id()));
        std::fs::write(&path, b"existing sentinel").unwrap();
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(SourceKind::Creature)
            .insert_resource(Game {
                world: CellWorld::new(14, Diet::Herbivore), seed: 14, diet: Diet::Herbivore,
                paused: false, elapsed: 0., notice: String::new(), rebuild: false,
                save_path: path.clone(), shot: None, demo_seconds: None, capture_frame: None, frames: 0,
            }).add_systems(Update, drive);
        let step = |app: &mut App, key: KeyCode| {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.reset_all(); input.press(key);
            drop(input);
            app.update();
        };
        step(&mut app, KeyCode::F5);
        assert!(app.world().resource::<Game>().notice.contains("Save unavailable"));
        assert_eq!(std::fs::read(&path).unwrap(), b"existing sentinel");
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, KeyCode::F9);
        assert!(app.world().resource::<Game>().notice.contains("Load unavailable"));
        assert_eq!(app.world().resource::<Game>().world, before);
        step(&mut app, KeyCode::KeyR);
        assert_eq!(app.world().resource::<Game>().world, CellWorld::new(14, Diet::Herbivore));
        assert!(*app.world().resource::<SourceKind>() == SourceKind::Creature);
        assert!(app.world().resource::<Game>().rebuild);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn offscreen_simulation_waits_and_stops_at_requested_duration() {
        assert_eq!(offscreen_step_delta(0.1, 0.0, 1.5, false), 0.0);
        assert_eq!(offscreen_step_delta(0.5, 0.0, 1.5, true), 0.1);
        let final_step = offscreen_step_delta(0.1, 1.48, 1.5, true);
        assert!((final_step - 0.02).abs() < 1e-12);
        assert_eq!(offscreen_step_delta(0.1, 1.5, 1.5, true), 0.0);
        assert_eq!(offscreen_step_delta(0.1, 1.6, 1.5, true), 0.0);
    }

    #[test]
    fn drive_waits_for_offscreen_simulation_latch() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<OffscreenSimulationReady>()
            .insert_resource(Game {
                world: CellWorld::new(14, Diet::Herbivore), seed: 14, diet: Diet::Herbivore,
                paused: false, elapsed: 0., notice: String::new(), rebuild: true,
                save_path: PathBuf::from("unused"), shot: None, demo_seconds: Some(1.0),
                capture_frame: None, frames: 0,
            })
            .add_systems(Update, drive);
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();
        assert_eq!(app.world().resource::<Game>().elapsed, 0.0);
        assert_eq!(app.world().resource::<Game>().world.tick, 0);
        app.world_mut().resource_mut::<OffscreenSimulationReady>().0 = true;
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();
        assert!((app.world().resource::<Game>().elapsed - 0.1).abs() < 1e-12);
        assert!(app.world().resource::<Game>().world.tick > 0);
    }

    #[test]
    fn keyboard_mapping_controls_the_same_simulation_input_as_gameplay() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        keys.press(KeyCode::Space);
        keys.press(KeyCode::ShiftLeft);
        let input = keyboard_input(&keys);
        assert_eq!(input.thrust, 1.0);
        assert_eq!(input.turn, -1.0);
        assert!(input.bite && input.boost);
        keys.press(KeyCode::KeyS);
        keys.press(KeyCode::KeyA);
        let input = keyboard_input(&keys);
        assert_eq!(input.thrust, 0.0);
        assert_eq!(input.turn, 0.0);
    }

    #[test]
    fn save_file_replacement_roundtrips_and_keeps_old_data_on_failed_destination() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-cell-save-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("cell.rsp");
        let world = CellWorld::new(42, Diet::Carnivore);
        let bytes = world.snapshot().to_bytes().unwrap();
        write_save(&path, b"previous").unwrap();
        write_save(&path, &bytes).unwrap();
        let loaded = CellWorld::from_snapshot(
            CellSnapshot::from_bytes(&std::fs::read(&path).unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(loaded.seed(), 42);
        assert_eq!(loaded.player.diet, Diet::Carnivore);
        assert!(write_save(&path.join("invalid"), b"must not overwrite").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    #[test]
    fn gameplay_controls_pause_restart_save_load_and_failed_load() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-cell-controls-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("cell.rsp");
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game {
                world: CellWorld::new(14, Diet::Herbivore),
                seed: 14,
                diet: Diet::Herbivore,
                paused: false,
                elapsed: 0.,
                notice: String::new(),
                rebuild: true,
                save_path: path.clone(),
                shot: None,
                demo_seconds: None,
                capture_frame: None,
                frames: 0,
            })
            .add_systems(Update, drive);
        let step = |app: &mut App, seconds: f64, pressed: &[KeyCode]| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            for key in pressed {
                keys.press(*key);
            }
            drop(keys);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f64(seconds));
            app.update();
        };
        step(&mut app, 0.1, &[KeyCode::KeyW]);
        let moved = app.world().resource::<Game>().world.clone();
        assert!(moved.player.position.x > 0.0 && moved.tick > 0);
        step(&mut app, 0.1, &[KeyCode::KeyP, KeyCode::KeyW]);
        assert_eq!(app.world().resource::<Game>().world, moved);
        step(&mut app, 0., &[KeyCode::KeyP]);
        step(&mut app, 0., &[KeyCode::F5]);
        let saved = app
            .world()
            .resource::<Game>()
            .world
            .snapshot()
            .to_bytes()
            .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        step(&mut app, 0.1, &[KeyCode::KeyW]);
        assert_ne!(app.world().resource::<Game>().world.tick, moved.tick);
        step(&mut app, 0., &[KeyCode::F9]);
        assert_eq!(
            app.world()
                .resource::<Game>()
                .world
                .snapshot()
                .to_bytes()
                .unwrap(),
            saved
        );
        step(&mut app, 0., &[KeyCode::Digit2]);
        assert_eq!(
            app.world().resource::<Game>().world.player.diet,
            Diet::Carnivore
        );
        step(&mut app, 0., &[KeyCode::F9]);
        step(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world,
            CellWorld::new(14, Diet::Herbivore)
        );
        std::fs::write(&path, b"broken save").unwrap();
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, 0., &[KeyCode::F9]);
        let game = app.world().resource::<Game>();
        assert_eq!(game.world, before);
        assert!(game.notice.starts_with("Load failed:"));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    fn authored_creation() -> Creation {
        use spore_core::editor::{AuthoredPart, PartArchetype};
        let mut creation = Creation::new(CreationKind::Cell, "Painted swimmer", 100).unwrap();
        for (id, kind) in [
            (2, PartArchetype::CarnivoreMouth),
            (3, PartArchetype::Flagellum),
            (4, PartArchetype::Eye),
        ] {
            creation
                .parts
                .push(AuthoredPart::new(id, format!("Part {id}"), kind, Some(1)));
        }
        creation.palette[0] = [0.8, 0.2, 0.4];
        creation
    }

    #[test]
    fn source_inspection_rejects_authored_snapshot_without_changing_it() {
        let mut world = CellWorld::new(77, Diet::Carnivore);
        world.apply_creation(authored_creation()).unwrap();
        assert!(validate_source_snapshot(&world, true).unwrap_err().contains("authored creation"));
        assert!(validate_source_snapshot(&world, false).is_ok());
        assert!(validate_source_snapshot(&CellWorld::new(14, Diet::Herbivore), true).is_ok());
    }

    #[test]
    fn authored_app_save_load_restart_preserves_appearance_stats_and_mouth_diet() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-cell-authored-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("cell.rsp");
        let creation = authored_creation();
        let mut world = CellWorld::new(77, Diet::Carnivore);
        world.apply_creation(creation.clone()).unwrap();
        let expected = world.loadout.clone();
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game {
                world,
                seed: 77,
                diet: Diet::Carnivore,
                paused: false,
                elapsed: 0.,
                notice: String::new(),
                rebuild: true,
                save_path: path.clone(),
                shot: None,
                demo_seconds: None,
                capture_frame: None,
                frames: 0,
            })
            .add_systems(Update, drive);
        let step = |app: &mut App, keys: &[KeyCode]| {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.reset_all();
            for key in keys {
                input.press(*key);
            }
            drop(input);
            app.update();
        };
        step(&mut app, &[KeyCode::F5]);
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.world = CellWorld::new(999, Diet::Herbivore);
            game.seed = 999;
            game.diet = Diet::Herbivore;
        }
        step(&mut app, &[KeyCode::F9]);
        assert_eq!(
            app.world().resource::<Game>().world.creation,
            Some(creation.clone())
        );
        assert_eq!(app.world().resource::<Game>().world.loadout, expected);
        step(&mut app, &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world.creation,
            Some(creation.clone())
        );
        assert_eq!(app.world().resource::<Game>().world.loadout, expected);
        assert_eq!(app.world().resource::<Game>().world.seed(), 77);
        step(&mut app, &[KeyCode::Digit1]);
        let game = app.world().resource::<Game>();
        assert_eq!(game.world.creation, Some(creation));
        assert_eq!(game.diet, Diet::Carnivore);
        assert!(game.notice.contains("mouth"));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    #[test]
    fn creation_cli_rejects_invalid_wrong_kind_and_conflicting_diet_before_opening_gpu() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rustspore-cell-cli-{}-{nonce}.rsc",
            std::process::id()
        ));
        std::fs::write(&path, b"invalid").unwrap();
        let args = vec!["--creation".into(), path.to_string_lossy().into_owned()];
        assert!(run(&args).is_err());
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: Creation::new(CreationKind::Creature, "Wrong kind", 100).unwrap(),
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        assert!(run(&args).unwrap_err().contains("Cell creation"));
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: authored_creation(),
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        let mut conflict = args.clone();
        conflict.extend(["--diet".into(), "herbivore".into()]);
        assert!(run(&conflict).unwrap_err().contains("contradicts"));
        let world = configured_world(14, Diet::Herbivore, false, Some(&path)).unwrap();
        assert_eq!(world.player.diet, Diet::Carnivore);
        assert!(world
            .food
            .iter()
            .take(8)
            .all(|food| food.kind == FoodKind::Meat));
        std::fs::write(&path, vec![0; MAX_SAVE_BYTES + 1]).unwrap();
        assert!(run(&args).unwrap_err().contains("bounded"));
        std::fs::remove_file(path).unwrap();
    }
}
