//! Source-prepared differential checks of the real generated End cache.

#![feature(portable_simd)]

use std::array::from_fn;

use steel_worldgen::density::{ColumnCache, DimensionNoises, NoiseSettings};
use steel_worldgen::density_functions::end::{EndColumnCache, EndNoiseSettings, EndNoises};
use steel_worldgen::noise_parameters::get_noise_parameters;
use steel_worldgen::random::{Random, legacy_random::LegacyRandom};

#[path = "support/end_column_grid_frozen.rs"]
mod frozen;

macro_rules! active_bits {
    ($cache:expr) => {{
        let cache = $cache;
        [
            cache.x as u32,
            cache.z as u32,
            cache.df_end__islands.to_bits(),
            cache.router_continentalness.to_bits(),
            cache.router_depth.to_bits(),
            cache.router_erosion.to_bits(),
            cache.router_ridges.to_bits(),
            cache.router_temperature.to_bits(),
            cache.router_vegetation.to_bits(),
        ]
    }};
}

fn create_noises(seed: u64) -> EndNoises {
    let splitter = LegacyRandom::from_seed(seed).next_positional();
    EndNoises::create(seed, &splitter, &get_noise_parameters())
}

#[test]
fn end_grid_and_raw_fallback_match_frozen_active_bits() {
    for seed in [13_579, 2_965_282_071_327_931_563] {
        let noises = create_noises(seed);
        let mut live = EndColumnCache::new();
        let mut reference = frozen::EndColumnCache::new();
        for (origin_x, origin_z) in [
            (0_i32, 0_i32),
            (16, -16),
            (-17, 5),
            (4, 12),
            (8, -8),
            (-4, -12),
            (i32::MAX, i32::MAX),
            (i32::MAX - 4, i32::MAX - 8),
            (29_999_984, -30_000_000),
            (i32::MAX - 15, i32::MIN),
        ] {
            // Exercise reinitialization after valid raw coordinates/cache values.
            live.ensure(31, -23, &noises);
            reference.ensure(31, -23, &noises);
            live.init_grid(origin_x, origin_z, &noises);
            reference.init_grid(origin_x, origin_z, &noises);
            assert_eq!(
                active_bits!(&live),
                active_bits!(&reference),
                "init at {origin_x},{origin_z}"
            );

            for dx in [-1_i32, 0, 1, 4, 7, 8, 12, 16, 20] {
                for dz in [-1_i32, 0, 1, 4, 7, 8, 12, 16, 20] {
                    let x = origin_x.wrapping_add(dx);
                    let z = origin_z.wrapping_add(dz);
                    for _ in 0..2 {
                        live.ensure(x, z, &noises);
                        reference.ensure(x, z, &noises);
                        assert_eq!(
                            active_bits!(&live),
                            active_bits!(&reference),
                            "seed={seed} ({x},{z})"
                        );
                    }
                }
            }

            let mut live_copy = live.clone();
            let mut reference_copy = reference.clone();
            live_copy.ensure(origin_x, origin_z, &noises);
            reference_copy.ensure(origin_x, origin_z, &noises);
            assert_eq!(active_bits!(&live_copy), active_bits!(&reference_copy));
        }
    }
}

#[test]
fn end_corner_column_noise_bits_match_frozen_cache_for_real_grid() {
    let width = EndNoiseSettings::CELL_WIDTH;
    let rows = EndNoiseSettings::HEIGHT / EndNoiseSettings::CELL_HEIGHT;
    let ys: Vec<i32> = (0..=rows)
        .map(|row| EndNoiseSettings::MIN_Y + row * EndNoiseSettings::CELL_HEIGHT)
        .collect();
    for seed in [13_579, 2_965_282_071_327_931_563] {
        let noises = create_noises(seed);
        for (origin_x, origin_z) in [(0, 0), (-4112, 2048)] {
            let mut live = EndColumnCache::new();
            let mut reference = frozen::EndColumnCache::new();
            live.init_grid(origin_x, origin_z, &noises);
            reference.init_grid(origin_x, origin_z, &noises);
            for cx in 0..=16 / width {
                for cz in 0..=16 / width {
                    let x = origin_x + cx * width;
                    let z = origin_z + cz * width;
                    live.ensure(x, z, &noises);
                    reference.ensure(x, z, &noises);
                    let mut blended = vec![0.0_f32; ys.len()];
                    noises.compute_noise_column(x, &ys, z, &mut blended);
                    let count = EndNoises::interpolated_count();
                    for first in (0..ys.len())
                        .step_by(8)
                        .filter(|first| first + 8 <= ys.len())
                    {
                        let y_batch: [i32; 8] = from_fn(|lane| ys[first + lane]);
                        let blended_batch: [f32; 8] = from_fn(|lane| blended[first + lane]);
                        let sentinel = f32::from_bits(0x7fc0_5eed);
                        let mut actual = vec![sentinel; count * 8 + 2];
                        let mut expected = vec![sentinel; count * 8 + 2];
                        noises.fill_cell_corner_densities_y_simd::<8>(
                            &mut live,
                            x,
                            y_batch,
                            z,
                            blended_batch,
                            &mut actual,
                        );
                        frozen::fill_cell_corner_densities_y_simd::<8>(
                            &noises,
                            &reference,
                            x,
                            y_batch,
                            z,
                            blended_batch,
                            &mut expected,
                        );
                        assert_eq!(
                            actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                            expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                            "N8seed={seed} ({x},{z}) batch{first}"
                        );
                    }
                    for (&y, &blended_value) in ys.iter().zip(&blended) {
                        let mut actual = [0.0_f32; 1];
                        let mut expected = [0.0_f32; 1];
                        noises.fill_cell_corner_densities(
                            &mut live,
                            x,
                            y,
                            z,
                            blended_value,
                            &mut actual,
                        );
                        frozen::fill_cell_corner_densities(
                            &noises,
                            &reference,
                            x,
                            y,
                            z,
                            blended_value,
                            &mut expected,
                        );
                        assert_eq!(
                            actual[0].to_bits(),
                            expected[0].to_bits(),
                            "seed={seed} ({x},{y},{z})"
                        );
                    }
                }
            }
        }
    }
}
