//! Measured header-7 source shader packs (type 0x0469A3F7).
//!
//! Preserves D3D9 programs, fragment selectors and uniform metadata exactly.
//! Reading a pack does not compile or execute its shaders. Recipe lookup follows
//! the measured native cache key; actual draw bindings remain separate.
use std::fmt;

const MAX_SHADERS: usize = 4096;
const MAX_PROGRAMS: usize = 100_000;
const MAX_UNIFORMS: usize = 2048;
const MAX_SELECTORS: usize = 4096;
const MAX_STRING: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    Truncated,
    UnsupportedHeader(u32),
    Limit(&'static str, u32),
    InvalidUtf8,
    DuplicateRenderType(u8),
    TrailingBytes(usize),
}
impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source shader pack: {self:?}")
    }
}
impl std::error::Error for PackError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeError {
    MissingTerminator,
    InputTooLong,
    AmbiguousRecipe,
}
impl fmt::Display for RecipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source shader recipe: {self:?}")
    }
}
impl std::error::Error for RecipeError {}

/// Seven big-endian words precede each standard shader or fragment builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderHeader {
    pub words: [u32; 7],
    pub name: String,
}
impl ShaderHeader {
    pub fn shader_id(&self) -> u32 {
        self.words[0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardVariant {
    pub render_type: u8,
    /// Original D3D9 token streams; token words are little-endian inside these
    /// length-prefixed blobs, despite the surrounding big-endian pack.
    pub vertex_bytecode: Vec<u8>,
    pub pixel_bytecode: Vec<u8>,
    pub vertex_uniforms: Vec<[u8; 12]>,
    pub pixel_uniforms: Vec<[u8; 12]>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardShader {
    pub header: ShaderHeader,
    pub variants: Vec<StandardVariant>,
}

/// Fixed-size source selector, retained without guessed predicate meanings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentSelector {
    pub raw: [u8; 25],
}
impl FragmentSelector {
    /// Selector byte stored in the native descriptor at +1. Predicate meanings
    /// still require its consumer; this accessor only exposes source framing.
    pub fn condition_code(&self) -> u8 {
        self.raw[0]
    }
    pub fn short_words(&self) -> [u16; 3] {
        std::array::from_fn(|i| {
            u16::from_be_bytes(self.raw[1 + i * 2..3 + i * 2].try_into().unwrap())
        })
    }
    pub fn words(&self) -> [u32; 4] {
        std::array::from_fn(|i| {
            u32::from_be_bytes(self.raw[7 + i * 4..11 + i * 4].try_into().unwrap())
        })
    }
    pub fn trailing_byte(&self) -> u8 {
        self.raw[23]
    }
    /// Written separately into the native descriptor at +0 by the builder
    /// reader. Mapping this identity to fragment storage needs its loader.
    pub fn fragment_id(&self) -> u8 {
        self.raw[24]
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderVariant {
    pub render_type: u8,
    pub vertex: Vec<FragmentSelector>,
    pub pixel: Vec<FragmentSelector>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderBuilder {
    pub header: ShaderHeader,
    pub variants: Vec<BuilderVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProgram {
    pub signature: [u8; 32],
    pub bytecode: Vec<u8>,
    pub uniforms: Vec<[u8; 12]>,
    /// One big-endian constant-register start per uniform, consumed by the
    /// native vertex/pixel upload dispatch. Association and order are preserved.
    pub uniform_words: Vec<u32>,
    pub flags_word: u32,
}
impl CompiledProgram {
    /// Native cache lookup compares the zero-terminated fragment byte list,
    /// not all 32 saved bytes. Unused source padding remains preserved.
    pub fn fragment_recipe(&self) -> Result<&[u8], RecipeError> {
        let end = self
            .signature
            .iter()
            .position(|id| *id == 0)
            .ok_or(RecipeError::MissingTerminator)?;
        Ok(&self.signature[..end])
    }
    /// Native entry construction ORs the uniform dependency masks. This is a
    /// consistency check; it does not supply the missing per-draw bindings.
    pub fn uniform_dependency_mask(&self) -> u32 {
        self.uniforms.iter().fold(0, |flags, record| {
            flags | u32::from_be_bytes(record[8..12].try_into().unwrap())
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTable {
    /// Both source stage tables contain zero here; it is not a stage enum.
    pub header_word: u32,
    pub programs: Vec<CompiledProgram>,
}
impl CompiledTable {
    /// Resolve an emitted recipe to its original compiled program. Input zero
    /// terminates the recipe as in the native consumer; unresolved/ambiguous
    /// choices never fall back to an unrelated program.
    pub fn lookup(&self, fragment_ids: &[u8]) -> Result<Option<&CompiledProgram>, RecipeError> {
        let end = fragment_ids
            .iter()
            .position(|id| *id == 0)
            .unwrap_or(fragment_ids.len());
        if end >= 32 {
            return Err(RecipeError::InputTooLong);
        }
        let key = &fragment_ids[..end];
        let mut found = None;
        for program in &self.programs {
            if program.fragment_recipe()? == key {
                if found.is_some() {
                    return Err(RecipeError::AmbiguousRecipe);
                }
                found = Some(program);
            }
        }
        Ok(found)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderPack {
    pub header_word: u32,
    pub standard: Vec<StandardShader>,
    pub builders: Vec<ShaderBuilder>,
    /// First table contains vertex tokens, second contains pixel tokens.
    pub compiled_vertex: CompiledTable,
    pub compiled_pixel: CompiledTable,
    pub name: String,
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, size: usize) -> Result<&'a [u8], PackError> {
        let end = self
            .offset
            .checked_add(size)
            .filter(|n| *n <= self.data.len())
            .ok_or(PackError::Truncated)?;
        let bytes = &self.data[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }
    fn word(&mut self) -> Result<u32, PackError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, field: &'static str, limit: usize) -> Result<usize, PackError> {
        let count = self.word()?;
        if count as usize > limit {
            return Err(PackError::Limit(field, count));
        }
        Ok(count as usize)
    }
    fn blob(&mut self) -> Result<Vec<u8>, PackError> {
        let size = self.word()? as usize;
        Ok(self.take(size)?.to_vec())
    }
    fn text(&mut self) -> Result<String, PackError> {
        let size = self.count("string bytes", MAX_STRING)?;
        String::from_utf8(self.take(size)?.to_vec()).map_err(|_| PackError::InvalidUtf8)
    }
    fn header(&mut self) -> Result<ShaderHeader, PackError> {
        let raw = self.take(28)?;
        let words =
            std::array::from_fn(|i| u32::from_be_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap()));
        let name = if words[6] & 0x10 != 0 {
            self.text()?
        } else {
            String::new()
        };
        Ok(ShaderHeader { words, name })
    }
    fn uniforms(&mut self) -> Result<Vec<[u8; 12]>, PackError> {
        let count = self.count("uniforms", MAX_UNIFORMS)?;
        // Bound before allocating, even if a hostile count is below the cap.
        let bytes = self.take(count * 12)?;
        Ok(bytes
            .chunks_exact(12)
            .map(|b| b.try_into().unwrap())
            .collect())
    }
    fn selectors(&mut self) -> Result<Vec<FragmentSelector>, PackError> {
        let count = self.count("selectors", MAX_SELECTORS)?;
        let bytes = self.take(count * 25)?;
        Ok(bytes
            .chunks_exact(25)
            .map(|b| FragmentSelector {
                raw: b.try_into().unwrap(),
            })
            .collect())
    }
    fn render_type(&mut self, seen: &mut [bool; 255]) -> Result<Option<u8>, PackError> {
        let value = self.take(1)?[0];
        if value == 255 {
            return Ok(None);
        }
        if std::mem::replace(&mut seen[value as usize], true) {
            return Err(PackError::DuplicateRenderType(value));
        }
        Ok(Some(value))
    }
    fn standard(&mut self) -> Result<StandardShader, PackError> {
        let header = self.header()?;
        let mut variants = Vec::new();
        let mut seen = [false; 255];
        while let Some(render_type) = self.render_type(&mut seen)? {
            variants.push(StandardVariant {
                render_type,
                vertex_bytecode: self.blob()?,
                pixel_bytecode: self.blob()?,
                vertex_uniforms: self.uniforms()?,
                pixel_uniforms: self.uniforms()?,
            });
        }
        Ok(StandardShader { header, variants })
    }
    fn builder(&mut self) -> Result<ShaderBuilder, PackError> {
        let header = self.header()?;
        let mut variants = Vec::new();
        let mut seen = [false; 255];
        while let Some(render_type) = self.render_type(&mut seen)? {
            variants.push(BuilderVariant {
                render_type,
                vertex: self.selectors()?,
                pixel: self.selectors()?,
            });
        }
        Ok(ShaderBuilder { header, variants })
    }
    fn compiled(&mut self) -> Result<CompiledTable, PackError> {
        let header_word = self.word()?;
        let count = self.count("compiled programs", MAX_PROGRAMS)?;
        let mut programs = Vec::new();
        for _ in 0..count {
            let signature = self.take(32)?.try_into().unwrap();
            let bytecode = self.blob()?;
            let uniforms = self.uniforms()?;
            let raw = self.take(uniforms.len() * 4)?;
            let uniform_words = raw
                .chunks_exact(4)
                .map(|w| u32::from_be_bytes(w.try_into().unwrap()))
                .collect();
            let flags_word = self.word()?;
            programs.push(CompiledProgram {
                signature,
                bytecode,
                uniforms,
                uniform_words,
                flags_word,
            });
        }
        Ok(CompiledTable {
            header_word,
            programs,
        })
    }
}

pub fn parse(data: &[u8]) -> Result<ShaderPack, PackError> {
    let mut r = Reader { data, offset: 0 };
    let header_word = r.word()?;
    if header_word != 7 {
        return Err(PackError::UnsupportedHeader(header_word));
    }
    let count = r.count("standard shaders", MAX_SHADERS)?;
    let mut standard = Vec::new();
    for _ in 0..count {
        standard.push(r.standard()?);
    }
    let count = r.count("builders", MAX_SHADERS)?;
    let mut builders = Vec::new();
    for _ in 0..count {
        builders.push(r.builder()?);
    }
    let compiled_vertex = r.compiled()?;
    let compiled_pixel = r.compiled()?;
    let name = r.text()?;
    if r.offset != data.len() {
        return Err(PackError::TrailingBytes(data.len() - r.offset));
    }
    Ok(ShaderPack {
        header_word,
        standard,
        builders,
        compiled_vertex,
        compiled_pixel,
        name,
    })
}

fn word(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn count(out: &mut Vec<u8>, count: usize) {
    word(
        out,
        u32::try_from(count).expect("source shader count exceeds u32"),
    );
}
fn blob(out: &mut Vec<u8>, bytes: &[u8]) {
    count(out, bytes.len());
    out.extend_from_slice(bytes);
}
fn header(out: &mut Vec<u8>, header: &ShaderHeader) {
    for value in header.words {
        word(out, value);
    }
    if header.words[6] & 0x10 != 0 {
        blob(out, header.name.as_bytes());
    }
}
fn uniforms(out: &mut Vec<u8>, records: &[[u8; 12]]) {
    count(out, records.len());
    for record in records {
        out.extend_from_slice(record);
    }
}
fn selectors(out: &mut Vec<u8>, records: &[FragmentSelector]) {
    count(out, records.len());
    for record in records {
        out.extend_from_slice(&record.raw);
    }
}
fn compiled(out: &mut Vec<u8>, table: &CompiledTable) {
    word(out, table.header_word);
    count(out, table.programs.len());
    for program in &table.programs {
        assert_eq!(program.uniforms.len(), program.uniform_words.len());
        out.extend_from_slice(&program.signature);
        blob(out, &program.bytecode);
        uniforms(out, &program.uniforms);
        for value in &program.uniform_words {
            word(out, *value);
        }
        word(out, program.flags_word);
    }
}
impl ShaderPack {
    /// Byte-preserving serialization used to check the independently measured
    /// framing. It does not synthesize missing shader recipes or bindings.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        word(&mut out, self.header_word);
        count(&mut out, self.standard.len());
        for shader in &self.standard {
            header(&mut out, &shader.header);
            for variant in &shader.variants {
                out.push(variant.render_type);
                blob(&mut out, &variant.vertex_bytecode);
                blob(&mut out, &variant.pixel_bytecode);
                uniforms(&mut out, &variant.vertex_uniforms);
                uniforms(&mut out, &variant.pixel_uniforms);
            }
            out.push(255);
        }
        count(&mut out, self.builders.len());
        for builder in &self.builders {
            header(&mut out, &builder.header);
            for variant in &builder.variants {
                out.push(variant.render_type);
                selectors(&mut out, &variant.vertex);
                selectors(&mut out, &variant.pixel);
            }
            out.push(255);
        }
        compiled(&mut out, &self.compiled_vertex);
        compiled(&mut out, &self.compiled_pixel);
        blob(&mut out, self.name.as_bytes());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> ShaderPack {
        let header = ShaderHeader {
            words: [0x7000_0060, 0, 0, 0x5D, 0xFFFE0300, 0xFFFF0300, 0x18],
            name: "sample".into(),
        };
        let empty = CompiledTable {
            header_word: 0,
            programs: vec![],
        };
        ShaderPack {
            header_word: 7,
            standard: vec![StandardShader {
                header: header.clone(),
                variants: vec![StandardVariant {
                    render_type: 0,
                    vertex_bytecode: vec![0, 3, 254, 255],
                    pixel_bytecode: vec![0, 3, 255, 255],
                    vertex_uniforms: vec![[7; 12]],
                    pixel_uniforms: vec![],
                }],
            }],
            builders: vec![ShaderBuilder {
                header,
                variants: vec![BuilderVariant {
                    render_type: 45,
                    vertex: vec![FragmentSelector { raw: [11; 25] }],
                    pixel: vec![],
                }],
            }],
            compiled_vertex: CompiledTable {
                header_word: 0,
                programs: vec![CompiledProgram {
                    signature: [0xCD; 32],
                    bytecode: vec![0, 3, 254, 255, 255, 255, 0, 0],
                    uniforms: vec![[12; 12], [9; 12]],
                    uniform_words: vec![0, 192],
                    flags_word: 0x4001,
                }],
            },
            compiled_pixel: empty,
            name: "independent fixture".into(),
        }
    }
    #[test]
    fn mixed_tables_preserve_all_source_fields() {
        let expected = sample();
        let bytes = expected.to_bytes();
        assert_eq!(parse(&bytes).unwrap(), expected);
        assert_eq!(parse(&bytes).unwrap().to_bytes(), bytes);
        assert_eq!(expected.standard[0].header.shader_id(), 0x7000_0060);
    }
    #[test]
    fn all_truncated_prefixes_and_trailing_data_fail() {
        let bytes = sample().to_bytes();
        for prefix in 0..bytes.len() {
            assert!(parse(&bytes[..prefix]).is_err(), "accepted prefix {prefix}");
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(matches!(parse(&trailing), Err(PackError::TrailingBytes(1))));
    }
    #[test]
    fn count_limits_and_duplicate_variants_fail() {
        let mut bytes = sample().to_bytes();
        bytes[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(
            parse(&bytes),
            Err(PackError::Limit("standard shaders", _))
        ));
        assert!(matches!(
            parse(&[0, 0, 0, 1]),
            Err(PackError::UnsupportedHeader(1))
        ));
        let mut duplicate = sample();
        let variant = duplicate.standard[0].variants[0].clone();
        duplicate.standard[0].variants.push(variant);
        assert!(matches!(
            parse(&duplicate.to_bytes()),
            Err(PackError::DuplicateRenderType(0))
        ));
    }
    #[test]
    fn native_optional_name_and_selector_field_framing() {
        let mut pack = sample();
        pack.standard[0].header.words[6] &= !0x10;
        pack.standard[0].header.name.clear();
        assert_eq!(parse(&pack.to_bytes()).unwrap(), pack);
        let selector = FragmentSelector {
            raw: [
                1, 2, 44, 0, 0, 0, 0, 0, 8, 0, 68, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 255, 20,
            ],
        };
        assert_eq!(selector.condition_code(), 1);
        assert_eq!(selector.short_words(), [0x22C, 0, 0]);
        assert_eq!(selector.words(), [0x0008_0044, 0, 1, 1]);
        assert_eq!(selector.trailing_byte(), 255);
        assert_eq!(selector.fragment_id(), 20);
    }
    #[test]
    fn native_cache_ignores_saved_padding_and_stops_at_input_terminator() {
        let mut table = sample().compiled_vertex;
        table.programs[0].signature = [0xCD; 32];
        table.programs[0].signature[..3].copy_from_slice(&[3, 17, 0]);
        assert_eq!(
            table
                .lookup(&[3, 17])
                .unwrap()
                .unwrap()
                .fragment_recipe()
                .unwrap(),
            [3, 17]
        );
        assert!(table.lookup(&[3, 17, 0, 99]).unwrap().is_some());
        assert!(table.lookup(&[3, 18]).unwrap().is_none());
        assert_eq!(table.lookup(&[1; 32]), Err(RecipeError::InputTooLong));
        let mut duplicate = table.programs[0].clone();
        duplicate.signature[4] = 0xAB;
        table.programs.push(duplicate);
        assert_eq!(table.lookup(&[3, 17]), Err(RecipeError::AmbiguousRecipe));
    }
    #[test]
    fn unbounded_source_recipe_is_an_error_and_dependency_masks_use_every_uniform() {
        let mut program = sample().compiled_vertex.programs.remove(0);
        assert_eq!(
            program.fragment_recipe(),
            Err(RecipeError::MissingTerminator)
        );
        program.uniforms[0][8..12].copy_from_slice(&0x4001u32.to_be_bytes());
        program.uniforms[1][8..12].copy_from_slice(&8u32.to_be_bytes());
        assert_eq!(program.uniform_dependency_mask(), 0x4009);
    }
}
