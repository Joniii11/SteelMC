use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_utils::{BlockPos, BlockStateId};

use crate::{
    behavior::{BlockBehavior, BlockPlaceContext, EntityFallDamage, EntityFallOnContext},
    world::World,
};

/// Behavior for beds.
///
/// TODO: Add two-block placement, bed block entities, sleep interaction, and
/// invalid-dimension explosion behavior with the rest of the bed system.
#[block_behavior]
pub struct BedBlock {
    block: BlockRef,
}

impl BedBlock {
    /// Creates a bed block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    #[must_use]
    fn fall_context(context: EntityFallOnContext<'_>) -> EntityFallOnContext<'_> {
        context.with_fall_distance(context.fall_distance * 0.5)
    }
}

impl BlockBehavior for BedBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn fall_on(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        context: EntityFallOnContext<'_>,
    ) -> Option<EntityFallDamage> {
        self.default_fall_on(state, world, pos, Self::fall_context(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use steel_registry::{sound_events, vanilla_entities};

    use crate::behavior::EntityFallOnFacts;

    #[test]
    fn bed_halves_fall_distance_before_default_damage() {
        let context = BedBlock::fall_context(EntityFallOnContext::new(
            12.0,
            false,
            EntityFallOnFacts::new(
                &vanilla_entities::PLAYER,
                true,
                0.6,
                1.8,
                (
                    &sound_events::ENTITY_PLAYER_SMALL_FALL,
                    &sound_events::ENTITY_PLAYER_BIG_FALL,
                ),
            ),
            None,
        ));

        assert!((context.fall_distance - 6.0).abs() < f64::EPSILON);
        assert!(!context.suppresses_bounce);
        assert!(context.entity.is_player());
    }
}
