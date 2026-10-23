//! Canonical comparison of bounded FPGM setup with original instructions.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    spui_font,
    spui_font_hint_program::{self, Bootstrap, BootstrapLimits},
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};

const FACES: [u32; 5] = [0x511b49ca, 0x61639dfb, 0x6bbc4e0e, 0xbacaaaa5, 0xffe355df];
const SAMPLES: [(&str, &[u8]); 9] = [
    ("empty", &[]),
    ("npushb0", &[0x40, 0]),
    ("npushw0", &[0x41, 0]),
    ("npushb3", &[0x40, 3, 0, 128, 255]),
    ("pushb2", &[0xb1, 45, 255]),
    ("signedwords", &[0x41, 3, 0xff, 0xff, 0x80, 0, 0x7f, 0xff]),
    ("fixedwords", &[0xb9, 0xff, 0xff, 0x80, 0]),
    (
        "opaque_endf_data",
        &[0xb0, 2, 0x2c, 0x41, 1, 0x2d, 0x2d, 0xb1, 0x2d, 0x2d, 0x2d],
    ),
    (
        "overwrite",
        &[0xb0, 1, 0x2c, 0x21, 0x2d, 0xb0, 1, 0x2c, 0, 0x2d],
    ),
];

fn emit_setup(
    output: &mut impl Write,
    key: &str,
    program: &[u8],
    limits: BootstrapLimits,
    setup: &Bootstrap<'_>,
) -> std::io::Result<()> {
    let remaining = setup
        .remaining_stack
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    writeln!(
        output,
        "{key}\tlimits\t{}\t{}\t{}\t{}\t{remaining}",
        limits.max_functions,
        limits.max_stack_elements,
        program.len(),
        setup.peak_stack_elements
    )?;
    for (ordinal, step) in setup.dispatches.iter().enumerate() {
        let defined = step
            .defined_index
            .map(|index| index.to_string())
            .unwrap_or_else(|| "-".into());
        writeln!(
            output,
            "{key}\tstep\t{ordinal}\t{}\t{:02X}\t{}\t{}\t{defined}",
            step.pc, step.opcode, step.next_pc, step.stack_after
        )?;
    }
    for (index, binding) in setup.functions.iter().enumerate() {
        if let Some(binding) = binding {
            let body = binding
                .body
                .iter()
                .map(|value| format!("{value:02x}"))
                .collect::<String>();
            writeln!(
                output,
                "{key}\tfunction\t{index}\t{}\t{}\t{}\t{body}",
                binding.offset, binding.length, binding.program_selector
            )?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: spui_font_hint_program_verify <Spore_Graphics.package> <new FPGM TSV>".into(),
        );
    }
    let output_path = std::path::absolute(&args[1])?;
    output_path.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let mut faces: Vec<_> = package
        .entries()
        .iter()
        .filter(|entry| {
            entry.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && entry.key.group_id == spui_font::FONT_NAMESPACE
                && FACES.contains(&entry.key.instance_id)
        })
        .collect();
    faces.sort_by_key(|entry| (entry.key.type_id, entry.key.group_id, entry.key.instance_id));
    if faces.len() != 5 {
        return Err("expected exactly five canonical source faces".into());
    }
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output_path)?,
    );
    for opcode in 0u8..=u8::MAX {
        let counts: Vec<i16> = if opcode == 0x40 || opcode == 0x41 {
            (0..=255).collect()
        } else {
            vec![-1]
        };
        for count in counts {
            let mut program = vec![opcode];
            match opcode {
                0x40 | 0x41 => {
                    program.push(count as u8);
                    program.resize(
                        2 + count as usize * if opcode == 0x40 { 1 } else { 2 },
                        0x2d,
                    );
                }
                0xb0..=0xb7 => program.resize(1 + usize::from(opcode - 0xaf), 0x2d),
                0xb8..=0xbf => program.resize(1 + usize::from(opcode - 0xb7) * 2, 0x2d),
                _ => {}
            }
            writeln!(
                output,
                "scan\t{opcode:02X}\t{count}\t{}",
                spui_font_hint_program::instruction_end(&program, 0)?
            )?;
        }
    }
    let word_limits = BootstrapLimits {
        max_functions: 0,
        max_stack_elements: 1,
    };
    for raw in 0u16..=u16::MAX {
        let [high, low] = raw.to_be_bytes();
        let program = [0x41, 1, high, low];
        let setup = spui_font_hint_program::bootstrap_fpgm(&program, word_limits)?;
        writeln!(output, "word\t{raw}\t{}", setup.remaining_stack[0])?;
    }
    let sample_limits = BootstrapLimits {
        max_functions: 3,
        max_stack_elements: 16,
    };
    for (name, program) in SAMPLES {
        let setup = spui_font_hint_program::bootstrap_fpgm(program, sample_limits)?;
        emit_setup(
            &mut output,
            &format!("sample:{name}"),
            program,
            sample_limits,
            &setup,
        )?;
    }
    let (mut definitions, mut dispatches) = (0usize, 0usize);
    for entry in faces {
        let data = package.read(entry)?;
        let font = spui_font::parse(&data)?;
        let limits = BootstrapLimits::from_maxp(font.table(*b"maxp")?)?;
        let program = if font.tables().iter().any(|table| table.tag == *b"fpgm") {
            font.table(*b"fpgm")?
        } else {
            &[]
        };
        let setup = spui_font_hint_program::bootstrap_fpgm(program, limits)?;
        let key = format!(
            "{:08X}:{:08X}:{:08X}",
            entry.key.type_id, entry.key.group_id, entry.key.instance_id
        );
        emit_setup(&mut output, &key, program, limits, &setup)?;
        definitions += setup
            .functions
            .iter()
            .filter(|value| value.is_some())
            .count();
        dispatches += setup.dispatches.len();
    }
    output.flush()?;
    println!(
        "scanner cases=766 signed words=65536 source faces=5 FPGM definitions={definitions} setup dispatches={dispatches}; bodies unexecuted, no hint/raster comparison"
    );
    Ok(())
}
