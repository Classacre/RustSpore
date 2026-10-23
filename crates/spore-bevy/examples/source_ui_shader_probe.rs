//! Execute the installed UI program in a hidden offscreen D3D9 device.
//! This checks original bytecode/constant/input binding, not a whole SPUI screen.
//! No presentation, visible window, desktop input, or embedded EA asset is used.
#[cfg(windows)]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 1;

// `no_mangle` alone does not put a symbol in an MSVC executable's PE export
// table. The Optimus driver explicitly looks for that export before device
// creation. Keep the linker directive local to this example's object file.
#[cfg(all(windows, target_env = "msvc"))]
#[used]
#[link_section = ".drectve"]
static NVIDIA_EXPORT: [u8; b" /EXPORT:NvOptimusEnablement ".len()] =
    *b" /EXPORT:NvOptimusEnablement ";

#[cfg(windows)]
mod probe {
    use image::ImageEncoder;
    use spore_core::spui_draw::{ImageDrawFlags, UiClipPath, UiMatrix, UiVertex, UiViewport};
    use std::{ffi::CStr, path::Path, ptr};
    use windows::{
        core::w,
        Win32::{
            Foundation::HWND,
            Graphics::Direct3D9::*,
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WS_EX_NOACTIVATE, WS_POPUP},
        },
    };
    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }
    fn tokens(bytes: &[u8], profile: u32) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        if bytes.len() < 8 || bytes.len() % 4 != 0 {
            return Err("invalid source shader token extent".into());
        }
        let words: Vec<_> = bytes
            .chunks_exact(4)
            .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
            .collect();
        if words[0] != profile || words.last() != Some(&0x0000_FFFF) {
            return Err("unexpected source shader profile/end token".into());
        }
        Ok(words)
    }
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if !(3..=5).contains(&args.len()) {
            return Err("usage: source_ui_shader_probe <install> <png:packed-instance-hex|PNG file> <new PNG> [--native-blend] [--scissor]".into());
        }
        let mut native_blend = false;
        let mut clip_path = UiClipPath::Shader;
        for option in &args[3..] {
            match option.as_str() {
                "--native-blend" if !native_blend => native_blend = true,
                "--scissor" if clip_path == UiClipPath::Shader => clip_path = UiClipPath::Scissor,
                _ => return Err(format!("unknown or duplicate UI probe option {option}").into()),
            }
        }
        let output = Path::new(&args[2]);
        if output.exists() {
            return Err("output exists; choose a new PNG".into());
        }
        let package = Path::new(&args[0]).join("Data/PatchData.package");
        let index = dbpf::PackageIndex::open(&package)?;
        let entry = index
            .find_instance(0x0469_A3F7, 0x4021_2004_0000_0003)
            .ok_or("missing source profile-3 shader pack")?;
        let pack = spore_core::shader_pack::parse(&index.read(entry)?)?;
        let shader = pack
            .standard
            .iter()
            .find(|s| {
                s.header.shader_id() == clip_path.shader_id()
                    && s.header.name == clip_path.shader_name()
            })
            .ok_or("missing measured source UI shader")?;
        let variant = shader
            .variants
            .iter()
            .find(|v| v.render_type == 0)
            .ok_or("missing UI render-type0")?;
        let vertex = tokens(&variant.vertex_bytecode, 0xFFFE_0300)?;
        let pixel = tokens(&variant.pixel_bytecode, 0xFFFF_0300)?;
        if variant.vertex_uniforms.len() != 1 || !variant.pixel_uniforms.is_empty() {
            return Err("unmeasured source UI uniform layout".into());
        }
        let uniform = spore_core::shader_binding::UniformDescriptor {
            raw: variant.vertex_uniforms[0],
        };
        if uniform.callback_id() != 0x234
            || uniform.dirty_data_id() != 0x234
            || uniform.register_count() != 96
            || uniform.fourth_word() != 0
        {
            return Err("source UI uniform descriptor changed".into());
        }
        let source = if let Some(id) = args[1].strip_prefix("png:") {
            let instance = u64::from_str_radix(id.trim_start_matches("0x"), 16)?;
            let mut found = None;
            // Explicit base-game overlay priority, matching the chosen base
            // PatchData shader pack. Do not silently substitute another image.
            for name in ["PatchData", "Spore_Graphics", "Spore_Content", "Spore_Game"] {
                let path = Path::new(&args[0])
                    .join("Data")
                    .join(format!("{name}.package"));
                let library = dbpf::PackageIndex::open(&path)?;
                if let Some(entry) = library.find_instance(0x2F7D_0004, instance) {
                    println!(
                        "SOURCE_UI_IMAGE package={} key={}",
                        path.display(),
                        entry.key
                    );
                    found = Some(library.read(entry)?);
                    break;
                }
            }
            let bytes = found.ok_or("no matching source PNG in measured base-game overlay")?;
            let mut reader = image::ImageReader::with_format(
                std::io::Cursor::new(bytes),
                image::ImageFormat::Png,
            );
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(4096);
            limits.max_image_height = Some(4096);
            limits.max_alloc = Some(128 * 1024 * 1024);
            reader.limits(limits);
            reader.decode()?.to_rgba8()
        } else {
            image::open(&args[1])?.to_rgba8()
        };
        let (width, height) = source.dimensions();
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("source image exceeds probe bounds".into());
        }
        // All unsafe operations below are FFI calls or bounded copies from/to
        // surfaces locked by that device. Native resources outlive every lock.
        let rgba = unsafe {
            render(
                &vertex,
                &pixel,
                source.as_raw(),
                width,
                height,
                native_blend,
                clip_path,
            )?
        };
        let mut changed = 0usize;
        let mut max_error = 0u8;
        for (actual, original) in rgba.chunks_exact(4).zip(source.as_raw().chunks_exact(4)) {
            // Independent integer source-over fixture. The measured batch uses
            // one blend factor pair for all four channels. Separate-alpha and
            // sRGB are explicitly disabled here; their retail values are open.
            let expected: [u8; 4] = if native_blend {
                let alpha = u32::from(original[3]);
                std::array::from_fn(|i| {
                    ((u32::from(original[i]) * alpha + [32, 48, 64, 255][i] * (255 - alpha) + 127)
                        / 255) as u8
                })
            } else {
                original.try_into().unwrap()
            };
            let error = actual
                .iter()
                .zip(&expected)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            max_error = max_error.max(error);
            if error > 0 {
                changed += 1;
            }
        }
        if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?;
        image::codecs::png::PngEncoder::new(file).write_image(
            &rgba,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )?;
        println!("SOURCE_UI_SHADER id={:08X} render_type=0 size={width}x{height} changed_pixels={changed} max_channel_error={max_error} native_vertex_stride=24 native_batch_objects=16 native_viewport=true native_blend={native_blend} source_program=true presented=false", clip_path.shader_id());
        if max_error > u8::from(native_blend) {
            return Err("source UI shader fixture agreement failed".into());
        }
        Ok(())
    }
    unsafe fn render(
        vertex: &[u32],
        pixel: &[u32],
        rgba: &[u8],
        width: u32,
        height: u32,
        native_blend: bool,
        clip_path: UiClipPath,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let window = HiddenWindow(CreateWindowExW(
            WS_EX_NOACTIVATE,
            w!("STATIC"),
            w!("RustSpore offscreen shader probe"),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(GetModuleHandleW(None)?.into()),
            None,
        )?);
        let d3d = Direct3DCreate9(D3D_SDK_VERSION).ok_or("Direct3DCreate9 unavailable")?;
        let mut chosen = None;
        for adapter in 0..d3d.GetAdapterCount() {
            let mut id = D3DADAPTER_IDENTIFIER9::default();
            d3d.GetAdapterIdentifier(adapter, 0, &mut id)?;
            let description = CStr::from_ptr(id.Description.as_ptr()).to_string_lossy();
            println!(
                "SOURCE_UI_ADAPTER index={adapter} vendor={:04X} description={description:?}",
                id.VendorId
            );
            if description.contains("RTX 3060") {
                chosen = Some(adapter);
            }
        }
        let adapter = chosen.ok_or(
            "RTX 3060 is not exposed through D3D9; no software or alternate adapter substituted",
        )?;
        let mut present = D3DPRESENT_PARAMETERS {
            BackBufferWidth: 1,
            BackBufferHeight: 1,
            BackBufferFormat: D3DFMT_X8R8G8B8,
            BackBufferCount: 1,
            SwapEffect: D3DSWAPEFFECT_DISCARD,
            hDeviceWindow: window.0,
            Windowed: true.into(),
            PresentationInterval: D3DPRESENT_INTERVAL_IMMEDIATE as u32,
            ..Default::default()
        };
        let mut device = None;
        d3d.CreateDevice(
            adapter,
            D3DDEVTYPE_HAL,
            window.0,
            (D3DCREATE_HARDWARE_VERTEXPROCESSING
                | D3DCREATE_NOWINDOWCHANGES
                | D3DCREATE_FPU_PRESERVE) as u32,
            &mut present,
            &mut device,
        )?;
        let device = device.ok_or("D3D9 did not return a device")?;
        let mut texture = None;
        device.CreateTexture(
            width,
            height,
            1,
            0,
            D3DFMT_A8R8G8B8,
            D3DPOOL_MANAGED,
            &mut texture,
            ptr::null_mut(),
        )?;
        let texture = texture.ok_or("missing source texture")?;
        let mut lock = D3DLOCKED_RECT::default();
        texture.LockRect(0, &mut lock, ptr::null(), 0)?;
        if lock.Pitch < width as i32 * 4 || lock.pBits.is_null() {
            let _ = texture.UnlockRect(0);
            return Err("invalid texture lock".into());
        }
        for y in 0..height as usize {
            let row = std::slice::from_raw_parts_mut(
                (lock.pBits as *mut u8).add(y * lock.Pitch as usize),
                width as usize * 4,
            );
            for (dst, src) in row
                .chunks_exact_mut(4)
                .zip(rgba[y * width as usize * 4..(y + 1) * width as usize * 4].chunks_exact(4))
            {
                dst.copy_from_slice(&[src[2], src[1], src[0], src[3]]);
            }
        }
        texture.UnlockRect(0)?;
        let mut target = None;
        device.CreateRenderTarget(
            width,
            height,
            D3DFMT_A8R8G8B8,
            D3DMULTISAMPLE_NONE,
            0,
            false,
            &mut target,
            ptr::null_mut(),
        )?;
        let target = target.ok_or("missing offscreen render target")?;
        device.SetDepthStencilSurface(None)?;
        device.SetRenderTarget(0, &target)?;
        device.SetViewport(&D3DVIEWPORT9 {
            X: 0,
            Y: 0,
            Width: width,
            Height: height,
            MinZ: 0.,
            MaxZ: 1.,
        })?;
        for (state, value) in [
            (D3DRS_ZENABLE, 0),
            (D3DRS_ZWRITEENABLE, 0),
            (D3DRS_LIGHTING, 0),
            (D3DRS_ALPHABLENDENABLE, 0),
            (D3DRS_ALPHATESTENABLE, 0),
            (D3DRS_CULLMODE, D3DCULL_NONE.0 as u32),
            (D3DRS_SRGBWRITEENABLE, 0),
            (D3DRS_COLORWRITEENABLE, 15),
            (D3DRS_SEPARATEALPHABLENDENABLE, 0),
            (D3DRS_BLENDOP, D3DBLENDOP_ADD.0 as u32),
        ] {
            device.SetRenderState(state, value)?;
        }
        for (state, value) in [
            (D3DSAMP_MINFILTER, D3DTEXF_POINT.0 as u32),
            (D3DSAMP_MAGFILTER, D3DTEXF_POINT.0 as u32),
            (D3DSAMP_MIPFILTER, D3DTEXF_NONE.0 as u32),
            (D3DSAMP_ADDRESSU, D3DTADDRESS_CLAMP.0 as u32),
            (D3DSAMP_ADDRESSV, D3DTADDRESS_CLAMP.0 as u32),
            (D3DSAMP_SRGBTEXTURE, 0),
        ] {
            device.SetSamplerState(0, state, value)?;
        }
        if native_blend {
            let state = ImageDrawFlags::SERIALIZED_RESOURCE.draw_state();
            for render in state.render_states {
                device.SetRenderState(D3DRENDERSTATETYPE(render.id as i32), render.value)?;
            }
            for sampler in state.samplers {
                for setting in sampler.states() {
                    device.SetSamplerState(
                        sampler.unit,
                        D3DSAMPLERSTATETYPE(setting.id as i32),
                        setting.value,
                    )?;
                }
            }
        }
        device.SetRenderState(
            D3DRS_SCISSORTESTENABLE,
            u32::from(clip_path.scissor_enabled()),
        )?;
        if clip_path.scissor_enabled() {
            device.SetScissorRect(&windows::Win32::Foundation::RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            })?;
        }
        device.SetTexture(0, &texture)?;
        let vertex_shader = device.CreateVertexShader(vertex.as_ptr())?;
        let pixel_shader = device.CreatePixelShader(pixel.as_ptr())?;
        device.SetVertexShader(&vertex_shader)?;
        device.SetPixelShader(&pixel_shader)?;
        let element = |offset, typ, usage, index| D3DVERTEXELEMENT9 {
            Stream: 0,
            Offset: offset,
            Type: typ,
            Method: D3DDECLMETHOD_DEFAULT.0 as u8,
            Usage: usage,
            UsageIndex: index,
        };
        let declaration = [
            element(
                0,
                D3DDECLTYPE_FLOAT2.0 as u8,
                D3DDECLUSAGE_POSITION.0 as u8,
                0,
            ),
            element(
                8,
                D3DDECLTYPE_D3DCOLOR.0 as u8,
                D3DDECLUSAGE_COLOR.0 as u8,
                0,
            ),
            element(
                12,
                D3DDECLTYPE_D3DCOLOR.0 as u8,
                D3DDECLUSAGE_COLOR.0 as u8,
                1,
            ),
            element(
                16,
                D3DDECLTYPE_FLOAT2.0 as u8,
                D3DDECLUSAGE_TEXCOORD.0 as u8,
                0,
            ),
            D3DVERTEXELEMENT9 {
                Stream: 255,
                Type: D3DDECLTYPE_UNUSED.0 as u8,
                ..Default::default()
            },
        ];
        let declaration = device.CreateVertexDeclaration(declaration.as_ptr())?;
        device.SetVertexDeclaration(&declaration)?;
        // Native viewport and object composition, 16 six-register objects.
        // Each object draws one atlas strip from local pixel coordinates;
        // different local translations exercise all relative-register slots.
        let viewport = UiViewport::new([0., 0., width as f32, height as f32], true)?;
        let mut constants = [0f32; 96 * 4];
        let mut vertices = Vec::new();
        let mut indices = Vec::<u16>::new();
        for object in 0..16u8 {
            let u0 = f32::from(object) / 16.;
            let u1 = f32::from(object + 1) / 16.;
            let x0 = u0 * width as f32;
            let x1 = u1 * width as f32;
            let local = UiMatrix {
                rows: [
                    [1., 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [x0, 0., 0., 1.],
                ],
                kind: 1,
            };
            let registers = viewport.object_registers(
                0xffff_ffff,
                local,
                [x0, 0., x1, height as f32],
                clip_path,
            )?;
            for (row, values) in registers.iter().enumerate() {
                let start = usize::from(object) * 24 + row * 4;
                constants[start..start + 4].copy_from_slice(values);
            }
            let base = vertices.len() as u16;
            for (xy, uv) in [
                ([0., 0.], [u0, 0.]),
                ([x1 - x0, 0.], [u1, 0.]),
                ([0., height as f32], [u0, 1.]),
                ([x1 - x0, height as f32], [u1, 1.]),
            ] {
                vertices.push(UiVertex::new(xy, 0xffff_ffff, uv, object, true)?);
            }
            indices.extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
        }
        assert_eq!(std::mem::size_of::<UiVertex>(), 24);
        device.SetVertexShaderConstantF(0, constants.as_ptr(), 96)?;
        device.Clear(
            0,
            ptr::null(),
            D3DCLEAR_TARGET as u32,
            if native_blend { 0xFF203040 } else { 0xFFFF00FF },
            1.,
            0,
        )?;
        device.BeginScene()?;
        device.DrawIndexedPrimitiveUP(
            D3DPT_TRIANGLELIST,
            0,
            vertices.len() as u32,
            (indices.len() / 3) as u32,
            indices.as_ptr().cast(),
            D3DFMT_INDEX16,
            vertices.as_ptr().cast(),
            24,
        )?;
        device.EndScene()?;
        let mut staging = None;
        device.CreateOffscreenPlainSurface(
            width,
            height,
            D3DFMT_A8R8G8B8,
            D3DPOOL_SYSTEMMEM,
            &mut staging,
            ptr::null_mut(),
        )?;
        let staging = staging.ok_or("missing readback surface")?;
        device.GetRenderTargetData(&target, &staging)?;
        staging.LockRect(&mut lock, ptr::null(), D3DLOCK_READONLY as u32)?;
        if lock.Pitch < width as i32 * 4 || lock.pBits.is_null() {
            let _ = staging.UnlockRect();
            return Err("invalid readback lock".into());
        }
        let mut result = Vec::with_capacity(rgba.len());
        for y in 0..height as usize {
            let row = std::slice::from_raw_parts(
                (lock.pBits as *const u8).add(y * lock.Pitch as usize),
                width as usize * 4,
            );
            for src in row.chunks_exact(4) {
                result.extend_from_slice(&[src[2], src[1], src[0], src[3]]);
            }
        }
        staging.UnlockRect()?;
        Ok(result)
    }
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    probe::run()
}
#[cfg(not(windows))]
fn main() {
    eprintln!("This source D3D9 execution probe requires Windows.");
    std::process::exit(2);
}
