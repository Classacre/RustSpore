//! Independently generated synthetic runtime topology fixtures, not source rigs.
//! tools\build.bat run -p spore-bevy --example anim_order_verify -- FIXTURES DIGEST
use spore_core::anim_order::{order_runtime_blocks, RuntimeBlock, RuntimeBlockOrder};
use std::io::{BufRead, Write};

struct Digest(u64);
impl Digest {
    fn bytes(&mut self, values: &[u8]) {
        for byte in values {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    }
    fn word(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
    fn index(&mut self, value: usize) -> Result<(), String> {
        self.word(u32::try_from(value).map_err(|_| "fixture index exceeds u32")?);
        Ok(())
    }
    fn optional(&mut self, value: Option<usize>) -> Result<(), String> {
        if let Some(value) = value {
            self.index(value)?;
        } else {
            self.word(u32::MAX);
        }
        Ok(())
    }
}

fn digest(input: &[RuntimeBlock], root: usize, ordered: &RuntimeBlockOrder) -> Result<u64, String> {
    let mut d = Digest(0xcbf29ce484222325);
    d.bytes(b"runtime-block-order-v1\0");
    d.index(input.len())?;
    d.index(root)?;
    for value in input {
        d.optional(value.parent)?;
        d.word(value.capability_mask);
    }
    for &old in &ordered.new_to_old {
        d.index(old)?;
    }
    for &new in &ordered.old_to_new {
        d.index(new)?;
    }
    for value in &ordered.blocks {
        d.index(value.original_index)?;
        d.optional(value.parent)?;
        d.index(value.children.start)?;
        d.index(value.children.len())?;
        d.index(value.descendants.start)?;
        d.index(value.descendants.len())?;
    }
    Ok(d.0)
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: anim_order_verify <synthetic-fixtures.tsv> <digest-out.txt>".into());
    }
    let input = std::io::BufReader::new(std::fs::File::open(&args[0]).map_err(|e| e.to_string())?);
    let mut output =
        std::io::BufWriter::new(std::fs::File::create(&args[1]).map_err(|e| e.to_string())?);
    let mut cases = 0usize;
    let mut blocks = 0usize;
    let mut max_blocks = 0usize;
    for (line_index, line) in input.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err(format!(
                "line {}: need id, root, parents, masks",
                line_index + 1
            ));
        }
        let root = fields[1].parse::<usize>().map_err(|e| e.to_string())?;
        let parents: Vec<_> = fields[2]
            .split(',')
            .map(|value| {
                if value == "-" {
                    Ok(None)
                } else {
                    value.parse::<usize>().map(Some).map_err(|e| e.to_string())
                }
            })
            .collect::<Result<_, _>>()?;
        let masks: Vec<_> = fields[3]
            .split(',')
            .map(|value| u32::from_str_radix(value, 16).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        if parents.len() != masks.len() {
            return Err(format!("{}: parent and mask counts differ", fields[0]));
        }
        let input: Vec<_> = parents
            .into_iter()
            .zip(masks)
            .map(|(parent, capability_mask)| RuntimeBlock {
                parent,
                capability_mask,
            })
            .collect();
        let ordered =
            order_runtime_blocks(&input, root).map_err(|e| format!("{}: {e}", fields[0]))?;
        let value = digest(&input, root, &ordered)?;
        writeln!(output, "{}\t{value:016X}", fields[0]).map_err(|e| e.to_string())?;
        cases += 1;
        blocks += input.len();
        max_blocks = max_blocks.max(input.len());
    }
    output.flush().map_err(|e| e.to_string())?;
    if cases == 0 {
        return Err("no synthetic topology fixtures read".into());
    }
    println!(
        "verified_synthetic_cases={cases} supplied_runtime_blocks={blocks} max_blocks={max_blocks}"
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
