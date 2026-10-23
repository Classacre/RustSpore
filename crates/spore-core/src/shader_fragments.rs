//! Source HLSL fragment tables stored in a subset of type 0x0469A3F7 (.smt).
//!
//! Layout is measured from the installed payloads, independently walked by
//! tools/shader_fragment_probe.py. Metadata is retained without inventing shader
//! binding/state meanings. Original HLSL stays in the user's packages or memory.
use std::fmt;

pub const VERTEX_SLOTS: usize = 255;
pub const PIXEL_SLOTS: usize = 256;
const MAX_STRING: usize = 1_048_576;
const MAX_UNIFORMS: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentError {
    Truncated,
    UnsupportedHeader(u32),
    StringTooLarge(u32),
    InvalidUtf8,
    TooManyUniforms(u32),
    TrailingBytes(usize),
}
impl fmt::Display for FragmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "source shader fragments: {self:?}")
    }
}
impl std::error::Error for FragmentError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Pixel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderUniform {
    pub declaration: String,
    pub metadata: [u8; 12],
}
impl ShaderUniform {
    /// The four measured big-endian shorts. `descriptor` exposes the roles
    /// established by the native upload dispatch.
    pub fn words(&self) -> [u16; 4] {
        std::array::from_fn(|i| {
            u16::from_be_bytes(self.metadata[2 * i..2 * i + 2].try_into().unwrap())
        })
    }
    pub fn flags_word(&self) -> u32 {
        u32::from_be_bytes(self.metadata[8..12].try_into().unwrap())
    }
    pub fn descriptor(&self) -> crate::shader_binding::UniformDescriptor {
        crate::shader_binding::UniformDescriptor { raw: self.metadata }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderFragment {
    pub stage: ShaderStage,
    /// Zero-based storage slot, not a claimed native shader/builder ID.
    pub slot: usize,
    pub metadata: Vec<u8>,
    pub code: String,
    pub declarations: String,
    pub uniforms: Vec<ShaderUniform>,
    /// Extra pixel-stage payload, including source texture/sampler metadata.
    /// Exact individual field roles remain unassigned.
    pub texture_metadata: Option<[u8; 36]>,
    pub name: String,
}
impl ShaderFragment {
    /// Native readers gate the optional name on bit 1 of the final metadata
    /// word. Preserve the other flags until the fragment consumer is recovered.
    pub fn flags_word(&self) -> u32 {
        u32::from_be_bytes(self.metadata[self.metadata.len() - 4..].try_into().unwrap())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderFragments {
    pub header_word: u32,
    pub vertex: Vec<ShaderFragment>,
    pub pixel: Vec<ShaderFragment>,
}
struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], FragmentError> {
        let end = self
            .offset
            .checked_add(count)
            .filter(|end| *end <= self.data.len())
            .ok_or(FragmentError::Truncated)?;
        let value = &self.data[self.offset..end];
        self.offset = end;
        Ok(value)
    }
    fn word(&mut self) -> Result<u32, FragmentError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn text(&mut self) -> Result<String, FragmentError> {
        let count = self.word()?;
        if count as usize > MAX_STRING {
            return Err(FragmentError::StringTooLarge(count));
        }
        String::from_utf8(self.take(count as usize)?.to_vec())
            .map_err(|_| FragmentError::InvalidUtf8)
    }
    fn fragment(
        &mut self,
        stage: ShaderStage,
        slot: usize,
    ) -> Result<ShaderFragment, FragmentError> {
        let metadata = self
            .take(if stage == ShaderStage::Vertex { 15 } else { 14 })?
            .to_vec();
        let code = self.text()?;
        let declarations = self.text()?;
        let count = self.word()?;
        if count as usize > MAX_UNIFORMS {
            return Err(FragmentError::TooManyUniforms(count));
        }
        let mut uniforms = Vec::with_capacity(count as usize);
        for _ in 0..count {
            uniforms.push(ShaderUniform {
                declaration: self.text()?,
                metadata: self.take(12)?.try_into().unwrap(),
            });
        }
        let texture_metadata = if stage == ShaderStage::Pixel {
            Some(self.take(36)?.try_into().unwrap())
        } else {
            None
        };
        let flags = u32::from_be_bytes(metadata[metadata.len() - 4..].try_into().unwrap());
        let name = if flags & 2 != 0 {
            self.text()?
        } else {
            String::new()
        };
        Ok(ShaderFragment {
            stage,
            slot,
            metadata,
            code,
            declarations,
            uniforms,
            texture_metadata,
            name,
        })
    }
}

pub fn parse(data: &[u8]) -> Result<ShaderFragments, FragmentError> {
    let mut reader = Reader { data, offset: 0 };
    let header_word = reader.word()?;
    if header_word != 1 {
        return Err(FragmentError::UnsupportedHeader(header_word));
    }
    let mut vertex = Vec::with_capacity(VERTEX_SLOTS);
    let mut pixel = Vec::with_capacity(PIXEL_SLOTS);
    for slot in 0..VERTEX_SLOTS {
        vertex.push(reader.fragment(ShaderStage::Vertex, slot)?);
    }
    for slot in 0..PIXEL_SLOTS {
        pixel.push(reader.fragment(ShaderStage::Pixel, slot)?);
    }
    if reader.offset != data.len() {
        return Err(FragmentError::TrailingBytes(data.len() - reader.offset));
    }
    Ok(ShaderFragments {
        header_word,
        vertex,
        pixel,
    })
}

impl ShaderFragments {
    /// Native vertex IDs 1..=255 index a table whose entry zero is reserved.
    /// The on-disk stream begins with that table's entry one.
    pub fn vertex_by_id(&self, id: u8) -> Option<&ShaderFragment> {
        self.vertex.get(usize::from(id.checked_sub(1)?))
    }
    /// Native pixel storage includes entry zero, although zero terminates
    /// fragment recipe byte lists and cannot normally select that entry.
    pub fn pixel_by_id(&self, id: u8) -> Option<&ShaderFragment> {
        self.pixel.get(usize::from(id))
    }
    /// Re-encode measured fields for corpus verification, without compiling code.
    pub fn to_bytes(&self) -> Vec<u8> {
        fn text(output: &mut Vec<u8>, value: &str) {
            output.extend((value.len() as u32).to_be_bytes());
            output.extend(value.as_bytes());
        }
        let mut output = self.header_word.to_be_bytes().to_vec();
        for fragment in self.vertex.iter().chain(&self.pixel) {
            output.extend(&fragment.metadata);
            text(&mut output, &fragment.code);
            text(&mut output, &fragment.declarations);
            output.extend((fragment.uniforms.len() as u32).to_be_bytes());
            for uniform in &fragment.uniforms {
                text(&mut output, &uniform.declaration);
                output.extend(uniform.metadata);
            }
            if let Some(metadata) = fragment.texture_metadata {
                output.extend(metadata);
            }
            if fragment.flags_word() & 2 != 0 {
                text(&mut output, &fragment.name);
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> ShaderFragments {
        let make = |stage, slot| ShaderFragment {
            stage,
            slot,
            metadata: vec![0; if stage == ShaderStage::Vertex { 15 } else { 14 }],
            code: String::new(),
            declarations: String::new(),
            uniforms: vec![],
            texture_metadata: if stage == ShaderStage::Pixel {
                Some([0; 36])
            } else {
                None
            },
            name: String::new(),
        };
        let mut library = ShaderFragments {
            header_word: 1,
            vertex: (0..VERTEX_SLOTS)
                .map(|i| make(ShaderStage::Vertex, i))
                .collect(),
            pixel: (0..PIXEL_SLOTS)
                .map(|i| make(ShaderStage::Pixel, i))
                .collect(),
        };
        library.vertex[3].code = "Out.value = In.value;".into();
        library.vertex[3].name = "synthetic_vertex".into();
        library.vertex[3].metadata[14] = 2;
        library.vertex[3].uniforms.push(ShaderUniform {
            declaration: "float4 syntheticRows[2]".into(),
            metadata: [2, 28, 2, 28, 0, 2, 0, 0, 0, 0, 0, 8],
        });
        library.pixel[7].name = "synthetic_pixel".into();
        library.pixel[7].metadata[13] = 2;
        library.pixel[7].texture_metadata.as_mut().unwrap()[0..4]
            .copy_from_slice(&u32::MAX.to_be_bytes());
        library
    }
    #[test]
    fn independent_stage_widths_empty_slots_uniform_words_and_texture_payload_survive() {
        let original = fixture();
        let bytes = original.to_bytes();
        let parsed = parse(&bytes).unwrap();
        assert_eq!(parsed, original);
        assert_eq!(parsed.to_bytes(), bytes);
        assert_eq!(parsed.vertex[3].uniforms[0].words(), [540, 540, 2, 0]);
        assert_eq!(parsed.vertex[3].uniforms[0].flags_word(), 8);
        assert_eq!(parsed.pixel[7].texture_metadata.unwrap()[0..4], [255; 4]);
        assert_eq!(parsed.vertex_by_id(4).unwrap().name, "synthetic_vertex");
        assert!(parsed.vertex_by_id(0).is_none());
        assert_eq!(parsed.pixel_by_id(7).unwrap().name, "synthetic_pixel");
        assert_eq!(parsed.vertex_by_id(255).unwrap().slot, 254);
        assert_eq!(parsed.pixel_by_id(255).unwrap().slot, 255);
    }
    #[test]
    fn all_short_headers_table_boundaries_and_trailing_bytes_fail() {
        let mut bytes = fixture().to_bytes();
        for length in (0..64).chain([bytes.len() / 2, bytes.len() - 1]) {
            assert!(parse(&bytes[..length]).is_err());
        }
        bytes.push(0);
        assert_eq!(parse(&bytes), Err(FragmentError::TrailingBytes(1)));
        bytes[0..4].copy_from_slice(&7u32.to_be_bytes());
        assert_eq!(parse(&bytes), Err(FragmentError::UnsupportedHeader(7)));
    }
    #[test]
    fn oversized_strings_uniform_counts_and_invalid_text_are_explicit_errors() {
        let mut bytes = fixture().to_bytes();
        bytes[19..23].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(parse(&bytes), Err(FragmentError::StringTooLarge(u32::MAX)));
        let mut bytes = fixture().to_bytes();
        bytes[27..31].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(parse(&bytes), Err(FragmentError::TooManyUniforms(u32::MAX)));
        let mut original = fixture();
        original.vertex[0].code = "x".into();
        let mut bytes = original.to_bytes();
        bytes[23] = 0xFF;
        assert_eq!(parse(&bytes), Err(FragmentError::InvalidUtf8));
    }
}
