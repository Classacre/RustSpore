//! Fixed seven-input 797970 comparison against a retained original execution.
//!
//! This example admits only the exact pinned 32-byte channel rows and source
//! records. It uses synthetic identity basis/no-op callback, fixed parent flags
//! and MXCSR 1F80. It does not sample animation, resolve live identity or model
//! general inputs or packed exception status. See re/e219-evaluator-handoff-static-v1.md.
#![allow(unused_assignments)]
use sha2::{Digest, Sha256};
use std::{arch::asm, fs};
#[inline(never)]
fn mul(mut a: f32, b: f32) -> f32 {
    unsafe {
        asm!("mulss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nomem,nostack,preserves_flags));
    }
    a
}
#[inline(never)]
fn add(mut a: f32, b: f32) -> f32 {
    unsafe {
        asm!("addss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nomem,nostack,preserves_flags));
    }
    a
}
#[inline(never)]
fn sub(mut a: f32, b: f32) -> f32 {
    unsafe {
        asm!("subss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nomem,nostack,preserves_flags));
    }
    a
}
fn compose(s: [f32; 4], r: [f32; 4]) -> [f32; 4] {
    // 797BC5..7C13: (r.z*s.y-r.y*s.z)+(r.w*s.x+s.w*r.x)
    // and cyclic permutations, each operation rounded by scalar SSE.
    let x = add(
        sub(mul(r[2], s[1]), mul(r[1], s[2])),
        add(mul(r[3], s[0]), mul(s[3], r[0])),
    );
    let y = add(
        sub(mul(r[0], s[2]), mul(r[2], s[0])),
        add(mul(r[3], s[1]), mul(s[3], r[1])),
    );
    let z = add(
        sub(mul(r[1], s[0]), mul(r[0], s[1])),
        add(mul(r[3], s[2]), mul(s[3], r[2])),
    );
    // Source horizontal ordering was checked against the fixed intermediate capture.
    let xyz = add(add(mul(r[2], s[2]), mul(r[1], s[1])), mul(r[0], s[0]));
    let w = sub(mul(r[3], s[3]), xyz);
    [x, y, z, w]
}

#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn rotation(q: [f32; 4]) -> [[f32; 4]; 3] {
    unsafe {
        use std::arch::x86_64::*;
        let v = _mm_loadu_ps(q.as_ptr());
        let sqrt = _mm_set1_ps(f32::from_bits(0x3fb504f3));
        let half = _mm_set1_ps(0.5);
        let zero = _mm_setzero_ps();
        let mut x = _mm_mul_ps(sqrt, v);
        let mut product = _mm_shuffle_ps::<9>(x, x);
        product = _mm_mul_ps(product, x);
        let mut w = _mm_shuffle_ps::<255>(x, x);
        let mut diagonal = _mm_sub_ps(zero, x);
        diagonal = _mm_mul_ps(diagonal, x);
        diagonal = _mm_add_ps(diagonal, half);
        x = _mm_shuffle_ps::<18>(x, x);
        w = _mm_mul_ps(w, x);
        let mut plus = _mm_add_ps(w, product);
        let diff = _mm_sub_ps(product, w);
        let diag = _mm_add_ps(_mm_shuffle_ps::<9>(diagonal, diagonal), diagonal);
        let row0 = _mm_shuffle_ps::<40>(_mm_shuffle_ps::<1>(diag, plus), diff);
        let row1 = _mm_shuffle_ps::<24>(_mm_shuffle_ps::<32>(diff, diag), plus);
        plus = _mm_shuffle_ps::<8>(_mm_shuffle_ps::<18>(plus, diff), diag);
        let mut out = [[0f32; 4]; 3];
        _mm_storeu_ps(out[0].as_mut_ptr(), row0);
        _mm_storeu_ps(out[1].as_mut_ptr(), row1);
        _mm_storeu_ps(out[2].as_mut_ptr(), plus);
        out
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn prematrix(q: [f32; 4]) -> [[f32; 4]; 4] {
    unsafe {
        use std::arch::x86_64::*;
        let a = rotation(q);
        let rot0 = _mm_loadu_ps(a[0].as_ptr());
        let rot1 = _mm_loadu_ps(a[1].as_ptr());
        let rot2 = _mm_loadu_ps(a[2].as_ptr());
        // Under the admitted unit-scale source records, source's 797D52..797DB3
        // shuffles yield these three scale vectors, plus a zero fourth vector.
        let v0 = _mm_set_ps(0., 0., 0., 1.);
        let v1 = _mm_set_ps(0., 0., 1., 0.);
        let v2 = _mm_set_ps(0., 1., 0., 0.);
        let v3 = _mm_setzero_ps();
        fn row(v: __m128, a: __m128, b: __m128, c: __m128) -> __m128 {
            unsafe {
                let x = _mm_mul_ps(_mm_shuffle_ps::<0>(v, v), a);
                let y = _mm_mul_ps(_mm_shuffle_ps::<85>(v, v), b);
                let z = _mm_mul_ps(_mm_shuffle_ps::<170>(v, v), c);
                _mm_add_ps(_mm_add_ps(x, y), z)
            }
        }
        let x = row(v0, rot0, rot1, rot2);
        let y = row(v1, rot0, rot1, rot2);
        let z = row(v2, rot0, rot1, rot2);
        let w = _mm_add_ps(row(v3, rot0, rot1, rot2), _mm_mul_ps(v3, _mm_set1_ps(1.)));
        let mut out = [[0f32; 4]; 4];
        for (r, v) in out.iter_mut().zip([x, y, z, w]) {
            _mm_storeu_ps(r.as_mut_ptr(), v);
        }
        out
    }
}
fn f32word(b: &[u8], o: usize) -> f32 {
    f32::from_bits(u32::from_le_bytes(b[o..o + 4].try_into().unwrap()))
}
struct Guard(u32);
impl Drop for Guard {
    fn drop(&mut self) {
        unsafe {
            asm!("ldmxcsr [{p}]",p=in(reg)&self.0,options(nostack,preserves_flags));
        }
    }
}

#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn postrecord(
    sample: [f32; 8],
    rec: &[f32; 40],
    pre: [[f32; 4]; 4],
    enabled: bool,
) -> [[f32; 4]; 4] {
    unsafe {
        use std::arch::x86_64::*;
        let mut stack = [0f32; 0x600 / 4];
        stack[0x20 / 4] = 1.;
        stack[0x24 / 4] = 1.;
        stack[0x28 / 4] = 1.;
        for i in 0..3 {
            stack[(0x54 / 4) + i] = add(rec[i], sample[4 + i]);
        }
        stack[0x1a0 / 4..0x1a0 / 4 + 4].copy_from_slice(&pre[1]);
        stack[0x1b0 / 4..0x1b0 / 4 + 4].copy_from_slice(&pre[2]);
        let mut x0 = _mm_setzero_ps();
        let x1 = _mm_set1_ps(1.);
        let mut x2 = _mm_loadu_ps(pre[3].as_ptr());
        let mut x3 = _mm_loadu_ps(pre[0].as_ptr());
        let mut x4 = _mm_setzero_ps();
        let mut x5 = _mm_setzero_ps();
        let mut x6 = _mm_setzero_ps();
        let mut x7 = _mm_setzero_ps();
        if enabled {
            x4 = _mm_load_ss(rec.as_ptr().add(36));
            x6 = _mm_load_ss(rec.as_ptr().add(38));
            x7 = _mm_load_ss(rec.as_ptr().add(37));
            x5 = x4;
            x4 = _mm_unpacklo_ps(x4, x6);
            x6 = x0;
            x7 = _mm_unpacklo_ps(x7, x5);
            x4 = _mm_unpacklo_ps(x4, x7);
            x6 = _mm_shuffle_ps::<175>(x6, x4);
            x5 = x0;
            x5 = _mm_shuffle_ps::<5>(x5, x4);
            x5 = _mm_shuffle_ps::<226>(x5, x0);
            x7 = x5;
            _mm_storeu_ps(stack.as_mut_ptr().add(16), x7);
            x5 = x0;
            x5 = _mm_shuffle_ps::<80>(x5, x4);
            x4 = x3;
            x4 = _mm_shuffle_ps::<85>(x4, x3);
            x5 = _mm_shuffle_ps::<232>(x5, x0);
            _mm_storeu_ps(stack.as_mut_ptr().add(12), x5);
            x4 = _mm_mul_ps(x4, _mm_loadu_ps(stack.as_ptr().add(12)));
            x5 = x0;
            x5 = _mm_shuffle_ps::<36>(x5, x6);
            x6 = x3;
            x6 = _mm_shuffle_ps::<0>(x6, x3);
            x6 = _mm_mul_ps(x6, x7);
            x4 = _mm_add_ps(x4, x6);
            x6 = x3;
            x6 = _mm_shuffle_ps::<170>(x6, x3);
            x6 = _mm_mul_ps(x6, x5);
            x4 = _mm_add_ps(x4, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(104));
            x3 = x6;
            x3 = _mm_shuffle_ps::<85>(x3, x6);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(16)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(104));
            x3 = _mm_add_ps(x3, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(108));
            x7 = _mm_mul_ps(x7, x5);
            x3 = _mm_add_ps(x3, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(196), x3);
            x3 = x6;
            x3 = _mm_shuffle_ps::<85>(x3, x6);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(16)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(108));
            x3 = _mm_add_ps(x3, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x7 = _mm_mul_ps(x7, x5);
            x3 = _mm_add_ps(x3, x7);
            x7 = _mm_load_ss(stack.as_ptr().add(22));
            _mm_storeu_ps(stack.as_mut_ptr().add(200), x3);
            x3 = x2;
            x3 = _mm_shuffle_ps::<85>(x3, x2);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(12)));
            x6 = x2;
            x6 = _mm_shuffle_ps::<0>(x6, x2);
            x6 = _mm_mul_ps(x6, _mm_loadu_ps(stack.as_ptr().add(16)));
            x3 = _mm_add_ps(x3, x6);
            x6 = x2;
            x6 = _mm_shuffle_ps::<170>(x6, x2);
            x6 = _mm_mul_ps(x6, x5);
            x3 = _mm_add_ps(x3, x6);
            x6 = _mm_load_ss(stack.as_ptr().add(23));
            x2 = x0;
            x2 = _mm_mul_ps(x2, x1);
            x3 = _mm_add_ps(x3, x2);
            x2 = _mm_load_ss(stack.as_ptr().add(21));
            x5 = x2;
            x2 = _mm_unpacklo_ps(x2, x6);
            x7 = _mm_unpacklo_ps(x7, x5);
            x2 = _mm_unpacklo_ps(x2, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(88), x2);
            x2 = x0;
            x2 = _mm_shuffle_ps::<5>(x2, x1);
            x2 = _mm_shuffle_ps::<226>(x2, x0);
            x6 = x2;
            _mm_storeu_ps(stack.as_mut_ptr().add(4), x6);
            x2 = x0;
            x2 = _mm_shuffle_ps::<80>(x2, x1);
            x2 = _mm_shuffle_ps::<232>(x2, x0);
            x7 = x2;
            _mm_storeu_ps(stack.as_mut_ptr().add(12), x7);
            x5 = x0;
            x5 = _mm_shuffle_ps::<175>(x5, x1);
            x2 = x0;
            x2 = _mm_shuffle_ps::<36>(x2, x5);
            x5 = x4;
            x5 = _mm_shuffle_ps::<85>(x5, x4);
            x5 = _mm_mul_ps(x5, x7);
            x7 = x4;
            x7 = _mm_shuffle_ps::<0>(x7, x4);
            x7 = _mm_mul_ps(x7, x6);
            x5 = _mm_add_ps(x5, x7);
            x6 = x4;
            x6 = _mm_shuffle_ps::<170>(x6, x4);
            x6 = _mm_mul_ps(x6, x2);
            x5 = _mm_add_ps(x5, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(196));
            x4 = x6;
            x4 = _mm_shuffle_ps::<85>(x4, x6);
            x4 = _mm_mul_ps(x4, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(4)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(196));
            x4 = _mm_add_ps(x4, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(200));
            x6 = _mm_shuffle_ps::<85>(x6, x6);
            x6 = _mm_mul_ps(x6, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = _mm_mul_ps(x7, x2);
            _mm_storeu_ps(stack.as_mut_ptr().add(16), x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(200));
            x4 = _mm_add_ps(x4, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(4)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(16));
            x6 = _mm_add_ps(x6, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(16), x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(200));
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            _mm_storeu_ps(stack.as_mut_ptr().add(124), x2);
            x7 = _mm_mul_ps(x7, x2);
            x2 = _mm_loadu_ps(stack.as_ptr().add(16));
            x2 = _mm_add_ps(x2, x7);
            x7 = _mm_loadu_ps(rec.as_ptr().add(24));
            x6 = x3;
            x6 = _mm_shuffle_ps::<0>(x6, x3);
            x6 = _mm_mul_ps(x6, _mm_loadu_ps(stack.as_ptr().add(4)));
            _mm_storeu_ps(stack.as_mut_ptr().add(328), x2);
            x2 = x3;
            x2 = _mm_shuffle_ps::<85>(x2, x3);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(12)));
            x2 = _mm_add_ps(x2, x6);
            x6 = x3;
            x6 = _mm_shuffle_ps::<170>(x6, x3);
            x6 = _mm_mul_ps(x6, _mm_loadu_ps(stack.as_ptr().add(124)));
            x3 = _mm_loadu_ps(stack.as_ptr().add(88));
            x3 = _mm_mul_ps(x3, x1);
            x2 = _mm_add_ps(x2, x6);
            x2 = _mm_add_ps(x2, x3);
            x3 = _mm_loadu_ps(rec.as_ptr().add(20));
            x6 = x5;
            x6 = _mm_shuffle_ps::<0>(x6, x5);
            x6 = _mm_mul_ps(x6, x3);
            x3 = x5;
            x3 = _mm_shuffle_ps::<85>(x3, x5);
            x3 = _mm_mul_ps(x3, x7);
            x6 = _mm_add_ps(x6, x3);
            x3 = x5;
            x3 = _mm_shuffle_ps::<170>(x3, x5);
            _mm_storeu_ps(stack.as_mut_ptr().add(88), x3);
            x3 = _mm_loadu_ps(rec.as_ptr().add(28));
            x5 = _mm_loadu_ps(stack.as_ptr().add(88));
            x5 = _mm_mul_ps(x5, x3);
            x6 = _mm_add_ps(x6, x5);
            x5 = _mm_loadu_ps(rec.as_ptr().add(20));
            x3 = x4;
            x3 = _mm_shuffle_ps::<0>(x3, x4);
            x3 = _mm_mul_ps(x3, x5);
            x5 = x4;
            x5 = _mm_shuffle_ps::<85>(x5, x4);
            x5 = _mm_mul_ps(x5, x7);
            x3 = _mm_add_ps(x3, x5);
            x5 = x4;
            x5 = _mm_shuffle_ps::<170>(x5, x4);
            x4 = _mm_loadu_ps(rec.as_ptr().add(28));
            x5 = _mm_mul_ps(x5, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(328));
            x3 = _mm_add_ps(x3, x5);
            x5 = _mm_loadu_ps(rec.as_ptr().add(20));
            _mm_storeu_ps(stack.as_mut_ptr().add(228), x3);
            x3 = x4;
            x3 = _mm_shuffle_ps::<0>(x3, x4);
            x3 = _mm_mul_ps(x3, x5);
            x5 = x4;
            x5 = _mm_shuffle_ps::<85>(x5, x4);
            x5 = _mm_mul_ps(x5, x7);
            x3 = _mm_add_ps(x3, x5);
            x5 = x4;
            x5 = _mm_shuffle_ps::<170>(x5, x4);
            x4 = _mm_loadu_ps(rec.as_ptr().add(28));
            x5 = _mm_mul_ps(x5, x4);
            x3 = _mm_add_ps(x3, x5);
            x5 = _mm_loadu_ps(rec.as_ptr().add(20));
            _mm_storeu_ps(stack.as_mut_ptr().add(232), x3);
            x3 = x2;
            x3 = _mm_shuffle_ps::<0>(x3, x2);
            x3 = _mm_mul_ps(x3, x5);
            x5 = x2;
            x5 = _mm_shuffle_ps::<85>(x5, x2);
            x5 = _mm_mul_ps(x5, x7);
            x7 = _mm_load_ss(stack.as_ptr().add(9));
            x3 = _mm_add_ps(x3, x5);
            x5 = x2;
            x5 = _mm_shuffle_ps::<170>(x5, x2);
            x2 = _mm_loadu_ps(rec.as_ptr().add(32));
            x5 = _mm_mul_ps(x5, x4);
            x2 = _mm_mul_ps(x2, x1);
            x3 = _mm_add_ps(x3, x5);
            x5 = _mm_load_ss(stack.as_ptr().add(10));
            x3 = _mm_add_ps(x3, x2);
            x2 = _mm_load_ss(stack.as_ptr().add(8));
            x4 = x2;
            x2 = _mm_unpacklo_ps(x2, x5);
            x7 = _mm_unpacklo_ps(x7, x4);
            x2 = _mm_unpacklo_ps(x2, x7);
            x4 = x0;
            x4 = _mm_shuffle_ps::<5>(x4, x2);
            x4 = _mm_shuffle_ps::<226>(x4, x0);
            x7 = x4;
            x4 = x0;
            x4 = _mm_shuffle_ps::<80>(x4, x2);
            x4 = _mm_shuffle_ps::<232>(x4, x0);
            _mm_storeu_ps(stack.as_mut_ptr().add(12), x4);
            x4 = x0;
            x4 = _mm_shuffle_ps::<175>(x4, x2);
            x5 = x0;
            x5 = _mm_shuffle_ps::<36>(x5, x4);
            x4 = x6;
            x4 = _mm_shuffle_ps::<85>(x4, x6);
            x4 = _mm_mul_ps(x4, _mm_loadu_ps(stack.as_ptr().add(12)));
            x2 = x6;
            x2 = _mm_shuffle_ps::<0>(x2, x6);
            x2 = _mm_mul_ps(x2, x7);
            x4 = _mm_add_ps(x4, x2);
            x2 = x6;
            x2 = _mm_shuffle_ps::<170>(x2, x6);
            x2 = _mm_mul_ps(x2, x5);
            _mm_storeu_ps(stack.as_mut_ptr().add(4), x7);
            x4 = _mm_add_ps(x4, x2);
            x6 = _mm_loadu_ps(stack.as_ptr().add(228));
            x2 = x6;
            x2 = _mm_shuffle_ps::<85>(x2, x6);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(4)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(228));
            x2 = _mm_add_ps(x2, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(232));
            x7 = _mm_mul_ps(x7, x5);
            x2 = _mm_add_ps(x2, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(4)));
            _mm_storeu_ps(stack.as_mut_ptr().add(212), x2);
            x2 = x6;
            x2 = _mm_shuffle_ps::<85>(x2, x6);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(12)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(232));
            x2 = _mm_add_ps(x2, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x7 = _mm_mul_ps(x7, x5);
            x2 = _mm_add_ps(x2, x7);
            x7 = _mm_load_ss(rec.as_ptr().add(16));
            x6 = x3;
            x6 = _mm_shuffle_ps::<0>(x6, x3);
            x6 = _mm_mul_ps(x6, _mm_loadu_ps(stack.as_ptr().add(4)));
            _mm_storeu_ps(stack.as_mut_ptr().add(216), x2);
            x2 = x3;
            x2 = _mm_shuffle_ps::<85>(x2, x3);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(12)));
            x2 = _mm_add_ps(x2, x6);
            x6 = x3;
            x6 = _mm_shuffle_ps::<170>(x6, x3);
            x6 = _mm_mul_ps(x6, x5);
            x5 = _mm_load_ss(rec.as_ptr().add(15));
            x2 = _mm_add_ps(x2, x6);
            x6 = _mm_load_ss(rec.as_ptr().add(17));
            x3 = x0;
            x3 = _mm_mul_ps(x3, x1);
            x2 = _mm_add_ps(x2, x3);
            x3 = x5;
            x5 = _mm_unpacklo_ps(x5, x6);
            x6 = x0;
            x6 = _mm_shuffle_ps::<80>(x6, x1);
            x6 = _mm_shuffle_ps::<232>(x6, x0);
            x7 = _mm_unpacklo_ps(x7, x3);
            x5 = _mm_unpacklo_ps(x5, x7);
            x7 = x6;
            x3 = x0;
            x3 = _mm_shuffle_ps::<5>(x3, x1);
            x3 = _mm_shuffle_ps::<226>(x3, x0);
            _mm_storeu_ps(stack.as_mut_ptr().add(4), x7);
            x6 = x0;
            x6 = _mm_shuffle_ps::<175>(x6, x1);
            x0 = _mm_shuffle_ps::<36>(x0, x6);
            _mm_storeu_ps(stack.as_mut_ptr().add(16), x3);
            x6 = x4;
            x6 = _mm_shuffle_ps::<85>(x6, x4);
            x6 = _mm_mul_ps(x6, x7);
            x7 = x4;
            x7 = _mm_shuffle_ps::<0>(x7, x4);
            x7 = _mm_mul_ps(x7, x3);
            x3 = x4;
            x3 = _mm_shuffle_ps::<170>(x3, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(212));
            x3 = _mm_mul_ps(x3, x0);
            x6 = _mm_add_ps(x6, x7);
            x6 = _mm_add_ps(x6, x3);
            x3 = x4;
            x3 = _mm_shuffle_ps::<85>(x3, x4);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(4)));
            x7 = x4;
            x7 = _mm_shuffle_ps::<0>(x7, x4);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(16)));
            x4 = _mm_loadu_ps(stack.as_ptr().add(212));
            x3 = _mm_add_ps(x3, x7);
            x7 = x4;
            x7 = _mm_shuffle_ps::<170>(x7, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(216));
            x7 = _mm_mul_ps(x7, x0);
            x3 = _mm_add_ps(x3, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(276), x3);
            x3 = x4;
            x3 = _mm_shuffle_ps::<85>(x3, x4);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(4)));
            x7 = x4;
            x7 = _mm_shuffle_ps::<0>(x7, x4);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(16)));
            x4 = _mm_loadu_ps(stack.as_ptr().add(216));
            x3 = _mm_add_ps(x3, x7);
            x7 = x4;
            x7 = _mm_shuffle_ps::<170>(x7, x4);
            x7 = _mm_mul_ps(x7, x0);
            x3 = _mm_add_ps(x3, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(280), x3);
            x3 = x2;
            x3 = _mm_shuffle_ps::<85>(x3, x2);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(4)));
            x4 = x2;
            x4 = _mm_shuffle_ps::<0>(x4, x2);
            x4 = _mm_mul_ps(x4, _mm_loadu_ps(stack.as_ptr().add(16)));
            x3 = _mm_add_ps(x3, x4);
            x4 = x2;
            x4 = _mm_shuffle_ps::<170>(x4, x2);
            x4 = _mm_mul_ps(x4, x0);
            x0 = _mm_loadu_ps(stack.as_ptr().add(276));
            x3 = _mm_add_ps(x3, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(280));
            x5 = _mm_mul_ps(x5, x1);
            x3 = _mm_add_ps(x3, x5);
        } else {
            x4 = _mm_load_ss(stack.as_ptr().add(8));
            x6 = _mm_load_ss(stack.as_ptr().add(10));
            x7 = _mm_load_ss(stack.as_ptr().add(9));
            x5 = x4;
            x4 = _mm_unpacklo_ps(x4, x6);
            x6 = x0;
            x7 = _mm_unpacklo_ps(x7, x5);
            x4 = _mm_unpacklo_ps(x4, x7);
            x6 = _mm_shuffle_ps::<175>(x6, x4);
            x5 = x0;
            x5 = _mm_shuffle_ps::<5>(x5, x4);
            x5 = _mm_shuffle_ps::<226>(x5, x0);
            x7 = x5;
            x5 = x0;
            x5 = _mm_shuffle_ps::<80>(x5, x4);
            x4 = x3;
            x4 = _mm_shuffle_ps::<85>(x4, x3);
            x5 = _mm_shuffle_ps::<232>(x5, x0);
            _mm_storeu_ps(stack.as_mut_ptr().add(12), x5);
            x4 = _mm_mul_ps(x4, _mm_loadu_ps(stack.as_ptr().add(12)));
            _mm_storeu_ps(stack.as_mut_ptr().add(4), x7);
            x5 = x0;
            x5 = _mm_shuffle_ps::<36>(x5, x6);
            x6 = x3;
            x6 = _mm_shuffle_ps::<0>(x6, x3);
            x6 = _mm_mul_ps(x6, x7);
            x4 = _mm_add_ps(x4, x6);
            x6 = x3;
            x6 = _mm_shuffle_ps::<170>(x6, x3);
            x6 = _mm_mul_ps(x6, x5);
            x4 = _mm_add_ps(x4, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(104));
            x3 = x6;
            x3 = _mm_shuffle_ps::<85>(x3, x6);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(4)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(104));
            x3 = _mm_add_ps(x3, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x6 = _mm_loadu_ps(stack.as_ptr().add(108));
            x7 = _mm_mul_ps(x7, x5);
            x3 = _mm_add_ps(x3, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(244), x3);
            x3 = x6;
            x3 = _mm_shuffle_ps::<85>(x3, x6);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(12)));
            x7 = x6;
            x7 = _mm_shuffle_ps::<0>(x7, x6);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(4)));
            x6 = _mm_loadu_ps(stack.as_ptr().add(108));
            x3 = _mm_add_ps(x3, x7);
            x7 = x6;
            x7 = _mm_shuffle_ps::<170>(x7, x6);
            x7 = _mm_mul_ps(x7, x5);
            x3 = _mm_add_ps(x3, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(248), x3);
            x3 = x2;
            x3 = _mm_shuffle_ps::<85>(x3, x2);
            x3 = _mm_mul_ps(x3, _mm_loadu_ps(stack.as_ptr().add(12)));
            x6 = x2;
            x6 = _mm_shuffle_ps::<0>(x6, x2);
            x6 = _mm_mul_ps(x6, _mm_loadu_ps(stack.as_ptr().add(4)));
            x3 = _mm_add_ps(x3, x6);
            x6 = x2;
            x6 = _mm_shuffle_ps::<170>(x6, x2);
            x6 = _mm_mul_ps(x6, x5);
            x5 = _mm_load_ss(rec.as_ptr().add(11));
            x5 = _mm_mul_ss(x5, _mm_loadu_ps(stack.as_ptr().add(21)));
            x3 = _mm_add_ps(x3, x6);
            x6 = _mm_load_ss(rec.as_ptr().add(13));
            x6 = _mm_mul_ss(x6, _mm_loadu_ps(stack.as_ptr().add(23)));
            x2 = x0;
            x2 = _mm_mul_ps(x2, x1);
            x3 = _mm_add_ps(x3, x2);
            x2 = _mm_load_ss(rec.as_ptr().add(12));
            x2 = _mm_mul_ss(x2, _mm_loadu_ps(stack.as_ptr().add(22)));
            x7 = x5;
            x5 = _mm_unpacklo_ps(x5, x6);
            x2 = _mm_unpacklo_ps(x2, x7);
            x5 = _mm_unpacklo_ps(x5, x2);
            x2 = x0;
            x2 = _mm_shuffle_ps::<5>(x2, x1);
            x2 = _mm_shuffle_ps::<226>(x2, x0);
            x7 = x2;
            x2 = x0;
            x2 = _mm_shuffle_ps::<80>(x2, x1);
            x2 = _mm_shuffle_ps::<232>(x2, x0);
            _mm_storeu_ps(stack.as_mut_ptr().add(4), x2);
            x6 = x0;
            x6 = _mm_shuffle_ps::<175>(x6, x1);
            x0 = _mm_shuffle_ps::<36>(x0, x6);
            x6 = x4;
            x6 = _mm_shuffle_ps::<85>(x6, x4);
            x6 = _mm_mul_ps(x6, x2);
            x2 = x4;
            x2 = _mm_shuffle_ps::<0>(x2, x4);
            x2 = _mm_mul_ps(x2, x7);
            x6 = _mm_add_ps(x6, x2);
            x2 = x4;
            x2 = _mm_shuffle_ps::<170>(x2, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(244));
            x2 = _mm_mul_ps(x2, x0);
            x6 = _mm_add_ps(x6, x2);
            _mm_storeu_ps(stack.as_mut_ptr().add(16), x7);
            x2 = x4;
            x2 = _mm_shuffle_ps::<85>(x2, x4);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(4)));
            x7 = x4;
            x7 = _mm_shuffle_ps::<0>(x7, x4);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(16)));
            x4 = _mm_loadu_ps(stack.as_ptr().add(244));
            x2 = _mm_add_ps(x2, x7);
            x7 = x4;
            x7 = _mm_shuffle_ps::<170>(x7, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(248));
            x7 = _mm_mul_ps(x7, x0);
            x2 = _mm_add_ps(x2, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(260), x2);
            x2 = x4;
            x2 = _mm_shuffle_ps::<85>(x2, x4);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(4)));
            x7 = x4;
            x7 = _mm_shuffle_ps::<0>(x7, x4);
            x7 = _mm_mul_ps(x7, _mm_loadu_ps(stack.as_ptr().add(16)));
            x4 = _mm_loadu_ps(stack.as_ptr().add(248));
            x2 = _mm_add_ps(x2, x7);
            x7 = x4;
            x7 = _mm_shuffle_ps::<170>(x7, x4);
            x7 = _mm_mul_ps(x7, x0);
            x2 = _mm_add_ps(x2, x7);
            _mm_storeu_ps(stack.as_mut_ptr().add(264), x2);
            x2 = x3;
            x2 = _mm_shuffle_ps::<85>(x2, x3);
            x2 = _mm_mul_ps(x2, _mm_loadu_ps(stack.as_ptr().add(4)));
            x4 = x3;
            x4 = _mm_shuffle_ps::<0>(x4, x3);
            x4 = _mm_mul_ps(x4, _mm_loadu_ps(stack.as_ptr().add(16)));
            x2 = _mm_add_ps(x2, x4);
            x4 = x3;
            x4 = _mm_shuffle_ps::<170>(x4, x3);
            x4 = _mm_mul_ps(x4, x0);
            x0 = _mm_loadu_ps(stack.as_ptr().add(260));
            x2 = _mm_add_ps(x2, x4);
            x4 = _mm_loadu_ps(stack.as_ptr().add(264));
            x5 = _mm_mul_ps(x5, x1);
            x2 = _mm_add_ps(x2, x5);
            x3 = x2;
        }
        let mut out = [[0f32; 4]; 4];
        for (r, v) in out.iter_mut().zip([x6, x0, x4, x3]) {
            _mm_storeu_ps(r.as_mut_ptr(), v);
        }
        out
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn parent_compose(local: [[f32; 4]; 4], parent: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    unsafe {
        use std::arch::x86_64::*;
        let p = [
            _mm_loadu_ps(parent[0].as_ptr()),
            _mm_loadu_ps(parent[1].as_ptr()),
            _mm_loadu_ps(parent[2].as_ptr()),
            _mm_loadu_ps(parent[3].as_ptr()),
        ];
        let mut out = [[0f32; 4]; 4];
        for (i, row) in local.iter().enumerate() {
            let x = _mm_loadu_ps(row.as_ptr());
            let yprod = _mm_mul_ps(_mm_shuffle_ps::<85>(x, x), p[1]);
            let xprod = _mm_mul_ps(_mm_shuffle_ps::<0>(x, x), p[0]);
            let mut result = _mm_add_ps(yprod, xprod);
            let zprod = _mm_mul_ps(_mm_shuffle_ps::<170>(x, x), p[2]);
            result = _mm_add_ps(result, zprod);
            if i == 3 {
                result = _mm_add_ps(result, _mm_mul_ps(p[3], _mm_set1_ps(1.)));
            }
            _mm_storeu_ps(out[i].as_mut_ptr(), result);
        }
        out
    }
}

fn digest(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn admitted(samples: &[u8], records: &[u8]) -> bool {
    samples.len() == 224
        && records.len() == 1120
        && digest(samples) == "eb503e42d394adc8c68237dc8cf275b4fb4ac3a0ece1dfba3755cda710562541"
        && digest(records) == "2a22078124ea9197be988c65d2067803ce389459b4ee9bba96c920958ceadcea"
}
fn main() {
    assert!(std::is_x86_feature_detected!("sse2"));
    let mut a = std::env::args().skip(1);
    let samples = fs::read(a.next().expect("sample rows")).unwrap();
    let records = fs::read(a.next().expect("source records")).unwrap();
    let native = fs::read(a.next().expect("retained native output")).unwrap();
    assert!(a.next().is_none());
    assert!(
        admitted(&samples, &records),
        "input is outside the fixed fixture"
    );
    assert_eq!(native.len(), 352);
    assert_eq!(
        digest(&native),
        "fdba0758b851ac2e5d9e283fc7e73dd75ac6ef9fb3314904b3e755b39ac7406d"
    );
    let mut previous = 0u32;
    let requested = 0x1f80u32;
    unsafe {
        asm!("stmxcsr [{old}]","ldmxcsr [{new}]",old=in(reg)&mut previous,new=in(reg)&requested,options(nostack,preserves_flags));
    }
    let _guard = Guard(previous);
    let mut parent = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let mut saved: Vec<[[f32; 4]; 4]> = Vec::new();
    let flags = [0, 2, 1, 2, 1, 0, 1];
    for i in 0..7 {
        let sample = std::array::from_fn(|j| f32word(&samples, i * 32 + j * 4));
        let rec = std::array::from_fn(|j| f32word(&records, i * 160 + j * 4));
        let q = compose(
            std::array::from_fn(|j| sample[j]),
            std::array::from_fn(|j| rec[4 + j]),
        );
        let local = postrecord(sample, &rec, prematrix(q), records[i * 160 + 56] != 0);
        let rows = parent_compose(local, parent);
        let basis = [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ];
        let final_rows = parent_compose(basis, rows);
        for row in 0..3 {
            for col in 0..4 {
                let offset = i * 48 + (row * 4 + col) * 4;
                let expected = u32::from_le_bytes(native[offset..offset + 4].try_into().unwrap());
                assert_eq!(
                    final_rows[col][row].to_bits(),
                    expected,
                    "bone {i} row {row} column {col}"
                );
            }
        }
        match flags[i] {
            0 => parent = rows,
            2 => {
                saved.push(parent);
                parent = rows
            }
            1 => {
                if let Some(p) = saved.pop() {
                    parent = p
                } else {
                    assert_eq!(i, 6);
                }
            }
            _ => unreachable!(),
        }
    }
    println!("matched 84 fixed seven-bone words; synthetic basis/callback, no live sampling or packed-status claim");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unadmitted_inputs() {
        assert!(!admitted(&[], &[]));
        assert!(!admitted(&[0; 224], &[0; 1120]));
    }
}
