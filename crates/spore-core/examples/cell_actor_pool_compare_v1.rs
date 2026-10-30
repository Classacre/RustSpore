//! Adapter for synthetic checked-pool model comparisons, not gameplay.
use spore_core::cell_actor_pool_checked_v1::{CheckedActorPool, PoolError};
use std::{error::Error, fs::OpenOptions, io::{BufReader, BufWriter, Read, Write}};

fn word(input: &mut impl Read) -> Result<u32, Box<dyn Error>> {
    let mut bytes=[0;4]; input.read_exact(&mut bytes)?; Ok(u32::from_le_bytes(bytes))
}
fn write_word(output: &mut impl Write, value:u32) -> Result<(), Box<dyn Error>> {
    output.write_all(&value.to_le_bytes())?; Ok(())
}

fn main() -> Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().collect();
    if args.len()!=3 { return Err("usage: pool-adapter input new-output".into()); }
    let mut input=BufReader::new(std::fs::File::open(&args[1])?);
    let mut magic=[0u8;8]; input.read_exact(&mut magic)?;
    if &magic!=b"CAPIN001" { return Err("bad input magic".into()); }
    let mut output=BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&args[2])?);
    output.write_all(b"CAPOUT01")?;
    let mut cases=0usize; let mut total=0usize;
    loop {
        let cap=word(&mut input)?; let counter=word(&mut input)?; let count=word(&mut input)?;
        if cap==0 && counter==0 && count==0 { break; }
        if !(1..=16).contains(&cap) || count>100 || cases>=1000 { return Err("protocol bounds".into()); }
        let mut pool=CheckedActorPool::<u32>::new(cap as usize,counter).map_err(|e| format!("new: {e:?}"))?;
        for _ in 0..count {
            let op=word(&mut input)?; let arg=word(&mut input)?;
            let mut status=0u32; let mut value=0u32; let mut extra=Vec::new();
            match op {
                1 => match pool.allocate(arg) {
                    Ok(h)=>value=h,
                    Err(PoolError::Full)=>status=1,
                    Err(PoolError::GenerationBoundary)=>status=2,
                    Err(e)=>return Err(format!("unexpected allocation error: {e:?}").into()),
                },
                2 => match pool.release(arg) {
                    Ok(v)=>value=v,
                    Err(PoolError::InvalidHandle)=>status=3,
                    Err(e)=>return Err(format!("unexpected release error: {e:?}").into()),
                },
                3 => match pool.get(arg) { Some(v)=>value=*v, None=>status=4 },
                4 => for (h,v) in pool.iter() { extra.push(h); extra.push(*v); },
                5 => extra=pool.reset(),
                _ => return Err("unknown operation".into()),
            }
            for field in [status,value,pool.counter(),u32::try_from(pool.live_count())?,
                          u32::try_from(pool.high_water())?,u32::try_from(extra.len())?] {
                write_word(&mut output,field)?;
            }
            for field in extra { write_word(&mut output,field)?; }
            total+=1;
        }
        cases+=1;
    }
    let mut trailing=[0]; if input.read(&mut trailing)?!=0 { return Err("trailing input".into()); }
    output.flush()?;
    println!("cases={cases} operations={total} model_only=true source_execution=false");
    Ok(())
}
