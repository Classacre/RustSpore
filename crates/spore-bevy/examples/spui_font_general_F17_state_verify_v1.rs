//! Unexported all-glyph bounded coverage verifier; source provenance required.
use dbpf::native_index::NativePackageIndex;
use spore_core::spui_font_hint_program;
use spore_core::spui_font_intersection;
use spore_core::spui_font_hint_general_f20_state_v1 as axis;
use spore_core::{
    spui_font, spui_font_hint_prep,
    spui_font_scaling::{self, DesignCoordinate},
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};
fn hex(data: &[u8]) -> String {
    data.iter().map(|x| format!("{x:02x}")).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "usage: verifier <Spore_Graphics.package> <owned source input TSV> <new target TSV>"
                .into(),
        );
    }
    let output = std::path::absolute(&args[2])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let entry = package
        .entries()
        .iter()
        .find(|e| {
            e.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && e.key.group_id == spui_font::FONT_NAMESPACE
                && e.key.instance_id == 0x511b49ca
        })
        .ok_or("missing font0")?;
    let data = package.read(entry)?;
    let font = spui_font::parse(&data)?;
    let metrics = font.metrics();
    let _cmap = font.cmap_format4(3, 1)?;
    let fpgm = font.table(*b"fpgm")?;
    let maxp = font.table(*b"maxp")?;
    let prep = spui_font_hint_prep::execute_font0_prep(
        fpgm,
        font.table(*b"prep")?,
        maxp,
        font.table(*b"cvt ")?,
        metrics.units_per_em,
    )
    .map_err(|e| format!("PREP: {e:?}"))?;
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?,
    );
    let design = vec![DesignCoordinate { x: 0, y: 0 }; 8];
    let scaled = vec![spui_font_scaling::Coordinate26Dot6 { x: 0, y: 0 }; 8];
    let ends = [3];
    for line in std::fs::read_to_string(&args[1])?.lines() {
        let v: Vec<i32> = line.split('\t').map(str::parse).collect::<Result<_, _>>()?;
        if v.len() != 8 {
            return Err("owned F17 input shape".into());
        }
        let x = i16::try_from(v[3])?.to_be_bytes();
        let mut program = if v[1] < 2 {
            vec![u8::try_from(v[1])?]
        } else {
            vec![0xb1, 0, 2, 0x07]
        };
        program.extend_from_slice(&[
            u8::try_from(v[2])?,
            0xb0,
            41,
            0xb8,
            x[0],
            x[1],
            0xb0,
            17,
            0x2b,
            0x44,
        ]);
        let mut current = scaled.clone();
        current[0] = spui_font_scaling::Coordinate26Dot6 { x: v[4], y: v[5] };
        current[2] = spui_font_scaling::Coordinate26Dot6 { x: v[6], y: v[7] };
        let r = axis::execute_simple(
            &program,
            &design,
            &current,
            &ends,
            &prep,
            fpgm,
            maxp,
            [65536, 65536],
        )
        .map_err(|e| format!("F17 owned{}: {e:?}", v[0]))?;
        for (got, before) in r.points.iter().zip(&current) {
            assert_eq!(*got, [before.x, before.y]);
        }
        assert!(r.touched.iter().all(|t| *t == 0));
        assert_eq!(r.storage, prep.storage);
        for i in 0..r.cvt.len() {
            if i != 41 {
                assert_eq!(r.cvt[i], prep.cvt[i]);
            }
        }
        assert!(r.stack.is_empty());
        writeln!(out, "owned{}\tresult\t{}", v[0], r.cvt[41])?;
        writeln!(
            out,
            "owned{}\tframe\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:08X}\t{:08X}\t{:08X}",
            v[0],
            r.projection[0],
            r.projection[1],
            r.freedom[0],
            r.freedom[1],
            r.dual[0],
            r.dual[1],
            r.dot,
            r.callbacks[0],
            r.callbacks[1],
            r.callbacks[2]
        )?;
        writeln!(out, "owned{}\tgraphics\t{}", v[0], hex(&r.graphics))?;
        writeln!(
            out,
            "owned{}\tfinal\t0\t{}\t{}\t{}\t{}",
            v[0], r.calls, r.reference_points[0], r.reference_points[1], r.reference_points[2]
        )?;
    }
    assert!(matches!(
        axis::execute_simple(
            &[0xb1, 1, 0, 0x62],
            &design,
            &scaled,
            &ends,
            &prep,
            fpgm,
            maxp,
            [65536, 65536]
        ),
        Err(axis::Error::Division)
    ));
    assert!(matches!(
        axis::execute_simple(
            &[0xb0, 10, 0x2b],
            &design,
            &scaled,
            &ends,
            &prep,
            fpgm,
            maxp,
            [65536, 65536]
        ),
        Err(axis::Error::UnsupportedFunction(10))
    ));
    out.flush()?;
    Ok(())
}
