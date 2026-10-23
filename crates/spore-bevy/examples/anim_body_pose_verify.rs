//! Compare bounded native body-pose arithmetic with the source instruction oracle.
//! tools\build.bat run -p spore-bevy --example anim_body_pose_verify -- FIXTURES DIGEST
use spore_core::anim_body_pose::{
    evaluate_body_palette, prepare_rest_palette, RestBlock, RuntimeBlockPose,
};
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
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: anim_body_pose_verify <synthetic-fixtures.tsv> <digest-out.txt>".into(),
        );
    }
    let input = std::io::BufReader::new(std::fs::File::open(&args[0]).map_err(|e| e.to_string())?);
    let mut output =
        std::io::BufWriter::new(std::fs::File::create(&args[1]).map_err(|e| e.to_string())?);
    let mut cases = 0usize;
    let mut matrices = 0usize;
    for (line_index, line) in input.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        let words: Vec<_> = fields[1..]
            .iter()
            .map(|value| u32::from_str_radix(value, 16).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        if words.len() < 4 {
            return Err(format!("line {}: missing fixture header", line_index + 1));
        }
        let count = words[1] as usize;
        let rest_count = words[2] as usize;
        let live_count = words[3] as usize;
        if rest_count > 1024
            || live_count > 1024
            || words.len() != 4 + (rest_count + live_count) * 8
        {
            return Err(format!(
                "line {}: invalid bounded fixture extent",
                line_index + 1
            ));
        }
        let rest_blocks: Vec<_> = words[4..4 + rest_count * 8]
            .chunks_exact(8)
            .map(|row| RestBlock {
                palette_id: row[0],
                center: std::array::from_fn(|i| f32::from_bits(row[i + 1])),
                orientation_xyzw: std::array::from_fn(|i| f32::from_bits(row[i + 4])),
            })
            .collect();
        let runtime_blocks: Vec<_> = words[4 + rest_count * 8..]
            .chunks_exact(8)
            .map(|row| RuntimeBlockPose {
                palette_id: row[0],
                position: std::array::from_fn(|i| f32::from_bits(row[i + 1])),
                orientation_xyzw: std::array::from_fn(|i| f32::from_bits(row[i + 4])),
            })
            .collect();
        let rest =
            prepare_rest_palette(count, &rest_blocks).map_err(|e| format!("{}: {e}", fields[0]))?;
        let result = evaluate_body_palette(&rest, &runtime_blocks, f32::from_bits(words[0]))
            .map_err(|e| format!("{}: {e}", fields[0]))?;
        let mut digest = Digest(0xCBF29CE484222325);
        digest.bytes(b"native-body-pose-v1\0");
        for value in &words {
            digest.word(*value);
        }
        for value in rest.matrices() {
            for component in value.basis.iter().chain(&value.translation) {
                digest.word(component.to_bits());
            }
        }
        for value in &result {
            for component in value {
                digest.word(component.to_bits());
            }
        }
        writeln!(output, "{}\t{:016X}", fields[0], digest.0).map_err(|e| e.to_string())?;
        cases += 1;
        matrices += result.len();
    }
    output.flush().map_err(|e| e.to_string())?;
    if cases == 0 {
        return Err("no synthetic body-pose fixtures read".into());
    }
    println!("verified_synthetic_cases={cases} generated_matrices={matrices}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
