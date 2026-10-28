//! Unexported original font-vector integer primitives. No VM opcode admission.
//! Exact original truncation/reduction/wrapping policies, bounded owned inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Bounds,
    ZeroDenominator,
}
fn bounded(v: i32) -> Result<u32, Error> {
    if v.unsigned_abs() > (1 << 27) {
        Err(Error::Bounds)
    } else {
        Ok(v.unsigned_abs())
    }
}
///8D0DB0: signed-magnitude16.16; truncate integer and reduced remainder separately.
pub fn div16(a: i32, b: i32) -> Result<i32, Error> {
    let n = bounded(a)?;
    let mut d = bounded(b)?;
    if d == 0 {
        return Err(Error::ZeroDenominator);
    }
    let whole = n / d;
    let mut rem = n % d;
    while rem > 65535 {
        rem >>= 1;
        d >>= 1;
    }
    if d == 0 {
        return Err(Error::ZeroDenominator);
    }
    let v = whole
        .wrapping_shl(16)
        .wrapping_add(rem.wrapping_shl(16) / d) as i32;
    Ok(if (a ^ b) < 0 { v.wrapping_neg() } else { v })
}
///8D0C70: lowDWORD of unsigned magnitude product>>16, then source sign.
pub fn mul16(a: i32, b: i32) -> Result<i32, Error> {
    let n = bounded(a)? as u64;
    let d = bounded(b)? as u64;
    let v = ((n * d) >> 16) as u32 as i32;
    Ok(if (a ^ b) < 0 { v.wrapping_neg() } else { v })
}
///8D0D50: signed2.14 WORD factor; magnitude+2000h rounding before source sign.
pub fn mul14(a: i32, b: i16) -> Result<i32, Error> {
    let n = bounded(a)? as u64;
    let d = (b as i32).unsigned_abs() as u64;
    let v = ((n * d + 8192) >> 14) as u32 as i32;
    Ok(if (a ^ (b as i32)) < 0 {
        v.wrapping_neg()
    } else {
        v
    })
}
///8D0EB0: original three fixed-point Newton updates, not ideal Euclidean sqrt.
pub fn length(x: i32, y: i32) -> Result<i32, Error> {
    if x.unsigned_abs() > (1 << 24) || y.unsigned_abs() > (1 << 24) {
        return Err(Error::Bounds);
    }
    let x = x.abs();
    let y = y.abs();
    if x == 0 {
        return Ok(y);
    }
    if y == 0 {
        return Ok(x);
    }
    let mut guess = x.max(y).wrapping_add(x.min(y) >> 1);
    for _ in 0..3 {
        let a = mul16(x, div16(x, guess)?)?;
        let b = mul16(y, div16(y, guess)?)?;
        guess = b.wrapping_add(a.wrapping_add(guess)).wrapping_add(1) >> 1;
    }
    Ok(guess)
}
///8AC050/8AC100: shift10, normalize16.16, add1/SAR2, retain WORDs.
pub fn line_vector(dx: i32, dy: i32, perpendicular: bool) -> Result<[i16; 2], Error> {
    if dx.unsigned_abs() > 16384 || dy.unsigned_abs() > 16384 {
        return Err(Error::Bounds);
    }
    let mut x = dx.wrapping_shl(10);
    let mut y = dy.wrapping_shl(10);
    let n = length(x, y)?;
    if n != 0 {
        x = div16(x, n)?;
        y = div16(y, n)?;
    }
    let v = [
        (x.wrapping_add(1) >> 2) as i16,
        (y.wrapping_add(1) >> 2) as i16,
    ];
    Ok(if perpendicular {
        [v[1].wrapping_neg(), v[0]]
    } else {
        v
    })
}
///8ABE80: separately rounded component products,WORDsum,near-zero signed clamp.
pub fn dot(projection: [i16; 2], freedom: [i16; 2]) -> u16 {
    let a = (projection[0] as i32 * freedom[0] as i32 + 8192) >> 14;
    let b = (projection[1] as i32 * freedom[1] as i32 + 8192) >> 14;
    let word = a.wrapping_add(b) as u16;
    if word.wrapping_add(1023) <= 2046 {
        if (word as i16) >= 0 { 0x4000 } else { 0xc000 }
    } else {
        word
    }
}
///8AB9D0/8ABA00: each component is rounded separately, then DWORD addition.
pub fn project(delta: [i32; 2], vector: [i16; 2]) -> Result<i32, Error> {
    Ok(mul14(delta[0], vector[0])?.wrapping_add(mul14(delta[1], vector[1])?))
}
///8AB870: source4000dot fast path; otherdot divides/multiplies thenADD4/SAR4.
pub fn movement(delta: i32, freedom: [i16; 2], dot: u16) -> Result<([i32; 2], u8), Error> {
    bounded(delta)?;
    let mut result = [0; 2];
    let mut touched = 0;
    for axis in 0..2 {
        let f = freedom[axis];
        if f == 0 {
            continue;
        }
        result[axis] = if dot == 0x4000 {
            mul14(delta, f)?
        } else {
            let ratio = div16((f as i32).wrapping_shl(4), dot as i16 as i32)?;
            mul16(ratio, delta)?.wrapping_add(4) >> 4
        };
        touched |= 1 << axis;
    }
    Ok((result, touched))
}
