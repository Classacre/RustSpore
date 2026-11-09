//! Compare four fixed synthetic 4F8AE0 receipts against the bounded kernel.
//! Original constant addressing and append are substituted in the host.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_kernel_v1::{self as kernel, KernelAdmission, Lane},
    native_sse::Policy,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "cc9c268f7cc20561fa91577836705a68dd6b6db7c868d1bb7bc951063580a731";
fn hex(v: &serde_json::Value, key: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let s = v[key].as_str().ok_or("missing hex field")?;
    if s.len() != width || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("invalid {key}").into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn bytes(v: &serde_json::Value, key: &str, n: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let s = v[key].as_str().ok_or("missing bytes")?;
    if s.len() != n * 2 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("invalid {key}").into());
    }
    Ok((0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()?)
}
fn word(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}
fn pair(v: &serde_json::Value) -> Result<(u32, u32), Box<dyn Error>> {
    let a = v.as_array().ok_or("pair not array")?;
    if a.len() != 2 {
        return Err("pair size".into());
    }
    let parse = |v: &serde_json::Value| -> Result<u32, Box<dyn Error>> {
        let s = v.as_str().ok_or("pair word not string")?;
        if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("pair word invalid".into());
        }
        Ok(u32::from_str_radix(s, 16)?)
    };
    Ok((parse(&a[0])?, parse(&a[1])?))
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err(
            "usage: creature_skin_kernel_compare_v1 <receipt.json> <reviewed-receipt-sha256>"
                .into(),
        );
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != RECEIPT_SHA {
        return Err("receipt too large or SHA mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (key, expected) in [
        ("schema", "creature-skin-kernel-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "5afb84edd5565d0748d0b4bec62a67d822d34793099287fdb111c4e46025a0b8",
        ),
        (
            "manifest_sha256",
            "1bf33c0dd5509512dd6612d15a704cb79d01ff6f306ef68007a86c90665e3072",
        ),
        (
            "instruction_sha256",
            "5564201678116a791b1b99a9c15942a00c604007823ff1e59448352083f45c33",
        ),
        (
            "runner_sha256",
            "3cc4bea9bd64df303189d612f30f99c62749cdfd49fc88c6f69511ac95c34ba8",
        ),
        (
            "stdout_sha256",
            "145d9460061d79de656743fd1c02239df941ecbc3d3eac6136dd564ae41c8f74",
        ),
    ] {
        if v[key].as_str() != Some(expected) {
            return Err(format!("{key} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("synthetic status mismatch".into());
    }
    let fixtures: [([u32; 3], [u32; 4], [u32; 4], [u32; 4], [u32; 4]); 4] = [
        (
            [0x3f800000, 0, 0],
            [0x80000000, 0x3f800000, 0x3f800001, 0x40000000],
            [0x3e800000; 4],
            [0x3f000000, 0x3f000000, 0x3f000000, 0xbf000000],
            [1, 2, 0xdeadbeef, 4],
        ),
        (
            [0x3de5123d, 0, 0],
            [0x40000000, 0, 0, 0],
            [0x3fd878f4, 0, 0, 0],
            [0x3f06af70, 0, 0, 0],
            [9, 0, 0, 0],
        ),
        (
            [0x80000000, 0, 0],
            [0x3f800000, 0, 0, 0],
            [0; 4],
            [1, 0, 0, 0],
            [10, 0, 0, 0],
        ),
        (
            [0x3f800000, 0, 0],
            [0x40000000; 4],
            [0x3e800000; 4],
            [0x3f000000; 4],
            [0, 1, 2, 3],
        ),
    ];
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 4 {
        return Err("case count".into());
    }
    let mut stdout = Vec::new();
    for (i, (row, (point, cutoff, scale, coeff, keys))) in cases.iter().zip(fixtures).enumerate() {
        if row["id"].as_u64() != Some(i as u64) {
            return Err("case order".into());
        }
        let mut ep = Vec::new();
        for x in point {
            ep.extend(x.to_le_bytes())
        }
        let mut eb = vec![0u8; 128];
        for j in 0..4 {
            for (offset, x) in [
                (0x30, cutoff[j]),
                (0x40, scale[j]),
                (0x50, coeff[j]),
                (0x70, keys[j]),
            ] {
                eb[offset + j * 4..offset + j * 4 + 4].copy_from_slice(&x.to_le_bytes())
            }
        }
        let gotp = bytes(row, "point_hex", 12)?;
        let gotb = bytes(row, "block_hex", 128)?;
        if gotp != ep || gotb != eb {
            return Err(format!("case {i} input mismatch").into());
        }
        let lanes = std::array::from_fn(|j| Lane {
            center: [
                word(&gotb, j * 4),
                word(&gotb, 0x10 + j * 4),
                word(&gotb, 0x20 + j * 4),
            ]
            .map(f32::from_bits),
            cutoff: f32::from_bits(word(&gotb, 0x30 + j * 4)),
            scale: f32::from_bits(word(&gotb, 0x40 + j * 4)),
            coefficient: f32::from_bits(word(&gotb, 0x50 + j * 4)),
            key: word(&gotb, 0x70 + j * 4),
        });
        let model = kernel::evaluate(
            point.map(f32::from_bits),
            lanes,
            Policy::Mxcsr1f80,
            KernelAdmission,
        )?;
        let used = row["append_words"].as_array().ok_or("missing append")?;
        let unused = row["unused_words"].as_array().ok_or("missing unused")?;
        if used.len() != model.len || used.len() + unused.len() != 4 {
            return Err(format!("case {i} count mismatch").into());
        }
        stdout.extend((used.len() as u32).to_le_bytes());
        for (j, item) in used.iter().enumerate() {
            let (k, w) = pair(item)?;
            let entry = model.entries[j].ok_or("missing model append")?;
            if (k, w) != (entry.key, entry.weight.to_bits()) {
                return Err(format!("case {i} append {j} mismatch").into());
            }
            stdout.extend(k.to_le_bytes());
            stdout.extend(w.to_le_bytes())
        }
        for item in unused {
            let (k, w) = pair(item)?;
            if (k, w) != (0, 0) {
                return Err(format!("case {i} unused slot changed").into());
            }
            stdout.extend(k.to_le_bytes());
            stdout.extend(w.to_le_bytes())
        }
        let cw = hex(row, "cw", 4)?;
        let sw = hex(row, "sw", 4)?;
        let mx = hex(row, "mxcsr", 8)?;
        let flags = hex(row, "eflags", 8)?;
        if cw != 0x27f
            || sw & 0x38cd != 0
            || mx & !0x3f != 0x1f80
            || mx & 0x0d != 0
            || flags & 0x400 != 0
        {
            return Err(format!("case {i} FP/DF mismatch").into());
        }
        stdout.extend((cw as u16).to_le_bytes());
        stdout.extend((sw as u16).to_le_bytes());
        stdout.extend(mx.to_le_bytes());
        stdout.extend(flags.to_le_bytes());
    }
    if format!("{:x}", Sha256::digest(&stdout))
        != v["stdout_sha256"].as_str().ok_or("missing stdout hash")?
    {
        return Err("stdout hash mismatch".into());
    }
    println!("MATCH: four synthetic point/block fixtures, all append words, unused slots and FP envelopes.");
    println!("The host patches the constant address and substitutes bounded append; no original append allocator, spatial query, palette, mesh or live editor was tested.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1);
    }
}
