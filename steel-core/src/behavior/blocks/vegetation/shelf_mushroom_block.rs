use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::{
    blocks::{
        BlockRef,
        block_state_ext::BlockStateExt,
        properties::{BlockStateProperties, Direction},
    },
    entity_type::EntityTypeRef,
    sound_events, vanilla_blocks, vanilla_entities,
};
use steel_utils::{BlockPos, BlockStateId, types::UpdateFlags};

use crate::{
    behavior::{
        BlockBehavior, blocks::vegetation::bonemealable::Bonemealable, context::BlockPlaceContext,
    },
    entity::{Entity, ai::path::PathComputationType},
    world::{LevelAccessor, LevelReader, ScheduledTickAccess, World},
};

/// Vanilla `ShelfMushroomBlock`
#[block_behavior]
pub struct ShelfMushroomBlock {
    block: BlockRef,
}

impl ShelfMushroomBlock {
    /// Creates a new shelf mushroom block
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    fn plays_bounce_sound_for_entity_type(entity_type: EntityTypeRef) -> bool {
        entity_type != &vanilla_entities::ITEM
    }

    fn plays_bounce_sound_for(entity: &dyn Entity) -> bool {
        Self::plays_bounce_sound_for_entity_type(entity.entity_type())
    }
}

impl BlockBehavior for ShelfMushroomBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        for direction in context.get_nearest_looking_directions() {
            if !direction.is_horizontal() {
                continue;
            }

            let state = self
                .block
                .default_state()
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    direction.opposite(),
                )
                .set_value(&BlockStateProperties::AGE_1, 0u8);
            if self.can_survive(state, context.world, context.place_pos()) {
                return Some(state);
            }
        }

        None
    }

    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        let facing: Direction = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        let support_pos = pos.relative(facing.opposite());
        let support_state = world.get_block_state(support_pos);
        support_state.is_face_sturdy_at(support_pos, facing)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let facing: Direction = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        if direction == facing.opposite() && !self.can_survive(state, world, pos) {
            return vanilla_blocks::AIR.default_state();
        }

        state
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }

    fn bounce_on(
        &self,
        _state: BlockStateId,
        world: &dyn LevelAccessor,
        pos: BlockPos,
        entity: &dyn Entity,
        _fall_distance: f64,
    ) {
        if Self::plays_bounce_sound_for(entity) {
            world.play_block_sound(
                &sound_events::BLOCK_SHELF_MUSHROOM_BOUNCE,
                pos,
                1.0,
                1.0,
                None,
            );
        }
    }

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }
}

impl Bonemealable for ShelfMushroomBlock {
    fn is_valid_bonemeal_target(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
    ) -> bool {
        state.get_value(&BlockStateProperties::AGE_1) < 1u8
    }

    fn perform_bonemeal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        _rng: &mut dyn rand::Rng,
        pos: BlockPos,
    ) {
        let age: u8 = state.get_value(&BlockStateProperties::AGE_1);
        world.set_block(
            pos,
            state.set_value(&BlockStateProperties::AGE_1, age + 1),
            UpdateFlags::UPDATE_CLIENTS,
        );
    }
}
