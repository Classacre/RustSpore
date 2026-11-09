//! Compare one controlled five-group gather with the retained original 73BB00
//! output. The four part buffers are supplied snapshots, not evaluated poses.
use sha2::{Digest, Sha256};
use spore_core::anim_palette::{
    gather_runtime_palette, BoneMatrix, GeneratedTable, GroupMatrices, RuntimePaletteGroup,
};
use std::{env, fs, path::PathBuf};

fn read_pinned(path: PathBuf, expected: &str, rows: usize) -> Vec<BoneMatrix> {
    let bytes = fs::read(path).expect("read supplied matrix input");
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        expected,
        "matrix input hash"
    );
    assert_eq!(bytes.len(), rows * 48);
    bytes
        .chunks_exact(48)
        .map(|row| {
            std::array::from_fn(|i| {
                f32::from_bits(u32::from_le_bytes(
                    row[i * 4..i * 4 + 4].try_into().unwrap(),
                ))
            })
        })
        .collect()
}

fn main() {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let prefix = read_pinned(
        args.next().expect("prefix file"),
        "58da65a8674f00994361b8eced7d2da5748930e3cd650a3104c8c4e56c51135f",
        10,
    );
    let first = read_pinned(
        args.next().expect("part 0"),
        "30faf68cb04f89e92509de924ead1d2f2431e4cace145803f24095cb24bb89fb",
        5,
    );
    let second = read_pinned(
        args.next().expect("part 1"),
        "b231275e583d06209366486c3c75cc433c388db5c60ba0d7b55b77cc3a12e54e",
        7,
    );
    let third = read_pinned(
        args.next().expect("part 2"),
        "b231275e583d06209366486c3c75cc433c388db5c60ba0d7b55b77cc3a12e54e",
        7,
    );
    let fourth = read_pinned(
        args.next().expect("part 3"),
        "b231275e583d06209366486c3c75cc433c388db5c60ba0d7b55b77cc3a12e54e",
        7,
    );
    let native =
        fs::read(args.next().expect("retained original stdout")).expect("read original stdout");
    assert!(args.next().is_none(), "unexpected arguments");
    assert_eq!(
        format!("{:x}", Sha256::digest(&native)),
        "81be67cf29226b220ec448ac9deab578fca57f5beb553adf152cfceccd5c9682"
    );
    assert_eq!(native.len(), 1744);
    let groups = [
        RuntimePaletteGroup {
            first_bone: 0,
            bone_count: 10,
            matrices: GroupMatrices::Identity,
        },
        RuntimePaletteGroup {
            first_bone: 10,
            bone_count: 5,
            matrices: GroupMatrices::Evaluated(&first),
        },
        RuntimePaletteGroup {
            first_bone: 15,
            bone_count: 7,
            matrices: GroupMatrices::Evaluated(&second),
        },
        RuntimePaletteGroup {
            first_bone: 22,
            bone_count: 7,
            matrices: GroupMatrices::Evaluated(&third),
        },
        RuntimePaletteGroup {
            first_bone: 29,
            bone_count: 7,
            matrices: GroupMatrices::Evaluated(&fourth),
        },
    ];
    let result = gather_runtime_palette(
        true,
        &groups,
        GeneratedTable {
            requested_count: 10,
            matrices: Some(&prefix),
        },
    )
    .expect("bounded gather");
    assert_eq!(
        (
            result.bound_generated_count,
            result.copied_generated_count,
            result.matrices.len()
        ),
        (10, 10, 36)
    );
    let bytes: Vec<u8> = result
        .matrices
        .iter()
        .flatten()
        .flat_map(|x| x.to_bits().to_le_bytes())
        .collect();
    assert_eq!(bytes.as_slice(), &native[..1728], "36 matrix output bytes");
    assert_eq!(
        u32::from_le_bytes(native[1728..1732].try_into().unwrap()),
        36
    );
    assert_eq!(
        u32::from_le_bytes(native[1732..1736].try_into().unwrap()),
        4
    );
    println!("matched 36 supplied matrices against retained 73BB00 output; synthetic prefix and scripted, unevaluated part snapshots; live binding and pose unverified");
}
