//! Explicit, hash-bound E219 position diagnostic; not a live pose or native draw.
//!
//! The caller supplies ten conditional generated-pose matrices and 26
//! source-stored, unevaluated identity matrices. Direct-draw path selection is
//! admitted, not observed. Normals and tangents remain unchanged.
use sha2::{Digest, Sha256};
use spore_core::{
    gmdl,
    gmdl_direct_draw_influences_v1::{self, DrawPath},
    gmdl_direct_draw_position_diagnostic_v1::{deform_positions, Matrix},
    gmdl_skin,
};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
};
#[cfg(target_arch = "x86_64")]
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 8 || args[1] != "--admit-synthetic-direct-path" {
        eprintln!("usage: e219_controlled_palette_position_v1 --admit-synthetic-direct-path <model.gmdl> <10-matrix-prefix.bin> <26-stored-identity-matrices.bin> <captured-vertices.bin> <captured-u32-indices.bin> <new-output.bin>\nControlled position-only diagnostic; no native/live pose or material. Normals and tangents remain unchanged.");
        std::process::exit(2)
    }
    let read = |path: &str, want: &str| {
        let b = fs::read(path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&b)),
            want,
            "input hash mismatch: {path}"
        );
        b
    };
    let raw = read(
        &args[2],
        "95ba0f5e5e71761ccf0f2ef919cce9bcc8d40303be4b1b7a9fa954a7c7cb8ff4",
    );
    let prefix = read(
        &args[3],
        "58da65a8674f00994361b8eced7d2da5748930e3cd650a3104c8c4e56c51135f",
    );
    let fallback = read(
        &args[4],
        "7f38e9297f5133f85ecb392f59bfb5826c7c2493ea2a5b092eb918f6c6834738",
    );
    let verts = read(
        &args[5],
        "52304dc47a2ba1097fef45ef21c315aab36e677a7b3883ce8c744e3d5240dcb4",
    );
    let indices = read(
        &args[6],
        "6891da4e135b3dbd32426d96b663448f6978427df99006d95dee06ea150a2fad",
    );
    assert_eq!(
        (prefix.len(), fallback.len(), verts.len(), indices.len()),
        (480, 1248, 2555 * 36, 12732 * 4)
    );
    let identity: [f32; 12] = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0.];
    for row in fallback.chunks_exact(48) {
        for (j, v) in row.chunks_exact(4).enumerate() {
            assert_eq!(
                u32::from_le_bytes(v.try_into().unwrap()),
                identity[j].to_bits()
            );
        }
    }
    let model = gmdl::parse(&raw).unwrap();
    let skin = gmdl_skin::parse(&raw, &model).unwrap().unwrap();
    assert_eq!(model.meshes.len(), 1);
    let mesh = &model.meshes[0];
    let vb = &model.vertex_buffers[mesh.vertex_index as usize];
    let tri = &model.triangle_buffers[mesh.triangle_index as usize];
    assert_eq!(
        (vb.vertex_count, vb.stride, tri.indices.len()),
        (2555, 36, 12732)
    );
    assert_eq!(&raw[vb.data_offset..vb.data_offset + verts.len()], verts);
    let encoded: Vec<_> = tri.indices.iter().flat_map(|x| x.to_le_bytes()).collect();
    assert_eq!(encoded, indices);
    let draw = gmdl_direct_draw_influences_v1::resolve(
        &raw,
        &model,
        &skin,
        0,
        DrawPath::DirectFresh769a80Unmodified,
    )
    .unwrap();
    assert_eq!(
        (
            draw.descriptor.first_word,
            draw.descriptor.bone_count,
            draw.referenced.len()
        ),
        (0, 36, 2555)
    );
    for v in verts.chunks_exact(36) {
        assert_eq!(v[32..36].iter().map(|x| *x as u16).sum::<u16>(), 255);
        assert!(v[28..32].iter().all(|x| x % 3 == 0 && *x <= 105));
    }
    let mut palette: Vec<Matrix> = Vec::new();
    for row in prefix.chunks_exact(48).chain(fallback.chunks_exact(48)) {
        let mut m = [[0f32; 4]; 3];
        for (j, x) in row.chunks_exact(4).enumerate() {
            m[j / 4][j % 4] = f32::from_bits(u32::from_le_bytes(x.try_into().unwrap()));
            assert!(m[j / 4][j % 4].is_finite());
        }
        palette.push(m);
    }
    assert_eq!(palette.len(), 36);
    let mut csr = 0u32;
    unsafe {
        std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut csr,options(nostack,preserves_flags));
    }
    struct Restore(u32);
    impl Drop for Restore {
        fn drop(&mut self) {
            unsafe {
                std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&self.0,options(nostack,preserves_flags));
            }
        }
    }
    let _guard = Restore(csr);
    let mode = 0x1f80u32;
    unsafe {
        std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&mode,options(nostack,preserves_flags));
    }
    let identity_matrix: Matrix = [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.]];
    let control = deform_positions(
        &raw,
        &model,
        &skin,
        0,
        DrawPath::DirectFresh769a80Unmodified,
        &[identity_matrix; 36],
    )
    .unwrap();
    for (a, b) in verts.chunks_exact(36).zip(control.chunks_exact(36)) {
        assert_eq!(&a[12..], &b[12..]);
        for j in 0..3 {
            let x = f32::from_le_bytes(a[j * 4..j * 4 + 4].try_into().unwrap());
            let y = f32::from_le_bytes(b[j * 4..j * 4 + 4].try_into().unwrap());
            assert!(x.is_finite() && y.is_finite() && (x - y).abs() <= 0.00002 * (1. + x.abs()));
        }
    }
    let output = deform_positions(
        &raw,
        &model,
        &skin,
        0,
        DrawPath::DirectFresh769a80Unmodified,
        &palette,
    )
    .unwrap();
    assert_eq!(output.len(), verts.len());
    let mut changed = 0;
    let mut significant = 0;
    for ((a, b), c) in verts
        .chunks_exact(36)
        .zip(output.chunks_exact(36))
        .zip(control.chunks_exact(36))
    {
        assert_eq!(&a[12..], &b[12..]);
        changed += usize::from(a[..12] != b[..12]);
        let mut different = false;
        for j in 0..3 {
            let x = f32::from_le_bytes(b[4 * j..4 * j + 4].try_into().unwrap());
            let y = f32::from_le_bytes(c[4 * j..4 * j + 4].try_into().unwrap());
            different |= (x - y).abs() > 0.000001 * (1. + y.abs());
        }
        significant += usize::from(different);
    }
    assert!(significant > 0);
    assert_eq!(
        format!("{:x}", Sha256::digest(&output)),
        "2d610c41b6e008cd67033632eb55058cca8da434b8a51f092f820df49a8a905a",
        "diagnostic output differs from retained fixture"
    );
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[7])
        .unwrap();
    f.write_all(&output).unwrap();
    f.flush().unwrap();
    println!("controlled=true vertices=2555 indices=12732 matrices=36 changed_bits={changed} changed_vs_identity_control={significant} normals_tangents=unchanged source_stored_fallback=26 native_or_live_pose=false");
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("this fixed SSE diagnostic requires x86_64");
    std::process::exit(2);
}
