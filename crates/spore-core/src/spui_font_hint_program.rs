//! Bounded original FPGM setup: PUSH operands and FDEF body registration.
//!
//! Source 8ABCA0 dispatches PUSH handlers and 8AE840 records borrowed bodies
//! after a PUSH-aware scan through 8AE1E0. Only this setup is admitted here;
//! bodies are not executed. Caller-provided maxp capacities bound stack/table
//! access. Malformed-input guards are owned API behavior, not native recovery.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapLimits {
    pub max_functions: u16,
    pub max_stack_elements: u16,
}

impl BootstrapLimits {
    /// The ordinary native maxp1.0 fields consumed at 8CD3AD and 8CD3C1.
    pub fn from_maxp(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < 32 {
            return Err(ProgramError::TruncatedMaxp);
        }
        if data[..4] != [0, 1, 0, 0] {
            return Err(ProgramError::UnsupportedMaxpVersion);
        }
        Ok(Self {
            max_functions: u16::from_be_bytes([data[20], data[21]]),
            max_stack_elements: u16::from_be_bytes([data[24], data[25]]),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramError {
    TruncatedMaxp,
    UnsupportedMaxpVersion,
    TruncatedInstruction {
        pc: usize,
    },
    UnsupportedOpcode {
        pc: usize,
        opcode: u8,
    },
    StackOverflow {
        pc: usize,
        capacity: u16,
    },
    StackUnderflow {
        pc: usize,
    },
    FunctionIndexOutOfBounds {
        pc: usize,
        index: i32,
        capacity: u16,
    },
    MissingEndFunction {
        pc: usize,
    },
    DefinitionOffsetTooLarge {
        pc: usize,
    },
    DefinitionLengthTooLarge {
        pc: usize,
    },
}

impl fmt::Display for ProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "bounded FPGM setup: {self:?}")
    }
}

impl std::error::Error for ProgramError {}

/// 8AE1E0 advances the cursor already beyond an opcode by its PUSH payload.
/// Here `pc` points at that opcode. Non-PUSH opcodes have width one, including
/// unsupported body instructions; this function does not execute them.
pub fn instruction_end(program: &[u8], pc: usize) -> Result<usize, ProgramError> {
    let opcode = *program
        .get(pc)
        .ok_or(ProgramError::TruncatedInstruction { pc })?;
    let after_opcode = pc + 1;
    let payload = match opcode {
        0x40 | 0x41 => {
            let count = *program
                .get(after_opcode)
                .ok_or(ProgramError::TruncatedInstruction { pc })?;
            1 + usize::from(count) * if opcode == 0x40 { 1 } else { 2 }
        }
        0xb0..=0xb7 => usize::from(opcode - 0xaf),
        0xb8..=0xbf => usize::from(opcode - 0xb7) * 2,
        _ => 0,
    };
    let end = after_opcode
        .checked_add(payload)
        .filter(|end| *end <= program.len())
        .ok_or(ProgramError::TruncatedInstruction { pc })?;
    Ok(end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionDefinition<'a> {
    pub offset: u32,
    pub length: u16,
    /// Native FPGM is program selector1, stored as a word at table entry+6.
    pub program_selector: u16,
    /// Opaque original bytes excluding ENDF; no body handler executes here.
    pub body: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapDispatch {
    pub pc: usize,
    pub opcode: u8,
    pub next_pc: usize,
    pub stack_after: usize,
    pub defined_index: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bootstrap<'a> {
    /// Indexed bindings; None does not claim native unused-table bytes.
    pub functions: Vec<Option<FunctionDefinition<'a>>>,
    pub dispatches: Vec<BootstrapDispatch>,
    pub remaining_stack: Vec<i32>,
    pub peak_stack_elements: usize,
}

/// Admit an initially empty operand stack and the native FPGM selector1.
/// PUSHB/NPUSHB are unsigned; PUSHW/NPUSHW are signed big-endian words.
/// FDEF scans to the first opcode ENDF, skips PUSH data, and overwrites an
/// existing indexed binding exactly as the native store does. Other top-level
/// opcodes are unsupported, including CALL, PREP and point/hint operations.
pub fn bootstrap_fpgm(
    program: &[u8],
    limits: BootstrapLimits,
) -> Result<Bootstrap<'_>, ProgramError> {
    let mut result = Bootstrap {
        functions: vec![None; usize::from(limits.max_functions)],
        dispatches: Vec::new(),
        remaining_stack: Vec::new(),
        peak_stack_elements: 0,
    };
    let mut pc = 0;
    while pc < program.len() {
        let opcode = program[pc];
        let next_pc;
        let mut defined_index = None;
        match opcode {
            0x40 | 0x41 | 0xb0..=0xbf => {
                next_pc = instruction_end(program, pc)?;
                let variable_count = opcode == 0x40 || opcode == 0x41;
                let words = opcode == 0x41 || opcode >= 0xb8;
                let payload_start = pc + if variable_count { 2 } else { 1 };
                let payload = &program[payload_start..next_pc];
                let count = payload.len() / if words { 2 } else { 1 };
                if result.remaining_stack.len() + count > usize::from(limits.max_stack_elements) {
                    return Err(ProgramError::StackOverflow {
                        pc,
                        capacity: limits.max_stack_elements,
                    });
                }
                if words {
                    result.remaining_stack.extend(
                        payload
                            .chunks_exact(2)
                            .map(|pair| i32::from(i16::from_be_bytes([pair[0], pair[1]]))),
                    );
                } else {
                    result
                        .remaining_stack
                        .extend(payload.iter().map(|value| i32::from(*value)));
                }
                result.peak_stack_elements =
                    result.peak_stack_elements.max(result.remaining_stack.len());
            }
            0x2c => {
                let index = result
                    .remaining_stack
                    .pop()
                    .ok_or(ProgramError::StackUnderflow { pc })?;
                if index < 0 || index >= i32::from(limits.max_functions) {
                    return Err(ProgramError::FunctionIndexOutOfBounds {
                        pc,
                        index,
                        capacity: limits.max_functions,
                    });
                }
                let start = pc + 1;
                let mut end = start;
                loop {
                    let body_opcode = *program
                        .get(end)
                        .ok_or(ProgramError::MissingEndFunction { pc })?;
                    if body_opcode == 0x2d {
                        break;
                    }
                    end = instruction_end(program, end)?;
                }
                let definition = FunctionDefinition {
                    offset: start
                        .try_into()
                        .map_err(|_| ProgramError::DefinitionOffsetTooLarge { pc })?,
                    length: (end - start)
                        .try_into()
                        .map_err(|_| ProgramError::DefinitionLengthTooLarge { pc })?,
                    program_selector: 1,
                    body: &program[start..end],
                };
                result.functions[index as usize] = Some(definition);
                defined_index = Some(index as u16);
                next_pc = end + 1;
            }
            _ => return Err(ProgramError::UnsupportedOpcode { pc, opcode }),
        }
        result.dispatches.push(BootstrapDispatch {
            pc,
            opcode,
            next_pc,
            stack_after: result.remaining_stack.len(),
            defined_index,
        });
        pc = next_pc;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: BootstrapLimits = BootstrapLimits {
        max_functions: 3,
        max_stack_elements: 8,
    };

    #[test]
    fn native_push_widths_keep_endf_bytes_opaque() {
        let program = [0xb0, 2, 0x2c, 0x41, 1, 0x2d, 0x2d, 0xb1, 0x2d, 0x2d, 0x2d];
        let setup = bootstrap_fpgm(&program, LIMITS).unwrap();
        let definition = setup.functions[2].unwrap();
        assert_eq!((definition.offset, definition.length), (3, 7));
        assert_eq!(definition.program_selector, 1);
        assert_eq!(definition.body, &program[3..10]);
        assert_eq!(setup.dispatches.len(), 2);
        assert!(setup.remaining_stack.is_empty());
    }

    #[test]
    fn words_sign_extend_and_byte_pushes_remain_unsigned() {
        let program = [0x41, 3, 0xff, 0xff, 0x80, 0, 0x7f, 0xff, 0xb0, 0xff];
        let setup = bootstrap_fpgm(&program, LIMITS).unwrap();
        assert_eq!(setup.remaining_stack, [-1, -32768, 32767, 255]);
        assert_eq!(setup.peak_stack_elements, 4);
        assert_eq!(instruction_end(&program, 0), Ok(8));
        assert_eq!(instruction_end(&program, 8), Ok(10));
    }

    #[test]
    fn duplicate_definition_overwrites_only_its_indexed_binding() {
        let program = [0xb0, 1, 0x2c, 0x21, 0x2d, 0xb0, 1, 0x2c, 0x00, 0x2d];
        let setup = bootstrap_fpgm(&program, LIMITS).unwrap();
        assert_eq!(setup.functions[0], None);
        assert_eq!(setup.functions[2], None);
        assert_eq!(setup.functions[1].unwrap().body, &[0x00]);
        assert_eq!(setup.functions[1].unwrap().offset, 8);
        assert_eq!(setup.dispatches[1].defined_index, Some(1));
        assert_eq!(setup.dispatches[3].defined_index, Some(1));
    }

    #[test]
    fn owned_bounds_reject_unsafe_native_inputs_and_later_execution() {
        assert!(matches!(
            bootstrap_fpgm(&[0xb8, 0xff, 0xff, 0x2c, 0x2d], LIMITS),
            Err(ProgramError::FunctionIndexOutOfBounds { index: -1, .. })
        ));
        assert!(matches!(
            bootstrap_fpgm(&[0xb0, 3, 0x2c, 0x2d], LIMITS),
            Err(ProgramError::FunctionIndexOutOfBounds { index: 3, .. })
        ));
        assert!(matches!(
            bootstrap_fpgm(&[0x2c], LIMITS),
            Err(ProgramError::StackUnderflow { .. })
        ));
        assert!(matches!(
            bootstrap_fpgm(&[0x40, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0], LIMITS),
            Err(ProgramError::StackOverflow { .. })
        ));
        assert!(matches!(
            bootstrap_fpgm(&[0xb0, 0, 0x2c, 0x21], LIMITS),
            Err(ProgramError::MissingEndFunction { .. })
        ));
        assert_eq!(
            instruction_end(&[0xbf, 0, 0], 0),
            Err(ProgramError::TruncatedInstruction { pc: 0 })
        );
        assert_eq!(
            bootstrap_fpgm(&[0x2b], LIMITS),
            Err(ProgramError::UnsupportedOpcode {
                pc: 0,
                opcode: 0x2b,
            })
        );
    }

    #[test]
    fn ordinary_maxp_limits_and_empty_program_are_explicit() {
        let mut maxp = [0; 32];
        maxp[..4].copy_from_slice(&[0, 1, 0, 0]);
        maxp[20..22].copy_from_slice(&85u16.to_be_bytes());
        maxp[24..26].copy_from_slice(&327u16.to_be_bytes());
        let limits = BootstrapLimits::from_maxp(&maxp).unwrap();
        assert_eq!(limits.max_functions, 85);
        assert_eq!(limits.max_stack_elements, 327);
        let empty = bootstrap_fpgm(&[], limits).unwrap();
        assert_eq!(empty.functions.len(), 85);
        assert!(empty.functions.iter().all(Option::is_none));
        assert!(empty.dispatches.is_empty());
        assert!(empty.remaining_stack.is_empty());
        assert_eq!(empty.peak_stack_elements, 0);
    }
}
