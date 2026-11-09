//! Compare the explicit retained Cell-definition cohort with Python's raw-name reference.
use spore_core::cell_definition::{self, ConditionalName, NameLocale};
use spore_core::content_manifest::LocaleOutputState;
use spore_core::locale_table;
use std::fmt::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 { return Err("usage: cell_definition_name_compare_v1 <corpus> <locale> <expected>".into()); }
    let bytes=std::fs::read(&args[1])?;
    if bytes.len()<12 || &bytes[..8]!=b"RSCELLD1" { return Err("invalid corpus".into()); }
    let count=u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
    let table=locale_table::parse(&std::fs::read(&args[2])?)?;
    let expected=std::fs::read_to_string(&args[3])?;
    let state=LocaleOutputState { cached_flag:false, cached_pointer_present:false,
        classifier_nonzero:false, global_nonzero:false };
    let mut actual=String::new();let mut pos=12usize;let mut totals=[0usize;3];
    for i in 0..count {
        let header=bytes.get(pos..pos+20).ok_or("truncated header")?;
        let size=u32::from_le_bytes(header[8..12].try_into()?) as usize;pos+=20;
        let def=cell_definition::parse(bytes.get(pos..pos+size).ok_or("truncated record")?)?;pos+=size;
        assert_eq!(def.conditional_name(NameLocale::Unresolved,None),ConditionalName::LocaleProviderUnresolved);
        let decision=def.conditional_name(NameLocale::Selected(&table),Some(state));
        let (category,units)=match decision {
            ConditionalName::LocaleRaw(units)=>{totals[0]+=1;("locale",units.to_vec())},
            ConditionalName::Embedded(units)=>{totals[1]+=1;("embedded",units)},
            ConditionalName::CreationProviderUnresolved=>{totals[2]+=1;("creation",vec![])},
            x=>return Err(format!("unexpected outcome at {i}: {x:?}").into()),
        };
        if category=="locale" {
            assert!(matches!(def.conditional_name(NameLocale::Selected(&table),None), ConditionalName::LocaleProcessingUnresolved{..}));
        }
        write!(&mut actual,"{i}\t{:08x}\t{category}\t",def.label_trailing_word())?;
        for unit in units {write!(&mut actual,"{unit:04x}")?;}
        actual.push('\n');
    }
    if pos!=bytes.len() || actual!=expected { return Err("corpus or per-record comparison mismatch".into()); }
    println!("{count}/{count} copies matched: locale={}, embedded={}, creation-unresolved={}; explicit table and raw-return state only",totals[0],totals[1],totals[2]);
    Ok(())
}
