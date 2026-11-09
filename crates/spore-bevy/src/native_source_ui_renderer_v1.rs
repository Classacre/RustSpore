//! Explicit-input D3D9 source UI draw routine. Caller owns device, target,
//! scene lifetime, source bytes, clip choice, and framebuffer contents.
//! Mutates device state; no restoration, runtime state, effect, or HUD claim.
use spore_core::spui_draw::{ImageDrawFlags, UiClipPath, UiMatrix, UiVertex, UiViewport};
use std::ptr;
use windows::Win32::Graphics::Direct3D9::*;

/// Issue one ordered source UI batch into the current target inside an active
/// scene. Caller must bind an A8R8G8B8 single-sample target of the specified
/// extent; multisampled and other render targets are not admitted here.
/// This narrow fixture accepts only a white tint, 0..viewport clip,
/// linear/clamp source-alpha blending and explicit half-pixel capability.
/// The caller must supply trusted, fully validated installed-source shader token
/// streams and RGBA atlas bytes. Profile/end checks below do not validate token
/// lengths or shader safety: D3D Create*Shader receives no token length.
pub unsafe fn draw(
    device: &IDirect3DDevice9, vertex: &[u32], pixel: &[u32], rgba: &[u8],
    texture_width: u32, texture_height: u32, width: u32, height: u32,
    clip_path: UiClipPath, half_pixel: bool, rectangles: &[([f32; 4], [f32; 4])],
) -> Result<(), Box<dyn std::error::Error>> {
    if texture_width == 0 || texture_height == 0 || texture_width > 4096 || texture_height > 4096
        || width == 0 || height == 0 || width > 4096 || height > 4096
        || rgba.len() != texture_width as usize * texture_height as usize * 4
        || rectangles.is_empty() || rectangles.len() > 9
        || vertex.len() < 2 || pixel.len() < 2
        || vertex[0] != 0xfffe0300 || pixel[0] != 0xffff0300
        || vertex.last() != Some(&0xffff) || pixel.last() != Some(&0xffff)
        || rectangles.iter().any(|(r,u)| r.iter().chain(u).any(|x| !x.is_finite())) {
        return Err("unsupported source UI draw input".into());
    }
        let mut texture = None;
        device.CreateTexture(
            texture_width,
            texture_height,
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
        if lock.Pitch < texture_width as i32 * 4 || lock.pBits.is_null() {
            let _ = texture.UnlockRect(0);
            return Err("invalid texture lock".into());
        }
        for y in 0..texture_height as usize {
            let row = std::slice::from_raw_parts_mut(
                (lock.pBits as *mut u8).add(y * lock.Pitch as usize),
                texture_width as usize * 4,
            );
            for (dst, src) in row
                .chunks_exact_mut(4)
                .zip(rgba[y * texture_width as usize * 4..(y + 1) * texture_width as usize * 4].chunks_exact(4))
            {
                dst.copy_from_slice(&[src[2], src[1], src[0], src[3]]);
            }
        }
        texture.UnlockRect(0)?;
        device.SetViewport(&D3DVIEWPORT9 {
            X: 0,
            Y: 0,
            Width: width,
            Height: height,
            MinZ: 0.,
            MaxZ: 1.,
        })?;
        for (state, value) in [
            (D3DRS_STENCILENABLE, 0),
            (D3DRS_CLIPPING, 1),
            (D3DRS_FILLMODE, D3DFILL_SOLID.0 as u32),
            (D3DRS_CLIPPLANEENABLE, 0),
            (D3DRS_WRAP0, 0),
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
        {
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
        device.SetStreamSourceFreq(0, 1)?;
        // One explicitly specified source object; every call has its own
        // destination and mapped atlas rectangle, in source call order.
        let viewport = UiViewport::new([0., 0., width as f32, height as f32], half_pixel)?;
        let local = UiMatrix { rows: [[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]], kind: 0 };
        let registers = viewport.object_registers(0xffff_ffff, local, [0.,0.,width as f32,height as f32], clip_path)?;
        let mut constants = [0f32; 96 * 4];
        for (row, values) in registers.iter().enumerate() { constants[row*4..row*4+4].copy_from_slice(values); }
        let mut vertices = Vec::new();
        let mut indices = Vec::<u16>::new();
        for (r, uv) in rectangles {
            let base = vertices.len() as u16;
            for (xy, tc) in [([r[0],r[1]],[uv[0],uv[1]]),([r[2],r[1]],[uv[2],uv[1]]),([r[0],r[3]],[uv[0],uv[3]]),([r[2],r[3]],[uv[2],uv[3]])] {
                vertices.push(UiVertex::new(xy, 0xffff_ffff, tc, 0, true)?);
            }
            indices.extend([base,base+1,base+2,base+2,base+1,base+3]);
        }
        assert_eq!(std::mem::size_of::<UiVertex>(), 24);
        device.SetVertexShaderConstantF(0, constants.as_ptr(), 96)?;
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
        Ok(())
}
