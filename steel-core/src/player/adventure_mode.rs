//! Adventure mode item predicate evaluation

use std::cmp::Ordering;

use steel_registry::{
    blocks::{BlockRef, block_state_ext::BlockStateExt},
    data_components::{
        components::{
            AdventureModePredicate, BlockHolderSet, BlockPredicate, StatePropertiesPredicate,
            StatePropertyValueMatcher,
        },
        vanilla_components::{CAN_BREAK, CAN_PLACE_ON},
    },
    item_stack::ItemStack,
};
use steel_utils::{BlockPos, BlockStateId, nbt::compare_nbt_compounds, types::GameType};

use crate::{
    player::Player,
    world::{LevelReader, World},
};

/// Returns whether `item` may be used on `pos` while building is restricted
#[must_use]
pub(crate) fn can_place_on(item: &ItemStack, world: &World, pos: BlockPos) -> bool {
    item.get(CAN_PLACE_ON)
        .is_some_and(|predicate| matches(predicate, world, pos))
}

/// Returns whether `item` may break the block at `pos` while building is restricted
#[must_use]
pub(crate) fn can_break(item: &ItemStack, world: &World, pos: BlockPos) -> bool {
    item.get(CAN_BREAK)
        .is_some_and(|predicate| matches(predicate, world, pos))
}

/// Mirrors Vanilla `Player.blockActionRestricted` for block destruction
#[must_use]
pub(crate) fn block_action_restricted(player: &Player, world: &World, pos: BlockPos) -> bool {
    match player.game_mode() {
        GameType::Survival | GameType::Creative => false,
        GameType::Spectator => true,
        GameType::Adventure => {
            if player.get_abilities().may_build {
                return false;
            }

            let inventory = player.inventory.lock();
            let item = inventory.get_selected_item();
            item.is_empty() || !can_break(item, world, pos)
        }
    }
}

fn matches(predicate: &AdventureModePredicate, world: &impl LevelReader, pos: BlockPos) -> bool {
    predicate
        .predicates
        .iter()
        .any(|predicate| block_predicate_matches(predicate, world, pos))
}

fn block_predicate_matches(
    predicate: &BlockPredicate,
    world: &impl LevelReader,
    pos: BlockPos,
) -> bool {
    let state = world.get_block_state(pos);
    if !state_matches(predicate, state) {
        return false;
    }

    let Some(nbt) = &predicate.nbt else {
        return true;
    };
    let Some(block_entity) = world.get_block_entity(pos) else {
        return false;
    };

    let actual = block_entity.lock().save_with_full_metadata();
    compare_nbt_compounds(&nbt.tag, &actual, true)
}

fn state_matches(predicate: &BlockPredicate, state: BlockStateId) -> bool {
    let block = state.get_block();
    if let Some(blocks) = &predicate.blocks
        && !block_holder_set_matches(blocks, block)
    {
        return false;
    }

    predicate
        .properties
        .as_ref()
        .is_none_or(|properties| state_properties_match(properties, state))
}

fn block_holder_set_matches(blocks: &BlockHolderSet, block: BlockRef) -> bool {
    match blocks {
        BlockHolderSet::Tag(tag) => block.has_tag(tag),
        BlockHolderSet::Blocks(blocks) => blocks.iter().any(|candidate| candidate.key == block.key),
    }
}

fn state_properties_match(properties: &StatePropertiesPredicate, state: BlockStateId) -> bool {
    let block = state.get_block();

    properties.properties.iter().all(|matcher| {
        let Some(property) = block
            .properties
            .iter()
            .find(|property| property.get_name() == matcher.name.as_str())
        else {
            return false;
        };
        let Some(actual) = state.get_property_str(&matcher.name) else {
            return false;
        };

        match &matcher.value {
            StatePropertyValueMatcher::Exact(expected) => property
                .compare_values(&actual, expected)
                .is_some_and(|comparison| comparison == Ordering::Equal),
            StatePropertyValueMatcher::Ranged { min, max } => {
                let minimum_matches = min.as_ref().is_none_or(|minimum| {
                    property
                        .compare_values(&actual, minimum)
                        .is_some_and(|comparison| comparison != Ordering::Less)
                });
                let maximum_matches = max.as_ref().is_none_or(|maximum| {
                    property
                        .compare_values(&actual, maximum)
                        .is_some_and(|comparison| comparison != Ordering::Greater)
                });
                minimum_matches && maximum_matches
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use simdnbt::owned::{NbtCompound, NbtList};
    use steel_registry::{
        REGISTRY,
        data_components::components::{
            AdventureModePredicate, BlockHolderSet, BlockPredicate, DataComponentExactPredicate,
            DataComponentMatchers, ExactDataComponentPredicate, StatePropertiesPredicate,
            StatePropertyMatcher, StatePropertyValueMatcher,
        },
        data_components::{ComponentData, vanilla_components::MAX_DAMAGE},
        test_support::init_test_registry,
        vanilla_blocks,
    };

    use super::{matches, state_properties_match};
    use crate::world::LevelReader;
    use steel_utils::{BlockPos, BlockStateId, nbt::compare_nbt_compounds};

    struct TestLevel {
        state: BlockStateId,
    }

    impl LevelReader for TestLevel {
        fn get_block_state(&self, _pos: BlockPos) -> BlockStateId {
            self.state
        }

        fn raw_brightness(&self, _pos: BlockPos, _sky_darkening: u8) -> u8 {
            0
        }

        fn min_y(&self) -> i32 {
            -64
        }

        fn height(&self) -> i32 {
            384
        }
    }

    #[test]
    fn nbt_match_uses_vanilla_partial_compound_and_list_semantics() {
        let mut expected_nested = NbtCompound::new();
        expected_nested.insert("required", 4_i32);

        let mut expected = NbtCompound::new();
        expected.insert("nested", expected_nested);
        expected.insert("values", NbtList::Int(vec![2, 3]));

        let mut actual_nested = NbtCompound::new();
        actual_nested.insert("required", 4_i32);
        actual_nested.insert("extra", 9_i32);

        let mut actual = NbtCompound::new();
        actual.insert("nested", actual_nested);
        actual.insert("values", NbtList::Int(vec![1, 2, 3]));
        actual.insert("extra", "allowed");

        assert!(compare_nbt_compounds(&expected, &actual, true));
    }

    #[test]
    fn empty_expected_list_requires_an_empty_actual_list() {
        let mut expected = NbtCompound::new();
        expected.insert("values", NbtList::Int(Vec::new()));

        let mut actual = NbtCompound::new();
        actual.insert("values", NbtList::Int(vec![1]));

        assert!(!compare_nbt_compounds(&expected, &actual, true));
    }

    #[test]
    fn state_property_ranges_use_the_property_value_order() {
        init_test_registry();
        let Some(state) = REGISTRY
            .blocks
            .state_id_from_block_defaulted_properties(&vanilla_blocks::WATER, [("level", "10")])
        else {
            panic!("water level property should be registered");
        };

        let matching = StatePropertiesPredicate {
            properties: vec![StatePropertyMatcher {
                name: "level".into(),
                value: StatePropertyValueMatcher::Ranged {
                    min: Some("2".into()),
                    max: Some("12".into()),
                },
            }],
        };
        let non_matching = StatePropertiesPredicate {
            properties: vec![StatePropertyMatcher {
                name: "level".into(),
                value: StatePropertyValueMatcher::Ranged {
                    min: Some("11".into()),
                    max: None,
                },
            }],
        };

        assert!(state_properties_match(&matching, state));
        assert!(!state_properties_match(&non_matching, state));
    }

    #[test]
    fn block_in_world_adventure_predicates_ignore_component_matchers() {
        init_test_registry();
        let Some(oak_log_state) = REGISTRY
            .blocks
            .state_id_from_block_defaulted_properties(&vanilla_blocks::OAK_LOG, [("axis", "y")])
        else {
            panic!("oak log axis property should be registered");
        };

        let predicate = AdventureModePredicate {
            predicates: vec![BlockPredicate {
                blocks: Some(BlockHolderSet::Blocks(vec![&vanilla_blocks::OAK_LOG])),
                properties: Some(StatePropertiesPredicate {
                    properties: vec![StatePropertyMatcher {
                        name: "axis".into(),
                        value: StatePropertyValueMatcher::Exact("y".into()),
                    }],
                }),
                nbt: None,
                components: DataComponentMatchers {
                    exact: DataComponentExactPredicate {
                        components: vec![ExactDataComponentPredicate {
                            component: MAX_DAMAGE.key.clone(),
                            value: ComponentData::I32(1),
                        }],
                    },
                    partial: Vec::new(),
                },
            }],
        };

        assert!(matches(
            &predicate,
            &TestLevel {
                state: oak_log_state,
            },
            BlockPos::ZERO,
        ));
        assert!(!matches(
            &predicate,
            &TestLevel {
                state: vanilla_blocks::STONE.default_state(),
            },
            BlockPos::ZERO,
        ));
    }
}
