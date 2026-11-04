//! gradient lookup without scalar component gathers

use std::arch::x86_64::_mm_shuffle_epi8;
use std::simd::Simd;
use std::simd::num::SimdInt;

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
pub(super) fn dot<const N: usize>(
    hashes: [u8; N],
    x: Simd<f32, N>,
    y: Simd<f32, N>,
    z: Simd<f32, N>,
) -> Simd<f32, N> {
    assert!(
        N == 4 || N == 8,
        "gradient lookup requires four or eight lanes"
    );

    let indices = (Simd::<u8, N>::from_array(hashes) & Simd::splat(15)).resize::<16>(0);
    let [gx, gy, gz] = COMPONENTS.map(|component| {
        let table = Simd::<i8, 16>::from_array(component);

        // SAFETY: the module requires SSSE3; this only shuffles register values
        let selected: Simd<i8, 16> =
            unsafe { _mm_shuffle_epi8(table.into(), indices.into()) }.into();
        selected.resize::<N>(0).cast::<f32>()
    });

    (gx * x + gy * y) + gz * z
}
