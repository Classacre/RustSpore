//! Explicit-input, identity-local subset of Window matrix update 95A250.
//! Usage: cell_ui_matrix_fixture_v2 <left> <top> <right> <bottom> <parent-sx> <parent-sy> <parent-tx> <parent-ty>
//! Every input is exactly eight hexadecimal digits representing one f32 word.
//! Input rectangles are already laid out. The parent matrix is an explicit
//! axis-scale/translation fixture, not a captured retail Window or screen.
use spore_core::spui_draw::UiMatrix;
use std::error::Error;

/// 95A250's +14C update with identity local +C4, kind-1 affine parent,
/// parent +1E8 null, and recomputation already selected. Parent composition
/// and center/half-span operations follow 95E9B0/95F980 and assembly order.
/// Does not compute visibility, clipping, events, root size or paint ordering.
pub fn identity_local_window_matrix(rect: [f32; 4], parent: UiMatrix) -> Result<UiMatrix, Box<dyn Error>> {
    if rect.iter().chain(parent.rows.iter().flatten()).any(|v| !v.is_finite()) {
        return Err("nonfinite input".into());
    }
    let m = parent.rows;
    if parent.kind != 1 || m[0][1] != 0. || m[0][2] != 0. || m[0][3] != 0.
        || m[1][0] != 0. || m[1][2] != 0. || m[1][3] != 0.
        || m[2] != [0., 0., 1., 0.] || m[3][2] != 0. || m[3][3] != 1. {
        return Err("parent must be kind-1 2D scale/translation".into());
    }
    let cx = (rect[2] + rect[0]) * 0.5;
    let cy = (rect[3] + rect[1]) * 0.5;
    let mut current = UiMatrix { kind: 1, rows: [
        [1.,0.,0.,0.], [0.,1.,0.,0.], [0.,0.,1.,0.],
        [0. + 1. * cx, 0. + 1. * cy, 0. + 1. * 0., 1. + 1. * 0.],
    ]}.then(parent)?;
    let hx = (rect[2] - rect[0]) * 0.5;
    let hy = (rect[3] - rect[1]) * 0.5;
    for column in 0..4 {
        let subtract = current.rows[0][column] * hx + current.rows[1][column] * hy;
        current.rows[3][column] = current.rows[3][column] - subtract;
    }
    if current.rows.iter().flatten().any(|v| !v.is_finite()) { return Err("nonfinite output".into()); }
    Ok(current)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 8 { return Err("usage: cell_ui_matrix_fixture_v2 <left> <top> <right> <bottom> <parent-sx> <parent-sy> <parent-tx> <parent-ty>".into()); }
    let words: Vec<u32> = args.iter().map(|s| {
        if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("each input must be exactly eight hexadecimal digits");
        }
        u32::from_str_radix(s, 16).map_err(|_| "invalid f32 word")
    }).collect::<Result<_, _>>()?;
    let v: Vec<f32> = words.into_iter().map(f32::from_bits).collect();
    let parent = UiMatrix { kind: 1, rows: [
        [v[4],0.,0.,0.], [0.,v[5],0.,0.], [0.,0.,1.,0.], [v[6],v[7],0.,1.],
    ]};
    let result = identity_local_window_matrix([v[0],v[1],v[2],v[3]], parent)?;
    println!("# Explicit input fixture only; no active parent, runtime frame or clipping established");
    print!("matrix");
    for row in result.rows { for x in row { print!("\t{:08X}", x.to_bits()); } }
    println!();
    Ok(())
}
