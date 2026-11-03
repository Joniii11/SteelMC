use std::ops::{Add, Mul, Neg, Sub};
use std::simd::Simd;
#[cfg(any(test, target_feature = "avx512f"))]
use std::simd::{
    Select,
    cmp::{SimdPartialEq, SimdPartialOrd},
};

/// Gradient vectors shared between Perlin and simplex noise (from vanilla `SimplexNoise.GRADIENT`).
pub const GRADIENT: [[f64; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, -1.0],
];

/// See `GRADIENT` for details. This is a f32 version of the gradient table for use in f32 noise functions
pub const GRADIENT_F32: [[f32; 3]; 16] = gradient_f32();

/// Dot product of gradient vector and offset vector.
#[inline]
#[must_use]
pub fn dot(g: &[f64; 3], x: f64, y: f64, z: f64) -> f64 {
    g[0] * x + g[1] * y + g[2] * z
}
/// f32 N-lane gradient dot product.
///
/// AVX-512 builds evaluate Minecraft's 16-entry `GRADIENT` table branchlessly
/// from the hash bits. Baseline builds assemble component vectors from the
/// table, which avoids expensive mask work on current non-AVX-512 targets.
#[inline]
#[must_use]
pub fn grad_dot_simd<const N: usize>(
    hashes: [usize; N],
    x: Simd<f32, N>,
    y: Simd<f32, N>,
    z: Simd<f32, N>,
) -> Simd<f32, N>
where
    Simd<f32, N>: Mul<Output = Simd<f32, N>>
        + Add<Output = Simd<f32, N>>
        + Sub<Output = Simd<f32, N>>
        + Neg<Output = Simd<f32, N>>,
{
    #[cfg(target_feature = "avx512f")]
    {
        let [gx, gy, gz] = gradient_components_simd(hashes);
        (gx * x + gy * y) + gz * z
    }

    #[cfg(not(target_feature = "avx512f"))]
    {
        let gradients = hashes.map(|hash| GRADIENT_F32[hash & 15]);
        let gx = Simd::from_array(gradients.map(|gradient| gradient[0]));
        let gy = Simd::from_array(gradients.map(|gradient| gradient[1]));
        let gz = Simd::from_array(gradients.map(|gradient| gradient[2]));
        gx * x + gy * y + gz * z
    }
}

#[cfg(any(test, target_feature = "avx512f"))]
#[inline]
fn gradient_components_simd<const N: usize>(hashes: [usize; N]) -> [Simd<f32, N>; 3] {
    let hashes = Simd::<i32, N>::from_array(hashes.map(|hash| (hash & 15) as i32));
    let zero = Simd::splat(0.0);
    let one = Simd::splat(1.0);
    let low_four = hashes.simd_lt(Simd::splat(4));
    let low_eight = hashes.simd_lt(Simd::splat(8));
    let x_from_v = hashes.simd_eq(Simd::splat(12)) | hashes.simd_eq(Simd::splat(14));
    let u = (hashes & Simd::splat(1))
        .simd_eq(Simd::splat(0))
        .select(one, -one);
    let v = (hashes & Simd::splat(2))
        .simd_eq(Simd::splat(0))
        .select(one, -one);

    // Zero coefficients stay positive, including before multiplying nonfinite inputs.
    let gx = low_eight.select(u, x_from_v.select(v, zero));
    let gy = low_eight.select(low_four.select(v, zero), u);
    let gz = (!low_four & !x_from_v).select(v, zero);
    [gx, gy, gz]
}

/// Calculate the f32 dot product used by vanilla Perlin noise.
#[expect(clippy::inline_always, reason = "hot-path noise primitive")]
#[inline(always)]
#[must_use]
pub fn grad_dot(hash: usize, x: f32, y: f32, z: f32) -> f32 {
    let g = &GRADIENT_F32[hash & 15];
    g[0] * x + g[1] * y + g[2] * z
}

/// Compute corner noise contribution for a simplex vertex.
#[inline]
#[must_use]
pub fn corner_noise_3d(index: usize, x: f64, y: f64, z: f64, base: f64) -> f64 {
    let t0 = base - x * x - y * y - z * z;
    if t0 < 0.0 {
        0.0
    } else {
        let t0 = t0 * t0;
        t0 * t0 * dot(&GRADIENT[index], x, y, z)
    }
}

const fn gradient_f32() -> [[f32; 3]; 16] {
    let mut result = [[0.0; 3]; 16];
    let mut i = 0;

    while i < GRADIENT.len() {
        result[i] = [
            GRADIENT[i][0] as f32,
            GRADIENT[i][1] as f32,
            GRADIENT[i][2] as f32,
        ];
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use std::array;

    use super::*;

    #[test]
    fn hash_selected_gradients_keep_scalar_float_semantics() {
        fn check_width<const N: usize>() {
            let samples = [
                [0.0, 0.0, 0.0],
                [-0.0, 0.0, 0.0],
                [0.0, -0.0, 0.0],
                [0.0, 0.0, -0.0],
                [-0.0, -0.0, 0.0],
                [-0.0, 0.0, -0.0],
                [0.0, -0.0, -0.0],
                [-0.0, -0.0, -0.0],
                [0.125, -0.25, 0.75],
                [f32::from_bits(1), -f32::MIN_POSITIVE, -f32::from_bits(1)],
                [f32::MAX, -f32::MAX, f32::MAX],
                [1.0, f32::INFINITY, 2.0],
                [1.0, f32::NEG_INFINITY, 2.0],
                [f32::INFINITY, 1.0, 2.0],
                [1.0, 2.0, f32::INFINITY],
                [f32::NAN, 0.25, 0.75],
                [0.125, f32::NAN, 0.75],
                [0.125, 0.25, f32::NAN],
            ];

            for first_hash in (0..16).step_by(N) {
                let hashes: [usize; N] = array::from_fn(|lane| first_hash + lane);
                let [gx, gy, gz] = gradient_components_simd(hashes);

                for [x, y, z] in samples {
                    let xs = Simd::splat(x);
                    let ys = Simd::splat(y);
                    let zs = Simd::splat(z);
                    let selected = (gx * xs + gy * ys) + gz * zs;
                    let production = grad_dot_simd(hashes, xs, ys, zs);

                    for (lane, hash) in hashes.into_iter().enumerate() {
                        let expected = grad_dot(hash, x, y, z);
                        for actual in [selected[lane], production[lane]] {
                            if expected.is_nan() {
                                assert!(actual.is_nan(), "hash {hash}: ({x}, {y}, {z})");
                            } else {
                                assert_eq!(
                                    actual.to_bits(),
                                    expected.to_bits(),
                                    "hash {hash}: ({x}, {y}, {z})",
                                );
                            }
                        }
                    }
                }
            }
        }

        check_width::<4>();
        check_width::<8>();
        check_width::<16>();
    }
}
