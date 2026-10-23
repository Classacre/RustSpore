//! Local evidence protocol, shared only by the optional-foot verifier examples.
use spore_core::{anim_body_capabilities::CapabilitySlots,
    anim_body_optional_feet::FootConstructionBlock};
pub const RECORD_BYTES:usize=316;
pub const INPUT_BYTES:usize=16+255*RECORD_BYTES;
pub fn word(data:&[u8],at:usize)->u32 { u32::from_le_bytes(data[at..at+4].try_into().unwrap()) }
pub fn decode(data:&[u8])->FootConstructionBlock {
    assert_eq!(data.len(),RECORD_BYTES);
    let mut capabilities=CapabilitySlots::default();
    for i in 0..32 { capabilities.ids[i]=word(data,8+i*4);capabilities.numbers[i]=data[136+i]; }
    let w: [u32;37]=std::array::from_fn(|i|word(data,168+i*4));
    let optional=|i:usize|if w[i]==u32::MAX { None } else { Some(w[i] as usize) };
    let xyz=|i:usize|std::array::from_fn(|j|f32::from_bits(w[i+j]));
    let xyzw=|i:usize|std::array::from_fn(|j|f32::from_bits(w[i+j]));
    FootConstructionBlock { capabilities,presence_mask:word(data,0),nonzero_mask:word(data,4),
        part_instance:w[0],part_scope:w[1],ordinal:w[2],parent:optional(3),symmetric:optional(4),
        source_palette_id:w[5],child_count:w[6],center:xyz(7),absolute_orientation:xyzw(10),
        local_orientation:xyzw(14),half_extents:xyz(18),local_origin:xyz(21),child_anchor:xyz(24),
        parent_anchor:xyz(27),parent_delta:xyz(30),parent_delta_length:f32::from_bits(w[33]),
        field36c:f32::from_bits(w[34]),source_scale:f32::from_bits(w[35]),field37c:f32::from_bits(w[36]) }
}
pub fn encode(b:&FootConstructionBlock)->[u8;RECORD_BYTES] {
    let mut out=[0u8;RECORD_BYTES];
    out[..4].copy_from_slice(&b.presence_mask.to_le_bytes());out[4..8].copy_from_slice(&b.nonzero_mask.to_le_bytes());
    for i in 0..32 { out[8+i*4..12+i*4].copy_from_slice(&b.capabilities.ids[i].to_le_bytes());out[136+i]=b.capabilities.numbers[i]; }
    let mut words=[0u32;37];
    words[..7].copy_from_slice(&[b.part_instance,b.part_scope,b.ordinal,b.parent.map_or(u32::MAX,|v|v as u32),
        b.symmetric.map_or(u32::MAX,|v|v as u32),b.source_palette_id,b.child_count]);
    for (start,values) in [(7,&b.center[..]),(10,&b.absolute_orientation[..]),(14,&b.local_orientation[..]),
        (18,&b.half_extents[..]),(21,&b.local_origin[..]),(24,&b.child_anchor[..]),(27,&b.parent_anchor[..]),(30,&b.parent_delta[..])] {
        for (j,v) in values.iter().enumerate() { words[start+j]=v.to_bits(); }
    }
    words[33..].copy_from_slice(&[b.parent_delta_length.to_bits(),b.field36c.to_bits(),b.source_scale.to_bits(),b.field37c.to_bits()]);
    for (i,w) in words.into_iter().enumerate() { out[168+i*4..172+i*4].copy_from_slice(&w.to_le_bytes()); }
    out
}
