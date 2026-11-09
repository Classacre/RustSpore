//! Compare four fixed native endpoint cases with the admitted closed-domain model.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_legacy_ciasin_interior_v1::{calculate_finite_closed, FiniteClosedAdmission},
    native_x87::{Context, Precision, Rounding},
};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "4160fef416ed1b079d46fd1a695b5ee468d1b9eff92971e38ffde1c63693ea64";
fn hex(v: &serde_json::Value, n: usize) -> Result<u32, Box<dyn Error>> {
    let s = v.as_str().ok_or("missing hex")?;
    if s.len() != n || !s.bytes().all(|x| x.is_ascii_hexdigit()) {
        return Err("bad hex".into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 2 || !a[1].eq_ignore_ascii_case(PIN) {
        return Err(
            "usage: cell_legacy_ciasin_endpoint_compare_v1 <receipt> <reviewed-sha256>".into(),
        );
    }
    let mut raw = Vec::new();
    File::open(&a[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        (
            "schema",
            "cell-legacy-ciasin-endpoint-relocated-native-fixed-v1",
        ),
        (
            "dll_sha256",
            "ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185",
        ),
        (
            "host_sha256",
            "702be2f0e0f9bfa00b7b43909b8b634ccf761d736f2fab4eba2f1585982ec934",
        ),
        (
            "asm_sha256",
            "b9b11f309a367bda51139c2aeabdde6e56ab64d3f3c4e8e42859efb77bb1e58f",
        ),
        (
            "generator_sha256",
            "023192b833727cf0bf944a6ddd39846f6345e8e3c6992b13ebaa5706c5bb11f7",
        ),
        (
            "manifest_sha256",
            "592f4572292d5b658aa11fa52f6bee32ce24117308c25a5e4f7ce4564db0b1f3",
        ),
        (
            "source_manifest_sha256",
            "7569b2d5bb4d017f46110ca4ad8c2d16f7977a4f9892455ce2892a99607ff1af",
        ),
        (
            "runner_sha256",
            "eba8f4ed7fb389c9ed84b5ec0f4d4d4c0f387310e7b0f82945221c4266dd7fec",
        ),
        (
            "stdout_sha256",
            "a5f1bbd50c9e5db671d8bf94da41aeca166cef7c14b81a2c7ed09e7a476aac92",
        ),
    ] {
        if v[k].as_str() != Some(w) {
            return Err(format!("{k} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("status mismatch".into());
    }
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 4 {
        return Err("case count".into());
    }
    let mut stdout = Vec::new();
    for (i, row) in cases.iter().enumerate() {
        let input = hex(&row["input_bits"], 8)?;
        let output = hex(&row["output_bits"], 8)?;
        if row["return_global"].as_u64() != Some((i / 2) as u64)
            || input != [0x3f800000, 0xbf800000][i % 2]
        {
            return Err(format!("case {i} fixture mismatch").into());
        }
        let got = calculate_finite_closed(
            f32::from_bits(input),
            Context {
                precision: Precision::Bits53,
                rounding: Rounding::NearestEven,
            },
            FiniteClosedAdmission,
        )?;
        if got.to_bits() != output {
            return Err(format!("case {i} output mismatch").into());
        }
        let cw = hex(&row["cw"], 4)?;
        let sw = hex(&row["sw"], 4)?;
        let mx = hex(&row["mxcsr"], 8)?;
        let flags = hex(&row["eflags"], 8)?;
        if cw != 0x27f || sw & 0x38cd != 0 || mx != 0x1f80 || flags & 0x400 != 0 {
            return Err(format!("case {i} FP/DF mismatch").into());
        }
        stdout.extend(input.to_le_bytes());
        stdout.extend(output.to_le_bytes());
        stdout.extend((cw as u16).to_le_bytes());
        stdout.extend((sw as u16).to_le_bytes());
        stdout.extend(mx.to_le_bytes());
        stdout.extend(flags.to_le_bytes());
        stdout.extend(0u32.to_le_bytes());
    }
    if format!("{:x}", Sha256::digest(&stdout))
        != v["stdout_sha256"].as_str().ok_or("missing stdout")?
    {
        return Err("stdout hash".into());
    }
    println!("MATCH: four synthetic +/-1 endpoint/return-global cases; receipt and FP metadata validated.");
    println!("Legacy dispatch is admitted; SSE2 and live game behavior are unproven. The relocated host patches four addresses, does not load the DLL or instrument branches, and has no guard pages. The model does not reproduce native FP status.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
