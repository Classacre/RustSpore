//! Compile borrowed original HLSL with the installed Microsoft D3DX9 compiler.
//! No original game module is loaded; include resolution is disabled.
use std::{
    ffi::{c_void, CString},
    ptr,
};
use windows::{
    core::{s, w},
    Win32::{
        Foundation::{FreeLibrary, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
    },
};

type Failure = Box<dyn std::error::Error>;
type Compile = unsafe extern "system" fn(
    *const u8,
    u32,
    *const c_void,
    *mut c_void,
    *const u8,
    *const u8,
    u32,
    *mut *mut c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> i32;

pub struct Compiler {
    module: HMODULE,
    compile: Compile,
}
impl Drop for Compiler {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.module);
        }
    }
}

// ID3DXBuffer is an IUnknown followed by GetBufferPointer and DWORD GetBufferSize.
// Instances are owned compiler outputs and never come from resource bytes.
struct Buffer(*mut c_void);
impl Buffer {
    unsafe fn method(&self, slot: usize) -> *const c_void {
        *(*(self.0.cast::<*const *const c_void>())).add(slot)
    }
    unsafe fn bytes(&self) -> Result<Vec<u8>, Failure> {
        if self.0.is_null() {
            return Ok(Vec::new());
        }
        let pointer: unsafe extern "system" fn(*mut c_void) -> *const u8 =
            std::mem::transmute(self.method(3));
        let size: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(self.method(4));
        let count = size(self.0) as usize;
        if count > 16 * 1024 * 1024 {
            return Err("D3DX output exceeds bounded compiler buffer".into());
        }
        if count == 0 {
            return Ok(Vec::new());
        }
        let data = pointer(self.0);
        if data.is_null() {
            return Err("D3DX returned a null nonempty buffer".into());
        }
        Ok(std::slice::from_raw_parts(data, count).to_vec())
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                    std::mem::transmute(self.method(2));
                release(self.0);
            }
        }
    }
}

pub struct Compiled {
    pub bytecode: Vec<u8>,
    pub diagnostics: String,
}
impl Compiler {
    pub fn system() -> Result<Self, Failure> {
        unsafe {
            let module = LoadLibraryExW(w!("d3dx9_27.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32)?;
            let Some(address) = GetProcAddress(module, s!("D3DXCompileShader")) else {
                let _ = FreeLibrary(module);
                return Err("system d3dx9_27.dll has no D3DXCompileShader".into());
            };
            Ok(Self {
                module,
                compile: std::mem::transmute(address),
            })
        }
    }
    pub fn compile(
        &self,
        source: &str,
        entrypoint: &str,
        profile: &str,
        flags: u32,
    ) -> Result<Compiled, Failure> {
        if source.len() > 4 * 1024 * 1024 || source.as_bytes().contains(&0) {
            return Err("unsupported HLSL source extent or embedded NUL".into());
        }
        if !matches!(profile, "vs_3_0" | "ps_3_0") || !matches!(flags, 0x12 | 0x32) {
            return Err("unestablished original compiler profile or flags".into());
        }
        let entrypoint = CString::new(entrypoint)?;
        let profile = CString::new(profile)?;
        let mut code = Buffer(ptr::null_mut());
        let mut errors = Buffer(ptr::null_mut());
        let status = unsafe {
            (self.compile)(
                source.as_ptr(),
                source.len() as u32,
                ptr::null(),
                ptr::null_mut(),
                entrypoint.as_ptr().cast(),
                profile.as_ptr().cast(),
                flags,
                &mut code.0,
                &mut errors.0,
                ptr::null_mut(),
            )
        };
        let diagnostics = unsafe { errors.bytes()? };
        let diagnostics = String::from_utf8_lossy(&diagnostics)
            .trim_end_matches('\0')
            .to_string();
        if status < 0 {
            return Err(format!(
                "D3DXCompileShader HRESULT={:08X}: {diagnostics}",
                status as u32
            )
            .into());
        }
        let bytecode = unsafe { code.bytes()? };
        if bytecode.len() < 8 || bytecode.len() % 4 != 0 {
            return Err("D3DX returned malformed shader tokens".into());
        }
        Ok(Compiled {
            bytecode,
            diagnostics,
        })
    }
}
