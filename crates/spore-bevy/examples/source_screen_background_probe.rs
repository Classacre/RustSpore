//! Diagnostic offscreen composition of indexed original source assets.
//! Usage: source_screen_background_probe <PatchData.package|Spore_Graphics.package>
//!        <texture.package> <new.png>
//! Profile 3 is the default; prepend --profile2 for the controlled profile 2.
//! All paths are explicit. This is not a retail screenshot.
//! A separate --inspect mode joins a hash-checked decoded EFFDIR to an explicit
//! shader pack and texture, with caller-supplied viewport and color inputs.
//! Package files must be trusted, stable installed assets: the bounded header
//! preflight and index reader reopen the path, leaving a TOCTOU window. A
//! matching file name and key do not authenticate contents or prove precedence.
#[cfg(all(windows, feature = "native_shader_probe"))]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 1;
#[cfg(all(windows, target_env = "msvc", feature = "native_shader_probe"))]
#[used]
#[link_section = ".drectve"]
static NVIDIA_EXPORT: [u8; b" /EXPORT:NvOptimusEnablement ".len()] =
    *b" /EXPORT:NvOptimusEnablement ";

#[cfg(all(windows, feature = "native_shader_probe"))]
#[path = "../src/native_d3d9.rs"]
mod native_d3d9;

#[cfg(all(windows, feature = "native_shader_probe"))]
fn profile_tokens(profile: u64) -> Result<(u32, u32), &'static str> {
    match profile {
        2 => Ok((0xfffe_0200, 0xffff_0201)),
        3 => Ok((0xfffe_0300, 0xffff_0300)),
        _ => Err("only explicitly supported source profiles 2 and 3 are admitted"),
    }
}

#[cfg(all(test, windows, feature = "native_shader_probe"))]
#[test]
fn source_profile_admission_is_exact() {
    assert_eq!(profile_tokens(2), Ok((0xfffe_0200, 0xffff_0201)));
    assert_eq!(profile_tokens(3), Ok((0xfffe_0300, 0xffff_0300)));
    assert!(profile_tokens(0).is_err());
    assert!(profile_tokens(4).is_err());
    let valid_vs2 = [0x00, 0x02, 0xfe, 0xff, 0xff, 0xff, 0x00, 0x00];
    assert!(native_d3d9::tokens(&valid_vs2, 0xfffe_0200).is_ok());
    assert!(native_d3d9::tokens(&valid_vs2, 0xfffe_0300).is_err());
}

#[cfg(all(windows, feature = "native_shader_probe"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
    use image::ImageEncoder;
    use sha2::{Digest, Sha256};
    use spore_core::{cell_effdir_framing_v3, cell_screen_quad_v1, cell_skybox_inspection_v1, raster, rw4, shader_pack};
    use std::{fs::{File, OpenOptions}, io::Read, path::Path, ptr};
    use windows::Win32::Graphics::Direct3D9::*;

    let mut args: Vec<_> = std::env::args_os().skip(1).collect();
    let profile = if args.first().is_some_and(|arg| arg == "--profile2") {
        args.remove(0);
        2
    } else {
        3
    };
    let (vertex_version, pixel_version) = profile_tokens(profile)?;
    fn argument<'a>(args: &'a [std::ffi::OsString], index: usize) -> Result<&'a str, Box<dyn std::error::Error>> {
        args.get(index).and_then(|s| s.to_str()).ok_or_else(|| format!("argument {index} must be UTF-8").into())
    }
    fn sha(expected: &str, bytes: &[u8], label: &str) -> Result<String, Box<dyn std::error::Error>> {
        if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("{label} SHA-256 must be 64 hex digits").into());
        }
        let actual = format!("{:x}", Sha256::digest(bytes));
        if !actual.eq_ignore_ascii_case(expected) { return Err(format!("{label} SHA-256 mismatch: {actual}").into()); }
        Ok(actual)
    }
    fn finite(text: &str) -> Result<f32, Box<dyn std::error::Error>> {
        let value = text.parse::<f32>()?;
        if !value.is_finite() { return Err("inspection numbers must be finite".into()); }
        Ok(value)
    }

    let inspect = args.first().is_some_and(|arg| arg == "--inspect");
    if (inspect && args.len() != 14) || (!inspect && args.len() != 3) {
        return Err("usage: source_screen_background_probe [--profile2] <PatchData.package|Spore_Graphics.package> <texture.package> <new.png>\n       source_screen_background_probe [--profile2] --inspect <decoded-effdir> <effdir-sha256> <shader-package> <pack-sha256> <shader-id-hex> <texture-package> <rw4-sha256> <width>x<height> <offset-x> <offset-y> <r,g,b,a> <rtx|amd> <new.png>".into());
    }
    let (shader_path, texture_path, output, shader_id, width, height, offset_x, offset_y, color, amd,
         pack_hash, texture_hash, screen) = if inspect {
        let effdir_path = Path::new(&args[1]);
        let mut effdir_bytes = Vec::new();
        File::open(effdir_path)?.take(32 * 1024 * 1024 + 1).read_to_end(&mut effdir_bytes)?;
        if effdir_bytes.len() > 32 * 1024 * 1024 { return Err("decoded EFFDIR exceeds bound".into()); }
        let effdir_hash = sha(argument(&args, 2)?, &effdir_bytes, "EFFDIR")?;
        let directory = cell_effdir_framing_v3::parse(&effdir_bytes).map_err(|e| format!("EFFDIR: {e:?}"))?;
        let selected = cell_skybox_inspection_v1::select(&directory).map_err(|e| format!("EFFDIR selection: {e:?}"))?;
        let shader_text = argument(&args, 5)?;
        if shader_text.len() != 8 || !shader_text.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("shader ID must be eight hex digits".into()); }
        let shader_id = u32::from_str_radix(shader_text, 16)?;
        let (width, height) = argument(&args, 8)?.split_once('x').ok_or("viewport must be WIDTHxHEIGHT")?;
        let (width, height) = (width.parse::<u16>()?, height.parse::<u16>()?);
        if width == 0 || height == 0 || width > 4096 || height > 4096 { return Err("viewport must be 1..4096 in each dimension".into()); }
        let (offset_x, offset_y) = (finite(argument(&args, 9)?)?, finite(argument(&args, 10)?)?);
        let values = argument(&args, 11)?.split(',').map(finite).collect::<Result<Vec<_>, _>>()?;
        if values.len() != 4 || values.iter().any(|v| !(0.0..=1.0).contains(v)) { return Err("color must contain four finite components in 0..1".into()); }
        let color = [values[0], values[1], values[2], values[3]];
        let amd = match argument(&args, 12)? { "amd" => true, "rtx" => false, _ => return Err("adapter must be rtx or amd".into()) };
        println!("SCREEN_INSPECTION effdir={} sha256={} export=00000000:05C7A4D0 component={} mode={} flags={:08X} texture={:08X}:{:08X} runtime_selection=unverified", effdir_path.display(), effdir_hash, selected.component_index, selected.mode, selected.flags, selected.texture.group, selected.texture.instance);
        (Path::new(&args[3]), Path::new(&args[6]), Path::new(&args[13]), shader_id, width, height,
         offset_x, offset_y, color, amd, Some(argument(&args, 4)?), Some(argument(&args, 7)?), Some(selected))
    } else {
        let shader_path = Path::new(&args[0]);
        let shader_id = match shader_path.file_name().and_then(|s| s.to_str()) {
            Some(s) if s.eq_ignore_ascii_case("PatchData.package") => 0x7000_0091,
            Some(s) if s.eq_ignore_ascii_case("Spore_Graphics.package") => 0x7000_008c,
            _ => return Err("explicit shader package must be PatchData.package or Spore_Graphics.package".into()),
        };
        (shader_path, Path::new(&args[1]), Path::new(&args[2]), shader_id, 768, 768,
         0.0, 0.0, [1.0; 4], false, None, None, None)
    };
    if output.exists() { return Err("output already exists".into()); }
    fn bounded_package(path: &Path) -> Result<NativePackageIndex, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut bytes = [0u8; dbpf::HEADER_SIZE];
        file.read_exact(&mut bytes)?;
        let header = dbpf::DbpfHeader::parse(&bytes)?;
        if header.index_size > 64 * 1024 * 1024 || header.index_entry_count > 2_000_000 {
            return Err("package index exceeds preflight bounds".into());
        }
        // open() reopens the path; preflight is not atomic against replacement.
        // The caller must provide a trusted file that remains unchanged.
        Ok(NativePackageIndex::open(path)?)
    }
    let shader_package = bounded_package(shader_path)?;
    let texture_package = bounded_package(texture_path)?;
    let pack_key = NativeResourceKey::from_scoped_instance(0x0469_a3f7, 0x4021_2004_0000_0000 | profile);
    let texture_key = match screen {
        Some(selected) => NativeResourceKey { instance_id: selected.texture.instance, type_id: 0x2f4e_681b, group_id: selected.texture.group },
        None => NativeResourceKey { instance_id: 0xbd08_bd8a, type_id: 0x2f4e_681b, group_id: 0 },
    };
    let read = |package: &NativePackageIndex, key| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let entry = package.index().find_unique(key)?.ok_or_else(|| format!("missing exact key {key} in {}", package.path().display()))?;
        if entry.decompressed_size.max(entry.stored_size) > 32 * 1024 * 1024 { return Err("resource exceeds bound".into()); }
        Ok(package.read(entry)?)
    };
    let pack_bytes = read(&shader_package, pack_key)?;
    if let Some(expected) = pack_hash { println!("SCREEN_INSPECTION shader_pack_sha256={}", sha(expected, &pack_bytes, "shader pack")?); }
    let pack = shader_pack::parse(&pack_bytes)?;
    let matches: Vec<_> = pack.standard.iter().filter(|s| s.header.shader_id() == shader_id && s.header.name == "ScreenEffectsTextureShader").collect();
    if matches.len() != 1 { return Err("expected unique named source shader".into()); }
    let variants: Vec<_> = matches[0].variants.iter().filter(|v| v.render_type == 0).collect();
    if variants.len() != 1 { return Err("expected unique render type zero".into()); }
    let v = variants[0];
    if !v.vertex_uniforms.is_empty() || !v.pixel_uniforms.is_empty() { return Err("unexpected shader uniforms".into()); }
    let vs = native_d3d9::tokens(&v.vertex_bytecode, vertex_version)?;
    let ps = native_d3d9::tokens(&v.pixel_bytecode, pixel_version)?;

    let rw_bytes = read(&texture_package, texture_key)?;
    if let Some(expected) = texture_hash { println!("SCREEN_INSPECTION texture_rw4_sha256={}", sha(expected, &rw_bytes, "texture RW4")?); }
    let rw = rw4::parse(&rw_bytes)?;
    // The helper returns a raster; this probe has not established uniqueness
    // of raster sections in the RW4. 681B is an inventoried fallback candidate,
    // not proof that the original 681C request resolves to this exact resource.
    let t = rw.texture(&rw_bytes).ok_or("candidate RW4 has no texture")?;
    if t.width != 256 || t.height != 256 || t.mipmap_levels != 9 || t.format != raster::FORMAT_DXT5 {
        return Err("candidate RW4 differs from inventoried 256x256 nine-level DXT5".into());
    }
    let mut data = Vec::new();
    let mut at = 0usize;
    let mut base = 0u32;
    for level in 0..t.mipmap_levels {
        let w = (t.width >> level).max(1) as usize;
        let h = (t.height >> level).max(1) as usize;
        let size = w.div_ceil(4) * h.div_ceil(4) * 16;
        if level == 0 { base = size as u32; } else { data.extend_from_slice(&(size as u32).to_le_bytes()); }
        let end = at.checked_add(size).ok_or("mip size overflow")?;
        data.extend_from_slice(t.payload.get(at..end).ok_or("truncated RW4 mip")?);
        at = end;
    }
    if at != t.payload.len() { return Err("unconsumed RW4 texture bytes".into()); }
    let raster = raster::Raster { width: t.width, height: t.height, mipmaps: t.mipmap_levels, format: t.format, data_size: base, data };

    // Both legacy and inspection inputs are explicit assumptions, not observed
    // runtime viewport, offsets, component color, or floating-point state.
    let packed_color = cell_screen_quad_v1::pack_color_mxcsr1f80(color)?;
    let quad = cell_screen_quad_v1::successful_quad_mxcsr1f80(width, height, offset_x, offset_y, packed_color)?;
    if quad.iter().any(|v| v.position.iter().chain(v.uv.iter()).any(|x| !x.is_finite())) {
        return Err("inspection inputs produce a non-finite quad".into());
    }
    let mut vertex_bytes = Vec::with_capacity(4 * 24);
    for vertex in quad {
        for x in vertex.position { vertex_bytes.extend_from_slice(&x.to_bits().to_le_bytes()); }
        vertex_bytes.extend_from_slice(&vertex.packed_color.to_le_bytes());
        for x in vertex.uv { vertex_bytes.extend_from_slice(&x.to_bits().to_le_bytes()); }
    }
    // Mode 3 reaches indexed TRIANGLELIST; default index contents remain unknown.
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    println!("SCREEN_PROBE shader_package={} pack_key={} shader={shader_id:08X} name=ScreenEffectsTextureShader profile={profile} render_type=0 texture_package={} texture_key={} source_rw4=true runtime_selection=unverified", shader_path.display(), pack_key, texture_path.display(), texture_key);
    println!("SCREEN_PROBE package_content_authenticity=unverified package_index_preflight_max_bytes=67108864 package_index_preflight_max_entries=2000000 package_reopen_toctou=trusted_stable_input texture_type=681B_fallback_candidate original_request_type=681C rw4_first_raster_uniqueness=unverified provider_precedence=unverified");
    println!("SCREEN_PROBE assumptions viewport={width}x{height} offsets={offset_x:?},{offset_y:?} color={color:?} mxcsr=1F80 declaration=float3,color,float2 indices=0,1,2,0,2,3 culling=none mip_filter=none srgb=off clear=FF18202E runtime_fidelity=false");
    println!("SCREEN_PROBE stored_candidate_states depth=off depth_write=off alpha_test=off alpha_ref=127 alpha_func=8 blend=off sampler_uv=clamp min_mag=linear effective_runtime_state=unverified");
    let target = unsafe {
        if amd { native_d3d9::OffscreenDevice::new_with_amd(width as u32, height as u32, true)? }
        else { native_d3d9::OffscreenDevice::new(width as u32, height as u32)? }
    };
    let device = &target.device;
    let texture = unsafe { target.upload_raster(&raster)? };
    let decl = [
        D3DVERTEXELEMENT9 { Stream: 0, Offset: 0, Type: D3DDECLTYPE_FLOAT3.0 as u8, Method: 0, Usage: D3DDECLUSAGE_POSITION.0 as u8, UsageIndex: 0 },
        D3DVERTEXELEMENT9 { Stream: 0, Offset: 12, Type: D3DDECLTYPE_D3DCOLOR.0 as u8, Method: 0, Usage: D3DDECLUSAGE_COLOR.0 as u8, UsageIndex: 0 },
        D3DVERTEXELEMENT9 { Stream: 0, Offset: 16, Type: D3DDECLTYPE_FLOAT2.0 as u8, Method: 0, Usage: D3DDECLUSAGE_TEXCOORD.0 as u8, UsageIndex: 0 },
        D3DVERTEXELEMENT9 { Stream: 255, Type: D3DDECLTYPE_UNUSED.0 as u8, ..Default::default() },
    ];
    unsafe {
        for (state, value) in [
            (D3DRS_ZENABLE, 0), (D3DRS_ZWRITEENABLE, 0), (D3DRS_ALPHATESTENABLE, 0),
            (D3DRS_ALPHAREF, 127), (D3DRS_ALPHAFUNC, 8), (D3DRS_ALPHABLENDENABLE, 0),
            (D3DRS_CULLMODE, D3DCULL_NONE.0 as u32), (D3DRS_SRGBWRITEENABLE, 0), (D3DRS_COLORWRITEENABLE, 15),
        ] { device.SetRenderState(state, value)?; }
        for (state, value) in [
            (D3DSAMP_ADDRESSU, 3), (D3DSAMP_ADDRESSV, 3), (D3DSAMP_MAGFILTER, 2),
            (D3DSAMP_MINFILTER, 2), (D3DSAMP_MIPFILTER, 0), (D3DSAMP_SRGBTEXTURE, 0),
        ] { device.SetSamplerState(0, state, value)?; }
        device.Clear(0, ptr::null(), D3DCLEAR_TARGET as u32, 0xff18_202e, 1.0, 0)?;
        device.BeginScene()?;
        device.SetTexture(0, &texture)?;
        device.SetVertexDeclaration(&device.CreateVertexDeclaration(decl.as_ptr())?)?;
        device.SetVertexShader(&device.CreateVertexShader(vs.as_ptr())?)?;
        device.SetPixelShader(&device.CreatePixelShader(ps.as_ptr())?)?;
        device.DrawIndexedPrimitiveUP(D3DPT_TRIANGLELIST, 0, 4, 2, indices.as_ptr().cast(), D3DFMT_INDEX16, vertex_bytes.as_ptr().cast(), 24)?;
        device.EndScene()?;
    }
    let rgba = unsafe { target.read_rgba()? };
    let file = OpenOptions::new().write(true).create_new(true).open(output)?;
    image::codecs::png::PngEncoder::new(file).write_image(&rgba, width as u32, height as u32, image::ExtendedColorType::Rgba8)?;
    println!("SCREEN_PROBE output={} presented=false acceptance=exploratory", output.display());
    Ok(())
}

#[cfg(not(all(windows, feature = "native_shader_probe")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Windows and the native_shader_probe feature".into())
}
