//! Bounded original font0 glyph0 axis hint path at fresh15ppem.
//! Content identity belongs to the caller. Shape/state guards are owned API
//! policy. Native hinted coordinates were not emitted; the reference executes
//! original source instructions. Rasterization and other glyphs remain open.
use crate::spui_font_hint_prep::PrepResult;
use crate::spui_font_scaling::{Coordinate26Dot6, DesignCoordinate};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Shape,
    State,
    Stack,
    Index(i32),
    Unsupported(u8),
    IupState,
    Fuel,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispatch {
    pub pc: usize,
    pub opcode: u8,
    pub stack: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphResult {
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
}
fn grid(d: i32) -> i32 {
    let r = if d >= 0 {
        d.wrapping_add(32) & !63
    } else {
        (0i32.wrapping_sub(d).wrapping_add(32) & !63).wrapping_neg()
    };
    if d != 0 && (r ^ d) < 0 { 0 } else { r }
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
fn move_axis(r: &mut GlyphResult, p: usize, delta: i32) {
    r.points[p][r.axis] = r.points[p][r.axis].wrapping_add(delta);
    r.touched[p] |= 1 << r.axis;
}
fn iup(
    r: &mut GlyphResult,
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
/// Input includes8 real+4 phantom original design/scaled points. The source
/// phantom grid lock occurs after original-coordinate copies8CC684..6B1.
/// Only the original glyph0 shape/fresh PREP state and axis callbacks are
/// admitted. Identical lengths/topology do not establish font content identity.
pub fn execute_glyph0(
    program: &[u8],
    design: &[DesignCoordinate],
    scaled: &[Coordinate26Dot6],
    ends: &[u16],
    prep: &PrepResult,
) -> Result<GlyphResult, Error> {
    if program.len() != 38
        || design.len() != 12
        || scaled.len() != 12
        || ends != [3, 7]
        || prep.cvt.len() != 173
        || prep.storage.len() != 47
    {
        return Err(Error::Shape);
    }
    if g32(&prep.graphics, 24) != 0x8ab710 || g32(&prep.graphics, 4) != 0 || prep.graphics[50] != 1
    {
        return Err(Error::State);
    }
    let original: Vec<_> = scaled.iter().map(|p| [p.x, p.y]).collect();
    let mut r = GlyphResult {
        points: original.clone(),
        original,
        touched: vec![0; 12],
        graphics: prep.graphics,
        cvt: prep.cvt.clone(),
        storage: prep.storage.clone(),
        dispatches: Vec::new(),
        reference_points: [0; 3],
        stack: Vec::new(),
        axis: 0,
    };
    //Source8CC743..7AF rounds first phantom and its advance independently.
    for (axis, at) in [(0, 8), (1, 10)] {
        let distance = r.points[at + 1][axis].wrapping_sub(r.points[at][axis]);
        r.points[at][axis] = r.points[at][axis].wrapping_add(32) & !63;
        r.points[at + 1][axis] = r.points[at][axis].wrapping_add(distance.wrapping_add(32) & !63);
    }
    let mut pc = 0;
    let mut fuel = 1000;
    while pc < program.len() {
        if fuel == 0 {
            return Err(Error::Fuel);
        }
        fuel -= 1;
        let op = program[pc];
        r.dispatches.push(Dispatch {
            pc,
            opcode: op,
            stack: r.stack.len(),
        });
        pc += 1;
        match op {
            0x40 => {
                let count = *program.get(pc).ok_or(Error::Stack)? as usize;
                pc += 1;
                let bytes = program.get(pc..pc + count).ok_or(Error::Stack)?;
                if r.stack.len() + count > 512 {
                    return Err(Error::Stack);
                }
                r.stack.extend(bytes.iter().map(|&v| v as i32));
                pc += count;
            }
            0x00 | 0x01 => {
                r.axis = if op == 1 { 0 } else { 1 };
            }
            0x2f => {
                let p = index(pop(&mut r.stack)?, 12)?;
                r.reference_points[0] = p as i32;
                r.reference_points[1] = p as i32;
                let before = r.points[p][r.axis];
                move_axis(&mut r, p, grid(before).wrapping_sub(before));
            }
            0xfd | 0xed => {
                let cv = index(pop(&mut r.stack)?, r.cvt.len())?;
                let p = index(pop(&mut r.stack)?, 12)?;
                let reference = index(r.reference_points[0], 12)?;
                let original = r.original[p][r.axis].wrapping_sub(r.original[reference][r.axis]);
                let current = r.points[p][r.axis].wrapping_sub(r.points[reference][r.axis]);
                let mut wanted = r.cvt[cv];
                if (original ^ wanted) < 0 {
                    wanted = wanted.wrapping_neg();
                }
                if wanted.wrapping_sub(original).wrapping_abs() > g32(&r.graphics, 0) {
                    wanted = original;
                }
                wanted = grid(wanted);
                let minimum = g32(&r.graphics, 20);
                if original >= 0 {
                    if wanted < minimum {
                        wanted = minimum;
                    }
                } else if wanted > minimum.wrapping_neg() {
                    wanted = minimum.wrapping_neg();
                }
                r.reference_points[1] = reference as i32;
                r.reference_points[2] = p as i32;
                if op & 16 != 0 {
                    r.reference_points[0] = p as i32;
                }
                move_axis(&mut r, p, wanted.wrapping_sub(current));
            }
            0x5f => {
                let v = pop(&mut r.stack)? as u16;
                r.graphics[44..46].copy_from_slice(&v.to_le_bytes());
            }
            0x5d => {
                let count = usize::try_from(pop(&mut r.stack)?).map_err(|_| Error::Stack)?;
                let words = count.checked_mul(2).ok_or(Error::Stack)?;
                let at = r.stack.len().checked_sub(words).ok_or(Error::Stack)?;
                let args = r.stack.split_off(at);
                let target = 15 - g16(&r.graphics, 42) as i32;
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
                            let p = index(args[at + 1], 12)?;
                            let low = arg & 15;
                            let delta = (low - if low < 8 { 8 } else { 7 }).wrapping_shl(6)
                                >> ((g16(&r.graphics, 44) as u32) & 31);
                            move_axis(&mut r, p, delta);
                        } else if high > target << 4 {
                            break;
                        }
                        at += 2;
                    }
                }
            }
            0x30 | 0x31 => iup(&mut r, design, &ends, if op == 0x31 { 0 } else { 1 })?,
            _ => return Err(Error::Unsupported(op)),
        }
    }
    if !r.stack.is_empty() {
        return Err(Error::Stack);
    }
    Ok(r)
}
