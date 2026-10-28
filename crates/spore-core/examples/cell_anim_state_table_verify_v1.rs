//! Print the bounded table projection for comparison with the independent audit.
//! Input is supplied locally from ignored target; no source bytes are embedded.
use spore_core::cell_anim_state_table_v1 as table;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected table byte file")?;
    if args.next().is_some() { return Err("expected exactly one argument".into()); }
    let bytes = std::fs::read(path)?;
    let records = table::parse_table(&bytes).ok_or("incorrect table length")?;
    for (index, record) in records.iter().enumerate() {
        print!("{}", index);
        for word in record.words { print!(" {:08X}", word); }
        println!(" {:08X} {:08X}", record.state_word(), record.request_key());
    }
    Ok(())
}
