//! Pending binary200/180-byte comparison; no admitted native/Rust run yet.
//! AL+value+whole aggregate are172 modeled bytes;8 native FP bytes checked.
//! Reached finite-API errors retain their native outcomes in a policy TSV.
use spore_core::{
    cell_runtime_data::{
        CellRuntimeData, CellRuntimeDataAccumulator, CellRuntimeDataError, AGGREGATE_SIZE,
    },
    native_sse::Policy,
    native_x87::{Context, X87Error},
};
use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};
const INPUT: usize = 200;
const OUTPUT: usize = 180;
const MODELED: usize = 172;
fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
fn half(data: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(data[at..at + 2].try_into().unwrap())
}
fn finite_bits(bits: u32) -> bool {
    bits & 0x7f80_0000 != 0x7f80_0000
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mxcsr() -> u32 {
    let mut value = 0;
    unsafe {
        std::arch::asm!("stmxcsr [{value}]", value=in(reg)&mut value, options(nostack,preserves_flags));
    }
    value
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn set_mxcsr(value: u32) {
    unsafe {
        std::arch::asm!("ldmxcsr [{value}]", value=in(reg)&value, options(nostack,preserves_flags));
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct RestoreMxcsr(u32);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Drop for RestoreMxcsr {
    fn drop(&mut self) {
        set_mxcsr(self.0);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    Modeled([u8; MODELED]),
    CheckedError(CellRuntimeDataError),
}
fn evaluate(request: &[u8; INPUT]) -> Result<Outcome, Box<dyn Error>> {
    let mode = word(request, 0);
    let id = word(request, 4);
    let context = Context::from_control_word(half(request, 8))?;
    let aggregate: [u8; AGGREGATE_SIZE] = request[36..].try_into().unwrap();
    let mut result = [0; MODELED];
    if mode == 0 {
        match CellRuntimeData::from_aggregate_a0(&aggregate)?.lookup(id, context) {
            Ok(sample) => {
                result[..4].copy_from_slice(&u32::from(sample.found).to_le_bytes());
                result[4..8].copy_from_slice(&sample.value.to_bits().to_le_bytes());
                result[8..].copy_from_slice(&aggregate);
            }
            Err(error) => return Ok(Outcome::CheckedError(error)),
        }
    } else {
        let mut data = CellRuntimeDataAccumulator::from_snapshot(aggregate)?;
        let written = if mode == 1 {
            data.accumulate_scalar(
                id,
                word(request, 12),
                word(request, 16),
                word(request, 24),
                Policy::Mxcsr1f80,
            )?
        } else if mode == 2 {
            data.accumulate_pair(
                id,
                word(request, 12),
                word(request, 16),
                word(request, 20),
                word(request, 24),
                Policy::Mxcsr1f80,
            )?
        } else {
            return Err("invalid native input mode".into());
        };
        result[..4].copy_from_slice(&u32::from(written.accepted).to_le_bytes());
        result[4..8].copy_from_slice(&0xa5a5_a5a5u32.to_le_bytes());
        result[8..].copy_from_slice(data.raw_bytes());
    }
    Ok(Outcome::Modeled(result))
}
fn expected_error(request: &[u8; INPUT], native: &[u8; OUTPUT]) -> Option<CellRuntimeDataError> {
    if word(request, 0) != 0 || word(native, 0) == 0 {
        return None;
    }
    let count = word(request, 36) as i32;
    let index = (0..count.max(0) as usize)
        .find(|&i| word(request, 40 + i * 20) == word(request, 4))
        .expect("nativeAL1 must reach an ID");
    let at = 40 + index * 20;
    if !finite_bits(word(request, at + 8)) || !finite_bits(word(request, at + 16)) {
        Some(CellRuntimeDataError::X87(X87Error::NonFiniteInput))
    } else if !finite_bits(word(native, 4)) {
        Some(CellRuntimeDataError::X87(X87Error::NonFiniteResult))
    } else {
        None
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "usage: cell_runtime_data_verify input.bin native-output.bin modeled.bin policy.tsv"
                .into(),
        );
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    return Err("x86 SSE/x87 target required".into());
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let _restore = RestoreMxcsr(mxcsr());
    let mut requests = BufReader::new(File::open(&args[0])?);
    let mut expected = BufReader::new(File::open(&args[1])?);
    let mut output = BufWriter::new(File::create(&args[2])?);
    let mut projection = BufWriter::new(File::create(
        std::path::Path::new(&args[2]).with_extension("native-projection.bin"),
    )?);
    let mut policies = BufWriter::new(File::create(&args[3])?);
    writeln!(policies, "case\terror\tnative_AL\tnative_value_bits")?;
    let (mut count, mut modeled_count, mut policy_count) = (0usize, 0usize, 0usize);
    loop {
        let mut request = [0; INPUT];
        if requests.read(&mut request[..1])? == 0 {
            break;
        }
        requests.read_exact(&mut request[1..])?;
        let mut native = [0; OUTPUT];
        expected.read_exact(&mut native)?;
        let mode = word(&request, 0);
        let cw = half(&request, 8);
        let rows = word(&request, 36) as i32;
        if mode > 2
            || cw & 0xf0ff != 0x007f
            || cw & 0x300 == 0x100
            || half(&request, 10) != 0
            || word(&request, 28) != 0
            || word(&request, 32) != 0
            || rows > 8
            || (mode != 0 && rows < 0)
        {
            return Err(format!("malformed native input at case{count}").into());
        }
        if half(&native, 172) != cw
            || half(&native, 174) & 0x3800 != 0
            || word(&native, 176) & !0x3f != 0x1f80
            || word(&native, 0) > 1
        {
            return Err(format!("invalid native metadata at case{count}").into());
        }
        if mode == 0 && native[8..MODELED] != request[36..] {
            return Err(format!("native reader changed aggregate at case{count}").into());
        }
        let mut actual = None;
        for incoming in [0x1f80, 0xdf81, 0xbf84] {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            set_mxcsr(incoming);
            let next = evaluate(&request)?;
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if mxcsr() != incoming {
                return Err(format!("caller MXCSR changed at case{count}").into());
            }
            if actual.as_ref().is_some_and(|old| old != &next) {
                return Err(format!("incoming state changes result at case{count}").into());
            }
            actual = Some(next);
        }
        match (actual.unwrap(), expected_error(&request, &native)) {
            (Outcome::Modeled(actual), None) => {
                if actual != native[..MODELED] {
                    let byte = actual
                        .iter()
                        .zip(&native)
                        .position(|(a, b)| a != b)
                        .unwrap();
                    return Err(format!("case{count} differs at modeled byte{byte}").into());
                }
                output.write_all(&actual)?;
                projection.write_all(&native[..MODELED])?;
                modeled_count += 1;
            }
            (Outcome::CheckedError(actual), Some(expected)) if actual == expected => {
                writeln!(
                    policies,
                    "{count}\t{actual:?}\t{}\t{:08X}",
                    word(&native, 0),
                    word(&native, 4)
                )?;
                policy_count += 1;
            }
            (actual, expected) => {
                return Err(format!("case{count} policy differs: {actual:?}/{expected:?}").into())
            }
        }
        count += 1;
    }
    let mut trailing = [0];
    if count == 0 || expected.read(&mut trailing)? != 0 {
        return Err("empty input or extra native replies".into());
    }
    output.flush()?;
    projection.flush()?;
    policies.flush()?;
    println!("CELL_RUNTIME_DATA_VERIFY cases={count} modeled={modeled_count} policy={policy_count} caller_states=3 exact_bits=true");
    Ok(())
}
