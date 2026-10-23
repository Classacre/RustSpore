//! Compare the independently reconstructed finite SSE2 policy with a reviewed
//! native fixture. This executable only evaluates Rust and external data.
use spore_core::native_crt_acos::{self, AcosPolicy, ConstantWindow};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "usage: native_crt_acos_verify <constant-window.bin> <inputs.tsv> <new-output.tsv>"
                .into(),
        );
    }
    let bytes = std::fs::read(&args[0])?;
    let constants = ConstantWindow::new(&bytes)?;
    let input = std::fs::read_to_string(&args[1])?;
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[2])?;
    let mut rows = 0usize;
    for line in input
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 2 {
            return Err(format!("invalid input row: {line}").into());
        }
        let bits = u64::from_str_radix(fields[1], 16)?;
        let value = f64::from_bits(bits);
        let branch = native_crt_acos::finite_branch(value)?;
        let result =
            native_crt_acos::finite_nonnegative(value, constants, AcosPolicy::Sse2Mxcsr1f80)?;
        writeln!(
            output,
            "{}\t{}\t{:016x}",
            fields[0],
            branch.name(),
            result.to_bits()
        )?;
        rows += 1;
    }
    println!("rows={rows} policy=SSE2-MXCSR1F80 original_host_executed=false");
    Ok(())
}
