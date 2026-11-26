//! Three kinds of unoccupied cart stacks on one ordinary rail. Each mode starts in a fresh
//! ticking world and runs one settling tick, five timed ticks, then a separate diagnostic tick.
//! Timing comes from the complete synchronous world tick. Cart inspection runs after the timer.
//! The reference uses the original ordered contact loop. The optimized pass uses the default
//! contact path. Both builds of the same fixture must agree after every tick.

use glam::{DVec3, Vec3Swizzles as _};
use serde_json::json;
use steel_registry::vanilla_blocks;
use steel_utils::{ChunkPos, Downcast as _};

use super::{
    TntMinecartEntity,
    minecart_behavior::measure_cart_stack_skips,
    test_support::{CART_BLOCK_POS, set_block, spawned_minecart, spawned_tnt_minecart, test_world},
};
use crate::{
    chunk::{chunk_holder::TickingReadiness, chunk_ticket_manager::ChunkTicketLevel},
    entity::{Entity, EntityEventSource as _, SharedEntity},
    test_support::{TestWorld, advance_test_game_time_to},
};

const SAMPLES: i32 = 5;

#[derive(Clone, Copy, Debug)]
enum Stack {
    Ordinary,
    Mixed,
    Tnt,
}

impl Stack {
    fn spawn(self, count: usize) -> (TestWorld, Vec<SharedEntity>) {
        let chunk = ChunkPos::from_block_pos(CART_BLOCK_POS);
        let fixture = test_world("stationary_minecart_benchmark", chunk);
        let world = &fixture.world;
        let holder = world
            .chunk_map
            .active_full_chunk_holder(chunk)
            .expect("fixture chunk");
        holder.set_simulation_level(Some(ChunkTicketLevel::ENTITY_TICKING_CHUNK));
        assert_eq!(
            holder.transition_ticking_readiness(TickingReadiness::EntityTicking),
            Some(TickingReadiness::BlockTicking)
        );
        world.update_entity_chunk_visibility(chunk, holder.entity_visibility());
        world.chunk_map.rebuild_ticking_chunk_snapshot();
        set_block(
            world,
            CART_BLOCK_POS.below(),
            vanilla_blocks::STONE.default_state(),
        );
        set_block(world, CART_BLOCK_POS, vanilla_blocks::RAIL.default_state());
        let carts = (0..count)
            .map(|index| {
                let tnt = match self {
                    Self::Ordinary => false,
                    Self::Mixed => index % 2 == 1,
                    Self::Tnt => true,
                };
                if tnt {
                    spawned_tnt_minecart(world).into_shared_entity()
                } else {
                    spawned_minecart(world).into_shared_entity()
                }
            })
            .collect();
        (fixture, carts)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CartState {
    position: [u64; 3],
    velocity: [u64; 3],
    rotation: [u32; 2],
    horizontal_collision: bool,
    needs_velocity_sync: bool,
}

impl CartState {
    fn check_and_capture(cart: &dyn Entity, tick: i32, settled: DVec3) -> Self {
        assert_eq!(cart.tick_count(), tick);
        assert!(cart.is_on_rails() && !cart.is_removed());
        assert!(!cart.is_passenger() && !cart.is_vehicle());
        let position = cart.position();
        let velocity = cart.velocity();
        assert_eq!(position, settled);
        assert_eq!(velocity, DVec3::ZERO);
        if let Some(tnt) = cart.downcast_ref::<TntMinecartEntity>() {
            assert_eq!(tnt.fuse(), None);
        }
        let (yaw, pitch) = cart.rotation();
        Self {
            position: position.to_array().map(f64::to_bits),
            velocity: velocity.to_array().map(f64::to_bits),
            rotation: [yaw.to_bits(), pitch.to_bits()],
            horizontal_collision: cart.horizontal_collision(),
            needs_velocity_sync: cart.needs_velocity_sync(),
        }
    }
}

#[expect(
    clippy::print_stdout,
    reason = "emit machine-readable benchmark results"
)]
fn measure(stack: Stack, count: usize, optimized: bool) -> Vec<Vec<CartState>> {
    let (fixture, carts) = stack.spawn(count);
    let world = &fixture.world;
    let mut snapshots = Vec::new();
    let mut entities_ns = Vec::new();
    let mut world_ns = Vec::new();
    let mut diagnostic_skips = 0;
    let mut settled = None;
    let diagnostic_tick = SAMPLES + 2;
    for tick in 1..=diagnostic_tick {
        advance_test_game_time_to(world, i64::from(tick));
        let tick_world = || world.tick_game(tick as u64, true);
        let timings = if tick == diagnostic_tick {
            let (timings, skipped) = measure_cart_stack_skips(optimized, tick_world);
            assert_eq!(skipped, if optimized { count } else { 0 });
            diagnostic_skips = skipped;
            timings
        } else if optimized {
            tick_world()
        } else {
            measure_cart_stack_skips(false, tick_world).0
        };
        if tick > 1 && tick < diagnostic_tick {
            entities_ns.push(timings.entity_tick.as_nanos());
            world_ns.push(timings.elapsed.as_nanos());
        }
        // The first tick projects the identical spawn positions onto the rail.
        let position = *settled.get_or_insert_with(|| carts[0].position());
        assert_eq!(position.xz(), CART_BLOCK_POS.get_bottom_center().xz());
        snapshots.push(
            carts
                .iter()
                .map(|cart| CartState::check_and_capture(cart.as_ref(), tick, position))
                .collect(),
        );
    }
    world.chunk_map.stop_generation_refill_loop();
    println!(
        "{}",
        json!({
            "stack": format!("{stack:?}"), "count": count,
            "mode": if optimized { "optimized" } else { "reference" },
            "potential_other_carts": count * (count - 1),
            "entities_ns": entities_ns, "world_ns": world_ns,
            "diagnostic_skips": diagnostic_skips,
            "diagnostic_fallbacks": count - diagnostic_skips,
        })
    );
    snapshots
}

#[test]
#[ignore = "release world tick benchmark"]
fn stationary_minecart_benchmark() {
    for stack in [Stack::Ordinary, Stack::Mixed, Stack::Tnt] {
        for count in [270, 900, 2700] {
            let reference = measure(stack, count, false);
            let actual = measure(stack, count, true);
            for (tick, (a, b)) in actual.iter().zip(&reference).enumerate() {
                for (slot, (left, right)) in a.iter().zip(b).enumerate() {
                    assert_eq!(
                        left,
                        right,
                        "{stack:?} count {count} tick {} cart {slot}",
                        tick + 1
                    );
                }
            }
        }
    }
}
