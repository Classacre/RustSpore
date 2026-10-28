//! Shipped Cell-stage creations, loaded from the user's install.
//!
//! This path uses the baked gmdl and its own painted atlas. It never substitutes
//! assembled creature-part geometry, generated paint, or procedural artwork.
//! The measured chain is `.cell` -> `structure` -> Cell creation prop -> gmdl.
//! Simulation, source shaders, effects and camera fidelity remain separate work.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::mesh::MeshData;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
        renderer::RenderAdapterInfo,
        view::screenshot::save_to_disk,
    },
};

use crate::assets::{TextureData, GMDL, PNG, PROP, RASTER};

pub(crate) const CELL: u32 = 0xDFAD_9F51;
pub(crate) const STRUCTURE: u32 = 0x4B9E_F6DC;
pub(crate) const JAWHEAD: u32 = 0x7D00_E384;
const MAX_RESOURCE_BYTES: u32 = 32 * 1024 * 1024;
const CELL_EDITOR_SETUP: u64 = 0x4060_0100_3615_A30B;
const CAMERA_NAMESPACE: u64 = 0x4041_0100_0000_0000;
const THUMBNAIL_CAMERA_KEY: u32 = 0xB02D_871C;

/// Full on-disk identity, including the package providing the overlay winner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceIdentity {
    pub package: PathBuf,
    pub key: NativeResourceKey,
}

/// Only the fixed Cell fields used by the asset chain are interpreted here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CellRecord {
    pub structure_id: u32,
    /// Developer labels include PLACEHOLDER names and can be empty.
    pub embedded_name: String,
    /// Preserved behavior bytes, whose semantics are not inferred by this loader.
    pub raw: Vec<u8>,
}

impl CellRecord {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let definition = spore_core::cell_definition::parse(bytes)
            .map_err(|error| error.to_string())?;
        let embedded_name = definition.embedded_label()
            .map_err(|_| "invalid UTF-16 Cell developer label".to_string())?;
        Ok(Self {
            structure_id: definition.structure_id(),
            embedded_name,
            raw: bytes.to_vec(),
        })
    }
}

/// A measured 40-byte structure entry. Most fields are deliberately unnamed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructureEntry {
    pub kind: u32,
    pub reference_id: u32,
    pub raw_words: [u32; 10],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CellStructure {
    pub header_words: [u32; 5],
    pub entries: Vec<StructureEntry>,
}

impl CellStructure {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 28 {
            return Err("truncated Cell structure header".to_string());
        }
        let offset = le32(bytes, 20) as usize;
        let count = le32(bytes, 24) as usize;
        if offset != 28 || count > 1024 || bytes.len() != 28 + count * 40 {
            return Err(format!(
                "unsupported Cell structure layout: offset={offset}, count={count}, bytes={}",
                bytes.len()
            ));
        }
        let mut entries = Vec::with_capacity(count);
        for chunk in bytes[28..].chunks_exact(40) {
            let raw_words = std::array::from_fn(|i| le32(chunk, i * 4));
            entries.push(StructureEntry {
                kind: raw_words[1],
                reference_id: raw_words[4],
                raw_words,
            });
        }
        Ok(Self {
            header_words: std::array::from_fn(|i| le32(bytes, i * 4)),
            entries,
        })
    }

    fn baked_creation(&self) -> Result<u64, String> {
        let mut baked = self.entries.iter().filter(|entry| entry.kind == 3);
        let Some(entry) = baked.next() else {
            return Err("Cell structure has no baked creation (effect/simple-model path unsupported)".to_string());
        };
        if baked.next().is_some() || entry.reference_id == 0 {
            return Err("Cell structure has ambiguous or empty baked creation references".to_string());
        }
        Ok(0x4061_6200_0000_0000 | u64::from(entry.reference_id))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SourceCellDefinition {
    pub cell: SourceIdentity,
    pub record: CellRecord,
    pub structure: SourceIdentity,
    pub structure_data: CellStructure,
    pub creation_instance: u64,
}

pub(crate) struct SourceCellAsset {
    pub definition: SourceCellDefinition,
    pub model_prop: SourceIdentity,
    pub model: SourceIdentity,
    pub mesh: MeshData,
    /// Native shader inputs remain separate from the inspection PBR material.
    pub native_materials: Vec<SourceCellMaterial>,
    pub skin_source: SourceIdentity,
    pub skin: TextureData,
    pub thumbnail: Option<(SourceIdentity, TextureData)>,
    /// Source effects are retained as references, not silently replaced by art.
    pub pending_structure_entries: Vec<StructureEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct SourceCellMaterial {
    pub material_id: u32,
    pub declaration_features: u32,
    pub parameters: Option<spore_core::shader_binding::MaterialParams>,
    pub textures: Vec<spore_core::shader_binding::TextureBinding>,
}

pub(crate) fn source_skin_sampler(asset: &SourceCellAsset) -> Result<ImageSamplerDescriptor, String> {
    let mut settings = None;
    for material in &asset.native_materials {
        let binding = material.textures.iter().rev().find(|binding| binding.sampler == 0)
            .ok_or("Cell material has no explicit source skin sampler")?;
        if binding.raster_instance != asset.skin_source.key.scoped_instance() {
            return Err("Cell preview needs separate draws for different source skin textures".to_string());
        }
        let values = (binding.address_mode(), binding.min_mag_filter(), binding.mip_filter(), binding.availability());
        if settings.is_some_and(|previous| previous != values) {
            return Err("Cell preview needs separate draws for different source samplers".to_string());
        }
        settings = Some(values);
    }
    let (Some(address), Some(filter), Some(mip), Some(available)) = settings
        .ok_or("Cell preview has no source material bindings")? else {
            return Err("Cell sampler inherits unestablished native state".to_string());
        };
    if available == 0 { return Err("source Cell skin sampler is disabled".to_string()); }
    let address = match address {
        1 => ImageAddressMode::Repeat,
        2 => ImageAddressMode::MirrorRepeat,
        3 => ImageAddressMode::ClampToEdge,
        _ => return Err("source Cell sampler addressing is unsupported".to_string()),
    };
    let filter_mode = |value| match value {
        1 => Ok(ImageFilterMode::Nearest),
        2 => Ok(ImageFilterMode::Linear),
        _ => Err("source Cell sampler filtering is unsupported".to_string()),
    };
    Ok(ImageSamplerDescriptor {
        address_mode_u: address, address_mode_v: address, address_mode_w: address,
        mag_filter: filter_mode(filter)?, min_filter: filter_mode(filter)?,
        mipmap_filter: filter_mode(mip)?, ..default()
    })
}

#[derive(Debug, Clone)]
struct SourceCellLighting {
    source: SourceIdentity,
    /// Saved RGB coefficients, before native normalization/rotation/upload.
    coefficients: Vec<[f32; 3]>,
}

/// Only camera fields established by the native reader and matrix consumers.
/// The source thumbnail's model framing and capture-time offsets remain open.
#[derive(Debug, Clone)]
struct SourceOrbitCamera {
    source: SourceIdentity,
    parent: SourceIdentity,
    azimuth_degrees: f32,
    elevation_degrees: f32,
    source_distance: f32,
    source_horizontal_offset: f32,
    source_vertical_offset: f32,
    fov_radians: f32,
    near: f32,
    far: f32,
}

impl SourceOrbitCamera {
    fn from_props(source: SourceIdentity, parent: SourceIdentity,
                  local: &spore_core::prop::PropTree, inherited: &spore_core::prop::PropTree)
        -> Result<Self, String>
    {
        use spore_core::prop::PropValue;
        let value = |id| local.value(id).or_else(|| inherited.value(id));
        let scalar = |id| match value(id) {
            Some(PropValue::Float(number)) if number.is_finite() => Ok(*number),
            _ => Err(format!("source camera property {id:08X} requires a finite FLOAT")),
        };
        // Native camera manager reads this class ID; the factory registers
        // EDD3BB -> 7DA4C0 -> 7D9CE0. Other camera classes have different math.
        if value(0x00ED_3928) != Some(&PropValue::Uint32(0x00ED_D3BB)) {
            return Err("source thumbnail does not select the measured orbit camera class EDD3BB".to_string());
        }
        let camera = Self {
            source, parent,
            source_distance: scalar(0x00C7_C4FB)?,
            elevation_degrees: scalar(0x00C7_C4FC)?,
            azimuth_degrees: scalar(0x00C7_C4FD)?,
            source_horizontal_offset: scalar(0x6FDA_2E1C)?,
            source_vertical_offset: scalar(0x8FDA_2E23)?,
            fov_radians: scalar(0x044C_6220)?,
            near: scalar(0x0110_2B20)?, far: scalar(0x0110_2B2F)?,
        };
        if camera.near <= 0. || camera.far <= camera.near
            || !(0.001..std::f32::consts::PI - 0.001).contains(&camera.fov_radians)
            || camera.source_distance <= 0.
            || camera.azimuth_degrees.abs() > 3600. || camera.elevation_degrees.abs() > 3600.
        {
            return Err("unsupported source camera planes, field of view or orbit parameters".to_string());
        }
        Ok(camera)
    }

    /// Native identity basis is right +X, forward +Y, up +Z. 7DA320 applies
    /// local Z(-FD), then X(FC); its third orbit angle is zero for this preset.
    fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let radians = 0.017_453_292_f32;
        let rotation = Quat::from_rotation_z(-self.azimuth_degrees * radians)
            * Quat::from_rotation_x(self.elevation_degrees * radians);
        (rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z)
    }

    /// 7C4CD0 uses tan(FOV/2); exact square aspect skips the 0.75 multiplier.
    /// Ordered non-square aspect applies it. The source FOV is not blindly
    /// interchangeable with Bevy's vertical field of view.
    fn vertical_fov(&self, aspect: f32) -> Result<f32, String> {
        if !aspect.is_finite() || aspect <= 0. {
            return Err("source camera aspect must be finite and positive".to_string());
        }
        let tangent = (self.fov_radians * 0.5).tan()
            * if aspect == 1. { 1. } else { 0.75 };
        Ok(2. * tangent.atan())
    }

    fn projection(&self, aspect: f32) -> Result<PerspectiveProjection, String> {
        Ok(PerspectiveProjection {
            fov: self.vertical_fov(aspect)?, aspect_ratio: aspect,
            near: self.near, far: self.far,
            // Bevy0.18 stores this separately. Retaining Default's -0.1 while
            // changing near would silently override the recovered source plane.
            near_clip_plane: Vec4::new(0., 0., -1., -self.near),
        })
    }
}

/// A deliberately separate inspection fit. These viewport margins and the
/// normalized mesh scale are not attributed to the original thumbnail renderer.
fn inspection_camera_fit(data: &MeshData, camera: &SourceOrbitCamera, aspect: f32)
    -> Result<(Transform, Transform, f32), String>
{
    let (min, max) = data.bounds().ok_or("source geometry has no bounds")?;
    let min = Vec3::new(min.x, min.y, min.z);
    let max = Vec3::new(max.x, max.y, max.z);
    let centre = (min + max) * 0.5;
    let scale = 1. / data.radius();
    let (right, forward, up) = camera.basis();
    let fov = camera.vertical_fov(aspect)?;
    let tangent_y = (fov * 0.5).tan();
    let tangent_x = tangent_y * aspect;
    let (left, right_edge, bottom, top) = (-0.94_f32, 0.44_f32, -0.90_f32, 0.34_f32);
    let (centre_x, centre_y) = (-0.25_f32, -0.28_f32);
    let mut distance = camera.near * 2.;
    let mut max_depth = 0_f32;
    for x in [min.x, max.x] { for y in [min.y, max.y] { for z in [min.z, max.z] {
        let point = (Vec3::new(x, y, z) - centre) * scale;
        let px = point.dot(right);
        let py = point.dot(up);
        let depth = point.dot(forward);
        distance = distance.max(camera.near * 2. - depth)
            .max((left * depth - px / tangent_x) / (centre_x - left))
            .max((px / tangent_x - right_edge * depth) / (right_edge - centre_x))
            .max((bottom * depth - py / tangent_y) / (centre_y - bottom))
            .max((py / tangent_y - top * depth) / (top - centre_y));
        max_depth = max_depth.max(depth);
    } } }
    distance *= 1.03;
    if !distance.is_finite() || distance + max_depth >= camera.far {
        return Err("source camera far plane cannot contain the inspection bounds fit".to_string());
    }
    let offset = right * (centre_x * distance * tangent_x)
        + up * (centre_y * distance * tangent_y);
    let model = Transform::from_translation(-centre * scale + offset).with_scale(Vec3::splat(scale));
    let view = Transform::from_translation(-forward * distance).looking_to(forward, up);
    Ok((model, view, fov))
}

#[derive(Debug, Default)]
pub(crate) struct CellCensus {
    pub definitions: usize,
    pub structures_resolved: usize,
    pub baked_definitions: usize,
    pub available_baked_definitions: usize,
    pub unique_available_creations: usize,
    pub missing_baked_props: usize,
    pub baked_loaded: usize,
    pub thumbnails_loaded: usize,
    pub failures: Vec<String>,
    pub unsupported: Vec<String>,
}

/// Index-only reads keep a source preview from loading gigabytes of packages.
pub(crate) struct SourceCellLibrary {
    packages: Vec<NativePackageIndex>,
}

impl SourceCellLibrary {
    pub fn open_install(install: &Path) -> Result<Self, String> {
        // Explicit inspection order. Active retail product/tie registration remains
        // a separate resource-manager gate.
        let mut packages = Vec::new();
        for name in ["PatchData.package", "Spore_Game.package", "Spore_Content.package", "Spore_Graphics.package"] {
            let path = install.join("Data").join(name);
            packages.push(NativePackageIndex::open(&path)
                .map_err(|e| format!("could not index {}: {e}", path.display()))?);
        }
        Ok(Self { packages })
    }

    pub fn cell_ids(&self) -> Vec<u32> {
        self.packages.iter().flat_map(|package| package.entries())
            .filter(|entry| entry.key.type_id == CELL && entry.key.group_id == 0)
            .map(|entry| entry.key.instance_id)
            .collect::<BTreeSet<_>>().into_iter().collect()
    }

    fn read(&self, type_id: u32, instance: u64) -> Result<(SourceIdentity, Vec<u8>), String> {
        for package in &self.packages {
            if let Some(entry) = package.index().find_unique(NativeResourceKey::from_scoped_instance(type_id, instance))
                .map_err(|error| error.to_string())? {
                if entry.decompressed_size > MAX_RESOURCE_BYTES {
                    return Err(format!("source resource {} exceeds 32 MiB limit", entry.key));
                }
                let identity = SourceIdentity { package: package.path().to_path_buf(), key: entry.key };
                let bytes = package.read(entry).map_err(|e| format!("{}: {e}", entry.key))?;
                if bytes.len() != entry.decompressed_size as usize {
                    return Err(format!("source resource {} has inconsistent decoded size", entry.key));
                }
                return Ok((identity, bytes));
            }
        }
        Err(format!("source resource missing {type_id:08X}:{instance:016X}"))
    }

    pub fn definition(&self, id: u32) -> Result<SourceCellDefinition, String> {
        let (cell, bytes) = self.read(CELL, u64::from(id))?;
        let record = CellRecord::parse(&bytes)?;
        let (structure, bytes) = self.read(STRUCTURE, u64::from(record.structure_id))?;
        let structure_data = CellStructure::parse(&bytes)?;
        let creation_instance = structure_data.baked_creation()?;
        Ok(SourceCellDefinition { cell, record, structure, structure_data, creation_instance })
    }

    pub fn load(&self, id: u32) -> Result<SourceCellAsset, String> {
        self.load_definition(self.definition(id)?)
    }

    fn thumbnail_camera(&self) -> Result<SourceOrbitCamera, String> {
        let (_, bytes) = self.read(PROP, CELL_EDITOR_SETUP)?;
        let setup = exact_camera_prop(&bytes)?;
        let (instance, kind) = setup.key(THUMBNAIL_CAMERA_KEY)
            .ok_or("Cell Editor Setup has no explicit source thumbnail camera key")?;
        if instance > u64::from(u32::MAX) || kind != PROP {
            return Err("unsupported Cell Editor Setup thumbnail camera reference".to_string());
        }
        // Camera manager 7C64F0 selects properties from namespace 40410100;
        // editor setup and parent keys carry that namespace's low ID only.
        let (source, bytes) = self.read(PROP, CAMERA_NAMESPACE | instance)?;
        let local = exact_camera_prop(&bytes)?;
        let (parent_instance, parent_kind) = local.key(spore_core::prop::idents::PARENT)
            .ok_or("source thumbnail camera has no measured template parent")?;
        if parent_instance > u64::from(u32::MAX) || parent_kind != 0 {
            return Err("unsupported source thumbnail template parent reference".to_string());
        }
        let (parent, bytes) = self.read(PROP, CAMERA_NAMESPACE | parent_instance)?;
        let inherited = exact_camera_prop(&bytes)?;
        SourceOrbitCamera::from_props(source, parent, &local, &inherited)
    }

    fn editor_lighting(&self, paint_mode: bool) -> Result<SourceCellLighting, String> {
        use spore_core::prop::{types, PropValue, ARRAY_MARKER_1, ARRAY_MARKER_2};
        let (_, bytes) = self.read(PROP, CELL_EDITOR_SETUP)?;
        let setup = exact_camera_prop(&bytes)?;
        // Native 5791F0 chooses the normal key and replaces it in mode1.
        // 70DAB0 loads light properties in namespace 40200100.
        let id = if paint_mode { 0x0141_40DD } else { 0xB00D_B8C8 };
        let (low, kind) = setup.key(id)
            .ok_or_else(|| format!("Cell editor has no explicit lighting key {id:08X}"))?;
        if low > u64::from(u32::MAX) || kind != 0 {
            return Err("unsupported source editor lighting reference".to_string());
        }
        let (source, bytes) = self.read(PROP, 0x4020_0100_0000_0000 | low)?;
        let lighting = exact_camera_prop(&bytes)?;
        let item = lighting.items.iter().find(|i| i.id == 0x0100_EAC5)
            .ok_or("Cell lighting has no saved coefficient array")?;
        if item.prop_type != types::CRGB
            || !matches!(item.specifier, ARRAY_MARKER_1 | ARRAY_MARKER_2)
            || item.values.len() != 25
        {
            return Err("unsupported Cell lighting coefficient layout".to_string());
        }
        let coefficients = item.values.iter().map(|value| match value {
            PropValue::ColorRGB(r, g, b) if r.is_finite() && g.is_finite() && b.is_finite()
                => Ok([*r, *g, *b]),
            _ => Err("Cell lighting needs finite float RGB coefficients".to_string()),
        }).collect::<Result<_, _>>()?;
        Ok(SourceCellLighting { source, coefficients })
    }

    pub fn load_definition(&self, definition: SourceCellDefinition) -> Result<SourceCellAsset, String> {
        let instance = definition.creation_instance;
        let (model_prop, bytes) = self.read(PROP, instance)?;
        let prop = spore_core::prop::parse(&bytes).map_err(|e| e.to_string())?;
        if !prop.exact || prop.consumed != bytes.len() {
            return Err(format!("Cell model prop {instance:016X} is not parsed exactly"));
        }
        let (model_instance, model_type) = prop.key(spore_core::prop::idents::MODEL_LOD0)
            .ok_or_else(|| format!("Cell model prop {instance:016X} has no explicit LOD0"))?;
        if model_type != GMDL {
            return Err(format!("unsupported Cell model type {model_type:08X}; expected baked gmdl"));
        }
        let (model, bytes) = self.read(model_type, model_instance)?;
        let parsed = spore_core::gmdl::parse(&bytes).map_err(|e| e.to_string())?;
        // Gmdl::consumed is the geometry-core endpoint, before its optional
        // trailing material/skin section. exact covers that trailing section.
        if !parsed.exact || parsed.total_size != bytes.len() {
            return Err(format!("Cell gmdl {model_instance:016X} is not parsed exactly"));
        }
        let tail = spore_core::gmdl_skin::parse(&bytes, &parsed).map_err(|e| e.to_string())?;
        if tail.as_ref().is_some_and(|tail| tail.materials.len() != parsed.meshes.len()) {
            return Err("Cell material slots do not match the source meshes".to_string());
        }
        let mut native_materials = Vec::with_capacity(parsed.meshes.len());
        for (i, source_mesh) in parsed.meshes.iter().enumerate() {
            let vb = parsed.vertex_buffers.get(source_mesh.vertex_index as usize)
                .ok_or("Cell mesh has an invalid source vertex buffer")?;
            let declaration = parsed.vertex_formats.get(vb.format_index as usize)
                .ok_or("Cell mesh has an invalid source declaration")?;
            let parameters = tail.as_ref().and_then(|tail| tail.materials.get(i))
                .and_then(|material| material.shader_data.iter().rev().find(|field| field.id == 0x210))
                .map(|field| spore_core::shader_binding::MaterialParams::decode(&field.payload))
                .transpose().map_err(|e| e.to_string())?;
            let textures = tail.as_ref().and_then(|tail| tail.materials.get(i))
                .and_then(|material| material.shader_data.iter().rev().find(|field| field.id == 0x20D))
                .map(|field| spore_core::shader_binding::texture_bindings(&field.payload))
                .transpose().map_err(|e| e.to_string())?.unwrap_or_default();
            native_materials.push(SourceCellMaterial {
                material_id: source_mesh.material_id as u32,
                declaration_features: declaration.native_shader_features().map_err(|e| e.to_string())?,
                parameters,
                textures,
            });
        }
        let mesh = spore_core::mesh::from_gmdl(&bytes, &parsed).map_err(|e| e.to_string())?;
        mesh.validate().map_err(|e| e.to_string())?;
        // Measured Cell atlas namespace. The adjacent 40612901 data map is not
        // used as an albedo fallback.
        let skin_instance = 0x4061_2900_0000_0000 | (instance & 0xFFFF_FFFF);
        let (skin_source, bytes) = self.read(RASTER, skin_instance)?;
        let raster = spore_core::raster::parse(&bytes).map_err(|e| e.to_string())?;
        if raster.width == 0 || raster.height == 0 || raster.width > 4096 || raster.height > 4096 {
            return Err("Cell painted atlas dimensions exceed limits".to_string());
        }
        let skin = TextureData { width: raster.width, height: raster.height,
            rgba: raster.decode_rgba().map_err(|e| e.to_string())? };
        let thumbnail = match self.read(PNG, instance) {
            Ok((identity, bytes)) => Some((identity, decode_thumbnail(&bytes)?)),
            Err(_) => None,
        };
        let pending_structure_entries = definition.structure_data.entries.iter()
            .filter(|entry| entry.kind != 3).cloned().collect();
        Ok(SourceCellAsset { definition, model_prop, model, mesh, native_materials, skin_source, skin,
            thumbnail, pending_structure_entries })
    }

    pub fn census(&self) -> CellCensus {
        let mut census = CellCensus::default();
        let mut available_creations = BTreeSet::new();
        for id in self.cell_ids() {
            census.definitions += 1;
            let result = (|| {
                let (_, bytes) = self.read(CELL, u64::from(id))?;
                let record = CellRecord::parse(&bytes)?;
                let (_, bytes) = self.read(STRUCTURE, u64::from(record.structure_id))?;
                let structure = CellStructure::parse(&bytes)?;
                census.structures_resolved += 1;
                if !structure.entries.iter().any(|entry| entry.kind == 3) {
                    census.unsupported.push(format!("{id:08X}: structure uses no baked creation"));
                    return Ok(());
                }
                census.baked_definitions += 1;
                let instance = structure.baked_creation()?;
                if self.packages.iter().any(|package| package.index()
                    .find_unique(NativeResourceKey::from_scoped_instance(PROP, instance)).ok().flatten().is_some()) {
                    census.available_baked_definitions += 1;
                    available_creations.insert(instance);
                } else {
                    census.missing_baked_props += 1;
                }
                let asset = self.load(id)?;
                census.baked_loaded += 1;
                census.thumbnails_loaded += usize::from(asset.thumbnail.is_some());
                Ok::<_, String>(())
            })();
            if let Err(error) = result { census.failures.push(format!("{id:08X}: {error}")); }
        }
        census.unique_available_creations = available_creations.len();
        census
    }
}

fn exact_camera_prop(bytes: &[u8]) -> Result<spore_core::prop::PropTree, String> {
    let tree = spore_core::prop::parse(bytes).map_err(|e| e.to_string())?;
    let declared = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
    if !tree.exact || tree.consumed != bytes.len() || tree.items.len() != declared
        || tree.items.iter().any(|item| item.anonymous)
        || tree.items.iter().map(|item| item.id).collect::<BTreeSet<_>>().len() != declared
    {
        return Err("source camera prop does not have exact named framing".to_string());
    }
    Ok(tree)
}

fn decode_thumbnail(bytes: &[u8]) -> Result<TextureData, String> {
    let reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = reader;
    reader.limits(limits);
    let rgba = reader.decode().map_err(|e| format!("Cell thumbnail PNG: {e}"))?.to_rgba8();
    Ok(TextureData { width: rgba.width(), height: rgba.height(), rgba: rgba.into_raw() })
}

fn le32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// Isolated source-art inspection, always offscreen. This is not a retail scene
/// recreation: its plain background and preview lighting are explicit controls.
pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let options = PreviewOptions::parse(args)?;
    let source = SourceCellLibrary::open_install(&options.install)?;
    if options.audit {
        let census = source.census();
        println!("SOURCE_CELL_CENSUS definitions={} structures={} baked_references={} available_baked={} unique_available={} missing_baked_props={} loaded={} thumbnails={} unsupported={} failures={}",
            census.definitions, census.structures_resolved, census.baked_definitions,
            census.available_baked_definitions, census.unique_available_creations, census.missing_baked_props,
            census.baked_loaded, census.thumbnails_loaded, census.unsupported.len(), census.failures.len());
        for error in &census.failures { println!("SOURCE_CELL_UNRESOLVED {error}"); }
        if census.baked_loaded != census.available_baked_definitions {
            return Err("one or more available baked Cell assets failed to load exactly".to_string());
        }
        if options.shot.is_none() { return Ok(()); }
    }
    let asset = source.load(options.cell)?;
    if !options.pbr_inspection {
        let path = options.shot.as_ref().ok_or("source Cell capture requires --shot <new.png>")?;
        println!("SOURCE_CELL_ART cell={} structure={} creation={:016X} model={} skin={} pending_effects={} renderer=original-d3d9 fit=inspection diagnostic_skin_translation={}",
            asset.definition.cell.key, asset.definition.structure.key, asset.definition.creation_instance,
            asset.model.key, asset.skin_source.key, asset.pending_structure_entries.len(), options.diagnostic_skin_translation);
        #[cfg(all(windows, feature = "native_renderer"))]
        return crate::native_cell_renderer::capture_key(&options.install, path,
            asset.model.key, options.diagnostic_skin_translation)
            .map_err(|e| e.to_string());
        #[cfg(not(all(windows, feature = "native_renderer")))]
        return Err("Original shader capture requires Windows native rendering. Use --pbr-inspection for the separate diagnostic view.".to_string());
    }
    let camera = source.thumbnail_camera()?;
    let skin_sampler = source_skin_sampler(&asset)?;
    // Retain and validate source lighting without passing these raw coefficients
    // to PBR as an approximation of the native SH shader.
    for paint_mode in [false, true] {
        let lighting = source.editor_lighting(paint_mode)?;
        println!("SOURCE_CELL_LIGHTING paint_mode={paint_mode} key={} coefficients={} first={:?} raw_source_only=true",
            lighting.source.key, lighting.coefficients.len(), lighting.coefficients[0]);
    }
    for (i, material) in asset.native_materials.iter().enumerate() {
        println!("SOURCE_CELL_NATIVE_MATERIAL mesh={i} id={:08X} declaration_features={:08X} parameters={:?} source_shader_bound=false",
            material.material_id, material.declaration_features, material.parameters.map(|p| p.float4()));
        for texture in &material.textures {
            println!("SOURCE_CELL_TEXTURE mesh={i} sampler={} raster={:016X} address={:?} min_mag={:?} mip={:?} availability={:?}",
                texture.sampler, texture.raster_instance, texture.address_mode(), texture.min_mag_filter(),
                texture.mip_filter(), texture.availability());
        }
    }
    inspection_camera_fit(&asset.mesh, &camera, 1280. / 800.)?;
    println!("SOURCE_CELL_ART cell={} structure={} creation={:016X} model={} skin={} vertices={} triangles={} pending_effects={}",
        asset.definition.cell.key, asset.definition.structure.key, asset.definition.creation_instance,
        asset.model.key, asset.skin_source.key, asset.mesh.positions.len(), asset.mesh.triangle_count(),
        asset.pending_structure_entries.len());
    let path = options.shot.ok_or_else(|| "source Cell preview requires --shot <new.png>; use --audit for a read-only corpus check".to_string())?;
    if path.exists() { return Err("screenshot path already exists; choose a new output".to_string()); }
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut app = App::new();
    crate::offscreen::configure(&mut app, 1280, 800);
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(SourcePreview { asset, camera, skin_sampler, path: path.clone(), frames: 0, requested_at: None })
        .add_systems(Startup, preview_setup)
        .add_systems(Update, preview_capture)
        .run();
    if !path.exists() { return Err("source Cell offscreen screenshot was not written".to_string()); }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct PreviewOptions { install: PathBuf, cell: u32, shot: Option<PathBuf>, audit: bool, pbr_inspection: bool, diagnostic_skin_translation: bool }

impl PreviewOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut result = Self {
            install: std::env::var_os("RUSTSPORE_INSTALL").map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("D:/Games/SPORE Collection")),
            cell: JAWHEAD, shot: None, audit: false, pbr_inspection: false, diagnostic_skin_translation: false,
        };
        let mut index = 0;
        while index < args.len() {
            let value = |index: usize| args.get(index + 1)
                .ok_or_else(|| format!("{} requires a value", args[index]));
            match args[index].as_str() {
                "--install" => { result.install = PathBuf::from(value(index)?); index += 2; }
                "--cell" => {
                    let text = value(index)?.trim();
                    result.cell = u32::from_str_radix(text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text), 16)
                        .map_err(|_| "--cell expects an exact 32-bit hexadecimal source definition ID".to_string())?;
                    index += 2;
                }
                "--shot" => { result.shot = Some(PathBuf::from(value(index)?)); index += 2; }
                "--audit" => { result.audit = true; index += 1; }
                "--pbr-inspection" => { result.pbr_inspection = true; index += 1; }
                "--diagnostic-skin-translation" => { result.diagnostic_skin_translation = true; index += 1; }
                other => return Err(format!("unknown source Cell option {other:?}; use --install <dir> --cell <hex> --shot <new.png> or --audit")),
            }
        }
        if !result.audit && result.shot.is_none() { return Err("source Cell preview requires --shot <new.png> or --audit".to_string()); }
        if result.diagnostic_skin_translation && (result.cell != JAWHEAD || result.shot.is_none() || result.audit || result.pbr_inspection) {
            return Err("--diagnostic-skin-translation requires Jawhead, --shot, and original D3D9 rendering; it cannot be combined with --audit or --pbr-inspection".to_string());
        }
        Ok(result)
    }
}

#[derive(Resource)]
struct SourcePreview {
    asset: SourceCellAsset,
    camera: SourceOrbitCamera,
    skin_sampler: ImageSamplerDescriptor,
    path: PathBuf,
    frames: u32,
    requested_at: Option<u32>,
}

#[derive(Component)]
struct SourcePreviewText;

fn preview_image(texture: &TextureData, repeat: bool) -> Image {
    let mut image = Image::new(
        Extent3d { width: texture.width, height: texture.height, depth_or_array_layers: 1 },
        TextureDimension::D2, texture.rgba.clone(), TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    if repeat {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat, ..default()
        });
    }
    image
}

fn preview_setup(
    mut commands: Commands, preview: Res<SourcePreview>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>, adapter: Res<RenderAdapterInfo>,
    mut readiness: ResMut<crate::offscreen::OffscreenReadiness>,
) {
    println!("SOURCE_CELL_GPU {} {:?} {:?}", adapter.name, adapter.backend, adapter.device_type);
    let asset = &preview.asset;
    let data = &asset.mesh;
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());
    mesh.insert_indices(Indices::U32(data.indices.clone()));
    let mut skin_image = preview_image(&asset.skin, false);
    skin_image.sampler = ImageSampler::Descriptor(preview.skin_sampler.clone());
    let skin = images.add(skin_image);
    readiness.require_image(&skin);
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(skin),
        perceptual_roughness: 0.7, metallic: 0.0,
        cull_mode: None, double_sided: true,
        ..default()
    });
    let camera = &preview.camera;
    let (model_transform, view_transform, vertical_fov) = inspection_camera_fit(data, camera, 1280. / 800.)
        .expect("validated source camera and supported inspection bounds");
    println!("SOURCE_CELL_CAMERA source={} parent={} elevation_deg={} azimuth_deg={} source_distance={} source_offsets={},{} source_fov_rad={} vertical_fov_rad={} near={} far={} framing=inspection-bounds-fit",
        camera.source.key, camera.parent.key, camera.elevation_degrees, camera.azimuth_degrees,
        camera.source_distance, camera.source_horizontal_offset, camera.source_vertical_offset,
        camera.fov_radians, vertical_fov, camera.near, camera.far);
    commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material),
        model_transform));
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(camera.projection(1280. / 800.).expect("validated source projection")),
        AmbientLight { color: Color::WHITE, brightness: 300., ..default() },
        view_transform,
    ));
    commands.spawn((DirectionalLight { illuminance: 9000., ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -0.35, 0.))));
    let label = format!(
        "SPORE SOURCE CELL ART\n{}\nCell {:08X}  |  baked {:016X}\n{} vertices / {} triangles / {}x{} painted atlas\n\nSource geometry, paint and orbit orientation / perspective.\nInspection bounds-fit and lighting. Retail effects and behavior remain open.",
        asset.definition.record.embedded_name, asset.definition.cell.key.instance_id,
        asset.definition.creation_instance, data.positions.len(), data.triangle_count(),
        asset.skin.width, asset.skin.height,
    );
    commands.spawn((SourcePreviewText, Text::new(label), TextFont { font_size: 16., ..default() },
        TextColor(Color::srgb(0.88, 0.9, 0.94)),
        Node { position_type: PositionType::Absolute, left: Val::Px(20.), top: Val::Px(16.), ..default() }));
    if let Some((_, thumbnail)) = &asset.thumbnail {
        let image = images.add(preview_image(thumbnail, false));
        readiness.require_image(&image);
        commands.spawn((ImageNode::new(image),
            Node { position_type: PositionType::Absolute, right: Val::Px(30.), bottom: Val::Px(110.),
                width: Val::Px(256.), height: Val::Px(256.), ..default() }));
        commands.spawn((SourcePreviewText, Text::new("Shipped thumbnail (reference)"), TextFont { font_size: 16., ..default() },
            Node { position_type: PositionType::Absolute, right: Val::Px(25.), bottom: Val::Px(80.), ..default() }));
    }
}

fn preview_capture(
    time: Res<Time>, mut commands: Commands, mut preview: ResMut<SourcePreview>,
    target: Res<crate::offscreen::OffscreenTarget>, mut exit: MessageWriter<AppExit>,
    mut readiness: ResMut<crate::offscreen::OffscreenReadiness>,
    labels: Query<&bevy::text::TextLayoutInfo, With<SourcePreviewText>>,
) {
    preview.frames += 1;
    if let Some(error) = readiness.failure() {
        eprintln!("source Cell GPU pipeline failed: {error}");
        exit.write(AppExit::error());
        return;
    }
    if preview.requested_at.is_none() && time.elapsed_secs() > 120. {
        eprintln!("source Cell renderer did not become ready for capture");
        exit.write(AppExit::error());
        return;
    }
    let expected_labels = if preview.asset.thumbnail.is_some() { 2 } else { 1 };
    let text_ready = labels.iter().count() == expected_labels
        && labels.iter().all(|layout| readiness.require_text(layout));
    if preview.requested_at.is_none() && preview.frames >= 40 && time.elapsed_secs() >= 3. && text_ready && readiness.ready() {
        println!("SOURCE_CELL_CAPTURE render_pipelines=ready required_images=ready stable_render_frames=8");
        commands.spawn(target.screenshot()).observe(save_to_disk(preview.path.clone()));
        preview.requested_at = Some(preview.frames);
    }
    if let Some(frame) = preview.requested_at {
        if preview.frames > frame + 10 && preview.path.exists() { exit.write(AppExit::Success); }
        if preview.frames > frame + 600 {
            eprintln!("source Cell offscreen screenshot timed out");
            exit.write(AppExit::error());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_camera() -> SourceOrbitCamera {
        let identity = |id| SourceIdentity { package: PathBuf::from("source.package"),
            key: NativeResourceKey::from_scoped_instance(PROP, CAMERA_NAMESPACE | id) };
        let properties = |fields: &[(u32, u16, u32)]| {
            let mut bytes = (fields.len() as u32).to_be_bytes().to_vec();
            for &(id, kind, bits) in fields {
                bytes.extend(id.to_be_bytes()); bytes.extend(kind.to_be_bytes());
                bytes.extend(0u16.to_be_bytes()); bytes.extend(bits.to_be_bytes());
            }
            exact_camera_prop(&bytes).unwrap()
        };
        let inherited = properties(&[(0x00ED_3928, 10, 0x00ED_D3BB),
            (0x6FDA_2E1C, 13, 0), (0x00C7_C4FB, 13, 10f32.to_bits()),
            (0x00C7_C4FC, 13, 0), (0x00C7_C4FD, 13, 0),
            (0x0110_2B20, 13, 1f32.to_bits()), (0x0110_2B2F, 13, 200f32.to_bits())]);
        let local = properties(&[(0x00C7_C4FB, 13, 0x3F80_0000),
            (0x00C7_C4FC, 13, 0xC2B4_0000), (0x00C7_C4FD, 13, 0xC2B4_0000),
            (0x0110_2B20, 13, 0x3CA3_D70A), (0x0110_2B2F, 13, 0x4348_0000),
            (0x044C_6220, 13, 0x3F5F_62B7), (0x8FDA_2E23, 13, 0xBF0C_CCCD)]);
        SourceOrbitCamera::from_props(identity(0x02AD_26C9), identity(0x3F3B_2714), &local, &inherited).unwrap()
    }

    #[test]
    fn source_orbit_matches_native_basis_rows_and_bevy_camera_convention() {
        let mut camera = test_camera();
        let (right, forward, up) = camera.basis();
        assert!(right.distance(Vec3::Y) < 0.000001);
        assert!(forward.distance(-Vec3::Z) < 0.000001);
        assert!(up.distance(-Vec3::X) < 0.000001);
        camera.azimuth_degrees = 30.; camera.elevation_degrees = 20.;
        let (right, forward, up) = camera.basis();
        // Independent row oracle from the native Z and X basis-mixing routines.
        let (s, c) = (-30f32).to_radians().sin_cos();
        let (sp, cp) = 20f32.to_radians().sin_cos();
        assert!(right.distance(Vec3::new(c, s, 0.)) < 0.000001);
        assert!(forward.distance(Vec3::new(-s * cp, c * cp, sp)) < 0.000001);
        assert!(up.distance(Vec3::new(s * sp, -c * sp, cp)) < 0.000001);
        assert!((right.cross(forward).dot(up) - 1.).abs() < 0.000001);
        let view = Transform::IDENTITY.looking_to(forward, up);
        assert!((view.rotation * Vec3::NEG_Z).distance(forward) < 0.000001);
        assert!((view.rotation * Vec3::Y).distance(up) < 0.000001);
    }

    #[test]
    fn native_projection_flag_branch_distinguishes_exact_square_from_non_square() {
        use bevy::camera::CameraProjection;
        let camera = test_camera();
        // UCOMISS -> LAHF -> TEST AH,44 -> JNP: square's masked40 has
        // odd parity and skips the factor. Non-square's masked00 does not.
        let square_masked_ah = 0x40u8;
        let unequal_masked_ah = 0u8;
        assert_eq!(square_masked_ah.count_ones() % 2, 1);
        assert_eq!(unequal_masked_ah.count_ones() % 2, 0);
        let source_tan = (camera.fov_radians * 0.5).tan();
        assert!((camera.vertical_fov(1.).unwrap() - camera.fov_radians).abs() < 0.000001);
        for aspect in [1., 1280. / 800., 4. / 3., 0.5, 1. + f32::EPSILON] {
            let fov = camera.vertical_fov(aspect).unwrap();
            let native_tan_y = source_tan * if aspect == 1. { 1. } else { 0.75 };
            assert!(((fov * 0.5).tan() - native_tan_y).abs() < 0.000001);
            let projection = camera.projection(aspect).unwrap();
            // Compare Bevy's actual matrix output with the independently
            // recovered native x/y diagonals and native forward +Z clip w.
            let matrix = projection.get_clip_from_view();
            let bevy_clip = matrix * Vec4::new(0.4, -0.2, -2., 1.);
            let bevy_ndc = bevy_clip.truncate().truncate() / bevy_clip.w;
            let native_ndc = Vec2::new(0.4 / (2. * aspect * native_tan_y), -0.2 / (2. * native_tan_y));
            assert!(bevy_ndc.distance(native_ndc) < 0.000001);
            let near_clip = matrix * Vec4::new(0., 0., -camera.near, 1.);
            assert!((near_clip.z / near_clip.w - 1.).abs() < 0.000001, "Bevy reverse depth");
        }
        for invalid in [0., -1., f32::INFINITY, f32::NAN] {
            assert!(camera.vertical_fov(invalid).is_err());
        }
    }

    #[test]
    fn inspection_fit_contains_every_corner_with_source_planes_and_orientation() {
        let camera = test_camera();
        let mut data = MeshData::default();
        for x in [-2., 4.] { for y in [-3., 5.] { for z in [-0.4, 0.7] {
            data.positions.push(spore_core::Vec3::new(x, y, z));
        } } }
        let (model, view, fov) = inspection_camera_fit(&data, &camera, 1.6).unwrap();
        let (right, forward, up) = camera.basis();
        let ty = (fov * 0.5).tan();
        for p in &data.positions {
            let point = model.transform_point(Vec3::new(p.x, p.y, p.z)) - view.translation;
            let depth = point.dot(forward);
            assert!(depth > camera.near && depth < camera.far);
            let ndc = Vec2::new(point.dot(right) / (depth * ty * 1.6), point.dot(up) / (depth * ty));
            assert!((-0.94..=0.44).contains(&ndc.x), "{ndc:?}");
            assert!((-0.90..=0.34).contains(&ndc.y), "{ndc:?}");
        }
        let mut impossible = camera.clone(); impossible.far = 0.05;
        assert!(inspection_camera_fit(&data, &impossible, 1.6).is_err());
    }

    #[test]
    fn source_camera_does_not_accept_partial_or_fabricated_property_framing() {
        let mut blob = 2u32.to_be_bytes().to_vec();
        blob.extend(0x0110_2B20u32.to_be_bytes()); blob.extend(13u16.to_be_bytes());
        blob.extend(0u16.to_be_bytes()); blob.extend(0.02f32.to_bits().to_be_bytes());
        assert!(exact_camera_prop(&blob).is_err(), "wrong declared item count");
        blob[..4].copy_from_slice(&1u32.to_be_bytes());
        let empty = exact_camera_prop(&0u32.to_be_bytes()).unwrap();
        let partial = exact_camera_prop(&blob).unwrap();
        let camera = test_camera();
        assert!(SourceOrbitCamera::from_props(camera.source, camera.parent, &partial, &empty).is_err());
    }

    #[test]
    fn source_preview_cli_requires_explicit_output_and_valid_source_ids() {
        let args = |items: &[&str]| items.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        assert!(PreviewOptions::parse(&[]).is_err());
        for input in [vec!["--cell"], vec!["--cell", "100000000", "--audit"],
            vec!["--cell", "invalid", "--audit"], vec!["--unknown"], vec!["--shot"]] {
            assert!(PreviewOptions::parse(&args(&input)).is_err());
        }
        let parsed = PreviewOptions::parse(&args(&["--cell", "0x7d00e384", "--shot", "target/source.png"])).unwrap();
        assert_eq!(parsed.cell, JAWHEAD);
        assert_eq!(parsed.shot, Some(PathBuf::from("target/source.png")));
        assert!(!parsed.pbr_inspection);
        assert!(PreviewOptions::parse(&args(&["--pbr-inspection", "--shot", "target/pbr.png"])).unwrap().pbr_inspection);
        assert!(PreviewOptions::parse(&args(&["--diagnostic-skin-translation", "--shot", "target/diagnostic.png"])).unwrap().diagnostic_skin_translation);
        assert!(PreviewOptions::parse(&args(&["--diagnostic-skin-translation", "--audit"])).is_err());
        assert!(PreviewOptions::parse(&args(&["--diagnostic-skin-translation", "--pbr-inspection", "--shot", "target/diagnostic.png"])).is_err());
        assert!(PreviewOptions::parse(&args(&["--diagnostic-skin-translation", "--cell", "1", "--shot", "target/diagnostic.png"])).is_err());
        assert!(PreviewOptions::parse(&args(&["--audit"])).unwrap().audit);
    }

    #[test]
    fn cell_record_requires_exact_fixed_layout_and_valid_developer_label() {
        let mut bytes = vec![0; 796];
        bytes[..4].copy_from_slice(&0x285A_339Bu32.to_le_bytes());
        bytes[4..6].copy_from_slice(&('J' as u16).to_le_bytes());
        let record = CellRecord::parse(&bytes).unwrap();
        assert_eq!(record.structure_id, 0x285A_339B);
        assert_eq!(record.embedded_name, "J");
        assert_eq!(record.raw, bytes);
        assert!(CellRecord::parse(&bytes[..795]).is_err());
        bytes.push(0);
        assert!(CellRecord::parse(&bytes).is_err());
        bytes.truncate(796);
        bytes[4..6].copy_from_slice(&0xD800u16.to_le_bytes());
        assert!(CellRecord::parse(&bytes).is_err());
        // The native text member is 80 UTF-16 units. A full label must never
        // consume its trailing localization/numeric words as extra text.
        for unit in bytes[4..164].chunks_exact_mut(2) {
            unit.copy_from_slice(&('A' as u16).to_le_bytes());
        }
        bytes[164..168].copy_from_slice(&0x0042_0042u32.to_le_bytes());
        assert_eq!(CellRecord::parse(&bytes).unwrap().embedded_name, "A".repeat(80));
    }

    #[test]
    fn measured_structure_entries_preserve_effects_and_reject_ambiguous_creations() {
        let mut bytes = vec![0u8; 108];
        bytes[20..24].copy_from_slice(&28u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&2u32.to_le_bytes());
        bytes[32..36].copy_from_slice(&3u32.to_le_bytes());
        bytes[44..48].copy_from_slice(&0x780A_9EAFu32.to_le_bytes());
        bytes[72..76].copy_from_slice(&1u32.to_le_bytes());
        bytes[84..88].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        let structure = CellStructure::parse(&bytes).unwrap();
        assert_eq!(structure.baked_creation().unwrap(), 0x4061_6200_780A_9EAF);
        assert_eq!(structure.entries[1].kind, 1);
        assert_eq!(structure.entries[1].reference_id, 0x1234_5678);
        bytes[72..76].copy_from_slice(&3u32.to_le_bytes());
        assert!(CellStructure::parse(&bytes).unwrap().baked_creation().is_err());
    }

    #[test]
    fn malformed_structures_and_images_fail_without_substitute_art() {
        for size in 0..28 { assert!(CellStructure::parse(&vec![0; size]).is_err()); }
        let mut bytes = vec![0; 68];
        bytes[20..24].copy_from_slice(&28u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&1u32.to_le_bytes());
        assert!(CellStructure::parse(&bytes).unwrap().baked_creation().is_err());
        for changed in [0, 24] {
            bytes[20..24].copy_from_slice(&(changed as u32).to_le_bytes());
            assert!(CellStructure::parse(&bytes).is_err());
        }
        bytes[20..24].copy_from_slice(&28u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(CellStructure::parse(&bytes).is_err());
        assert!(decode_thumbnail(b"not a PNG").is_err());
    }

    /// Explicit opt-in corpus test reads the user's install and no bundled art.
    #[test]
    #[ignore = "requires the local SPORE installation; run with --ignored --nocapture"]
    fn installed_baked_cells_load_their_exact_models_and_own_painted_skins() {
        let install = std::env::var_os("RUSTSPORE_INSTALL")
            .map(PathBuf::from).unwrap_or_else(|| PathBuf::from("D:/Games/SPORE Collection"));
        let source = SourceCellLibrary::open_install(&install).unwrap();
        let camera = source.thumbnail_camera().unwrap();
        assert_eq!(camera.source.key.scoped_instance(), 0x4041_0100_02AD_26C9);
        assert_eq!(camera.parent.key.scoped_instance(), 0x4041_0100_3F3B_2714);
        assert!(camera.source.package.ends_with("PatchData.package"));
        assert_eq!((camera.azimuth_degrees, camera.elevation_degrees), (-90., -90.));
        assert_eq!(camera.fov_radians.to_bits(), 0x3F5F_62B7);
        assert_eq!(camera.near.to_bits(), 0x3CA3_D70A);
        assert_eq!(camera.far, 200.);
        let census = source.census();
        println!("source Cell census: {census:#?}");
        assert_eq!(census.definitions, 223);
        assert_eq!(census.structures_resolved, 222);
        assert_eq!(census.baked_definitions, 90);
        assert_eq!(census.available_baked_definitions, 65);
        assert_eq!(census.unique_available_creations, 28);
        assert_eq!(census.missing_baked_props, 25);
        assert_eq!(census.baked_loaded, 65);
        assert_eq!(census.thumbnails_loaded, 65);
        // Twenty-five baked references have no installed model prop. Another
        // unused definition names structure zero. Every one remains unresolved.
        assert_eq!(census.failures.len(), 26, "{:?}", census.failures);
        assert!(census.failures.iter().all(|failure| failure.contains("source resource missing 00B1B104:")
            || failure.contains("source resource missing 4B9EF6DC:0000000000000000")));
        let jawhead = source.load(JAWHEAD).unwrap();
        assert_eq!(jawhead.definition.record.structure_id, 0x285A_339B);
        assert_eq!(jawhead.model.key.scoped_instance(), 0x4061_7100_780A_9EAF);
        assert_eq!(jawhead.skin_source.key.scoped_instance(), 0x4061_2900_780A_9EAF);
        assert_eq!((jawhead.skin.width, jawhead.skin.height), (512, 512));
        assert_eq!(jawhead.mesh.positions.len(), 11894);
        assert!(jawhead.model_prop.package.ends_with("PatchData.package"));
        assert!(source.load(0xFFFF_FFFF).is_err());
    }
}
