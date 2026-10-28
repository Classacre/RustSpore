//! Bounded original font0 PREP path, square 15ppem, integer-only.
//! Unexported draft. Source operands wrap as DWORDs; checked memory and fuel
//! are owned guards. Unsupported opcodes/state fail closed. No glyph program.
use crate::spui_font_hint_program::{BootstrapLimits, bootstrap_fpgm, instruction_end};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispatch {
    pub selector: u16,
    pub pc: usize,
    pub opcode: u8,
    pub stack: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepResult {
    pub cvt: Vec<i32>,
    pub storage: Vec<i32>,
    pub graphics: [u8; 52],
    pub twilight: Vec<u8>,
    pub dispatches: Vec<Dispatch>,
    pub calls: usize,
    pub stack: Vec<i32>,
    pub reference_points: [i32; 3],
    pub axis: u8,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Program,
    Unsupported(u8),
    State,
    Stack,
    Index(i32),
    Fuel,
    Truncated,
    UnsupportedSuperRoundPeriod(i32),
    CallDepth,
}
struct Vm<'a> {
    fpgm: &'a [u8],
    prep: &'a [u8],
    defs: Vec<Option<(usize, usize)>>,
    result: PrepResult,
    capacity: usize,
    fuel: usize,
    round: u32,
    period: i32,
    phase: i32,
    threshold: i32,
    mask: i32,
    zone: i32,
}
impl Vm<'_> {
    fn pop(&mut self) -> Result<i32, Error> {
        self.result.stack.pop().ok_or(Error::Stack)
    }
    fn push(&mut self, x: i32) -> Result<(), Error> {
        if self.result.stack.len() >= self.capacity {
            return Err(Error::Stack);
        }
        self.result.stack.push(x);
        Ok(())
    }
    fn index(index: i32, len: usize) -> Result<usize, Error> {
        let i = usize::try_from(index).map_err(|_| Error::Index(index))?;
        if i >= len {
            return Err(Error::Index(index));
        }
        Ok(i)
    }
    fn g32(&mut self, at: usize, value: i32) {
        self.result.graphics[at..at + 4].copy_from_slice(&value.to_le_bytes())
    }
    fn g16(&mut self, at: usize, value: i32) {
        self.result.graphics[at..at + 2].copy_from_slice(&(value as i16).to_le_bytes())
    }
    fn read16(&self, at: usize) -> i32 {
        i16::from_le_bytes(self.result.graphics[at..at + 2].try_into().unwrap()) as i32
    }
    fn scale(x: i32) -> i32 {
        x.wrapping_mul(15).wrapping_add(16) >> 5
    }
    fn rounded(&self, d: i32) -> Result<i32, Error> {
        let (mask, phase, threshold) = match self.round {
            0x8ab710 => (!63, 0, 32),
            0x8ab740 => (!63, 32, 32),
            0x8ab780 => (self.mask, self.phase, self.threshold),
            _ => return Err(Error::State),
        };
        let base = threshold.wrapping_sub(phase);
        let r = if d >= 0 {
            base.wrapping_add(d) & mask
        } else {
            base.wrapping_sub(d) & mask
        };
        let r = r.wrapping_add(phase);
        let r = if d < 0 { r.wrapping_neg() } else { r };
        Ok(if (r ^ d) < 0 && d != 0 {
            if self.round == 0x8ab780 {
                if d > 0 { phase } else { phase.wrapping_neg() }
            } else {
                0
            }
        } else {
            r
        })
    }
    fn super_round(&mut self, value: i32) -> Result<(), Error> {
        self.period = match value & 0xc0 {
            0 => 32,
            64 => 64,
            128 => 128,
            _ => return Err(Error::UnsupportedSuperRoundPeriod(value)),
        };
        self.mask = !(self.period - 1);
        self.phase = match value & 0x30 {
            0 => 0,
            16 => (self.period + 2) >> 2,
            32 => (self.period + 1) >> 1,
            _ => (self.period * 3 + 2) >> 2,
        };
        let n = value & 15;
        self.threshold = if n == 0 {
            self.period - 1
        } else {
            ((n - 4) * self.period + 4) >> 3
        };
        self.round = 0x8ab780;
        self.g32(24, self.round as i32);
        self.g32(28, self.mask);
        self.g16(36, self.period);
        self.g16(38, self.phase);
        self.g16(40, self.threshold);
        Ok(())
    }
    fn skip(&self, program: &[u8], mut pc: usize, stop_else: bool) -> Result<usize, Error> {
        let mut depth = 1;
        while pc < program.len() {
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
                _ => pc = instruction_end(program, pc - 1).map_err(|_| Error::Truncated)?,
            }
        }
        Err(Error::Truncated)
    }
    fn execute(
        &mut self,
        selector: u16,
        start: usize,
        end: usize,
        depth: usize,
    ) -> Result<(), Error> {
        if depth > 32 {
            return Err(Error::CallDepth);
        }
        let program = if selector == 0 {
            self.prep
        } else if selector == 1 {
            self.fpgm
        } else {
            return Err(Error::State);
        };
        let mut pc = start;
        while pc < end {
            if self.fuel == 0 {
                return Err(Error::Fuel);
            }
            self.fuel -= 1;
            let op = *program.get(pc).ok_or(Error::Truncated)?;
            self.result.dispatches.push(Dispatch {
                selector,
                pc,
                opcode: op,
                stack: self.result.stack.len(),
            });
            pc += 1;
            match op {
                0x40 | 0x41 | 0xb0..=0xbf => {
                    let words = op == 0x41 || op >= 0xb8;
                    let count = if op <= 0x41 {
                        let n = *program.get(pc).ok_or(Error::Truncated)? as usize;
                        pc += 1;
                        n
                    } else if words {
                        (op - 0xb7) as usize
                    } else {
                        (op - 0xaf) as usize
                    };
                    for _ in 0..count {
                        let first = *program.get(pc).ok_or(Error::Truncated)?;
                        pc += 1;
                        let value = if words {
                            let second = *program.get(pc).ok_or(Error::Truncated)?;
                            pc += 1;
                            i16::from_be_bytes([first, second]) as i32
                        } else {
                            first as i32
                        };
                        self.push(value)?;
                    }
                }
                0x2b => {
                    let id = self.pop()?;
                    let i = Self::index(id, self.defs.len())?;
                    let (at, len) = self.defs[i].ok_or(Error::Index(id))?;
                    self.result.calls += 1;
                    self.execute(1, at, at + len, depth + 1)?;
                }
                0x20 => {
                    let value = *self.result.stack.last().ok_or(Error::Stack)?;
                    self.push(value)?;
                }
                0x21 => {
                    self.pop()?;
                }
                0x23 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(b)?;
                    self.push(a)?;
                }
                0x8a => {
                    let c = self.pop()?;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(b)?;
                    self.push(c)?;
                    self.push(a)?;
                }
                0x50..=0x55 | 0x5a | 0x5b | 0x60 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let value = match op {
                        0x50 => (a < b) as i32,
                        0x51 => (a <= b) as i32,
                        0x52 => (a > b) as i32,
                        0x53 => (a >= b) as i32,
                        0x54 => (a == b) as i32,
                        0x55 => (a != b) as i32,
                        0x5a => (a != 0 && b != 0) as i32,
                        0x5b => (a != 0 || b != 0) as i32,
                        _ => a.wrapping_add(b),
                    };
                    self.push(value)?;
                }
                0x5c => {
                    let x = self.pop()?;
                    self.push((x == 0) as i32)?;
                }
                0x58 => {
                    if self.pop()? == 0 {
                        pc = self.skip(program, pc, true)?;
                    }
                }
                0x1b => {
                    pc = self.skip(program, pc, false)?;
                }
                0x59 => {}
                0x4b => {
                    self.push(15)?;
                }
                0x00 | 0x01 => {
                    self.result.axis = op;
                }
                0x42 => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let i = Self::index(index, self.result.storage.len())?;
                    self.result.storage[i] = value;
                }
                0x43 => {
                    let index = self.pop()?;
                    let i = Self::index(index, self.result.storage.len())?;
                    self.push(self.result.storage[i])?;
                }
                0x44 | 0x70 => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let i = Self::index(index, self.result.cvt.len())?;
                    self.result.cvt[i] = if op == 0x70 {
                        Self::scale(value)
                    } else {
                        value
                    };
                }
                0x45 => {
                    let index = self.pop()?;
                    let i = Self::index(index, self.result.cvt.len())?;
                    self.push(self.result.cvt[i])?;
                }
                0x18 | 0x19 => {
                    self.round = if op == 0x18 { 0x8ab710 } else { 0x8ab740 };
                    self.g32(24, self.round as i32);
                }
                0x68 | 0x69 => {
                    let d = self.pop()?;
                    let value = self.rounded(d)?;
                    self.push(value)?;
                }
                0x76 => {
                    let v = self.pop()?;
                    self.super_round(v)?;
                }
                0x1d => {
                    let v = self.pop()?;
                    self.g32(0, v);
                }
                0x85 => {
                    let v = self.pop()?;
                    let old = i32::from_le_bytes(self.result.graphics[12..16].try_into().unwrap()); // 8AD714 masks old high WORD, 8AD71A ORs the full popped DWORD,
                    // 8AD71C stores a DWORD. Do not narrow the popped value here.
                    self.g32(12, (old & !65535) | v);
                }
                0x8d => {
                    let v = self.pop()?;
                    if v != 1 {
                        return Err(Error::State);
                    }
                    self.g16(14, 1);
                }
                0x88 => {
                    let query = self.pop()?;
                    self.push(query & 1)?;
                }
                0x16 => {
                    let z = self.pop()?;
                    if z != 0 {
                        return Err(Error::State);
                    }
                    self.zone = z;
                }
                0x10 => {
                    self.result.reference_points[0] = self.pop()?;
                }
                0x3e => {
                    let cv = self.pop()?;
                    let pt = self.pop()?;
                    let cv = Self::index(cv, self.result.cvt.len())?;
                    let n = self.result.twilight.len() / 22;
                    let pt = Self::index(pt, n)?;
                    if self.zone != 0 || self.result.axis != 1 {
                        return Err(Error::State);
                    }
                    let v = self.result.cvt[cv];
                    for off in [pt * 4, n * 8 + pt * 4] {
                        self.result.twilight[off..off + 4].copy_from_slice(&v.to_le_bytes())
                    }
                    self.result.twilight[n * 21 + pt] |= 1;
                    self.result.reference_points[0] = pt as i32;
                    self.result.reference_points[1] = pt as i32;
                }
                0x73 | 0x74 => {
                    let count = self.pop()?;
                    let count = usize::try_from(count).map_err(|_| Error::Stack)?;
                    let words = count.checked_mul(2).ok_or(Error::Stack)?;
                    let start = self
                        .result
                        .stack
                        .len()
                        .checked_sub(words)
                        .ok_or(Error::Stack)?;
                    let args = self.result.stack.split_off(start);
                    let base = self.read16(42) + if op == 0x74 { 16 } else { 0 };
                    let target = 15 - base;
                    if (0..=15).contains(&target) {
                        // Exact source lower-bound search8AEB01..2B then forward scan.
                        let limit = args.len();
                        let mut width = (limit >> 1) & !1;
                        let mut at = 0;
                        while width > 2 {
                            let next = width + at;
                            if (args[next] & !15) < (target << 4) {
                                at = next;
                            }
                            width = (width >> 1) & !1;
                        }
                        while at < limit {
                            let arg = args[at];
                            let high = arg & !15;
                            if high != (target << 4) {
                                if high > (target << 4) {
                                    break;
                                }
                            } else {
                                let i = Self::index(args[at + 1], self.result.cvt.len())?;
                                let low = arg & 15;
                                let delta = low - if low < 8 { 8 } else { 7 };
                                let shift = (self.read16(44) as u32) & 31;
                                let delta = delta.wrapping_shl(6) >> shift;
                                self.result.cvt[i] = self.result.cvt[i].wrapping_add(delta);
                            }
                            at += 2;
                        }
                    }
                }
                _ => return Err(Error::Unsupported(op)),
            }
            if pc > end {
                return Err(Error::Truncated);
            }
        }
        Ok(())
    }
}
/// Source15ppem coefficients, empty storage and font0 table lengths are owned
/// bounded API admission policy. Lengths do not establish font content identity;
/// the caller must independently bind tables to its intended original font.
/// Only the measured shape, source15ppem coefficients, empty storage,
/// source default graphics and16 zero twilight points are admitted.
pub fn execute_font0_prep(
    fpgm: &[u8],
    prep: &[u8],
    maxp: &[u8],
    cvt_be: &[u8],
    units_per_em: u16,
) -> Result<PrepResult, Error> {
    let limits = BootstrapLimits::from_maxp(maxp).map_err(|_| Error::Program)?;
    if units_per_em != 2048
        || fpgm.len() != 1514
        || prep.len() != 457
        || cvt_be.len() != 346
        || maxp.len() < 32
        || u16::from_be_bytes([maxp[16], maxp[17]]) != 16
        || u16::from_be_bytes([maxp[18], maxp[19]]) != 47
    {
        return Err(Error::State);
    }
    let setup = bootstrap_fpgm(fpgm, limits).map_err(|_| Error::Program)?;
    let defs = setup
        .functions
        .iter()
        .map(|f| f.as_ref().map(|f| (f.offset as usize, f.length as usize)))
        .collect();
    let cvt = cvt_be
        .chunks_exact(2)
        .map(|x| Vm::scale(i16::from_be_bytes([x[0], x[1]]) as i32))
        .collect();
    let result = PrepResult {
        cvt,
        storage: vec![0; 47],
        graphics: [0; 52],
        twilight: vec![0; 16 * 22],
        dispatches: Vec::new(),
        calls: 0,
        stack: Vec::new(),
        reference_points: [0; 3],
        axis: 1,
    };
    let mut vm = Vm {
        fpgm,
        prep,
        defs,
        result,
        capacity: limits.max_stack_elements as usize,
        fuel: 10000,
        round: 0x8ab710,
        period: 0,
        phase: 0,
        threshold: 0,
        mask: 0,
        zone: 0,
    };
    vm.g32(0, 68);
    vm.g32(12, 0x307c0);
    vm.g32(20, 64);
    vm.g32(24, 0x8ab710);
    vm.g16(42, 9);
    vm.g16(44, 3);
    vm.g16(46, 128);
    vm.result.graphics[50] = 1;
    vm.execute(0, 0, prep.len(), 0)?;
    if !vm.result.stack.is_empty() {
        return Err(Error::Stack);
    }
    Ok(vm.result)
}
