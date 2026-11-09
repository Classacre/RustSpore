use std::io::Write;
use spore_core::{cell_fresh_sweep_registration_v1::{Endpoint,Slot},cell_sweep_remove_v1::Storage,cell_e780_manager_handoff_v1::apply};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let cases=[([0,0,1,1],2,2,u32::MAX),([0x8000,0x8000,0x8001,0x8001],2,2,u32::MAX),([0,0x8002,0x8001,3],1,2,1),([0x8003,2,0x8000,0x8001],2,2,u32::MAX),([0,0,1,1],2,2,u32::MAX)];let mut out=Vec::new();
 for (i,(links,live,high,head)) in cases.into_iter().enumerate(){
  let slots=vec![Slot{actor:0x12345678,tag:0x87654321,endpoint_links:links},Slot{actor:if head==1{u32::MAX}else{0x55667788},tag:if head==1{u32::MAX}else{0x11223344},endpoint_links:[0xbbcc,0x99aa,0xff00,0xddee]}];
  let arr=|bank:usize|(0..4).map(|j|Endpoint{opening:j as u16,slot:(0x1100+bank) as u16,value_bits:0x3f000000+bank as u32*0x10000+j as u32}).collect::<Vec<_>>();
  let mut state=Storage{slots,old:[arr(0),arr(1)],pending:[arr(2),arr(3)],live,high,free_head:head};
  if i==4{apply(u32::MAX,true,false,None)}else{apply(0,true,true,Some(&mut state))}.map_err(|e|format!("handoff: {e:?}"))?;
  for x in [state.live,state.high,state.free_head]{out.extend(x.to_le_bytes())}
  for slot in state.slots{out.extend(slot.actor.to_le_bytes());out.extend(slot.tag.to_le_bytes());for x in slot.endpoint_links{out.extend(x.to_le_bytes())}}
  for array in state.old.into_iter().chain(state.pending){for e in array{out.extend(e.opening.to_le_bytes());out.extend(e.slot.to_le_bytes());out.extend(e.value_bits.to_le_bytes())}}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
