//! Bounded translation of original font handler 008AC3C0; not VM admission.
use crate::spui_font_vector_primitives as vector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Coordinates,
    Arithmetic(vector::Error),
}
impl From<vector::Error> for Error {
    fn from(value: vector::Error) -> Self { Self::Arithmetic(value) }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intersection {
    pub point: [i32; 2],
    pub touched: u8,
}

/// Current 26.6 coordinates. Original line and operand order is significant.
/// The coordinate bound is owned policy, not an original resource identity test.
pub fn intersect(
    a0: [i32; 2], a1: [i32; 2], b0: [i32; 2], b1: [i32; 2],
    previous_touch: u8,
) -> Result<Intersection, Error> {
    if [a0,a1,b0,b1].into_iter().flatten().any(|v| v.unsigned_abs()>8192) {
        return Err(Error::Coordinates);
    }
    let ax=a1[0].wrapping_sub(a0[0]);
    let ay=a1[1].wrapping_sub(a0[1]);
    let bx=b1[0].wrapping_sub(b0[0]);
    let by=b1[1].wrapping_sub(b0[1]);
    let touched=previous_touch|3;
    let (numerator,denominator)=if ay==0 {
        if bx==0 { return Ok(Intersection{point:[b0[0],a0[1]],touched}); }
        (b0[1].wrapping_sub(a0[1]),by.wrapping_neg())
    } else if ax==0 {
        if by==0 { return Ok(Intersection{point:[a0[0],b0[1]],touched}); }
        (b0[0].wrapping_sub(a0[0]),bx.wrapping_neg())
    } else if ax.abs()>ay.abs() {
        let slope=vector::div16(ay,ax)?;
        let offset=vector::mul16(b0[0].wrapping_sub(a0[0]),slope)?;
        (b0[1].wrapping_sub(a0[1]).wrapping_sub(offset),
         vector::mul16(bx,slope)?.wrapping_sub(by))
    } else {
        let slope=vector::div16(ax,ay)?;
        let offset=vector::mul16(b0[1].wrapping_sub(a0[1]),slope)?;
        (offset.wrapping_add(a0[0].wrapping_sub(b0[0])),
         bx.wrapping_sub(vector::mul16(by,slope)?))
    };
    let point=if denominator==0 {
        // Source separately shifts each line delta before adding its base.
        [(ax>>1).wrapping_add((bx>>1).wrapping_add(b0[0])).wrapping_add(a0[0])>>1,
         (ay>>1).wrapping_add((by>>1).wrapping_add(b0[1])).wrapping_add(a0[1])>>1]
    } else if numerator.abs()<denominator.abs() {
        let ratio=vector::div16(numerator,denominator)?;
        [b0[0].wrapping_add(vector::mul16(bx,ratio)?),
         b0[1].wrapping_add(vector::mul16(by,ratio)?)]
    } else if numerator!=0 {
        let inverse=vector::div16(denominator,numerator)?;
        [b0[0].wrapping_add(vector::div16(bx,inverse)?),
         b0[1].wrapping_add(vector::div16(by,inverse)?)]
    } else { b0 };
    Ok(Intersection{point,touched})
}
