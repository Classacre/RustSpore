//! Original ordinary CALL/LOOPCALL range and shared-frame foundation.
//!
//! Source 8AE800/8AE8B0 resolve a definition once and invoke the same frame
//! through its 8ABCA0 callback. This module does not execute any opcode body.
//! The caller supplies an ordinary body processor. Bounds/errors are owned
//! fail-closed policy, distinct from native unchecked access/nonlocal errors.

use crate::spui_font_hint_program::FunctionDefinition;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramWindow<'a> {
    /// Explicit modeled IA-32 address, supplied by the caller, never dereferenced.
    pub base_address: u32,
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedHintFrame {
    pub operand_stack: Vec<i32>,
    pub cursor: u32,
    pub last_opcode: u32,
    /// Source inner-state+C9. CALL does not switch this to the definition selector.
    pub program_selector: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallInvocation<'a> {
    pub function_index: u32,
    pub program_selector: u16,
    pub start_address: u32,
    pub end_address: u32,
    pub body: &'a [u8],
    pub iterations: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlError {
    StackUnderflow,
    FunctionIndexOutOfBounds(i32),
    UndefinedFunction(u32),
    MissingProgram(u16),
    DefinitionBodyMismatch,
    AddressOverflow,
}

impl fmt::Display for ControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "bounded ordinary hint call: {self:?}")
    }
}

impl std::error::Error for ControlError {}

/// Native MOV AX,[count]; DEC AX; MOVZX EBX,AX; signed TEST BX/JL.
/// Low-word counts1..32768 run that many times. Other low words run zero.
/// The high word has no effect; this is not a generic TrueType count policy.
pub fn loop_iterations(count: i32) -> u16 {
    let initial = (count as u16).wrapping_sub(1);
    if (initial as i16) < 0 {
        0
    } else {
        initial + 1
    }
}

fn resolve<'a>(
    index: i32,
    definitions: &[Option<FunctionDefinition<'a>>],
    programs: &[ProgramWindow<'a>],
) -> Result<CallInvocation<'a>, ControlError> {
    if index < 0 || index as usize >= definitions.len() {
        return Err(ControlError::FunctionIndexOutOfBounds(index));
    }
    let definition =
        definitions[index as usize].ok_or(ControlError::UndefinedFunction(index as u32))?;
    let window = programs
        .get(usize::from(definition.program_selector))
        .ok_or(ControlError::MissingProgram(definition.program_selector))?;
    let start: usize = definition
        .offset
        .try_into()
        .map_err(|_| ControlError::AddressOverflow)?;
    let end = start
        .checked_add(usize::from(definition.length))
        .ok_or(ControlError::AddressOverflow)?;
    let body = window
        .bytes
        .get(start..end)
        .filter(|body| *body == definition.body)
        .ok_or(ControlError::DefinitionBodyMismatch)?;
    let start_address = window
        .base_address
        .checked_add(definition.offset)
        .ok_or(ControlError::AddressOverflow)?;
    let end_address = start_address
        .checked_add(u32::from(definition.length))
        .ok_or(ControlError::AddressOverflow)?;
    Ok(CallInvocation {
        function_index: index as u32,
        program_selector: definition.program_selector,
        start_address,
        end_address,
        body,
        iterations: 1,
    })
}

/// Pop a full dword function index and resolve offset/length/program selector.
/// The cursor, last opcode and shared program-selector byte are unchanged.
pub fn prepare_call<'a>(
    frame: &mut SharedHintFrame,
    definitions: &[Option<FunctionDefinition<'a>>],
    programs: &[ProgramWindow<'a>],
) -> Result<CallInvocation<'a>, ControlError> {
    let index = frame
        .operand_stack
        .pop()
        .ok_or(ControlError::StackUnderflow)?;
    resolve(index, definitions, programs)
}

/// Native order is index pop, record resolution, then count-word pop.
/// Resolution still happens for a count that subsequently selects zero calls.
pub fn prepare_loop_call<'a>(
    frame: &mut SharedHintFrame,
    definitions: &[Option<FunctionDefinition<'a>>],
    programs: &[ProgramWindow<'a>],
) -> Result<CallInvocation<'a>, ControlError> {
    let mut invocation = prepare_call(frame, definitions, programs)?;
    let count = frame
        .operand_stack
        .pop()
        .ok_or(ControlError::StackUnderflow)?;
    invocation.iterations = loop_iterations(count);
    Ok(invocation)
}

/// 8ABCA0 saves/restores the incoming cursor for each ordinary range call.
/// The callback receives the same frame each time. Stack, last opcode, selector
/// and external state mutations persist. The target is resolved once before
/// any iteration; repeated calls do not re-read a changed definition table.
/// On callback Err, cursor cleanup is owned policy, not native longjmp recovery.
pub fn dispatch_ordinary<'a, E>(
    frame: &mut SharedHintFrame,
    invocation: CallInvocation<'a>,
    mut body_processor: impl FnMut(&mut SharedHintFrame, CallInvocation<'a>) -> Result<(), E>,
) -> Result<(), E> {
    for _ in 0..invocation.iterations {
        let previous = frame.cursor;
        frame.cursor = invocation.start_address;
        let result = body_processor(frame, invocation);
        frame.cursor = previous;
        result?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &[u8] = &[0xb0, 1, 0x60, 0x2d];

    fn inputs() -> (
        [Option<FunctionDefinition<'static>>; 2],
        [ProgramWindow<'static>; 2],
    ) {
        (
            [
                Some(FunctionDefinition {
                    offset: 0,
                    length: 3,
                    program_selector: 1,
                    body: &BODY[..3],
                }),
                None,
            ],
            [
                ProgramWindow {
                    base_address: 0x320000,
                    bytes: &[],
                },
                ProgramWindow {
                    base_address: 0x350000,
                    bytes: BODY,
                },
            ],
        )
    }

    fn frame(stack: Vec<i32>) -> SharedHintFrame {
        SharedHintFrame {
            operand_stack: stack,
            cursor: 0x320007,
            last_opcode: 0x2a,
            program_selector: 0,
        }
    }

    #[test]
    fn source_loop_count_uses_low_word_and_signed_post_decrement() {
        assert_eq!(loop_iterations(0), 0);
        assert_eq!(loop_iterations(1), 1);
        assert_eq!(loop_iterations(32767), 32767);
        assert_eq!(loop_iterations(32768), 32768);
        assert_eq!(loop_iterations(32769), 0);
        assert_eq!(loop_iterations(65535), 0);
        assert_eq!(loop_iterations(-32768), 32768);
        assert_eq!(loop_iterations(-32767), 0);
        assert_eq!(loop_iterations(65537), 1);
        assert_eq!(loop_iterations(i32::MIN + 1), 1);
    }

    #[test]
    fn ordinary_call_shares_stack_and_last_opcode_but_restores_cursor() {
        let (definitions, programs) = inputs();
        let mut state = frame(vec![i32::MAX, 0]);
        let invocation = prepare_call(&mut state, &definitions, &programs).unwrap();
        assert_eq!(
            (invocation.start_address, invocation.end_address),
            (0x350000, 0x350003)
        );
        assert_eq!(state.operand_stack, [i32::MAX]);
        let mut visits = 0;
        dispatch_ordinary(&mut state, invocation, |shared, target| {
            assert_eq!(shared.cursor, target.start_address);
            shared.operand_stack[0] = shared.operand_stack[0].wrapping_add(1);
            shared.last_opcode = 0x60;
            shared.cursor = target.end_address;
            visits += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        assert_eq!(visits, 1);
        assert_eq!(state.operand_stack, [i32::MIN]);
        assert_eq!(state.last_opcode, 0x60);
        assert_eq!(state.cursor, 0x320007);
        assert_eq!(state.program_selector, 0);
    }

    #[test]
    fn repeated_calls_keep_mutations_and_zero_calls_preserve_frame() {
        let (definitions, programs) = inputs();
        let mut state = frame(vec![7, 3, 0]);
        let invocation = prepare_loop_call(&mut state, &definitions, &programs).unwrap();
        dispatch_ordinary(&mut state, invocation, |shared, _| {
            shared.operand_stack[0] += 1;
            shared.program_selector += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        assert_eq!(state.operand_stack, [10]);
        assert_eq!(state.program_selector, 3);
        let mut zero = frame(vec![7, 32769, 0]);
        let invocation = prepare_loop_call(&mut zero, &definitions, &programs).unwrap();
        dispatch_ordinary(&mut zero, invocation, |_, _| -> Result<(), ()> {
            panic!("source count excludes body entry")
        })
        .unwrap();
        assert_eq!(zero, frame(vec![7]));
    }

    #[test]
    fn resolution_precedes_count_pop_and_owned_guards_are_explicit() {
        let (definitions, programs) = inputs();
        let mut undefined = frame(vec![7, 0, 1]);
        assert_eq!(
            prepare_loop_call(&mut undefined, &definitions, &programs),
            Err(ControlError::UndefinedFunction(1))
        );
        assert_eq!(undefined.operand_stack, [7, 0]);
        let mut negative = frame(vec![-1]);
        assert_eq!(
            prepare_call(&mut negative, &definitions, &programs),
            Err(ControlError::FunctionIndexOutOfBounds(-1))
        );
        let mut underflow = frame(vec![0]);
        assert_eq!(
            prepare_loop_call(&mut underflow, &definitions, &programs),
            Err(ControlError::StackUnderflow)
        );
    }

    #[test]
    fn callback_error_restores_only_cursor_as_owned_cleanup() {
        let (definitions, programs) = inputs();
        let mut state = frame(vec![7, 0]);
        let invocation = prepare_call(&mut state, &definitions, &programs).unwrap();
        let error = dispatch_ordinary(&mut state, invocation, |shared, _| {
            shared.cursor = 0;
            shared.operand_stack[0] = 99;
            Err("unsupported body handler")
        });
        assert_eq!(error, Err("unsupported body handler"));
        assert_eq!(state.cursor, 0x320007);
        assert_eq!(state.operand_stack, [99]);
    }
}
