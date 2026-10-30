//! Adapter for the synthetic/explicit-source equal-size rational corpus.
//! Runs the registered Rust model, not the original executable or game.
//! Input: label plus seven uppercase eight-digit hexadecimal fields per row.
//! Output: status, sample bits, scale bits, RNG state; unsupported rows use `-`.
use spore_core::{
    cell_definition, cell_globals,
    cell_start_equal_size_v1::{candidate_start_size, AdmittedCellKey, StartSizeError, CELL_TYPE, CANDIDATE_ID},
    native_x87::X87Error,
};
use std::{error::Error, fs::OpenOptions, io::Write, path::Path};

const MAX_INPUT: u64 = 1024 * 1024;
const MAX_ROWS: usize = 4096;

fn hex_word(s: &str) -> Result<u32, Box<dyn Error>> {
    if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b)) {
        return Err("expected exactly eight uppercase hexadecimal digits".into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}

fn run(input: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::metadata(input)?.len() > MAX_INPUT { return Err("input exceeds byte limit".into()); }
    let bytes = std::fs::read(input)?;
    if bytes.len() as u64 > MAX_INPUT { return Err("input exceeds byte limit".into()); }
    let text = std::str::from_utf8(&bytes)?;
    if text.is_empty() || !text.ends_with('\n') || text.contains('\r') { return Err("invalid TSV framing".into()); }
    let mut reply = String::new();
    let mut count = 0usize;
    for line in text.strip_suffix('\n').unwrap().split('\n') {
        if count >= MAX_ROWS { return Err("input exceeds row limit".into()); }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 8 || !matches!(fields[0], "candidate" | "synthetic" | "rejection") {
            return Err(format!("invalid record {}", count + 1).into());
        }
        let values: Vec<_> = fields[1..].iter().map(|s| hex_word(s)).collect::<Result<_, _>>()?;
        let [id, low, high, divisor, arg, seed, control]: [u32; 7] = values.try_into().map_err(|_| "invalid field count")?;
        let control = u16::try_from(control).map_err(|_| "control word exceeds u16")?;
        // These are explicit fixture records, not loaded native definitions.
        let mut globals = [0u8; cell_globals::GLOBALS_SIZE];
        globals[0x34..0x38].copy_from_slice(&CANDIDATE_ID.to_le_bytes());
        let mut definition = [0u8; cell_definition::DEFINITION_SIZE];
        definition[..4].copy_from_slice(&CANDIDATE_ID.to_le_bytes());
        definition[0x304..0x308].copy_from_slice(&low.to_le_bytes());
        definition[0x308..0x30c].copy_from_slice(&high.to_le_bytes());
        let key = AdmittedCellKey { instance: id, type_id: CELL_TYPE, scope: 0 };
        let result = candidate_start_size(key, cell_definition::parse(&definition)?, cell_globals::parse(&globals)?,
            f32::from_bits(divisor), f32::from_bits(arg), seed, control);
        match result {
            Ok(v) => reply.push_str(&format!("OK\t{:08X}\t{:08X}\t{:08X}\n", v.sampled_bits, v.actor_scale_bits, v.rng_state_after)),
            Err(e) => {
                let status = match e {
                    StartSizeError::UnsupportedCandidate => "UnsupportedCandidate",
                    StartSizeError::UnsupportedRange => "UnsupportedRange",
                    StartSizeError::UnsupportedInput => "UnsupportedInput",
                    StartSizeError::UnsupportedControl => "UnsupportedControl",
                    StartSizeError::UnsupportedRng => "UnsupportedRng",
                    StartSizeError::Arithmetic(X87Error::NonFiniteResult) => "Arithmetic.NonFiniteResult",
                    other => return Err(format!("unsupported model error on row {}: {other:?}", count + 1).into()),
                };
                reply.push_str(status);
                reply.push_str("\t-\t-\t-\n");
            }
        }
        count += 1;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(output)?;
    file.write_all(reply.as_bytes())?;
    println!("rows={count} model_only=true source_execution=false");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: cell_start_equal_size_compare_v1 input.tsv new-output.tsv".into()); }
    run(Path::new(&args[1]), Path::new(&args[2]))
}
