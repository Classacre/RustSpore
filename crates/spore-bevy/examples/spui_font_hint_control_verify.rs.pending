//! Pending module export: original ordinary call/control comparison only.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    spui_font,
    spui_font_hint_control::{self, ProgramWindow, SharedHintFrame},
    spui_font_hint_program::{self, BootstrapLimits},
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};

const FACES: [u32; 5] = [0x511b49ca, 0x61639dfb, 0x6bbc4e0e, 0xbacaaaa5, 0xffe355df];
const BODY: &[u8] = &[0xb0, 1, 0x60, 0x2d];
const MAIN_BASE: u32 = 0x320000;
const FONT_BASE: u32 = 0x350000;
const CALLER_CURSOR: u32 = 0xfeedface;
const ALIASES: [i32; 9] = [
    i32::MIN,
    i32::MIN + 1,
    -65536,
    -32768,
    -32767,
    -1,
    65536,
    65537,
    i32::MAX,
];
const CASES: [(&str, i32, i32); 12] = [
    ("call", 0, 0),
    ("call", 0, i32::MAX),
    ("loop", 0, 7),
    ("loop", 1, 7),
    ("loop", 2, 7),
    ("loop", 32767, 7),
    ("loop", 32768, 7),
    ("loop", 32769, 7),
    ("loop", 65535, 7),
    ("loop", -32768, 7),
    ("loop", 65537, 7),
    ("loop", i32::MIN + 1, 7),
];

fn frame(stack: Vec<i32>, opcode: u32) -> SharedHintFrame {
    SharedHintFrame {
        operand_stack: stack,
        cursor: CALLER_CURSOR,
        last_opcode: opcode,
        program_selector: 0,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: spui_font_hint_control_verify <Spore_Graphics.package> <new call TSV>".into(),
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
    for count in (0..65536).chain(ALIASES) {
        writeln!(
            output,
            "count\t{count}\t{}",
            spui_font_hint_control::loop_iterations(count)
        )?;
    }
    let owned_definitions = [
        Some(spui_font_hint_program::FunctionDefinition {
            offset: 0,
            length: 3,
            program_selector: 1,
            body: &BODY[..3],
        }),
        None,
    ];
    let owned_programs = [
        ProgramWindow {
            base_address: MAIN_BASE,
            bytes: &[],
        },
        ProgramWindow {
            base_address: FONT_BASE,
            bytes: BODY,
        },
    ];
    for (kind, count, seed) in CASES {
        let mut state = if kind == "loop" {
            frame(vec![seed, count, 0], 0x2a)
        } else {
            frame(vec![seed, 0], 0x2b)
        };
        let invocation = if kind == "loop" {
            spui_font_hint_control::prepare_loop_call(
                &mut state,
                &owned_definitions,
                &owned_programs,
            )?
        } else {
            spui_font_hint_control::prepare_call(&mut state, &owned_definitions, &owned_programs)?
        };
        let mut calls = 0usize;
        spui_font_hint_control::dispatch_ordinary(&mut state, invocation, |shared, target| {
            assert_eq!(target.body, &BODY[..3]);
            assert_eq!(shared.cursor, target.start_address);
            shared.operand_stack[0] = shared.operand_stack[0].wrapping_add(1);
            shared.last_opcode = 0x60;
            shared.cursor = target.end_address;
            calls += 1;
            Ok::<(), std::io::Error>(())
        })?;
        writeln!(
            output,
            "owned\t{kind}\t{count}\t{seed}\t{calls}\t{}\t{}\t{}\t{}\t{}",
            state.operand_stack[0],
            state.cursor,
            state.last_opcode,
            state.program_selector,
            state.operand_stack.len()
        )?;
    }
    let mut bindings = 0usize;
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
        let count = setup
            .functions
            .iter()
            .filter(|value| value.is_some())
            .count();
        writeln!(output, "{key}\tbindings\t{count}")?;
        let programs = [
            ProgramWindow {
                base_address: MAIN_BASE,
                bytes: &[],
            },
            ProgramWindow {
                base_address: FONT_BASE,
                bytes: program,
            },
        ];
        for (index, definition) in setup.functions.iter().enumerate() {
            if definition.is_none() {
                continue;
            }
            let mut state = frame(vec![7, index as i32], 0x2b);
            let invocation =
                spui_font_hint_control::prepare_call(&mut state, &setup.functions, &programs)?;
            writeln!(
                output,
                "{key}\tresolve\t{index}\t{}\t{}\t{}\t{}",
                invocation.start_address,
                invocation.end_address,
                invocation.program_selector,
                state.operand_stack.len()
            )?;
            bindings += 1;
        }
    }
    output.flush()?;
    println!(
        "low-word counts=65536 upper aliases=9 owned body cases=12 installed FPGM range bindings={bindings}; source font bodies/PREP/glyphs unexecuted"
    );
    Ok(())
}
