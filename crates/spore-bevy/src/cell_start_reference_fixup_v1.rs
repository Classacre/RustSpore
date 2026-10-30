//! Bounded symbolic fixture for Cell globals.startCell type-3 reference fixup.
//!
//! Static source: re/cell-player-start-definition-static-v1.md, original
//! SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! E4AAD0 replaces a nonzero type-3 field through E82450(raw ID, descriptor);
//! E82450 interns by the raw ID AND descriptor pointer. E82200 builds an
//! (ID, descriptor type, scope 0) request under its ordinary path. This fixture
//! uses symbolic descriptor and handle tokens: no pointer, resource-manager
//! cache, original allocator, load/fallback, actor, or live winner is modeled.
//!
//! Private, unwired source fixture using dbpf and spore-core; it is not a Cell
//! update system. Do not interpret fixture success as evidence of the native
//! process, selected package or gameplay reachability. Cell parsing checks exact structural extent,
//! not semantic validity, source fixups, or that bytes match claimed provenance.

use std::sync::atomic::{AtomicU64, Ordering};
use dbpf::native_index::NativeResourceKey;
use spore_core::{cell_definition, cell_globals};

// Monotonic process-local fixture identities. Exhaustion fails closed.
static NEXT_TABLE_ID: AtomicU64 = AtomicU64::new(1);

const GLOBALS_BYTES: usize = cell_globals::GLOBALS_SIZE;
const CELL_BYTES: usize = cell_definition::DEFINITION_SIZE;
const GLOBALS_ID: u32 = 0xa426_730b;
const GLOBALS_TYPE: u32 = 0x2a3c_e5b7;
const CELL_TYPE: u32 = 0xdfad_9f51;
const MAX_HANDLES: usize = 256;

/// Symbolic identity represents a descriptor object, not its type hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Descriptor { pub identity: DescriptorId, pub type_id: u32 }
/// Identifies an entry only within this fixture's table; never a native pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolicHandle { table_id: u64, index: usize }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry { pub id: u32, pub descriptor: Descriptor }
#[derive(Debug)]
pub struct HandleTable { table_id: u64, entries: Vec<Entry> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TableIdentityExhausted,
    DescriptorIdentityConflict,
    HandleCapacity,
    BadGlobalsKey,
    GlobalsLayout,
    BadCellDescriptor,
    NullStartCell,
    BadCellKey,
    CellLayout,
    Unavailable,
    Ambiguous,
    FallbackUnresolved,
}

impl HandleTable {
    /// Each table gets a never-reused process-local identity. There is no reset
    /// or deletion; dropping a table invalidates its handles for later tables.
    pub fn new() -> Result<Self, Error> {
        let table_id = NEXT_TABLE_ID.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
            |next| next.checked_add(1)).map_err(|_| Error::TableIdentityExhausted)?;
        Ok(Self { table_id, entries: Vec::new() })
    }
    /// Intern the exact (raw nonzero word, descriptor identity) pair. Type-3
    /// zero fields do not call E82450. Contradictory descriptor type metadata
    /// is rejected instead of silently conflating symbolic descriptor objects.
    pub fn fixup(&mut self, id: u32, descriptor: Descriptor) -> Result<Option<SymbolicHandle>, Error> {
        if id == 0 { return Ok(None); }
        if self.entries.iter().any(|e| e.descriptor.identity == descriptor.identity && e.descriptor.type_id != descriptor.type_id) {
            return Err(Error::DescriptorIdentityConflict);
        }
        if let Some(index) = self.entries.iter().position(|e| e.id == id && e.descriptor.identity == descriptor.identity) {
            return Ok(Some(SymbolicHandle { table_id: self.table_id, index }));
        }
        if self.entries.len() >= MAX_HANDLES { return Err(Error::HandleCapacity); }
        let handle = SymbolicHandle { table_id: self.table_id, index: self.entries.len() };
        self.entries.push(Entry { id, descriptor });
        Ok(Some(handle))
    }
    pub fn get(&self, handle: SymbolicHandle) -> Option<Entry> {
        if handle.table_id != self.table_id { return None; }
        self.entries.get(handle.index).copied()
    }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

/// A choice already supplied by the caller, not chosen by this fixture.
/// `provenance` is an opaque caller label; it is not an authenticated package.
#[derive(Debug)]
pub struct SuppliedResource<'a> { pub key: NativeResourceKey, pub bytes: &'a [u8], pub provenance: &'a str }
#[derive(Debug)]
pub enum Resolution<'a> {
    Supplied(SuppliedResource<'a>),
    Unavailable,
    Ambiguous,
    FallbackUnresolved,
}
fn resolved(r: Resolution<'_>) -> Result<SuppliedResource<'_>, Error> {
    match r {
        Resolution::Supplied(r) => Ok(r),
        Resolution::Unavailable => Err(Error::Unavailable),
        Resolution::Ambiguous => Err(Error::Ambiguous),
        Resolution::FallbackUnresolved => Err(Error::FallbackUnresolved),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StartCandidate<'a> {
    pub handle: SymbolicHandle,
    pub cell_key: NativeResourceKey,
    pub globals_provenance: &'a str,
    pub cell_provenance: &'a str,
    /// Original pre-fixup bytes, structurally validated by `cell_definition`.
    cell_bytes: &'a [u8],
}
impl StartCandidate<'_> {
    pub fn cell_definition(&self) -> cell_definition::CellDefinition<'_> {
        cell_definition::parse(self.cell_bytes).expect("candidate was structurally validated")
    }
}

/// Scope-zero, current-layout, ordinary-path fixture. The caller supplies the
/// exact resource winner for *each* request; no priority, fallback, alias map,
/// cached provider or loading behavior is inferred. The complete input is
/// checked before the handle table is mutated, including the selected cell.
/// This atomicity is a fixture safety policy: native fixup interns before
/// resource acquisition, and nonzero FFFFFFFF is interned before E82200 rejects
/// its ordinary lookup. Supplied resource provenance and descriptor identity
/// are caller assertions, not verified native pointer or package evidence.
pub fn prepare_start<'a>(table: &mut HandleTable, globals: Resolution<'a>, cell: Resolution<'a>,
    cell_descriptor: Descriptor) -> Result<StartCandidate<'a>, Error> {
    let globals = resolved(globals)?;
    if globals.key != (NativeResourceKey { instance_id: GLOBALS_ID, type_id: GLOBALS_TYPE, group_id: 0 }) {
        return Err(Error::BadGlobalsKey);
    }
    let globals_view = cell_globals::parse(globals.bytes).map_err(|_| Error::GlobalsLayout)?;
    if cell_descriptor.type_id != CELL_TYPE { return Err(Error::BadCellDescriptor); }
    let id = globals_view.start_cell();
    if id == 0 { return Err(Error::NullStartCell); }
    // E82200 rejects -1 rather than treating it as an ordinary request.
    if id == u32::MAX { return Err(Error::FallbackUnresolved); }
    let expected = NativeResourceKey { instance_id: id, type_id: CELL_TYPE, group_id: 0 };
    let cell = resolved(cell)?;
    if cell.key != expected { return Err(Error::BadCellKey); }
    cell_definition::parse(cell.bytes).map_err(|_| Error::CellLayout)?;
    let handle = table.fixup(id, cell_descriptor)?.ok_or(Error::NullStartCell)?;
    Ok(StartCandidate { handle, cell_key: expected,
        globals_provenance: globals.provenance, cell_provenance: cell.provenance, cell_bytes: cell.bytes })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn desc(identity: u64) -> Descriptor { Descriptor { identity: DescriptorId(identity), type_id: CELL_TYPE } }
    fn globals(bytes: &[u8]) -> Resolution<'_> { Resolution::Supplied(SuppliedResource { key: NativeResourceKey { instance_id: GLOBALS_ID, type_id: GLOBALS_TYPE, group_id: 0 }, bytes, provenance: "fixture globals" }) }
    fn cell(id: u32, bytes: &[u8]) -> Resolution<'_> { Resolution::Supplied(SuppliedResource { key: NativeResourceKey { instance_id: id, type_id: CELL_TYPE, group_id: 0 }, bytes, provenance: "fixture cell" }) }
    fn current(id: u32) -> [u8; GLOBALS_BYTES] { let mut b = [0; GLOBALS_BYTES]; b[0x34..0x38].copy_from_slice(&id.to_le_bytes()); b }

    #[test]
    fn interning_is_pair_identity_and_zero_has_no_entry() {
        let mut t = HandleTable::new().unwrap();
        assert_eq!(t.fixup(0, desc(1)), Ok(None));
        let a = t.fixup(77, desc(1)).unwrap().unwrap();
        assert_eq!(t.fixup(77, desc(1)), Ok(Some(a)));
        assert_ne!(t.fixup(77, desc(2)).unwrap(), Some(a));
        assert_ne!(t.fixup(78, desc(1)).unwrap(), Some(a));
        assert_eq!(t.len(), 3);
        assert_eq!(t.fixup(79, Descriptor { identity: DescriptorId(1), type_id: 5 }), Err(Error::DescriptorIdentityConflict));
        assert_eq!(t.len(), 3);
    }
    #[test]
    fn caller_selects_both_resources_and_id_comes_from_bytes() {
        let mut t=HandleTable::new().unwrap(); let g=current(0xcbcd_1287); let mut c=[0; CELL_BYTES]; c[..4].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        let candidate=prepare_start(&mut t, globals(&g), cell(0xcbcd_1287,&c), desc(12)).unwrap();
        assert_eq!(candidate.cell_key, NativeResourceKey { instance_id:0xcbcd_1287, type_id:CELL_TYPE, group_id:0 });
        assert_eq!(t.get(candidate.handle).unwrap(), Entry { id:0xcbcd_1287, descriptor:desc(12) });
        assert_eq!((candidate.globals_provenance,candidate.cell_provenance), ("fixture globals","fixture cell"));
        assert_eq!(candidate.cell_definition().structure_id(), 0x1234_5678);
    }
    #[test]
    fn unavailable_ambiguous_fallback_and_bad_key_do_not_intern() {
        let g=current(42); let c=[0;CELL_BYTES]; let mut t=HandleTable::new().unwrap();
        for choice in [Resolution::Unavailable,Resolution::Ambiguous,Resolution::FallbackUnresolved] {
            assert!(prepare_start(&mut t,globals(&g),choice,desc(3)).is_err()); assert!(t.is_empty());
        }
        assert_eq!(prepare_start(&mut t,globals(&g),cell(43,&c),desc(3)),Err(Error::BadCellKey));
        assert!(t.is_empty());
    }
    #[test]
    fn canonical_scope_is_checked_and_bytes_do_not_authenticate_provenance() {
        let g=current(42); let c=[0;CELL_BYTES]; let mut t=HandleTable::new().unwrap();
        let wrong_scope = Resolution::Supplied(SuppliedResource {
            key: NativeResourceKey { instance_id:42, type_id:CELL_TYPE, group_id:0x0001_ffff },
            bytes:&c, provenance:"asserted package label",
        });
        assert_eq!(prepare_start(&mut t,globals(&g),wrong_scope,desc(3)),Err(Error::BadCellKey));
        assert!(t.is_empty());
    }
    #[test]
    fn null_sentinel_and_unsupported_layout_fail_before_interning() {
        let c=[0;CELL_BYTES]; let mut t=HandleTable::new().unwrap();
        assert_eq!(prepare_start(&mut t,globals(&current(0)),cell(0,&c),desc(3)),Err(Error::NullStartCell));
        assert_eq!(prepare_start(&mut t,globals(&current(u32::MAX)),cell(u32::MAX,&c),desc(3)),Err(Error::FallbackUnresolved));
        assert_eq!(prepare_start(&mut t,globals(&[0;264]),cell(1,&c),desc(3)),Err(Error::GlobalsLayout));
        assert_eq!(prepare_start(&mut t,globals(&current(1)),cell(1,&[0;CELL_BYTES-1]),desc(3)),Err(Error::CellLayout));
        assert!(t.is_empty());
    }
    #[test]
    fn handles_cannot_cross_tables_or_resolve_after_issuer_is_dropped() {
        let mut first = HandleTable::new().unwrap();
        let old = first.fixup(12, desc(1)).unwrap().unwrap();
        let mut second = HandleTable::new().unwrap();
        let other = second.fixup(99, desc(2)).unwrap().unwrap();
        assert_eq!(first.get(other), None);
        assert_eq!(second.get(old), None);
        drop(first);
        let mut replacement = HandleTable::new().unwrap();
        replacement.fixup(12, desc(1)).unwrap();
        assert_eq!(replacement.get(old), None);
    }
    /// Read only the explicitly named local PatchData package. This is not an
    /// overlay search or evidence that the original game selects this copy.
    /// Precondition: the package is stable and is not concurrently modified or
    /// replaced throughout the test. Hashes before/after cannot rule out a
    /// transient replacement, because the index reader reopens the path.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    #[ignore = "requires a local SPORE install; explicit PatchData fixture only"]
    fn explicit_patchdata_candidate_uses_canonical_keys() {
        use dbpf::native_index::NativePackageIndex;
        use std::{io::Read, path::{Path, PathBuf}};
        use sha2::{Digest, Sha256};
        fn file_hash(path: &Path) -> String {
            let mut file = std::fs::File::open(path).expect("open selected package for hashing");
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let count = file.read(&mut buffer).expect("read selected package for hashing");
                if count == 0 { break; }
                hasher.update(&buffer[..count]);
            }
            format!("{:x}", hasher.finalize())
        }
        let root = std::env::var_os("RUSTSPORE_TEST_INSTALL")
            .map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"D:\Games\SPORE Collection"));
        let path = root.join("Data").join("PatchData.package");
        const PACKAGE_SHA256: &str = "cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff";
        let before = file_hash(&path);
        assert_eq!(before, PACKAGE_SHA256, "selected package differs from fixture snapshot");
        let package = NativePackageIndex::open(&path).expect("open explicitly selected PatchData");
        let globals_key = NativeResourceKey { instance_id: GLOBALS_ID, type_id: GLOBALS_TYPE, group_id: 0 };
        let cell_key = NativeResourceKey { instance_id: 0xcbcd_1287, type_id: CELL_TYPE, group_id: 0 };
        let list_key = NativeResourceKey { instance_id: 0x9a39_1385, type_id: 0xf9c3_d770, group_id: 0 };
        let read = |key: NativeResourceKey| {
            let mut matches = package.entries().iter().filter(|entry| entry.key == key);
            let entry = matches.next().expect("exact scope-zero key absent");
            assert!(matches.next().is_none(), "duplicate exact resource key");
            package.read(entry).expect("decode selected resource")
        };
        let globals = read(globals_key);
        let cell = read(cell_key);
        let list = read(list_key);
        assert_eq!(file_hash(&path), before, "selected package changed across extraction");
        assert_eq!(format!("{:x}", Sha256::digest(&globals)),
            "e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72");
        assert_eq!(format!("{:x}", Sha256::digest(&cell)),
            "f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542");
        assert_eq!(format!("{:x}", Sha256::digest(&list)),
            "29b0f195b26bf2480586518e385933284819ff360a991ea706a939717a5d5f8a");
        assert_eq!(cell_globals::parse(&globals).unwrap().start_cell(), 0xcbcd_1287);
        let mut table = HandleTable::new().unwrap();
        // Symbolic descriptor identity and package label are explicit fixture
        // admissions, not captured native pointer or winner provenance.
        let descriptor = desc(0x016a_e490);
        let result = prepare_start(&mut table,
            Resolution::Supplied(SuppliedResource { key: globals_key, bytes: &globals, provenance: "explicit PatchData" }),
            Resolution::Supplied(SuppliedResource { key: cell_key, bytes: &cell, provenance: "explicit PatchData" }),
            descriptor).unwrap();
        assert_eq!(result.cell_key, cell_key);
        assert_eq!(result.globals_provenance, "explicit PatchData");
        assert_eq!(result.cell_provenance, "explicit PatchData");
        assert_eq!(table.len(), 1);
        assert_eq!(table.get(result.handle), Some(Entry { id: 0xcbcd_1287, descriptor }));
        assert_eq!(result.cell_definition().structure_id(), 0xcbcd_1287);
        // Explicit synthetic runtime inputs: these do not assert a live actor,
        // original FP environment, native RNG seed, or package selection.
        let size = spore_core::cell_start_equal_size_v1::candidate_start_size(
            spore_core::cell_start_equal_size_v1::AdmittedCellKey {
                instance: cell_key.instance_id, type_id: cell_key.type_id, scope: cell_key.group_id,
            },
            result.cell_definition(), cell_globals::parse(&globals).unwrap(),
            1.0, 1.0, 1, 0x027f,
        ).expect("bounded size transition with explicit synthetic inputs");
        assert_eq!(size.sampled_bits, 0x3e8c_cccd);
        assert_eq!(size.actor_scale_bits, 0x3e8c_cccd);
        assert_eq!(size.rng_state_after, 0x278d_de6d);
        // This direct fixture handoff assumes no intervening RNG changes. The
        // actual allocation, intervening effects, runtime global/axis values
        // and original floating-point state are not established here.
        let orientation = spore_core::cell_start_null_orientation_v1::null_orientation(
            size.rng_state_after, 0x40c9_0fdb, [0, 0, 0x3f80_0000], 0x027f,
            spore_core::native_sse::Policy::Mxcsr1f80,
        ).expect("bounded null-orientation fixture");
        assert_eq!(orientation.rng_state_after, 0xfb53_3a69);
        assert_eq!(orientation.quaternion_bits[0], 0);
        assert_eq!(orientation.quaternion_bits[1], 0);
        assert!(orientation.quaternion_bits.iter().all(|&bits| f32::from_bits(bits).is_finite()));
        assert_eq!(orientation.quaternion_bits[2], orientation.sine_bits);
        assert_eq!(orientation.quaternion_bits[3], orientation.cosine_bits);
        // This admits the parsed disk rows as a runtime snapshot for this
        // fixture only. Cache, structure handle, RNG, control and third word
        // are synthetic and do not establish an original source selection.
        let rows = spore_core::cell_kind4_list_disk_v1::parse(&list).expect("selected source list");
        let mut cache = spore_core::cell_kind4_cache_selection_v1::Cache::default();
        let selected = spore_core::cell_kind4_cache_selection_v1::select(
            &mut cache, 0x42, Some(&rows), 1, 0x027f, 7,
            spore_core::cell_kind4_cache_selection_v1::SnapshotAdmission,
        );
        assert_eq!(selected, spore_core::cell_kind4_cache_selection_v1::ResultV1::Selected {
            triple: [0xe219_e5d0, 0x3d97_a8e4, 7], selected: 1,
            rng_state: 0xd369_bf11, draws: 4,
        });
        // A separate fresh cache chains the orientation state only under the
        // fixture assumption of no intervening RNG mutation. Neither the
        // actor nor the original cache or source selection is established.
        let mut chained_cache = spore_core::cell_kind4_cache_selection_v1::Cache::default();
        let chained = spore_core::cell_kind4_cache_selection_v1::select(
            &mut chained_cache, 0x42, Some(&rows), orientation.rng_state_after, 0x027f, 7,
            spore_core::cell_kind4_cache_selection_v1::SnapshotAdmission,
        );
        assert_eq!(chained, spore_core::cell_kind4_cache_selection_v1::ResultV1::Selected {
            triple: [0xe219_e5d3, 0x3d97_a8e4, 7], selected: 2,
            rng_state: 0x492c_37f9, draws: 4,
        });
        let other = HandleTable::new().unwrap();
        assert_eq!(other.get(result.handle), None);
    }

    #[test]
    fn table_capacity_rejects_new_but_existing_entry_still_resolves() {
        let mut t=HandleTable::new().unwrap();
        for i in 1..=MAX_HANDLES { t.fixup(i as u32,desc(1)).unwrap(); }
        assert!(t.fixup(1,desc(1)).unwrap().is_some());
        assert_eq!(t.fixup(1000,desc(1)),Err(Error::HandleCapacity));
        assert_eq!(t.len(),MAX_HANDLES);
    }
}
