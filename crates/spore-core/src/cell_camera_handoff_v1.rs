//! Bounded E60D03..E60DEB vector handoff with scripted callbacks.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Unadmitted, UnsupportedControl, UnsupportedFloat }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handoff {
    pub first: [u32; 3],
    pub second: [u32; 3],
    pub up: [u32; 3],
    /// Source setter arguments, in call order, before the matrix setter.
    pub setter_words: [u32; 3],
    pub matrix_setter_pending: bool,
    pub auxiliary_update_pending: bool,
}

/// Projects the source SSE additions with the indicated operand order.
///
/// The caller must admit stable disjoint frame and camera storage, a returning
/// scripted getter and matrix helper, and returning setters. The up vector is
/// an explicit current snapshot. Inputs and intermediate results must be finite
/// normal binary32 values or zeros, with active MXCSR exactly 1F80. This does
/// not calculate the matrix, execute a setter, or identify a live camera.
pub fn project(a: [u32; 3], b: [u32; 3], c: [u32; 3], up: [u32; 3], mxcsr: u32, admitted: bool) -> Result<Handoff, Error> {
    if !admitted { return Err(Error::Unadmitted); }
    if mxcsr != 0x1f80 { return Err(Error::UnsupportedControl); }
    fn valid(x: u32) -> bool {
        let e = x & 0x7f80_0000;
        e != 0x7f80_0000 && (e != 0 || x & 0x007f_ffff == 0)
    }
    fn add(x: u32, y: u32) -> Result<u32, Error> {
        if !valid(x) || !valid(y) { return Err(Error::UnsupportedFloat); }
        let z = (f32::from_bits(x) + f32::from_bits(y)).to_bits();
        if !valid(z) { return Err(Error::UnsupportedFloat); }
        Ok(z)
    }
    if !up.iter().all(|&x| valid(x)) { return Err(Error::UnsupportedFloat); }
    let first = [add(b[0],a[0])?, add(a[1],b[1])?, add(a[2],b[2])?];
    let second = [add(add(c[0],b[0])?,a[0])?, add(add(b[1],c[1])?,a[1])?, add(add(b[2],c[2])?,a[2])?];
    Ok(Handoff { first, second, up, setter_words: [0x4000_0000,0x44fa_0000,0x4248_0000], matrix_setter_pending: true, auxiliary_update_pending: true })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_and_admitted() {
        let up=[0,1.0f32.to_bits(),0];
        let a=[1e20f32.to_bits();3]; let b=[(-1e20f32).to_bits();3];
        let c=[3.0f32.to_bits(),5.0f32.to_bits(),7.0f32.to_bits()];
        let x=project(a,b,c,up,0x1f80,true).unwrap();
        assert_eq!(x.first,[0;3]); assert_eq!(x.second,[0;3]);
        assert_eq!(x.setter_words,[0x4000_0000,0x44fa_0000,0x4248_0000]);
        assert_eq!(project(a,b,c,up,0x1f80,false),Err(Error::Unadmitted));
        assert_eq!(project(a,b,c,up,0,true),Err(Error::UnsupportedControl));
        assert_eq!(project([1,0,0],b,c,up,0x1f80,true),Err(Error::UnsupportedFloat));
        assert_eq!(project([0x7f7f_ffff;3],[0x7f7f_ffff;3],c,up,0x1f80,true),Err(Error::UnsupportedFloat));
        let z=project([0x8000_0000,0,0x8000_0000],[0,0x8000_0000,0x8000_0000],[0x8000_0000,0,0x8000_0000],up,0x1f80,true).unwrap();
        assert_eq!(z.first,[0,0,0x8000_0000]); assert_eq!(z.second,[0,0,0x8000_0000]);
    }
}
