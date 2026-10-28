//! Bounded pixel-source transcription of 6FAE60 and its helpers.
//!
//! Templates come from the pinned executable and fragment text from the caller's
//! library. Generated code and compilation are diagnostic, not runtime proof.
use crate::shader_codegen::{formatted, Argument, CodegenError, SourceTemplates};
use crate::shader_fragments::{ShaderFragments, ShaderUniform};
use std::collections::BTreeMap;

pub struct GeneratedPixelSource {
    pub hlsl: String,
    pub uniforms: Vec<ShaderUniform>,
    pub registers: Vec<u32>,
    pub varying_count: usize,
}

fn placeholders(code: &str, varying_base: usize, sampler_base: usize) -> Result<String, CodegenError> {
    let mut out = Vec::with_capacity(code.len());
    let b = code.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b.get(i) == Some(&b'<') && b.get(i + 3) == Some(&b'>')
            && b[i + 1].is_ascii_alphabetic() && b[i + 2].is_ascii_digit() {
            let base = match b[i + 1] { b't' => varying_base, b's' => sampler_base, _ => 0 };
            let value = usize::from(b[i + 2] - b'0') + base;
            if value > 99 { return Err(CodegenError::ExcessVaryings); }
            if value > 10 { out.push((value / 10) as u8 + b'0'); out.push((value % 10) as u8 + b'0'); }
            else { out.push(value as u8 + b'0'); }
            i += 4;
        } else { out.push(b[i]); i += 1; }
    }
    String::from_utf8(out).map_err(|_| CodegenError::InvalidFormat)
}

fn fields(out: &mut String, t: &SourceTemplates<'_>, mask: u32, items: &[(u32,u32)]) -> Result<(), CodegenError> {
    for &(bit, address) in items { if mask & bit != 0 { out.push_str(t.text(address)?); } }
    Ok(())
}

/// Build pixel HLSL for an explicit fragment sequence. Caller selects the
/// profile, compiles it, and verifies the linked stage and live bindings.
pub fn generate_pixel(library: &ShaderFragments, recipe: &[u8], templates: &SourceTemplates<'_>) -> Result<GeneratedPixelSource, CodegenError> {
    let end = recipe.iter().position(|&x| x == 0).unwrap_or(recipe.len());
    if end == 0 || end > 32 { return Err(CodegenError::InvalidRecipe); }
    let selected: Vec<_> = recipe[..end].iter().map(|&id| library.pixel_by_id(id).ok_or(CodegenError::MissingFragment(id))).collect::<Result<_,_>>()?;
    let mut incoming = 0u32; let mut outgoing = 0u32;
    let mut varying_count = 0usize; let mut sampler_count = 0usize;
    let mut positions = Vec::new(); let mut uniforms = Vec::new(); let mut registers = Vec::new();
    let mut seen = BTreeMap::new(); let mut register = 0u32; let mut hlsl = String::new();
    for (&id, fragment) in recipe[..end].iter().zip(&selected) {
        let m = &fragment.metadata;
        if m.len() != 14 { return Err(CodegenError::InvalidMetadata(id)); }
        let mask = u32::from_be_bytes(m[..4].try_into().unwrap());
        incoming |= mask; outgoing |= u32::from_be_bytes(m[4..8].try_into().unwrap());
        let n = ((mask >> 16) & 0xff) as usize;
        positions.push((varying_count, sampler_count));
        // Native caps varyings at eight. Reject instead of producing an
        // incomplete interface when a recipe would exceed that cap.
        if n > 8 - varying_count { return Err(CodegenError::ExcessVaryings); }
        varying_count += n;
        let samplers = usize::from(m[8]);
        if sampler_count + samplers > 16 { return Err(CodegenError::ExcessVaryings); }
        if m[9] != 0 || samplers != 0 || !fragment.declarations.is_empty() {
            for sampler in sampler_count..sampler_count + samplers {
                hlsl.push_str(&formatted(templates.text(0x140bad0)?, &[Argument::Number(sampler), Argument::Number(sampler)])?);
            }
            for uniform in &fragment.uniforms {
                if let Some(previous) = seen.get(&uniform.declaration) {
                    if previous != &uniform.metadata { return Err(CodegenError::InvalidMetadata(id)); }
                    continue;
                }
                seen.insert(uniform.declaration.clone(), uniform.metadata);
                hlsl.push_str(&formatted(templates.text(0x140baac)?, &[Argument::Text(&uniform.declaration), Argument::Number(register as usize)])?);
                registers.push(register); uniforms.push(uniform.clone());
                register = register.checked_add(u32::from(uniform.descriptor().register_count())).ok_or(CodegenError::RegisterOverflow)?;
            }
            if !fragment.declarations.is_empty() {
                hlsl.push_str(&formatted(templates.text(0x140baa4)?, &[Argument::Text(&fragment.declarations)])?);
            }
        }
        sampler_count += samplers;
    }
    hlsl.push_str(templates.text(0x140b8e8)?);
    fields(&mut hlsl, templates, incoming, &[(2,0x140b8c8),(4,0x140b8ac),(8,0x140b890),(16,0x140b870),(32,0x140b858),(0x140,0x140b83c)])?;
    for i in 0..varying_count { hlsl.push_str(&formatted(templates.text(0x140b818)?, &[Argument::Number(i), Argument::Number(i)])?); }
    fields(&mut hlsl, templates, incoming, &[(0x80,0x140b7f8)])?;
    hlsl.push_str(templates.text(0x140b7f0)?);
    hlsl.push_str(templates.text(0x140b94c)?);
    fields(&mut hlsl, templates, outgoing, &[(2,0x140b93c),(4,0x140b92c),(8,0x140b91c),(16,0x140b90c),(32,0x140b8fc)])?;
    hlsl.push_str(templates.text(0x140b7f0)?);
    hlsl.push_str(templates.text(0x140b9f8)?);
    fields(&mut hlsl, templates, outgoing, &[(2,0x140b9e0),(4,0x140b9c8),(8,0x140b9b0),(16,0x140b998),(32,0x140b980)])?;
    hlsl.push_str(templates.text(0x140b7f0)?);
    hlsl.push_str(templates.pointed_text(0x15355a4)?);
    hlsl.push_str(templates.text(0x1401b50)?);
    if outgoing & 2 != 0 && incoming & 0x20 != 0 { hlsl.push_str(templates.text(0x140b964)?); }
    for (fragment, &(varying, sampler)) in selected.iter().zip(&positions) { hlsl.push_str(&placeholders(&fragment.code, varying, sampler)?); }
    hlsl.push_str(templates.text(0x1401b50)?);
    fields(&mut hlsl, templates, outgoing, &[(2,0x140ba88),(4,0x140ba68),(8,0x140ba48),(16,0x140ba28),(32,0x140ba0c)])?;
    hlsl.push_str(templates.pointed_text(0x15355a0)?);
    Ok(GeneratedPixelSource { hlsl, uniforms, registers, varying_count })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pixel_placeholders_keep_distinct_bases_and_native_ten_behavior() {
        assert_eq!(placeholders("<t0>/<t1>/<s0>/<s2>/<u4>/<v3>", 10, 2).unwrap(), ":/11/2/4/4/3");
        assert_eq!(placeholders("<s9>", 0, 99), Err(CodegenError::ExcessVaryings));
        assert_eq!(placeholders("<tX>/α", 3, 4).unwrap(), "<tX>/α");
    }
}
