//! Compare explicit x87 contexts with independent integer/rational fixtures.
use spore_core::native_x87::{Context, Extended, X87Error};
use std::{cmp::Ordering, io::Write};

fn relation(value: Ordering) -> i32 {
    match value {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: native_x87_verify <fixtures TSV> <new result TSV>".into());
    }
    let source = std::fs::read_to_string(&args[0])?;
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])?;
    let mut count = 0;
    for (index, line) in source.lines().enumerate() {
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 8 {
            return Err(format!("malformed fixture line{}", index + 1).into());
        }
        let ctx = Context::from_control_word(u16::from_str_radix(row[0], 16)?)?;
        let left = Extended::from_f32(f32::from_bits(u32::from_str_radix(row[2], 16)?))?;
        let right = Extended::from_f32(f32::from_bits(u32::from_str_radix(row[3], 16)?))?;
        let result = match row[1] {
            "A" => ctx.add(left, right),
            "S" => ctx.sub(left, right),
            "M" => ctx.mul(left, right),
            "D" => ctx.div(left, right),
            "Q" => ctx.sqrt(left),
            _ => return Err("unknown fixture operation".into()),
        }?;
        let bytes: String = result
            .to_le_bytes()
            .iter()
            .map(|v| format!("{v:02X}"))
            .collect();
        let (stored, compare_commit) = match ctx.store_f32(result) {
            Ok(value) => (
                format!("{:08X}", value.to_bits()),
                relation(ctx.compare(result, Extended::from_f32(value)?)?).to_string(),
            ),
            Err(X87Error::NonFiniteResult) => ("-".into(), "-".into()),
            Err(error) => return Err(error.into()),
        };
        let actual = format!(
            "{}\t{}\t{}\t{}\t{bytes}\t{stored}\t{}\t{compare_commit}",
            row[0],
            row[1],
            row[2],
            row[3],
            relation(ctx.compare(result, left)?)
        );
        if actual != line {
            return Err(format!(
                "x87 fixture{} differs\nexpected:{line}\nactual:  {actual}",
                index + 1
            )
            .into());
        }
        writeln!(output, "{actual}")?;
        count += 1;
    }
    println!("NATIVE_X87_VERIFY records={count} exact=true; explicit contexts, retail control word unknown");
    Ok(())
}
