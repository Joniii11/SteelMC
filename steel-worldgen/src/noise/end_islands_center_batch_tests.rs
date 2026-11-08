//! Source-prepared batch/scalar comparisons using actual EndIslands instances.

use std::array::from_fn;

use super::EndIslands;
use crate::density_functions::end::EndNoiseSettings;

fn assert_batch<const N: usize>(noise: &EndIslands, points: [(f64, f64); N]) -> bool {
    let actual = noise.sample_batch(points);
    let expected = points.map(|(x, z)| noise.sample(x, f64::NAN, z));
    let saw_effective_contribution = expected
        .iter()
        .any(|value| value.is_finite() && *value > -0.84375_f64);
    assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
    saw_effective_contribution
}

#[test]
fn end_center_batch_matches_scalar_real_corner_grids() {
    let mut saw_effective_contribution = false;
    let width = EndNoiseSettings::CELL_WIDTH;
    let side = (16 / width + 1) as usize;
    assert_eq!(
        side * side,
        9,
        "this N9 case covers the current extracted End grid"
    );
    for seed in [13_579, 2_965_282_071_327_931_563] {
        let noise = EndIslands::new(seed);
        for (x0, z0) in [
            (0_i32, 0_i32),
            (-16, 0),
            (16, -32),
            (4096, -2048),
            (-4112, 2064),
            (29_999_984, -30_000_000),
            (i32::MAX - 15, i32::MIN),
        ] {
            let points: [(f64, f64); 9] = from_fn(|index| {
                let x = x0.wrapping_add((index / side) as i32 * width);
                let z = z0.wrapping_add((index % side) as i32 * width);
                (f64::from(x), f64::from(z))
            });
            saw_effective_contribution |= assert_batch(&noise, points);
        }
    }
    assert!(
        saw_effective_contribution,
        "real seeded batch fixtures must exercise contribution FP updates"
    );
}

#[test]
fn end_center_batch_preserves_truncation_fallbacks_and_batch_edges() {
    for seed in [13_579, 2_965_282_071_327_931_563] {
        let noise = EndIslands::new(seed);
        // All nine signed remainder pairs share exact center(0,0), including-1.
        assert_batch::<9>(
            &noise,
            from_fn(|index| {
                let x = (index / 3) as i32 * 8 - 8;
                let z = (index % 3) as i32 * 8 - 8;
                (f64::from(x), f64::from(z))
            }),
        );
        // Nine distinct centers exercise the worst-case no-sharing path.
        assert_batch::<9>(
            &noise,
            from_fn(|index| (4096.0 + index as f64 * 32.0, -2048.0 - index as f64 * 48.0)),
        );
        assert_batch::<0>(&noise, []);
        assert_batch::<1>(&noise, [(-0.0, 0.0)]);
        for first in [
            f64::NEG_INFINITY,
            f64::MIN,
            f64::from(i32::MIN),
            -30_000_000.0,
            -17.0,
            -9.0,
            -8.0,
            -7.0,
            -1.0,
            0.0,
            1.0,
            7.0,
            8.0,
            9.0,
            17.0,
            30_000_000.0,
            f64::from(i32::MAX),
            f64::MAX,
            f64::INFINITY,
            f64::NAN,
        ] {
            assert_batch(
                &noise,
                [
                    (first, -4096.0),
                    (-4096.0, first),
                    (first, first),
                    (4096.0, 2048.0),
                    (4096.0, 2048.0),
                    (-8.0, 8.0),
                    (8.0, -8.0),
                    (-0.0, 0.0),
                    (first, first),
                ],
            );
        }
    }
}

#[test]
fn exact_legacy_and_cell_grid_leaf_values_match_scalar_at_quart_and_wrap_edges() {
    // These are actual public cache-coordinate formulas, including unaligned
    // origins and upper i32 wrapping; no Euclidean center approximation is used.
    let origins = [
        (0_i32, 0_i32),
        (16, 16),
        (-16, -16),
        (-16, 16),
        (16, -16),
        (4, 12),
        (8, -8),
        (-4, -12),
        (-17, 5),
        (29_999_984, -30_000_000),
        (i32::MAX - 15, i32::MIN),
        (i32::MAX, i32::MAX),
        (i32::MAX - 4, i32::MAX - 8),
        (i32::MIN, i32::MAX - 12),
    ];
    for seed in [13_579, 2_965_282_071_327_931_563] {
        let islands = EndIslands::new(seed);
        for (origin_x, origin_z) in origins {
            for quart_step in [1_i32, 2] {
                let points: [(f64, f64); 9] = from_fn(|index| {
                    let rel_x = index as i32 % 3;
                    let rel_z = index as i32 / 3;
                    let x = (((origin_x >> 2) + rel_x * quart_step) << 2) as f64;
                    let z = (((origin_z >> 2) + rel_z * quart_step) << 2) as f64;
                    (x, z)
                });
                let expected = points.map(|(x, z)| islands.sample(x, 0.0, z).to_bits());
                let actual = islands.sample_batch(points).map(f64::to_bits);
                assert_eq!(
                    actual, expected,
                    "seed={seed} origin={origin_x},{origin_z} step={quart_step}"
                );
            }
        }
    }
}
