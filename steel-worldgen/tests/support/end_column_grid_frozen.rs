//! Frozen baseline cache/flat expression bodies from the original generated End module.
//! Uses the real, unchanged current EndNoises instance; no fake data or sampler.
#![expect(
    unused_variables,
    reason = "frozen generated signatures keep unused coordinate/cache arguments"
)]
#![expect(
    unused_parens,
    reason = "frozen generated density expressions retain their exact syntax"
)]

use std::simd::cmp::{SimdOrd, SimdPartialEq};
use std::simd::num::{SimdFloat, SimdInt};
use std::simd::{Select, Simd};
use steel_worldgen::density_functions::end::EndNoises;

#[derive(Clone)]
pub struct EndColumnCache {
    #[doc = r" Raw x block coordinate (for non-flat router functions)."]
    pub x: i32,
    #[doc = r" Raw z block coordinate (for non-flat router functions)."]
    pub z: i32,
    #[doc = r" X coordinate used to evaluate the cached values."]
    qx: i32,
    #[doc = r" Z coordinate used to evaluate the cached values."]
    qz: i32,
    valid: bool,
    grid_first_quart_x: i32,
    grid_first_quart_z: i32,
    has_grid: bool,
    pub df_end__islands: f32,
    pub router_continentalness: f32,
    pub router_depth: f32,
    pub router_erosion: f32,
    pub router_ridges: f32,
    pub router_temperature: f32,
    pub router_vegetation: f32,
    grid_df_end__islands: [f32; 9],
    grid_router_continentalness: [f32; 9],
    grid_router_depth: [f32; 9],
    grid_router_erosion: [f32; 9],
    grid_router_ridges: [f32; 9],
    grid_router_temperature: [f32; 9],
    grid_router_vegetation: [f32; 9],
}

impl EndColumnCache {
    #[doc = r" Grid side length (quart positions per axis)."]
    const GRID_SIDE: i32 = 3;
    #[doc = r" Create a new column cache without a pre-computed grid."]
    #[must_use]
    pub fn new() -> Self {
        Self {
            x: 0,
            z: 0,
            qx: i32::MIN,
            qz: i32::MIN,
            valid: false,
            grid_first_quart_x: 0,
            grid_first_quart_z: 0,
            has_grid: false,
            df_end__islands: 0.0,
            router_continentalness: 0.0,
            router_depth: 0.0,
            router_erosion: 0.0,
            router_ridges: 0.0,
            router_temperature: 0.0,
            router_vegetation: 0.0,
            grid_df_end__islands: [0.0; 9],
            grid_router_continentalness: [0.0; 9],
            grid_router_depth: [0.0; 9],
            grid_router_erosion: [0.0; 9],
            grid_router_ridges: [0.0; 9],
            grid_router_temperature: [0.0; 9],
            grid_router_vegetation: [0.0; 9],
        }
    }
    #[doc = r" Pre-compute flat-cached values for all quart positions in a chunk."]
    #[doc = r""]
    #[doc = r" After this call, `ensure()` for matching grid points copies from"]
    #[doc = r" the grid (O(1)). Other positions are evaluated at their exact"]
    #[doc = r" coordinates."]
    pub fn init_grid(&mut self, chunk_block_x: i32, chunk_block_z: i32, noises: &EndNoises) {
        self.grid_first_quart_x = chunk_block_x >> 2;
        self.grid_first_quart_z = chunk_block_z >> 2;
        self.has_grid = true;
        self.valid = false;
        for rel_z in 0..Self::GRID_SIDE {
            for rel_x in 0..Self::GRID_SIDE {
                let x = ((self.grid_first_quart_x + rel_x) << 2) as f64;
                let z = ((self.grid_first_quart_z + rel_z) << 2) as f64;
                let idx = (rel_z * Self::GRID_SIDE + rel_x) as usize;
                let val = compute_end__islands(noises, &*self, x, z);
                self.df_end__islands = val;
                self.grid_df_end__islands[idx] = self.df_end__islands;
                let val = compute_router_continentalness(noises, &*self, x, z);
                self.router_continentalness = val;
                let val = compute_router_depth(noises, &*self, x, z);
                self.router_depth = val;
                let val = compute_router_erosion(noises, &*self, x, z);
                self.router_erosion = val;
                let val = compute_router_ridges(noises, &*self, x, z);
                self.router_ridges = val;
                let val = compute_router_temperature(noises, &*self, x, z);
                self.router_temperature = val;
                let val = compute_router_vegetation(noises, &*self, x, z);
                self.router_vegetation = val;
                self.grid_router_continentalness[idx] = self.router_continentalness;
                self.grid_router_depth[idx] = self.router_depth;
                self.grid_router_erosion[idx] = self.router_erosion;
                self.grid_router_ridges[idx] = self.router_ridges;
                self.grid_router_temperature[idx] = self.router_temperature;
                self.grid_router_vegetation[idx] = self.router_vegetation;
            }
        }
    }
    #[doc = r" Ensure the cache is populated for the given `(x, z)` block coordinates."]
    #[doc = r""]
    #[doc = r" Grid-aligned in-bounds positions load from the pre-computed grid."]
    #[doc = r" Every other position is evaluated at its exact coordinates."]
    pub fn ensure(&mut self, x: i32, z: i32, noises: &EndNoises) {
        self.x = x;
        self.z = z;
        let quart_x = x >> 2;
        let quart_z = z >> 2;
        if self.has_grid {
            let rel_x = quart_x - self.grid_first_quart_x;
            let rel_z = quart_z - self.grid_first_quart_z;
            if x & 3 == 0
                && z & 3 == 0
                && rel_x >= 0
                && rel_z >= 0
                && rel_x < Self::GRID_SIDE
                && rel_z < Self::GRID_SIDE
            {
                let eval_x = quart_x << 2;
                let eval_z = quart_z << 2;
                if self.valid && self.qx == eval_x && self.qz == eval_z {
                    return;
                }
                let idx = (rel_z * Self::GRID_SIDE + rel_x) as usize;
                self.df_end__islands = self.grid_df_end__islands[idx];
                self.router_continentalness = self.grid_router_continentalness[idx];
                self.router_depth = self.grid_router_depth[idx];
                self.router_erosion = self.grid_router_erosion[idx];
                self.router_ridges = self.grid_router_ridges[idx];
                self.router_temperature = self.grid_router_temperature[idx];
                self.router_vegetation = self.grid_router_vegetation[idx];
                self.qx = eval_x;
                self.qz = eval_z;
                self.valid = true;
                return;
            }
            if self.valid && self.qx == x && self.qz == z {
                return;
            }
            self.qx = x;
            self.qz = z;
            let x = x as f64;
            let z = z as f64;
            let val = compute_end__islands(noises, &*self, x, z);
            self.df_end__islands = val;
            let val = compute_router_continentalness(noises, &*self, x, z);
            self.router_continentalness = val;
            let val = compute_router_depth(noises, &*self, x, z);
            self.router_depth = val;
            let val = compute_router_erosion(noises, &*self, x, z);
            self.router_erosion = val;
            let val = compute_router_ridges(noises, &*self, x, z);
            self.router_ridges = val;
            let val = compute_router_temperature(noises, &*self, x, z);
            self.router_temperature = val;
            let val = compute_router_vegetation(noises, &*self, x, z);
            self.router_vegetation = val;
            self.valid = true;
            return;
        }
        if self.valid && self.qx == x && self.qz == z {
            return;
        }
        self.qx = x;
        self.qz = z;
        let x = x as f64;
        let z = z as f64;
        let val = compute_end__islands(noises, &*self, x, z);
        self.df_end__islands = val;
        let val = compute_router_continentalness(noises, &*self, x, z);
        self.router_continentalness = val;
        let val = compute_router_depth(noises, &*self, x, z);
        self.router_depth = val;
        let val = compute_router_erosion(noises, &*self, x, z);
        self.router_erosion = val;
        let val = compute_router_ridges(noises, &*self, x, z);
        self.router_ridges = val;
        let val = compute_router_temperature(noises, &*self, x, z);
        self.router_temperature = val;
        let val = compute_router_vegetation(noises, &*self, x, z);
        self.router_vegetation = val;
        self.valid = true;
    }
}

fn compute_end__islands(noises: &EndNoises, cache: &EndColumnCache, x: f64, z: f64) -> f32 {
    ({
        let __sc_a = ({
            let y = 0f64;
            (((((((((100.0) as f32)
                - ((((0f32 - x as f32).powi(2)
                    + (0f32 - y as f32).powi(2)
                    + (0f32 - z as f32).powi(2))
                .sqrt()) as f32)) as f32)
                .clamp(-100.0, 80.0)) as f32)
                - ((8.0) as f32)) as f32)
                * ((0.0078125) as f32)) as f32
        }) as f32;
        if __sc_a >= 80.0 {
            __sc_a
        } else {
            let __sc_b = (noises.end_islands.sample(x, 0.0, z) as f32) as f32;
            if __sc_a > __sc_b { __sc_a } else { __sc_b }
        }
    }) as f32
}

fn compute_router_continentalness(
    noises: &EndNoises,
    cache: &EndColumnCache,
    x: f64,
    z: f64,
) -> f32 {
    (0.0) as f32
}

fn compute_router_depth(noises: &EndNoises, cache: &EndColumnCache, x: f64, z: f64) -> f32 {
    (0.0) as f32
}

fn compute_router_erosion(noises: &EndNoises, cache: &EndColumnCache, x: f64, z: f64) -> f32 {
    (cache.df_end__islands) as f32
}

fn compute_router_ridges(noises: &EndNoises, cache: &EndColumnCache, x: f64, z: f64) -> f32 {
    (0.0) as f32
}

fn compute_router_temperature(noises: &EndNoises, cache: &EndColumnCache, x: f64, z: f64) -> f32 {
    (0.0) as f32
}

fn compute_router_vegetation(noises: &EndNoises, cache: &EndColumnCache, x: f64, z: f64) -> f32 {
    (0.0) as f32
}

pub fn fill_cell_corner_densities(
    noises: &EndNoises,
    cache: &EndColumnCache,
    x: i32,
    y: i32,
    z: i32,
    blended_noise_value: f32,
    out: &mut [f32],
) {
    let x = cache.x as f64;
    let z = cache.z as f64;
    let y = y as f64;
    out[0] = (((({
        let __lerp_alpha = ({
            let __coordinate = (y as i32).clamp(4, 32) - 4;
            0.0 + __coordinate as f32 * ((1.0 - 0.0) / (32 - 4) as f32)
        }) as f32;
        if __lerp_alpha == 0.0 {
            (-0.234375) as f32
        } else if __lerp_alpha == 1.0 {
            ({
                let __lerp_alpha = ({
                    let __coordinate = (y as i32).clamp(56, 312) - 56;
                    1.0 + __coordinate as f32 * ((0.0 - 1.0) / (312 - 56) as f32)
                }) as f32;
                if __lerp_alpha == 0.0 {
                    (-23.4375) as f32
                } else if __lerp_alpha == 1.0 {
                    ((((cache.df_end__islands) as f32)
                        + (((blended_noise_value as f32) as f32) as f32))
                        as f32) as f32
                } else {
                    let __lerp_first = (-23.4375) as f32;
                    __lerp_first
                        + __lerp_alpha
                            * ((((((cache.df_end__islands) as f32)
                                + (((blended_noise_value as f32) as f32) as f32))
                                as f32) as f32)
                                - __lerp_first)
                }
            }) as f32
        } else {
            let __lerp_first = (-0.234375) as f32;
            __lerp_first
                + __lerp_alpha
                    * ((({
                        let __lerp_alpha = ({
                            let __coordinate = (y as i32).clamp(56, 312) - 56;
                            1.0 + __coordinate as f32 * ((0.0 - 1.0) / (312 - 56) as f32)
                        }) as f32;
                        if __lerp_alpha == 0.0 {
                            (-23.4375) as f32
                        } else if __lerp_alpha == 1.0 {
                            ((((cache.df_end__islands) as f32)
                                + (((blended_noise_value as f32) as f32) as f32))
                                as f32) as f32
                        } else {
                            let __lerp_first = (-23.4375) as f32;
                            __lerp_first
                                + __lerp_alpha
                                    * ((((((cache.df_end__islands) as f32)
                                        + (((blended_noise_value as f32) as f32) as f32))
                                        as f32) as f32)
                                        - __lerp_first)
                        }
                    }) as f32)
                        - __lerp_first)
        }
    }) as f32) as f32)
        * ((0.64) as f32)) as f32;
}

pub const INTERPOLATED_COUNT: usize = 1;

pub fn fill_cell_corner_densities_y_simd<const N: usize>(
    noises: &EndNoises,
    cache: &EndColumnCache,
    x: i32,
    ys: [i32; N],
    z: i32,
    blended_noise_values: [f32; N],
    out: &mut [f32],
) {
    let x = cache.x as f64;
    let z = cache.z as f64;
    let ys = Simd::from_array(ys.map(f64::from));
    let blended_noise_value_v = Simd::from_array(blended_noise_values);
    let __values_simd_0 = (({
        let __lerp_alpha = {
            let __coordinate = ys
                .cast::<i32>()
                .simd_max(Simd::splat(4))
                .simd_min(Simd::splat(32))
                - Simd::splat(4);
            Simd::<f32, N>::splat(0.0)
                + __coordinate.cast::<f32>() * Simd::<f32, N>::splat((1.0 - 0.0) / (32 - 4) as f32)
        };
        let __zero = __lerp_alpha.simd_eq(Simd::<f32, N>::splat(0.0));
        let __one = __lerp_alpha.simd_eq(Simd::<f32, N>::splat(1.0));
        if __zero.all() {
            Simd::<f32, N>::splat(-0.234375)
        } else if __one.all() {
            {
                let __lerp_alpha = {
                    let __coordinate = ys
                        .cast::<i32>()
                        .simd_max(Simd::splat(56))
                        .simd_min(Simd::splat(312))
                        - Simd::splat(56);
                    Simd::<f32, N>::splat(1.0)
                        + __coordinate.cast::<f32>()
                            * Simd::<f32, N>::splat((0.0 - 1.0) / (312 - 56) as f32)
                };
                let __zero = __lerp_alpha.simd_eq(Simd::<f32, N>::splat(0.0));
                let __one = __lerp_alpha.simd_eq(Simd::<f32, N>::splat(1.0));
                if __zero.all() {
                    Simd::<f32, N>::splat(-23.4375)
                } else if __one.all() {
                    ((Simd::<f32, N>::splat(cache.df_end__islands)) + (blended_noise_value_v))
                } else {
                    let __lerp_first = Simd::<f32, N>::splat(-23.4375);
                    let __lerp_second =
                        ((Simd::<f32, N>::splat(cache.df_end__islands)) + (blended_noise_value_v));
                    let __lerp_value = __lerp_first + __lerp_alpha * (__lerp_second - __lerp_first);
                    __zero.select(__lerp_first, __one.select(__lerp_second, __lerp_value))
                }
            }
        } else {
            let __lerp_first = Simd::<f32, N>::splat(-0.234375);
            let __lerp_second = {
                let __lerp_alpha = {
                    let __coordinate = ys
                        .cast::<i32>()
                        .simd_max(Simd::splat(56))
                        .simd_min(Simd::splat(312))
                        - Simd::splat(56);
                    Simd::<f32, N>::splat(1.0)
                        + __coordinate.cast::<f32>()
                            * Simd::<f32, N>::splat((0.0 - 1.0) / (312 - 56) as f32)
                };
                let __zero = __lerp_alpha.simd_eq(Simd::<f32, N>::splat(0.0));
                let __one = __lerp_alpha.simd_eq(Simd::<f32, N>::splat(1.0));
                if __zero.all() {
                    Simd::<f32, N>::splat(-23.4375)
                } else if __one.all() {
                    ((Simd::<f32, N>::splat(cache.df_end__islands)) + (blended_noise_value_v))
                } else {
                    let __lerp_first = Simd::<f32, N>::splat(-23.4375);
                    let __lerp_second =
                        ((Simd::<f32, N>::splat(cache.df_end__islands)) + (blended_noise_value_v));
                    let __lerp_value = __lerp_first + __lerp_alpha * (__lerp_second - __lerp_first);
                    __zero.select(__lerp_first, __one.select(__lerp_second, __lerp_value))
                }
            };
            let __lerp_value = __lerp_first + __lerp_alpha * (__lerp_second - __lerp_first);
            __zero.select(__lerp_first, __one.select(__lerp_second, __lerp_value))
        }
    }) * (Simd::<f32, N>::splat(0.64)))
    .to_array();
    for lane in 0..N {
        out[lane * INTERPOLATED_COUNT + 0] = __values_simd_0[lane];
    }
}
