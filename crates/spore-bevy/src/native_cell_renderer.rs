//! Execute original source 3D programs on untouched GMDL buffers and textures.
//! Camera fitting and identity lighting frame are explicit inspection controls.
//! This is not a claim of the original game's complete draw state or screenshot.
#[cfg(windows)]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 1;
#[cfg(all(windows, target_env = "msvc"))]
#[used]
#[link_section = ".drectve"]
static NVIDIA_EXPORT: [u8; b" /EXPORT:NvOptimusEnablement ".len()] =
    *b" /EXPORT:NvOptimusEnablement ";

#[cfg(windows)]
#[path = "native_d3d9.rs"]
mod native_d3d9;
#[cfg(windows)]
#[path = "native_d3dx.rs"]
mod native_d3dx;

#[cfg(windows)]
mod probe {
    use super::native_d3d9::{self, Failure, OffscreenDevice};
    use super::native_d3dx;
    use bevy::math::{Mat4, Quat, Vec3, Vec4};
    use image::ImageEncoder;
    use spore_core::{
        gmdl, gmdl_skin,
        prop::{self, PropTree, PropValue},
        raster,
        shader_binding::{self, MaterialParams, UniformDescriptor},
        shader_codegen::{self, SourceTemplates},
        shader_fragments, shader_pack,
        shader_selection::{self, SelectionContext},
    };
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::Path,
        ptr,
    };
    use windows::Win32::Graphics::Direct3D9::*;

    use dbpf::native_index::{NativePackageIndex, NativeResourceKey};

    struct Sources(Vec<NativePackageIndex>);
    impl Sources {
        fn open_install(install: &Path) -> Result<Self, Failure> {
            fn collect(root: &Path, paths: &mut Vec<std::path::PathBuf>) -> Result<(), Failure> {
                for entry in std::fs::read_dir(root)? {
                    let entry = entry?;
                    let kind = entry.file_type()?;
                    if kind.is_symlink() {
                        continue;
                    }
                    if kind.is_dir() {
                        collect(&entry.path(), paths)?;
                    } else if kind.is_file()
                        && entry
                            .path()
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("package"))
                    {
                        paths.push(entry.path());
                    }
                }
                Ok(())
            }
            let mut paths = ["PatchData", "Spore_Game", "Spore_Content", "Spore_Graphics"]
                .map(|name| install.join("Data").join(format!("{name}.package")))
                .to_vec();
            let mut remaining = Vec::new();
            collect(install, &mut remaining)?;
            remaining.sort();
            for path in remaining {
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
            let packages = paths
                .iter()
                .map(NativePackageIndex::open)
                .collect::<Result<Vec<_>, _>>()?;
            println!("SOURCE_3D_PACKAGES count={} precedence=inspection_base_then_path_order retail_precedence=false", packages.len());
            println!("SOURCE_3D_INDEX layout=native_flag_driven key=instance_type_scope metadata=current_record");
            Ok(Self(packages))
        }
        fn read_key(&self, key: NativeResourceKey) -> Result<Vec<u8>, Failure> {
            for package in &self.0 {
                if let Some(entry) = package.index().find_unique(key)? {
                    if entry.decompressed_size.max(entry.stored_size) > 32 * 1024 * 1024 {
                        return Err("source resource too large".into());
                    }
                    return Ok(package.read(entry)?);
                }
            }
            Err(format!("source missing exact model key {key}").into())
        }
        fn read(&self, typ: u32, instance: u64) -> Result<Vec<u8>, Failure> {
            self.read_key(NativeResourceKey::from_scoped_instance(typ, instance))
        }
        fn properties(&self, instance: u64) -> Result<PropTree, Failure> {
            let p = prop::parse(&self.read(0x00b1b104, instance)?)?;
            if !p.exact {
                return Err("inexact source property framing".into());
            }
            Ok(p)
        }
        fn raster(&self, instance: u64) -> Result<raster::Raster, Failure> {
            if self
                .0
                .iter()
                .map(|p| {
                    p.index()
                        .find_unique(NativeResourceKey::from_scoped_instance(
                            0x2f4e681c, instance,
                        ))
                })
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(Option::is_some)
            {
                return Ok(raster::parse(&self.read(0x2f4e681c, instance)?)?);
            }
            let source = self.read(0x2f4e681b, instance)?;
            let rw = spore_core::rw4::parse(&source)?;
            let texture = rw.texture(&source).ok_or("source RW4 has no raster")?;
            if texture.width == 0
                || texture.height == 0
                || texture.mipmap_levels == 0
                || texture.mipmap_levels > texture.width.max(texture.height).ilog2() + 1
            {
                return Err("unsupported RW4 mip extent".into());
            }
            // RW4 BaseResource stores contiguous levels; the standalone
            // RASTER container prefixes each later level. Normalize only this
            // in-memory container framing, preserving every source block byte.
            let mut data = Vec::new();
            let mut at = 0;
            let mut base_size = 0;
            for level in 0..texture.mipmap_levels {
                let w = (texture.width >> level).max(1) as usize;
                let h = (texture.height >> level).max(1) as usize;
                let size = match texture.format {
                    raster::FORMAT_DXT1 => w.div_ceil(4) * h.div_ceil(4) * 8,
                    raster::FORMAT_DXT3 | raster::FORMAT_DXT5 => w.div_ceil(4) * h.div_ceil(4) * 16,
                    raster::FORMAT_RAW_RGBA => w * h * 4,
                    raster::FORMAT_L8 => w * h,
                    _ => return Err("unsupported RW4 texture format".into()),
                };
                if level == 0 {
                    base_size = u32::try_from(size)?;
                } else {
                    data.extend(u32::try_from(size)?.to_le_bytes());
                }
                let end = at + size;
                data.extend_from_slice(
                    texture
                        .payload
                        .get(at..end)
                        .ok_or("truncated RW4 mip chain")?,
                );
                at = end;
            }
            if at != texture.payload.len() {
                return Err("unconsumed RW4 texture bytes".into());
            }
            Ok(raster::Raster {
                width: texture.width,
                height: texture.height,
                mipmaps: texture.mipmap_levels,
                format: texture.format,
                data_size: base_size,
                data,
            })
        }
    }
    fn scalar(tree: &PropTree, id: u32) -> Result<f32, Failure> {
        match tree.value(id) {
            Some(PropValue::Float(v)) if v.is_finite() => Ok(*v),
            _ => Err(format!("source property {id:08X} needs finite FLOAT").into()),
        }
    }
    fn color(tree: &PropTree, id: u32) -> Result<[f32; 3], Failure> {
        match tree.value(id) {
            Some(PropValue::ColorRGB(r, g, b))
                if r.is_finite() && g.is_finite() && b.is_finite() =>
            {
                Ok([*r, *g, *b])
            }
            _ => Err(format!("source property {id:08X} needs finite RGB").into()),
        }
    }
    fn rows(matrix: Mat4) -> Vec<[f32; 4]> {
        (0..4).map(|i| matrix.row(i).to_array()).collect()
    }
    struct Inputs {
        model: Mat4,
        clip: Mat4,
        direction: [f32; 4],
        sh: Vec<[f32; 4]>,
        fresh_world_sh_extra: [f32; 4],
        camera_distance: [f32; 4],
        params: [f32; 4],
        lights: [[f32; 4]; 2],
    }
    impl Inputs {
        fn uniform(&self, id: u16, count: usize) -> Result<Vec<[f32; 4]>, Failure> {
            let data = match id {
                0x22 => vec![[1.; 4]], // explicit white material color
                8 => rows(self.model).into_iter().take(3).collect(),
                6 => rows(self.clip),
                0x21 => vec![self.direction],
                0x21a if count <= self.sh.len() => self.sh[..count].to_vec(),
                0x219 => vec![self.fresh_world_sh_extra],
                0x21b => vec![self.camera_distance],
                0x210 => vec![self.params],
                0x214 => self.lights.to_vec(),
                0x20c => vec![[1.; 4]], // measured absent-pointer native fallback
                _ => return Err(format!("unestablished uniform {id:04X}/{count}").into()),
            };
            if data.len() != count || data.iter().flatten().any(|x| !x.is_finite()) {
                return Err(format!("invalid uniform extent/value {id:04X}/{count}").into());
            }
            Ok(data)
        }
    }
    unsafe fn upload(
        device: &IDirect3DDevice9,
        program: &shader_pack::CompiledProgram,
        inputs: &Inputs,
        vertex: bool,
    ) -> Result<(), Failure> {
        if program.uniforms.len() != program.uniform_words.len() {
            return Err("source uniform/register cardinality mismatch".into());
        }
        for (record, start) in program.uniforms.iter().zip(&program.uniform_words) {
            let descriptor = UniformDescriptor { raw: *record };
            let count = usize::from(descriptor.register_count());
            let data = inputs.uniform(descriptor.callback_id(), count)?;
            if vertex {
                device.SetVertexShaderConstantF(*start, data.as_ptr().cast(), count as u32)?;
            } else {
                device.SetPixelShaderConstantF(*start, data.as_ptr().cast(), count as u32)?;
            }
            println!("SOURCE_3D_UNIFORM stage={} callback={:04X} dirty={:04X} start={start} count={count}",
                if vertex {"vs"} else {"ps"}, descriptor.callback_id(), descriptor.dirty_data_id());
        }
        Ok(())
    }
    pub fn render(
        install: &Path,
        output: &Path,
        instance: u64,
        model_key: Option<NativeResourceKey>,
        creature_camera: bool,
        diagnostic_skin_translation: bool,
    ) -> Result<(), Failure> {
        if output.exists() {
            return Err("capture output already exists".into());
        }
        let sources = Sources::open_install(install)?;
        let raw = match model_key {
            Some(key) if key.type_id == 0x00e6bce5 && key.scoped_instance() == instance => {
                sources.read_key(key)?
            }
            Some(_) => return Err("source model key requires the exact requested GMDL".into()),
            None => sources.read(0x00e6bce5, instance)?,
        };
        let model = gmdl::parse(&raw)?;
        let tail = gmdl_skin::parse(&raw, &model)?.ok_or("source GMDL has no material tail")?;
        if tail.materials.len() != model.meshes.len() {
            return Err("material slot mismatch".into());
        }
        let pack = shader_pack::parse(&sources.read(0x0469a3f7, 0x4021200400000003)?)?;
        let builder = pack
            .builders
            .iter()
            .find(|b| b.header.shader_id() == 0x80000002 && b.header.name == "generic_static")
            .ok_or("missing source static builder")?;
        let variant = builder
            .variants
            .iter()
            .find(|v| v.render_type == 0)
            .ok_or("missing source static render type0")?;
        // Validate the complete diagnostic draw set before issuing either draw.
        // The direct-path value below is a conditional association for this
        // experiment, not an assertion that this renderer is Spore's path.
        if diagnostic_skin_translation {
            if instance != 0x4061_7100_780a_9eaf || model.meshes.len() != 2
                || model.vertex_buffers.len() != 1 || model.vertex_buffers[0].vertex_count != 5947
                || model.meshes.iter().any(|m| m.vertex_index != 0)
                || model.meshes[0].triangle_index == model.meshes[1].triangle_index
                || tail.palettes.len() != 2
                || (tail.palettes[0].first_word, tail.palettes[0].bone_count) != (0, 64)
                || (tail.palettes[1].first_word, tail.palettes[1].bone_count) != (64, 19)
            { return Err("diagnostic requires the known Jawhead two-mesh shared-buffer layout and palette descriptors".into()); }
            for (i, mesh) in model.meshes.iter().enumerate() {
                let vb = model.vertex_buffers.get(usize::try_from(mesh.vertex_index)?).ok_or("diagnostic vertex reference")?;
                let format = model.vertex_formats.get(usize::try_from(vb.format_index)?).ok_or("diagnostic vertex format")?;
                let position = format.elements.iter().find(|e| e.usage == 0 && e.usage_index == 0)
                    .ok_or("diagnostic position declaration absent")?;
                if position.offset < 0 { return Err("diagnostic position offset invalid".into()); }
                let position_start = position.offset as usize;
                for element in format.elements.iter().filter(|e| e.usage == 3 || e.usage == 6) {
                    if element.offset < 0 { return Err("diagnostic normal/tangent offset invalid".into()); }
                    let start = element.offset as usize;
                    if start < position_start + 12 && position_start < start + element.size() {
                        return Err("diagnostic position overlaps normal/tangent".into());
                    }
                }
                let mut data: BTreeMap<u16, Vec<u8>> = tail.materials[i].shader_data.iter()
                    .map(|d| Ok((u16::try_from(d.id)?, d.payload.clone())))
                    .collect::<Result<_, std::num::TryFromIntError>>()?;
                data.insert(0x219, vec![]);
                if format.native_normal_unpack_required(true) { data.insert(0x22c, vec![]); }
                let bindings = shader_binding::texture_bindings(data.get(&0x20d).ok_or("diagnostic textures absent")?)?;
                let samplers: BTreeSet<u16> = bindings.iter().map(|b| u16::try_from(b.sampler)).collect::<Result<_, _>>()?;
                let context = SelectionContext { declaration_features: format.native_shader_features()?, shader_data: &data, available_samplers: &samplers };
                let vs = shader_selection::select(&variant.vertex, &context)?;
                if vs.fragment_ids.as_slice() != [0x14, 0x17, 0x19, 0x15, 0x31, 0x30, 0x29, 0x01] {
                    return Err(format!("diagnostic requires non-skinning vertex recipe for mesh {i}; got {:02X?}", vs.fragment_ids).into());
                }
            }
            println!("SOURCE_3D_DIAGNOSTIC kind=synthetic_position_translation direct_path=conditional_assumption actor_pose=false live_animation=false normals_tangents=unchanged");
        }
        let light = sources.properties(0x40200100fced73a2)?;
        if light.value(0x0100eac6) != Some(&PropValue::Bool(false)) {
            return Err("camera-space lighting requires an unrecovered SH rotation".into());
        }
        let exposure = scalar(&light, 0x0100eac4)?;
        let coefficients = light
            .items
            .iter()
            .find(|i| i.id == 0x0100eac5)
            .ok_or("source SH coefficients absent")?;
        if coefficients.prop_type != 0x32 || coefficients.values.len() != 25 {
            return Err("unestablished source SH coefficient layout".into());
        }
        let mut colors = Vec::new();
        for value in &coefficients.values {
            match value {
                PropValue::ColorRGB(r, g, b) => colors.push([*r, *g, *b]),
                _ => return Err("invalid SH coefficient type".into()),
            }
        }
        // No optional SH color/combination properties occur in this measured
        // preset. Reject a changed preset rather than silently dropping them.
        let allowed = [
            0x0100eab6, 0x0100eab7, 0x0100eab8, 0x0100eab9, 0x0100eaba, 0x0100eabb, 0x0100eabc,
            0x0100eabd, 0x0100eabe, 0x0100eabf, 0x0100eac0, 0x0100eac1, 0x0100eac2, 0x0100eac3,
            0x0100eac4, 0x0100eac5, 0x0100eac6,
        ];
        if light.items.iter().any(|i| !allowed.contains(&i.id)) {
            return Err("unestablished additional source lighting property".into());
        }
        // Fresh lighting-world ctor70CC10 clears219 RGB/W. Preset ctor706770
        // defaults optional-light strength+D0 and VECTOR2+124/+128 to +0;
        // 708E10 copies+124 to219.w and disables the709F00 optional RGB writer.
        // Both override properties are excluded above. Reused worlds can retain
        // prior RGB, so this applies only to this explicit fresh context.
        let fresh_world_sh_extra = [0.; 4];
        let sh = shader_binding::identity_sh_prefix(&colors, exposure, 16)?;
        let direction = match light.value(0x0100eab8) {
            Some(PropValue::Vector3(x, y, z)) => Vec3::new(*x, *y, *z),
            _ => return Err("source direct-light direction absent".into()),
        };
        // 70A820 normalizes the source direction. 708890 negates it into the
        // light object, then callback 779140 negates it again for this uniform.
        let direction = direction / (direction.length_squared() + 1e-8).sqrt();
        let light_color = color(&light, 0x0100eab9)?;
        let strength = scalar(&light, 0x0100eaba)?;
        let lights = [
            direction.extend(0.).to_array(),
            [
                light_color[0] * strength,
                light_color[1] * strength,
                light_color[2] * strength,
                1.,
            ],
        ];
        let (setup_key, camera_low) = if creature_camera {
            (0x40600100cf2dc8d0, 0xb0d43c20u32)
        } else {
            (0x406001003615a30b, 0x02ad26c9u32)
        };
        let setup = sources.properties(setup_key)?;
        if setup.key(0xb02d871c) != Some((u64::from(camera_low), 0x00b1b104)) {
            return Err("source editor thumbnail-camera link changed".into());
        }
        let camera_key = 0x4041010000000000 | u64::from(camera_low);
        let camera = sources.properties(camera_key)?;
        if camera.key(0x00b2cccb).map(|(id, _)| id) != Some(0x3f3b2714) {
            return Err("unestablished source thumbnail-camera parent".into());
        }
        let camera_parent = sources.properties(0x404101003f3b2714)?;
        if camera_parent.value(0x00ed3928) != Some(&PropValue::Uint32(0x00edd3bb)) {
            return Err("source thumbnail-camera class changed".into());
        }
        let angles = (scalar(&camera, 0x00c7c4fd)?, scalar(&camera, 0x00c7c4fc)?);
        let rotation = Quat::from_rotation_z(-angles.0 * 0.017453292)
            * Quat::from_rotation_x(angles.1 * 0.017453292);
        let (right, forward, up) = (rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z);
        let near = scalar(&camera, 0x01102b20)?;
        let far = scalar(&camera, 0x01102b2f)?;
        let fov = scalar(&camera, 0x044c6220)?;
        let minimum = Vec3::new(model.bbox_min.x, model.bbox_min.y, model.bbox_min.z);
        let maximum = Vec3::new(model.bbox_max.x, model.bbox_max.y, model.bbox_max.z);
        let centre = (minimum + maximum) * 0.5;
        let radius = (maximum - minimum).length() * 0.5;
        if !radius.is_finite() || radius <= 0. {
            return Err("invalid source bounds".into());
        }
        // Complete both diagnostic deformations before creating a device or
        // beginning a scene. Each draw receives its own full buffer copy.
        let diagnostic_buffers = if diagnostic_skin_translation {
            use spore_core::gmdl_direct_draw_position_diagnostic_v1::{deform_positions, Matrix};
            use spore_core::gmdl_direct_draw_influences_v1::DrawPath;
            let identity: Matrix = [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.]];
            let mut palette = vec![identity; 83];
            for matrix in &mut palette[..64] { matrix[0][3] = radius * 0.15; }
            for matrix in &mut palette[64..] { matrix[0][3] = -radius * 0.15; }
            let mut buffers = Vec::with_capacity(2);
            for i in 0..2 {
                let copy = deform_positions(&raw, &model, &tail, i,
                    DrawPath::DirectFresh769a80Unmodified, &palette)
                    .map_err(|e| format!("position diagnostic mesh {i}: {e:?}"))?;
                buffers.push(copy);
            }
            for i in 0..2 {
                println!("SOURCE_3D_DIAGNOSTIC_DRAW mesh={i} translation_x={} copied_buffer=true normals_tangents=unchanged", if i == 0 { radius * 0.15 } else { -radius * 0.15 });
            }
            Some(buffers)
        } else { None };
        let model_matrix =
            Mat4::from_scale(Vec3::splat(1. / radius)) * Mat4::from_translation(-centre);
        let distance = 1.15 / (fov * 0.5).sin();
        let eye = -forward * distance;
        let camera_distance = shader_binding::camera_distance_constant(
            model_matrix.w_axis.truncate().to_array(),
            eye.to_array(),
            spore_core::native_x87::Context::from_control_word(0x027f)?,
        )?;
        println!("SOURCE_3D_CAMERA_DISTANCE world_translation={:?} camera_override={:?} constant={camera_distance:?} x87_control=027F context=inspection", model_matrix.w_axis.truncate(), eye);
        // Explicit inspection fit with recovered source orientation/FOV and
        // finite D3D depth. Complete native camera anchor/timing remains open.
        let view = Mat4::from_cols(
            Vec4::new(right.x, up.x, forward.x, 0.),
            Vec4::new(right.y, up.y, forward.y, 0.),
            Vec4::new(right.z, up.z, forward.z, 0.),
            Vec4::new(-right.dot(eye), -up.dot(eye), -forward.dot(eye), 1.),
        );
        let clip = Mat4::perspective_lh(fov, 1., near, far) * view * model_matrix;
        println!("SOURCE_3D_CONTEXT model={instance:016X} profile=3 builder=generic_static render_type=0 unpack_enabled=true light=40200100FCED73A2 light_frame=identity lighting_world=fresh sh_extra219=source_initial_zero camera={camera_key:016X} editor_setup={setup_key:016X} fit=inspection material_color=white tint=fallback_white retail_fidelity=false sh_band_roundtrip=true");
        let target = unsafe { OffscreenDevice::new(768, 768)? };
        let device = &target.device;
        unsafe {
            for (state, value) in [
                (D3DRS_ZENABLE, 1),
                (D3DRS_ZWRITEENABLE, 1),
                (D3DRS_ZFUNC, D3DCMP_LESSEQUAL.0 as u32),
                (D3DRS_LIGHTING, 0),
                (D3DRS_ALPHABLENDENABLE, 0),
                (D3DRS_ALPHATESTENABLE, 0),
                (D3DRS_CULLMODE, D3DCULL_NONE.0 as u32),
                (D3DRS_SRGBWRITEENABLE, 0),
                (D3DRS_COLORWRITEENABLE, 15),
            ] {
                device.SetRenderState(state, value)?;
            }
            device.Clear(
                0,
                ptr::null(),
                (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                0xff18202e,
                1.,
                0,
            )?;
            device.BeginScene()?;
        }
        let mut total_triangles = 0;
        for (i, mesh) in model.meshes.iter().enumerate() {
            let vb = model
                .vertex_buffers
                .get(usize::try_from(mesh.vertex_index)?)
                .ok_or("invalid vertex reference")?;
            let format = model
                .vertex_formats
                .get(usize::try_from(vb.format_index)?)
                .ok_or("invalid declaration reference")?;
            let triangles = model
                .triangle_buffers
                .get(usize::try_from(mesh.triangle_index)?)
                .ok_or("invalid triangle reference")?;
            if triangles.primitive_type != 4
                || triangles.indices.len() % 3 != 0
                || triangles
                    .indices
                    .iter()
                    .any(|v| *v >= vb.vertex_count as u32)
                || vb.vertex_count <= 0
                || vb.buffer_size < 0
                || vb.stride == 0
                || vb.buffer_size as usize != vb.stride * vb.vertex_count as usize
            {
                return Err("unsupported source vertex/triangle extent".into());
            }
            let vertices = raw
                .get(vb.data_offset..vb.data_offset + vb.buffer_size as usize)
                .ok_or("source vertex data truncated")?;
            let vertices = if let Some(buffers) = &diagnostic_buffers {
                buffers[i].as_slice()
            } else { vertices };
            let material = &tail.materials[i];
            let mut data: BTreeMap<u16, Vec<u8>> = material
                .shader_data
                .iter()
                .map(|d| Ok((u16::try_from(d.id)?, d.payload.clone())))
                .collect::<Result<_, std::num::TryFromIntError>>()?;
            data.insert(0x219, vec![]);
            if format.native_normal_unpack_required(true) {
                data.insert(0x22c, vec![]);
            }
            let bindings = shader_binding::texture_bindings(
                data.get(&0x20d).ok_or("source textures absent")?,
            )?;
            let mut samplers = BTreeSet::new();
            let mut textures = Vec::new();
            for binding in &bindings {
                if binding.availability() != Some(1) {
                    return Err("unestablished source sampler availability".into());
                }
                if binding.sampler > 7 || samplers.contains(&(binding.sampler as u16)) {
                    return Err("unsupported or duplicate source sampler".into());
                }
                let raster = sources.raster(binding.raster_instance)?;
                let texture = unsafe { target.upload_raster(&raster)? };
                let address = binding
                    .address_mode()
                    .ok_or("source address state absent")?;
                let filter = binding
                    .min_mag_filter()
                    .ok_or("source min/mag state absent")?;
                let mip = binding.mip_filter().ok_or("source mip state absent")?;
                if !(1..=3).contains(&address)
                    || !(1..=2).contains(&filter)
                    || !(0..=2).contains(&mip)
                {
                    return Err("unsupported source sampler value".into());
                }
                unsafe {
                    device.SetTexture(binding.sampler, &texture)?;
                    for (state, value) in [
                        (D3DSAMP_ADDRESSU, address),
                        (D3DSAMP_ADDRESSV, address),
                        (D3DSAMP_ADDRESSW, address),
                        (D3DSAMP_MINFILTER, filter),
                        (D3DSAMP_MAGFILTER, filter),
                        (D3DSAMP_MIPFILTER, mip),
                        (D3DSAMP_SRGBTEXTURE, 0),
                    ] {
                        device.SetSamplerState(binding.sampler, state, value as u32)?;
                    }
                }
                println!("SOURCE_3D_TEXTURE mesh={i} sampler={} resource={:016X} size={}x{} mips={} format={:08X} source_blocks=true",
                    binding.sampler,binding.raster_instance,raster.width,raster.height,raster.mipmaps,raster.format);
                samplers.insert(u16::try_from(binding.sampler)?);
                textures.push(texture);
            }
            if !samplers.contains(&0) {
                return Err("source diffuse sampler absent".into());
            }
            let context = SelectionContext {
                declaration_features: format.native_shader_features()?,
                shader_data: &data,
                available_samplers: &samplers,
            };
            let vs = shader_selection::select(&variant.vertex, &context)?;
            let ps = shader_selection::select(&variant.pixel, &context)?;
            let generated_program;
            let vs_program = match pack.compiled_vertex.lookup(&vs.fragment_ids)? {
                Some(cached) => cached,
                None => {
                    if builder.header.words[4] != 0xfffe0300 {
                        return Err("unestablished source vertex compiler profile".into());
                    }
                    let image = std::fs::read(install.join("SporeBin/SporeApp.exe"))?;
                    let templates = SourceTemplates::from_original_executable(&image)?;
                    let library =
                        shader_fragments::parse(&sources.read(0x0469a3f7, 0x4021200200000003)?)?;
                    let source =
                        shader_codegen::generate_vertex(&library, &vs.fragment_ids, &templates)?;
                    let flags = if builder.header.words[6] & 2 != 0 {
                        0x32
                    } else {
                        0x12
                    };
                    let compiled = native_d3dx::Compiler::system()?.compile(
                        &source.hlsl,
                        templates.entrypoint()?,
                        "vs_3_0",
                        flags,
                    )?;
                    if !compiled.diagnostics.is_empty() {
                        println!("SOURCE_3D_COMPILE_DIAGNOSTICS {:?}", compiled.diagnostics);
                    }
                    let mut signature = [0; 32];
                    signature[..vs.fragment_ids.len()].copy_from_slice(&vs.fragment_ids);
                    let uniforms: Vec<_> = source.uniforms.iter().map(|u| u.metadata).collect();
                    let flags_word = source
                        .uniforms
                        .iter()
                        .fold(0, |mask, u| mask | u.flags_word());
                    println!("SOURCE_3D_COMPILE stage=vs recipe={:02X?} source_bytes={} bytecode_bytes={} compiler=system-d3dx9_27 flags={flags:02X} original_templates=true",
                        vs.fragment_ids,source.hlsl.len(),compiled.bytecode.len());
                    generated_program = shader_pack::CompiledProgram {
                        signature,
                        bytecode: compiled.bytecode,
                        uniforms,
                        uniform_words: source.registers,
                        flags_word,
                    };
                    &generated_program
                }
            };
            let ps_program = pack
                .compiled_pixel
                .lookup(&ps.fragment_ids)?
                .ok_or_else(|| {
                    format!(
                        "source pixel recipe uncached: mesh={i} recipe={:02X?}",
                        ps.fragment_ids
                    )
                })?;
            let vs_tokens = native_d3d9::tokens(&vs_program.bytecode, 0xfffe0300)?;
            let ps_tokens = native_d3d9::tokens(&ps_program.bytecode, 0xffff0300)?;
            let mut elements = Vec::new();
            for e in &format.elements {
                if e.stream != 0
                    || e.offset < 0
                    || e.method != 0
                    || !(0..=16).contains(&e.element_type)
                    || !(0..=13).contains(&e.usage)
                    || e.usage_index < 0
                    || e.offset as usize + e.size() > vb.stride
                {
                    return Err("unsupported source declaration element".into());
                }
                elements.push(D3DVERTEXELEMENT9 {
                    Stream: 0,
                    Offset: e.offset as u16,
                    Type: e.element_type as u8,
                    Method: e.method as u8,
                    Usage: e.usage as u8,
                    UsageIndex: e.usage_index as u8,
                });
            }
            elements.push(D3DVERTEXELEMENT9 {
                Stream: 255,
                Type: D3DDECLTYPE_UNUSED.0 as u8,
                ..Default::default()
            });
            let inputs = Inputs {
                model: model_matrix,
                clip,
                direction: forward.extend(0.).to_array(),
                sh: sh.clone(),
                fresh_world_sh_extra,
                camera_distance,
                params: MaterialParams::decode(
                    data.get(&0x210).ok_or("source material params absent")?,
                )?
                .float4(),
                lights,
            };
            let count = u32::try_from(triangles.indices.len() / 3)?;
            unsafe {
                device.SetVertexDeclaration(&device.CreateVertexDeclaration(elements.as_ptr())?)?;
                device.SetVertexShader(&device.CreateVertexShader(vs_tokens.as_ptr())?)?;
                device.SetPixelShader(&device.CreatePixelShader(ps_tokens.as_ptr())?)?;
                upload(device, vs_program, &inputs, true)?;
                upload(device, ps_program, &inputs, false)?;
                device.DrawIndexedPrimitiveUP(
                    D3DPT_TRIANGLELIST,
                    0,
                    vb.vertex_count as u32,
                    count,
                    triangles.indices.as_ptr().cast(),
                    D3DFMT_INDEX32,
                    vertices.as_ptr().cast(),
                    vb.stride as u32,
                )?;
            }
            total_triangles += count;
            println!("SOURCE_3D_DRAW mesh={i} vertices={} triangles={count} vs={:02X?} ps={:02X?} untouched_vertex_bytes={}",vb.vertex_count,vs.fragment_ids,ps.fragment_ids,!diagnostic_skin_translation);
        }
        unsafe {
            device.EndScene()?;
        }
        let rgba = unsafe { target.read_rgba()? };
        let changed = rgba
            .chunks_exact(4)
            .filter(|p| *p != [0x18, 0x20, 0x2e, 0xff])
            .count();
        if changed < 1000 {
            return Err(
                format!("source draw produced only {changed} non-background pixels").into(),
            );
        }
        if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        image::codecs::png::PngEncoder::new(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?,
        )
        .write_image(&rgba, 768, 768, image::ExtendedColorType::Rgba8)?;
        println!("SOURCE_3D_RESULT triangles={total_triangles} non_background_pixels={changed} output={output:?} presented=false");
        Ok(())
    }
}
#[cfg(windows)]
pub fn capture(
    install: &std::path::Path,
    output: &std::path::Path,
    instance: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    probe::render(install, output, instance, None, false, false)
}
#[cfg(windows)]
pub fn capture_key(
    install: &std::path::Path,
    output: &std::path::Path,
    model: dbpf::native_index::NativeResourceKey,
    diagnostic_skin_translation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    probe::render(install, output, model.scoped_instance(), Some(model), false, diagnostic_skin_translation)
}

#[cfg(windows)]
pub fn capture_creature_key(
    install: &std::path::Path,
    output: &std::path::Path,
    model: dbpf::native_index::NativeResourceKey,
) -> Result<(), Box<dyn std::error::Error>> {
    probe::render(install, output, model.scoped_instance(), Some(model), true, false)
}
#[cfg(not(windows))]
pub fn capture(
    _install: &std::path::Path,
    _output: &std::path::Path,
    _instance: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("Original D3D9 rendering requires Windows".into())
}
