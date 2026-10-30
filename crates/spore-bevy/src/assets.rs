//! Turning package resources into renderable data.
//!
//! This is the layer the roadmap calls the asset registry: it knows which resource
//! types are meshes and which are textures, and it decodes one on demand. It does
//! not depend on Bevy, so it can be exercised from the command line.

use std::collections::HashMap;
use std::sync::Mutex;
use std::path::{Path, PathBuf};

use dbpf::native_index::{NativePackageIndex, NativeResourceEntry, NativeResourceKey};
use spore_core::mesh::{MeshData, MeshError};

/// Why a resource could not be turned into renderable data.
#[derive(Debug, Clone)]
pub enum AssetError {
    /// The resource is valid but holds no geometry, which happens for placeholder
    /// models. This is not a failure.
    Empty,
    /// A reference in the data points at something no attached package holds. Kept
    /// separate from `Other` so a caller can count dangling references rather than
    /// reporting them as conversion failures, which are different problems with
    /// different fixes.
    Unresolved(String),
    Other(String),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::Empty => write!(f, "resource contains no geometry"),
            AssetError::Unresolved(m) => f.write_str(m),
            AssetError::Other(m) => f.write_str(m),
        }
    }
}

impl From<MeshError> for AssetError {
    fn from(e: MeshError) -> Self {
        match e {
            MeshError::Empty => AssetError::Empty,
            other => AssetError::Other(other.to_string()),
        }
    }
}

/// Type ids this layer understands.
pub const GMSH: u32 = 0x01C1_35DA;
pub const GMDL: u32 = 0x00E6_BCE5;
pub const RW4: u32 = 0x2F4E_681B;
pub const RASTER: u32 = 0x2F4E_681C;
pub const PNG: u32 = 0x2F7D_0004;
pub const PROP: u32 = 0x00B1_B104;
pub const CREATURE_DATA: u32 = 0x0F43_029A;

fn scoped_entry(index: &NativePackageIndex, type_id: u32, instance: u64) -> Option<&NativeResourceEntry> {
    index.index().find_unique(NativeResourceKey::from_scoped_instance(type_id, instance)).ok().flatten()
}

/// Inspection references with no type can only resolve an unambiguous native key.
/// Package order remains an explicit caller policy, separate from retail overlays.
fn unspecified_entry(index: &NativePackageIndex, instance: u64) -> Option<&NativeResourceEntry> {
    let mut matches = index.entries().iter().filter(|entry| entry.key.scoped_instance() == instance);
    let first = matches.next()?;
    if matches.next().is_some() { None } else { Some(first) }
}

/// A creature assembled from its rig.
pub struct CreatureAssembly {
    /// The creature resource itself.
    pub source: (u32, u64),
    /// Rig entries in the file.
    pub parts_total: usize,
    /// Parts whose model was found and converted.
    pub parts_drawn: usize,
    /// Parts whose prop or model could not be resolved in the attached packages.
    pub parts_missing: usize,
    /// Parts that resolved but failed to convert, with the reason.
    pub parts_failed: usize,
    /// The first few failures, for diagnosis.
    pub errors: Vec<String>,
    /// Vertices in the assembled mesh.
    pub vertices: usize,
    /// Triangles in the assembled mesh.
    pub triangles: usize,
}

/// A prop resolved to the resources it names.
///
/// This is the material path for gmdl models: unlike RW4, a gmdl container has no
/// texture, so the only way to shade one is to follow the prop that describes it.
pub struct PropModel {
    /// The prop itself.
    pub prop: NativeResourceKey,
    /// The highest-detail model it names, as `(type id, instance)`. Not a
    /// `NativeResourceKey`: the existing PROP accessor spells scope and low ID
    /// together as a u64. It carries all three native key words.
    pub model: (u32, u64),
    /// Every model slot filled, highest detail first.
    pub lods: Vec<(u32, u64)>,
    /// Colour maps named by the same prop or inherited from its parent.
    pub textures: Vec<(u32, u64)>,
}

/// A decoded texture: RGBA8, top-left origin.
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// All declared, validated standalone RASTER levels in largest-first order.
pub struct TextureMipChain {
    pub levels: Vec<TextureData>,
}

fn decode_standalone_raster_mips(data: &[u8]) -> Result<TextureMipChain, String> {
    if data.len() > 128 * 1024 * 1024 { return Err("source raster payload exceeds limit".into()); }
    let raster = spore_core::raster::parse(data).map_err(|e| e.to_string())?;
    if raster.width == 0 || raster.height == 0 || raster.width > 4096 || raster.height > 4096 {
        return Err("source raster mip dimensions unsupported".into());
    }
    let source_levels = raster.mip_levels().map_err(|e| e.to_string())?;
    let mut levels = Vec::with_capacity(source_levels.len());
    let mut total = 0usize;
    for level in source_levels {
        let size = (level.width as usize).checked_mul(level.height as usize)
            .and_then(|n| n.checked_mul(4)).ok_or("source raster mip extent overflow")?;
        total = total.checked_add(size).ok_or("source raster mip total overflow")?;
        if total > 128 * 1024 * 1024 { return Err("source raster mip chain exceeds limit".into()); }
        let rgba = spore_core::raster::decode_pixels(raster.format, level.width, level.height, level.data)
            .map_err(|e| e.to_string())?;
        if rgba.len() != size { return Err("source raster mip decoded extent mismatch".into()); }
        levels.push(TextureData { width: level.width, height: level.height, rgba });
    }
    Ok(TextureMipChain { levels })
}

/// An exact authored LOD reference at the source creation's own PROP instance.
#[derive(Debug, Clone, Copy)]
pub struct AuthoredCreatureModel {
    pub creature: NativeResourceKey,
    pub prop: NativeResourceKey,
    pub model: NativeResourceKey,
    pub lod: usize,
}

/// Saved source art for the inspection viewer, with its real sampler-zero map.
/// Native animation, shaders and the full retail draw state are separate gates.
pub struct AuthoredCreatureAsset {
    pub reference: AuthoredCreatureModel,
    pub mesh: MeshData,
    pub texture_key: NativeResourceKey,
    pub texture_binding: spore_core::shader_binding::TextureBinding,
    pub texture: TextureData,
}

fn authored_lod(data: &[u8], lod: usize) -> Result<(u64, u32), AssetError> {
    let slots = [spore_core::prop::idents::MODEL_LOD0, spore_core::prop::idents::MODEL_LOD1,
                 spore_core::prop::idents::MODEL_LOD2, spore_core::prop::idents::MODEL_LOD3];
    if lod >= slots.len() {
        return Err(AssetError::Other(format!("unsupported authored LOD {lod}")));
    }
    let declared = data.get(..4).ok_or_else(|| AssetError::Other("model PROP has no declared count".into()))?;
    let declared = u32::from_be_bytes(declared.try_into().unwrap()) as usize;
    let tree = spore_core::prop::parse(data).map_err(|e| AssetError::Other(e.to_string()))?;
    if !tree.exact || tree.consumed != data.len() || tree.items.len() != declared
        || tree.items.iter().any(|item| item.anonymous)
    {
        return Err(AssetError::Other("model PROP requires exact declared named framing".into()));
    }
    let mut selected = None;
    // Native 74A7F0 stops at the first absent F9EFBB+i property. Later slots
    // beyond a hole are not silently promoted into the requested LOD.
    for (ordinal, slot) in slots.iter().enumerate().take(lod + 1) {
        let mut items = tree.items.iter().filter(|item| item.id == *slot);
        let item = items.next().ok_or_else(|| AssetError::Unresolved(format!("model PROP has no authored LOD{ordinal}")))?;
        if items.next().is_some() || item.prop_type != spore_core::prop::types::KEY
            || item.specifier != 0 || item.values.len() != 1
        {
            return Err(AssetError::Other(format!("model PROP LOD{ordinal} is not one explicit scalar key")));
        }
        selected = item.values[0].key();
    }
    let selected = selected.ok_or_else(|| AssetError::Other("authored model key is invalid".into()))?;
    if selected.0 >> 32 == u64::from(u32::MAX) {
        return Err(AssetError::Unresolved("authored model inherits an unestablished native namespace".into()));
    }
    Ok(selected)
}

fn shared_sampler_zero(
    tail: &spore_core::gmdl_skin::GmdlSkin,
    meshes: usize,
) -> Result<spore_core::shader_binding::TextureBinding, AssetError> {
    if tail.materials.len() != meshes || meshes == 0 {
        return Err(AssetError::Other("saved GMDL material slots do not match its meshes".into()));
    }
    let mut shared: Option<spore_core::shader_binding::TextureBinding> = None;
    for material in &tail.materials {
        let field = material.shader_data.iter().rev().find(|field| field.id == 0x20D)
            .ok_or_else(|| AssetError::Unresolved("GMDL material has no source texture bindings".into()))?;
        let bindings = spore_core::shader_binding::texture_bindings(&field.payload)
            .map_err(|e| AssetError::Other(e.to_string()))?;
        let binding = bindings.iter().rev().find(|binding| binding.sampler == 0).copied()
            .ok_or_else(|| AssetError::Unresolved("GMDL material has no source sampler zero".into()))?;
        if binding.availability() != Some(1)
            || !matches!(binding.address_mode(), Some(1..=3))
            || !matches!(binding.min_mag_filter(), Some(1..=2))
            || !matches!(binding.mip_filter(), Some(0..=2))
        {
            return Err(AssetError::Other("source sampler zero has disabled or unresolved state".into()));
        }
        if shared.is_some_and(|previous| previous.raster_instance != binding.raster_instance
            || previous.address_mode() != binding.address_mode()
            || previous.min_mag_filter() != binding.min_mag_filter()
            || previous.mip_filter() != binding.mip_filter())
        {
            return Err(AssetError::Other("inspection viewer needs separate draws for different source sampler-zero maps".into()));
        }
        shared = Some(binding);
    }
    shared.ok_or_else(|| AssetError::Unresolved("GMDL has no source sampler zero".into()))
}

/// Other packages, searched when the open one lacks a resource.
///
/// A prop's parent and its texture routinely live somewhere else: every one of the
/// 1,009 textureless model props in `Spore_Content` names a parent in `PatchData` or
/// `Spore_Graphics`. Only the indexes are held open, which is about a megabyte each,
/// and a payload is read by seeking when it is actually wanted. Holding the packages
/// themselves would cost 5.75 GB.
pub struct PackageSet {
    paths: Vec<PathBuf>,
    /// Opened indexes, keyed by path. A `Mutex` rather than a `RefCell` because a
    /// Bevy resource has to be `Sync`.
    indexes: Mutex<HashMap<PathBuf, Option<NativePackageIndex>>>,
}

impl PackageSet {
    /// Every `.package` under a directory, searched in path order.
    pub fn discover(dir: impl AsRef<Path>) -> Self {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("package"))
                    .unwrap_or(false)
                {
                    out.push(path);
                }
            }
        }
        let mut paths = Vec::new();
        walk(dir.as_ref(), &mut paths);
        paths.sort();
        Self {
            paths,
            indexes: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            indexes: Mutex::new(HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// The packages this set searches, in order.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Read a resource from whichever package holds it.
    pub fn find(&self, type_id: u32, instance: u64) -> Option<Vec<u8>> {
        for path in &self.paths {
            let mut indexes = self.indexes.lock().ok()?;
            let slot = indexes
                .entry(path.clone())
                .or_insert_with(|| NativePackageIndex::open(path).ok());
            let Some(index) = slot.as_ref() else {
                continue;
            };
            let Some(entry) = scoped_entry(index, type_id, instance) else {
                continue;
            };
            let entry = entry.clone();
            let out = index.read(&entry).ok();
            // Release the guard before returning; the read above did its own I/O.
            drop(indexes);
            return out;
        }
        None
    }

    /// Resolve `(type, instance)` to a concrete type and instance.
    ///
    /// A type of 0 is treated as unspecified and matched on the instance alone.
    pub fn lookup(&self, type_id: u32, instance: u64) -> Option<(u32, u64)> {
        for path in &self.paths {
            let mut indexes = self.indexes.lock().ok()?;
            let slot = indexes
                .entry(path.clone())
                .or_insert_with(|| NativePackageIndex::open(path).ok());
            let Some(index) = slot.as_ref() else {
                continue;
            };
            let hit = if type_id == 0 { unspecified_entry(index, instance) }
                else { scoped_entry(index, type_id, instance) };
            if let Some(entry) = hit {
                return Some((entry.key.type_id, entry.key.scoped_instance()));
            }
        }
        None
    }

    /// Every resource of a type whose instance shares this low half.
    ///
    /// A creature's skin is not named by any prop. It is keyed by the creature's own
    /// instance low half with a different high half, so the only way to find it is to
    /// look for that shared half.
    pub fn with_low_half(&self, type_id: u32, low: u32) -> Vec<(u32, u64)> {
        let mut out = Vec::new();
        for path in &self.paths {
            let mut indexes = match self.indexes.lock() {
                Ok(g) => g,
                Err(_) => return out,
            };
            let slot = indexes
                .entry(path.clone())
                .or_insert_with(|| NativePackageIndex::open(path).ok());
            if let Some(index) = slot.as_ref() {
                for e in index.entries() {
                    if e.key.type_id == type_id && e.key.instance_id == low {
                        out.push((e.key.type_id, e.key.scoped_instance()));
                    }
                }
            }
        }
        out
    }

    /// Which package holds a resource, for diagnostics.
    pub fn locate(&self, type_id: u32, instance: u64) -> Option<PathBuf> {
        for path in &self.paths {
            let mut indexes = self.indexes.lock().ok()?;
            let slot = indexes
                .entry(path.clone())
                .or_insert_with(|| NativePackageIndex::open(path).ok());
            if let Some(index) = slot.as_ref() {
                if scoped_entry(index, type_id, instance).is_some() {
                    return Some(path.clone());
                }
            }
        }
        None
    }

    fn key_for(&self, type_id: u32, instance: u64) -> Option<NativeResourceKey> {
        for path in &self.paths {
            let mut indexes = self.indexes.lock().ok()?;
            let slot = indexes.entry(path.clone()).or_insert_with(|| NativePackageIndex::open(path).ok());
            if let Some(entry) = slot.as_ref().and_then(|index| scoped_entry(index, type_id, instance)) {
                return Some(entry.key);
            }
        }
        None
    }
}

/// A package opened for rendering.
pub struct AssetSource {
    file: NativePackageIndex,
    meshes: Vec<NativeResourceKey>,
    textures: Vec<NativeResourceKey>,
    /// Packages to consult when this one does not have what a prop names.
    set: Option<PackageSet>,
}

impl AssetSource {
    /// Read the exact saved sampler texture as a complete standalone RASTER chain.
    /// RW4 texture chains are a separate format and are deliberately rejected here.
    pub fn authored_creature_raster_mips(&self, key: NativeResourceKey) -> Result<TextureMipChain, String> {
        if key.type_id != RASTER { return Err("source Creature mip chain requires standalone RASTER; RW4 unsupported".into()); }
        let data = self.read(key.type_id, key.scoped_instance()).ok_or("source Creature raster unreadable")?;
        decode_standalone_raster_mips(&data)
    }
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = NativePackageIndex::open(path).map_err(|e| e.to_string())?;
        let mut meshes = Vec::new();
        let mut textures = Vec::new();
        for entry in file.entries() {
            match entry.key.type_id {
                GMSH | GMDL | RW4 => meshes.push(entry.key),
                RASTER | PNG => textures.push(entry.key),
                _ => {}
            }
        }
        // Group first, then instance: a stable order that groups a model's parts.
        meshes.sort_by_key(|k| (k.type_id, k.group_id, k.instance_id));
        textures.sort_by_key(|k| (k.type_id, k.group_id, k.instance_id));
        Ok(Self {
            file,
            meshes,
            textures,
            set: None,
        })
    }

    /// Attach a set of other packages to search for missing resources.
    pub fn with_set(mut self, set: PackageSet) -> Self {
        self.set = Some(set);
        self
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// All canonical keys in archive order, including non-renderable resources.
    pub fn resource_keys(&self) -> impl Iterator<Item = NativeResourceKey> + '_ {
        self.file.entries().iter().map(|entry| entry.key)
    }

    pub fn mesh_key(&self, i: usize) -> Option<NativeResourceKey> {
        self.meshes.get(i).copied()
    }

    pub fn texture_key(&self, i: usize) -> Option<NativeResourceKey> {
        self.textures.get(i).copied()
    }

    fn data(&self, key: &NativeResourceKey) -> Result<Vec<u8>, String> {
        let entry = self
            .file
            .index().find_unique(*key).map_err(|error| error.to_string())?
            .ok_or_else(|| format!("resource {key} not found"))?;
        self.file.read(entry).map_err(|e| e.to_string())
    }

    /// Decode one mesh by index into renderable geometry.
    pub fn mesh(&self, i: usize) -> Result<MeshData, AssetError> {
        let key = self
            .mesh_key(i)
            .ok_or_else(|| AssetError::Other(format!("mesh index {i} out of range")))?;
        let data = self.data(&key).map_err(AssetError::Other)?;
        match key.type_id {
            GMSH => {
                let parsed =
                    spore_core::gmsh::parse(&data).map_err(|e| AssetError::Other(e.to_string()))?;
                Ok(spore_core::mesh::from_gmsh(&parsed)?)
            }
            GMDL => {
                let parsed =
                    spore_core::gmdl::parse(&data).map_err(|e| AssetError::Other(e.to_string()))?;
                Ok(spore_core::mesh::from_gmdl(&data, &parsed)?)
            }
            RW4 => {
                let parsed =
                    spore_core::rw4::parse(&data).map_err(|e| AssetError::Other(e.to_string()))?;
                Ok(spore_core::mesh::from_rw4(&data, &parsed)?)
            }
            other => Err(AssetError::Other(format!("type 0x{other:08X} is not a mesh"))),
        }
    }

    /// Decode one texture by index to RGBA8.
    pub fn texture(&self, i: usize) -> Result<TextureData, String> {
        let key = self
            .texture_key(i)
            .ok_or_else(|| format!("texture index {i} out of range"))?;
        let data = self.data(&key)?;
        match key.type_id {
            RASTER => {
                let r = spore_core::raster::parse(&data).map_err(|e| e.to_string())?;
                let rgba = r.decode_rgba().map_err(|e| e.to_string())?;
                Ok(TextureData {
                    width: r.width,
                    height: r.height,
                    rgba,
                })
            }
            PNG => {
                let img = image::load_from_memory_with_format(&data, image::ImageFormat::Png)
                    .map_err(|e| e.to_string())?;
                let rgba = img.to_rgba8();
                Ok(TextureData {
                    width: rgba.width(),
                    height: rgba.height(),
                    rgba: rgba.into_raw(),
                })
            }
            other => Err(format!("type 0x{other:08X} is not a texture")),
        }
    }

    /// The texture that belongs to a mesh, when the mesh carries one.
    ///
    /// An RW4 container usually stores its own texture, which is the only material
    /// information available without decoding the prop trees. gmsh and gmdl do not,
    /// so they return `None` and the viewer falls back to an indexed texture.
    pub fn mesh_texture(&self, i: usize) -> Option<TextureData> {
        let key = self.mesh_key(i)?;
        if key.type_id != RW4 {
            return None;
        }
        let data = self.data(&key).ok()?;
        let rw = spore_core::rw4::parse(&data).ok()?;
        let tex = rw.texture(&data)?;
        let rgba = spore_core::raster::decode_pixels(
            tex.format,
            tex.width,
            tex.height,
            tex.payload,
        )
        .ok()?;
        if rgba.is_empty() {
            return None;
        }
        Some(TextureData {
            width: tex.width,
            height: tex.height,
            rgba,
        })
    }

    /// One line describing a mesh's own texture, for the heads-up display.
    pub fn mesh_texture_label(&self, i: usize) -> Option<String> {
        let key = self.mesh_key(i)?;
        if key.type_id != RW4 {
            return None;
        }
        let data = self.data(&key).ok()?;
        let rw = spore_core::rw4::parse(&data).ok()?;
        let tex = rw.texture(&data)?;
        Some(format!(
            "embedded {} {}x{} ({} mips)",
            spore_core::raster::format_name(tex.format),
            tex.width,
            tex.height,
            tex.mipmap_levels
        ))
    }

    /// Exact native lookup; the u64 argument is the scope/low inspection spelling.
    fn find_by_instance(&self, type_id: u32, instance: u64) -> Option<&NativeResourceEntry> {
        scoped_entry(&self.file, type_id, instance)
    }

    fn key_for(&self, type_id: u32, instance: u64) -> Option<NativeResourceKey> {
        self.find_by_instance(type_id, instance).map(|e| e.key)
            .or_else(|| self.set.as_ref()?.key_for(type_id, instance))
    }

    /// Read a resource by type and instance, from this package or any attached one.
    fn read(&self, type_id: u32, instance: u64) -> Option<Vec<u8>> {
        let (type_id, instance) = self.lookup(type_id, instance)?;
        if let Some(entry) = self.find_by_instance(type_id, instance) {
            return self.data(&entry.key).ok();
        }
        self.set.as_ref()?.find(type_id, instance)
    }

    /// Whether a resource exists here or in an attached package.
    fn has(&self, type_id: u32, instance: u64) -> bool {
        self.lookup(type_id, instance).is_some()
    }

    /// Resolve a reference to a concrete type and instance, local or foreign.
    fn lookup(&self, type_id: u32, instance: u64) -> Option<(u32, u64)> {
        if let Some(e) = self.find_by_instance(type_id, instance) {
            return Some((e.key.type_id, e.key.scoped_instance()));
        }
        if type_id == 0 {
            if let Some(e) = unspecified_entry(&self.file, instance) {
                return Some((e.key.type_id, e.key.scoped_instance()));
            }
        }
        self.set.as_ref()?.lookup(type_id, instance)
    }

    /// Parse the prop with this instance.
    fn prop_tree(&self, prop_instance: u64) -> Option<(NativeResourceKey, spore_core::prop::PropTree)> {
        let key = self.key_for(PROP, prop_instance)?;
        let data = self.read(PROP, prop_instance)?;
        let tree = spore_core::prop::parse(&data).ok()?;
        Some((key, tree))
    }

    /// Texture keys for a prop, following the parent chain when the prop has none.
    ///
    /// Only 153 of the 1,162 model props in `Spore_Content` carry a texture
    /// directly, so the rest must inherit one. This walks up to four levels, which
    /// is enough for every chain measured.
    pub fn prop_texture_keys(&self, prop_instance: u64) -> Vec<(u32, u64)> {
        let mut out = Vec::new();
        let mut cur = Some(prop_instance);
        for _ in 0..4 {
            let Some(instance) = cur else { break };
            let Some((_, tree)) = self.prop_tree(instance) else {
                break;
            };
            let keys: Vec<(u32, u64)> = tree
                .textures()
                .into_iter()
                .filter_map(|(i, t)| self.lookup(t, i))
                .collect();
            if !keys.is_empty() {
                out = keys;
                break;
            }
            cur = tree.parent().map(|(i, _)| i);
        }
        out
    }

    /// Resolve a prop to the model and textures it names.
    pub fn resolve_prop(&self, prop_instance: u64) -> Result<PropModel, AssetError> {
        // The prop may be in another package: a creature's parts all are, since the
        // part props live in `PatchData` and the creature in `Spore_Content`.
        let (prop_type, prop_instance) = self.lookup(PROP, prop_instance).ok_or_else(|| {
            AssetError::Unresolved(format!("no prop with instance {prop_instance:016X}"))
        })?;
        let prop = NativeResourceKey::from_scoped_instance(prop_type, prop_instance);
        let data = self.read(prop_type, prop_instance).ok_or_else(|| {
            AssetError::Other(format!("prop {prop_instance:016X} unreadable"))
        })?;
        let tree = spore_core::prop::parse(&data)
            .map_err(|e| AssetError::Other(format!("prop parse: {e}")))?;
        let Some((model_instance, model_type)) = tree.model() else {
            return Err(AssetError::Other(
                "prop names no model (no LOD0 to LOD3 slot)".to_string(),
            ));
        };
        let model = self.lookup(model_type, model_instance).ok_or_else(|| {
            AssetError::Unresolved(format!(
                "prop names model {model_type:08X}:{model_instance:016X}, which is in no attached package"
            ))
        })?;
        let lods = tree
            .models()
            .into_iter()
            .filter_map(|(i, t)| self.lookup(t, i))
            .collect();
        let textures = self.prop_texture_keys(prop_instance);
        Ok(PropModel {
            prop,
            model,
            lods,
            textures,
        })
    }

    /// Decode the model a prop names.
    pub fn prop_mesh(&self, prop_instance: u64) -> Result<(PropModel, MeshData), AssetError> {
        let resolved = self.resolve_prop(prop_instance)?;
        let (model_type, model_instance) = resolved.model;
        let data = self.read(model_type, model_instance).ok_or_else(|| {
            AssetError::Unresolved("model payload unreadable".to_string())
        })?;
        let mesh = match model_type {
            GMDL => {
                let parsed = spore_core::gmdl::parse(&data)
                    .map_err(|e| AssetError::Other(e.to_string()))?;
                spore_core::mesh::from_gmdl(&data, &parsed)?
            }
            GMSH => {
                let parsed = spore_core::gmsh::parse(&data)
                    .map_err(|e| AssetError::Other(e.to_string()))?;
                spore_core::mesh::from_gmsh(&parsed)?
            }
            RW4 => {
                let parsed = spore_core::rw4::parse(&data)
                    .map_err(|e| AssetError::Other(e.to_string()))?;
                spore_core::mesh::from_rw4(&data, &parsed)?
            }
            other => {
                return Err(AssetError::Other(format!(
                    "prop names type 0x{other:08X}, which is not a mesh"
                )))
            }
        };
        Ok((resolved, mesh))
    }

    /// A creature's own skin, when the install ships one.
    ///
    /// No prop names it. The skin is a RASTER whose instance shares the creature's low
    /// half and differs in the high half: creature `0x40666200D1B4BB56` has
    /// `0x40662900D1B4BB56`, `0x40662800D1B4BB56` and four more. Measured over the 366
    /// creatures in `Spore_Content` that have any, the largest is always the skin atlas:
    /// 512x512 DXT5, holding eyes, mouths and body patches laid out for the UVs, while
    /// the others are 256x256 detail or specular maps. So the rule is "largest wins",
    /// not a guessed ident.
    pub fn creature_skin(&self, instance: u64) -> Option<(TextureData, String)> {
        let low = (instance & 0xFFFF_FFFF) as u32;
        let mut candidates: Vec<(u32, u64)> = self
            .file
            .entries()
            .iter()
            .filter(|e| {
                e.key.type_id == RASTER && e.key.instance_id == low
            })
            .map(|e| (e.key.type_id, e.key.scoped_instance()))
            .collect();
        if let Some(set) = self.set.as_ref() {
            for c in set.with_low_half(RASTER, low) {
                if !candidates.contains(&c) {
                    candidates.push(c);
                }
            }
        }

        let mut best: Option<(u32, u64, u32, u32, Vec<u8>)> = None;
        for (type_id, inst) in candidates {
            let Some(data) = self.read(type_id, inst) else {
                continue;
            };
            let Ok(raster) = spore_core::raster::parse(&data) else {
                continue;
            };
            let Ok(rgba) = raster.decode_rgba() else {
                continue;
            };
            let area = raster.width * raster.height;
            if best.as_ref().map(|b| area > b.2 * b.3).unwrap_or(true) {
                best = Some((type_id, inst, raster.width, raster.height, rgba));
            }
        }

        best.map(|(_, inst, width, height, rgba)| {
            (
                TextureData {
                    width,
                    height,
                    rgba,
                },
                format!("skin {inst:016X} {width}x{height}"),
            )
        })
    }

    /// A creature's paint palette, three RGB colours in 0..1.
    ///
    /// This is what a creature with no shipped skin still has: its colours. It is not
    /// a generated atlas, so it cannot give a creature markings, but it is the
    /// creature's own colouring rather than a texture borrowed from the package list.
    pub fn creature_palette(&self, instance: u64) -> Option<[[f32; 3]; 3]> {
        let data = self.read(CREATURE_DATA, instance)?;
        let parsed = spore_core::creature_data::parse(&data).ok()?;
        parsed.palette()
    }

    /// The raw creature resource, for callers that want the rig itself.
    pub fn creature_raw(&self, instance: u64) -> Option<Vec<u8>> {
        self.read(CREATURE_DATA, instance)
    }

    /// Resolve the requested authored LOD exactly. Missing PROP/slot/model is
    /// an explicit miss, with no namespace rewrite, lower LOD or part assembly.
    pub fn authored_creature_model(&self, instance: u64, lod: usize) -> Result<AuthoredCreatureModel, AssetError> {
        let creature = self.key_for(CREATURE_DATA, instance)
            .ok_or_else(|| AssetError::Unresolved(format!("no source creation {instance:016X}")))?;
        let prop = self.key_for(PROP, instance)
            .ok_or_else(|| AssetError::Unresolved(format!("source creation {instance:016X} has no local model PROP")))?;
        let data = self.read(PROP, instance)
            .ok_or_else(|| AssetError::Unresolved("authored model PROP is unreadable".into()))?;
        let (model_instance, model_type) = authored_lod(&data, lod)?;
        if model_type != GMDL {
            return Err(AssetError::Other(format!("authored creation LOD{lod} has unsupported model type {model_type:08X}")));
        }
        let model = self.key_for(model_type, model_instance)
            .ok_or_else(|| AssetError::Unresolved(format!("authored LOD{lod} model {model_instance:016X} is absent")))?;
        Ok(AuthoredCreatureModel { creature, prop, model, lod })
    }

    /// Saved GMDL geometry and its explicit sampler-zero texture for inspection.
    /// The single-material viewer rejects differing per-mesh maps explicitly.
    pub fn authored_creature_asset(&self, instance: u64, lod: usize) -> Result<AuthoredCreatureAsset, AssetError> {
        let reference = self.authored_creature_model(instance, lod)?;
        let data = self.read(GMDL, reference.model.scoped_instance())
            .ok_or_else(|| AssetError::Unresolved("authored GMDL is unreadable".into()))?;
        let model = spore_core::gmdl::parse(&data).map_err(|e| AssetError::Other(e.to_string()))?;
        // The geometry parser's legacy skipped-tail flag only understands v8.
        // The strict companion below validates complete v8/v9 tail framing,
        // including expansion material-presence words and source shader data.
        let tail = spore_core::gmdl_skin::parse(&data, &model).map_err(|e| AssetError::Other(e.to_string()))?
            .ok_or_else(|| AssetError::Unresolved("authored GMDL has no saved material bindings".into()))?;
        let texture_binding = shared_sampler_zero(&tail, model.meshes.len())?;
        let texture_key = self.key_for(RASTER, texture_binding.raster_instance)
            .or_else(|| self.key_for(RW4, texture_binding.raster_instance))
            .ok_or_else(|| AssetError::Unresolved(format!("source texture {:016X} is absent", texture_binding.raster_instance)))?;
        let source = self.read(texture_key.type_id, texture_key.scoped_instance())
            .ok_or_else(|| AssetError::Unresolved("source texture is unreadable".into()))?;
        let texture = if texture_key.type_id == RASTER {
            let raster = spore_core::raster::parse(&source).map_err(|e| AssetError::Other(e.to_string()))?;
            TextureData { width: raster.width, height: raster.height,
                rgba: raster.decode_rgba().map_err(|e| AssetError::Other(e.to_string()))? }
        } else {
            let rw = spore_core::rw4::parse(&source).map_err(|e| AssetError::Other(e.to_string()))?;
            let raster = rw.texture(&source).ok_or_else(|| AssetError::Unresolved("source RW4 has no raster".into()))?;
            TextureData { width: raster.width, height: raster.height,
                rgba: spore_core::raster::decode_pixels(raster.format, raster.width, raster.height, raster.payload)
                    .map_err(|e| AssetError::Other(e.to_string()))? }
        };
        let mesh = spore_core::mesh::from_gmdl(&data, &model)?;
        mesh.validate()?;
        Ok(AuthoredCreatureAsset { reference, mesh, texture_key, texture_binding, texture })
    }

    /// A creature's diffuse map: the shipped skin when the install has one, otherwise
    /// one generated from its paint palette and its own UV layout.
    ///
    /// The generated path exists because 656 of the 1,022 creatures in `Spore_Content`
    /// ship no skin raster. They do carry a palette, and their geometry does carry UVs,
    /// so a diffuse map can be produced without inventing anything: the colours are the
    /// creature's own and the layout is the one its triangles use. It cannot reproduce
    /// hand-painted markings, and the note says so.
    pub fn creature_texture(&self, instance: u64) -> Option<(TextureData, String)> {
        if let Some((texture, note)) = self.creature_skin(instance) {
            return Some((texture, note));
        }
        let palette = self.creature_palette(instance)?;
        let (_, mesh) = self.creature(instance).ok()?;
        let stats = spore_core::skin::uv_stats(&mesh);
        let skin = spore_core::skin::generate(&mesh, &palette, 512)?;
        let note = format!(
            "generated {}x{} from palette, {}/{} triangles textured, {} texels ({} marked), no hand-painted markings",
            skin.width,
            skin.height,
            stats.triangles_with_uv,
            stats.triangles_total,
            skin.covered,
            skin.marked
        );
        Some((
            TextureData {
                width: skin.width,
                height: skin.height,
                rgba: skin.rgba,
            },
            note,
        ))
    }

    /// Assemble a creature: every rig entry's part model, placed by its transform.
    ///
    /// A creature is not one resource. The `.creaturedata` file holds a rig table
    /// whose rows name a part prop and give a rotation and translation for it; each
    /// part prop names a model. This walks that chain and merges the results.
    ///
    /// Missing parts are counted rather than treated as failure: 3,745 of the 31,704
    /// rig rows in `Spore_Content` name a model that exists in no package in this
    /// install, and a creature with 9 of 11 parts still tells you whether the rig is
    /// right.
    pub fn creature(
        &self,
        instance: u64,
    ) -> Result<(CreatureAssembly, MeshData), AssetError> {
        let data = self
            .read(CREATURE_DATA, instance)
            .ok_or_else(|| AssetError::Other(format!("no creature {instance:016X}")))?;
        let parsed = spore_core::creature_data::parse(&data)
            .map_err(|e| AssetError::Other(format!("creaturedata parse: {e}")))?;
        if parsed.parts.is_empty() {
            return Err(AssetError::Other(
                "creature rig has no rows".to_string(),
            ));
        }

        let mut assembly = MeshData::assembly();
        let mut drawn = 0usize;
        let mut missing = 0usize;
        let mut failed = 0usize;
        let mut errors: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for part in &parsed.parts {
            // A rig can repeat a part; drawing it twice would double the geometry.
            if !seen.insert((part.part_instance, part.index)) {
                continue;
            }
            let part_mesh = match self.prop_mesh(part.part_instance) {
                Ok((_, mesh)) => mesh,
                Err(AssetError::Unresolved(msg)) => {
                    missing += 1;
                    if errors.len() < 3 {
                        errors.push(format!("{:016X}: {msg}", part.part_instance));
                    }
                    continue;
                }
                Err(AssetError::Other(msg)) => {
                    failed += 1;
                    if errors.len() < 3 {
                        errors.push(format!("{:016X}: {msg}", part.part_instance));
                    }
                    continue;
                }
                Err(err) => {
                    failed += 1;
                    if errors.len() < 3 {
                        errors.push(format!("{:016X}: {err}", part.part_instance));
                    }
                    continue;
                }
            };
            spore_core::mesh::append_transformed(
                &mut assembly,
                &part_mesh,
                part.rotation,
                part.translation,
            );
            drawn += 1;
        }

        if assembly.positions.is_empty() {
            return Err(AssetError::Other(format!(
                "creature {instance:016X}: none of its {} parts resolved ({} missing, {} failed)\n{}",
                parsed.parts.len(),
                missing,
                failed,
                errors.join("\n")
            )));
        }

        let out = CreatureAssembly {
            source: (CREATURE_DATA, instance),
            parts_total: parsed.parts.len(),
            parts_drawn: drawn,
            parts_missing: missing,
            parts_failed: failed,
            errors,
            vertices: assembly.positions.len(),
            triangles: assembly.triangle_count(),
        };
        assembly.validate()?;
        Ok((out, assembly))
    }

    /// Count what props resolve to. Returns
    /// `(props, naming a model, model found, with a texture, LOD slots, missing)`.
    pub fn prop_census(&self) -> (usize, usize, usize, usize, usize, usize) {
        let mut props = 0usize;
        let mut named = 0usize;
        let mut resolved = 0usize;
        let mut with_tex = 0usize;
        let mut lods = 0usize;
        let mut missing = 0usize;
        for entry in self.file.entries() {
            if entry.key.type_id != PROP {
                continue;
            }
            props += 1;
            let Ok(data) = self.file.read(entry) else {
                continue;
            };
            let Ok(tree) = spore_core::prop::parse(&data) else {
                continue;
            };
            if tree.model().is_none() {
                continue;
            }
            named += 1;
            let Ok(r) = self.resolve_prop(entry.key.scoped_instance()) else {
                missing += 1;
                continue;
            };
            resolved += 1;
            lods += r.lods.len();
            if !r.textures.is_empty() {
                with_tex += 1;
            }
        }
        (props, named, resolved, with_tex, lods, missing)
    }

    /// The first colour map a prop names, decoded.
    ///
    /// The prop holds two; both decode to colour images of the same object at
    /// different sizes, so either is a defensible albedo. The first is used.
    pub fn prop_texture(&self, prop_instance: u64) -> Option<(TextureData, String)> {
        let resolved = self.resolve_prop(prop_instance).ok()?;
        for (type_id, instance) in &resolved.textures {
            let Some(data) = self.read(*type_id, *instance) else {
                continue;
            };
            let Ok(raster) = spore_core::raster::parse(&data) else {
                continue;
            };
            let Ok(rgba) = raster.decode_rgba() else {
                continue;
            };
            let note = format!(
                "prop {} {}x{}",
                spore_core::raster::format_name(raster.format),
                raster.width,
                raster.height
            );
            return Some((
                TextureData {
                    width: raster.width,
                    height: raster.height,
                    rgba,
                },
                note,
            ));
        }
        None
    }
}

#[cfg(test)]
mod authored_tests {
    use super::*;
    #[test]
    fn standalone_raster_mips_decode_each_level_and_reject_bad_framing() {
        let mut data = Vec::new();
        for word in [1u32, 2, 2, 2, 8, spore_core::raster::FORMAT_L8, 4] {
            data.extend_from_slice(&word.to_le_bytes());
        }
        data.extend_from_slice(&[1, 2, 3, 4]);
        data.extend_from_slice(&1u32.to_le_bytes());
        data.push(9);
        let chain = decode_standalone_raster_mips(&data).unwrap();
        assert_eq!(chain.levels.len(), 2);
        assert_eq!((chain.levels[0].width, chain.levels[0].height), (2, 2));
        assert_eq!(chain.levels[0].rgba, [1,1,1,255, 2,2,2,255, 3,3,3,255, 4,4,4,255]);
        assert_eq!((chain.levels[1].width, chain.levels[1].height), (1, 1));
        assert_eq!(chain.levels[1].rgba, [9,9,9,255]);
        assert!(decode_standalone_raster_mips(&data[..data.len()-1]).is_err());
        data[32..36].copy_from_slice(&2u32.to_le_bytes());
        assert!(decode_standalone_raster_mips(&data).is_err());
    }
    #[test]
    fn standalone_dxt5_odd_mips_decode_blocks_and_reject_bad_length() {
        fn block(rgb565: u16) -> [u8; 16] {
            let mut bytes = [0u8; 16];
            bytes[0] = 255;
            bytes[1] = 255;
            bytes[8..10].copy_from_slice(&rgb565.to_le_bytes());
            bytes[10..12].copy_from_slice(&rgb565.to_le_bytes());
            bytes
        }
        let mut data = Vec::new();
        for word in [1u32, 5, 3, 3, 8, spore_core::raster::FORMAT_DXT5, 32] {
            data.extend_from_slice(&word.to_le_bytes());
        }
        data.extend_from_slice(&block(0xf800));
        data.extend_from_slice(&block(0x07e0));
        data.extend_from_slice(&16u32.to_le_bytes());
        data.extend_from_slice(&block(0x001f));
        data.extend_from_slice(&16u32.to_le_bytes());
        data.extend_from_slice(&block(0xf800));

        let chain = decode_standalone_raster_mips(&data).unwrap();
        assert_eq!(chain.levels.iter().map(|level| (level.width, level.height, level.rgba.len())).collect::<Vec<_>>(),
                   [(5, 3, 60), (2, 1, 8), (1, 1, 4)]);
        for row in chain.levels[0].rgba.chunks_exact(20) {
            for pixel in row[..16].chunks_exact(4) { assert_eq!(pixel, [255, 0, 0, 255]); }
            assert_eq!(&row[16..20], &[0, 255, 0, 255]);
        }
        assert_eq!(chain.levels[1].rgba, [0, 0, 255, 255, 0, 0, 255, 255]);
        assert_eq!(chain.levels[2].rgba, [255, 0, 0, 255]);
        data[60..64].copy_from_slice(&15u32.to_le_bytes());
        assert!(decode_standalone_raster_mips(&data).is_err());
    }
    use spore_core::gmdl_skin::{GmdlMaterial, GmdlShaderData, GmdlSkin};
    use spore_core::prop::{idents, types};
    use spore_core::shader_binding::TextureBinding;

    fn prop(keys: &[(u32, u64)]) -> Vec<u8> {
        let mut data = (keys.len() as u32).to_be_bytes().to_vec();
        for (id, instance) in keys {
            data.extend_from_slice(&id.to_be_bytes());
            data.extend_from_slice(&types::KEY.to_be_bytes());
            data.extend_from_slice(&0u16.to_be_bytes());
            data.extend_from_slice(&(*instance as u32).to_le_bytes());
            data.extend_from_slice(&GMDL.to_le_bytes());
            data.extend_from_slice(&((*instance >> 32) as u32).to_le_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
        }
        data
    }

    #[test]
    fn authored_lod_uses_each_saved_key_without_namespace_rewriting() {
        let data = prop(&[(idents::MODEL_LOD0, 0x4062_7100_1234_5678),
            (idents::MODEL_LOD1, 0x4062_6201_1234_5678)]);
        assert_eq!(authored_lod(&data, 0).unwrap(), (0x4062_7100_1234_5678, GMDL));
        assert_eq!(authored_lod(&data, 1).unwrap(), (0x4062_6201_1234_5678, GMDL));
        assert!(matches!(authored_lod(&data, 2), Err(AssetError::Unresolved(_))));
        assert!(matches!(authored_lod(&data, 4), Err(AssetError::Other(_))));
    }

    #[test]
    fn authored_lod_never_falls_back_or_crosses_a_native_slot_hole() {
        let only_lower = prop(&[(idents::MODEL_LOD1, 5)]);
        assert!(matches!(authored_lod(&only_lower, 0), Err(AssetError::Unresolved(_))));
        assert!(matches!(authored_lod(&only_lower, 1), Err(AssetError::Unresolved(_))));
        let hole = prop(&[(idents::MODEL_LOD0, 6), (idents::MODEL_LOD2, 7)]);
        assert!(matches!(authored_lod(&hole, 2), Err(AssetError::Unresolved(_))));
    }

    #[test]
    fn authored_lod_rejects_ambiguous_framing_and_unestablished_inheritance() {
        let duplicate = prop(&[(idents::MODEL_LOD0, 5), (idents::MODEL_LOD0, 6)]);
        assert!(matches!(authored_lod(&duplicate, 0), Err(AssetError::Other(_))));
        let mut trailing = prop(&[(idents::MODEL_LOD0, 5)]);
        trailing.push(0);
        assert!(matches!(authored_lod(&trailing, 0), Err(AssetError::Other(_))));
        let mut count_mismatch = prop(&[(idents::MODEL_LOD0, 5)]);
        count_mismatch[..4].copy_from_slice(&2u32.to_be_bytes());
        assert!(matches!(authored_lod(&count_mismatch, 0), Err(AssetError::Other(_))));
        let inherited = prop(&[(idents::MODEL_LOD0, 0xFFFF_FFFF_1234_5678)]);
        assert!(matches!(authored_lod(&inherited, 0), Err(AssetError::Unresolved(_))));
    }

    #[test]
    fn authored_resolution_reports_missing_prop_and_exact_model_without_part_fallback() {
        // Synthetic archive routing fixture: raw creation payloads are markers;
        // this test exercises key resolution before any geometry decoding.
        let rows = vec![(CREATURE_DATA, 1u64, vec![1]), (CREATURE_DATA, 2, vec![2]),
            (CREATURE_DATA, 3, vec![3]), (CREATURE_DATA, 4, vec![4]),
            (PROP, 2, prop(&[(idents::MODEL_LOD1, 5)])),
            (PROP, 3, prop(&[(idents::MODEL_LOD0, 99), (idents::MODEL_LOD1, 5)])),
            (PROP, 4, prop(&[(idents::MODEL_LOD0, 0x7777_0000_0005)])),
            (GMDL, 5, vec![5]), (GMDL, 0x7777_0000_0005, vec![6])];
        let mut data = vec![0u8; 0x60];
        data[..4].copy_from_slice(b"DBPF");
        data[4..8].copy_from_slice(&3u32.to_le_bytes());
        data[0x24..0x28].copy_from_slice(&(rows.len() as u32).to_le_bytes());
        let mut entries = Vec::new();
        for (kind, instance, bytes) in rows {
            entries.push((kind, instance, data.len() as u32, bytes.len() as u32));
            data.extend_from_slice(&bytes);
        }
        let offset = data.len() as u32;
        data[0x40..0x44].copy_from_slice(&offset.to_le_bytes());
        let size = (8 + entries.len() * 28) as u32;
        data[0x2C..0x30].copy_from_slice(&size.to_le_bytes());
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes()); // Shared native instance-high.
        for (kind, instance, offset, length) in entries {
            for word in [kind, (instance >> 32) as u32, instance as u32,
                offset, length | 0x8000_0000, length, 0x0001_0000] {
                data.extend_from_slice(&word.to_le_bytes());
            }
        }
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("rustspore_authored_{}_{nonce}.package", std::process::id()));
        std::fs::write(&path, data).unwrap();
        let source = AssetSource::open(&path).unwrap();
        assert!(matches!(source.authored_creature_model(1, 0), Err(AssetError::Unresolved(_))));
        assert!(matches!(source.authored_creature_model(2, 0), Err(AssetError::Unresolved(_))));
        assert!(matches!(source.authored_creature_model(3, 0), Err(AssetError::Unresolved(_))));
        assert!(matches!(source.authored_creature_model(1234, 0), Err(AssetError::Unresolved(_))));
        let saved = source.authored_creature_model(4, 0).unwrap();
        assert_eq!(saved.model, NativeResourceKey { type_id: GMDL, group_id: 0x7777, instance_id: 5 });
        assert_eq!(source.data(&saved.model).unwrap(), vec![6]);
        assert_eq!(source.read(GMDL, 5).unwrap(), vec![5]);
        assert_eq!(source.lookup(0, 4), None); // Both PROP and CreatureData share this scope/low.
        drop(source);
        std::fs::remove_file(path).unwrap();
    }

    fn binding(sampler: u32, instance: u64) -> TextureBinding {
        TextureBinding { sampler, saved_pointer_word: 0,
            settings: [15, 3, 2, 2, 1, 0, 0, 0], raster_instance: instance }
    }

    fn tail(materials: &[Vec<TextureBinding>]) -> GmdlSkin {
        GmdlSkin { source_offset: 0,
            materials: materials.iter().map(|bindings| {
                let mut payload = (bindings.len() as u32).to_le_bytes().to_vec();
                for binding in bindings {
                    payload.extend_from_slice(&binding.sampler.to_le_bytes());
                    payload.extend_from_slice(&binding.saved_pointer_word.to_le_bytes());
                    payload.extend_from_slice(&binding.settings);
                    payload.extend_from_slice(&binding.raster_instance.to_le_bytes());
                }
                GmdlMaterial { version9_prefix: Some(1),
                    shader_data: vec![GmdlShaderData { id: 0x20D, payload }] }
            }).collect(), palettes: vec![], parts: vec![], footer: [0; 3] }
    }

    #[test]
    fn authored_texture_follows_sampler_zero_across_materials() {
        let expected = binding(0, 0x4062_6100_1234_5678);
        let mut pointer_changed = expected;
        pointer_changed.saved_pointer_word = 0xDEAD_BEEF;
        let skin = tail(&[vec![binding(0, 1), expected, binding(1, 999)],
            vec![pointer_changed, binding(2, 888)]]);
        let actual = shared_sampler_zero(&skin, 2).unwrap();
        assert_eq!(actual.raster_instance, expected.raster_instance);
        assert_eq!(actual.settings, expected.settings);
    }

    #[test]
    fn authored_texture_requires_explicit_matching_map_and_state() {
        let first = binding(0, 11);
        assert!(shared_sampler_zero(&tail(&[vec![first], vec![binding(0, 12)]]), 2).is_err());
        let mut other_state = first;
        other_state.settings[1] = 1;
        assert!(shared_sampler_zero(&tail(&[vec![first], vec![other_state]]), 2).is_err());
        for (byte, value) in [(0, 7), (0, 8), (1, 4), (2, 0), (3, 3), (4, 0)] {
            let mut disabled = first;
            disabled.settings[byte] = value;
            assert!(shared_sampler_zero(&tail(&[vec![disabled]]), 1).is_err());
        }
        assert!(shared_sampler_zero(&tail(&[vec![binding(1, 11)]]), 1).is_err());
        assert!(shared_sampler_zero(&tail(&[vec![first]]), 2).is_err());
        let mut malformed = tail(&[vec![first]]);
        malformed.materials[0].shader_data[0].payload.pop();
        assert!(shared_sampler_zero(&malformed, 1).is_err());
    }
}
