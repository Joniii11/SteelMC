use std::simd::{Simd, f64x4};

use crate::noise::{ImprovedNoise, PerlinNoise};
use crate::random::{PositionalRandom, Random, RandomSource, RandomSplitter, name_hash::NameHash};

const INPUT_FACTOR: f64 = 1.0181268882175227;
const TARGET_DEVIATION: f64 = 1.0 / 3.0;
const DEVIATION_COEFFICIENT: f64 = 0.2702247831245211;

#[derive(Debug, Clone)]
struct Layer {
    noise: ImprovedNoise,
    frequency: f64,
    amplitude: f32,
}

#[derive(Debug, Clone)]
enum Sampler {
    Stack {
        layers: Box<[Layer]>,
        max_value: f64,
    },
    Legacy {
        first: PerlinNoise,
        second: PerlinNoise,
        value_factor: f64,
        max_value: f64,
    },
}

/// Vanilla's 26.3 normal-noise sampler.
#[derive(Debug, Clone)]
pub struct NormalNoise {
    sampler: Sampler,
}

impl NormalNoise {
    #[must_use]
    /// Creates parity noise from a positional worldgen splitter.
    pub fn create(
        splitter: &RandomSplitter,
        noise_id: &str,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Self {
        let mut random = splitter.with_hash_of(&NameHash::new(noise_id));
        Self::create_from_random(&mut random, first_octave, amplitudes)
    }

    #[must_use]
    /// Creates parity noise from a vanilla random source.
    pub fn create_from_random(
        random: &mut RandomSource,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Self {
        let base_amplitude = parity_base_amplitude(first_octave, amplitudes);
        Self::create_from_params(
            random,
            first_octave,
            base_amplitude,
            amplitudes.len() as i32,
            true,
            amplitudes,
        )
    }

    #[must_use]
    /// Creates the legacy-Nether variant, whose octave sources are sequentially seeded.
    pub fn create_legacy_nether_biome(
        random: &mut RandomSource,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Self {
        let first = PerlinNoise::create_legacy_for_nether(random, first_octave, amplitudes);
        let second = PerlinNoise::create_legacy_for_nether(random, first_octave, amplitudes);
        let value_factor = parity_normalization_factor(1.0, amplitudes);
        let max_value = (first.max_value() + second.max_value()) * value_factor;
        Self {
            sampler: Sampler::Legacy {
                first,
                second,
                value_factor,
                max_value,
            },
        }
    }

    #[must_use]
    /// Creates current datapack noise from a positional worldgen splitter.
    pub fn create_with_params(
        splitter: &RandomSplitter,
        noise_id: &str,
        base_octave: i32,
        base_amplitude: f64,
        octave_count: i32,
        normalize: bool,
        amplitude_modifiers: &[f64],
    ) -> Self {
        let mut random = splitter.with_hash_of(&NameHash::new(noise_id));
        Self::create_from_params(
            &mut random,
            base_octave,
            base_amplitude,
            octave_count,
            normalize,
            amplitude_modifiers,
        )
    }

    #[must_use]
    /// Creates current datapack noise from a vanilla random source.
    pub fn create_from_random_with_params(
        random: &mut RandomSource,
        base_octave: i32,
        base_amplitude: f64,
        octave_count: i32,
        normalize: bool,
        amplitude_modifiers: &[f64],
    ) -> Self {
        Self::create_from_params(
            random,
            base_octave,
            base_amplitude,
            octave_count,
            normalize,
            amplitude_modifiers,
        )
    }

    fn create_from_params(
        random: &mut RandomSource,
        base_octave: i32,
        base_amplitude: f64,
        octave_count: i32,
        normalize: bool,
        amplitude_modifiers: &[f64],
    ) -> Self {
        let octaves = build_octaves(
            base_octave,
            base_amplitude,
            octave_count,
            normalize,
            amplitude_modifiers,
        );
        let target_amplitude = octaves
            .iter()
            .map(|octave| octave.amplitude.abs())
            .sum::<f64>();
        let normalization_factor = normalization_factor(target_amplitude, &octaves);

        let first_random = random.next_positional();
        let second_random = random.next_positional();
        let mut layers = Vec::with_capacity(octaves.len() * 2);
        for octave in octaves {
            let name = format!("octave_{}", octave.index);
            let mut first = first_random.with_hash_of(&NameHash::new(&name));
            let mut second = second_random.with_hash_of(&NameHash::new(&name));
            let amplitude = (normalization_factor * octave.amplitude) as f32;
            layers.push(Layer {
                noise: ImprovedNoise::new(&mut first),
                frequency: octave.frequency,
                amplitude,
            });
            layers.push(Layer {
                noise: ImprovedNoise::new(&mut second),
                frequency: octave.frequency * INPUT_FACTOR,
                amplitude,
            });
        }

        Self {
            sampler: Sampler::Stack {
                layers: layers.into_boxed_slice(),
                max_value: target_amplitude * TARGET_DEVIATION * 6.0,
            },
        }
    }

    #[inline]
    #[must_use]
    /// Samples the float-valued vanilla noise stack.
    pub fn get_value_f32(&self, x: f64, y: f64, z: f64) -> f32 {
        match &self.sampler {
            Sampler::Stack { layers, .. } => layers.iter().fold(0.0_f32, |value, layer| {
                value
                    + layer.amplitude
                        * layer.noise.noise_f32(
                            x * layer.frequency,
                            y * layer.frequency,
                            z * layer.frequency,
                        )
            }),
            Sampler::Legacy {
                first,
                second,
                value_factor,
                ..
            } => {
                ((first.get_value(x, y, z)
                    + second.get_value(x * INPUT_FACTOR, y * INPUT_FACTOR, z * INPUT_FACTOR))
                    * value_factor) as f32
            }
        }
    }

    #[inline]
    #[must_use]
    /// Samples the float-valued vanilla noise stack with a fixed Y coordinate.
    pub fn get_value_xz_f32(&self, x: f64, z: f64) -> f32 {
        self.get_value_f32(x, 0.0, z)
    }

    #[inline]
    #[must_use]
    /// Samples the float-valued vanilla noise stack with a fixed Z coordinate.
    pub fn get_value_xy_f32(&self, x: f64, y: f64) -> f32 {
        self.get_value_f32(x, y, 0.0)
    }

    #[inline]
    #[must_use]
    /// Samples the noise and widens the resulting vanilla float for legacy callers.
    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        f64::from(self.get_value_f32(x, y, z))
    }

    #[inline]
    #[must_use]
    /// Samples with a fixed Y coordinate and widens the result for legacy callers.
    pub fn get_value_xz(&self, x: f64, z: f64) -> f64 {
        f64::from(self.get_value_xz_f32(x, z))
    }

    #[inline]
    #[must_use]
    /// Samples with a fixed Z coordinate and widens the result for legacy callers.
    pub fn get_value_xy(&self, x: f64, y: f64) -> f64 {
        f64::from(self.get_value_xy_f32(x, y))
    }

    #[inline]
    #[must_use]
    /// Samples four Y coordinates.
    pub fn get_value_y_4x(&self, x: f64, ys: f64x4, z: f64) -> f64x4 {
        self.get_value_y_simd(x, ys, z)
    }

    #[inline]
    #[must_use]
    /// Samples a SIMD Y column, preserving each lane's vanilla float value.
    pub fn get_value_y_simd<const N: usize>(
        &self,
        x: f64,
        ys: Simd<f64, N>,
        z: f64,
    ) -> Simd<f64, N> {
        Simd::from_array(std::array::from_fn(|index| self.get_value(x, ys[index], z)))
    }

    #[inline]
    #[must_use]
    /// Returns the vanilla sampler's conservative range bound.
    pub fn max_value(&self) -> f64 {
        match &self.sampler {
            Sampler::Stack { max_value, .. } | Sampler::Legacy { max_value, .. } => *max_value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Octave {
    index: i32,
    frequency: f64,
    amplitude: f64,
}

fn build_octaves(
    base_octave: i32,
    base_amplitude: f64,
    octave_count: i32,
    normalize: bool,
    amplitude_modifiers: &[f64],
) -> Vec<Octave> {
    let mut frequency = 2.0_f64.powi(base_octave);
    let mut amplitude = base_amplitude;
    if normalize {
        amplitude *= normalization_constant(octave_count);
    }

    let mut octaves = Vec::with_capacity(octave_count as usize);
    for index in 0..octave_count {
        let modifier = amplitude_modifiers
            .get(index as usize)
            .copied()
            .unwrap_or(1.0);
        if modifier != 0.0 {
            octaves.push(Octave {
                index: base_octave + index,
                frequency,
                amplitude: amplitude * modifier,
            });
        }
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    octaves
}

fn normalization_constant(octave_count: i32) -> f64 {
    2.0_f64.powi(octave_count - 1) / (2.0_f64.powi(octave_count) - 1.0)
}

fn normalization_factor(target_amplitude: f64, octaves: &[Octave]) -> f64 {
    let variance = octaves
        .iter()
        .map(|octave| (DEVIATION_COEFFICIENT * octave.amplitude.abs()).powi(2))
        .sum::<f64>();
    if variance == 0.0 {
        return 0.0;
    }
    target_amplitude * TARGET_DEVIATION / (variance.sqrt() * 2.0_f64.sqrt())
}

fn parity_normalization_factor(base_amplitude: f64, amplitudes: &[f64]) -> f64 {
    let mut range = None;
    for (index, amplitude) in amplitudes.iter().enumerate() {
        if *amplitude != 0.0 {
            range = Some(match range {
                Some((first, _)) => (first, index as i32),
                None => (index as i32, index as i32),
            });
        }
    }
    let Some((first, last)) = range else {
        return 0.0;
    };
    base_amplitude * 0.5 * TARGET_DEVIATION / (0.1 * (1.0 + 1.0 / f64::from(last - first + 1)))
}

fn parity_base_amplitude(first_octave: i32, amplitudes: &[f64]) -> f64 {
    if amplitudes.is_empty() {
        return 1.0;
    }
    let octaves = build_octaves(first_octave, 1.0, amplitudes.len() as i32, true, amplitudes);
    let target_amplitude = octaves
        .iter()
        .map(|octave| octave.amplitude.abs())
        .sum::<f64>();
    let new_factor = normalization_factor(target_amplitude, &octaves);
    if new_factor == 0.0 {
        return 1.0;
    }
    parity_normalization_factor(1.0, amplitudes) / new_factor
}
