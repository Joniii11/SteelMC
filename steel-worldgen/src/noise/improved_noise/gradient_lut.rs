//! eight-lane gradient lookup without scalar component gathers

use std::arch::x86_64::_mm_shuffle_epi8;
use std::simd::num::SimdInt;
use std::simd::{Simd, f32x8};

use steel_math::GRADIENT_F32;

const COMPONENTS: [[i8; 16]; 3] = {
    let mut components = [[0; 16]; 3];
    let mut axis = 0;
    while axis < components.len() {
        let mut hash = 0;
        while hash < GRADIENT_F32.len() {
            let original = GRADIENT_F32[hash][axis];
            let compact = original as i8;
            assert!(matches!(compact, -1..=1));
            assert!((compact as f32).to_bits() == original.to_bits());
            components[axis][hash] = compact;
            hash += 1;
        }

        axis += 1;
    }

    components
};

#[inline]
pub(super) fn dot(hashes: [u8; 8], x: f32x8, y: f32x8, z: f32x8) -> f32x8 {
    let indices = (Simd::<u8, 8>::from_array(hashes) & Simd::splat(15)).resize::<16>(0);
    let [gx, gy, gz] = COMPONENTS.map(|component| {
        let table = Simd::<i8, 16>::from_array(component);

        // SAFETY: the module requires SSSE3; this only shuffles register values
        let selected: Simd<i8, 16> =
            unsafe { _mm_shuffle_epi8(table.into(), indices.into()) }.into();
        selected.extract::<0, 8>().cast::<f32>()
    });

    (gx * x + gy * y) + gz * z
}
