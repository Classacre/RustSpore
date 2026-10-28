//! Fresh native clock -> curves -> contribution -> data2 composition gate.
use spore_core::{anim,
    cell_runtime_clock::{ClockInput,ClockTime,ClockError},
    cell_runtime_source_clock_root_data::{evaluate_source_clocked_root_data,SourceClockRootDataError,SourceMetadataAdmission},
    cell_runtime_clock_root_data::ClockRootDataError,
    cell_runtime_contributions::CellRuntimeScalarSlot,
    cell_runtime_root_data::{ConsumerContext,ConsumerOutcome,PreparedChannel,RootBinding,RootDataContext,
        source_frame,FrameInput,RootDataError},
    native_sse::Policy,native_x87::Context};
use std::{error::Error,fs::{self,File,OpenOptions},io::{Read,Write},path::Path};
fn word(raw:&[u8],at:usize)->u32 {u32::from_le_bytes(raw[at..at+4].try_into().unwrap())}
fn wide(raw:&[u8],at:usize)->u64 {u64::from_le_bytes(raw[at..at+8].try_into().unwrap())}
fn half(raw:&[u8],at:usize)->u16 {u16::from_le_bytes(raw[at..at+2].try_into().unwrap())}
fn csr()->u32 {let mut x=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut x,options(nostack,preserves_flags));}x}
fn set_csr(x:u32) {unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&x,options(nostack,preserves_flags));}}
struct Restore(u32);impl Drop for Restore {fn drop(&mut self){set_csr(self.0);}}
fn metadata(raw:&[u8],at:usize) {assert_eq!(half(raw,at),0x027f);assert_eq!(half(raw,at+2)&0x3800,0);assert_eq!(word(raw,at+4)&!0x3f,0x1f80);}
fn slots_bytes(slots:[CellRuntimeScalarSlot;6])->Vec<u8> {slots.iter().flat_map(|s|[s.value_bits.to_le_bytes(),s.weight_bits.to_le_bytes()].concat()).collect()}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=7 {return Err("usage: FINAL-STAGE-DIR METADATA.bin CLOCK.bin CURVE.bin CONTRIBUTION.bin CONSUMER.bin RUST.bin".into());}
    let meta_native=fs::read(&args[1])?;assert_eq!(meta_native.len(),5*24);
    let root=Path::new(&args[0]);let table=fs::read_to_string(root.join("cases.tsv"))?;
    let mut clock_inputs=File::open(root.join("clock-input.bin"))?;
    let mut curve_inputs=File::open(root.join("curve-input.bin"))?;
    let mut contribution_inputs=File::open(root.join("contribution-input.bin"))?;
    let mut consumer_inputs=File::open(root.join("consumer-input.bin"))?;
    let mut clocks=File::open(&args[2])?;let mut curves=File::open(&args[3])?;
    let mut contributions=File::open(&args[4])?;let mut consumers=File::open(&args[5])?;
    let mut output=OpenOptions::new().write(true).create_new(true).open(&args[6])?;
    let mut projection=OpenOptions::new().write(true).create_new(true).open(Path::new(&args[6]).with_extension("native-projection.bin"))?;
    let _restore=Restore(csr());let context=RootDataContext {x87:Context::from_control_word(0x027f)?,sse:Policy::Mxcsr1f80};
    let(mut cases,mut curve_cases)=(0,0);
    for line in table.lines().skip(1) {
        let f:Vec<_>=line.split('\t').collect();assert_eq!(f.len(),10);assert_eq!(f[0].parse::<usize>()?,cases);
        let payload=fs::read(root.join(f[1]))?;assert_eq!(word(&payload,8),25);let animation=anim::parse(&payload)?;
        let meta_index=f[9].parse::<usize>()?;let meta_reply=&meta_native[24*meta_index..24*(meta_index+1)];metadata(meta_reply,16);let channel=f[3].parse::<usize>()?;
        let mut prepared:Vec<_>=animation.bones.iter().map(|bone|PreparedChannel {descriptors:bone.components.clone(),
            prior_slots:[CellRuntimeScalarSlot {value_bits:0xa5a5a5a5,weight_bits:0xa5a5a5a5};6]}).collect();
        let raw=fs::read(root.join(f[2]))?;assert_eq!(raw.len(),prepared[channel].descriptors.len()*32);
        for(i,d)in prepared[channel].descriptors.iter_mut().enumerate() {let at=32*i;
            assert_eq!(d.tag,raw[at+4..at+8]);assert_eq!(d.offset,word(&raw,at+8));assert_eq!(d.stride,word(&raw,at+12));
            d.flags=word(&raw,at);d.slot=word(&raw,at+16);}
        let binding=RootBinding {record:std::array::from_fn(|i|u32::from_str_radix(f[5+i],16).unwrap()),prior_position_mask:0,prior_rotation_mask:0};
        let mut clock_request=[0;64];clock_inputs.read_exact(&mut clock_request)?;
        let mut clock_reply=[0;28];clocks.read_exact(&mut clock_reply)?;metadata(&clock_reply,20);
        let input=ClockInput {time:match word(&clock_request,0) {0=>ClockTime::Absolute {time_bits:word(&clock_request,8)},
            1=>ClockTime::Advance {previous_time_bits:wide(&clock_request,12),delta_bits:word(&clock_request,8)},_=>return Err("bad clock mode".into())},
            duration_bits:wide(&clock_request,20),resource_frames_bits:word(&clock_request,28),looping:false};
        assert_eq!(input.duration_bits,wide(meta_reply,4));assert_eq!(input.resource_frames_bits,animation.duration.to_bits());assert_eq!(word(meta_reply,12),0);
        let bone=&animation.bones[channel];let info=bone.components.iter().find(|d|d.flags&15==0&&d.tag!=[0;4]).unwrap();
        let mut expected_writes=Vec::new();let mut expected_slots=prepared[channel].prior_slots;
        for(component,d)in prepared[channel].descriptors.iter().enumerate() {
            if d.flags&15!=3 {continue;}
            let mut request=[0;9632];curve_inputs.read_exact(&mut request)?;
            let mut reply=[0;476];curves.read_exact(&mut reply)?;metadata(&reply,468);
            assert_eq!(word(&request,0),word(&clock_reply,16),"fresh curve frame case{cases}");
            assert_eq!(word(&request,4),bone.keyframe_count);assert_eq!(word(&request,8),animation.duration.to_bits());
            assert_eq!(word(&request,12),animation.header_flags_280&8);assert_eq!(word(&request,16),0x027f);
            let flags=[0x83,0x93,0xa3,0xb3].iter().position(|&x|x==d.flags).unwrap();
            assert_eq!(word(&request,20),flags as u32*6+d.slot-3);
            for row in 0..bone.keyframe_count as usize {let dst=32+60*row;
                let info_at=row*info.stride as usize+info.offset as usize;let custom_at=row*d.stride as usize+d.offset as usize;
                assert_eq!(request[dst..dst+20],bone.raw_keyframes[info_at..info_at+20]);
                assert_eq!(request[dst+20..dst+60],bone.raw_keyframes[custom_at..custom_at+40]);}
            assert!(request[32+60*bone.keyframe_count as usize..].iter().all(|&b|b==0));
            let at=200+40*(d.slot-3)as usize;let slot=CellRuntimeScalarSlot {value_bits:word(&reply,at),weight_bits:word(&reply,at+4)};
            expected_writes.push((component,d.slot,slot));expected_slots[(d.slot-3)as usize]=slot;curve_cases+=1;
        }
        let mut contribution_request=[0;1268];contribution_inputs.read_exact(&mut contribution_request)?;
        let mut contribution_reply=[0;184];contributions.read_exact(&mut contribution_reply)?;metadata(&contribution_reply,176);
        assert_eq!(slots_bytes(expected_slots),contribution_request[196..244]);
        let mut consumer_request=[0;200];consumer_inputs.read_exact(&mut consumer_request)?;
        let mut consumer_reply=[0;28];consumers.read_exact(&mut consumer_reply)?;metadata(&consumer_reply,20);
        assert_eq!(consumer_request[36..],contribution_reply[..164]);
        let mut native=Vec::new();native.extend(&meta_reply[..16]);native.extend(&clock_reply[..20]);native.extend(slots_bytes(expected_slots));
        native.extend(&contribution_reply[..164]);native.extend(&consumer_reply[..20]);assert_eq!(native.len(),268);
        let mut computed=None;
        for incoming in [0x1f80,0xdf81,0xbf84] {
            set_csr(incoming);
            let result=evaluate_source_clocked_root_data(&animation,&prepared,&[binding],1u64<<channel,1u64<<channel,0,input.time,SourceMetadataAdmission::ExplicitValidatedV25Resource,[0xa5;164],
                ConsumerContext::ExplicitResolvedRoot {actor_state_18c:0,actor_handle_248:1,actor_word_fc:1},context)?;
            assert_eq!(csr(),incoming);assert_eq!(result.metadata.to_le_bytes(),meta_reply[..16]);assert_eq!(result.clock.to_le_bytes(),clock_reply[..20]);
            for(component,slot,value)in &expected_writes {
                let write=result.root.sample_writes.iter().find(|w|w.channel==channel&&w.component==*component).unwrap();
                assert_eq!((write.slot,write.value_bits,write.weight_bits),(*slot,value.value_bits,value.weight_bits));}
            assert_eq!(result.root.aggregate.raw_bytes(),&contribution_reply[..164]);
            let ConsumerOutcome::Evaluated(value)=result.root.consumer else{return Err("explicit route unresolved".into());};
            let mut consumer=[0;20];consumer[..10].copy_from_slice(&value.retained.to_le_bytes());
            consumer[12..16].copy_from_slice(&context.x87.store_f32(value.retained)?.to_bits().to_le_bytes());
            consumer[16..].copy_from_slice(&u32::from(value.manager_lookup_called).to_le_bytes());
            let mut actual=Vec::new();actual.extend(result.metadata.to_le_bytes());actual.extend(result.clock.to_le_bytes());actual.extend(slots_bytes(result.root.sampled_slots[channel]));
            actual.extend(result.root.aggregate.raw_bytes());actual.extend(consumer);assert_eq!(actual,native,"composed case{cases}");
            if let Some(old)=&computed {assert_eq!(&actual,old);}computed=Some(actual);
        }
        output.write_all(&computed.unwrap())?;projection.write_all(&native)?;
        let unresolved=evaluate_source_clocked_root_data(&animation,&prepared,&[binding],1u64<<channel,1u64<<channel,0,input.time,SourceMetadataAdmission::ExplicitValidatedV25Resource,[0xa5;164],
            ConsumerContext::UnresolvedActorAssociation,context)?;
        assert_eq!(unresolved.root.consumer,ConsumerOutcome::UnresolvedActorAssociation);
        assert_eq!(evaluate_source_clocked_root_data(&animation,&prepared,&[binding],1u64<<channel,1u64<<channel,0,input.time,SourceMetadataAdmission::Unestablished,[0xa5;164],ConsumerContext::UnresolvedActorAssociation,context),Err(SourceClockRootDataError::UnestablishedResourceMapping));
        let duration=f64::from_bits(input.duration_bits)as f32;
        assert_eq!(evaluate_source_clocked_root_data(&animation,&prepared,&[binding],1u64<<channel,1u64<<channel,0,ClockTime::Absolute{time_bits:duration.to_bits()},SourceMetadataAdmission::ExplicitValidatedV25Resource,[0xa5;164],ConsumerContext::UnresolvedActorAssociation,context),Err(SourceClockRootDataError::Composition(ClockRootDataError::Clock(ClockError::DurationCrossing))));
        cases+=1;
    }
    for file in [&mut clock_inputs,&mut curve_inputs,&mut contribution_inputs,&mut consumer_inputs,&mut clocks,&mut curves,&mut contributions,&mut consumers] {
        assert_eq!(file.read(&mut[0])?,0,"trailing request/native bytes");}
    assert_eq!(cases,32);assert_eq!(curve_cases,72);
    let sources=fs::read_to_string(root.join("metadata-sources.tsv"))?;let mut loop_rejections=0;
    for line in sources.lines().skip(1){let f:Vec<_>=line.split('\t').collect();let i=f[0].parse::<usize>()?;let animation=anim::parse(&fs::read(root.join(f[1]))?)?;let reply=&meta_native[24*i..24*(i+1)];metadata(reply,16);
      if word(reply,12)==0{continue;}let d=f32::from_bits(word(reply,0));
      for mode in 0..2{for quarter in 0..4{let time=d*(quarter as f32)/4.;let time=if mode==0{ClockTime::Absolute{time_bits:time.to_bits()}}else{ClockTime::Advance{previous_time_bits:f64::from(time/2.).to_bits(),delta_bits:(time/2.).to_bits()}};
       assert_eq!(evaluate_source_clocked_root_data(&animation,&[],&[],0,0,0,time,SourceMetadataAdmission::ExplicitValidatedV25Resource,[0xa5;164],ConsumerContext::UnresolvedActorAssociation,context),Err(SourceClockRootDataError::Composition(ClockRootDataError::Clock(ClockError::UnsupportedLoop))));loop_rejections+=1;
      }}
    }assert_eq!(loop_rejections,8);
    assert_eq!(source_frame(FrameInput::NonLoopFirstDuration {time_bits:0x3f000000,duration_bits:1f64.to_bits(),resource_frames_bits:0x43050000},context),Err(RootDataError::UnsupportedClockDomain));
    output.flush()?;projection.flush()?;
    println!("CELL_RUNTIME_SOURCE_CLOCK_ROOT_DATA_VERIFY cases={cases} fresh_curve_requests={curve_cases} bytes_per_case=268 caller_states=3 positive_rejections={} unresolved_route_checked=true old_clock_variant_still_rejected=true",cases*2+8);Ok(())
}
