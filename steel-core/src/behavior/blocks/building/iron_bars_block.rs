use crate::behavior::block::BlockBehavior;
use crate::behavior::context::BlockPlaceContext;
use crate::world::{ScheduledTickAccess, World};
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, BoolProperty, Direction};
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_blocks;
use steel_utils::{BlockPos, BlockStateId};

use std::sync::Arc;

/// Vanilla iron bars connection and waterlogging behavior.
#[block_behavior]
pub struct IronBarsBlock {
    block: BlockRef,
}

impl IronBarsBlock {
    /// North connection property.
    pub const NORTH: BoolProperty = BlockStateProperties::NORTH;
    /// East connection property.
    pub const EAST: BoolProperty = BlockStateProperties::EAST;
    /// South connection property.
    pub const SOUTH: BoolProperty = BlockStateProperties::SOUTH;
    /// West connection property.
    pub const WEST: BoolProperty = BlockStateProperties::WEST;
    /// Waterlogged property.
    pub const WATERLOGGED: BoolProperty = BlockStateProperties::WATERLOGGED;

    /// Creates a new iron bars behavior for the given block.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    fn is_exception_for_connection(state: BlockStateId) -> bool {
        let block = state.get_block();
        block.has_tag(&BlockTag::LEAVES)
            || block == &vanilla_blocks::BARRIER
            || block == &vanilla_blocks::CARVED_PUMPKIN
            || block == &vanilla_blocks::JACK_O_LANTERN
            || block == &vanilla_blocks::MELON
            || block == &vanilla_blocks::PUMPKIN
            || block.has_tag(&BlockTag::SHULKER_BOXES)
    }

    fn attaches_to(
        &self,
        neighbor_state: BlockStateId,
        neighbor_pos: BlockPos,
        face_solid: Direction,
    ) -> bool {
        let neighbor_block = neighbor_state.get_block();
        !Self::is_exception_for_connection(neighbor_state)
            && neighbor_state.is_face_sturdy_at(neighbor_pos, face_solid)
            || neighbor_block == self.block
            || neighbor_block.has_tag(&BlockTag::WALLS)
    }

    fn get_connection_state(&self, world: &Arc<World>, pos: BlockPos) -> BlockStateId {
        let north_pos = pos.north();
        let south_pos = pos.south();
        let west_pos = pos.west();
        let east_pos = pos.east();
        let north_state = world.get_block_state(north_pos);
        let south_state = world.get_block_state(south_pos);
        let west_state = world.get_block_state(west_pos);
        let east_state = world.get_block_state(east_pos);

        self.block
            .default_state()
            .set_value(
                &Self::NORTH,
                self.attaches_to(north_state, north_pos, Direction::South),
            )
            .set_value(
                &Self::SOUTH,
                self.attaches_to(south_state, south_pos, Direction::North),
            )
            .set_value(
                &Self::WEST,
                self.attaches_to(west_state, west_pos, Direction::East),
            )
            .set_value(
                &Self::EAST,
                self.attaches_to(east_state, east_pos, Direction::West),
            )
    }
}

impl BlockBehavior for IronBarsBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.get_connection_state(context.world, context.relative_pos)
                .set_value(&Self::WATERLOGGED, context.is_water_source()),
        )
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        _world: &dyn ScheduledTickAccess,
        _pos: BlockPos,
        direction: Direction,
        neighbor_pos: BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        match direction {
            Direction::North => state.set_value(
                &Self::NORTH,
                self.attaches_to(neighbor_state, neighbor_pos, Direction::South),
            ),
            Direction::East => state.set_value(
                &Self::EAST,
                self.attaches_to(neighbor_state, neighbor_pos, Direction::West),
            ),
            Direction::South => state.set_value(
                &Self::SOUTH,
                self.attaches_to(neighbor_state, neighbor_pos, Direction::North),
            ),
            Direction::West => state.set_value(
                &Self::WEST,
                self.attaches_to(neighbor_state, neighbor_pos, Direction::East),
            ),
            Direction::Up | Direction::Down => state,
        }
    }
}
