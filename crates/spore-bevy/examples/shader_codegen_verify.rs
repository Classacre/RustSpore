//! Source-template assembly and installed D3DX compiler conformance.
#[cfg(windows)]
#[path = "../src/native_d3dx.rs"]
mod native_d3dx;

#[cfg(windows)]
fn instruction_tokens(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    if bytes.len() % 4 != 0 {
        return Err("unaligned shader".into());
    }
    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
        .collect();
    if words.first() != Some(&0xfffe0300) {
        return Err("expected vertex model3".into());
    }
    let mut result = vec![words[0]];
    let mut at = 1;
    while at < words.len() {
        let token = words[at];
        let opcode = token & 0xffff;
        let length = if opcode == 0xfffe {
            1 + ((token >> 16) & 0x7fff) as usize
        } else if opcode == 0xffff {
            1
        } else {
            1 + ((token >> 24) & 15) as usize
        };
        let row = words
            .get(at..at + length)
            .ok_or("truncated shader instruction")?;
        if opcode != 0xfffe {
            result.extend_from_slice(row);
        }
        at += length;
        if opcode == 0xffff {
            if at != words.len() {
                return Err("trailing shader words".into());
            }
            return Ok(result);
        }
    }
    Err("shader has no END".into())
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use spore_core::{
        shader_codegen::{self, SourceTemplates},
        shader_fragments, shader_pack,
    };
    use std::{collections::BTreeSet, io::Write, path::Path};
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) || (args.len() == 3 && args[2] != "--all") {
        return Err("usage: shader_codegen_verify <install> <new-output-directory> [--all]".into());
    }
    let install = Path::new(&args[0]);
    let directory = Path::new(&args[1]);
    std::fs::create_dir(directory)?;
    let image = std::fs::read(install.join("SporeBin/SporeApp.exe"))?;
    let templates = SourceTemplates::from_original_executable(&image)?;
    let package = dbpf::PackageIndex::open(install.join("Data/PatchData.package"))?;
    let read = |instance| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let entry = package
            .find_instance(0x0469a3f7, instance)
            .ok_or("source shader resource absent")?;
        Ok(package.read(entry)?)
    };
    let library = shader_fragments::parse(&read(0x4021200200000003)?)?;
    let pack = shader_pack::parse(&read(0x4021200400000003)?)?;
    let mut recipes = BTreeSet::new();
    recipes.insert(vec![0x14, 0x17, 0x19, 0x15, 0x31, 0x30, 0x29, 1]);
    recipes.insert(vec![0x14, 0x17, 0x18, 0x15, 0x31, 0x30, 0x2b, 1]);
    if args.len() == 3 {
        for program in &pack.compiled_vertex.programs {
            recipes.insert(program.fragment_recipe()?.to_vec());
        }
    }
    let compiler = native_d3dx::Compiler::system()?;
    let mut report = std::fs::File::create_new(directory.join("results.tsv"))?;
    writeln!(report,"recipe\tsource_bytes\tcompiled_bytes\tcached\tmetadata_equal\texecutable_tokens_equal\tvaryings")?;
    let (mut compiled_count, mut cached_count) = (0usize, 0usize);
    for recipe in recipes {
        let label: String = recipe.iter().map(|v| format!("{v:02x}")).collect();
        let source = shader_codegen::generate_vertex(&library, &recipe, &templates)?;
        let compiled = compiler.compile(&source.hlsl, templates.entrypoint()?, "vs_3_0", 0x12)?;
        if !compiled.diagnostics.is_empty() {
            println!("recipe={label} diagnostics={:?}", compiled.diagnostics);
        }
        let generated_metadata: Vec<_> = source.uniforms.iter().map(|u| u.metadata).collect();
        let (mut metadata_equal, mut tokens_equal) = (false, false);
        let cached = pack.compiled_vertex.lookup(&recipe)?;
        if let Some(cached) = cached {
            cached_count += 1;
            metadata_equal =
                cached.uniforms == generated_metadata && cached.uniform_words == source.registers;
            tokens_equal =
                instruction_tokens(&compiled.bytecode)? == instruction_tokens(&cached.bytecode)?;
        }
        let mut hlsl = std::fs::File::create_new(directory.join(format!("{label}.hlsl")))?;
        hlsl.write_all(source.hlsl.as_bytes())?;
        let mut bytecode = std::fs::File::create_new(directory.join(format!("{label}.vs")))?;
        bytecode.write_all(&compiled.bytecode)?;
        writeln!(
            report,
            "{label}\t{}\t{}\t{}\t{metadata_equal}\t{tokens_equal}\t{}",
            source.hlsl.len(),
            compiled.bytecode.len(),
            cached.is_some(),
            source.varying_count
        )?;
        report.flush()?;
        if cached.is_some() && (!metadata_equal || !tokens_equal) {
            return Err(format!("original cached program mismatch for recipe {label}").into());
        }
        compiled_count += 1;
    }
    println!("recipes_compiled={compiled_count} cached_programs_exact={cached_count} comments_excluded_only_from_comparison=true compiler=system-d3dx9_27 profile=vs_3_0 flags=0x12 original_executable_executed=false");
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This conformance example requires the installed Windows D3DX9 compiler.");
    std::process::exit(1);
}
