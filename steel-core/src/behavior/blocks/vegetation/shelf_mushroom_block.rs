use steel_macros::block_behavior;
use steel_registry::{
    blocks::{
        BlockRef,
        block_state_ext::BlockStateExt,
        properties::{BlockStateProperties, Direction},
    },
    vanilla_blocks,
};
use steel_utils::{BlockPos, BlockStateId};

use crate::{
    behavior::{BlockBehavior, context::BlockPlaceContext},
    world::{LevelReader, ScheduledTickAccess},
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
}

impl BlockBehavior for ShelfMushroomBlock {
    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        let facing: Direction = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        let support_pos = pos.relative(facing.opposite());
        let support_state = world.get_block_state(support_pos);
        support_state.is_face_sturdy_at(support_pos, facing)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        for direction in context.get_nearest_looking_directions() {
            if !direction.is_horizontal() {
                continue;
            }

            let state = self.block.default_state().set_value(
                &BlockStateProperties::HORIZONTAL_FACING,
                direction.opposite(),
            );
            if self.can_survive(state, context.world, context.place_pos) {
                return Some(state);
            }
        }

        None
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
}
