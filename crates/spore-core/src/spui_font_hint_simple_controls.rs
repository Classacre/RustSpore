//! Unexported successor: source minimum/three round controls, MIRP family, DELTAP3.
//! Caller original-package provenance is required; shape/length guards are
//! owned policy and do not establish content identity. Other opcodes/states
//! fail closed. Accepted glyph0/PREP implementations remain unchanged.
use crate::spui_font_hint_prep::PrepResult;
use crate::spui_font_hint_program::{
    BootstrapLimits, FunctionDefinition, bootstrap_fpgm, instruction_end,
};
use crate::spui_font_scaling::{Coordinate26Dot6, DesignCoordinate};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Shape,
    State,
    Program,
    Stack,
    Index(i32),
    Unsupported(u8),
    UnsupportedFunction(i32),
    UnsupportedFunctionBranch(i32),
    IupState,
    Fuel,
    CallDepth,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispatch {
    pub selector: u16,
    pub pc: usize,
    pub opcode: u8,
    pub stack: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleGlyphResult {
    pub points: Vec<[i32; 2]>,
    pub original: Vec<[i32; 2]>,
    pub touched: Vec<u8>,
    pub graphics: [u8; 52],
    pub cvt: Vec<i32>,
    pub storage: Vec<i32>,
    pub dispatches: Vec<Dispatch>,
    pub reference_points: [i32; 3],
    pub stack: Vec<i32>,
    pub axis: usize,
    pub calls: usize,
    pub loop_minus_one: u16,
}
// Original callbacks receive literal compensation0 from admitted MDAP/MIAP/MDRP/MIRP.
fn rounded(r: &SimpleGlyphResult, d: i32) -> Result<i32, Error> {
    let callback = g32(&r.graphics, 24);
    let value = match callback {
        0x8ab710 | 0x8ab680 => {
            let (bias, mask) = if callback == 0x8ab710 {
                (32, !63)
            } else {
                (16, !31)
            };
            let value = if d >= 0 {
                d.wrapping_add(bias) & mask
            } else {
                (0i32.wrapping_sub(d).wrapping_add(bias) & mask).wrapping_neg()
            };
            if d != 0 && (value ^ d) < 0 { 0 } else { value }
        }
        0x8ab740 => {
            let value = if d >= 0 {
                (d & !63).wrapping_add(32)
            } else {
                (-32i32).wrapping_sub(d.wrapping_neg() & !63)
            };
            if d != 0 && (value ^ d) < 0 {
                if d > 0 { 32 } else { -32 }
            } else {
                value
            }
        }
        _ => return Err(Error::State),
    };
    Ok(value)
}
fn index(v: i32, len: usize) -> Result<usize, Error> {
    let i = usize::try_from(v).map_err(|_| Error::Index(v))?;
    if i >= len {
        Err(Error::Index(v))
    } else {
        Ok(i)
    }
}
fn pop(stack: &mut Vec<i32>) -> Result<i32, Error> {
    stack.pop().ok_or(Error::Stack)
}
fn g32(g: &[u8; 52], at: usize) -> i32 {
    i32::from_le_bytes(g[at..at + 4].try_into().unwrap())
}
fn g16(g: &[u8; 52], at: usize) -> u16 {
    u16::from_le_bytes(g[at..at + 2].try_into().unwrap())
}
//8AC70B/70C stores lowWORD(n-1); consumers test it as signed WORD.
//Negative values execute zero points; otherwise exactly stored+1, then reset0.
fn take_loop(r: &mut SimpleGlyphResult) -> Result<usize, Error> {
    let stored = r.loop_minus_one;
    r.loop_minus_one = 0;
    let count = if (stored as i16) < 0 {
        0
    } else {
        stored as usize + 1
    };
    if count > r.stack.len() {
        return Err(Error::Stack);
    }
    Ok(count)
}
fn move_axis(r: &mut SimpleGlyphResult, p: usize, delta: i32) {
    r.points[p][r.axis] = r.points[p][r.axis].wrapping_add(delta);
    r.touched[p] |= 1 << r.axis;
}
fn iup(
    r: &mut SimpleGlyphResult,
    design: &[DesignCoordinate],
    ends: &[u16],
    axis: usize,
) -> Result<(), Error> {
    let coord = |i: usize| {
        if axis == 0 {
            design[i].x as i32
        } else {
            design[i].y as i32
        }
    };
    let bit = 1 << axis;
    let mut start = 0;
    for &end in ends {
        let end = end as usize;
        if end >= design.len() || end < start {
            return Err(Error::Shape);
        }
        let anchors: Vec<_> = (start..=end).filter(|&i| r.touched[i] & bit != 0).collect();
        for pair in 0..anchors.len() {
            let a = anchors[pair];
            let b = anchors[(pair + 1) % anchors.len()];
            let mut p = if a == end { start } else { a + 1 };
            let (lo, hi) = if coord(a) < coord(b) { (a, b) } else { (b, a) };
            let span = coord(hi) - coord(lo);
            let orig_lo = r.original[lo][axis];
            let orig_hi = r.original[hi][axis];
            let now_lo = r.points[lo][axis];
            let now_hi = r.points[hi][axis];
            while p != b {
                if span == 0 {
                    r.points[p][axis] =
                        r.points[p][axis].wrapping_add(now_lo.wrapping_sub(orig_lo));
                } else {
                    let before = r.original[p][axis];
                    r.points[p][axis] = if before <= orig_lo {
                        before.wrapping_add(if before < orig_hi {
                            now_lo.wrapping_sub(orig_lo)
                        } else {
                            now_hi.wrapping_sub(orig_hi)
                        })
                    } else if before >= orig_hi {
                        before.wrapping_add(now_hi.wrapping_sub(orig_hi))
                    } else {
                        let distance = now_hi.wrapping_sub(now_lo);
                        if span >= 32768 || distance >= 32768 {
                            return Err(Error::IupState);
                        }
                        let numerator = coord(p)
                            .wrapping_sub(coord(lo))
                            .wrapping_mul(distance)
                            .wrapping_add(span >> 1);
                        now_lo.wrapping_add(numerator / span)
                    };
                }
                p = if p == end { start } else { p + 1 };
            }
        }
        start = end + 1;
    }
    Ok(())
}

fn fixed_div(a: i32, b: i32) -> Result<i32, Error> {
    if b == 0 || a == i32::MIN || b == i32::MIN || a.abs() > 32767 || b.abs() > 32767 {
        return Err(Error::State);
    }
    let magnitude = ((a.abs() as i64) << 16) / (b.abs() as i64);
    let magnitude = i32::try_from(magnitude).map_err(|_| Error::State)?;
    Ok(if (a ^ b) < 0 { -magnitude } else { magnitude })
}
fn fixed_mul(a: i32, b: i32) -> Result<i32, Error> {
    if a == i32::MIN || b == i32::MIN {
        return Err(Error::State);
    }
    let magnitude = ((a.abs() as i64) * (b.abs() as i64)) >> 16;
    let magnitude = i32::try_from(magnitude).map_err(|_| Error::State)?;
    Ok(if (a ^ b) < 0 { -magnitude } else { magnitude })
}
fn skip(program: &[u8], mut pc: usize, end: usize, stop_else: bool) -> Result<usize, Error> {
    let mut depth = 1;
    while pc < end {
        let op = program[pc];
        pc += 1;
        match op {
            0x58 => depth += 1,
            0x59 => {
                depth -= 1;
                if depth == 0 {
                    return Ok(pc);
                }
            }
            0x1b if stop_else && depth == 1 => return Ok(pc),
            _ => {
                pc = instruction_end(program, pc - 1).map_err(|_| Error::Program)?;
            }
        }
    }
    Err(Error::Program)
}
fn execute(
    r: &mut SimpleGlyphResult,
    program: &[u8],
    selector: u16,
    start: usize,
    end: usize,
    design: &[DesignCoordinate],
    ends: &[u16],
    fpgm: &[u8],
    definitions: &[Option<FunctionDefinition<'_>>],
    capacity: usize,
    fuel: &mut usize,
    depth: usize,
) -> Result<(), Error> {
    if depth > 32 {
        return Err(Error::CallDepth);
    }
    let mut pc = start;
    while pc < end {
        if *fuel == 0 {
            return Err(Error::Fuel);
        }
        *fuel -= 1;
        let op = *program.get(pc).ok_or(Error::Program)?;
        r.dispatches.push(Dispatch {
            selector,
            pc,
            opcode: op,
            stack: r.stack.len(),
        });
        pc += 1;
        match op {
            0x40 | 0x41 | 0xb0..=0xbf => {
                let words = op == 0x41 || op >= 0xb8;
                let count = if op <= 0x41 {
                    let count = *program.get(pc).ok_or(Error::Program)? as usize;
                    pc += 1;
                    count
                } else if words {
                    (op - 0xb7) as usize
                } else {
                    (op - 0xaf) as usize
                };
                if r.stack.len() + count > capacity {
                    return Err(Error::Stack);
                }
                for _ in 0..count {
                    let a = *program.get(pc).ok_or(Error::Program)?;
                    pc += 1;
                    let value = if words {
                        let b = *program.get(pc).ok_or(Error::Program)?;
                        pc += 1;
                        i16::from_be_bytes([a, b]) as i32
                    } else {
                        a as i32
                    };
                    r.stack.push(value);
                }
            }
            0x00 | 0x01 => r.axis = if op == 1 { 0 } else { 1 },
            0x10..=0x12 => r.reference_points[(op - 0x10) as usize] = pop(&mut r.stack)?,
            0x1a => {
                let value = pop(&mut r.stack)?;
                r.graphics[20..24].copy_from_slice(&value.to_le_bytes());
            }
            0x18 | 0x19 | 0x3d => {
                let callback: i32 = match op {
                    0x18 => 0x8ab710,
                    0x19 => 0x8ab740,
                    _ => 0x8ab680,
                };
                r.graphics[24..28].copy_from_slice(&callback.to_le_bytes());
            }
            0x17 => {
                r.loop_minus_one = pop(&mut r.stack)?.wrapping_sub(1) as u16;
            }
            0x21 => {
                pop(&mut r.stack)?;
            }
            0x43 => {
                let i = index(pop(&mut r.stack)?, r.storage.len())?;
                r.stack.push(r.storage[i]);
            }
            0x54 => {
                let b = pop(&mut r.stack)?;
                let a = pop(&mut r.stack)?;
                r.stack.push((a == b) as i32);
            }
            0x58 => {
                let condition = pop(&mut r.stack)?;
                if selector == 1 && condition != 0 {
                    return Err(Error::UnsupportedFunctionBranch(75));
                }
                if condition == 0 {
                    pc = skip(program, pc, end, true)?;
                }
            }
            0x1b => pc = skip(program, pc, end, false)?,
            0x59 => {}
            0x2b => {
                let id = pop(&mut r.stack)?;
                if id != 75 {
                    return Err(Error::UnsupportedFunction(id));
                }
                let i = index(id, definitions.len())?;
                let definition = definitions[i].ok_or(Error::Index(id))?;
                if definition.program_selector != 1 {
                    return Err(Error::State);
                }
                r.calls += 1;
                execute(
                    r,
                    fpgm,
                    1,
                    definition.offset as usize,
                    definition.offset as usize + definition.length as usize,
                    design,
                    ends,
                    fpgm,
                    definitions,
                    capacity,
                    fuel,
                    depth + 1,
                )?;
            }
            0x2f => {
                let p = index(pop(&mut r.stack)?, r.points.len())?;
                r.reference_points[0] = p as i32;
                r.reference_points[1] = p as i32;
                let before = r.points[p][r.axis];
                let wanted = rounded(r, before)?;
                move_axis(r, p, wanted.wrapping_sub(before));
            }
            0x3f => {
                let cv = index(pop(&mut r.stack)?, r.cvt.len())?;
                let p = index(pop(&mut r.stack)?, r.points.len())?;
                r.reference_points[0] = p as i32;
                r.reference_points[1] = p as i32;
                let before = r.points[p][r.axis];
                let mut wanted = r.cvt[cv];
                if wanted.wrapping_sub(before).wrapping_abs() > g32(&r.graphics, 0) {
                    wanted = before;
                }
                let wanted = rounded(r, wanted)?;
                move_axis(r, p, wanted.wrapping_sub(before));
            }
            0xc0..=0xdf => {
                let p = index(pop(&mut r.stack)?, r.points.len())?;
                let reference = index(r.reference_points[0], r.points.len())?;
                //8AE443..477 projects original DESIGN words, then the source units callback;
                //subtracting independently rounded original26.6 coordinates is different.
                let coord = |i: usize| {
                    if r.axis == 0 {
                        design[i].x as i32
                    } else {
                        design[i].y as i32
                    }
                };
                let distance = coord(p).wrapping_sub(coord(reference));
                let original = distance.wrapping_mul(15).wrapping_add(16) >> 5;
                let mut wanted = if op & 4 != 0 {
                    rounded(r, original)?
                } else {
                    original
                };
                if op & 8 != 0 {
                    let minimum = g32(&r.graphics, 20);
                    if original >= 0 {
                        wanted = wanted.max(minimum)
                    } else {
                        wanted = wanted.min(minimum.wrapping_neg())
                    }
                }
                let current = r.points[p][r.axis].wrapping_sub(r.points[reference][r.axis]);
                move_axis(r, p, wanted.wrapping_sub(current));
                r.reference_points[1] = reference as i32;
                r.reference_points[2] = p as i32;
                if op & 16 != 0 {
                    r.reference_points[0] = p as i32;
                }
            }
            0xe0..=0xff => {
                let cv = index(pop(&mut r.stack)?, r.cvt.len())?;
                let p = index(pop(&mut r.stack)?, r.points.len())?;
                let reference = index(r.reference_points[0], r.points.len())?;
                let original = r.original[p][r.axis].wrapping_sub(r.original[reference][r.axis]);
                let current = r.points[p][r.axis].wrapping_sub(r.points[reference][r.axis]);
                let mut wanted = r.cvt[cv];
                if (original ^ wanted) < 0 {
                    wanted = wanted.wrapping_neg();
                }
                if op & 4 != 0 && wanted.wrapping_sub(original).wrapping_abs() > g32(&r.graphics, 0)
                {
                    wanted = original;
                }
                if op & 4 != 0 {
                    wanted = rounded(r, wanted)?;
                }
                let minimum = g32(&r.graphics, 20);
                if op & 8 != 0 {
                    if original >= 0 {
                        wanted = wanted.max(minimum)
                    } else {
                        wanted = wanted.min(minimum.wrapping_neg())
                    }
                }
                if op & 16 != 0 {
                    r.reference_points[0] = p as i32;
                }
                r.reference_points[1] = reference as i32;
                r.reference_points[2] = p as i32;
                move_axis(r, p, wanted.wrapping_sub(current));
            }
            0x32 | 0x33 => {
                //8ACF30 chooses RP2 for32 and RP1 for33, projects current-original.
                //The admitted ordinary equal-axis vector makes8D0D50 exact identity.
                let reference = index(
                    r.reference_points[if op & 1 != 0 { 1 } else { 2 }],
                    r.points.len(),
                )?;
                let delta = r.points[reference][r.axis].wrapping_sub(r.original[reference][r.axis]);
                let count = take_loop(r)?;
                for _ in 0..count {
                    let p = index(pop(&mut r.stack)?, r.points.len())?;
                    move_axis(r, p, delta);
                }
            }
            0x39 => {
                let a = index(r.reference_points[1], r.points.len())?;
                let b = index(r.reference_points[2], r.points.len())?;

                let old_span = r.original[b][r.axis].wrapping_sub(r.original[a][r.axis]);
                let now_span = r.points[b][r.axis].wrapping_sub(r.points[a][r.axis]);
                // Original8AD388 uses signed-magnitude truncating16.16 divide.
                // Small bounded axis spans avoid its large remainder reduction path.
                let ratio = if old_span == 0 {
                    65536
                } else {
                    fixed_div(now_span, old_span)?
                };
                let count = take_loop(r)?;
                for _ in 0..count {
                    let p = index(pop(&mut r.stack)?, r.points.len())?;
                    let old_point = r.original[p][r.axis].wrapping_sub(r.original[a][r.axis]);
                    let wanted = fixed_mul(old_point, ratio)?;
                    let current = r.points[p][r.axis].wrapping_sub(r.points[a][r.axis]);
                    move_axis(r, p, wanted.wrapping_sub(current));
                }
            }
            0x3c => {
                let reference = index(r.reference_points[0], r.points.len())?;
                let count = take_loop(r)?;
                for _ in 0..count {
                    let p = index(pop(&mut r.stack)?, r.points.len())?;
                    let delta = r.points[reference][r.axis].wrapping_sub(r.points[p][r.axis]);
                    move_axis(r, p, delta);
                }
            }
            0x5f => {
                let v = pop(&mut r.stack)? as u16;
                r.graphics[44..46].copy_from_slice(&v.to_le_bytes());
            }
            0x5d | 0x71 | 0x72 => {
                let count = usize::try_from(pop(&mut r.stack)?).map_err(|_| Error::Stack)?;
                let words = count.checked_mul(2).ok_or(Error::Stack)?;
                let at = r.stack.len().checked_sub(words).ok_or(Error::Stack)?;
                let args = r.stack.split_off(at);
                //8AEBF9 adds in DX;8AEAF3 consumes signed WORD base.
                let offset: u16 = match op {
                    0x71 => 16,
                    0x72 => 32,
                    _ => 0,
                };
                let base = g16(&r.graphics, 42).wrapping_add(offset) as i16 as i32;
                let target = 15 - base;
                if (0..=15).contains(&target) {
                    let mut width = (args.len() >> 1) & !1;
                    let mut at = 0;
                    while width > 2 {
                        let next = width + at;
                        if args[next] & !15 < target << 4 {
                            at = next;
                        }
                        width = (width >> 1) & !1;
                    }
                    while at < args.len() {
                        let arg = args[at];
                        let high = arg & !15;
                        if high == target << 4 {
                            let p = index(args[at + 1], r.points.len())?;
                            let low = arg & 15;
                            let delta = (low - if low < 8 { 8 } else { 7 }).wrapping_shl(6)
                                >> ((g16(&r.graphics, 44) as u32) & 31);
                            move_axis(r, p, delta);
                        } else if high > target << 4 {
                            break;
                        }
                        at += 2;
                    }
                }
            }
            0x30 | 0x31 => iup(r, design, ends, if op == 0x31 { 0 } else { 1 })?,
            _ => return Err(Error::Unsupported(op)),
        }
        if pc > end {
            return Err(Error::Program);
        }
    }
    Ok(())
}
/// Fresh font0 15ppem is an explicit caller contract. Point/program bounds
/// are owned resource policy; supported axis bytecode is checked while executed.
/// Arbitrary same-sized content does not establish original-font identity.
pub fn execute_simple(
    program: &[u8],
    design: &[DesignCoordinate],
    scaled: &[Coordinate26Dot6],
    ends: &[u16],
    prep: &PrepResult,
    fpgm: &[u8],
    maxp: &[u8],
) -> Result<SimpleGlyphResult, Error> {
    let real = design.len().checked_sub(4).ok_or(Error::Shape)?;
    if !(4..=64).contains(&real)
        || program.len() > 512
        || program.is_empty()
        || scaled.len() != design.len()
        || ends.is_empty()
        || ends.len() > 8
        || ends.last().copied() != Some((real - 1) as u16)
        || ends.windows(2).any(|v| v[0] >= v[1])
        || prep.cvt.len() != 173
        || prep.storage.len() != 47
    {
        return Err(Error::Shape);
    }
    if g32(&prep.graphics, 24) != 0x8ab710 || g32(&prep.graphics, 4) != 0 || prep.graphics[50] != 1
    {
        return Err(Error::State);
    }
    let limits = BootstrapLimits::from_maxp(maxp).map_err(|_| Error::Program)?;
    let setup = bootstrap_fpgm(fpgm, limits).map_err(|_| Error::Program)?;
    let original: Vec<_> = scaled.iter().map(|p| [p.x, p.y]).collect();
    let mut r = SimpleGlyphResult {
        points: original.clone(),
        original,
        touched: vec![0; design.len()],
        graphics: prep.graphics,
        cvt: prep.cvt.clone(),
        storage: prep.storage.clone(),
        dispatches: Vec::new(),
        reference_points: [0; 3],
        stack: Vec::new(),
        axis: 0,
        calls: 0,
        loop_minus_one: 0,
    };
    for (axis, at) in [(0, real), (1, real + 2)] {
        let distance = r.points[at + 1][axis].wrapping_sub(r.points[at][axis]);
        r.points[at][axis] = r.points[at][axis].wrapping_add(32) & !63;
        r.points[at + 1][axis] = r.points[at][axis].wrapping_add(distance.wrapping_add(32) & !63);
    }
    let mut fuel = 4096;
    execute(
        &mut r,
        program,
        0,
        0,
        program.len(),
        design,
        ends,
        fpgm,
        &setup.functions,
        limits.max_stack_elements as usize,
        &mut fuel,
        0,
    )?;
    if !r.stack.is_empty() {
        return Err(Error::Stack);
    }
    Ok(r)
}
