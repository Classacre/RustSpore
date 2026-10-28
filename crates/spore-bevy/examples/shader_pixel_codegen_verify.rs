//! Controlled source-pixel generation and cached-program comparison.
//! Writes derived source only into a new directory under the ignored target tree.
#[cfg(windows)]
#[path = "../src/native_d3dx.rs"]
mod native_d3dx;

#[cfg(windows)]
fn instruction_tokens(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    if bytes.len() % 4 != 0 { return Err("unaligned shader".into()); }
    let words: Vec<u32> = bytes.chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap())).collect();
    if words.first() != Some(&0xffff0300) { return Err("expected pixel model3".into()); }
    let mut result = vec![words[0]];
    let mut at = 1;
    while at < words.len() {
        let token = words[at];
        let opcode = token & 0xffff;
        let length = if opcode == 0xfffe { 1 + ((token >> 16) & 0x7fff) as usize }
            else if opcode == 0xffff { 1 }
            else { 1 + ((token >> 24) & 15) as usize };
        let row = words.get(at..at + length).ok_or("truncated instruction")?;
        if opcode != 0xfffe { result.extend_from_slice(row); }
        at += length;
        if opcode == 0xffff {
            if at != words.len() { return Err("trailing shader words".into()); }
            return Ok(result);
        }
    }
    Err("shader has no END".into())
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
    use spore_core::{shader_codegen::SourceTemplates, shader_fragments, shader_pack, shader_pixel_codegen};
    use std::{collections::BTreeSet, io::{Read, Write}, path::Path};
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: shader_pixel_codegen_verify <install> <exact-package> <new-output-directory> <--all|hex-recipe>".into());
    }
    let install = Path::new(&args[0]);
    let package_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    if output.exists() { return Err("output directory must be new".into()); }
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target").canonicalize()?;
    let parent = output.parent().ok_or("output requires a parent directory")?.canonicalize()?;
    if !parent.starts_with(&target) { return Err("output must be under target".into()); }
    let image = std::fs::read(install.join("SporeBin/SporeApp.exe"))?;
    let templates = SourceTemplates::from_original_executable(&image)?;
    let mut file = std::fs::File::open(package_path)?;
    let mut header_bytes = [0; dbpf::HEADER_SIZE];
    file.read_exact(&mut header_bytes)?;
    let header = dbpf::DbpfHeader::parse(&header_bytes)?;
    if header.index_size > 64 * 1024 * 1024 || header.index_entry_count > 2_000_000 {
        return Err("package index exceeds preflight bounds".into());
    }
    // The index reader reopens the path: input files must remain stable.
    let package = NativePackageIndex::open(package_path)?;
    let fragment_key = NativeResourceKey::from_scoped_instance(0x0469a3f7, 0x4021200200000003);
    let pack_key = NativeResourceKey::from_scoped_instance(0x0469a3f7, 0x4021200400000003);
    let read = |key| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let entry = package.index().find_unique(key)?.ok_or("exact shader resource absent")?;
        if entry.stored_size > 64 * 1024 * 1024 || entry.decompressed_size > 64 * 1024 * 1024 {
            return Err("shader resource exceeds preflight bounds".into());
        }
        Ok(package.read(entry)?)
    };
    let library_bytes = read(fragment_key)?;
    let pack_bytes = read(pack_key)?;
    let library = shader_fragments::parse(&library_bytes)?;
    let pack = shader_pack::parse(&pack_bytes)?;
    let builder = pack.builders.iter().find(|b| b.header.shader_id() == 0x80000004
        && b.header.name == "generic_skinning").ok_or("generic_skinning builder absent")?;
    if builder.header.words[5] != 0xffff0300 { return Err("unexpected pixel profile".into()); }
    let flags = if builder.header.words[6] & 2 != 0 { 0x32 } else { 0x12 };
    let mut recipes = BTreeSet::new();
    if args[3] == "--all" {
        for program in &pack.compiled_pixel.programs {
            recipes.insert(program.fragment_recipe()?.to_vec());
        }
    } else {
        let text = args[3].as_bytes();
        if text.is_empty() || text.len() % 2 != 0 || text.len() > 62 {
            return Err("recipe must contain 1..31 hexadecimal bytes".into());
        }
        let recipe: Vec<u8> = text.chunks_exact(2).map(|pair| {
            let s = std::str::from_utf8(pair)?;
            Ok::<u8, Box<dyn std::error::Error>>(u8::from_str_radix(s, 16)?)
        }).collect::<Result<_, _>>()?;
        if recipe.contains(&0) { return Err("recipe must not contain zero terminator".into()); }
        recipes.insert(recipe);
    }
    std::fs::create_dir(output)?;
    std::fs::write(output.join("fragment-library.bin"), &library_bytes)?;
    std::fs::write(output.join("shader-pack.bin"), &pack_bytes)?;
    println!("SOURCE_PIXEL_INPUT package={} fragment_key={} pack_key={} profile=3 builder=generic_skinning flags={flags:02x} cached_original_builder_flags=unknown package_must_remain_stable=true", package_path.display(), fragment_key, pack_key);
    let compiler = native_d3dx::Compiler::system()?;
    let mut module_path = vec![0u16; 32768];
    let module_length = unsafe {
        windows::Win32::System::LibraryLoader::GetModuleFileNameW(Some(compiler.module_handle()), &mut module_path)
    } as usize;
    if module_length == 0 || module_length >= module_path.len() {
        return Err("cannot identify loaded D3DX compiler module path".into());
    }
    let module_path = String::from_utf16(&module_path[..module_length])?;
    println!("SOURCE_PIXEL_COMPILER loaded_module={module_path:?} process_bits={} architecture={}", usize::BITS, std::env::consts::ARCH);
    let mut report = std::fs::File::create_new(output.join("results.tsv"))?;
    writeln!(report, "recipe\tsource_bytes\tcompiled_bytes\tcached\tmetadata_equal\tregisters_equal\tflags_equal\texecutable_tokens_equal\tvaryings")?;
    let (mut compiled_count, mut cached_count, mut mismatches) = (0usize, 0usize, 0usize);
    for recipe in recipes {
        let label: String = recipe.iter().map(|v| format!("{v:02x}")).collect();
        let source = shader_pixel_codegen::generate_pixel(&library, &recipe, &templates)?;
        let compiled = compiler.compile(&source.hlsl, templates.entrypoint()?, "ps_3_0", flags)?;
        if !compiled.diagnostics.is_empty() { println!("recipe={label} diagnostics={:?}", compiled.diagnostics); }
        let metadata: Vec<_> = source.uniforms.iter().map(|u| u.metadata).collect();
        let dependency_mask = source.uniforms.iter().fold(0u32, |mask, u| mask | u.flags_word());
        let (mut metadata_equal, mut registers_equal, mut flags_equal, mut tokens_equal) = (false, false, false, false);
        let cached = pack.compiled_pixel.lookup(&recipe)?;
        if let Some(cached) = cached {
            cached_count += 1;
            metadata_equal = cached.uniforms == metadata;
            registers_equal = cached.uniform_words == source.registers;
            flags_equal = cached.flags_word == dependency_mask;
            tokens_equal = instruction_tokens(&compiled.bytecode)? == instruction_tokens(&cached.bytecode)?;
            std::fs::write(output.join(format!("{label}.cached.ps")), &cached.bytecode)?;
            if !(metadata_equal && registers_equal && flags_equal && tokens_equal) { mismatches += 1; }
        }
        let mut hlsl = std::fs::File::create_new(output.join(format!("{label}.hlsl")))?;
        hlsl.write_all(source.hlsl.as_bytes())?;
        let mut bytecode = std::fs::File::create_new(output.join(format!("{label}.ps")))?;
        bytecode.write_all(&compiled.bytecode)?;
        writeln!(report,"{label}\t{}\t{}\t{}\t{metadata_equal}\t{registers_equal}\t{flags_equal}\t{tokens_equal}\t{}",
            source.hlsl.len(), compiled.bytecode.len(), cached.is_some(), source.varying_count)?;
        report.flush()?;
        compiled_count += 1;
    }
    println!("recipes_compiled={compiled_count} cached_compared={cached_count} uncached_generated={} mismatches={mismatches} package={} profile=3 builder=generic_skinning flags={flags:02x} cached_original_builder_flags=unknown compiler_flags_common_assumption=true comments_excluded_only_from_comparison=true source_execution=false", compiled_count-cached_count, package_path.display());
    if cached_count == 0 { return Err("uncached source generated; no cached comparison was possible".into()); }
    if mismatches != 0 { return Err("cached pixel comparisons differed; inspect report and compile flags".into()); }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This conformance example requires the installed Windows D3DX9 compiler.");
    std::process::exit(1);
}
