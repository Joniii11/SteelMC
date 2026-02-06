//! Stair block behavior implementation.
//!
//! Stairs connect to adjacent stairs to form corners (inner and outer).

use std::ptr;

use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, Half, StairsShape};
use steel_registry::fluid::FluidState;
use steel_registry::{vanilla_blocks, vanilla_fluids};
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehaviour;
use crate::behavior::context::BlockPlaceContext;
use crate::world::World;

/// Behavior for stair blocks.
///
/// Stairs have 4 properties:
/// - `facing`: The horizontal direction the stairs face (north, south, east, west)
/// - `half`: Whether the stairs are on the top or bottom half of the block
/// - `shape`: The shape of the stairs (`straight`, `inner_left`, `inner_right`, `outer_left`, `outer_right`)
/// - `waterlogged`: Whether the stairs are waterlogged
///
/// The shape is determined by adjacent stair blocks to form corners.
pub struct StairBlock {
    block: BlockRef,
}

impl StairBlock {
    /// Creates a new stair block behavior for the given block.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Checks if a block state is a stair block.
    ///
    /// A block is considered a stair if it has the `STAIRS_SHAPE` property.
    fn is_stairs(state: BlockStateId) -> bool {
        state
            .try_get_value(&BlockStateProperties::STAIRS_SHAPE)
            .is_some()
    }

    /// Determines the shape of the stairs based on adjacent blocks.
    fn get_stairs_shape(state: BlockStateId, world: &World, pos: &BlockPos) -> StairsShape {
        let facing: Direction = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        let half: Half = state.get_value(&BlockStateProperties::HALF);

        // Check the block behind (in the facing direction) and do some other stuff
        let behind_pos = facing.relative(pos);
        let behind_state = world.get_block_state(&behind_pos);

        if Self::is_stairs(behind_state) {
            let behind_half: Half = behind_state.get_value(&BlockStateProperties::HALF);
            if half == behind_half {
                let behind_facing: Direction =
                    behind_state.get_value(&BlockStateProperties::HORIZONTAL_FACING);

                if behind_facing.axis() != facing.axis()
                    && Self::can_take_shape(state, world, pos, behind_facing.opposite())
                {
                    if behind_facing == facing.rotate_y_counter_clockwise() {
                        return StairsShape::OuterLeft;
                    }
                    return StairsShape::OuterRight;
                }
            }
        }

        let front_pos = facing.opposite().relative(pos);
        let front_state = world.get_block_state(&front_pos);

        if Self::is_stairs(front_state) {
            let front_half: Half = front_state.get_value(&BlockStateProperties::HALF);
            if half == front_half {
                let front_facing: Direction =
                    front_state.get_value(&BlockStateProperties::HORIZONTAL_FACING);

                if front_facing.axis() != facing.axis()
                    && Self::can_take_shape(state, world, pos, front_facing)
                {
                    if front_facing == facing.rotate_y_counter_clockwise() {
                        return StairsShape::InnerLeft;
                    }
                    return StairsShape::InnerRight;
                }
            }
        }

        StairsShape::Straight
    }

    /// Checks if this stair can take a corner shape by verifying the neighbor
    /// in the given direction doesn't block it.
    fn can_take_shape(
        state: BlockStateId,
        world: &World,
        pos: &BlockPos,
        neighbor_direction: Direction,
    ) -> bool {
        let neighbor_pos = neighbor_direction.relative(pos);
        let neighbor_state = world.get_block_state(&neighbor_pos);

        if !Self::is_stairs(neighbor_state) {
            return true;
        }

        let our_facing: Direction = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        let our_half: Half = state.get_value(&BlockStateProperties::HALF);
        let neighbor_facing: Direction =
            neighbor_state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        let neighbor_half: Half = neighbor_state.get_value(&BlockStateProperties::HALF);

        // Can take shape if the neighbor stair has a different facing or half
        neighbor_facing != our_facing || neighbor_half != our_half
    }
}

impl BlockBehaviour for StairBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let clicked_face = context.clicked_face;
        let click_y = context.click_location.y;
        let pos_y = f64::from(context.relative_pos.y());

        // Determine if stairs should be on top or bottom half
        let half = if clicked_face != Direction::Down
            && (clicked_face == Direction::Up || (click_y - pos_y) <= 0.5)
        {
            Half::Bottom
        } else {
            Half::Top
        };

        // Check if position is waterlogged (replacing water block)
        let existing_state = context.world.get_block_state(&context.relative_pos);
        let waterlogged = ptr::eq(existing_state.get_block(), vanilla_blocks::WATER);

        // Build the initial state
        let state = self
            .block
            .default_state()
            .set_value(
                &BlockStateProperties::HORIZONTAL_FACING,
                context.horizontal_direction,
            )
            .set_value(&BlockStateProperties::HALF, half)
            .set_value(&BlockStateProperties::WATERLOGGED, waterlogged);

        // Calculate the shape based on neighbors
        let shape = Self::get_stairs_shape(state, context.world, &context.relative_pos);
        Some(state.set_value(&BlockStateProperties::STAIRS_SHAPE, shape))
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &World,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        // Only update shape for horizontal neighbor changes
        if direction.is_horizontal() {
            let shape = Self::get_stairs_shape(state, world, &pos);
            state.set_value(&BlockStateProperties::STAIRS_SHAPE, shape)
        } else {
            state
        }
    }

    fn get_fluid_state(&self, state: BlockStateId) -> FluidState {
        if state.get_value(&BlockStateProperties::WATERLOGGED) {
            FluidState::source(&vanilla_fluids::WATER)
        } else {
            FluidState::EMPTY
        }
    }
}
