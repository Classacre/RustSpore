//! Verify native palette gathering with synthetic explicit runtime inputs.
//! tools\build.bat run -p spore-bevy --example anim_palette_verify -- FIXTURES DIGEST
use spore_core::anim_palette::{
    gather_runtime_palette, BoneMatrix, GeneratedTable, GroupMatrices, RuntimePaletteGroup,
};
use std::io::{BufRead, Write};

fn matrix(kind: u32, bone: usize) -> BoneMatrix {
    std::array::from_fn(|component| {
        let bits = if bone == 0 && component == 0 {
            0x8000_0000
        } else if bone == 0 && component == 1 {
            0x7FA1_2345 + kind
        } else {
            0x3F00_0000 + kind * 0x0080_0000 + bone as u32 * 0x1000 + component as u32
        };
        f32::from_bits(bits)
    })
}

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
        return Err("usage: anim_palette_verify <synthetic-fixtures.tsv> <digest-out.txt>".into());
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
        if fields.len() != 8 {
            return Err(format!(
                "line {}: expected id and seven integers",
                line_index + 1
            ));
        }
        let values: Vec<u32> = fields[1..]
            .iter()
            .map(|s| s.parse::<u32>().map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let first = u8::try_from(values[0]).map_err(|_| "first count exceeds a byte")?;
        let second = u8::try_from(values[1]).map_err(|_| "second count exceeds a byte")?;
        let requested = values[2] as usize;
        if requested > 4096 || values[3..].iter().any(|v| *v > 1) {
            return Err(format!(
                "{}: unsupported synthetic fixture values",
                fields[0]
            ));
        }
        let supplied = values[3] != 0;
        let first_evaluated = values[4] != 0;
        let second_evaluated = values[5] != 0;
        let ready = values[6] != 0;
        let generated: Vec<_> = (0..requested).map(|i| matrix(0, i)).collect();
        let first_matrices: Vec<_> = (0..usize::from(first)).map(|i| matrix(1, i)).collect();
        let second_matrices: Vec<_> = (0..usize::from(second)).map(|i| matrix(2, i)).collect();
        let groups = [
            RuntimePaletteGroup {
                first_bone: 0,
                bone_count: first,
                matrices: if first_evaluated {
                    GroupMatrices::Evaluated(&first_matrices)
                } else {
                    GroupMatrices::Identity
                },
            },
            RuntimePaletteGroup {
                first_bone: u16::from(first),
                bone_count: second,
                matrices: if second_evaluated {
                    GroupMatrices::Evaluated(&second_matrices)
                } else {
                    GroupMatrices::Identity
                },
            },
        ];
        let result = gather_runtime_palette(
            ready,
            &groups,
            GeneratedTable {
                requested_count: requested,
                matrices: supplied.then_some(generated.as_slice()),
            },
        )
        .map_err(|e| format!("{}: {e}", fields[0]))?;
        let mut digest = Digest(0xCBF29CE484222325);
        digest.bytes(b"runtime-palette-v1\0");
        for value in &values {
            digest.word(*value);
        }
        digest.word(result.bound_generated_count as u32);
        digest.word(result.copied_generated_count as u32);
        digest.word(result.matrices.len() as u32);
        for value in &result.matrices {
            for component in value {
                digest.word(component.to_bits());
            }
        }
        writeln!(output, "{}\t{:016X}", fields[0], digest.0).map_err(|e| e.to_string())?;
        cases += 1;
        matrices += result.matrices.len();
    }
    output.flush().map_err(|e| e.to_string())?;
    if cases == 0 {
        return Err("no synthetic palette fixtures read".into());
    }
    println!("verified_synthetic_cases={cases} assembled_matrices={matrices}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
