//! Pending exact ungrouped217-source-domain binding construction and reduction.
//! Prepared entry fields only; no complete actor or runtime allocation claim.
use crate::{anim_body_finalizer_fields::FinalizedBodyFields,anim_nonroot_selector::{self,FinalizedSelectorBlock},anim_nonroot_selector_z::{self,DefinitionZ},anim_selector_z_bounds::DefinitionBounds,native_sse::{self,Policy,SseError}};
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Channel{/// Exact source words CHAN88..D0, including both selectors and AC/AE/AF.
 pub words_88_d0:[u32;19],/// Exact source32-byte descriptors, preserving ordinal/slot identity.
 pub descriptors:Vec<[u32;8]>}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum Error{ChannelCount(usize),Channel(usize),Descriptor(usize),Selector(anim_nonroot_selector_z::Error),Body(anim_nonroot_selector::Error),Sse(SseError),Capacity}
impl From<anim_nonroot_selector_z::Error>for Error{fn from(e:anim_nonroot_selector_z::Error)->Self{Self::Selector(e)}}
impl From<anim_nonroot_selector::Error>for Error{fn from(e:anim_nonroot_selector::Error)->Self{Self::Body(e)}}
impl From<SseError>for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct ResultFields{pub fields:[u32;7],pub records:Vec<[u32;4]>,pub selected_zero_count:u32,pub empty_fallback_count:u32}
/// Full ordinary constructor domain for exactly source modes0/1, AF0,
/// secondarymode2, no specialtag. This does not drop disabled descriptors:
/// slot/owner admission precedes the enable test and controls empty fallback.
fn construct(channel:&Channel,channel_index:usize,selected:usize,mirror:bool,claims:&mut[u8;9],pass:u32)->Option<([u32;4],bool)>{
 let owner=(channel.words_88_d0[9]>>16)as u8;let mut mask=0u32;let mut admitted=false;let mut flags=if mirror{2u8}else{0};
 for(i,d)in channel.descriptors.iter().enumerate(){let slot=d[4]as usize;if slot>=9||claims[slot]!=255&&claims[slot]!=owner{continue}admitted=true;if d[0]&0x10==0{continue}claims[slot]=owner;mask|=1<<i;if d[0]&0xf==1{flags|=0x20}}
 let fallback=mask==0&&admitted&&channel.words_88_d0[9]&0x8!=0;if mask==0&&!fallback{return None}if fallback{flags|=0x8}
 // Source9A1E00 subsequently replaces the constructor's FFFFFFFF words with
 // the single64-bit pass bit. Initial words are not an admission mask here.
 Some(([1<<pass,0,mask,u32::from_le_bytes([selected as u8,channel_index as u8,255,flags])],fallback))
}
/// Sourcechannel order is the claim priority. Both mirror passes start with
/// independent FF tables. Z-before-hierarchy selection comes from the accepted
/// selector composition; later partitioning uses the original swap order.
pub fn build(blocks:&[FinalizedSelectorBlock],channels:&[Channel],bounds:DefinitionZ)->Result<ResultFields,Error>{
 if channels.is_empty()||channels.len()>14{return Err(Error::ChannelCount(channels.len()))}
 // Invoke the already conformed body/tree/capability/geometry guards without
 // inventing any capability0 results; the sentinel ID never matches.
 anim_nonroot_selector::select(blocks,[0,0,0,0],false,255)?;
 for(i,c)in channels.iter().enumerate(){let ac=c.words_88_d0[9];if !matches!(ac&0xffff,0x1|0x9|0x11)||ac>>24!=0||c.words_88_d0[5..9]!=[2,0,0,0]{return Err(Error::Channel(i))}if c.descriptors.is_empty()||c.descriptors.len()>24{return Err(Error::Descriptor(i))}if c.descriptors.iter().any(|d|d[4]>=9||d[0]&0xf==3&&d[1]==0x70e47545){return Err(Error::Descriptor(i))}}
 native_sse::scoped(Policy::Mxcsr1f80,||{
  let mut records=Vec::new();let(mut selected_zero_count,mut empty_fallback_count)=(0u32,0u32);let(mut live_mask,mut component_mask)=(0u64,0u64);
  for pass in 0..2u32{let mut claims=vec![[255u8;9];blocks.len()];
   for(ci,c)in channels.iter().enumerate(){let primary:[u32;4]=c.words_88_d0[1..5].try_into().unwrap();let selected=if matches!(primary,[1|5,0,0,0]){vec![0usize]}else{anim_nonroot_selector_z::select(blocks,primary,pass!=0,255,bounds)?.into_iter().map(|x|x.runtime_ordinal as usize).collect()};
    for index in selected{let mirror=match c.words_88_d0[9]&0x32{0=>pass!=0,0x10=>blocks[index].center[0]<=0.0,_=>return Err(Error::Channel(ci))};if let Some((row,fallback))=construct(c,ci,index,mirror,&mut claims[index],pass){if records.len()>=7140{return Err(Error::Capacity)}selected_zero_count+=u32::from(index==0);empty_fallback_count+=u32::from(fallback);live_mask|=1u64<<ci;if row[2]&0xffffff!=0{component_mask|=1u64<<ci}records.push(row)}
    }
   }
  }
  // Source first partition (F&1 special count) is empty for this exact mode0/1
  // no-pseudo domain. The second puts C==0 records first using tail searches;
  // it must not be replaced with a stable sort.
  let mut search=selected_zero_count as usize;for i in 0..selected_zero_count as usize{if records[i][3]&0xff!=0{while search<records.len()&&records[search][3]&0xff!=0{search+=1}if search<records.len(){records.swap(i,search)}}}
  let count=if records.is_empty(){1}else{2};Ok(ResultFields{fields:[live_mask as u32,(live_mask>>32)as u32,component_mask as u32,(component_mask>>32)as u32,0,count,1],records,selected_zero_count,empty_fallback_count})
 })
}
pub fn build_finalized(body:&FinalizedBodyFields,channels:&[Channel],bounds:&DefinitionBounds)->Result<ResultFields,Error>{let z=DefinitionZ::from_published_bounds(bounds,0)?;let blocks=body.blocks.iter().map(|b|FinalizedSelectorBlock{ordinal:b.ordinal,parent:b.parent,source_palette_id:b.source_palette_id,presence_mask:b.presence_mask,nonzero_mask:b.nonzero_mask,capabilities:b.capabilities,center:b.center,half_extents:b.half_extents}).collect::<Vec<_>>();build(&blocks,channels,z)}
