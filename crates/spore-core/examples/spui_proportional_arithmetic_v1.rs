//! TSV arithmetic driver; no source installation or native execution.
use spore_core::spui_proportional_layout::ProportionalLayout;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let words: Vec<_> = line.split('\t').collect();
        if words.len() != 13 { return Err("expected operation and twelve float32 hex words".into()); }
        let bits: Vec<u32> = words[1..].iter().map(|w| u32::from_str_radix(w, 16)).collect::<Result<_, _>>()?;
        let values: Vec<f32> = bits.into_iter().map(f32::from_bits).collect();
        let layout = ProportionalLayout { edges: values[0..4].try_into()? };
        let parent = values[4..8].try_into()?;
        let child = values[8..12].try_into()?;
        let result = match words[0] {
            "add" => layout.apply(parent, child),
            "sub" => layout.unapply(parent, child),
            _ => return Err("unknown operation".into()),
        };
        match result {
            Ok(v) => println!("{:08X}\t{:08X}\t{:08X}\t{:08X}", v[0].to_bits(), v[1].to_bits(), v[2].to_bits(), v[3].to_bits()),
            Err(_) => println!("error"),
        }
    }
    Ok(())
}
