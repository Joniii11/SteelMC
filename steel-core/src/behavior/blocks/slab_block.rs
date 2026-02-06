//! Slab block behavior implementation.
//!
//! Slabs are half-height blocks that can be placed on the top or bottom
//! half of a block space, or combined into a double slab.

use std::ptr;

use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, SlabType};
use steel_registry::fluid::FluidState;
use steel_registry::items::ItemRef;
use steel_registry::{REGISTRY, vanilla_blocks, vanilla_fluids};
use steel_utils::BlockStateId;

use crate::behavior::block::BlockBehaviour;
use crate::behavior::context::{BlockPlaceContext, UseOnContext};
use crate::world::World;

/// Behavior for slab blocks.
///
/// Slabs have 2 properties:
/// - `type`: The type of slab (`top`, `bottom`, or `double`)
/// - `waterlogged`: Whether the slab is waterlogged (only for non-double slabs)
///
/// When placing a slab on an existing slab of the same type, they merge into
/// a double slab.
pub struct SlabBlock {
    block: BlockRef,
}

impl SlabBlock {
    /// Create a new slab block
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Checks if it is the same slab block.
    fn is_same_slab(&self, state: BlockStateId) -> bool {
        ptr::eq(state.get_block(), self.block)
    }

    /// Gets the item that corresponds to this slab block.
    fn get_slab_item(&self) -> Option<ItemRef> {
        REGISTRY.items.by_key(&self.block.key)
    }
}

impl BlockBehaviour for SlabBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let pos = &context.relative_pos;
        let existing_state = context.world.get_block_state(pos);

        if self.is_same_slab(existing_state) {
            let existing_type: SlabType =
                existing_state.get_value(&BlockStateProperties::SLAB_TYPE);
            if existing_type != SlabType::Double {
                return Some(
                    existing_state
                        .set_value(&BlockStateProperties::SLAB_TYPE, SlabType::Double)
                        .set_value(&BlockStateProperties::WATERLOGGED, false),
                );
            }
        }

        let waterlogged = ptr::eq(existing_state.get_block(), vanilla_blocks::WATER);

        // Determine slab type based on click position
        let clicked_face = context.clicked_face;
        let click_y = context.click_location.y;
        let pos_y = f64::from(pos.y());

        let slab_type = if clicked_face != Direction::Down
            && (clicked_face == Direction::Up || (click_y - pos_y) <= 0.5)
        {
            SlabType::Bottom
        } else {
            SlabType::Top
        };

        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::SLAB_TYPE, slab_type)
                .set_value(&BlockStateProperties::WATERLOGGED, waterlogged),
        )
    }

    fn get_fluid_state(&self, state: BlockStateId) -> FluidState {
        let slab_type: SlabType = state.get_value(&BlockStateProperties::SLAB_TYPE);
        if slab_type != SlabType::Double && state.get_value(&BlockStateProperties::WATERLOGGED) {
            FluidState::source(&vanilla_fluids::WATER)
        } else {
            FluidState::EMPTY
        }
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        _world: &World,
        _pos: steel_utils::BlockPos,
        _direction: Direction,
        _neighbor_pos: steel_utils::BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        // Slabs don't change shape based on neighbors
        // TODO: Schedule water tick if waterlogged (when tick system is implemented)
        state
    }

    fn can_be_replaced(
        &self,
        state: BlockStateId,
        context: &UseOnContext<'_>,
        placing_block: BlockRef,
        _placement_pos: steel_utils::BlockPos,
        replace_clicked: bool,
    ) -> bool {
        let slab_type: SlabType = state.get_value(&BlockStateProperties::SLAB_TYPE);

        if slab_type == SlabType::Double {
            return false;
        }

        let Some(slab_item) = self.get_slab_item() else {
            return false;
        };

        if !ptr::eq(context.item_stack.item, slab_item) || !ptr::eq(placing_block, self.block) {
            return false;
        }

        let hit = &context.hit_result;
        let clicked_face = hit.direction;
        let click_y = hit.location.y;
        let pos_y = f64::from(hit.block_pos.y());
        let clicked_upper_half = (click_y - pos_y) > 0.5;

        if replace_clicked {
            // directly clicking on the slab to merg it
            match slab_type {
                SlabType::Bottom => {
                    clicked_face == Direction::Up
                        || (clicked_upper_half && clicked_face.is_horizontal())
                }
                SlabType::Top => {
                    clicked_face == Direction::Down
                        || (!clicked_upper_half && clicked_face.is_horizontal())
                }
                SlabType::Double => false,
            }
        } else {
            // placing adjacent to a block into an existing slab position
            match clicked_face {
                Direction::Up => slab_type == SlabType::Top,
                Direction::Down => slab_type == SlabType::Bottom,
                _ => match slab_type {
                    SlabType::Bottom => clicked_upper_half,
                    SlabType::Top => !clicked_upper_half,
                    SlabType::Double => false,
                },
            }
        }
    }
}
