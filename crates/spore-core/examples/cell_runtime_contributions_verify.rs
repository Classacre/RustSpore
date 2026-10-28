//! Frozen1268/184 protocol comparison. Root exports this pending example.
use spore_core::{
    anim::AnimComponent,
    cell_runtime_contributions::{accumulate_scalar_components, CellRuntimeContributionContext,
        CellRuntimeContributionError as E, CellRuntimeScalarSlot},
    cell_runtime_data::CellRuntimeDataAccumulator,
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs::File, io::{BufReader, BufWriter, Read, Write}};
const INPUT: usize = 1268;
const OUTPUT: usize = 184;
fn word(raw: &[u8], at: usize) -> u32 { u32::from_le_bytes(raw[at..at+4].try_into().unwrap()) }
fn half(raw: &[u8], at: usize) -> u16 { u16::from_le_bytes(raw[at..at+2].try_into().unwrap()) }
fn descriptor(flags: u32, slot: u32) -> AnimComponent {
    AnimComponent { flags, tag: 0x98c942b5u32.to_le_bytes(), offset: 0, stride: 40, slot, reserved: [0;3] }
}
fn context(cw: u16) -> CellRuntimeContributionContext {
    CellRuntimeContributionContext { x87: Context::from_control_word(cw).unwrap(), sse: Policy::Mxcsr1f80 }
}
fn empty() -> CellRuntimeDataAccumulator {
    CellRuntimeDataAccumulator::from_snapshot([0;164]).unwrap()
}
fn slots() -> [CellRuntimeScalarSlot;6] {
    [CellRuntimeScalarSlot { value_bits: 0x3f000000, weight_bits: 0x3f800000 };6]
}
fn current_mxcsr() -> u32 {
    let mut result = 0;
    unsafe { std::arch::asm!("stmxcsr [{p}]", p=in(reg)&mut result, options(nostack,preserves_flags)); }
    result
}
fn set_mxcsr(value: u32) {
    unsafe { std::arch::asm!("ldmxcsr [{p}]", p=in(reg)&value, options(nostack,preserves_flags)); }
}
struct Restore(u32);
impl Drop for Restore { fn drop(&mut self) { set_mxcsr(self.0); } }
fn evaluate(request: &[u8;INPUT]) -> Result<[u8;176],Box<dyn Error>> {
    let count = word(request,20) as usize;
    let mut descriptors = Vec::new();
    for i in 0..count {
        let at = 244+32*i;
        descriptors.push(AnimComponent { flags: word(request,at), tag: request[at+4..at+8].try_into().unwrap(),
            offset: word(request,at+8), stride: word(request,at+12), slot: word(request,at+16),
            reserved: [word(request,at+20),word(request,at+24),word(request,at+28)] });
    }
    let slots = std::array::from_fn(|i| CellRuntimeScalarSlot {
        value_bits: word(request,196+8*i), weight_bits: word(request,200+8*i) });
    let mut aggregate = CellRuntimeDataAccumulator::from_snapshot(request[32..196].try_into().unwrap())?;
    let result = accumulate_scalar_components(&mut aggregate,&descriptors,&slots,word(request,8),
        word(request,12),word(request,16),word(request,0),context(half(request,4)))?;
    let mut output = [0;176];
    output[..164].copy_from_slice(aggregate.raw_bytes());
    output[164..168].copy_from_slice(&result.remaining_mask.to_le_bytes());
    let last = result.contributions.last().map_or(0xa5a5a5a5,|x| x.scaled_weight_bits);
    output[168..172].copy_from_slice(&last.to_le_bytes());
    output[172..].copy_from_slice(&u32::from(result.active).to_le_bytes());
    Ok(output)
}
fn positive_policy_gate() -> Result<usize,Box<dyn Error>> {
    let mut checks = 0;
    for incoming in [0x1f80,0xdf81,0xbf84] {
        set_mxcsr(incoming);
        let tests = [
            (vec![descriptor(3,3)],0x1000000,E::BindingMaskOutsideLow24(0x1000000)),
            (vec![descriptor(3,3);33],1,E::DescriptorCapacity(33)),
            (vec![descriptor(0x43,3)],1,E::AngularDescriptor(0)),
            (vec![descriptor(3,2)],1,E::SampleSlot { descriptor:0,slot:2 }),
            (vec![descriptor(3,9)],1,E::SampleSlot { descriptor:0,slot:9 }),
            (vec![descriptor(3,u32::MAX)],1,E::SampleSlot { descriptor:0,slot:u32::MAX }),
        ];
        for (descriptors,mask,expected) in tests {
            let mut data = empty();
            let before = data.clone();
            let actual = accumulate_scalar_components(&mut data,&descriptors,&slots(),mask,0,0,0x3f800000,context(0x027f));
            assert_eq!(actual,Err(expected));
            assert_eq!(data,before);
            assert_eq!(current_mxcsr(),incoming);
            checks += 1;
        }
        // Earlier accepted writes remain after a later checked unsupported field.
        let mut data = empty();
        let actual = accumulate_scalar_components(&mut data,&[descriptor(3,3),descriptor(0x43,3)],
            &slots(),3,0,0,0x3f800000,context(0x027f));
        assert_eq!(actual,Err(E::AngularDescriptor(1)));
        assert_eq!(word(data.raw_bytes(),0),1);
        assert_eq!(word(data.raw_bytes(),12),0x3f000000);
        assert_eq!(current_mxcsr(),incoming);
        checks += 1;
        // Outer skip precedes descriptor/mask domain checks.
        let result = accumulate_scalar_components(&mut data,&vec![descriptor(0x43,0);33],&slots(),
            u32::MAX,0,0,0x80000000,context(0x027f))?;
        assert!(!result.active && result.contributions.is_empty());
        assert_eq!(current_mxcsr(),incoming);
    }
    Ok(checks)
}
fn main() -> Result<(),Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len()!=3 { return Err("usage: INPUT.bin NATIVE.bin RUST-PROJECTION.bin".into()); }
    let _restore = Restore(current_mxcsr());
    let policies = positive_policy_gate()?;
    let mut requests = BufReader::new(File::open(&args[0])?);
    let mut replies = BufReader::new(File::open(&args[1])?);
    let mut output = BufWriter::new(File::create_new(&args[2])?);
    let mut cases = 0;
    loop {
        let mut request = [0;INPUT];
        if requests.read(&mut request[..1])? == 0 { break; }
        requests.read_exact(&mut request[1..])?;
        let mut native = [0;OUTPUT];
        replies.read_exact(&mut native)?;
        let cw = half(&request,4);
        if cw & 0xf0ff != 0x7f || cw & 0x300 == 0x100 || word(&request,20)>32
            || word(&request,32)>8 || half(&request,6)!=0 || word(&request,24)!=0 || word(&request,28)!=0 {
            return Err(format!("invalid input protocol case{cases}").into());
        }
        if half(&native,176)!=cw || half(&native,178)&0x3800!=0
            || word(&native,180)&!0x3f!=0x1f80 || word(&native,172)>1 {
            return Err(format!("invalid native metadata case{cases}").into());
        }
        let mut modeled = [0u8;176];
        for incoming in [0x1f80,0xdf81,0xbf84] {
            set_mxcsr(incoming);
            let actual = evaluate(&request)?;
            if current_mxcsr()!=incoming { return Err(format!("MXCSR not restored case{cases}").into()); }
            if actual!=native[..176] {
                let byte = actual.iter().zip(&native).position(|(a,b)|a!=b).unwrap();
                return Err(format!("case{cases} modeled byte{byte} Rust{:02X} native{:02X}",actual[byte],native[byte]).into());
            }
            modeled = actual;
        }
        output.write_all(&modeled)?;
        cases += 1;
    }
    if cases==0 || replies.read(&mut [0])?!=0 { return Err("empty input or trailing native bytes".into()); }
    output.flush()?;
    println!("CELL_RUNTIME_CONTRIBUTIONS_VERIFY cases={cases} raw_nonfinite_compared=true positive_policy_checks={policies} caller_states=3 exact_bits=true");
    Ok(())
}
