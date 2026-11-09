//! Unwired opt-in primitive for a prepared single-mesh source draw.
//!
//! The caller must obtain and validate source resources before construction;
//! this module does no package I/O, shader generation, compilation, or actor
//! selection. It is not yet connected to the one-shot path or Cell stage.
use super::native_d3d9::Failure;
use super::native_source_frame_v1::ValidatedFrame;
use spore_core::{native_x87, shader_binding::{self, UniformDescriptor}, shader_pack::CompiledProgram};
use windows::Win32::Graphics::Direct3D9::*;

pub const MODEL_INSTANCE: u64 = 0x4061_7100_a1c4_6df2;

/// Explicit, already resolved source binding. No default sampler state is inferred.
pub struct PreparedTexture {
    pub slot: u32,
    pub texture: IDirect3DTexture9,
    pub address: u32,
    pub filter: u32,
    pub mip: u32,
}

/// A caller-supplied fresh inspection lighting context, not retail scene lighting.
pub struct InspectionLighting {
    pub camera_direction: [f32; 4],
    pub sh: Vec<[f32; 4]>,
    pub fresh_world_sh_extra: [f32; 4],
    pub material_params: [f32; 4],
    pub lights: [[f32; 4]; 2],
}

/// Owned objects must have been created on the same device supplied to draw.
/// Resource identity, provenance, shader recipes and exact source bytes must be
/// checked by the future preparation layer; this struct alone does not prove them.
pub struct PreparedSourceDraw {
    pub model_instance: u64,
    pub mesh_count: usize,
    pub vertex_count: u32,
    pub stride: u32,
    pub vertices: Vec<u8>,
    pub indices: Vec<u32>,
    pub declaration: IDirect3DVertexDeclaration9,
    pub vertex_shader: IDirect3DVertexShader9,
    pub pixel_shader: IDirect3DPixelShader9,
    pub vertex_program: CompiledProgram,
    pub pixel_program: CompiledProgram,
    pub textures: Vec<PreparedTexture>,
}

impl PreparedSourceDraw {
    /// Enforce bounded structural invariants; source provenance is a separate gate.
    pub fn validate(&self) -> Result<(), Failure> {
        if self.model_instance != MODEL_INSTANCE || self.mesh_count != 1 || self.vertex_count == 0
            || self.vertex_count > 1_000_000 || self.stride == 0 || self.stride > 4096
            || usize::try_from(self.vertex_count)?.checked_mul(self.stride as usize) != Some(self.vertices.len())
            || self.indices.is_empty() || self.indices.len() > 3_000_000 || self.indices.len() % 3 != 0
            || self.indices.iter().any(|&i| i >= self.vertex_count) {
            return Err("unsupported prepared model geometry".into());
        }
        let mut seen = [false; 8];
        for binding in &self.textures {
            let slot=usize::try_from(binding.slot)?;
            if slot >= seen.len() || seen[slot] || !(1..=3).contains(&binding.address)
                || !(1..=2).contains(&binding.filter) || binding.mip > 2 {
                return Err("unsupported prepared sampler".into());
            }
            seen[slot]=true;
        }
        if !seen[0] { return Err("source sampler zero absent".into()); }
        Ok(())
    }

    /// Issue one draw inside the caller's active scene. The caller owns render
    /// target, viewport, depth/stencil/alpha/blend state and device lifetime.
    /// It must establish and restore these states explicitly. No readback or
    /// composition is implied. An error may leave device bindings changed.
    pub unsafe fn draw(&self, device: &IDirect3DDevice9, frame: &ValidatedFrame, light: &InspectionLighting) -> Result<(), Failure> {
        self.validate()?;
        if frame.width == 0 || frame.height == 0
            || frame.width > super::native_source_frame_v1::MAX_VIEWPORT
            || frame.height > super::native_source_frame_v1::MAX_VIEWPORT
            || !frame.model.is_finite() || !frame.clip.is_finite() || !frame.camera_world.is_finite() {
            return Err("invalid explicit frame".into());
        }
        let distance=shader_binding::camera_distance_constant(frame.model.w_axis.truncate().to_array(),
            frame.camera_world.to_array(), native_x87::Context::from_control_word(0x027f)?)?;
        if light.camera_direction.iter().chain(light.fresh_world_sh_extra.iter())
            .chain(light.material_params.iter()).chain(light.sh.iter().flatten())
            .chain(light.lights.iter().flatten()).any(|v| !v.is_finite()) {
            return Err("nonfinite inspection light".into());
        }
        for binding in &self.textures {
            device.SetTexture(binding.slot, &binding.texture)?;
            for (state,value) in [(D3DSAMP_ADDRESSU,binding.address),(D3DSAMP_ADDRESSV,binding.address),
                (D3DSAMP_ADDRESSW,binding.address),(D3DSAMP_MINFILTER,binding.filter),
                (D3DSAMP_MAGFILTER,binding.filter),(D3DSAMP_MIPFILTER,binding.mip),(D3DSAMP_SRGBTEXTURE,0)] {
                device.SetSamplerState(binding.slot,state,value)?;
            }
        }
        device.SetVertexDeclaration(&self.declaration)?;
        device.SetVertexShader(&self.vertex_shader)?;
        device.SetPixelShader(&self.pixel_shader)?;
        for (program, vertex) in [(&self.vertex_program,true),(&self.pixel_program,false)] {
            if program.uniforms.len()!=program.uniform_words.len() { return Err("uniform cardinality".into()); }
            for (raw,start) in program.uniforms.iter().zip(&program.uniform_words) {
                let desc=UniformDescriptor { raw:*raw };
                let count=usize::from(desc.register_count());
                let rows: Vec<[f32;4]> = match desc.callback_id() {
                    0x22 => vec![[1.;4]],
                    8 => (0..3).map(|i|frame.model.row(i).to_array()).collect(),
                    6 => (0..4).map(|i|frame.clip.row(i).to_array()).collect(),
                    0x21 => vec![light.camera_direction],
                    0x21a if count<=light.sh.len() => light.sh[..count].to_vec(),
                    0x219 => vec![light.fresh_world_sh_extra],
                    0x21b => vec![distance],
                    0x210 => vec![light.material_params],
                    0x214 => light.lights.to_vec(),
                    0x20c => vec![[1.;4]],
                    _ => return Err("unsupported uniform callback".into()),
                };
                if rows.len()!=count || rows.iter().flatten().any(|x|!x.is_finite()) { return Err("invalid uniform".into()); }
                if vertex { device.SetVertexShaderConstantF(*start,rows.as_ptr().cast(),count as u32)?; }
                else { device.SetPixelShaderConstantF(*start,rows.as_ptr().cast(),count as u32)?; }
            }
        }
        device.DrawIndexedPrimitiveUP(D3DPT_TRIANGLELIST,0,self.vertex_count,u32::try_from(self.indices.len()/3)?,
            self.indices.as_ptr().cast(),D3DFMT_INDEX32,self.vertices.as_ptr().cast(),self.stride)?;
        Ok(())
    }
}
