//! Compare a supplied manifest prefix with external expected locale text.
use spore_core::content_manifest::{localized_name_prefix, AdmittedLocale, LocalizedNameLookup};
use spore_core::locale_table;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 { return Err("usage: manifest_locale_sample_v1 <manifest> <locale> <expected-first> <expected-name>".into()); }
    let manifest = std::fs::read(&args[1])?;
    let locale = locale_table::parse(&std::fs::read(&args[2])?)?;
    let prefix = localized_name_prefix(&manifest, true)
        .map_err(|e| format!("prefix: {e:?}"))?.ok_or("framing unresolved")?;
    let result = prefix.lookup(AdmittedLocale::Selected { table_id: prefix.table_id, table: &locale });
    let LocalizedNameLookup::RawCandidates { first, name, optional } = result else {
        return Err(format!("lookup: {result:?}").into());
    };
    let expected_first: Vec<u16> = args[3].encode_utf16().collect();
    let expected_name: Vec<u16> = args[4].encode_utf16().collect();
    if first != expected_first || name != expected_name || optional.is_some() {
        return Err("supplied expectations did not match".into());
    }
    println!("Supplied raw locale units match; provider and processing not verified");
    Ok(())
}
