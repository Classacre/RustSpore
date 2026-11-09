//! Compare fixed synthetic native cases; no live service or stage claim.
use spore_core::space_category_accessor_loop_v1::project;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path=std::env::args_os().nth(1).ok_or("pass retained stdout path")?;
    let data=std::fs::read(path)?;
    if data.len()!=220 {return Err("expected five 44-byte rows".into());}
    let keys=[0xd49d_e19d,0x484f_f0b8,0x39e6_4a16,0x5bdc_ec35,0xb09a_bbda];
    let rows=keys.map(|x|[x,0,0]);
    for i in 0..5 {
        let mut actual=[0u32;11];
        for (j,w) in actual.iter_mut().enumerate(){let at=i*44+j*4;*w=u32::from_le_bytes(data[at..at+4].try_into()?);}
        let (success,typ,flags,count)=match i {0=>(true,0x20,0x10,5),1=>(true,0x20,0x10,0),2=>(true,0x21,0x10,5),3=>(true,0x20,0,5),_=>(false,0x20,0x10,5)};
        let result=project(success,typ,flags,count,&rows)?;
        let mut words=[0u32;5];words[..result.visited_first_words.len()].copy_from_slice(&result.visited_first_words);
        if actual[0]!=1 || actual[1]!=result.visited_first_words.len() as u32 || actual[2..7]!=words
            || actual[7]!=result.count as u32 || actual[8]!=u32::from(result.admitted_array)
            || actual[9]&0xffff!=0x027f || (actual[9]>>16)&0x3840!=0 || actual[10]&!0x3f!=0x1f80 {
            return Err(format!("row {i} differs").into());
        }
    }
    println!("matched five controlled accessor/loop rows; no live service claim");Ok(())
}
