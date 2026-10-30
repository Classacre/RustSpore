//! Synthetic D3D9 stencil/texkill fixture. No game assets or gameplay state.
//! Run only through a hash-pinning harness after review.
#[cfg(all(windows, feature = "native_renderer"))]
#[path = "../src/native_d3d9.rs"] mod native_d3d9;
#[cfg(all(windows, feature = "native_renderer"))]
#[path = "../src/native_d3dx.rs"] mod native_d3dx;
#[cfg(all(windows, feature = "native_renderer"))]
#[no_mangle] pub static NvOptimusEnablement: u32 = 1;
#[cfg(all(windows, target_env = "msvc", feature = "native_renderer"))]
#[used]
#[link_section = ".drectve"]
static NVIDIA_EXPORT: [u8; b" /EXPORT:NvOptimusEnablement ".len()] = *b" /EXPORT:NvOptimusEnablement ";

#[cfg(all(windows, feature = "native_renderer"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use native_d3d9::{OffscreenDevice,tokens};
    use std::ptr;
    use windows::Win32::Graphics::Direct3D9::*;
    const N: u32 = 64;
    const SHADER: &str = "float4 main(float2 p : VPOS) : COLOR0 { clip(p.x - 32.0); return float4(p.x < 40.0 ? 0.0 : 1.0, 0, 0, p.y < 32.0 ? 0.25 : 1.0); }";
    #[repr(C)] #[derive(Clone,Copy)] struct Vertex { x:f32,y:f32,z:f32,rhw:f32,color:u32 }
    fn quad(l:f32,t:f32,r:f32,b:f32,z:f32,color:u32)->[Vertex;4] {
        [Vertex{x:l,y:t,z,rhw:1.,color},Vertex{x:r,y:t,z,rhw:1.,color},
         Vertex{x:l,y:b,z,rhw:1.,color},Vertex{x:r,y:b,z,rhw:1.,color}]
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    let preflight = match args.as_slice() {
        [] => false,
        [flag] if flag == "--preflight" => true,
        _ => return Err("usage: d3d9_stencil_texkill_fixture_v1 [--preflight]".into()),
    };
    let compiler = native_d3dx::Compiler::system()?;
    let mut module_path = [0u16; 32768];
    let module_len = unsafe { windows::Win32::System::LibraryLoader::GetModuleFileNameW(Some(compiler.module_handle()), &mut module_path) } as usize;
    if module_len == 0 || module_len >= module_path.len() { return Err("compiler module path unavailable".into()); }
    println!("COMPILER_PATH={}", String::from_utf16(&module_path[..module_len])?);
    let compiled = compiler.compile(SHADER,"main","ps_3_0",0x12)?;
    if compiled.bytecode.len() > 16 * 1024 { return Err("compiled shader exceeds fixture cap".into()); }
    let ps = tokens(&compiled.bytecode,0xffff0300)?;
    // Walk instruction lengths; fail closed unless actual shader instruction is TEXKILL.
    let mut i = 1usize;
    let mut texkill = false;
    while i < ps.len() {
        let word=ps[i]; let op=word & 0xffff;
        if op==0xffff { break; }
        if op==0xfffe { i=i.checked_add(1+((word>>16)&0x7fff) as usize).ok_or("shader extent")?; continue; }
        if op==65 { texkill=true; }
        let len=((word>>24)&15) as usize;
        if len==0 { return Err("unrecognized shader instruction length".into()); }
        i=i.checked_add(1+len).ok_or("shader extent")?;
    }
    if !texkill || i>=ps.len() || ps[i]&0xffff!=0xffff { return Err("compiled shader lacks verified TEXKILL or END".into()); }
    if preflight {
        println!("PREFLIGHT_OK shader_hex={} gpu=false", compiled.bytecode.iter().map(|b|format!("{b:02x}")).collect::<String>());
        return Ok(());
    }
    let target=unsafe { OffscreenDevice::new(N,N)? }; let d=&target.device;
    let mut caps=D3DCAPS9::default(); unsafe { d.GetDeviceCaps(&mut caps)?; }
    if caps.StencilCaps & (D3DSTENCILCAPS_KEEP | D3DSTENCILCAPS_REPLACE) as u32 != (D3DSTENCILCAPS_KEEP | D3DSTENCILCAPS_REPLACE) as u32
        || caps.ZCmpCaps & D3DPCMPCAPS_LESSEQUAL as u32 == 0
        || caps.AlphaCmpCaps & D3DPCMPCAPS_GREATER as u32 == 0 { return Err("required stencil/depth/alpha capabilities unavailable".into()); }
    let shader=unsafe { d.CreatePixelShader(ps.as_ptr())? };
    let draw=|vertices:&[Vertex;4]| unsafe { d.DrawPrimitiveUP(D3DPT_TRIANGLESTRIP,2,vertices.as_ptr().cast(),std::mem::size_of::<Vertex>() as u32) };
    let full=quad(-0.5,-0.5,63.5,63.5,0.5,0xffff0000);
    let occluder=quad(47.5,-0.5,63.5,63.5,0.2,0xff0000ff);
    let maskquad=quad(-0.5,-0.5,63.5,63.5,0.0,0xffffffff);
    let mut images=Vec::new();
    for instrumented in [false,true] {
        unsafe {
            for (state,value) in [(D3DRS_ZENABLE,1),(D3DRS_ZWRITEENABLE,1),(D3DRS_ZFUNC,D3DCMP_LESSEQUAL.0 as u32),
                (D3DRS_CULLMODE,D3DCULL_NONE.0 as u32),(D3DRS_ALPHABLENDENABLE,0),(D3DRS_ALPHATESTENABLE,0),
                (D3DRS_LIGHTING,0),(D3DRS_SRGBWRITEENABLE,0),(D3DRS_COLORWRITEENABLE,15),(D3DRS_STENCILENABLE,0)] {d.SetRenderState(state,value)?;}
            d.Clear(0,ptr::null(),(D3DCLEAR_TARGET|D3DCLEAR_ZBUFFER|D3DCLEAR_STENCIL) as u32,0xff000000,1.,0)?;
            d.BeginScene()?;
            d.SetVertexShader(None::<&IDirect3DVertexShader9>)?; d.SetPixelShader(None::<&IDirect3DPixelShader9>)?;
            d.SetFVF(D3DFVF_XYZRHW|D3DFVF_DIFFUSE)?;
            d.SetTextureStageState(0,D3DTSS_COLOROP,D3DTOP_SELECTARG1.0 as u32)?;
            d.SetTextureStageState(0,D3DTSS_COLORARG1,D3DTA_DIFFUSE)?;
            d.SetTextureStageState(0,D3DTSS_ALPHAOP,D3DTOP_SELECTARG1.0 as u32)?;
            d.SetTextureStageState(0,D3DTSS_ALPHAARG1,D3DTA_DIFFUSE)?;
            draw(&occluder)?;
            if instrumented {
                for (state,value) in [(D3DRS_STENCILENABLE,1),(D3DRS_TWOSIDEDSTENCILMODE,0),
                    (D3DRS_STENCILFUNC,D3DCMP_ALWAYS.0 as u32),(D3DRS_STENCILREF,1),(D3DRS_STENCILMASK,0xff),
                    (D3DRS_STENCILWRITEMASK,0xff),(D3DRS_STENCILFAIL,D3DSTENCILOP_KEEP.0 as u32),
                    (D3DRS_STENCILZFAIL,D3DSTENCILOP_KEEP.0 as u32),(D3DRS_STENCILPASS,D3DSTENCILOP_REPLACE.0 as u32)] { d.SetRenderState(state,value)?; }
            }
            d.SetRenderState(D3DRS_ALPHATESTENABLE,1)?;
            d.SetRenderState(D3DRS_ALPHAFUNC,D3DCMP_GREATER.0 as u32)?;
            d.SetRenderState(D3DRS_ALPHAREF,127)?;
            d.SetPixelShader(&shader)?; draw(&full)?; d.EndScene()?;
            images.push(target.read_rgba()?);
        }
    }
    if images[0]!=images[1] { return Err("instrumentation changed color".into()); }
    unsafe {
        d.Clear(0,ptr::null(),D3DCLEAR_TARGET as u32,0xff000000,1.,0)?;
        d.BeginScene()?;
        for (state,value) in [(D3DRS_ZENABLE,0),(D3DRS_ZWRITEENABLE,0),(D3DRS_ALPHABLENDENABLE,0),(D3DRS_ALPHATESTENABLE,0),
            (D3DRS_STENCILFUNC,D3DCMP_EQUAL.0 as u32),(D3DRS_STENCILREF,1),(D3DRS_STENCILWRITEMASK,0),
            (D3DRS_STENCILPASS,D3DSTENCILOP_KEEP.0 as u32)] { d.SetRenderState(state,value)?; }
        d.SetPixelShader(None::<&IDirect3DPixelShader9>)?; draw(&maskquad)?; d.EndScene()?;
    }
    let mask=unsafe {target.read_rgba()?};
    if mask.len()!=images[0].len() { return Err("readback extent mismatch".into()); }
    let mut count=0usize;
    for (index,maskpixel) in mask.chunks_exact(4).enumerate() {
        let covered=match maskpixel { [255,255,255,255]=>true,[0,0,0,255]=>false,_=>return Err("mask is not binary".into()) };
        let x=index % N as usize; let y=index / N as usize;
        // Analytic geometry, clip, depth and alpha predicates; no RGB-derived mask.
        let expected=(32..48).contains(&x) && y>=32;
        if covered!=expected {return Err("mask/analytic coverage mismatch".into());}
        count+=usize::from(covered);
    }
    // Accepted black geometry matches the opaque black clear yet must be covered.
    let pixel = |x:usize,y:usize| (y*N as usize+x)*4;
    let black=pixel(36,48); let clear=pixel(16,48); let red=pixel(44,48);
    if images[0][black..black+4]!=images[0][clear..clear+4] || mask[black]!=255 || mask[clear]!=0
        || images[0][red..red+4]!=[255,0,0,255] { return Err("black/red coverage check failed".into()); }
    // Accepted, clipped, alpha-rejected and depth-rejected interiors.
    for (x,y,want) in [(16usize,48usize,false),(36,48,true),(44,48,true),(40,16,false),(56,48,false)] {
        let p=pixel(x,y); if (mask[p]==255)!=want {return Err("clip/depth/alpha interior check failed".into());}
    }
    println!("FIXTURE_OK width={N} height={N} coverage_pixels={count} shader_hex={} semantics=synthetic_only",compiled.bytecode.iter().map(|b|format!("{b:02x}")).collect::<String>());
    Ok(())
}
#[cfg(not(all(windows, feature = "native_renderer")))]
fn main() { eprintln!("Windows native_renderer required"); std::process::exit(1); }
