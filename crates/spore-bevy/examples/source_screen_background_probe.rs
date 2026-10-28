//! Diagnostic offscreen composition of indexed original source assets.
//! Usage: source_screen_background_probe <PatchData.package|Spore_Graphics.package>
//!        <texture.package> <new.png>
//! All paths and profile 3 are explicit. This is not a retail screenshot.
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
    use image::ImageEncoder;
    use spore_core::{cell_screen_quad_v1, raster, rw4, shader_pack};
    use std::{fs::{File, OpenOptions}, io::Read, path::Path, ptr};
    use windows::Win32::Graphics::Direct3D9::*;

    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: source_screen_background_probe <PatchData.package|Spore_Graphics.package> <texture.package> <new.png>".into());
    }
    let shader_path = Path::new(&args[0]);
    let texture_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    if output.exists() { return Err("output already exists".into()); }
    let shader_id = match shader_path.file_name().and_then(|s| s.to_str()) {
        Some(s) if s.eq_ignore_ascii_case("PatchData.package") => 0x7000_0091,
        Some(s) if s.eq_ignore_ascii_case("Spore_Graphics.package") => 0x7000_008c,
        _ => return Err("explicit shader package must be PatchData.package or Spore_Graphics.package".into()),
    };
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
    let pack_key = NativeResourceKey::from_scoped_instance(0x0469_a3f7, 0x4021_2004_0000_0003);
    let texture_key = NativeResourceKey { instance_id: 0xbd08_bd8a, type_id: 0x2f4e_681b, group_id: 0 };
    let read = |package: &NativePackageIndex, key| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let entry = package.index().find_unique(key)?.ok_or_else(|| format!("missing exact key {key} in {}", package.path().display()))?;
        if entry.decompressed_size.max(entry.stored_size) > 32 * 1024 * 1024 { return Err("resource exceeds bound".into()); }
        Ok(package.read(entry)?)
    };
    let pack = shader_pack::parse(&read(&shader_package, pack_key)?)?;
    let matches: Vec<_> = pack.standard.iter().filter(|s| s.header.shader_id() == shader_id && s.header.name == "ScreenEffectsTextureShader").collect();
    if matches.len() != 1 { return Err("expected unique named source shader".into()); }
    let variants: Vec<_> = matches[0].variants.iter().filter(|v| v.render_type == 0).collect();
    if variants.len() != 1 { return Err("expected unique render type zero".into()); }
    let v = variants[0];
    if !v.vertex_uniforms.is_empty() || !v.pixel_uniforms.is_empty() { return Err("unexpected shader uniforms".into()); }
    let vs = native_d3d9::tokens(&v.vertex_bytecode, 0xfffe_0300)?;
    let ps = native_d3d9::tokens(&v.pixel_bytecode, 0xffff_0300)?;

    let rw_bytes = read(&texture_package, texture_key)?;
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

    // Explicit inspection inputs: 768 square viewport, zero unverified globals,
    // white unverified component tuple, fixture MXCSR 1F80.
    let (width, height) = (768u16, 768u16);
    let color = cell_screen_quad_v1::pack_color_mxcsr1f80([1.0; 4])?;
    let quad = cell_screen_quad_v1::successful_quad_mxcsr1f80(width, height, 0.0, 0.0, color)?;
    let mut vertex_bytes = Vec::with_capacity(4 * 24);
    for vertex in quad {
        for x in vertex.position { vertex_bytes.extend_from_slice(&x.to_bits().to_le_bytes()); }
        vertex_bytes.extend_from_slice(&vertex.packed_color.to_le_bytes());
        for x in vertex.uv { vertex_bytes.extend_from_slice(&x.to_bits().to_le_bytes()); }
    }
    // Mode 3 reaches indexed TRIANGLELIST; default index contents remain unknown.
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    println!("SCREEN_PROBE shader_package={} pack_key={} shader={shader_id:08X} name=ScreenEffectsTextureShader profile=3 render_type=0 texture_package={} texture_key={} source_rw4=true runtime_selection=unverified", shader_path.display(), pack_key, texture_path.display(), texture_key);
    println!("SCREEN_PROBE package_content_authenticity=unverified package_index_preflight_max_bytes=67108864 package_index_preflight_max_entries=2000000 package_reopen_toctou=trusted_stable_input texture_type=681B_fallback_candidate original_request_type=681C rw4_first_raster_uniqueness=unverified provider_precedence=unverified");
    println!("SCREEN_PROBE assumptions viewport=768x768 offsets=0,0 color=white mxcsr=1F80 declaration=float3,color,float2 indices=0,1,2,0,2,3 culling=none mip_filter=none srgb=off clear=FF18202E runtime_fidelity=false");
    println!("SCREEN_PROBE stored_candidate_states depth=off depth_write=off alpha_test=off alpha_ref=127 alpha_func=8 blend=off sampler_uv=clamp min_mag=linear effective_runtime_state=unverified");
    let target = unsafe { native_d3d9::OffscreenDevice::new(width as u32, height as u32)? };
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
