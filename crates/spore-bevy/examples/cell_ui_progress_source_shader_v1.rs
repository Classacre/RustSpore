//! Explicit retained source-segment original-shader offscreen fixture.
//! Not active-provider selection, a runtime HUD, or retail parity.
#[cfg(windows)]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 1;
#[cfg(all(windows, target_env = "msvc"))]
#[used]
#[link_section = ".drectve"]
static NVIDIA_EXPORT: [u8; b" /EXPORT:NvOptimusEnablement ".len()] = *b" /EXPORT:NvOptimusEnablement ";
#[cfg(windows)]
#[path = "../src/native_source_ui_renderer_v1.rs"]
mod native_source_ui_renderer_v1;
#[cfg(windows)]
mod probe {
    use image::ImageEncoder;
    use sha2::{Digest, Sha256};
    use spore_core::{spui, spui_layout::ImageRegion, spui_draw::UiClipPath};
    use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
    use std::{ffi::CStr, path::Path, ptr};
    use windows::{core::w, Win32::{Foundation::HWND, Graphics::Direct3D9::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{CreateWindowExW,DestroyWindow,WS_EX_NOACTIVATE,WS_POPUP}}};
    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow { fn drop(&mut self) { unsafe { let _=DestroyWindow(self.0); } } }
    fn tokens(bytes:&[u8], profile:u32)->Result<Vec<u32>,Box<dyn std::error::Error>> {
        if bytes.len()<8 || bytes.len()%4!=0 { return Err("bad shader extent".into()); }
        let words:Vec<_>=bytes.chunks_exact(4).map(|w|u32::from_le_bytes(w.try_into().unwrap())).collect();
        if words[0]!=profile || words.last()!=Some(&0xffff) { return Err("bad shader profile".into()); } Ok(words)
    }
    fn resource(p:&NativePackageIndex,k:NativeResourceKey)->Result<Vec<u8>,Box<dyn std::error::Error>> { Ok(p.read(p.index().find_unique(k)?.ok_or("explicit resource absent")?)?) }
    pub fn run()->Result<(),Box<dyn std::error::Error>> {
        let a:Vec<_>=std::env::args().skip(1).collect();
        if a.len()!=9 { return Err("usage: <install> <layout.package> <atlas.package> <fixture.json> <width> <height> <shader|scissor> <half-pixel-0|1> <new.png>".into()); }
        let target=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("target").canonicalize()?;
        let out=Path::new(&a[8]);
        if out.exists() || !out.parent().ok_or("output parent")?.canonicalize()?.starts_with(target) { return Err("output must be a new file under target".into()); }
        let width:u32=a[4].parse()?; let height:u32=a[5].parse()?;
        if !(1..=4096).contains(&width) || !(1..=4096).contains(&height) { return Err("viewport bounds".into()); }
        let clip=match a[6].as_str() { "shader"=>UiClipPath::Shader, "scissor"=>UiClipPath::Scissor, _=>return Err("clip path".into()) };
        let half=match a[7].as_str() { "0"=>false, "1"=>true, _=>return Err("half-pixel value".into()) };
        let layout=NativePackageIndex::open(Path::new(&a[1]))?;
        let atlas=NativePackageIndex::open(Path::new(&a[2]))?;
        let lk=NativeResourceKey {instance_id:0x26b068d2,type_id:0x250fe9a2,group_id:0x40464200};
        let ak=NativeResourceKey {instance_id:0xb8b50133,type_id:0x2f7d0004,group_id:0x31a44893};
        let lb=resource(&layout,lk)?; let ab=resource(&atlas,ak)?;
        let lh=format!("{:x}",Sha256::digest(&lb)); let ah=format!("{:x}",Sha256::digest(&ab));
        let report_bytes=std::fs::read(&a[3])?;
        let fixture_hash=format!("{:x}",Sha256::digest(&report_bytes));
        let report:serde_json::Value=serde_json::from_slice(&report_bytes)?;
        if report["layout_sha256"].as_str()!=Some(lh.as_str()) || report["atlas_sha256"].as_str()!=Some(ah.as_str()) { return Err("fixture and explicit source hashes differ".into()); }
        if !["5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda","6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc"].contains(&lh.as_str()) || !["44221369585ea3732a378390726a205d73f9e86781b84a06f6ada8d66319e153","a5de353327759af85b9844b3ab626bb5fd25df90d78f54bd462f314e4d9b5514"].contains(&ah.as_str()) { return Err("unaudited payload".into()); }
        let file=spui::parse(&lb)?; let body=spui::parse_elements(&file)?;
        let image_index=u16::try_from(report["image_index"].as_u64().ok_or("image index")?)?;
        let matches:Vec<_>=body.elements.iter().filter(|x|x.index==image_index).collect(); if matches.len()!=1 { return Err("ambiguous image index".into()); }
        let region=ImageRegion::from_element(matches[0])?;
        if region.flags.0!=0 { return Err("unsupported source image flags".into()); }
        let resources:Vec<_>=file.image_resources.iter().chain(&file.atlas_resources).collect();
        let key=resources.get(usize::from(region.resource_reference)).ok_or("bad image key")?;
        if key.instance!=ak.instance_id || key.type_id!=ak.type_id || key.group!=ak.group_id { return Err("atlas key mismatch".into()); }
        let get_rect=|value:&serde_json::Value|->Result<[f32;4],Box<dyn std::error::Error>> {
            let arr=value.as_array().ok_or("rectangle array")?; if arr.len()!=4 { return Err("rectangle length".into()); }
            let mut result=[0f32;4]; for i in 0..4 { result[i]=arr[i].as_f64().ok_or("rectangle number")? as f32; }
            if result.iter().any(|x|!x.is_finite()) { return Err("nonfinite rectangle".into()); } Ok(result)
        };
        let calls=report["calls"].as_array().ok_or("fixture calls")?;
        if calls.is_empty() || calls.len()>9 || report["call_count"].as_u64()!=Some(calls.len() as u64) { return Err("unsupported call count".into()); }
        let mut rectangles=Vec::new();
        for call in calls {
            let d=get_rect(&call["destination"])?; let u=region.map_uv(get_rect(&call["relative_uv"])?)?;
            if d[0]>d[2] || d[1]>d[3] || u.iter().any(|x|*x<0. || *x>1.) { return Err("unsupported rectangle".into()); }
            rectangles.push((d,u));
        }
        let source=image::load_from_memory_with_format(&ab,image::ImageFormat::Png)?.to_rgba8();
        if source.dimensions()!=(512,512) { return Err("atlas dimensions".into()); }
        let pack_path=Path::new(&a[0]).join("Data/PatchData.package");
        let index=dbpf::PackageIndex::open(&pack_path)?;
        let entry=index.find_instance(0x0469a3f7,0x4021200400000003).ok_or("shader pack missing")?;
        let pack_data=index.read(entry)?;
        if format!("{:x}",Sha256::digest(&pack_data)) != "ab5cc3cf86340ec1f7489cda7f801e8bc2d969e6e8a3babd8b1f34734a506409" { return Err("unverified shader pack digest".into()); }
        let pack=spore_core::shader_pack::parse(&pack_data)?;
        let shader=pack.standard.iter().find(|s|s.header.shader_id()==clip.shader_id() && s.header.name==clip.shader_name()).ok_or("source shader missing")?;
        let variant=shader.variants.iter().find(|v|v.render_type==0).ok_or("render type missing")?;
        let vert=tokens(&variant.vertex_bytecode,0xfffe0300)?; let pix=tokens(&variant.pixel_bytecode,0xffff0300)?;
        if variant.vertex_uniforms.len()!=1 || !variant.pixel_uniforms.is_empty() { return Err("uniform layout".into()); }
        let uniform=spore_core::shader_binding::UniformDescriptor { raw:variant.vertex_uniforms[0] };
        if uniform.callback_id()!=0x234 || uniform.dirty_data_id()!=0x234 || uniform.register_count()!=96 || uniform.fourth_word()!=0 { return Err("uniform descriptor".into()); }
        let rgba=unsafe { render(&vert,&pix,source.as_raw(),512,512,width,height,clip,half,&rectangles)? };
        let changed=rgba.chunks_exact(4).filter(|p|p[3]!=0).count(); if changed==0 { return Err("empty readback".into()); }
        let output=std::fs::OpenOptions::new().write(true).create_new(true).open(out)?;
        image::codecs::png::PngEncoder::new(output).write_image(&rgba,width,height,image::ExtendedColorType::Rgba8)?;
        let png_hash=format!("{:x}",Sha256::digest(std::fs::read(out)?));
        println!("SOURCE_SEGMENT_SHADER calls={} nonzero_alpha={} atlas={} layout={} fixture={} image_index={} png={} shader_pack={} vertex={} pixel={} clip={} half_pixel={} target={}x{} atlas_size=512x512 blend=srcalpha/invsrcalpha linear_clamp=true transparent_clear=true present=false runtime_state=false",rectangles.len(),changed,ah,lh,fixture_hash,image_index,png_hash,format!("{:x}",Sha256::digest(&pack_data)),format!("{:x}",Sha256::digest(&variant.vertex_bytecode)),format!("{:x}",Sha256::digest(&variant.pixel_bytecode)),a[6],half,width,height);
        Ok(())
    }
    unsafe fn render(
        vertex: &[u32],
        pixel: &[u32],
        rgba: &[u8],
        texture_width: u32,
        texture_height: u32,
        width: u32,
        height: u32,
        clip_path: UiClipPath,
        half_pixel: bool,
        rectangles: &[([f32; 4], [f32; 4])],
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
        let mut target = None;
        device.CreateRenderTarget(width, height, D3DFMT_A8R8G8B8, D3DMULTISAMPLE_NONE, 0, false, &mut target, ptr::null_mut())?;
        let target = target.ok_or("missing offscreen render target")?;
        device.SetDepthStencilSurface(None)?;
        device.SetRenderTarget(0, &target)?;
        device.Clear(0, ptr::null(), D3DCLEAR_TARGET as u32, 0x00000000, 1., 0)?;
        device.BeginScene()?;
        // Seed inherited state to exercise the reusable draw's reset contract.
        device.SetRenderState(D3DRS_FILLMODE, D3DFILL_WIREFRAME.0 as u32)?;
        device.SetRenderState(D3DRS_CLIPPING, 0)?;
        device.SetRenderState(D3DRS_CLIPPLANEENABLE, 1)?;
        device.SetRenderState(D3DRS_WRAP0, 3)?;
        super::native_source_ui_renderer_v1::draw(&device,vertex,pixel,rgba,texture_width,texture_height,width,height,clip_path,half_pixel,rectangles)?;
        device.EndScene()?;
        let mut lock = D3DLOCKED_RECT::default();
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
fn main()->Result<(),Box<dyn std::error::Error>> { probe::run() }
#[cfg(not(windows))]
fn main() { eprintln!("Windows D3D9 required"); }
