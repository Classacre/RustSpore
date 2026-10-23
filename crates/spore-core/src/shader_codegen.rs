//! Native vertex-fragment assembly for a missing compiled recipe.
//!
//! Original HLSL and boilerplate are read from the user's fragment library and
//! pinned executable, never embedded here. 6FE4D0 and its helpers establish the
//! declaration, register, varying, placeholder and assignment order. Compilation
//! and draw-time uniform values belong to the renderer.
use crate::shader_fragments::{ShaderFragments, ShaderUniform};
use std::{collections::BTreeSet, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenError {
    UnsupportedExecutable,
    InvalidImage,
    InvalidTemplate(u32),
    InvalidRecipe,
    MissingFragment(u8),
    InvalidMetadata(u8),
    ExcessVaryings,
    RegisterOverflow,
    InvalidFormat,
}
impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source vertex assembly: {self:?}")
    }
}
impl std::error::Error for CodegenError {}

/// Read-only templates for base EXE SHA256 17cf32b3...df5f87a.
/// The complete FNV identity rejects other executable versions before VA reads.
pub struct SourceTemplates<'a> {
    image: &'a [u8],
    sections: Vec<(usize, usize, usize)>,
}
impl<'a> SourceTemplates<'a> {
    pub fn from_original_executable(image: &'a [u8]) -> Result<Self, CodegenError> {
        let fingerprint = image.iter().fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
        });
        if image.len() != 20_447_744 || fingerprint != 0xa2a33f99447d2b27 {
            return Err(CodegenError::UnsupportedExecutable);
        }
        let word = |at: usize| -> Result<usize, CodegenError> {
            Ok(u32::from_le_bytes(
                image
                    .get(at..at + 4)
                    .ok_or(CodegenError::InvalidImage)?
                    .try_into()
                    .unwrap(),
            ) as usize)
        };
        let short = |at: usize| -> Result<usize, CodegenError> {
            Ok(u16::from_le_bytes(
                image
                    .get(at..at + 2)
                    .ok_or(CodegenError::InvalidImage)?
                    .try_into()
                    .unwrap(),
            ) as usize)
        };
        let pe = word(60)?;
        if image.get(..2) != Some(b"MZ")
            || image.get(pe..pe + 4) != Some(b"PE\0\0")
            || short(pe + 24)? != 0x10b
        {
            return Err(CodegenError::InvalidImage);
        }
        let base = word(pe + 52)?;
        if base != 0x400000 {
            return Err(CodegenError::InvalidImage);
        }
        let count = short(pe + 6)?;
        let table = pe + 24 + short(pe + 20)?;
        let mut sections = Vec::with_capacity(count);
        for i in 0..count {
            let row = table + i * 40;
            let size = word(row + 16)?;
            let offset = word(row + 20)?;
            if offset.checked_add(size).is_none_or(|end| end > image.len()) {
                return Err(CodegenError::InvalidImage);
            }
            sections.push((base + word(row + 12)?, size, offset));
        }
        Ok(Self { image, sections })
    }
    fn bytes_at(&self, address: u32) -> Result<&'a [u8], CodegenError> {
        let address = address as usize;
        for &(base, size, offset) in &self.sections {
            if address >= base && address - base < size {
                return Ok(&self.image[offset + address - base..offset + size]);
            }
        }
        Err(CodegenError::InvalidTemplate(address as u32))
    }
    fn text(&self, address: u32) -> Result<&'a str, CodegenError> {
        let data = self.bytes_at(address)?;
        let end = data
            .iter()
            .take(4096)
            .position(|b| *b == 0)
            .ok_or(CodegenError::InvalidTemplate(address))?;
        std::str::from_utf8(&data[..end]).map_err(|_| CodegenError::InvalidTemplate(address))
    }
    fn pointed_text(&self, pointer: u32) -> Result<&'a str, CodegenError> {
        let bytes = self
            .bytes_at(pointer)?
            .get(..4)
            .ok_or(CodegenError::InvalidTemplate(pointer))?;
        self.text(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
    pub fn entrypoint(&self) -> Result<&'a str, CodegenError> {
        self.text(0x140bb04)
    }
}

enum Argument<'a> {
    Number(usize),
    Text(&'a str),
}
fn formatted(template: &str, arguments: &[Argument<'_>]) -> Result<String, CodegenError> {
    let mut output = String::new();
    let mut rest = template;
    let mut index = 0;
    while let Some(at) = rest.find('%') {
        output.push_str(&rest[..at]);
        let code = *rest
            .as_bytes()
            .get(at + 1)
            .ok_or(CodegenError::InvalidFormat)?;
        if code == b'%' {
            output.push('%');
        } else {
            let value = arguments.get(index).ok_or(CodegenError::InvalidFormat)?;
            match (code, value) {
                (b'd', Argument::Number(value)) => output.push_str(&value.to_string()),
                (b's', Argument::Text(value)) => output.push_str(value),
                _ => return Err(CodegenError::InvalidFormat),
            }
            index += 1;
        }
        rest = &rest[at + 2..];
    }
    if index != arguments.len() {
        return Err(CodegenError::InvalidFormat);
    }
    output.push_str(rest);
    Ok(output)
}

// 6FD560 substitutes any ASCII-letter/digit token, but only 't' has a
// nonzero base in this vertex caller. Native's >10 test deliberately preserves
// its ':' result for an exact index10; do not silently repair source semantics.
fn replace_placeholders(code: &str, texture_base: usize) -> Result<String, CodegenError> {
    let mut output = Vec::with_capacity(code.len());
    let code = code.as_bytes();
    let mut i = 0;
    while i < code.len() {
        if code.get(i) == Some(&b'<')
            && code.get(i + 3) == Some(&b'>')
            && code[i + 1].is_ascii_alphabetic()
            && code[i + 2].is_ascii_digit()
        {
            let index = usize::from(code[i + 2] - b'0')
                + if code[i + 1] == b't' { texture_base } else { 0 };
            if index > 99 {
                return Err(CodegenError::ExcessVaryings);
            }
            if index > 10 {
                output.push((index / 10) as u8 + b'0');
                output.push((index % 10) as u8 + b'0');
            } else {
                output.push(index as u8 + b'0');
            }
            i += 4;
        } else {
            output.push(code[i]);
            i += 1;
        }
    }
    String::from_utf8(output).map_err(|_| CodegenError::InvalidFormat)
}

const INPUT_HEAD: &[(u32, u32)] = &[(3, 0x140b8c8), (4, 0x140be80), (0x18, 0x140b858)];
const INPUT_TAIL: &[(u32, u32)] = &[
    (0x4000, 0x140be60),
    (0x8000, 0x140be40),
    (0x20000, 0x140be20),
    (0x40000, 0x140be04),
    (0x400000, 0x140bde4),
    (0x800000, 0x140bdc0),
    (0x80000, 0x140bda4),
    (0x100000, 0x140bd84),
    (0x20, 0x140b83c),
    (0x200000, 0x140bd70),
    (0x10000, 0x140bd54),
];
const CURRENT_HEAD: &[(u32, u32)] = &[(4, 0x140bf80), (0x18, 0x140b93c)];
const CURRENT_TAIL: &[(u32, u32)] = &[
    (0x4000, 0x140bf58),
    (0x8000, 0x140bf44),
    (0x20000, 0x140bf30),
    (0x40000, 0x140bf1c),
    (0x400000, 0x140bf0c),
    (0x800000, 0x140bef8),
    (0x80000, 0x140bee4),
    (0x100000, 0x140bed0),
    (0x20, 0x140b92c),
    (0x200000, 0x140bec4),
    (0x10000, 0x140beb0),
];
const OUTPUT_TAIL: &[(u32, u32)] = &[
    (0x20, 0x140b83c),
    (0x200000, 0x140bd70),
    (0x10000, 0x140bd54),
];
const ASSIGN_HEAD: &[(u32, u32)] = &[(3, 0x140c1e4), (4, 0x140c1c4), (0x18, 0x140b964)];
const ASSIGN_TAIL: &[(u32, u32)] = &[
    (0x4000, 0x140c17c),
    (0x8000, 0x140c15c),
    (0x20000, 0x140c138),
    (0x40000, 0x140c118),
    (0x400000, 0x140c0f4),
    (0x800000, 0x140c0d0),
    (0x80000, 0x140c0b0),
    (0x100000, 0x140c08c),
    (0x20, 0x140c06c),
    (0x200000, 0x140c054),
    (0x10000, 0x140c030),
];

pub struct GeneratedVertexSource {
    pub hlsl: String,
    pub uniforms: Vec<ShaderUniform>,
    pub registers: Vec<u32>,
    pub varying_count: usize,
}
pub fn generate_vertex(
    library: &ShaderFragments,
    recipe: &[u8],
    templates: &SourceTemplates<'_>,
) -> Result<GeneratedVertexSource, CodegenError> {
    let end = recipe.iter().position(|b| *b == 0).unwrap_or(recipe.len());
    if end == 0 || end >= 32 {
        return Err(CodegenError::InvalidRecipe);
    }
    let selected: Vec<_> = recipe[..end]
        .iter()
        .map(|id| {
            library
                .vertex_by_id(*id)
                .ok_or(CodegenError::MissingFragment(*id))
        })
        .collect::<Result<_, _>>()?;
    let mut incoming = 0u32;
    let mut outgoing = 0u32;
    let mut sizes = Vec::new();
    let mut uniforms = Vec::new();
    let mut registers = Vec::new();
    let mut register = 0u32;
    let mut seen = BTreeSet::new();
    let mut hlsl = String::new();
    for (&id, fragment) in recipe.iter().zip(&selected) {
        let m = &fragment.metadata;
        if m.len() != 15 {
            return Err(CodegenError::InvalidMetadata(id));
        }
        incoming |= u32::from_be_bytes(m[..4].try_into().unwrap());
        outgoing |= u32::from_be_bytes(m[4..8].try_into().unwrap());
        let count = usize::from(m[8]);
        if sizes.len() + count > 8 || (count != 0 && !(1..=4).contains(&m[9])) {
            return Err(CodegenError::ExcessVaryings);
        }
        sizes.extend(std::iter::repeat_n(usize::from(m[9]), count));
        if m[10] != 0 || !fragment.declarations.is_empty() {
            for uniform in &fragment.uniforms {
                if seen.insert(uniform.declaration.as_str()) {
                    hlsl.push_str(&formatted(
                        templates.text(0x140baac)?,
                        &[
                            Argument::Text(&uniform.declaration),
                            Argument::Number(register as usize),
                        ],
                    )?);
                    registers.push(register);
                    register = register
                        .checked_add(u32::from(uniform.descriptor().register_count()))
                        .ok_or(CodegenError::RegisterOverflow)?;
                    uniforms.push(uniform.clone());
                }
            }
            if !fragment.declarations.is_empty() {
                hlsl.push_str(&formatted(
                    templates.text(0x140baa4)?,
                    &[Argument::Text(&fragment.declarations)],
                )?);
            }
        }
    }
    hlsl.push_str(templates.text(0x140be9c)?);
    append_fields(&mut hlsl, templates, incoming, INPUT_HEAD)?;
    for i in 0..8 {
        if incoming & (1 << (i + 6)) != 0 {
            hlsl.push_str(&formatted(
                templates.text(0x140b818)?,
                &[Argument::Number(i), Argument::Number(i)],
            )?);
        }
    }
    append_fields(&mut hlsl, templates, incoming, INPUT_TAIL)?;
    hlsl.push_str(templates.text(0x140b7f0)?);
    hlsl.push_str(templates.text(0x140bfa4)?);
    hlsl.push_str(templates.text(0x140bf90)?);
    append_fields(&mut hlsl, templates, incoming | outgoing, CURRENT_HEAD)?;
    for (i, size) in sizes.iter().enumerate() {
        hlsl.push_str(&formatted(
            templates.text(0x140bf68)?,
            &[Argument::Number(*size), Argument::Number(i)],
        )?);
    }
    append_fields(&mut hlsl, templates, incoming | outgoing, CURRENT_TAIL)?;
    hlsl.push_str(templates.text(0x140b7f0)?);
    hlsl.push_str(templates.text(0x140c01c)?);
    hlsl.push_str(templates.text(0x140bffc)?);
    if outgoing & 0x18 != 0 {
        hlsl.push_str(templates.text(0x140bfe0)?);
    }
    for (i, size) in sizes.iter().enumerate() {
        hlsl.push_str(&formatted(
            templates.text(0x140bfbc)?,
            &[
                Argument::Number(*size),
                Argument::Number(i),
                Argument::Number(i),
            ],
        )?);
    }
    append_fields(&mut hlsl, templates, outgoing, OUTPUT_TAIL)?;
    hlsl.push_str(templates.text(0x140b7f0)?);
    hlsl.push_str(templates.pointed_text(0x1535e44)?);
    append_fields(&mut hlsl, templates, incoming, ASSIGN_HEAD)?;
    for i in 0..sizes.len() {
        if incoming & (1 << (i + 6)) == 0 {
            break;
        }
        hlsl.push_str(&formatted(
            templates.text(0x140c19c)?,
            &[Argument::Number(i), Argument::Number(i)],
        )?);
    }
    append_fields(&mut hlsl, templates, incoming, ASSIGN_TAIL)?;
    let mut texture_base = 0;
    for fragment in &selected {
        hlsl.push_str(&replace_placeholders(&fragment.code, texture_base)?);
        texture_base += usize::from(fragment.metadata[8]);
    }
    hlsl.push_str(templates.text(0x1401b50)?);
    hlsl.push_str(templates.text(0x140c28c)?);
    if outgoing & 0x18 != 0 {
        hlsl.push_str(templates.text(0x140c26c)?);
    }
    for i in 0..texture_base {
        hlsl.push_str(&formatted(
            templates.text(0x140c244)?,
            &[Argument::Number(i), Argument::Number(i)],
        )?);
    }
    append_fields(
        &mut hlsl,
        templates,
        outgoing,
        &[
            (0x20, 0x140ba68),
            (0x200000, 0x140c22c),
            (0x10000, 0x140c208),
        ],
    )?;
    hlsl.push_str(templates.pointed_text(0x1535e40)?);
    Ok(GeneratedVertexSource {
        hlsl,
        uniforms,
        registers,
        varying_count: sizes.len(),
    })
}
fn append_fields(
    output: &mut String,
    templates: &SourceTemplates<'_>,
    mask: u32,
    fields: &[(u32, u32)],
) -> Result<(), CodegenError> {
    for &(flag, address) in fields {
        if mask & flag != 0 {
            output.push_str(templates.text(address)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholder_formatter_preserves_native_ten_and_unknown_letter_behavior() {
        assert_eq!(
            replace_placeholders("<t0>/<t1>/<s2>/<u3>/<v4>/<a5>/<T6>", 10).unwrap(),
            ":/11/2/3/4/5/6"
        );
        assert_eq!(
            replace_placeholders("<tt>/<tX>/<t01>/α", 0).unwrap(),
            "<tt>/<tX>/<t01>/α"
        );
    }
    #[test]
    fn template_formatter_checks_argument_types_and_literal_percent() {
        assert_eq!(
            formatted(
                "[%s/%d/%%]",
                &[Argument::Text("value"), Argument::Number(17)]
            )
            .unwrap(),
            "[value/17/%]"
        );
        assert_eq!(
            formatted("%d", &[Argument::Text("bad")]),
            Err(CodegenError::InvalidFormat)
        );
        assert_eq!(formatted("%", &[]), Err(CodegenError::InvalidFormat));
        assert_eq!(
            formatted("", &[Argument::Number(0)]),
            Err(CodegenError::InvalidFormat)
        );
    }
    #[test]
    fn another_executable_is_not_used_as_a_template_provider() {
        assert!(matches!(
            SourceTemplates::from_original_executable(b"MZ"),
            Err(CodegenError::UnsupportedExecutable)
        ));
    }
}
