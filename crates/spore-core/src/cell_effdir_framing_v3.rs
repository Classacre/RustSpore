//! Read-only EFFDIR v4 framing and selected v1 records. This module does not
//! select packages, load resources, or render effects. Based on format facts in
//! tools/cell_skybox_effdir_audit_v1.py and its source-cell audit evidence.

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Truncated { offset: usize, requested: usize },
    UnsupportedDirectory(u16),
    UnsupportedVisual(u16),
    DuplicateSection(u16),
    InvalidSectionSize(u32),
    CountLimit(u32),
    SectionLength,
    InvalidExport(u32),
    UnsupportedAssignments(u32),
    TrailingBytes(usize),
}

type Result<T> = std::result::Result<T, Error>;

struct Reader<'a> { data: &'a [u8], pos: usize }
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&v| v <= self.data.len())
            .ok_or(Error::Truncated { offset: self.pos, requested: n })?;
        let out = &self.data[self.pos..end]; self.pos = end; Ok(out)
    }
    fn u8(&mut self) -> Result<u8> { Ok(self.take(1)?[0]) }
    fn u16(&mut self) -> Result<u16> { let b = self.take(2)?; Ok(u16::from_be_bytes([b[0], b[1]])) }
    fn u32(&mut self) -> Result<u32> { let b = self.take(4)?; Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]])) }
    fn count(&mut self, max: u32) -> Result<usize> {
        let n = self.u32()?; if n > max { return Err(Error::CountLimit(n)); } Ok(n as usize)
    }
    fn array(&mut self, width: usize, max: u32) -> Result<()> {
        let n = self.count(max)?;
        let size = n.checked_mul(width).ok_or(Error::CountLimit(n as u32))?;
        self.take(size)?; Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Section<'a> {
    pub kind: SectionKind, pub type_id: u16, pub version: u16,
    pub count: u32, pub body: &'a [u8],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind { Component, Resource }
#[derive(Debug, PartialEq, Eq)]
pub struct Block { pub type_id: u8, pub index: u32 }
#[derive(Debug, PartialEq, Eq)]
pub struct Visual { pub flags: u32, pub blocks: Vec<Block> }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key { pub group: u32, pub instance: u32 }
#[derive(Debug, PartialEq, Eq)]
pub struct Export { pub index: u32, pub key: Key }
#[derive(Debug, PartialEq, Eq)]
pub struct Screen { pub mode: u8, pub flags: u32, pub texture: Key }
#[derive(Debug, PartialEq, Eq)]
pub enum Screens { Absent, UnsupportedVersion(u16), V1(Vec<Screen>) }
#[derive(Debug, PartialEq, Eq)]
pub struct Directory<'a> {
    pub sections: Vec<Section<'a>>, pub screens: Screens, pub visuals: Vec<Visual>,
    pub exports: Vec<Export>, pub assignments_word: u32, pub imports: Vec<Key>,
}

fn sections<'a>(r: &mut Reader<'a>, kind: SectionKind, out: &mut Vec<Section<'a>>) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    loop {
        let type_id = r.u16()?;
        if type_id == 0xffff { break; }
        if !seen.insert(type_id) { return Err(Error::DuplicateSection(type_id)); }
        let version = r.u16()?; let size = r.u32()?;
        if size < 4 { return Err(Error::InvalidSectionSize(size)); }
        let mut s = Reader { data: r.take(size as usize)?, pos: 0 };
        let count = s.u32()?;
        out.push(Section { kind, type_id, version, count, body: s.take((size - 4) as usize)? });
    }
    Ok(())
}

fn screen_records(section: &Section<'_>) -> Result<Screens> {
    if section.version != 1 { return Ok(Screens::UnsupportedVersion(section.version)); }
    if section.count > 100_000 { return Err(Error::CountLimit(section.count)); }
    let mut r = Reader { data: section.body, pos: 0 };
    let mut screens = Vec::new();
    for _ in 0..section.count {
        let mode = r.u8()?; let flags = r.u32()?;
        for width in [12, 4, 4] { r.array(width, 100_000)?; }
        r.take(16)?;
        let texture = Key { group: r.u32()?, instance: r.u32()? };
        r.take(2)?;
        let n = r.count(100_000)?;
        for _ in 0..n { r.take(10)?; r.array(1, 100_000)?; }
        for width in [8, 4, 12, 8, 8] { r.array(width, 100_000)?; }
        screens.push(Screen { mode, flags, texture });
    }
    if r.pos != r.data.len() { return Err(Error::SectionLength); }
    Ok(Screens::V1(screens))
}

pub fn parse(data: &[u8]) -> Result<Directory<'_>> {
    let mut r = Reader { data, pos: 0 };
    let version = r.u16()?;
    if version != 4 { return Err(Error::UnsupportedDirectory(version)); }
    let mut all = Vec::new();
    sections(&mut r, SectionKind::Component, &mut all)?;
    sections(&mut r, SectionKind::Resource, &mut all)?;
    let screens = match all.iter().find(|s| s.kind == SectionKind::Component && s.type_id == 9) {
        Some(s) => screen_records(s)?, None => Screens::Absent,
    };
    let version = r.u16()?;
    if version != 1 { return Err(Error::UnsupportedVisual(version)); }
    let n = r.count(100_000)?; let mut visuals = Vec::new();
    for _ in 0..n {
        let flags = r.u32()?; r.take(21)?; r.array(4, 32)?; r.take(16)?;
        let n = r.count(10_000)?; let mut blocks = Vec::new();
        for _ in 0..n {
            let type_id = r.u8()?; r.take(60)?; r.array(12, 10_000)?; r.take(40)?;
            blocks.push(Block { type_id, index: r.u32()? });
        }
        visuals.push(Visual { flags, blocks });
    }
    let mut exports = Vec::new();
    loop {
        let index = r.u32()?; if index == u32::MAX { break; }
        let key = Key { group: r.u32()?, instance: r.u32()? };
        if !is_import(index) && index as usize >= visuals.len() { return Err(Error::InvalidExport(index)); }
        exports.push(Export { index, key });
    }
    let assignments_word = r.u32()?;
    if assignments_word != u32::MAX { return Err(Error::UnsupportedAssignments(assignments_word)); }
    let n = r.count(100_000)?; let mut imports = Vec::new();
    for _ in 0..n { imports.push(Key { group: r.u32()?, instance: r.u32()? }); }
    if r.pos != data.len() { return Err(Error::TrailingBytes(data.len() - r.pos)); }
    Ok(Directory { sections: all, screens, visuals, exports, assignments_word, imports })
}

fn is_import(index: u32) -> bool { index & 0x7f00_0000 == 0x7f00_0000 }

/// Returns every matching export. Import references remain unresolved; invalid
/// references and unsupported screen versions are explicit, not silently skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target<'a> { Visual(&'a Visual), Import(Option<&'a Key>), Invalid(u32) }
#[derive(Debug, PartialEq, Eq)]
pub enum ScreenRef<'a> { Screen(&'a Screen), Import(Option<&'a Key>), Null, Missing, Unsupported(u16) }
impl<'a> Directory<'a> {
    pub fn resolve(&self, key: Key) -> Vec<Target<'_>> {
        self.exports.iter().filter(|e| e.key == key).map(|e| {
            if is_import(e.index) { Target::Import(self.imports.get((e.index & 0x80ff_ffff) as usize)) }
            else { self.visuals.get(e.index as usize).map(Target::Visual).unwrap_or(Target::Invalid(e.index)) }
        }).collect()
    }
    pub fn screen_ref(&self, block: &Block) -> Option<ScreenRef<'_>> {
        if block.type_id != 9 { return None; }
        Some(if block.index == u32::MAX { ScreenRef::Null }
        else if is_import(block.index) { ScreenRef::Import(self.imports.get((block.index & 0x80ff_ffff) as usize)) }
        else { match &self.screens {
            Screens::Absent => ScreenRef::Missing,
            Screens::UnsupportedVersion(v) => ScreenRef::Unsupported(*v),
            Screens::V1(s) => s.get(block.index as usize).map(ScreenRef::Screen).unwrap_or(ScreenRef::Missing),
        } })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn u16be(b: &mut Vec<u8>, v: u16) { b.extend_from_slice(&v.to_be_bytes()); }
    fn u32be(b: &mut Vec<u8>, v: u32) { b.extend_from_slice(&v.to_be_bytes()); }
    fn fixture(screen_version: u16) -> Vec<u8> {
        let mut b = Vec::new(); u16be(&mut b, 4);
        let mut body = Vec::new(); body.push(5); u32be(&mut body, 1);
        for _ in 0..3 { u32be(&mut body, 0); } body.extend_from_slice(&[0; 16]);
        u32be(&mut body, 0); u32be(&mut body, 0xbd08bd8a); body.extend_from_slice(&[0; 2]);
        for _ in 0..6 { u32be(&mut body, 0); }
        u16be(&mut b, 9); u16be(&mut b, screen_version); u32be(&mut b, 4 + body.len() as u32); u32be(&mut b, 1); b.extend(body);
        u16be(&mut b, 0xffff); u16be(&mut b, 0xffff); u16be(&mut b, 1); u32be(&mut b, 1);
        u32be(&mut b, 0); b.extend_from_slice(&[0; 21]); u32be(&mut b, 0); b.extend_from_slice(&[0; 16]); u32be(&mut b, 1);
        b.push(9); b.extend_from_slice(&[0; 60]); u32be(&mut b, 0); b.extend_from_slice(&[0; 40]); u32be(&mut b, 0);
        u32be(&mut b, 0); u32be(&mut b, 0); u32be(&mut b, 0x05c7a4d0); u32be(&mut b, u32::MAX);
        u32be(&mut b, u32::MAX); u32be(&mut b, 0); b
    }
    #[test] fn resolves_explicit_key() {
        let bytes = fixture(1); let d = parse(&bytes).unwrap();
        let targets = d.resolve(Key { group: 0, instance: 0x05c7a4d0 });
        let Target::Visual(v) = targets[0] else { panic!() };
        assert_eq!(d.screen_ref(&v.blocks[0]), Some(ScreenRef::Screen(&Screen { mode: 5, flags: 1, texture: Key { group: 0, instance: 0xbd08bd8a } })));
        assert!(d.resolve(Key { group: 1, instance: 0x05c7a4d0 }).is_empty());
    }
    #[test] fn every_truncated_prefix_is_rejected() {
        let b = fixture(1); for n in 0..b.len() { assert!(parse(&b[..n]).is_err(), "prefix {n}"); }
    }
    #[test] fn unsupported_screen_body_is_preserved() {
        let b = fixture(7); let d = parse(&b).unwrap();
        assert_eq!(d.screens, Screens::UnsupportedVersion(7)); assert!(!d.sections[0].body.is_empty());
        assert_eq!(d.screen_ref(&d.visuals[0].blocks[0]), Some(ScreenRef::Unsupported(7)));
    }
    #[test] fn rejects_bad_section_and_trailing_bytes() {
        let mut b = fixture(1); b[6..10].copy_from_slice(&3u32.to_be_bytes()); assert!(matches!(parse(&b), Err(Error::InvalidSectionSize(3))));
        let mut b = fixture(1); b.push(0); assert!(matches!(parse(&b), Err(Error::TrailingBytes(1))));
    }
    #[test] fn rejects_unverified_assignments_and_duplicate_sections() {
        let mut b = fixture(1); let offset = b.len() - 8;
        b[offset..offset + 4].copy_from_slice(&0u32.to_be_bytes());
        assert_eq!(parse(&b), Err(Error::UnsupportedAssignments(0)));
        let mut b = vec![]; u16be(&mut b, 4);
        for _ in 0..2 { u16be(&mut b, 9); u16be(&mut b, 1); u32be(&mut b, 4); u32be(&mut b, 0); }
        assert_eq!(parse(&b), Err(Error::DuplicateSection(9)));
    }
    #[test] fn null_and_unresolved_import_are_not_decoded_as_screens() {
        let bytes = fixture(1); let d = parse(&bytes).unwrap();
        assert_eq!(d.screen_ref(&Block { type_id: 9, index: u32::MAX }), Some(ScreenRef::Null));
        assert_eq!(d.screen_ref(&Block { type_id: 9, index: 0x7f00_0000 }), Some(ScreenRef::Import(None)));
        assert!(d.screen_ref(&Block { type_id: 8, index: 0 }).is_none());
    }
    #[test] fn resolves_imports_duplicate_exports_and_mutated_indices_safely() {
        let bytes = fixture(1); let mut d = parse(&bytes).unwrap();
        let key = Key { group: 0, instance: 0x05c7a4d0 };
        d.imports.push(Key { group: 7, instance: 42 });
        d.exports.push(Export { index: 0x7f00_0000, key });
        let targets = d.resolve(key);
        assert_eq!(targets.len(), 2);
        assert!(matches!(targets[1], Target::Import(Some(&Key { group: 7, instance: 42 }))));
        d.exports[0].index = 12345;
        assert!(matches!(d.resolve(key)[0], Target::Invalid(12345)));
    }
}
