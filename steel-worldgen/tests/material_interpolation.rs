//! Checks that terrain fill and material prefill use the same vanilla volume interpolation.

use steel_worldgen::density::DimensionNoises;
use steel_worldgen::density_functions::overworld::{
    OverworldColumnCache, OverworldNoiseSettings, OverworldNoises,
};
use steel_worldgen::noise::NoiseChunk;
use steel_worldgen::noise_parameters::get_noise_parameters;
use steel_worldgen::random::{Random, xoroshiro::Xoroshiro};

#[test]
fn material_values_match_volume_prefill() {
    for seed in [0, 13579] {
        let splitter = Xoroshiro::from_seed(seed).next_positional();
        let noises = OverworldNoises::create(seed, &splitter, &get_noise_parameters());
        for (cx, cz) in [(0, 0), (-418_462, 366_791)] {
            let mut noise_chunk = NoiseChunk::<OverworldNoises>::new(cx * 16, cz * 16);
            let mut cache = OverworldColumnCache::default();
            cache.init_grid(cx * 16, cz * 16, &noises);
            let mut material_cache = OverworldColumnCache::default();
            let count = OverworldNoises::material_ore_vein_value_count();
            let mut filled = vec![0.0_f32; 256 * OverworldNoiseSettings::HEIGHT as usize * count];
            noise_chunk.fill(&noises, &mut cache, None, |x, y, z, _, values, _| {
                let offset =
                    (((y - OverworldNoiseSettings::MIN_Y) as usize * 16 + z) * 16 + x) * count;
                noises.fill_material_ore_vein_values(
                    &mut material_cache,
                    values,
                    cx * 16 + x as i32,
                    y,
                    cz * 16 + z as i32,
                    &mut filled[offset..offset + count],
                );
            });
            let mut material_cache = OverworldColumnCache::default();
            let prefilled =
                noise_chunk.prefill_material_ore_vein_values(&noises, &mut material_cache);
            for (i, (&actual, &expected)) in filled.iter().zip(prefilled.iter()).enumerate() {
                assert_eq!(
                    actual.to_bits(),
                    expected.to_bits(),
                    "seed={seed} chunk=({cx},{cz}) material offset={i}: fill={actual}, volume={expected}"
                );
            }
        }
    }
}
