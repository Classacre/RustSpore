//! Opt-in, source-bound composition of separately recorded E51EC0 spans.
//! No live actor/provider/winner or contiguous original invocation is asserted.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{
    cell_capability_curve::sample_capability_curve,
    cell_capability_geometry::{derive_cell_capability_geometry, CellCapabilityPart},
    cell_capability_scalar::{
        derive_cell_capability_scalars, CellCapabilityScalarContext, CellCapabilityScalarGlobals,
    },
    cell_globals,
    cell_part_classification::{
        classify_cell_part, native_local_property_present, CellPartPropertyList,
        CELL_CLASS_PREDICATES,
    },
    cell_part_geometry::build_cell_part,
    creature_data,
    native_sse::Policy,
    native_x87::Context,
    prop::{self, PropValue},
};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Component, Path, PathBuf},
};
#[path = "../../spore-core/examples/support/cell_classification_fixture.rs"]
mod fixture;
use fixture::{key, source_chains, source_properties};
type R<T> = Result<T, Box<dyn Error>>;
fn digest(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn read_pin(path: &Path, len: usize, sha: &str) -> R<Vec<u8>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() != len as u64 {
        return Err(format!("{}: wrong length", path.display()).into());
    }
    let bytes = fs::read(path)?;
    if bytes.len() != len || digest(&bytes) != sha {
        return Err(format!("{}: hash mismatch", path.display()).into());
    }
    Ok(bytes)
}
// Both inputs include 8 bytes of native FP metadata at 208..216.
fn non_fp_reply_matches(actual: &[u8; 240], expected: &[u8]) -> bool {
    expected.len() == 240
        && actual[..208] == expected[..208]
        && actual[216..240] == expected[216..240]
}
fn request_node_matches(request: &[u8], node: usize, ids: &[u32], parent: Option<usize>) -> bool {
    if request.len() != 1120 || node >= 8 || ids.len() > 32 {
        return false;
    }
    let at = 32 + node * 136;
    word(request, at) as usize == ids.len()
        && word(request, at + 4) == parent.map_or(u32::MAX, |p| p as u32)
        && ids
            .iter()
            .enumerate()
            .all(|(i, &id)| word(request, at + 8 + 4 * i) == id)
}
fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct Mxcsr(u32);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Mxcsr {
    fn enter() -> Self {
        let mut saved = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{p}]", p=in(reg) &mut saved, options(nostack,preserves_flags));
        }
        let value = 0x1f80u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &value, options(nostack,preserves_flags));
        }
        Self(saved)
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Drop for Mxcsr {
    fn drop(&mut self) {
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &self.0, options(nostack,preserves_flags));
        }
    }
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn main() {
    eprintln!("requires x86/x86_64 SSE2");
    std::process::exit(1);
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() -> R<()> {
    if !std::is_x86_feature_detected!("sse2") {
        return Err("SSE2 required".into());
    }
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cell_capability_source_join_v1 <Spore_Content.package> <target/source-cell>"
                .into(),
        );
    }
    let package_path = PathBuf::from(&args[0]);
    let root = PathBuf::from(&args[1]);
    // Explicit precondition: package and fixture files are stable and not concurrently
    // modified or replaced. Before/after hashes cannot detect transient replacement.
    let package_sha = "0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c";
    fn file_sha(path: &Path) -> R<String> {
        use std::io::Read;
        let mut f = fs::File::open(path)?;
        let mut h = Sha256::new();
        let mut b = [0; 65536];
        loop {
            let n = f.read(&mut b)?;
            if n == 0 {
                break;
            }
            h.update(&b[..n]);
        }
        Ok(format!("{:x}", h.finalize()))
    }
    if file_sha(&package_path)? != package_sha {
        return Err("selected package hash differs".into());
    }
    let index = NativePackageIndex::open(&package_path)?;
    let resource = NativeResourceKey {
        instance_id: 0xa1c46df2,
        type_id: 0x0f43029a,
        group_id: 0x40616200,
    };
    let mut matches = index.entries().iter().filter(|e| e.key == resource);
    let entry = matches.next().ok_or("candidate missing")?;
    if matches.next().is_some() {
        return Err("duplicate candidate".into());
    }
    let raw = index.read(entry)?;
    if raw.len() != 1889
        || digest(&raw) != "6852040fa9a9376ed9025f4823e05024a22685a147e00e64d1b0f536b33e8cd8"
        || file_sha(&package_path)? != package_sha
    {
        return Err("candidate/package mismatch".into());
    }
    // The exact pinned reports record zero exit/empty stderr and reviewed host
    // hashes for three separate runs. Hash equality binds those records; it
    // cannot independently prove how a historical run was actually performed.
    let original = package_path
        .parent()
        .and_then(Path::parent)
        .ok_or("install root")?
        .join("SporeBin/SporeApp.exe");
    if file_sha(&original)? != "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a" {
        return Err("source executable mismatch".into());
    }
    read_pin(
        &root.join("part-classification-fixture-v1/report.json"),
        1904,
        "c28d7b7ebb911b057cf21222c9789be9dc9f635a0380ab7c3a9d9270bf30ea39",
    )?;
    read_pin(
        &root.join("part-classification-fixture-v1/run.json"),
        67,
        "b6ba5ae96036f00a09f465be83a6298ae81d11405336f033dbbc26854c820bc4",
    )?;
    read_pin(
        &root.join("part-classification-fixture-v1/stderr.bin"),
        0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )?;
    read_pin(
        &root.join("native-part-classification-independent-review-v1.json"),
        5737,
        "d80a1bb25ef4e8e5aadd298498a0d5cb39d7c2503703a0439c1c999c070c96fc",
    )?;
    read_pin(
        &root.join("native-part-classification-host-v1/host-x86.asm"),
        12102,
        "5d4435e2d34f54416dfa157ee5ab48cf55247c026a945ecac06cefa3b4f78b8a",
    )?;
    read_pin(
        &root.join("native-part-classification-host-v1/host-x86.exe"),
        2767360,
        "624e29e203771827d764739cc5fe0652b78d686dd08e8f10bb576f9c50f871a3",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/report.json"),
        2568,
        "2495445e3dd6838b1f399f73fc8c447bf20a6bf8787bd6dda5c524a0ddd15e79",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/cache-status.json"),
        219,
        "f0a4ebda10b744c5dbae46b92fa227a0d6bb5bef1469eb94e6f4e671dffa3fdb",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/root-review-v1.json"),
        1256,
        "03a150edc0ee489ec47df76d781caf4aecf38fa3dfc8fc9f08946864872f9066",
    )?;
    read_pin(
        &root.join("native-capability-geometry-host-v2/host-x86.asm"),
        15278,
        "44dbd9fddf470a2fbf7a9752c284f72e19dcbdc349c397afcef1e62c79334232",
    )?;
    read_pin(
        &root.join("native-capability-geometry-host-v2/host-x86.exe"),
        14848,
        "7eaa8d9d868327601a55f37da9d34d1791b1d12836b8d98bd700623a4b8e463c",
    )?;
    read_pin(
        &root.join("capability-scalar-fixture-v1/report.json"),
        3780,
        "a0bb4580ff89ebfdb0d067a9f9c1b5d270550c74ff49491f7f9b946705699a89",
    )?;
    read_pin(
        &root.join("capability-scalar-fixture-v1/status.json"),
        157,
        "8df6ab05a45e011fa323262183dfefd60963181f80e5aec0ae472493f66f7178",
    )?;
    read_pin(
        &root.join("capability-scalar-fixture-v1/root-review-v1.json"),
        1370,
        "fbbf8842a2cc09b033d9f887de5055a087de1e5ed5c4093f540e378c4c6b121d",
    )?;
    read_pin(
        &root.join("native-capability-scalar-independent-review-v1.json"),
        7863,
        "4b524f0bd30494fe3e60273f84f4e6902aac11a78fddd65e180b10edf3cbc53e",
    )?;
    read_pin(
        &root.join("native-capability-scalar-host-v1/host-x86.asm"),
        33119,
        "aa2571c9efc37856b62757537a60cbaf45df5540b1e7ecc2784e8bde6950ffda",
    )?;
    read_pin(
        &root.join("native-capability-scalar-host-v1/host-x86.exe"),
        1327616,
        "9f473c6c13b42214b970bbee6042153edd2a5e8be33d760720a0b2e1988d5111",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/cache-stderr.bin"),
        0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )?;
    read_pin(
        &root.join("capability-scalar-fixture-v1/stderr.bin"),
        0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )?;
    // These exact historical records attest the recorded before/after hashes;
    // pinned identity does not independently reproduce the historical run.
    // No equivalent classification-run input immutability record has been
    // located. Classification provenance therefore remains unverified even
    // though its currently retained request, replies and report are pinned.
    read_pin(
        &root.join("capability-source-composition-v1/execution-pins-v1.json"),
        236280,
        "276dc612a046a261d68f00693fdc969bc437c264de52c167330b4a3486537129",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/immutability-v1.json"),
        137,
        "2873164491ef5672cf4584529ea508a9e1ef14343ec02063f0d2bf42133e0325",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/cache-input-immutability.json"),
        202,
        "9cef5a700a3edecb4c53c8646d05f4e8d2a21e12463e488a1c44d1dfe26690fa",
    )?;
    read_pin(
        &root.join("capability-source-composition-v1/cache-input-pin.json"),
        86,
        "0d97666b52413894b69a7140739799d03dffd7ac452fc794f4dff21196764945",
    )?;
    read_pin(
        &root.join("capability-scalar-fixture-v1/execution-pins-v1.json"),
        2670,
        "932f8a5be8d9a7a34450b860d5a681584ed42e62e89a7d035ca7df6047cdecd9",
    )?;
    read_pin(
        &root.join("capability-scalar-fixture-v1/immutability-v1.json"),
        74,
        "d5412adcb126d3b280c893d66ac5cc336c5ecf5dbb705035f825e47f82f67c20",
    )?;
    read_pin(
        &root.join("native-part-classification-host-v1/manifest.json"),
        13840,
        "5b0dea911f417143186096ed46dc0c45e196f7c55c80d3ea1adae2866e737aa6",
    )?;
    read_pin(
        &root.join("native-part-geometry-host-v1/manifest.json"),
        18184,
        "c159582dd825f3b646f1ff3c04de6cdad50fe3a61b42ddded4e3e793f11e8733",
    )?;
    read_pin(
        &root.join("native-capability-geometry-host-v2/manifest.json"),
        28842,
        "14425f8dd8dfbf370bf319072fad254a32897b419fc18484fd2ae925fa13c756",
    )?;
    read_pin(
        &root.join("native-capability-scalar-host-v1/manifest.json"),
        69396,
        "ea8a01c211a04dcf46ca5d6f60bc6d2376115d848391164abadcdad1818c805c",
    )?;
    read_pin(
        &root.join("native-part-geometry-host-v1/host-x86.asm"),
        13876,
        "d708381148c7fb7d833fc84bfb1e96f15d5977853f887a875074a57f06d8abca",
    )?;
    read_pin(
        &root.join("native-part-geometry-host-v1/host-x86.exe"),
        91136,
        "e17531352ba5a6e6e6929617cdc345c7289dbfd6d42b0b34da32b41339e8499e",
    )?;
    let classdir = root.join("part-classification-fixture-v1");
    let table = read_pin(
        &classdir.join("properties.tsv"),
        212479,
        "c7392dbaf0e69848be0a15cfea67bd1641de835b3ccd8ebb77567666c68d9ca1",
    )?;
    read_pin(
        &classdir.join("source-chains.tsv"),
        1420060,
        "a57017304113ed859587e4014e89d22e936d3f600ce44fe51f2649a0b948cb59",
    )?;
    let class_requests = read_pin(
        &classdir.join("input.bin"),
        85050560,
        "5e793c89519d228a84ac71c4bbf6845805449027572614c50ad0b830c5cba91f",
    )?;
    let native_classes = read_pin(
        &classdir.join("output.bin"),
        1215008,
        "c47eeda476dd9a90e3e0c368c4c6917a2479ddd2ebb5603ef8a67567d7283854",
    )?;
    // Pin all data before the existing helper parses it. The table's full hash
    // binds each listed package label/path/hash, not runtime provider selection.
    for line in std::str::from_utf8(&table)?.lines().skip(1) {
        let f: Vec<_> = line.split('\t').collect();
        if f.len() != 8 {
            return Err("property table fields".into());
        }
        let rel = Path::new(f[4]);
        if rel.is_absolute() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
            return Err("unsafe property path".into());
        }
        let p = classdir.join(rel);
        let size = fs::metadata(&p)?.len();
        if size > 16 * 1024 * 1024 {
            return Err("property too large".into());
        }
        read_pin(&p, size as usize, f[5])?;
    }
    let properties = source_properties(&classdir)?;
    let chains = source_chains(&classdir)?;
    let mut classes = BTreeMap::new();
    for (ordinal, chain) in chains.iter().enumerate() {
        let first = properties.get(chain.records[0]).ok_or("missing root")?;
        if (first.low, first.scope) != (chain.low, chain.scope) || chain.control != 0x027f {
            return Err("chain root/context".into());
        }
        let mut lists = Vec::new();
        for (i, &n) in chain.records.iter().enumerate() {
            let p = properties.get(n).ok_or("missing property")?;
            let parent = chain
                .records
                .get(i + 1)
                .map(|&j| properties.get(j).map(|v| key(v.low, v.scope)))
                .transpose_option()
                .ok_or("missing parent")?;
            if p.parent != parent {
                return Err("parent path mismatch".into());
            }
            lists.push(CellPartPropertyList {
                identifiers: &p.identifiers,
                parent: (i + 1 < chain.records.len()).then_some(i + 1),
            });
        }
        let request = &class_requests[ordinal * 1120..(ordinal + 1) * 1120];
        if word(request, 0) as usize != lists.len()
            || word(request, 4) != chain.low
            || word(request, 8) != chain.scope
            || u16::from_le_bytes(request[12..14].try_into()?) != chain.control
            || request[14..32].iter().any(|&b| b != 0)
        {
            return Err("classification request header differs".into());
        }
        for (i, list) in lists.iter().enumerate() {
            if !request_node_matches(request, i, list.identifiers, list.parent) {
                return Err("classification request node differs".into());
            }
        }
        let class = classify_cell_part(&lists, Some(0))?;
        let (mut queries, mut visits) = (0u32, 0u32);
        for (id, _) in CELL_CLASS_PREDICATES {
            queries += 1;
            let mut node = Some(0);
            let mut found = false;
            while let Some(i) = node {
                visits += 1;
                if native_local_property_present(lists[i].identifiers, id)? {
                    found = true;
                    break;
                }
                node = lists[i].parent;
            }
            if found {
                break;
            }
        }
        if word(&native_classes, ordinal * 16 + 8) != queries
            || word(&native_classes, ordinal * 16 + 12) != visits
        {
            return Err("classification reply counters differ".into());
        }
        if word(&native_classes, ordinal * 16) != class
            || word(&native_classes, ordinal * 16 + 4) != 1
        {
            return Err("prior classification differs".into());
        }
        if classes
            .insert(key(chain.low, chain.scope), class)
            .is_some_and(|c| c != class)
        {
            return Err("conflicting candidate classes".into());
        }
    }
    let creature = creature_data::parse(&raw)?;
    if !creature.exact || creature.consumed != raw.len() || creature.parts.len() != 10 {
        return Err("candidate framing".into());
    }
    creature.validate_part_ranges()?;
    let _mxcsr = Mxcsr::enter();
    let context = Context::from_control_word(0x027f)?;
    let mut parts = Vec::new();
    let mut native_rows = Vec::new();
    for i in 0..creature.parts.len().min(64) {
        let view = creature.part_view(i).ok_or("bad part view")?;
        let row: &[u8; 140] = view.raw_row.try_into()?;
        let id = key(word(row, 136), word(row, 132));
        if id != view.part.part_instance {
            return Err("part identity mismatch".into());
        }
        let class = *classes.get(&id).ok_or("missing class")?;
        let part = build_cell_part(row, class, 0xa1c46df2)?;
        let mut native_row = [0xa5; 72];
        part.write_native(&mut native_row);
        native_rows.push(native_row);
        parts.push(CellCapabilityPart::from(part));
    }
    let composition = root.join("capability-source-composition-v1");
    let cache_input = read_pin(
        &composition.join("cache-input.bin"),
        6530160,
        "6c4892174e31f7efb8a5d5e1101d2d2fa3c6ca3f23e1a72e03da7b6e47b93fdf",
    )?;
    let cache_output = read_pin(
        &composition.join("cache-output.bin"),
        196560,
        "a2a9964d1a5532970ed4a68106e472ac53329e3afdb57c51b9e37f4d9849e549",
    )?;
    let req = &cache_input[43 * 4784..44 * 4784];
    let prefix = &cache_output[43 * 144..44 * 144];
    if word(req, 0) != 10
        || u16::from_le_bytes(req[4..6].try_into()?) != 0x027f
        || [word(req, 8), word(req, 12), word(req, 16)] != [0; 3]
        || word(prefix, 0) != 1
    {
        return Err("prefix reference admission".into());
    }
    for (i, row) in native_rows.iter().enumerate() {
        if row[..] != req[32 + i * 72..104 + i * 72] {
            return Err("ordered geometry row differs from prefix input".into());
        }
    }
    let geometry = derive_cell_capability_geometry(&parts, [0.; 3], context)?;
    let mut tail: [u8; 140] = req[4640..4780].try_into()?;
    geometry.write_native_tail(&mut tail);
    if tail != prefix[4..] {
        return Err("prefix reference mismatch".into());
    }
    let scalar = root.join("capability-scalar-fixture-v1");
    let globals = read_pin(
        &scalar.join("globals-0.bin"),
        276,
        "e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72",
    )?;
    let curve = read_pin(
        &scalar.join("curve-0.prop"),
        1124,
        "c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462",
    )?;
    let input = read_pin(
        &scalar.join("input.bin"),
        49818240,
        "57495003b2988c7e20ee6b22554333c7b06f2099a2b7c5e0ee66e39dd1a0e11d",
    )?;
    let output = read_pin(
        &scalar.join("output.bin"),
        8035200,
        "b4fb59ab2c03527ef54ef4a9fb4f968ed775c9eb8871985a15c28d5f98a6e740",
    )?;
    let q = &input[172 * 1488..173 * 1488];
    let expected = &output[172 * 240..173 * 240];
    if q[32..172] != tail
        || word(q, 4) != 0x5a38c4ba
        || word(q, 28) != 1
        || u16::from_le_bytes(q[..2].try_into()?) != 0x027f
    {
        return Err("scalar reference admission".into());
    }
    let g = cell_globals::parse(&globals)?;
    let gs = CellCapabilityScalarGlobals {
        cilia_speed_as_jet: g.cilia_speed_as_jet(),
        flagella_speed_as_jet: g.flagella_speed_as_jet(),
        flagella_speed_as_cilia: g.flagella_speed_as_cilia(),
        cilia_speed_as_flagella: g.cilia_speed_as_flagella(),
        no_part_speed: g.no_part_speed(),
    };
    for (i, b) in [
        gs.cilia_speed_as_jet,
        gs.flagella_speed_as_jet,
        gs.flagella_speed_as_cilia,
        gs.cilia_speed_as_flagella,
        gs.no_part_speed,
    ]
    .iter()
    .enumerate()
    {
        if word(q, 8 + i * 4) != b.to_bits() {
            return Err("Globals reference mismatch".into());
        }
    }
    let tree = prop::parse(&curve)?;
    if !tree.exact || tree.consumed != curve.len() || tree.items.len() != 18 {
        return Err("curve framing".into());
    }
    let mut curves = BTreeMap::new();
    let mut count = 0;
    for item in tree.items {
        if item.anonymous || item.prop_type != 0x30 || item.specifier != 0x9c {
            return Err("curve type".into());
        }
        let mut points = Vec::new();
        for v in item.values {
            match v {
                PropValue::Vector2(x, y) => points.push([x, y]),
                _ => return Err("curve point".into()),
            }
        }
        count += points.len();
        if curves.insert(item.id, points).is_some() {
            return Err("duplicate curve".into());
        }
    }
    if count != 104 {
        return Err("curve point count".into());
    }
    // Missing IDs fail closed before the callback; admitted source copy has all nine.
    let ids = [
        0x77540c34, 0x2c574746, 0xbbab1bf5, 0x2ef68471, 0xe5015d0c, 0x5a38c4ba, 0x03ab589e,
        0xaee2835e, 0x92baae19,
    ];
    for (i, id) in ids.iter().enumerate() {
        let points = curves.get(id).ok_or("missing curve")?;
        let at = 192 + i * 144;
        if word(q, at) != 0 || word(q, at + 4) as usize != points.len() || points.len() > 16 {
            return Err("curve reference framing".into());
        }
        for (j, p) in points.iter().enumerate() {
            if [word(q, at + 16 + j * 8), word(q, at + 20 + j * 8)]
                != [p[0].to_bits(), p[1].to_bits()]
            {
                return Err("curve reference points".into());
            }
        }
    }
    let s = derive_cell_capability_scalars(
        &geometry.class_counts,
        gs,
        0x5a38c4ba,
        CellCapabilityScalarContext {
            x87: context,
            sse: Policy::Mxcsr1f80,
        },
        |id, x, c| {
            sample_capability_curve(
                Some(curves.get(&id).expect("validated scalar curve").as_slice()),
                x,
                c,
            )
        },
    )?;
    s.write_native_tail(&mut tail);
    // Mirror scalar_verify::modeled: compare all 232 non-FP bytes,
    // including untouched A5 query slots and both trailing guard words.
    let mut modeled = [0xa5u8; 240];
    modeled[..140].copy_from_slice(&tail);
    let calls = s.queries().len() as u32;
    let global_calls = if [8, 9, 10]
        .into_iter()
        .any(|i| geometry.class_counts[i] != 0)
    {
        1u32
    } else {
        2u32
    };
    for (at, value) in [
        (140, calls),
        (144, global_calls),
        (148, global_calls),
        (216, global_calls),
        (220, calls),
        (224, calls),
        (228, calls),
        (232, 0),
        (236, 0),
    ] {
        modeled[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (i, v) in s.queries().iter().enumerate() {
        let at = 152 + i * 8;
        modeled[at..at + 4].copy_from_slice(&v.identifier.to_le_bytes());
        modeled[at + 4..at + 8].copy_from_slice(&v.value.to_bits().to_le_bytes());
    }
    if !non_fp_reply_matches(&modeled, expected) {
        return Err("complete non-FP scalar reply mismatch".into());
    }
    if word(expected, 208) & !0x3f != 0x1f80
        || u16::from_le_bytes(expected[212..214].try_into()?) != 0x027f
        || u16::from_le_bytes(expected[214..216].try_into()?) & 0x3800 != 0
    {
        return Err("scalar reference FP context".into());
    }
    println!("source-bound candidate join: 10 ordered rows; retained prefix and scalar references match; classification historical input immutability unverified; fixture admissions only");
    Ok(())
}
trait TransposeOption<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}
impl<T> TransposeOption<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            None => Some(None),
            Some(Some(v)) => Some(Some(v)),
            Some(None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_file_intake_rejects_changed_and_truncated_payloads() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!("rustspore-join-pin-{}", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(b"source").unwrap();
        drop(file);
        let hash = digest(b"source");
        assert_eq!(read_pin(&path, 6, &hash).unwrap(), b"source");
        std::fs::write(&path, b"souRce").unwrap();
        assert!(read_pin(&path, 6, &hash).is_err());
        std::fs::write(&path, b"short").unwrap();
        assert!(read_pin(&path, 6, &hash).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn scalar_comparison_rejects_trace_counter_and_guard_changes_but_excludes_fp_metadata() {
        let a = [0xa5; 240];
        let mut b = a;
        assert!(non_fp_reply_matches(&a, &b));
        for i in [140, 152, 200, 216, 232, 239] {
            b[i] ^= 1;
            assert!(!non_fp_reply_matches(&a, &b));
            b[i] ^= 1;
        }
        b[208] ^= 1;
        assert!(non_fp_reply_matches(&a, &b));
        assert!(!non_fp_reply_matches(&a, &b[..239]));
    }
    #[test]
    fn classification_node_rejects_changed_order_parent_and_extent() {
        let mut b = [0u8; 1120];
        b[32..36].copy_from_slice(&2u32.to_le_bytes());
        b[36..40].copy_from_slice(&u32::MAX.to_le_bytes());
        b[40..44].copy_from_slice(&10u32.to_le_bytes());
        b[44..48].copy_from_slice(&20u32.to_le_bytes());
        assert!(request_node_matches(&b, 0, &[10, 20], None));
        assert!(!request_node_matches(&b, 0, &[20, 10], None));
        assert!(!request_node_matches(&b, 0, &[10, 20], Some(1)));
        assert!(!request_node_matches(&b[..1119], 0, &[10, 20], None));
    }
}
