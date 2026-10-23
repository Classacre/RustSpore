//! Offscreen original-program rendering. The HWND is never shown or presented.
use std::{ffi::CStr, ptr};
use windows::{
    core::w,
    Win32::{
        Foundation::HWND,
        Graphics::Direct3D9::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WS_EX_NOACTIVATE, WS_POPUP},
    },
};

pub type Failure = Box<dyn std::error::Error>;
struct HiddenWindow(HWND);
impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

pub struct OffscreenDevice {
    pub device: IDirect3DDevice9,
    color: IDirect3DSurface9,
    _depth: IDirect3DSurface9,
    _d3d: IDirect3D9,
    // Drop the COM objects before their hidden device window.
    _window: HiddenWindow,
    width: u32,
    height: u32,
}

impl OffscreenDevice {
    pub unsafe fn new(width: u32, height: u32) -> Result<Self, Failure> {
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("unsupported offscreen target extent".into());
        }
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
                "SOURCE_3D_ADAPTER index={adapter} vendor={:04X} description={description:?}",
                id.VendorId
            );
            if id.VendorId == 0x10de && description.contains("RTX 3060") {
                chosen = Some(adapter);
            }
        }
        let adapter = chosen.ok_or("RTX 3060 unavailable to D3D9; hardware check cannot run")?;
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
        let mut color = None;
        device.CreateRenderTarget(
            width,
            height,
            D3DFMT_A8R8G8B8,
            D3DMULTISAMPLE_NONE,
            0,
            false,
            &mut color,
            ptr::null_mut(),
        )?;
        let color = color.ok_or("missing color target")?;
        let mut depth = None;
        device.CreateDepthStencilSurface(
            width,
            height,
            D3DFMT_D24S8,
            D3DMULTISAMPLE_NONE,
            0,
            true,
            &mut depth,
            ptr::null_mut(),
        )?;
        let depth = depth.ok_or("missing depth target")?;
        device.SetRenderTarget(0, &color)?;
        device.SetDepthStencilSurface(&depth)?;
        device.SetViewport(&D3DVIEWPORT9 {
            X: 0,
            Y: 0,
            Width: width,
            Height: height,
            MinZ: 0.,
            MaxZ: 1.,
        })?;
        Ok(Self {
            device,
            color,
            _depth: depth,
            _d3d: d3d,
            _window: window,
            width,
            height,
        })
    }

    /// Upload the exact source compressed blocks and complete mip chain. D3D9
    /// performs the original GPU DXT sampling; no CPU recoloring or mips are made.
    pub unsafe fn upload_raster(
        &self,
        raster: &spore_core::raster::Raster,
    ) -> Result<IDirect3DTexture9, Failure> {
        use spore_core::raster::*;
        if raster.width == 0
            || raster.height == 0
            || raster.width > 8192
            || raster.height > 8192
            || raster.mipmaps == 0
            || raster.mipmaps > raster.width.max(raster.height).ilog2() + 1
        {
            return Err("unsupported source raster extent/mip count".into());
        }
        let block = match raster.format {
            FORMAT_DXT1 => 8,
            FORMAT_DXT3 | FORMAT_DXT5 => 16,
            FORMAT_RAW_RGBA => 4,
            FORMAT_L8 => 1,
            _ => return Err("unsupported source D3D9 texture format".into()),
        };
        let compressed = matches!(raster.format, FORMAT_DXT1 | FORMAT_DXT3 | FORMAT_DXT5);
        let mut texture = None;
        self.device.CreateTexture(
            raster.width,
            raster.height,
            raster.mipmaps,
            0,
            D3DFORMAT(raster.format),
            D3DPOOL_MANAGED,
            &mut texture,
            ptr::null_mut(),
        )?;
        let texture = texture.ok_or("missing source texture")?;
        let mut uploaded = 0;
        for (level, mip) in raster.mip_levels()?.iter().enumerate() {
            let level = level as u32;
            let width = mip.width as usize;
            let height = mip.height as usize;
            let (row_bytes, rows) = if compressed {
                (width.div_ceil(4) * block, height.div_ceil(4))
            } else {
                (width * block, height)
            };
            let source = mip.data;
            let mut lock = D3DLOCKED_RECT::default();
            texture.LockRect(level, &mut lock, ptr::null(), 0)?;
            if lock.Pitch < row_bytes as i32 || lock.pBits.is_null() {
                let _ = texture.UnlockRect(level);
                return Err("invalid source texture lock".into());
            }
            for y in 0..rows {
                ptr::copy_nonoverlapping(
                    source.as_ptr().add(y * row_bytes),
                    (lock.pBits as *mut u8).add(y * lock.Pitch as usize),
                    row_bytes,
                );
            }
            texture.UnlockRect(level)?;
            uploaded += source.len();
        }
        println!(
            "SOURCE_3D_MIP_CHAIN uploaded_bytes={uploaded} later_length_prefixes={} exact=true",
            raster.mipmaps - 1
        );
        Ok(texture)
    }

    pub unsafe fn read_rgba(&self) -> Result<Vec<u8>, Failure> {
        let mut staging = None;
        self.device.CreateOffscreenPlainSurface(
            self.width,
            self.height,
            D3DFMT_A8R8G8B8,
            D3DPOOL_SYSTEMMEM,
            &mut staging,
            ptr::null_mut(),
        )?;
        let staging = staging.ok_or("missing readback surface")?;
        self.device.GetRenderTargetData(&self.color, &staging)?;
        let mut lock = D3DLOCKED_RECT::default();
        staging.LockRect(&mut lock, ptr::null(), D3DLOCK_READONLY as u32)?;
        if lock.Pitch < self.width as i32 * 4 || lock.pBits.is_null() {
            let _ = staging.UnlockRect();
            return Err("invalid readback lock".into());
        }
        let mut rgba = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        for y in 0..self.height as usize {
            let row = std::slice::from_raw_parts(
                (lock.pBits as *const u8).add(y * lock.Pitch as usize),
                self.width as usize * 4,
            );
            for b in row.chunks_exact(4) {
                rgba.extend_from_slice(&[b[2], b[1], b[0], b[3]]);
            }
        }
        staging.UnlockRect()?;
        Ok(rgba)
    }
}

pub fn tokens(bytes: &[u8], profile: u32) -> Result<Vec<u32>, Failure> {
    if bytes.len() < 8 || bytes.len() % 4 != 0 {
        return Err("invalid source token extent".into());
    }
    let words: Vec<_> = bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    if words[0] != profile || words.last() != Some(&0x0000ffff) {
        return Err("unexpected source shader profile/end token".into());
    }
    Ok(words)
}
