//! Direct cave carver.
//!
//! Mirrors Snapshot-2's direct `CaveWorldCarver`.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use steel_math::trig;
use steel_registry::carver::CaveWorldCarver;
use steel_utils::random::{Random, legacy_random::LegacyRandom};
use steel_utils::{BlockPos, ChunkPos};
use steel_worldgen::density::DimensionNoises;

use crate::worldgen::carver::{CarveRun, CarveSkipChecker, can_reach, horizontal_tunnel_radius};

/// Vanilla `WorldCarver.getRange()` — range in chunks. 4 each direction.
const CARVER_RANGE: i32 = 4;
/// Vanilla `SectionPos.sectionToBlockCoord(getRange() * 2 - 1)` = 112.
const MAX_TUNNEL_DISTANCE: i32 = (CARVER_RANGE * 2 - 1) * 16;

/// Position + rotation state that evolves along a tunnel's length.
#[derive(Debug, Clone, Copy)]
struct TunnelState {
    x: f64,
    y: f64,
    z: f64,
    /// Yaw.
    horizontal_rotation: f32,
    /// Pitch.
    vertical_rotation: f32,
}

/// Static per-tunnel configuration passed through `create_tunnel` recursion
/// unchanged between iterations.
#[derive(Debug, Clone, Copy)]
struct TunnelParams {
    tunnel_seed: i64,
    horizontal_radius_multiplier: f64,
    vertical_radius_multiplier: f64,
    thickness: f32,
    step: i32,
    dist: i32,
    y_scale: f64,
}

/// Mirrors `CaveWorldCarver.getThickness`, including its conditional random
/// draws. Keeping this separate makes the source-carver RNG sequence explicit.
fn sample_tunnel_thickness(config: &CaveWorldCarver, random: &mut LegacyRandom) -> f32 {
    let mut thickness = config.thickness.sample(random);
    if config.weird_thickness_bias && random.next_i32_bounded(10) == 0 {
        thickness *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
    }
    thickness
}

impl<N, F> CarveRun<'_, '_, N, F>
where
    N: DimensionNoises,
    F: FnMut(BlockPos) -> u16,
{
    /// Runs one cave-carver pass rooted in `source_pos`. `random` must have
    /// been seeded by the caller via
    /// `LegacyRandom::set_large_feature_seed(seed + carver_index, cx, cz)`
    /// and the `isStartChunk` probability check must have already passed.
    ///
    /// Mirrors vanilla's `CaveWorldCarver.carve`.
    pub fn carve_cave(
        &mut self,
        config: &CaveWorldCarver,
        source_pos: ChunkPos,
        random: &mut LegacyRandom,
    ) {
        let cave_count = config.count.sample(random);

        let source_min_x = source_pos.0.x * 16;
        let source_min_z = source_pos.0.y * 16;

        for _ in 0..cave_count {
            let x = f64::from(source_min_x + random.next_i32_bounded(16));
            let y = f64::from(config.y.sample(random, self.ctx.min_y, self.ctx.gen_depth));
            let z = f64::from(source_min_z + random.next_i32_bounded(16));

            let horizontal_radius_multiplier =
                f64::from(config.horizontal_radius_multiplier.sample(random));
            let vertical_radius_multiplier =
                f64::from(config.vertical_radius_multiplier.sample(random));
            let start_vertical_radius_multiplier =
                f64::from(config.start_vertical_radius_multiplier.sample(random));
            let floor_level = f64::from(config.floor_level.sample(random));

            // Vanilla `CaveWorldCarver.shouldSkip`: skip blocks below the
            // noisy floor OR outside the unit sphere in ellipsoid-local
            // coords (xd²+yd²+zd² ≥ 1). Without the sphere test we'd carve
            // cylinders, not ellipsoids.
            let skip_checker = move |xd: f64, yd: f64, zd: f64, _world_y: i32| {
                yd <= floor_level || xd * xd + yd * yd + zd * zd >= 1.0
            };

            let mut tunnels = 1i32;
            if random.next_i32_bounded(4) == 0 {
                let y_scale = f64::from(config.room_vertical_radius_multiplier.sample(random));
                let thickness = 1.0 + random.next_f32() * 6.0;
                self.create_room(x, y, z, thickness, y_scale, &skip_checker);
                tunnels += random.next_i32_bounded(4);
            }

            for _ in 0..tunnels {
                // Java evaluates these arguments left-to-right before calling
                // `createTunnel`; spell out each draw to keep that ordering
                // evident and stable.
                let horizontal_rotation = random.next_f32() * TAU;
                let vertical_rotation = (random.next_f32() - 0.5) / 4.0;
                let thickness = sample_tunnel_thickness(config, random);
                let distance =
                    MAX_TUNNEL_DISTANCE - random.next_i32_bounded(MAX_TUNNEL_DISTANCE / 4);
                let tunnel_seed = random.next_i64();
                let state = TunnelState {
                    x,
                    y,
                    z,
                    horizontal_rotation,
                    vertical_rotation,
                };
                let tunnel = TunnelParams {
                    tunnel_seed,
                    horizontal_radius_multiplier,
                    vertical_radius_multiplier,
                    thickness,
                    step: 0,
                    dist: distance,
                    y_scale: start_vertical_radius_multiplier,
                };
                self.create_tunnel(state, tunnel, skip_checker);
            }
        }
    }

    /// Vanilla `CaveWorldCarver.createRoom`. Single ellipsoid at the tunnel
    /// origin, offset by +1 on X.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors vanilla CaveWorldCarver.createRoom"
    )]
    fn create_room<S: CarveSkipChecker>(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        thickness: f32,
        y_scale: f64,
        skip_checker: S,
    ) {
        // Vanilla: `1.5 + Mth.sin((float)(Math.PI / 2)) * thickness`. The
        // argument is a float (π/2 cast to f32), looked up in the SIN table;
        // the result equals 1.0f exactly, so the table detour doesn't
        // matter here.
        let horizontal_radius =
            1.5 + f64::from(trig::sin(f64::from(FRAC_PI_2))) * f64::from(thickness);
        let vertical_radius = horizontal_radius * y_scale;
        self.carve_ellipsoid(
            x + 1.0,
            y,
            z,
            horizontal_radius,
            vertical_radius,
            skip_checker,
        );
    }

    /// Vanilla `CaveWorldCarver.createTunnel`. Steps along a curve, carving
    /// an ellipsoid per step, with occasional mid-tunnel splits.
    fn create_tunnel<S>(&mut self, mut state: TunnelState, tunnel: TunnelParams, skip_checker: S)
    where
        S: CarveSkipChecker + Copy,
    {
        let mut random = LegacyRandom::from_seed(tunnel.tunnel_seed as u64);
        let split_point = random.next_i32_bounded(tunnel.dist / 2) + tunnel.dist / 4;
        let steep = random.next_i32_bounded(6) == 0;
        let mut y_rota: f32 = 0.0;
        let mut x_rota: f32 = 0.0;

        for current_step in tunnel.step..tunnel.dist {
            // Vanilla: `Mth.sin((float)Math.PI * currentStep / dist) *
            // thickness`. The `(float)Math.PI * currentStep / dist` term
            // keeps float precision through to the `Mth.sin` argument before
            // widening to double.
            let progress_arg = PI * current_step as f32 / tunnel.dist as f32;
            let horizontal_radius = horizontal_tunnel_radius(progress_arg, tunnel.thickness);
            let vertical_radius = horizontal_radius * tunnel.y_scale;
            let cos_x = trig::cos(f64::from(state.vertical_rotation));
            state.x += f64::from(trig::cos(f64::from(state.horizontal_rotation)) * cos_x);
            state.y += f64::from(trig::sin(f64::from(state.vertical_rotation)));
            state.z += f64::from(trig::sin(f64::from(state.horizontal_rotation)) * cos_x);
            state.vertical_rotation *= if steep { 0.92 } else { 0.7 };
            state.vertical_rotation += x_rota * 0.1;
            state.horizontal_rotation += y_rota * 0.1;
            x_rota *= 0.9;
            y_rota *= 0.75;
            x_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
            y_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;

            if current_step == split_point && tunnel.thickness > 1.0 {
                // Vanilla evaluates args left-to-right: `nextLong()` (seed)
                // is arg 5, `nextFloat() * 0.5 + 0.5` (thickness) is arg 11
                // — so the seed is drawn before the thickness.
                let sub_seed_a = random.next_i64();
                let sub_thickness_a = random.next_f32() * 0.5 + 0.5;
                let sub_state_a = TunnelState {
                    horizontal_rotation: state.horizontal_rotation - FRAC_PI_2,
                    vertical_rotation: state.vertical_rotation / 3.0,
                    ..state
                };
                let sub_seed_b = random.next_i64();
                let sub_thickness_b = random.next_f32() * 0.5 + 0.5;
                let sub_state_b = TunnelState {
                    horizontal_rotation: state.horizontal_rotation + FRAC_PI_2,
                    vertical_rotation: state.vertical_rotation / 3.0,
                    ..state
                };
                let sub_tunnel_a = TunnelParams {
                    tunnel_seed: sub_seed_a,
                    thickness: sub_thickness_a,
                    step: current_step,
                    y_scale: 1.0,
                    ..tunnel
                };
                let sub_tunnel_b = TunnelParams {
                    tunnel_seed: sub_seed_b,
                    thickness: sub_thickness_b,
                    step: current_step,
                    y_scale: 1.0,
                    ..tunnel
                };
                self.create_tunnel(sub_state_a, sub_tunnel_a, skip_checker);
                self.create_tunnel(sub_state_b, sub_tunnel_b, skip_checker);
                return;
            }

            if random.next_i32_bounded(4) == 0 {
                continue;
            }

            if !can_reach(
                self.chunk_min_x,
                self.chunk_min_z,
                state.x,
                state.z,
                current_step,
                tunnel.dist,
                tunnel.thickness,
            ) {
                return;
            }

            self.carve_ellipsoid(
                state.x,
                state.y,
                state.z,
                horizontal_radius * tunnel.horizontal_radius_multiplier,
                vertical_radius * tunnel.vertical_radius_multiplier,
                skip_checker,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::carver::CaveWorldCarver;
    use steel_utils::random::{Random, legacy_random::LegacyRandom};
    use steel_utils::value_providers::{
        FloatProvider, HeightProvider, IntProvider, VerticalAnchor,
    };

    use super::sample_tunnel_thickness;

    fn cave_config(weird_thickness_bias: bool) -> CaveWorldCarver {
        CaveWorldCarver {
            probability: 1.0,
            y: HeightProvider::Constant(VerticalAnchor::Absolute(0)),
            count: IntProvider::Constant(1),
            thickness: FloatProvider::Constant(2.0),
            weird_thickness_bias,
            room_vertical_radius_multiplier: FloatProvider::Constant(1.0),
            horizontal_radius_multiplier: FloatProvider::Constant(1.0),
            vertical_radius_multiplier: FloatProvider::Constant(1.0),
            start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
            floor_level: FloatProvider::Constant(-1.0),
        }
    }

    #[test]
    fn weird_thickness_bias_uses_vanilla_conditional_draws() {
        let mut actual = LegacyRandom::from_seed(0);
        let thickness = sample_tunnel_thickness(&cave_config(true), &mut actual);

        let mut expected = LegacyRandom::from_seed(0);
        assert_eq!(expected.next_i32_bounded(10), 0);
        let expected_thickness = 2.0 * (expected.next_f32() * expected.next_f32() * 3.0 + 1.0);

        assert_eq!(thickness, expected_thickness);
        assert_eq!(actual.next_i32(), expected.next_i32());
    }

    #[test]
    fn ordinary_thickness_does_not_consume_bias_draws() {
        let mut actual = LegacyRandom::from_seed(0);
        assert_eq!(
            sample_tunnel_thickness(&cave_config(false), &mut actual),
            2.0
        );

        let mut expected = LegacyRandom::from_seed(0);
        assert_eq!(actual.next_i32(), expected.next_i32());
    }
}
