//! Explicit four-manifest typed-prefix comparison; no live-provider inference.
use spore_core::content_manifest::{localized_name_prefix, AdmittedLocale, LocalizedNameLookup};
use spore_core::locale_table;
use std::fmt::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() < 3 { return Err("usage: manifest_four_compare_v1 <locale> <manifest>...".into()); }
    let table=locale_table::parse(&std::fs::read(&args[1])?)?;
    for path in &args[2..] {
        let path=std::path::Path::new(path);
        let id=path.file_stem().ok_or("missing file stem")?.to_str().ok_or("non-UTF8 file stem")?;
        let data=std::fs::read(path)?;
        let prefix=localized_name_prefix(&data,true).map_err(|e|format!("{id}: {e:?}"))?.ok_or("framing unresolved")?;
        if prefix.table_id!=0x04dc2f94 { return Err("unexpected table".into()); }
        let result=prefix.lookup(AdmittedLocale::Selected {table_id:prefix.table_id,table:&table});
        let LocalizedNameLookup::RawCandidates {first,name,optional}=result else {return Err(format!("{id}: {result:?}").into());};
        if optional.is_some() { return Err("unexpected optional field".into()); }
        let mut row=format!("{id}\t{:08X}\t{:08X}\t",prefix.first_id,prefix.name_id);
        for unit in first {write!(&mut row,"{unit:04x}")?;} row.push('\t');
        for unit in name {write!(&mut row,"{unit:04x}")?;} println!("{row}");
    }
    Ok(())
}
