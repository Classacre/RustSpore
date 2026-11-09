//! Compare one pinned-source-snapshot variant join; no live service claim.
use spore_core::space_category_accessor_loop_v1::project;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args=std::env::args_os().skip(1);
    let data=std::fs::read(args.next().ok_or("pass variant snapshot path")?)?;
    let native=std::fs::read(args.next().ok_or("pass retained native stdout path")?)?;
    if args.next().is_some() || data.len()!=72 || native.len()!=44 {return Err("expected 72-byte variant and one 44-byte row".into());}
    if data[..12] != [0,0x20,0,0x9c,0,0,0,5,0,0,0,12] {return Err("unexpected variant framing".into());}
    let words=[0xd49d_e19d,0x484f_f0b8,0x39e6_4a16,0x5bdc_ec35,0xb09a_bbda];
    let mut records=[[0u32;3];5];
    for (i,record) in records.iter_mut().enumerate() {
        for (j,word) in record.iter_mut().enumerate(){let at=12+i*12+j*4;*word=u32::from_le_bytes(data[at..at+4].try_into()?);}
        if *record != [words[i],0,0] {return Err(format!("snapshot record {i} differs").into());}
    }
    let result=project(true,0x20,0x9c,5,&records)?;
    let mut row=[0u32;11];for (i,word) in row.iter_mut().enumerate(){*word=u32::from_le_bytes(native[i*4..i*4+4].try_into()?);}
    if row[..2]!=[1,5] || row[2..7]!=result.visited_first_words || row[7..9]!=[5,1]
        || row[9]&0xffff!=0x027f || (row[9]>>16)&0x3840!=0 || row[10]&!0x3f!=0x1f80 {
        return Err("retained row differs".into());
    }
    println!("matched one controlled source-snapshot array join; no live service claim");Ok(())
}
