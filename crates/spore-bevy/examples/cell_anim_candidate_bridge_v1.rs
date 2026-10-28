//! Explicit, offline source candidate bridge. This does not select an actor,
//! determine package precedence, install a controller, or play an animation.
//!
//! Arguments: TABLE LIST_PACKAGE LIST_ORDINAL ANIM_PACKAGE ANIM_ORDINAL
//! ROW RECORD VARIANT ENTRY EXPECTED_ANIM FIXTURE_INPUT CASE NATIVE_REPLY
//!
//! All inputs and their claimed provenance are caller supplied and unverified.
//! TABLE must have the 131-row layout; EXPECTED_ANIM is compared byte for byte
//! but is not independently authenticated. FIXTURE_INPUT and NATIVE_REPLY
//! may be the retained builder cohort's paired files. CASE is zero-based.
use dbpf::native_index::NativePackageIndex;
use spore_core::{anim_body_capabilities::CapabilitySlots, anim_nonroot_selector::FinalizedSelectorBlock,
    anim_nonroot_selector_z::DefinitionZ, anim_selector_z_bounds::{self, BoundsBlock},
    anim_ungrouped_binding_builder as builder, animlist_v10, cell_anim_source_crosslink_v1 as link,
    cell_anim_state_table_v1, native_x87::Context};
use std::{error::Error, fs, io::{Cursor, Read, Seek, SeekFrom}, path::Path};

const FIELDS: [usize; 39] = [0x104,0x108,0x1f8,0x1fc,0x200,0x21c,0x208,0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x128,0x12c,0x130,0x134,0x138,0x13c,0x140,0x144,0x148,0x14c,0x344,0x348,0x34c,0x350,0x354,0x358,0x35c,0x360,0x364,0x368,0x36c,0x378,0x37c,0x204,0x20c];
const MODEL: usize = 114284;
const REPLY: usize = MODEL + 8;
const MAX_RESOURCE: u64 = 16 * 1024 * 1024;
const MAX_FIXTURE: u64 = 64 * 1024 * 1024;
const MAX_INDEX: u32 = 64 * 1024 * 1024;
fn word(b: &[u8], o: usize) -> u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn field(b: &[u8], o: usize) -> u32 { word(b,168 + FIELDS.iter().position(|x| *x == o).unwrap()*4) }
fn span(b: &[u8], start: usize, len: usize) -> Result<&[u8], Box<dyn Error>> {
    let end=start.checked_add(len).ok_or("ANIM extent overflow")?;
    b.get(start..end).ok_or_else(|| "ANIM extent outside payload".into())
}
/// Bounded channel/descriptor projection only; this does not validate all ANIM
/// semantics or sample keyframes. No decoded keyframe arrays are allocated.
fn channel_metadata(b: &[u8]) -> Result<Vec<builder::Channel>, Box<dyn Error>> {
    if b.len()<512 || &b[..4]!=b"ANIM" || word(b,4) as usize!=b.len() { return Err("invalid ANIM header or extent".into()); }
    let count=word(b,324) as usize;
    if !(1..=14).contains(&count) { return Err("unsupported ANIM channel count".into()); }
    let table=word(b,328) as usize;
    if table<512 { return Err("unsupported ANIM channel table".into()); }
    let offsets=span(b,table,count*4)?;
    let (mut total_rows,mut total_bytes)=(0usize,0usize); let mut out=Vec::with_capacity(count);
    for i in 0..count {
        let offset=word(offsets,i*4) as usize; let header=span(b,offset,228)?;
        if &header[..4]!=b"CHAN" { return Err("invalid ANIM channel magic".into()); }
        let rows=word(header,212) as usize; let key_offset=word(header,216) as usize;
        let descriptor_count=word(header,220) as usize; let descriptor_offset=word(header,224) as usize;
        if !(1..=24).contains(&descriptor_count) { return Err("unsupported ANIM descriptor count".into()); }
        let desc=span(b,descriptor_offset,descriptor_count*32)?;
        let mut descriptors=Vec::with_capacity(descriptor_count);
        for d in desc.chunks_exact(32) { descriptors.push(std::array::from_fn(|j|word(d,j*4))); }
        let stride=descriptors[0][3] as usize;
        if stride==0 || descriptors.iter().any(|d|d[3] as usize!=stride || d[2] as usize>=stride) { return Err("invalid ANIM descriptor stride or offset".into()); }
        let info=descriptors.iter().find(|d|d[1]==u32::from_le_bytes(*b"INFO")).ok_or("ANIM INFO descriptor missing")?;
        for (tag,width) in [(*b"INFO",20usize),(*b"POS\0",12),(*b"ROT\0",16)] {
            if let Some(d)=descriptors.iter().find(|d|d[1]==u32::from_le_bytes(tag)) {
                if (d[2] as usize).checked_add(width).is_none_or(|end|end>stride) { return Err("ANIM descriptor width exceeds stride".into()); }
            }
        }
        let bytes=rows.checked_mul(stride).ok_or("ANIM keyframe extent overflow")?;
        total_rows=total_rows.checked_add(rows).ok_or("ANIM row count overflow")?;
        total_bytes=total_bytes.checked_add(bytes).ok_or("ANIM keyframe total overflow")?;
        if total_rows>100_000 || total_bytes>MAX_RESOURCE as usize { return Err("ANIM cumulative keyframe limit exceeded".into()); }
        let raw=span(b,key_offset,bytes)?; let mut last=None;
        for row in raw.chunks_exact(stride) {
            let frame=word(row,info[2] as usize);
            if last.is_some_and(|previous|previous>frame) { return Err("ANIM keyframes are unordered".into()); }
            last=Some(frame);
        }
        out.push(builder::Channel { words_88_d0:std::array::from_fn(|j|word(header,136+j*4)), descriptors });
    }
    Ok(out)
}
fn read_exact(r: &mut impl Read, n: usize) -> Result<Vec<u8>, Box<dyn Error>> { let mut b=vec![0;n]; r.read_exact(&mut b)?; Ok(b) }
fn bounded_file(path: &str, max: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut out=Vec::new(); fs::File::open(path)?.take(max+1).read_to_end(&mut out)?;
    if out.len() as u64>max { return Err("supplied file exceeds size limit".into()); } Ok(out)
}
fn bounded_package(path: &str) -> Result<NativePackageIndex, Box<dyn Error>> {
    let mut file=fs::File::open(path)?; let mut raw=[0u8;dbpf::HEADER_SIZE]; file.read_exact(&mut raw)?;
    let header=dbpf::DbpfHeader::parse(&raw)?;
    if header.index_size>MAX_INDEX || header.index_entry_count>2_000_000 { return Err("package index exceeds size limit".into()); }
    // Package files must remain unchanged between this check and the index read.
    Ok(NativePackageIndex::open(Path::new(path))?)
}
fn number(s: &str) -> Result<usize, Box<dyn Error>> { Ok(s.parse()?) }
fn identity(path: &str, ordinal: usize, e: &dbpf::native_index::NativeResourceEntry) -> link::Identity {
    link::Identity { package: path.into(), index_ordinal: ordinal, type_id: e.key.type_id,
        group: e.key.group_id, instance: u64::from(e.key.instance_id) }
}
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len()!=13 { return Err("expected TABLE LIST_PACKAGE LIST_ORDINAL ANIM_PACKAGE ANIM_ORDINAL ROW RECORD VARIANT ENTRY EXPECTED_ANIM FIXTURE_INPUT CASE NATIVE_REPLY".into()); }
    let list_idx=bounded_package(&a[1])?;
    let anim_idx=bounded_package(&a[3])?;
    let lo=number(&a[2])?; let ao=number(&a[4])?;
    let le=list_idx.entries().get(lo).ok_or("list ordinal out of range")?;
    let ae=anim_idx.entries().get(ao).ok_or("ANIM ordinal out of range")?;
    if le.key.type_id!=0x4aeb6bc6 || ae.key.type_id!=0xee17c6ad { return Err("selected resource type mismatch".into()); }
    for e in [le,ae] { if u64::from(e.stored_size)>MAX_RESOURCE || u64::from(e.decompressed_size)>MAX_RESOURCE { return Err("selected resource exceeds size limit".into()); } }
    let list_bytes=list_idx.read(le)?; let anim_bytes=anim_idx.read(ae)?;
    if anim_bytes!=bounded_file(&a[9],MAX_RESOURCE)? { return Err("ANIM payload differs from caller-supplied comparison bytes".into()); }
    let rows=cell_anim_state_table_v1::parse_table(&bounded_file(&a[0],cell_anim_state_table_v1::TABLE_BYTES as u64)?).ok_or("invalid table extent")?;
    let list=animlist_v10::parse(&list_bytes,Default::default()).map_err(|e|format!("AnimList: {e:?}"))?;
    let lists=[link::List { identity: identity(&a[1],lo,le), records:link::Records::V10(&list.records) }];
    let anims=[identity(&a[3],ao,ae)];
    let links=link::crosslink(&rows,&lists,&anims,link::Limits { rows:131, records:100_000, entries:100_000, anim_matches:100_000 }).map_err(|e|format!("crosslink: {e:?}"))?;
    let (row,record,variant,entry)=(number(&a[5])?,number(&a[6])?,number(&a[7])?,number(&a[8])?);
    let selected=links.get(row).and_then(|r|r.records.iter().find(|r|r.list_index==0 && r.record_index==record))
        .and_then(|r|r.entries.iter().find(|e|e.variant==variant && e.entry==entry)).ok_or("selected table/list entry does not join")?;
    if !selected.anim_indexes.contains(&0) { return Err("selected ANIM identity does not match entry instance".into()); }
    let channels=channel_metadata(&anim_bytes)?;
    let case=number(&a[11])?; if case>100_000 { return Err("fixture case exceeds limit".into()); }
    let input=bounded_file(&a[10],MAX_FIXTURE)?; let mut cursor=Cursor::new(input);
    for current in 0..=case {
        let h=read_exact(&mut cursor,32)?; if word(&h,0)!=0x31444c42 || word(&h,12)!=0x27f || word(&h,28)!=0 { return Err("invalid fixture header".into()); }
        let n=word(&h,4) as usize; let c=word(&h,8) as usize;
        if !(1..=255).contains(&n) || !(1..=14).contains(&c) { return Err("fixture dimensions outside supported domain".into()); }
        let mut blocks=Vec::new(); let mut boxes=Vec::new();
        for _ in 0..n {
            let b=read_exact(&mut cursor,324)?; let mut caps=CapabilitySlots::default();
            for i in 0..32 { caps.ids[i]=word(&b,8+i*4); caps.numbers[i]=b[136+i]; }
            let parent=field(&b,0x1fc);
            blocks.push(FinalizedSelectorBlock { ordinal:field(&b,0x1f8),parent:if parent==u32::MAX {None} else {Some(parent as usize)},source_palette_id:field(&b,0x21c),presence_mask:word(&b,0),nonzero_mask:word(&b,4),capabilities:caps,center:std::array::from_fn(|i|f32::from_bits(field(&b,0x10c+i*4))),half_extents:std::array::from_fn(|i|f32::from_bits(field(&b,0x138+i*4))) });
            boxes.push(BoundsBlock { center:std::array::from_fn(|i|field(&b,0x10c+i*4)),orientation:std::array::from_fn(|i|field(&b,0x118+i*4)),half_extents:std::array::from_fn(|i|field(&b,0x138+i*4)) });
        }
        let mut fixture_channels=Vec::new(); let mut descriptors=0usize;
        for _ in 0..c {
            let b=read_exact(&mut cursor,80)?; let count=word(&b,76) as usize;
            if !(1..=24).contains(&count) { return Err("fixture descriptor count outside supported domain".into()); }
            descriptors+=count; let mut ds=Vec::new();
            for _ in 0..count { let d=read_exact(&mut cursor,32)?; ds.push(std::array::from_fn(|i|word(&d,i*4))); }
            fixture_channels.push(builder::Channel { words_88_d0:std::array::from_fn(|i|word(&b,i*4)), descriptors:ds });
        }
        if descriptors!=word(&h,24) as usize { return Err("fixture descriptor total mismatch".into()); }
        if current!=case { continue; }
        if channels!=fixture_channels { return Err("parsed ANIM channels do not exactly match selected fixture case".into()); }
        let bounds=anim_selector_z_bounds::publish(&boxes,Context::from_control_word(0x27f)?).map_err(|e|format!("bounds: {e:?}"))?;
        let z=DefinitionZ::from_published_bounds(&bounds,word(&h,16)).map_err(|e|format!("bounds projection: {e:?}"))?;
        let result=builder::build(&blocks,&channels,z).map_err(|e|format!("unsupported candidate builder input: {e:?}"))?;
        let mut projection=vec![0xff;MODEL];
        for (i,w) in [word(&h,16),word(&h,20),1,result.records.len() as u32].into_iter().chain(result.fields).enumerate() { projection[i*4..i*4+4].copy_from_slice(&w.to_le_bytes()); }
        for (i,r) in result.records.iter().enumerate() { for (j,w) in r.iter().enumerate() { let p=44+i*16+j*4; projection[p..p+4].copy_from_slice(&w.to_le_bytes()); } }
        let mut reply_file=fs::File::open(&a[12])?; reply_file.seek(SeekFrom::Start((case as u64).checked_mul(REPLY as u64).ok_or("reply offset overflow")?))?;
        let reply=read_exact(&mut reply_file,REPLY)?;
        if u16::from_le_bytes([reply[MODEL],reply[MODEL+1]])!=0x27f || u16::from_le_bytes([reply[MODEL+2],reply[MODEL+3]])&0x3800!=0 || word(&reply,MODEL+4)&!0x3f!=0x1f80 { return Err("supplied reply control state mismatch".into()); }
        if projection!=reply[..MODEL] { return Err("builder projection differs from supplied reply".into()); }
        println!("CANDIDATE row={row} request={:08X} list_package={:?} list_ordinal={lo} list_key={} record={record} variant={variant} entry={entry} anim_package={:?} anim_ordinal={ao} anim_key={} supplied_header_word_16={} supplied_header_word_20={} case={case} channels={} records={} supplied_comparison_bytes_equal=true supplied_reply_projection_equal=true input_authenticity=unverified runtime_selection=false playback=false",rows[row].request_key(),a[1],le.key,a[3],ae.key,word(&h,16),word(&h,20),channels.len(),result.records.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset+4].copy_from_slice(&value.to_le_bytes());
    }
    fn minimal() -> Vec<u8> {
        const CHANNEL: usize=516;
        const DESCRIPTOR: usize=CHANNEL+228;
        const KEYS: usize=DESCRIPTOR+32;
        let mut bytes=vec![0;KEYS+20];
        bytes[..4].copy_from_slice(b"ANIM");
        let len=bytes.len() as u32; put(&mut bytes,4,len);
        put(&mut bytes,324,1); put(&mut bytes,328,512); put(&mut bytes,512,CHANNEL as u32);
        bytes[CHANNEL..CHANNEL+4].copy_from_slice(b"CHAN");
        put(&mut bytes,CHANNEL+136,0x1234); put(&mut bytes,CHANNEL+212,1);
        put(&mut bytes,CHANNEL+216,KEYS as u32); put(&mut bytes,CHANNEL+220,1);
        put(&mut bytes,CHANNEL+224,DESCRIPTOR as u32);
        bytes[DESCRIPTOR+4..DESCRIPTOR+8].copy_from_slice(b"INFO");
        put(&mut bytes,DESCRIPTOR+12,20);
        bytes
    }
    #[test]
    fn accepts_minimal_bounded_metadata() {
        let channels=channel_metadata(&minimal()).unwrap();
        assert_eq!(channels.len(),1);
        assert_eq!(channels[0].words_88_d0[0],0x1234);
        assert_eq!(channels[0].descriptors[0],[0,u32::from_le_bytes(*b"INFO"),0,20,0,0,0,0]);
    }
    #[test]
    fn rejects_oversized_channel_count() {
        let mut bytes=minimal(); put(&mut bytes,324,15);
        assert!(channel_metadata(&bytes).is_err());
    }
    #[test]
    fn rejects_truncated_table_channel_descriptor_and_key_spans() {
        for (offset,value) in [(328, u32::MAX),(512, u32::MAX),
            (516+224, u32::MAX),(516+216, u32::MAX)] {
            let mut bytes=minimal(); put(&mut bytes,offset,value);
            assert!(channel_metadata(&bytes).is_err(),"offset {offset}");
        }
    }
}
