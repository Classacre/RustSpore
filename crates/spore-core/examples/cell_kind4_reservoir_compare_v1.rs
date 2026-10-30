//! Bounded adapter for synthetic kind-4 reservoir model comparisons.
//! This does not execute original instructions or establish runtime cache state.
use spore_core::{cell_kind4_cache_selection_v1::{self as model, Cache, ResultV1, Slot, SnapshotAdmission},
                 cell_kind4_list_disk_v1};
use std::{error::Error, fs::OpenOptions, io::{Read, Write}};

fn word(data: &[u8], at: &mut usize) -> Result<u32, Box<dyn Error>> {
    let end=at.checked_add(4).ok_or("offset overflow")?;
    let value=u32::from_le_bytes(data.get(*at..end).ok_or("truncated input")?.try_into()?);
    *at=end; Ok(value)
}
fn emit(output: &mut Vec<u8>, value:u32) { output.extend_from_slice(&value.to_le_bytes()); }

fn main() -> Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().collect();
    if args.len()!=3 { return Err("usage: kind4-adapter input new-output".into()); }
    let mut file=std::fs::File::open(&args[1])?;
    if file.metadata()?.len()>1024*1024 { return Err("input too large".into()); }
    let mut data=Vec::new(); file.take(1024*1024+1).read_to_end(&mut data)?;
    if data.len()>1024*1024 || data.get(..8)!=Some(&b"K4IN0001"[..]) { return Err("bad input framing".into()); }
    let mut at=8; let cases=word(&data,&mut at)?;
    if cases==0 || cases>1000 { return Err("case limit".into()); }
    let mut output=b"K4OUT001".to_vec(); emit(&mut output,cases);
    for _ in 0..cases {
        let mut fields=[0u32;22]; for field in &mut fields { *field=word(&data,&mut at)?; }
        let mut rng=fields[0]; let control=u16::try_from(fields[1]).map_err(|_| "control out of range")?;
        let present=fields[2]; let count=fields[7];
        if present>1 || !(1..=2).contains(&count) { return Err("bad case bounds".into()); }
        let mut cache=Cache::default();
        if present==1 {
            cache.admit_ready(fields[3],[fields[4],fields[5],fields[6]],SnapshotAdmission)
                .map_err(|e| format!("preloaded cache: {e:?}"))?;
        } else if fields[3..7].iter().any(|&v| v!=0) { return Err("nonzero empty-cache padding".into()); }
        if count==1 && fields[15..22].iter().any(|&v| v!=0) { return Err("nonzero unused-request padding".into()); }
        emit(&mut output,count);
        for i in 0..count as usize {
            let p=8+i*7; let key=fields[p]; let _list_handle=fields[p+1]; let third=fields[p+2];
            // On hits no row, RNG or control admission is performed: the
            // source cache path does not enter the selection helper.
            let hit=matches!(cache.slot(key),Some(Slot::Ready(_)));
            let excluded=!hit && (rng==0 || rng==0x8000_0000 || control&0x3f!=0x3f ||
                !matches!(control&0x300,0x200|0x300));
            if excluded {
                for v in [2,0,0,0,rng,u32::MAX,0] { emit(&mut output,v); }
                continue;
            }
            let result=if hit {
                model::select(&mut cache,key,None,rng,control,third,SnapshotAdmission)
            } else {
                let mut bytes=vec![0u8;8+4*0x1c];
                bytes[..4].copy_from_slice(&4u32.to_le_bytes()); bytes[4..8].copy_from_slice(&8u32.to_le_bytes());
                for j in 0..4 {
                    let off=8+j*0x1c;
                    bytes[off+4..off+8].copy_from_slice(&fields[p+3+j].to_le_bytes());
                    bytes[off+8..off+12].copy_from_slice(&1f32.to_bits().to_le_bytes());
                }
                let view=cell_kind4_list_disk_v1::parse(&bytes).map_err(|e| format!("rows: {e:?}"))?;
                model::select(&mut cache,key,Some(&view),rng,control,third,SnapshotAdmission)
            };
            let values=match result {
                ResultV1::Hit { triple,rng_state } => { rng=rng_state; [0,triple[0],triple[1],triple[2],rng,u32::MAX,0] },
                ResultV1::Selected { triple,selected,rng_state,draws } => {
                    rng=rng_state; [1,triple[0],triple[1],triple[2],rng,u32::try_from(selected)?,u32::from(draws)]
                },
                other => return Err(format!("unexpected model result: {other:?}").into()),
            };
            for v in values { emit(&mut output,v); }
        }
    }
    if at!=data.len() { return Err("trailing input".into()); }
    let mut out=OpenOptions::new().write(true).create_new(true).open(&args[2])?;
    out.write_all(&output)?;
    println!("cases={cases} model_only=true source_execution=false");
    Ok(())
}
