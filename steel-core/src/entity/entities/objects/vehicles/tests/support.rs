//! Fixtures shared by the vehicle tests.

use std::sync::Arc;

use glam::DVec3;
use steel_registry::vanilla_entities;
use steel_utils::{BlockPos, BlockStateId, ChunkPos, types::UpdateFlags};

use super::{MinecartEntity, TntMinecartEntity};
use crate::bootstrap::init_globals;
use crate::entity::{EntityEventSource as _, next_entity_id};
use crate::test_support::{TestWorld, fresh_test_world, insert_ready_full_chunk};
use crate::world::World;

pub(super) const CART_BLOCK_POS: BlockPos = BlockPos::new(8, 64, 8);

pub(super) fn set_block(world: &Arc<World>, pos: BlockPos, state: BlockStateId) {
    let flags = UpdateFlags::UPDATE_NONE
        | UpdateFlags::UPDATE_KNOWN_SHAPE
        | UpdateFlags::UPDATE_SKIP_ON_PLACE;
    assert!(world.set_block(pos, state, flags));
}

pub(crate) fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-12,
        "expected {expected}, got {actual}",
    );
}

pub(crate) fn assert_vec_close(actual: DVec3, expected: DVec3) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.z, expected.z);
}

/// A fresh test world with `chunk` loaded.
pub(crate) fn test_world(key: &'static str, chunk: ChunkPos) -> TestWorld {
    init_globals();

    let fixture = fresh_test_world(key);
    insert_ready_full_chunk(&fixture.world, chunk);
    fixture
}

/// A minecart added to `world` at `(8.5, 64, 8.5)`.
pub(crate) fn spawned_minecart(world: &Arc<World>) -> Arc<MinecartEntity> {
    let cart = Arc::new(MinecartEntity::new(
        &vanilla_entities::MINECART,
        next_entity_id(),
        CART_BLOCK_POS.get_bottom_center(),
        Arc::downgrade(world),
    ));

    world
        .try_add_entity(Arc::clone(&cart).into_shared_entity())
        .expect("test minecart should enter the loaded chunk");

    cart
}

pub(super) fn spawned_tnt_minecart(world: &Arc<World>) -> Arc<TntMinecartEntity> {
    let cart = Arc::new(TntMinecartEntity::new(
        &vanilla_entities::TNT_MINECART,
        next_entity_id(),
        CART_BLOCK_POS.get_bottom_center(),
        Arc::downgrade(world),
    ));
    world
        .try_add_entity(Arc::clone(&cart).into_shared_entity())
        .expect("test TNT minecart should enter the loaded chunk");
    cart
}
