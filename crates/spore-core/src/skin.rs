//! Generating a diffuse atlas for a creature that ships none.
//!
//! ## What this can and cannot do
//!
//! 656 of the 1,022 creatures in `Spore_Content` ship no skin raster. They do carry a
//! paint palette: nine floats at offset 32 of the decompressed `.creaturedata` header,
//! three RGB triples.
//!
//! `[MEASURED]` The palette is the creature's authored colour set, not a literal flat
//! palette for its shipped skin. Across 40 creatures that have both, all three palette
//! colours appear (within 0.08 RGB) somewhere in the shipped skin for 30 of them and at
//! least one for 37, but only a median 14 percent of the skin's opaque pixels lie that
//! close to a palette colour (p10 1 percent, p90 30 percent). The skins are painted with
//! shading and intermediate tones, so the palette gives the hues and not the picture.
//!
//! `[MEASURED]` The shipped skins are hand-painted atlases, not palette expansions. The
//! 512x512 skin of `406662008C9D0727` holds 455 distinct 5-bit colours and visible leaf,
//! stripe and scale motifs; the 299 skins at that size range from 169 to 2,312 colours.
//! A three-colour palette cannot reproduce that, and this module does not pretend to.
//! What it produces is a plausible base colouring: the creature's own colours laid over
//! the UV space its geometry actually uses, with a belly split and a marking pattern.
//!
//! `[MEASURED]` The atlas layout is not shared between creatures: pairwise alpha-mask
//! IoU of eight 512x512 shipped skins ranges from 0.369 to 0.773. There is no canonical
//! atlas to paint into, so the layout has to come from the mesh's own UVs, which is what
//! [`generate`] rasterises.
//!
//! `[MEASURED]` The rasteriser addresses the same space the artist did. On five
//! creatures that have both a shipped skin and a resolvable mesh, the shipped alpha mask
//! lies almost entirely inside the generated one: intersection over union equals the
//! shipped mask's own area fraction to two decimal places in every case (0.098 against
//! 0.092, 0.193 against 0.186, 0.268 against 0.269, 0.271 against 0.275, 0.348 against
//! 0.349). So the geometry's UVs, including the negative `v` range and its tile shift,
//! land on the artist's islands. The generated atlas covers more of the square than the
//! artist painted, which is the point: the rest of the body still needs a colour.
//!
//! ## Method
//!
//! 1. Rasterise every triangle into the unit square using its UVs, interpolating world
//!    position and normal at each texel.
//! 2. Colour each texel from the palette: the back colour where the surface faces up,
//!    the second colour where it faces down, with height breaking ties.
//! 3. Overlay markings in the third palette colour, on the upper 30 percent of a smooth
//!    wave field by rank, so the share of the body that is marked does not depend on the
//!    field's amplitude.
//! 4. Dilate the covered region by a few texels so bilinear sampling at island edges
//!    does not pull in transparent black.
//!
//! The output is a texture in the same form the shipped skins take, so a renderer does
//! not need to know which path produced it.

use crate::mesh::MeshData;
use crate::Vec3;

/// A decoded or generated diffuse map, ready for upload.
#[derive(Debug, Clone)]
pub struct SkinTexture {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA8, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
    /// Covered texels before dilation, for reporting.
    pub covered: usize,
    /// Texels where the marking weight reached at least half, so the third palette
    /// colour is more than a tint. Reported because "the generator paints markings" is
    /// a claim that has to be checkable from outside.
    pub marked: usize,
}

/// How much of the mesh's UV space the rasteriser actually reached.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvStats {
    /// Triangles whose three UVs are all finite.
    pub triangles_with_uv: usize,
    pub triangles_total: usize,
    /// Range of the UVs that were used.
    pub u_min: f32,
    pub u_max: f32,
    pub v_min: f32,
    pub v_max: f32,
    /// Triangles whose UVs cannot be placed inside one tile by shifting the whole
    /// triangle, which happens when a single triangle straddles the atlas seam. These
    /// are clipped rather than smeared across the whole texture.
    pub clipped: usize,
}

impl UvStats {
    /// Fraction of triangles that can be textured.
    pub fn usable_fraction(&self) -> f32 {
        if self.triangles_total == 0 {
            0.0
        } else {
            self.triangles_with_uv as f32 / self.triangles_total as f32
        }
    }
}

fn finite_uv(uv: [f32; 2]) -> bool {
    uv[0].is_finite() && uv[1].is_finite()
}

/// Measure the UV space a mesh uses, without rasterising anything.
pub fn uv_stats(mesh: &MeshData) -> UvStats {
    let triangles_total = mesh.triangle_count();
    let mut stats = UvStats {
        triangles_with_uv: 0,
        triangles_total,
        u_min: f32::INFINITY,
        u_max: f32::NEG_INFINITY,
        v_min: f32::INFINITY,
        v_max: f32::NEG_INFINITY,
        clipped: 0,
    };
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= mesh.uvs.len() || i1 >= mesh.uvs.len() || i2 >= mesh.uvs.len() {
            continue;
        }
        let uv = [mesh.uvs[i0], mesh.uvs[i1], mesh.uvs[i2]];
        if !uv.iter().all(|&p| finite_uv(p)) {
            continue;
        }
        // A degenerate UV triangle covers no area and would rasterise to nothing.
        let area = (uv[1][0] - uv[0][0]) * (uv[2][1] - uv[0][1])
            - (uv[2][0] - uv[0][0]) * (uv[1][1] - uv[0][1]);
        if area.abs() < 1e-12 {
            continue;
        }
        stats.triangles_with_uv += 1;
        for p in uv {
            stats.u_min = stats.u_min.min(p[0]);
            stats.u_max = stats.u_max.max(p[0]);
            stats.v_min = stats.v_min.min(p[1]);
            stats.v_max = stats.v_max.max(p[1]);
        }
        // The tile shift used by `generate`: the centroid picks the tile, and a triangle
        // that still falls outside it straddles a seam.
        let tu = ((uv[0][0] + uv[1][0] + uv[2][0]) / 3.0).floor();
        let tv = ((uv[0][1] + uv[1][1] + uv[2][1]) / 3.0).floor();
        if uv.iter().any(|p| {
            let (u, v) = (p[0] - tu, p[1] - tv);
            u < -0.001 || u > 1.001 || v < -0.001 || v > 1.001
        }) {
            stats.clipped += 1;
        }
    }
    if stats.triangles_with_uv == 0 {
        stats.u_min = 0.0;
        stats.u_max = 0.0;
        stats.v_min = 0.0;
        stats.v_max = 0.0;
    }
    stats
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// A smooth marking field in roughly `0.1..0.9`, from a sum of plane waves.
///
/// Waves rather than lattice noise. Trilinear value noise is only C0 continuous, so its
/// iso-contours have creases at the lattice planes, and on a 512 atlas those creases read
/// as straight-edged patches that do not belong on a body. A sum of sines has continuous
/// derivatives everywhere, and mixing incommensurate frequencies gives stripes and spots
/// together rather than blobs.
///
/// The field is deterministic and depends only on world position, so the same creature
/// always paints the same way and the pattern runs across part boundaries instead of
/// restarting at each UV island.
fn wave_field(p: [f32; 3], scale: f32) -> f32 {
    let (x, y, z) = (p[0] * scale, p[1] * scale, p[2] * scale);
    let a = (x * 2.1 + y * 1.3).sin();
    let b = (z * 1.9 - x * 0.7).sin();
    let c = (x * 4.3 + z * 3.7 + 1.1).sin();
    let d = (y * 3.1 - z * 2.3 + 0.7).cos();
    (0.5 + 0.18 * a * b + 0.14 * c + 0.10 * d * a).clamp(0.0, 1.0)
}

/// Linear blend from `a` at `t = 0` to `b` at `t = 1`.
///
/// Callers decide what `t` means; here it is the weight of the back colour, so a belly
/// texel arrives with a small `t` and a back texel with a large one.
fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Generate a diffuse atlas from a creature's paint palette and its own geometry.
///
/// Returns `None` when the mesh has no texturable triangles, which is the honest
/// answer: there is no UV space to paint into.
///
/// `size` is the texture edge in texels. 512 matches the shipped atlases.
pub fn generate(mesh: &MeshData, colours: &[[f32; 3]; 3], size: u32) -> Option<SkinTexture> {
    let size = size.clamp(16, 2048);
    let n = size as usize;
    let stats = uv_stats(mesh);
    if stats.triangles_with_uv == 0 {
        return None;
    }
    let (min, max) = mesh.bounds()?;
    let height = (max.y - min.y).max(1e-6);
    // The noise is sampled per world unit, so its scale has to follow the largest axis
    // rather than the vertical one: a flat mesh has no vertical extent, and dividing by
    // it would sample the hash at coordinates large enough to be meaningless.
    let extent = (max.x - min.x)
        .max(max.y - min.y)
        .max(max.z - min.z)
        .max(1e-3);

    // Per texel: the world position and normal of the surface that owns it, and how
    // many triangles contributed. Averaging over contributions softens the colour split
    // along island edges instead of producing hard steps.
    //
    // The pattern is computed from the mean world position after the pass, not from a
    // per-triangle value. An earlier version averaged a per-triangle hash, which over
    // 28,000 triangles converges to 0.5 everywhere and produced a flat colour: the
    // pattern has to be a function of the texel's position to survive the average.
    let mut pos_sum = vec![[0.0f32; 3]; n * n];
    let mut nrm_sum = vec![[0.0f32; 3]; n * n];
    let mut hits = vec![0u32; n * n];

    let mut covered = 0usize;
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= mesh.uvs.len() || i1 >= mesh.uvs.len() || i2 >= mesh.uvs.len() {
            continue;
        }
        let uv = [mesh.uvs[i0], mesh.uvs[i1], mesh.uvs[i2]];
        if !uv.iter().all(|&p| finite_uv(p)) {
            continue;
        }
        let pos = [mesh.positions[i0], mesh.positions[i1], mesh.positions[i2]];
        let nrm = if mesh.normals.len() > i2 {
            [mesh.normals[i0], mesh.normals[i1], mesh.normals[i2]]
        } else {
            [Vec3::new(0.0, 1.0, 0.0); 3]
        };
        // Shift the whole triangle into the unit square by an integer tile offset. A
        // creature's UVs run outside 0..1 (measured -3.5 to 0.98), and the sampler
        // repeats, so this is the tile the triangle actually appears in.
        let cu = (uv[0][0] + uv[1][0] + uv[2][0]) / 3.0;
        let cv = (uv[0][1] + uv[1][1] + uv[2][1]) / 3.0;
        let tu = cu.floor();
        let tv = cv.floor();
        let uv = [
            [uv[0][0] - tu, uv[0][1] - tv],
            [uv[1][0] - tu, uv[1][1] - tv],
            [uv[2][0] - tu, uv[2][1] - tv],
        ];
        let area = (uv[1][0] - uv[0][0]) * (uv[2][1] - uv[0][1])
            - (uv[2][0] - uv[0][0]) * (uv[1][1] - uv[0][1]);
        if area.abs() < 1e-12 {
            continue;
        }

        let x0 = (uv.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min) * n as f32)
            .floor()
            .max(0.0) as i32;
        let x1 = (uv.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max) * n as f32)
            .ceil()
            .min(n as f32 - 1.0) as i32;
        let y0 = (uv.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min) * n as f32)
            .floor()
            .max(0.0) as i32;
        let y1 = (uv.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max) * n as f32)
            .ceil()
            .min(n as f32 - 1.0) as i32;
        if x1 < x0 || y1 < y0 {
            continue;
        }

        for py in y0..=y1 {
            // Texel centres, so a triangle that exactly covers the square covers every
            // texel once rather than half of them.
            let v = (py as f32 + 0.5) / n as f32;
            for px in x0..=x1 {
                let u = (px as f32 + 0.5) / n as f32;
                let w0 = ((uv[1][0] - u) * (uv[2][1] - v) - (uv[2][0] - u) * (uv[1][1] - v)) / area;
                let w1 = ((uv[2][0] - u) * (uv[0][1] - v) - (uv[0][0] - u) * (uv[2][1] - v)) / area;
                let w2 = 1.0 - w0 - w1;
                // A small negative tolerance closes the cracks between adjacent
                // triangles that share an edge.
                if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                    continue;
                }
                // Interpolate the surface attributes across the triangle. Using the
                // triangle's centroid instead paints one flat patch per triangle, which
                // on a 512 texture with triangles this size reads as visible facets.
                let w = [w0.max(0.0), w1.max(0.0), w2.max(0.0)];
                let s = w[0] + w[1] + w[2];
                let w = if s > 1e-6 {
                    [w[0] / s, w[1] / s, w[2] / s]
                } else {
                    [1.0, 0.0, 0.0]
                };
                let i = py as usize * n + px as usize;
                if hits[i] == 0 {
                    covered += 1;
                }
                pos_sum[i][0] += w[0] * pos[0].x + w[1] * pos[1].x + w[2] * pos[2].x;
                pos_sum[i][1] += w[0] * pos[0].y + w[1] * pos[1].y + w[2] * pos[2].y;
                pos_sum[i][2] += w[0] * pos[0].z + w[1] * pos[1].z + w[2] * pos[2].z;
                nrm_sum[i][0] += w[0] * nrm[0].x + w[1] * nrm[1].x + w[2] * nrm[2].x;
                nrm_sum[i][1] += w[0] * nrm[0].y + w[1] * nrm[1].y + w[2] * nrm[2].y;
                nrm_sum[i][2] += w[0] * nrm[0].z + w[1] * nrm[1].z + w[2] * nrm[2].z;
                hits[i] += 1;
            }
        }
    }

    if covered == 0 {
        return None;
    }

    // Colour the covered texels. Two passes: the first computes the base colour and the
    // marking field, the second decides where the markings land. The threshold has to be
    // derived from the field rather than fixed, because the noise is sampled in world
    // space and a small creature covers only part of one noise cell. A fixed threshold
    // left small creatures with no markings at all and large ones with too many.
    let mut base_of = vec![[0.0f32; 3]; n * n];
    let mut upper_weight = vec![0.0f32; n * n];
    let mut field_of = vec![0.0f32; n * n];
    let lower = colours[1];
    let upper = colours[0];
    let accent = colours[2];
    for i in 0..n * n {
        if hits[i] == 0 {
            continue;
        }
        let k = hits[i] as f32;
        let p = [pos_sum[i][0] / k, pos_sum[i][1] / k, pos_sum[i][2] / k];
        let nrm = [nrm_sum[i][0] / k, nrm_sum[i][1] / k, nrm_sum[i][2] / k];

        // Belly against back. A downward-facing surface takes the second palette colour
        // and an upward-facing one the first. The weights are measured, not chosen for
        // looks: at 0.7/0.3 towards the normal the belly covered 3 to 18 percent of the
        // painted texels on three creatures, and at 0.5/0.5 it covered 95 percent of one,
        // because everything below a sphere's equator counts as downward. 0.8/0.2 puts
        // the boundary about 20 degrees below horizontal, which measured as 20 to 33
        // percent belly on four creatures.
        let down = (-nrm[1]).clamp(0.0, 1.0);
        // A mesh with no vertical extent has no bottom, so every surface is treated as
        // mid-height and the normal alone decides.
        let up = if height > 1e-4 {
            ((p[1] - min.y) / height).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let belly = (down * 0.8 + (1.0 - up) * 0.2).clamp(0.0, 1.0);
        let t = 1.0 - smooth((belly * 1.5).min(1.0));
        base_of[i] = mix(lower, upper, t);
        upper_weight[i] = t;

        // The marking field is sampled at the texel's world position, so a patch of
        // surface takes one colour and the pattern runs across part boundaries instead of
        // restarting at each UV island. The frequency has to put several features across
        // the body: at one feature per 2.4 body widths the whole creature fell inside a
        // single cell of the field, and the measured result was 0 marked texels.
        let scale = 1.0 / (extent * 0.6);
        field_of[i] = wave_field(p, scale);
    }

    // Markings are the top 30 percent of the field by rank among back texels. Rank, not a
    // threshold on the value: the field over a body a few features wide spans a narrow
    // range and is concentrated near its top, so any absolute cut either marked nothing or
    // marked two thirds of the body. Ranking gives a fixed share of the surface whatever
    // the field's shape. Restricting the candidates to back texels matters because the
    // field and the back need not overlap: on one creature the field's maximum sat on the
    // belly, and a global threshold then left no markings at all.
    let mut candidates: Vec<(f32, usize)> = (0..n * n)
        .filter(|&i| hits[i] > 0 && upper_weight[i] >= 0.5)
        .map(|i| (field_of[i], i))
        .collect();
    candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let take = candidates.len() * 30 / 100;
    let mut mask = vec![0.0f32; n * n];
    for &(_, i) in candidates.iter().skip(candidates.len() - take) {
        mask[i] = 1.0;
    }
    // Blur the mask so its edge is a gradient over several texels. The softness has to
    // come from here rather than from the field: the field's gradient is nearly flat, so
    // a ramp on it produced either a hard polygon edge or no marking at all.
    blur_mask(&mut mask, n, 3);

    let mut rgba = vec![0u8; n * n * 4];
    let mut marked = 0usize;
    for i in 0..n * n {
        if hits[i] == 0 {
            continue;
        }
        // Markings sit on the back, which is what the shipped skins do: the belly is a
        // flat band. The fade-in starts at 0.4 so a marking does not stop on a contour
        // where the belly meets the back.
        let fade = ((upper_weight[i] - 0.4) / 0.3).clamp(0.0, 1.0);
        let w = mask[i] * 0.9 * fade;
        if w >= 0.5 {
            marked += 1;
        }
        let c = mix(base_of[i], accent, w);

        let o = i * 4;
        rgba[o] = (c[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        rgba[o + 1] = (c[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        rgba[o + 2] = (c[2].clamp(0.0, 1.0) * 255.0).round() as u8;
        rgba[o + 3] = 255;
    }

    // Dilate into the transparent border. Without this, bilinear filtering at an island
    // edge mixes in black, which shows as a dark outline on every seam. Three passes is
    // enough for a 512 texture and cheap enough to do on every load.
    dilate(&mut rgba, n, 3);

    Some(SkinTexture {
        width: size,
        height: size,
        rgba,
        covered,
        marked,
    })
}

/// Box-blur a scalar mask in place, `passes` times.
fn blur_mask(mask: &mut [f32], n: usize, passes: usize) {
    let mut tmp = vec![0.0f32; n * n];
    for _ in 0..passes {
        for y in 0..n {
            for x in 0..n {
                let mut acc = 0.0;
                let mut count = 0.0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= n as i32 || ny >= n as i32 {
                            continue;
                        }
                        acc += mask[ny as usize * n + nx as usize];
                        count += 1.0;
                    }
                }
                tmp[y * n + x] = acc / count;
            }
        }
        mask.copy_from_slice(&tmp);
    }
}

/// Grow the opaque region outwards by `passes` texels, copying the nearest colour.
fn dilate(rgba: &mut [u8], n: usize, passes: usize) {
    for _ in 0..passes {
        let prev = rgba.to_vec();
        for y in 0..n {
            for x in 0..n {
                let i = y * n + x;
                if prev[i * 4 + 3] != 0 {
                    continue;
                }
                let mut acc = [0u32; 3];
                let mut count = 0u32;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= n as i32 || ny >= n as i32 {
                            continue;
                        }
                        let j = ny as usize * n + nx as usize;
                        if prev[j * 4 + 3] == 0 {
                            continue;
                        }
                        acc[0] += prev[j * 4] as u32;
                        acc[1] += prev[j * 4 + 1] as u32;
                        acc[2] += prev[j * 4 + 2] as u32;
                        count += 1;
                    }
                }
                if count > 0 {
                    rgba[i * 4] = (acc[0] / count) as u8;
                    rgba[i * 4 + 1] = (acc[1] / count) as u8;
                    rgba[i * 4 + 2] = (acc[2] / count) as u8;
                    rgba[i * 4 + 3] = 255;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One quad in the XY plane covering the whole unit UV square.
    fn quad() -> MeshData {
        MeshData {
            positions: vec![
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(-1.0, 1.0, 0.0),
            ],
            normals: vec![Vec3::new(0.0, 0.0, 1.0); 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            normals_from_file: false,
            sanitised: 0,
        }
    }

    #[test]
    fn uv_stats_reports_the_unit_square() {
        let s = uv_stats(&quad());
        assert_eq!(s.triangles_total, 2);
        assert_eq!(s.triangles_with_uv, 2);
        assert_eq!(s.usable_fraction(), 1.0);
        assert!((s.u_min - 0.0).abs() < 1e-6 && (s.u_max - 1.0).abs() < 1e-6);
        assert!((s.v_min - 0.0).abs() < 1e-6 && (s.v_max - 1.0).abs() < 1e-6);
        assert_eq!(s.clipped, 0);
    }

    #[test]
    fn a_triangle_across_the_seam_is_counted() {
        let mut m = quad();
        // One corner a tile and a half away: no single shift can put the triangle inside
        // the unit square, so it straddles a seam.
        m.uvs[2] = [2.5, 1.0];
        let s = uv_stats(&m);
        assert!(s.clipped >= 1, "clipped {}", s.clipped);
    }

    #[test]
    fn generate_covers_the_uv_square() {
        let colours = [[0.2, 0.4, 0.6], [0.9, 0.8, 0.7], [1.0, 0.0, 0.0]];
        let t = generate(&quad(), &colours, 64).expect("texture");
        assert_eq!(t.width, 64);
        assert_eq!(t.height, 64);
        assert_eq!(t.rgba.len(), 64 * 64 * 4);
        // A full-coverage quad leaves nothing transparent after dilation either.
        assert_eq!(t.rgba.iter().skip(3).step_by(4).filter(|&&a| a == 0).count(), 0);
        assert!(t.covered >= 64 * 64 - 64 * 3, "covered {} of 4096", t.covered);
    }

    /// The bottom of the mesh takes the palette's lower colour and the top its upper
    /// colour, so the vertical split is visible in the data and not just claimed.
    #[test]
    fn the_colour_split_follows_height() {
        let colours = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5, 0.5]];
        let t = generate(&quad(), &colours, 32).expect("texture");
        let at = |x: usize, y: usize| {
            let i = (y * 32 + x) * 4;
            (t.rgba[i], t.rgba[i + 1], t.rgba[i + 2])
        };
        // Row 0 is the bottom of the UV square and the bottom of the mesh.
        let bottom = at(16, 1);
        let top = at(16, 30);
        assert!(
            bottom.0 > top.0 + 20,
            "bottom {bottom:?} should be clearly lighter than top {top:?}"
        );
    }

    /// A downward-facing surface takes the lower colour whatever its height, which is
    /// what makes the underside of a limb come out as belly rather than back. The test
    /// compares the same texel on two otherwise identical meshes, so it measures the
    /// normal's effect and not the height's.
    #[test]
    fn downward_normals_take_the_lower_colour() {
        let colours = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5, 0.5]];
        let mut down = quad();
        down.normals = vec![Vec3::new(0.0, -1.0, 0.0); 4];
        let mut up = quad();
        up.normals = vec![Vec3::new(0.0, 1.0, 0.0); 4];
        let d = generate(&down, &colours, 32).expect("texture");
        let u = generate(&up, &colours, 32).expect("texture");
        let i = (16 * 32 + 16) * 4;
        assert!(
            d.rgba[i] > u.rgba[i] + 40,
            "downward {} should be clearly lighter than upward {}",
            d.rgba[i],
            u.rgba[i]
        );
    }

    /// A mesh with no UVs cannot be painted, and says so rather than returning a blank
    /// texture that would silently render as black.
    #[test]
    fn no_uvs_generates_nothing() {
        let mut m = quad();
        m.uvs = vec![[0.0, 0.0]; 4];
        assert!(generate(&m, &[[0.5; 3]; 3], 32).is_none());
    }

    /// Triangles outside the unit square are shifted into it rather than dropped. A
    /// creature's UVs run from -3.5 to 0.98, so without this most of the atlas would be
    /// empty.
    #[test]
    fn uvs_outside_the_square_still_paint() {
        let mut m = quad();
        for uv in m.uvs.iter_mut() {
            uv[0] -= 2.0;
            uv[1] -= 1.0;
        }
        let t = generate(&m, &[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5, 0.5]], 32)
            .expect("texture");
        assert!(t.covered > 32 * 32 / 2, "covered {} texels", t.covered);
    }

    /// Dilation must not touch a texel that is already opaque, and must stop after the
    /// requested number of passes instead of flooding the whole texture.
    #[test]
    fn dilation_does_not_flood() {
        let n = 8usize;
        let mut rgba = vec![0u8; n * n * 4];
        rgba[(4 * n + 4) * 4 + 3] = 255;
        dilate(&mut rgba, n, 1);
        let opaque = rgba.iter().skip(3).step_by(4).filter(|&&a| a != 0).count();
        // The centre plus its eight neighbours.
        assert_eq!(opaque, 9, "one pass should grow a single texel to 3x3");
    }

    /// A mesh that faces entirely downwards has no back, so there is nowhere to put a
    /// marking. That must produce a clean texture rather than a panic or a full-body
    /// accent.
    #[test]
    fn a_belly_only_mesh_gets_no_markings() {
        let mut m = quad();
        m.normals = vec![Vec3::new(0.0, -1.0, 0.0); 4];
        let colours = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let t = generate(&m, &colours, 32).expect("texture");
        assert_eq!(t.marked, 0, "no back means no markings");
        for px in t.rgba.chunks_exact(4) {
            assert!(px[2] < 40, "no blue should appear, got {:?}", px);
        }
    }

    #[test]
    fn size_is_clamped() {
        let colours = [[0.5; 3]; 3];
        assert_eq!(generate(&quad(), &colours, 1).unwrap().width, 16);
        assert_eq!(generate(&quad(), &colours, 99999).unwrap().width, 2048);
    }

    /// Two quads: a back at the left of the atlas facing up, and a belly at the right
    /// facing down. The point is that both appear in the same texture.
    fn back_and_belly() -> MeshData {
        MeshData {
            positions: vec![
                Vec3::new(-1.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(-1.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(1.0, 0.0, -1.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            normals: vec![
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
            ],
            uvs: vec![
                [0.0, 0.0],
                [0.5, 0.0],
                [0.5, 1.0],
                [0.0, 1.0],
                [0.5, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.5, 1.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            normals_from_file: true,
            sanitised: 0,
        }
    }

    /// All three palette colours have to appear. A generator that never reaches the
    /// third colour is not painting markings, whatever the code says it does.
    #[test]
    fn every_palette_colour_appears() {
        let colours = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let t = generate(&back_and_belly(), &colours, 64).expect("texture");
        let mut counts = [0usize; 3];
        for px in t.rgba.chunks_exact(4) {
            if px[3] == 0 {
                continue;
            }
            let mut best = 0;
            let mut best_d = f32::INFINITY;
            for (k, c) in colours.iter().enumerate() {
                let d = (px[0] as f32 / 255.0 - c[0]).powi(2)
                    + (px[1] as f32 / 255.0 - c[1]).powi(2)
                    + (px[2] as f32 / 255.0 - c[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = k;
                }
            }
            counts[best] += 1;
        }
        let total: usize = counts.iter().sum();
        for (k, n) in counts.iter().enumerate() {
            assert!(
                *n * 100 / total >= 2,
                "palette colour {k} covers {n} of {total} texels: {:?}",
                counts
            );
        }
    }

}
