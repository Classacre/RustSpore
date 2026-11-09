//! Bounded E6A548..E6A590 x87 distance from explicit fresh +90 snapshots.
//! No actor resolution, callbacks or E673A0 execution.
use spore_core::native_x87::{Context,Extended};

pub(super) fn distance(positions:[[u32;3];2],control:u16)->Result<u32,String> {
    if control!=0x027f {return Err("only admitted x87 control 027F is supported".into())}
    let ctx=Context::from_control_word(control).map_err(|e|format!("x87 control: {e:?}"))?;
    let load=|v:u32|Extended::from_f32(f32::from_bits(v)).map_err(|e|format!("position: {e:?}"));
    let sub=|i:usize|->Result<Extended,String>{ctx.sub(load(positions[0][i])?,load(positions[1][i])?).map_err(|e|format!("x87 subtraction: {e:?}"))};
    let x=sub(0)?;let y=sub(1)?;let z=sub(2)?;
    let zz=ctx.mul(z,z).map_err(|e|format!("x87 z square: {e:?}"))?;
    let yy=ctx.mul(y,y).map_err(|e|format!("x87 y square: {e:?}"))?;
    let zy=ctx.add(zz,yy).map_err(|e|format!("x87 z+y: {e:?}"))?;
    let xx=ctx.mul(x,x).map_err(|e|format!("x87 x square: {e:?}"))?;
    let sum=ctx.add(zy,xx).map_err(|e|format!("x87 sum: {e:?}"))?;
    let root=ctx.sqrt(sum).map_err(|e|format!("x87 sqrt: {e:?}"))?;
    Ok(ctx.store_f32(root).map_err(|e|format!("x87 store: {e:?}"))?.to_bits())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn distance_uses_fresh_positions_and_rejects_bad_state() {
        let bits=|v:f32|v.to_bits();
        assert_eq!(distance([[bits(3.),bits(4.),bits(0.)],[bits(0.);3]],0x027f).unwrap(),bits(5.));
        assert_eq!(distance([[bits(-0.),bits(0.),bits(0.)],[bits(0.);3]],0x027f).unwrap(),bits(0.));
        assert_eq!(distance([[1,0,0],[0;3]],0x027f).unwrap(),1);
        assert!(distance([[f32::NAN.to_bits(),0,0],[0;3]],0x027f).is_err());
        assert!(distance([[f32::MAX.to_bits(),0,0],[(-f32::MAX).to_bits(),0,0]],0x027f).is_err());
        assert!(distance([[0;3];2],0x037f).is_err());
    }
}
