//! Compare the source data2 consumer; finite errors retain their native wire.
use spore_core::{
    cell_runtime_data::{CellRuntimeData, CellRuntimeDataError, DATA2_ID},
    cell_runtime_modifiers::{data2_multiplier, ActorAnimationData},
    native_x87::{Context, X87Error},
};
use std::{error::Error, fs::File, io::{BufReader, BufWriter, Read, Write}};

fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at+4].try_into().unwrap())
}
fn half(data: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(data[at..at+2].try_into().unwrap())
}
fn finite(bits: u32) -> bool {
    bits & 0x7f80_0000 != 0x7f80_0000
}
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
fn mxcsr() -> u32 {
    let mut value = 0;
    // SAFETY: STMXCSR writes one local DWORD.
    unsafe { std::arch::asm!("stmxcsr [{v}]",v=in(reg)&mut value,options(nostack,preserves_flags)); }
    value
}
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
fn set_mxcsr(value: u32) {
    // SAFETY: callers use only valid masked control words below.
    unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&value,options(nostack,preserves_flags)); }
}
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
fn x87_state() -> (u16, u16) {
    let (mut cw, mut sw) = (0u16, 0u16);
    // SAFETY: non-popping snapshots write two local words.
    unsafe { std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut cw,s=in(reg)&mut sw,options(nostack,preserves_flags)); }
    (cw, sw)
}
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
struct Restore(u32);
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
impl Drop for Restore { fn drop(&mut self) { set_mxcsr(self.0); } }

fn evaluate(request: &[u8; 200]) -> Result<[u8; 20], CellRuntimeDataError> {
    let context = Context::from_control_word(half(request, 16))?;
    let result = data2_multiplier(word(request, 0) as i32, ActorAnimationData {
        actor_word_fc: word(request, 4),
        actor_handle_248: word(request, 8),
        resolved_aggregate: if word(request, 12) == 0 { None } else {
            Some(CellRuntimeData::from_aggregate_a0(&request[36..])?)
        },
    }, context)?;
    let mut output = [0; 20];
    output[..10].copy_from_slice(&result.retained.to_le_bytes());
    output[12..16].copy_from_slice(&context.store_f32(result.retained)?.to_bits().to_le_bytes());
    output[16..20].copy_from_slice(&u32::from(result.manager_lookup_called).to_le_bytes());
    Ok(output)
}

fn expected_error(request: &[u8; 200], native: &[u8; 28]) -> Option<CellRuntimeDataError> {
    if matches!(word(request, 0), 0x12|0x13|0x14|0x30) || word(request, 4) == 0
        || word(request, 8) == 0 || word(request, 12) == 0 {
        return None;
    }
    let count = word(request, 36) as i32;
    let index = (0..count.max(0) as usize).find(|i| word(request, 40+20*i) == DATA2_ID)?;
    if !finite(word(request, 48+20*index)) || !finite(word(request, 56+20*index)) {
        Some(CellRuntimeDataError::X87(X87Error::NonFiniteInput))
    } else if !finite(word(native, 12)) {
        Some(CellRuntimeDataError::X87(X87Error::NonFiniteResult))
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: cell_runtime_modifiers_verify input.bin native.bin modeled.bin policy.tsv".into());
    }
    #[cfg(not(any(target_arch="x86", target_arch="x86_64")))]
    return Err("x86 target required".into());
    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    let _restore = Restore(mxcsr());
    let mut inputs = BufReader::new(File::open(&args[0])?);
    let mut native = BufReader::new(File::open(&args[1])?);
    let mut output = BufWriter::new(File::create(&args[2])?);
    let mut projection = BufWriter::new(File::create(std::path::Path::new(&args[2]).with_extension("native-projection.bin"))?);
    let mut policies = BufWriter::new(File::create(&args[3])?);
    writeln!(policies,"case\terror\tnative_retained80\tnative_committed32\tmanager_calls")?;
    let (mut cases, mut modeled, mut policy) = (0usize, 0usize, 0usize);
    loop {
        let mut req = [0; 200];
        if inputs.read(&mut req[..1])? == 0 { break; }
        inputs.read_exact(&mut req[1..])?;
        let mut reply = [0; 28];
        native.read_exact(&mut reply)?;
        let cw = word(&req, 16);
        if word(&req, 12) > 1 || cw & 0xffff_f0ff != 0x7f || cw & 0x300 == 0x100
            || word(&req, 36) as i32 > 8 || (20..36).step_by(4).any(|at| word(&req, at) != 0)
            || u32::from(half(&reply, 20)) != cw || half(&reply, 22) & 0x3800 != 0
            || word(&reply, 24) & !0x3f != 0x1f80 || half(&reply, 10) != 0 {
            return Err(format!("invalid wire/context atcase{cases}").into());
        }
        let mut first = None;
        for incoming in [0x1f80, 0xdf81, 0xbf84] {
            #[cfg(any(target_arch="x86", target_arch="x86_64"))]
            set_mxcsr(incoming);
            #[cfg(any(target_arch="x86", target_arch="x86_64"))]
            let before = x87_state();
            let result = evaluate(&req);
            #[cfg(any(target_arch="x86", target_arch="x86_64"))]
            if mxcsr() != incoming || x87_state() != before {
                return Err(format!("caller floating environment changed atcase{cases}").into());
            }
            if first.as_ref().is_some_and(|old| old != &result) {
                return Err(format!("caller MXCSR affects result atcase{cases}").into());
            }
            first = Some(result);
        }
        match (first.unwrap(), expected_error(&req, &reply)) {
            (Ok(actual), None) => {
                if actual != reply[..20] {
                    let at = actual.iter().zip(&reply).position(|(a,b)| a != b).unwrap();
                    return Err(format!("case{cases} differs atbyte{at}").into());
                }
                output.write_all(&actual)?;
                projection.write_all(&reply[..20])?;
                modeled += 1;
            }
            (Err(actual), Some(expected)) if actual == expected => {
                let raw = reply[..10].iter().map(|b|format!("{b:02x}")).collect::<String>();
                writeln!(policies,"{cases}\t{actual:?}\t{raw}\t{:08X}\t{}",word(&reply,12),word(&reply,16))?;
                policy += 1;
            }
            (actual, expected) => return Err(format!("case{cases} errorpolicy differs:{actual:?}/{expected:?}").into()),
        }
        cases += 1;
    }
    if cases == 0 || native.read(&mut [0])? != 0 { return Err("empty input or trailing replies".into()); }
    output.flush()?;
    projection.flush()?;
    policies.flush()?;
    println!("CELL_RUNTIME_MODIFIERS_VERIFY cases={cases} modeled={modeled} policy={policy} caller_states=3 x87_restored=true exact_bits=true");
    Ok(())
}
