//! Conditional E219 source-basis projection from a retained parent capture.
//! No original code runs here. Source basis output is an unvalidated prediction;
//! the identity control compares with the retained instrumented original output.
use sha2::{Digest, Sha256};
use std::{arch::asm, env, fs};

fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn word(b: &[u8], at: usize) -> f32 {
    f32::from_bits(u32::from_le_bytes(b[at..at + 4].try_into().unwrap()))
}

#[cfg(target_arch = "x86_64")]
fn compose(local: [[f32; 4]; 4], parent: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    use std::arch::x86_64::*;
    // The source-order packed multiply/add sequence matches the prior fixed
    // identity-basis comparator; only the explicit basis rows are changed.
    unsafe {
        let p = parent.map(|v| _mm_loadu_ps(v.as_ptr()));
        let mut out = [[0.; 4]; 4];
        for (i, row) in local.iter().enumerate() {
            let x = _mm_loadu_ps(row.as_ptr());
            let yprod = _mm_mul_ps(_mm_shuffle_ps::<85>(x, x), p[1]);
            let xprod = _mm_mul_ps(_mm_shuffle_ps::<0>(x, x), p[0]);
            let mut v = _mm_add_ps(yprod, xprod);
            v = _mm_add_ps(v, _mm_mul_ps(_mm_shuffle_ps::<170>(x, x), p[2]));
            if i == 3 {
                v = _mm_add_ps(v, _mm_mul_ps(p[3], _mm_set1_ps(1.)));
            }
            _mm_storeu_ps(out[i].as_mut_ptr(), v);
        }
        out
    }
}
struct Guard(u32);
impl Drop for Guard {
    fn drop(&mut self) {
        unsafe {
            asm!("ldmxcsr [{p}]", p=in(reg)&self.0, options(nostack, preserves_flags));
        }
    }
}
fn run(asset: &[u8], capture: &[u8]) -> Result<Vec<u32>, &'static str> {
    if hash(asset) != "aed300c20813388a06b3dfa86644e53d573bb18e34167ba9b385563314037cb2" {
        return Err("unexpected RW4");
    }
    if capture.len() != 800
        || hash(capture) != "acc71f6a5f8713c914e725a012249fd02eba57ec87cd0605bc537a8b70cbdae6"
    {
        return Err("unexpected parent capture");
    }
    let source = asset.get(0x28f70..0x28f70 + 448).ok_or("missing basis")?;
    if hash(source) != "662b90e2ca27a297e7d7e9e2d5f47f27aae70c91d0dc2c2d5f3c4c7c581480f2" {
        return Err("unexpected basis");
    }
    if !std::is_x86_feature_detected!("sse2") {
        return Err("SSE2 required");
    }
    let mut prior = 0u32;
    let requested = 0x1f80u32;
    unsafe {
        asm!("stmxcsr [{old}]", "ldmxcsr [{new}]", old=in(reg)&mut prior, new=in(reg)&requested, options(nostack, preserves_flags));
    }
    let _guard = Guard(prior);
    let identity = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let mut predicted = Vec::with_capacity(84);
    for i in 0..7 {
        let parent = std::array::from_fn(|r| {
            std::array::from_fn(|c| word(capture, 336 + i * 64 + r * 16 + c * 4))
        });
        let control = compose(identity, parent);
        let basis =
            std::array::from_fn(|r| std::array::from_fn(|c| word(source, i * 64 + r * 16 + c * 4)));
        let candidate = compose(basis, parent);
        for r in 0..3 {
            for c in 0..4 {
                if control[c][r].to_bits() != word(capture, i * 48 + r * 16 + c * 4).to_bits() {
                    return Err("identity control mismatch");
                }
                predicted.push(candidate[c][r].to_bits());
            }
        }
    }
    Ok(predicted)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = env::args().collect();
    if a.len() != 3 {
        return Err(
            "usage: e219_source_basis_projection_v1 <external-1.rw4> <parent-capture.bin>".into(),
        );
    }
    let result = run(&fs::read(&a[1])?, &fs::read(&a[2])?)?;
    println!("Conditional source-basis prediction; 84 identity-control words matched; source-basis result has no original evaluator comparison");
    for (i, row) in result.chunks_exact(12).enumerate() {
        println!(
            "{i} {}",
            row.iter()
                .map(|v| format!("{v:08x}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unpinned() {
        assert!(run(&[], &[]).is_err());
    }
}
