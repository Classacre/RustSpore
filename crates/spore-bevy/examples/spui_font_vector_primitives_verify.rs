//! Unexported original integer vector helper/07/87 arithmetic verifier.
use spore_core::spui_font_vector_primitives as vector;
use std::io::{BufWriter, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage reference.tsv new-target.tsv".into());
    }
    let output = std::path::absolute(&args[1])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?,
    );
    for line in std::fs::read_to_string(&args[0])?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err("fields".into());
        }
        let a: Vec<i32> = fields[2]
            .split(',')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let get = |r: Result<i32, vector::Error>| {
            r.map_err(|e| format!("{} {}: {e:?}", fields[0], fields[1]))
        };
        let result: Vec<i32> = match fields[0] {
            "D16" => vec![get(vector::div16(a[0], a[1]))?],
            "M16" => vec![get(vector::mul16(a[0], a[1]))?],
            "M14" => vec![get(vector::mul14(a[0], a[1] as i16))?],
            "LEN" => vec![get(vector::length(a[0], a[1]))?],
            "DOT" => {
                vec![vector::dot([a[0] as i16, a[1] as i16], [a[2] as i16, a[3] as i16]) as i32]
            }
            "PROJ" => vec![get(vector::project(
                [a[0], a[1]],
                [a[2] as i16, a[3] as i16],
            ))?],
            "MOVE" => {
                let (v, t) = vector::movement(a[0], [a[1] as i16, a[2] as i16], a[3] as u16)
                    .map_err(|e| format!("MOVE:{e:?}"))?;
                vec![v[0], v[1], t as i32]
            }
            "V07" | "V87" => {
                let p = vector::line_vector(a[0], a[1], true).map_err(|e| format!("line:{e:?}"))?;
                let d = if fields[0] == "V87" {
                    vector::line_vector(a[2], a[3], true).map_err(|e| format!("dual:{e:?}"))?
                } else {
                    [16384, 0]
                };
                vec![
                    p[0] as i32,
                    p[1] as i32,
                    d[0] as i32,
                    d[1] as i32,
                    vector::dot(p, [a[4] as i16, a[5] as i16]) as i32,
                    0x8ab870,
                    0x8ab9d0,
                    if fields[0] == "V87" {
                        0x8aba00
                    } else {
                        0x8ab9d0
                    },
                ]
            }
            _ => return Err("kind".into()),
        };
        writeln!(
            out,
            "{}\t{}\t{}\t{}",
            fields[0],
            fields[1],
            fields[2],
            result
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )?;
    }
    assert!(matches!(
        vector::div16(1, 0),
        Err(vector::Error::ZeroDenominator)
    ));
    assert!(matches!(
        vector::div16(i32::MIN, 1),
        Err(vector::Error::Bounds)
    ));
    assert!(matches!(
        vector::length(16777217, 0),
        Err(vector::Error::Bounds)
    ));
    assert!(matches!(
        vector::line_vector(16385, 0, false),
        Err(vector::Error::Bounds)
    ));
    assert!(matches!(
        vector::movement(1, [16384, 0], 0),
        Err(vector::Error::ZeroDenominator)
    ));
    out.flush()?;
    Ok(())
}
