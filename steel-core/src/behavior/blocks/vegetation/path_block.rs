use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::{
    blocks::{BlockRef, block_state_ext::BlockStateExt},
    vanilla_block_tags::BlockTag,
    vanilla_game_events,
};
use steel_utils::{BlockPos, BlockStateId, Direction};

use crate::{
    behavior::{
        BlockBehavior, block::push_entities_up, blocks::vegetation::default_surviving_state,
        context::BlockPlaceContext,
    },
    entity::Entity,
    entity::ai::path::PathComputationType,
    world::{ScheduledTickAccess, World, game_event_context::GameEventContext},
};

/// Vanilla `PathBlock`, used by `dirt_path`.
#[block_behavior]
pub struct PathBlock {
    block: BlockRef,
    #[json_arg(vanilla_blocks, json = "base_block")]
    base_block: BlockRef,
}

impl PathBlock {
    /// Creates a path block behavior with its vanilla base block.
    #[must_use]
    pub const fn new(block: BlockRef, base_block: BlockRef) -> Self {
        Self { block, base_block }
    }

    fn turn_to_base_block(
        &self,
        source_entity: Option<&dyn Entity>,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
    ) {
        let new_state = push_entities_up(state, self.base_block.default_state(), world, pos);
        if world.set_block_and_update(pos, new_state) {
            world.game_event(
                &vanilla_game_events::BLOCK_CHANGE,
                pos,
                &GameEventContext::new(source_entity, Some(new_state)),
            );
        }
    }
}

impl BlockBehavior for PathBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        default_surviving_state(self.block, self, context).or_else(|| {
            Some(push_entities_up(
                self.block.default_state(),
                self.base_block.default_state(),
                context.world,
                context.place_pos,
            ))
        })
    }

    fn can_survive(
        &self,
        _state: BlockStateId,
        world: &dyn crate::world::LevelReader,
        pos: BlockPos,
    ) -> bool {
        let above = world.get_block_state(pos.above());
        !above.is_solid() || above.get_block().has_tag(&BlockTag::FENCE_GATES)
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
        if direction == Direction::Up && !self.can_survive(state, world, pos) {
            let _ = world.schedule_block_tick_default(pos, self.block, 1);
        }

        state
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.turn_to_base_block(None, state, world, pos);
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use steel_registry::{test_support::init_test_registry, vanilla_blocks};

    #[test]
    fn path_block_is_not_pathfindable() {
        init_test_registry();
        let behavior = PathBlock::new(&vanilla_blocks::DIRT_PATH, &vanilla_blocks::DIRT);
        let state = vanilla_blocks::DIRT_PATH.default_state();

        assert!(!behavior.is_pathfindable(state, PathComputationType::Land));
        assert!(!behavior.is_pathfindable(state, PathComputationType::Air));
        assert!(!behavior.is_pathfindable(state, PathComputationType::Water));
    }
}
