//! Checked E60A76 actor lookup and camera-vector interpolation boundary.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Unadmitted, UnsupportedControl, UnreadableSlot, UnsupportedFloat }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot { pub handle:u32, pub position:[u32;3] }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome { pub resolved_index:Option<usize>, pub vector:[u32;3], pub next_provider_pending:bool }

/// Caller supplies stable, disjoint state and readable slots; actor identity,
/// scalar producer and subsequent provider effects are not inferred. The
/// original signed high-water comparison permits an equal-index read, so that
/// slot must actually be present. Float inputs are consumed only on a match.
pub fn project(handle:u32, high_water:i32, slots:&[Slot], old:[u32;3], scalar:u32, mxcsr:u32, admitted:bool)->Result<Outcome,Error> {
    if !admitted { return Err(Error::Unadmitted); }
    if mxcsr!=0x1f80 { return Err(Error::UnsupportedControl); }
    let index=(handle&0xffff) as usize;
    let no_match=|| Ok(Outcome{resolved_index:None,vector:old,next_provider_pending:true});
    if handle==0 || (index as i32)>high_water { return no_match(); }
    let slot=slots.get(index).ok_or(Error::UnreadableSlot)?;
    if slot.handle!=handle { return no_match(); }
    fn valid(x:f32)->bool { x.is_finite() && (x==0.0 || x.is_normal()) }
    fn op(x:f32)->Result<f32,Error> { if valid(x) { Ok(x) } else { Err(Error::UnsupportedFloat) } }
    let o=old.map(f32::from_bits);let p=slot.position.map(f32::from_bits);let k=f32::from_bits(scalar);
    if !o.iter().chain(p.iter()).all(|&x|valid(x)) || !valid(k) { return Err(Error::UnsupportedFloat); }
    let dx=op(p[0]-o[0])?;let dy=op(p[1]-o[1])?;let dz=op(p[2]-o[2])?;
    let px=op(k*dx)?;let py=op(dy*k)?;let pz=op(dz*k)?;
    let result=[op(o[0]+px)?.to_bits(),op(py+o[1])?.to_bits(),op(pz+o[2])?.to_bits()];
    Ok(Outcome{resolved_index:Some(index),vector:result,next_provider_pending:true})
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn lookup_and_admission() {
  let slots=[Slot{handle:0x10000,position:[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()]},Slot{handle:0x10001,position:[0;3]}];
  let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];let k=0.5f32.to_bits();
  assert_eq!(project(0,1,&slots,old,k,0x1f80,true).unwrap().resolved_index,None);
  assert_eq!(project(0x20000,1,&slots,old,k,0x1f80,true).unwrap().resolved_index,None);
  assert_eq!(project(0x10000,1,&slots,old,k,0x1f80,true).unwrap().vector,[3f32.to_bits(),5f32.to_bits(),7f32.to_bits()]);
  assert_eq!(project(0x10001,1,&slots,old,k,0x1f80,true).unwrap().resolved_index,Some(1));
  assert_eq!(project(0x10001,1,&slots[..1],old,k,0x1f80,true),Err(Error::UnreadableSlot));
  assert_eq!(project(0x10001,0,&slots[..1],old,k,0x1f80,true).unwrap().resolved_index,None);
  assert_eq!(project(0x10000,1,&slots,old,k,0,true),Err(Error::UnsupportedControl));
  assert_eq!(project(0x10000,1,&slots,old,k,0x1f80,false),Err(Error::Unadmitted));
  assert_eq!(project(0x10000,1,&slots,old,1,0x1f80,true),Err(Error::UnsupportedFloat));
 }
}
