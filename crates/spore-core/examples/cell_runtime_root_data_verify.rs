//! Bounded source multi-component join; root exports after static review.
use spore_core::{
    anim,
    cell_runtime_contributions::CellRuntimeScalarSlot,
    cell_runtime_root_data::{evaluate_single_clip_root_data, source_frame, ConsumerContext, ConsumerOutcome,
        FrameInput, PreparedChannel, RootBinding, RootDataContext, RootDataError},
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error,fs::{self,File,OpenOptions},io::{Read,Write},path::Path};
fn word(raw:&[u8],at:usize)->u32 {u32::from_le_bytes(raw[at..at+4].try_into().unwrap())}
fn half(raw:&[u8],at:usize)->u16 {u16::from_le_bytes(raw[at..at+2].try_into().unwrap())}
fn csr()->u32 {let mut x=0;unsafe {std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut x,options(nostack,preserves_flags));} x}
fn set_csr(x:u32) {unsafe {std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&x,options(nostack,preserves_flags));}}
struct Restore(u32);
impl Drop for Restore {fn drop(&mut self){set_csr(self.0);}}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=4 {return Err("usage: FIXTURE-DIR CONTRIBUTION.bin CONSUMER.bin RUST.bin".into());}
    let root=Path::new(&args[0]);
    let table=fs::read_to_string(root.join("cases.tsv"))?;
    let mut contribution=File::open(&args[1])?;
    let mut consumer=File::open(&args[2])?;
    let mut requests=File::open(root.join("contribution-input.bin"))?;
    let mut output=OpenOptions::new().write(true).create_new(true).open(&args[3])?;
    let _restore=Restore(csr());
    let context=RootDataContext {x87:Context::from_control_word(0x027f)?,sse:Policy::Mxcsr1f80};
    for incoming in [0x1f80,0xdf81,0xbf84] {
        set_csr(incoming);
        assert_eq!(source_frame(FrameInput::NonLoopFirstDuration {time_bits:0x3f000000,
            duration_bits:1f64.to_bits(),resource_frames_bits:0x41200000},context),
            Err(RootDataError::UnsupportedClockDomain));
        assert_eq!(csr(),incoming);
    }
    let mut cases=0;
    for line in table.lines().skip(1) {
        let fields:Vec<_>=line.split('\t').collect();
        if fields.len()!=10 || fields[0].parse::<usize>()?!=cases {return Err("malformed case table".into());}
        let animation=anim::parse(&fs::read(root.join(fields[1]))?)?;
        let channel=fields[3].parse::<usize>()?;
        let frame=u32::from_str_radix(fields[4],16)?;
        assert_eq!(fields[5],"027F");
        let mut prepared:Vec<_>=animation.bones.iter().map(|bone|PreparedChannel {
            descriptors:bone.components.clone(),prior_slots:[CellRuntimeScalarSlot {
                value_bits:0xa5a5a5a5,weight_bits:0xa5a5a5a5 };6] }).collect();
        let raw=fs::read(root.join(fields[2]))?;
        assert_eq!(raw.len(),prepared[channel].descriptors.len()*32);
        for (i,descriptor)in prepared[channel].descriptors.iter_mut().enumerate() {
            let at=32*i;
            assert_eq!(descriptor.tag,raw[at+4..at+8]);
            assert_eq!(descriptor.offset,word(&raw,at+8));
            assert_eq!(descriptor.stride,word(&raw,at+12));
            descriptor.flags=word(&raw,at);
            descriptor.slot=word(&raw,at+16);
        }
        let binding=RootBinding {record:std::array::from_fn(|i|u32::from_str_radix(fields[6+i],16).unwrap()),
            prior_position_mask:0,prior_rotation_mask:0};
        let mut expected_contribution=[0;184];contribution.read_exact(&mut expected_contribution)?;
        let mut expected_consumer=[0;28];consumer.read_exact(&mut expected_consumer)?;
        let mut request=[0;1268];requests.read_exact(&mut request)?;
        assert_eq!(half(&expected_contribution,176),0x027f);
        assert_eq!(word(&expected_contribution,180)&!0x3f,0x1f80);
        assert_eq!(half(&expected_consumer,20),0x027f);
        assert_eq!(word(&expected_consumer,24)&!0x3f,0x1f80);
        let mut computed=None;
        for incoming in [0x1f80,0xdf81,0xbf84] {
            set_csr(incoming);
            let result=evaluate_single_clip_root_data(&animation,&prepared,&[binding],1u64<<channel,
                1u64<<channel,0,FrameInput::ExplicitSourceFrame(frame),[0xa5;164],
                ConsumerContext::ExplicitResolvedRoot {actor_state_18c:0,actor_handle_248:1,actor_word_fc:1},context)?;
            assert_eq!(csr(),incoming);
            if result.aggregate.raw_bytes()!=&expected_contribution[..164] {
                return Err(format!("aggregate differs case{cases}").into());
            }
            let mut actual_slots=Vec::new();
            for slot in result.sampled_slots[channel] {
                actual_slots.extend(slot.value_bits.to_le_bytes());actual_slots.extend(slot.weight_bits.to_le_bytes());
            }
            assert_eq!(actual_slots,request[196..244],"sample slot join case{cases}");
            let ConsumerOutcome::Evaluated(multiplier)=result.consumer else {return Err("explicit route unresolved".into());};
            let mut actual_consumer=[0;20];
            actual_consumer[..10].copy_from_slice(&multiplier.retained.to_le_bytes());
            actual_consumer[12..16].copy_from_slice(&context.x87.store_f32(multiplier.retained)?.to_bits().to_le_bytes());
            actual_consumer[16..].copy_from_slice(&u32::from(multiplier.manager_lookup_called).to_le_bytes());
            assert_eq!(actual_consumer,expected_consumer[..20],"consumer differs case{cases}");
            let mut actual=Vec::new();actual.extend(result.aggregate.raw_bytes());actual.extend(actual_slots);actual.extend(actual_consumer);
            if let Some(old)=&computed {assert_eq!(&actual,old);}computed=Some(actual);
        }
        output.write_all(&computed.unwrap())?;
        let unresolved=evaluate_single_clip_root_data(&animation,&prepared,&[binding],1u64<<channel,
            1u64<<channel,0,FrameInput::ExplicitSourceFrame(frame),[0xa5;164],
            ConsumerContext::UnresolvedActorAssociation,context)?;
        assert_eq!(unresolved.consumer,ConsumerOutcome::UnresolvedActorAssociation);
        let mut nonroot=binding;nonroot.record[3]|=1;
        assert!(matches!(evaluate_single_clip_root_data(&animation,&prepared,&[nonroot],1u64<<channel,
            1u64<<channel,0,FrameInput::ExplicitSourceFrame(frame),[0xa5;164],
            ConsumerContext::UnresolvedActorAssociation,context),Err(RootDataError::ReachedNonRootPose {binding:0,pose:1})));
        cases+=1;
    }
    if cases==0 || contribution.read(&mut[0])?!=0 || consumer.read(&mut[0])?!=0 || requests.read(&mut[0])?!=0 {
        return Err("empty cohort or trailing reference bytes".into());
    }
    output.flush()?;
    println!("CELL_RUNTIME_ROOT_DATA_VERIFY cases={cases} caller_states=3 source_multicomponent=true actor_route_explicit=true unresolved_route_checked=true nonroot_positive_errors={cases} unverified_clock_rejections=3");
    Ok(())
}
