use steel_registry::{
    REGISTRY, RegistryExt,
    blocks::{
        block_state_ext::BlockStateExt,
        properties::BlockStateProperties,
        shapes::{OffsetVoxelShape, VoxelShape},
    },
    data_components::components::{
        BlockTransformData, DropStrategy, TransformBlockState, TransformHolderSet,
        TransformNoiseParameters, TransformParticle, TransformPredicate, TransformStateProvider,
        TransformStateProviderRule, TransformType, WeightedTransformBlockState,
    },
    data_components::vanilla_components::{BLOCK_TRANSFORMER, BLOCKS_ATTACKS},
    item_stack::ItemStack,
    level_events::{PARTICLES_SCRAPE, PARTICLES_WAX_OFF, PARTICLES_WAX_ON},
    loot_table::{
        BlockEntityRef, EntityRef, EntityRefFlags, LootContext, LootRandom, LootTableRef,
        VanillaLootRandom,
    },
    vanilla_game_events,
};
use steel_utils::{
    BlockPos, BlockStateId, Identifier,
    axis::Axis,
    random::{Random, RandomSource, legacy_random::LegacyRandom},
    types::{InteractionHand, UpdateFlags},
};
use steel_worldgen::noise::NormalNoise;

use crate::{
    behavior::{BLOCK_BEHAVIORS, InteractionResult, UseOnContext},
    entity::{Entity, LivingEntity},
    inventory::lock::ContainerLockGuard,
    world::game_event::GameEventContext,
};

use super::copper_chest_events::{emit_connected_chest_block_change, is_copper_chest};

pub(super) fn use_on(context: &mut UseOnContext) -> InteractionResult {
    if player_has_blocking_item_use_intent(context) {
        return InteractionResult::Pass;
    }

    let transformer = context
        .inv
        .with_item(|item| item.get(BLOCK_TRANSFORMER).cloned());
    let Some(transformer) = transformer else {
        return InteractionResult::Pass;
    };

    for transform in &transformer.transforms {
        if transform
            .disallowed_faces
            .contains(&context.hit_result.direction)
        {
            continue;
        }

        let pos = context.hit_result.block_pos;
        // `Level.getRandom` provider scope
        let Some(new_state) = context.world.with_random(|random| {
            transform_provider_optional(&transform.block_state_provider, context, pos, random)
        }) else {
            continue;
        };
        let old_state = context.world.get_block_state(pos);
        let new_state = if transform.transform_type == TransformType::CopperChest {
            new_state
        } else {
            context
                .world
                .update_from_neighbor_shapes(new_state, context.hit_result.block_pos)
        };

        drop_loot(transform, context, old_state);
        consume_transform_use(transform, context);
        context.world.set_block(
            context.hit_result.block_pos,
            new_state,
            UpdateFlags::UPDATE_ALL_IMMEDIATE,
        );

        context.world.play_block_sound_holder(
            transform.sound.clone(),
            context.hit_result.block_pos,
            1.0,
            1.0,
            Some(context.player.id()),
        );
        let particle_event = particle_event(transform.particle);
        if let Some(event) = particle_event {
            context.world.level_event(
                event,
                context.hit_result.block_pos,
                0,
                Some(context.player.id()),
            );
        }
        context.world.game_event(
            &vanilla_game_events::BLOCK_CHANGE,
            context.hit_result.block_pos,
            &GameEventContext::new(Some(context.player), Some(new_state)),
        );
        if transform.transform_type == TransformType::CopperChest && is_copper_chest(old_state) {
            emit_connected_chest_block_change(
                context.world,
                context.hit_result.block_pos,
                old_state,
                context.player,
                particle_event,
                None,
            );
        }

        // TODO trigger `ITEM_USED_ON_BLOCK`
        return InteractionResult::Success;
    }

    InteractionResult::Pass
}

/// Transformer item cost
fn consume_transform_use(transform: &BlockTransformData, context: &UseOnContext) {
    let has_infinite_materials = context.player.has_infinite_materials();
    context.inv.with_item(|item| {
        consume_transform_item(
            item,
            transform.consume_on_use,
            transform.item_damage_per_use,
            has_infinite_materials,
        );
    });
}

fn consume_transform_item(
    item: &mut ItemStack,
    consume_on_use: bool,
    item_damage_per_use: i32,
    has_infinite_materials: bool,
) {
    if item.is_stackable() {
        if consume_on_use && !has_infinite_materials {
            item.shrink(1);
        }
    } else {
        item.hurt_and_break(item_damage_per_use, has_infinite_materials);
    }
}

fn player_has_blocking_item_use_intent(context: &UseOnContext) -> bool {
    if context.hand != InteractionHand::MainHand || context.player.is_secondary_use_active() {
        return false;
    }

    context
        .player
        .inventory
        .lock()
        .get_item_in_hand(InteractionHand::OffHand)
        .has(BLOCKS_ATTACKS)
}

/// `BlockStateProvider.getOptionalState`
fn transform_provider_optional<R: Random>(
    provider: &TransformStateProvider,
    context: &UseOnContext,
    pos: BlockPos,
    random: &mut R,
) -> Option<BlockStateId> {
    match provider {
        TransformStateProvider::RuleBased { fallback, rules } => {
            let selected = select_rule_based_provider(fallback.as_deref(), rules, |predicate| {
                predicate_matches(predicate, context, pos)
            })?;
            transform_provider_state(selected, context, pos, random)
        }
        _ => transform_provider_state(provider, context, pos, random),
    }
}

/// `RuleBasedStateProvider` selection
fn select_rule_based_provider<'a>(
    fallback: Option<&'a TransformStateProvider>,
    rules: &'a [TransformStateProviderRule],
    mut predicate_matches_rule: impl FnMut(&TransformPredicate) -> bool,
) -> Option<&'a TransformStateProvider> {
    rules
        .iter()
        .find(|rule| predicate_matches_rule(&rule.if_true))
        .map(|rule| &rule.then)
        .or(fallback)
}

/// `BlockStateProvider.getState`
fn transform_provider_state<R: Random>(
    provider: &TransformStateProvider,
    context: &UseOnContext,
    pos: BlockPos,
    random: &mut R,
) -> Option<BlockStateId> {
    match provider {
        TransformStateProvider::Simple { state } => resolve_transform_block_state(state),
        TransformStateProvider::Weighted { entries } => {
            let entry = weighted_entry(entries, random);
            resolve_transform_block_state(&entry.data)
        }
        TransformStateProvider::NoiseThreshold {
            seed,
            noise,
            scale,
            threshold,
            high_chance,
            default_state,
            low_states,
            high_states,
        } => {
            let noise = normal_noise(noise, *seed);
            let noise_value = noise_value(&noise, pos, *scale);
            if noise_value < f64::from(*threshold) {
                random_state(low_states, random)
            } else if random.next_f32() < *high_chance {
                random_state(high_states, random)
            } else {
                resolve_transform_block_state(default_state)
            }
        }
        TransformStateProvider::Noise {
            seed,
            noise,
            scale,
            states,
        } => noise_state(
            states,
            noise_value(&normal_noise(noise, *seed), pos, *scale),
        ),
        TransformStateProvider::DualNoise {
            variety,
            slow_noise,
            slow_scale,
            seed,
            noise,
            scale,
            states,
        } => dual_noise_state(
            *variety,
            slow_noise,
            *slow_scale,
            *seed,
            noise,
            *scale,
            states,
            pos,
        ),
        TransformStateProvider::RotatedBlock { block } => {
            let block = REGISTRY.blocks.by_key(block)?;
            let state = REGISTRY.blocks.get_default_state_id(block);
            let axis = match random.next_i32_bounded(3) {
                0 => Axis::X,
                1 => Axis::Y,
                _ => Axis::Z,
            };
            state
                .try_get_value(&BlockStateProperties::AXIS)
                .map_or(Some(state), |_| {
                    Some(state.set_value(&BlockStateProperties::AXIS, axis))
                })
        }
        TransformStateProvider::RandomizedInt {
            source,
            property,
            values,
        } => {
            let state = transform_provider_state(source, context, pos, random)?;
            Some(set_transform_integer_property(
                state,
                property,
                values.sample(random),
            ))
        }
        TransformStateProvider::RuleBased { .. } => {
            transform_provider_optional(provider, context, pos, random)
                .or_else(|| Some(context.world.get_block_state(pos)))
        }
        TransformStateProvider::CopyProperties { source } => {
            let state = transform_provider_state(source, context, pos, random)?;
            Some(state.with_properties_of(context.world.get_block_state(pos)))
        }
    }
}

fn set_transform_integer_property(state: BlockStateId, property: &str, value: i32) -> BlockStateId {
    let block = state.get_block();
    let value = value.to_string();
    let properties = REGISTRY
        .blocks
        .get_properties(state)
        .into_iter()
        .map(|(name, current)| {
            if name == property {
                (name, value.as_str())
            } else {
                (name, current)
            }
        })
        .collect::<Vec<_>>();
    REGISTRY
        .blocks
        .state_id_from_block_properties(block, &properties)
        .unwrap_or_else(|| {
            panic!(
                "block transformer generated invalid value {value} for property {property} on {}",
                block.key
            )
        })
}

/// `DualNoiseProvider.getState`
#[expect(
    clippy::too_many_arguments,
    reason = "the provider enum owns these decoded fields individually; grouping them would only duplicate that model"
)]
fn dual_noise_state(
    variety: (i32, i32),
    slow_noise_parameters: &TransformNoiseParameters,
    slow_scale: f32,
    seed: i64,
    noise_parameters: &TransformNoiseParameters,
    scale: f32,
    states: &[TransformBlockState],
    pos: BlockPos,
) -> Option<BlockStateId> {
    let slow_noise = normal_noise(slow_noise_parameters, seed);
    let variety_noise = slow_noise_value(&slow_noise, pos, slow_scale);
    let local_variety = steel_math::map_clamped(
        variety_noise,
        -1.0,
        1.0,
        f64::from(variety.0),
        f64::from(variety.1 + 1),
    ) as i32;
    let capacity = usize::try_from(local_variety).ok()?;
    let mut possible_states = Vec::with_capacity(capacity);
    for index in 0..local_variety {
        let offset = pos.offset(index * 54_545, 0, index * 34_234);
        possible_states.push(noise_state(
            states,
            slow_noise_value(&slow_noise, offset, slow_scale),
        )?);
    }

    let noise = normal_noise(noise_parameters, seed);
    let index = noise_state_index(possible_states.len(), noise_value(&noise, pos, scale))?;
    possible_states.get(index).copied()
}

fn resolve_transform_block_state(state: &TransformBlockState) -> Option<BlockStateId> {
    let block = REGISTRY.blocks.by_key(&state.block)?;
    let properties = state
        .properties
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()));
    REGISTRY
        .blocks
        .state_id_from_block_defaulted_properties(block, properties)
}

fn weighted_entry<'a, R: Random>(
    entries: &'a [WeightedTransformBlockState],
    random: &mut R,
) -> &'a WeightedTransformBlockState {
    let mut total_weight = 0_i64;
    for entry in entries {
        assert!(
            entry.weight >= 0,
            "weighted block-state provider entry weight must be non-negative"
        );
        total_weight += i64::from(entry.weight);
        assert!(
            total_weight <= i64::from(i32::MAX),
            "weighted block-state provider total weight must be at most {}",
            i32::MAX
        );
    }
    assert!(
        total_weight != 0,
        "weighted block-state provider has no selectable entries"
    );

    let mut selected = random.next_i32_bounded(total_weight as i32);
    for entry in entries {
        if selected < entry.weight {
            return entry;
        }
        selected -= entry.weight;
    }
    unreachable!("validated weighted block-state provider must select an entry");
}

fn normal_noise(parameters: &TransformNoiseParameters, seed: i64) -> NormalNoise {
    let mut random = RandomSource::Legacy(LegacyRandom::from_seed(seed as u64));
    NormalNoise::create_from_random(&mut random, parameters.first_octave, &parameters.amplitudes)
}

fn noise_value(noise: &NormalNoise, pos: BlockPos, scale: f32) -> f64 {
    let scale = f64::from(scale);
    noise.get_value(
        f64::from(pos.x()) * scale,
        f64::from(pos.y()) * scale,
        f64::from(pos.z()) * scale,
    )
}

/// `DualNoiseProvider` `f32` precision
fn slow_noise_value(noise: &NormalNoise, pos: BlockPos, scale: f32) -> f64 {
    noise.get_value(
        f64::from(pos.x() as f32 * scale),
        f64::from(pos.y() as f32 * scale),
        f64::from(pos.z() as f32 * scale),
    )
}

fn noise_state(states: &[TransformBlockState], value: f64) -> Option<BlockStateId> {
    let index = noise_state_index(states.len(), value)?;
    resolve_transform_block_state(states.get(index)?)
}

fn noise_state_index(state_count: usize, noise_value: f64) -> Option<usize> {
    if state_count == 0 {
        return None;
    }
    let placement_value = f64::midpoint(1.0, noise_value).clamp(0.0, 0.9999);
    Some((placement_value * state_count as f64) as usize)
}

fn random_state<R: Random>(states: &[TransformBlockState], random: &mut R) -> Option<BlockStateId> {
    let count = i32::try_from(states.len()).ok()?;
    if count == 0 {
        return None;
    }
    resolve_transform_block_state(states.get(random.next_i32_bounded(count) as usize)?)
}

fn predicate_matches(
    predicate: &TransformPredicate,
    context: &UseOnContext,
    pos: BlockPos,
) -> bool {
    match predicate {
        TransformPredicate::MatchingBlocks { offset, blocks } => {
            let block = context
                .world
                .get_block_state(offset_pos(pos, *offset))
                .get_block();
            holder_set_matches(blocks, &block.key, |tag| block.has_tag(tag))
        }
        TransformPredicate::MatchingBlockTag { offset, tag } => context
            .world
            .get_block_state(offset_pos(pos, *offset))
            .get_block()
            .has_tag(tag),
        TransformPredicate::MatchingFluids { offset, fluids } => {
            let fluid = context
                .world
                .get_block_state(offset_pos(pos, *offset))
                .get_fluid_state()
                .fluid_id;
            holder_set_matches(fluids, &fluid.key, |tag| fluid.has_tag(tag))
        }
        TransformPredicate::MatchingBiomes { biomes } => context
            .world
            .biome_at(pos)
            .is_some_and(|biome| holder_set_matches(biomes, &biome.key, |tag| biome.has_tag(tag))),
        TransformPredicate::HasSturdyFace { offset, direction } => {
            let position = offset_pos(pos, *offset);
            context
                .world
                .get_block_state(position)
                .is_face_sturdy_at(position, *direction)
        }
        TransformPredicate::Solid { offset } => context
            .world
            .get_block_state(offset_pos(pos, *offset))
            .is_solid(),
        TransformPredicate::Replaceable { offset } => context
            .world
            .get_block_state(offset_pos(pos, *offset))
            .is_replaceable(),
        TransformPredicate::WouldSurvive { offset, state } => {
            let Some(state) = resolve_transform_block_state(state) else {
                return false;
            };
            BLOCK_BEHAVIORS.get_behavior(state.get_block()).can_survive(
                state,
                context.world,
                offset_pos(pos, *offset),
            )
        }
        TransformPredicate::InsideWorldBounds { offset } => !context
            .world
            .is_outside_build_height(offset_pos(pos, *offset).y()),
        TransformPredicate::Any(predicates) => predicates
            .iter()
            .any(|predicate| predicate_matches(predicate, context, pos)),
        TransformPredicate::All(predicates) => predicates
            .iter()
            .all(|predicate| predicate_matches(predicate, context, pos)),
        TransformPredicate::Not(predicate) => !predicate_matches(predicate, context, pos),
        TransformPredicate::True => true,
        TransformPredicate::Unobstructed { .. } => context.world.is_unobstructed(
            OffsetVoxelShape::without_offset(VoxelShape::FULL_BLOCK),
            pos,
        ),
        TransformPredicate::HeightRange {
            min_inclusive,
            max_inclusive,
        } => {
            let min = min_inclusive.resolve_y_with_sea_level(
                context.world.get_min_y(),
                context.world.get_height(),
                context.world.sea_level,
            );
            let max = max_inclusive.resolve_y_with_sea_level(
                context.world.get_min_y(),
                context.world.get_height(),
                context.world.sea_level,
            );
            (min..=max).contains(&pos.y())
        }
    }
}

fn holder_set_matches(
    holders: &TransformHolderSet,
    key: &steel_utils::Identifier,
    has_tag: impl FnOnce(&steel_utils::Identifier) -> bool,
) -> bool {
    match holders {
        TransformHolderSet::Tag(tag) => has_tag(tag),
        TransformHolderSet::Entries(entries) => entries.contains(key),
    }
}

const fn offset_pos(pos: BlockPos, offset: (i32, i32, i32)) -> BlockPos {
    pos.offset(offset.0, offset.1, offset.2)
}

fn drop_loot(transform: &BlockTransformData, context: &UseOnContext, old_state: BlockStateId) {
    let Some(loot_key) = &transform.loot else {
        return;
    };
    let Some(loot_table) = REGISTRY.loot_tables.by_key(loot_key) else {
        return;
    };

    let loot_context_data = InteractionLootContextData::from_use_context(old_state, context);
    let drops = if let Some(sequence_key) = &loot_table.random_sequence {
        context
            .world
            .with_random_sequence(sequence_key, |sequence| {
                let mut rng = VanillaLootRandom::new(sequence);
                interaction_loot_drops(loot_table, &mut rng, &loot_context_data)
            })
    } else {
        // Level RNG loot scope
        context.world.with_random(|random| {
            let mut rng = VanillaLootRandom::new(random);
            interaction_loot_drops(loot_table, &mut rng, &loot_context_data)
        })
    };
    for drop in drops {
        if transform.drop_strategy == DropStrategy::ClickedFace {
            context.world.pop_resource_from_face(
                context.hit_result.block_pos,
                context.hit_result.direction,
                drop,
            );
        } else {
            context
                .world
                .pop_resource(context.hit_result.block_pos, drop);
        }
    }
}

fn interaction_loot_drops<R: LootRandom>(
    loot_table: LootTableRef,
    rng: &mut R,
    data: &InteractionLootContextData,
) -> Vec<ItemStack> {
    let interacting_entity = EntityRef {
        entity_type: Some(data.interacting_entity_type),
        flags: data.interacting_entity_flags,
        equipment: None,
        custom_name: None,
        sheep_color: None,
        sheep_sheared: None,
    };

    let mut loot_context = LootContext::new(rng)
        .with_block_state(data.old_state)
        .with_tool(&data.tool)
        .with_interacting_entity(interacting_entity)
        .with_game_time(data.game_time)
        .with_origin(data.origin.0, data.origin.1, data.origin.2);
    if let Some(block_entity) = data.block_entity.as_ref() {
        loot_context = loot_context.with_block_entity(block_entity.as_loot_context_ref());
    }
    loot_table.get_random_items(&mut loot_context)
}

/// `BLOCK_INTERACT` loot data
struct InteractionLootContextData {
    old_state: BlockStateId,
    tool: ItemStack,
    interacting_entity_type: &'static Identifier,
    interacting_entity_flags: EntityRefFlags,
    block_entity: Option<LootBlockEntityData>,
    game_time: i64,
    origin: (f64, f64, f64),
}

impl InteractionLootContextData {
    fn from_use_context(old_state: BlockStateId, context: &UseOnContext) -> Self {
        let tool = context.inv.with_item(|item| item.clone());
        let block_entity = context
            .world
            .get_block_entity(context.hit_result.block_pos)
            .map(LootBlockEntityData::from_block_entity);
        Self {
            old_state,
            tool,
            interacting_entity_type: &context.player.entity_type().key,
            interacting_entity_flags: EntityRefFlags {
                is_on_fire: context.player.is_on_fire(),
                is_sneaking: context.player.is_crouching(),
                is_sprinting: context.player.is_sprinting(),
                is_swimming: context.player.is_swimming(),
                is_baby: context.player.is_baby(),
            },
            block_entity,
            game_time: context.world.game_time(),
            origin: (
                f64::from(context.hit_result.block_pos.x()),
                f64::from(context.hit_result.block_pos.y()),
                f64::from(context.hit_result.block_pos.z()),
            ),
        }
    }
}

/// Block entity loot data
struct LootBlockEntityData {
    block_entity_type: &'static steel_utils::Identifier,
    inventory: Option<Vec<ItemStack>>,
}

impl LootBlockEntityData {
    fn from_block_entity(block_entity: crate::block_entity::SharedBlockEntity) -> Self {
        let inventory = block_entity.container_ref().map(|container_ref| {
            let guard = ContainerLockGuard::lock_all(&[&container_ref]);
            let container = guard
                .get(container_ref.container_id())
                .expect("locked block-entity container must be present");
            (0..container.get_container_size())
                .map(|slot| container.get_item(slot).clone())
                .collect()
        });
        Self {
            block_entity_type: &block_entity.get_type().key,
            inventory,
        }
    }

    fn as_loot_context_ref(&self) -> BlockEntityRef<'_> {
        BlockEntityRef {
            block_entity_type: Some(self.block_entity_type),
            custom_name: None,
            inventory: self.inventory.as_deref(),
        }
    }
}

const fn particle_event(particle: TransformParticle) -> Option<i32> {
    match particle {
        TransformParticle::None => None,
        TransformParticle::Scrape => Some(PARTICLES_SCRAPE),
        TransformParticle::WaxOn => Some(PARTICLES_WAX_ON),
        TransformParticle::WaxOff => Some(PARTICLES_WAX_OFF),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Weak};

    use steel_registry::{
        data_components::{
            components::{
                BlockTransformData, DropStrategy, TransformBlockState, TransformHolderSet,
                TransformNoiseParameters, TransformParticle, TransformPredicate,
                TransformStateProvider, TransformStateProviderRule, TransformType,
                WeightedTransformBlockState,
            },
            vanilla_components::BLOCK_TRANSFORMER,
        },
        init_vanilla_registry,
        item_stack::ItemStack,
        vanilla_block_entity_types, vanilla_blocks, vanilla_items,
    };
    use steel_utils::{BlockPos, Direction, Identifier, random::legacy_random::LegacyRandom};

    use crate::{
        block_entity::{
            SharedBlockEntity,
            entities::{BARREL_SLOTS, BarrelBlockEntity},
        },
        inventory::lock::ContainerLockGuard,
        world::World,
    };

    use super::{
        LootBlockEntityData, PARTICLES_SCRAPE, PARTICLES_WAX_OFF, PARTICLES_WAX_ON,
        consume_transform_item, noise_state_index, normal_noise, particle_event,
        select_rule_based_provider, slow_noise_value, weighted_entry,
    };

    fn only_rule(transform: &BlockTransformData) -> &TransformStateProviderRule {
        let TransformStateProvider::RuleBased { rules, .. } = &transform.block_state_provider
        else {
            panic!("generated tool transform must use a rule-based state provider");
        };
        assert_eq!(rules.len(), 1);
        &rules[0]
    }

    fn simple_state(provider: &TransformStateProvider) -> &TransformBlockState {
        let TransformStateProvider::Simple { state } = provider else {
            panic!("generated transform provider must be a simple state provider");
        };
        state
    }

    fn copy_properties_source_state(provider: &TransformStateProvider) -> &TransformBlockState {
        let TransformStateProvider::CopyProperties { source } = provider else {
            panic!("generated transform provider must copy matching properties");
        };
        simple_state(source)
    }

    fn matching_blocks_contains(predicate: &TransformPredicate, expected: &Identifier) -> bool {
        matches!(
            predicate,
            TransformPredicate::MatchingBlocks {
                offset: (0, 0, 0),
                blocks: TransformHolderSet::Entries(entries),
            } if entries.contains(expected)
        )
    }

    fn transform_state(block: &'static str) -> TransformBlockState {
        TransformBlockState {
            block: Identifier::vanilla_static(block),
            properties: Vec::new(),
        }
    }

    #[test]
    fn generated_shovel_transformer_requires_clear_space_and_blocks_downward_use() {
        init_vanilla_registry();

        let transformer = vanilla_items::WOODEN_SHOVEL
            .components
            .get_ref(BLOCK_TRANSFORMER)
            .expect("wooden shovel must have a block transformer");
        assert_eq!(transformer.transforms.len(), 1);

        let transform = &transformer.transforms[0];
        assert_eq!(transform.disallowed_faces, vec![Direction::Down]);
        assert_eq!(transform.item_damage_per_use, 1);
        assert!(transform.consume_on_use);

        let rule = only_rule(transform);
        let TransformPredicate::All(predicates) = &rule.if_true else {
            panic!("shovel transform must require both the path tag and clear space");
        };
        assert!(predicates.iter().any(|predicate| {
            matches!(
                predicate,
                TransformPredicate::MatchingBlockTag {
                    offset: (0, 0, 0),
                    tag,
                } if *tag == Identifier::vanilla_static("turns_into_dirt_path")
            )
        }));
        assert!(predicates.iter().any(|predicate| {
            matches!(
                predicate,
                TransformPredicate::MatchingBlockTag {
                    offset: (0, 1, 0),
                    tag,
                } if *tag == Identifier::vanilla_static("air")
            )
        }));
        assert_eq!(
            simple_state(&rule.then).block,
            Identifier::vanilla_static("dirt_path")
        );
    }

    #[test]
    fn generated_hoe_transformer_keeps_rooted_dirt_loot_and_face_drop() {
        init_vanilla_registry();

        let transformer = vanilla_items::WOODEN_HOE
            .components
            .get_ref(BLOCK_TRANSFORMER)
            .expect("wooden hoe must have a block transformer");
        assert_eq!(transformer.transforms.len(), 3);

        let rooted_dirt = transformer
            .transforms
            .iter()
            .find(|transform| {
                transform.loot.as_ref() == Some(&Identifier::vanilla_static("till/rooted_dirt"))
            })
            .expect("rooted dirt hoe transform must retain its interact loot table");
        assert_eq!(
            simple_state(&only_rule(rooted_dirt).then).block,
            Identifier::vanilla_static("dirt")
        );
        assert!(matching_blocks_contains(
            &only_rule(rooted_dirt).if_true,
            &Identifier::vanilla_static("rooted_dirt")
        ));
        assert_eq!(rooted_dirt.drop_strategy, DropStrategy::ClickedFace);
        assert_eq!(rooted_dirt.item_damage_per_use, 1);
    }

    #[test]
    fn generated_axe_transformer_preserves_stripping_and_copper_chest_rules() {
        init_vanilla_registry();

        let transformer = vanilla_items::WOODEN_AXE
            .components
            .get_ref(BLOCK_TRANSFORMER)
            .expect("wooden axe must have a block transformer");
        assert_eq!(transformer.transforms.len(), 130);

        let stripped_oak_log = transformer
            .transforms
            .iter()
            .find(|transform| {
                matching_blocks_contains(
                    &only_rule(transform).if_true,
                    &Identifier::vanilla_static("oak_log"),
                )
            })
            .expect("axe transformer must strip oak logs");
        assert_eq!(
            copy_properties_source_state(&only_rule(stripped_oak_log).then).block,
            Identifier::vanilla_static("stripped_oak_log")
        );

        let copper_chest = transformer
            .transforms
            .iter()
            .find(|transform| {
                transform.transform_type == TransformType::CopperChest
                    && matching_blocks_contains(
                        &only_rule(transform).if_true,
                        &Identifier::vanilla_static("exposed_copper_chest"),
                    )
            })
            .expect("axe transformer must scrape exposed copper chests");
        assert_eq!(copper_chest.particle, TransformParticle::Scrape);
        assert_eq!(copper_chest.item_damage_per_use, 1);
        assert_eq!(
            copy_properties_source_state(&only_rule(copper_chest).then).block,
            Identifier::vanilla_static("copper_chest")
        );
    }

    #[test]
    fn unmatched_rule_based_transformer_is_a_noop() {
        init_vanilla_registry();

        let transformer = vanilla_items::WOODEN_SHOVEL
            .components
            .get_ref(BLOCK_TRANSFORMER)
            .expect("wooden shovel must have a block transformer");
        let TransformStateProvider::RuleBased { fallback, rules } =
            &transformer.transforms[0].block_state_provider
        else {
            panic!("shovel transformer must use a rule-based state provider");
        };

        assert!(select_rule_based_provider(fallback.as_deref(), rules, |_| false).is_none());
        assert!(select_rule_based_provider(fallback.as_deref(), rules, |_| true).is_some());
    }

    #[test]
    fn rule_based_provider_uses_the_first_matching_rule() {
        let first = TransformStateProvider::Simple {
            state: transform_state("stone"),
        };
        let second = TransformStateProvider::Simple {
            state: transform_state("dirt"),
        };
        let rules = [
            TransformStateProviderRule {
                if_true: TransformPredicate::True,
                then: first,
            },
            TransformStateProviderRule {
                if_true: TransformPredicate::True,
                then: second,
            },
        ];

        let selected = select_rule_based_provider(None, &rules, |_| true);
        assert!(selected.is_some_and(|provider| matches!(
            provider,
            TransformStateProvider::Simple { state } if state.block == Identifier::vanilla_static("stone")
        )));
    }

    #[test]
    fn weighted_provider_never_selects_zero_weight_entries() {
        let entries = [
            WeightedTransformBlockState {
                data: transform_state("air"),
                weight: 0,
            },
            WeightedTransformBlockState {
                data: transform_state("stone"),
                weight: 1,
            },
            WeightedTransformBlockState {
                data: transform_state("dirt"),
                weight: 2,
            },
        ];
        let mut random = LegacyRandom::from_seed(42);
        let mut selected_stone = false;
        let mut selected_dirt = false;

        for _ in 0..128 {
            let selected = weighted_entry(&entries, &mut random);
            assert_ne!(selected, &entries[0]);
            selected_stone |= selected == &entries[1];
            selected_dirt |= selected == &entries[2];
        }

        assert!(selected_stone);
        assert!(selected_dirt);
    }

    #[test]
    fn noise_state_index_matches_vanilla_clamped_mapping() {
        assert_eq!(noise_state_index(0, 0.0), None);
        assert_eq!(noise_state_index(4, -2.0), Some(0));
        assert_eq!(noise_state_index(4, -1.0), Some(0));
        assert_eq!(noise_state_index(4, 0.0), Some(2));
        assert_eq!(noise_state_index(4, 1.0), Some(3));
        assert_eq!(noise_state_index(4, 2.0), Some(3));
    }

    #[test]
    fn transformer_particles_map_to_vanilla_level_events() {
        assert_eq!(particle_event(TransformParticle::None), None);
        assert_eq!(
            particle_event(TransformParticle::Scrape),
            Some(PARTICLES_SCRAPE)
        );
        assert_eq!(
            particle_event(TransformParticle::WaxOn),
            Some(PARTICLES_WAX_ON)
        );
        assert_eq!(
            particle_event(TransformParticle::WaxOff),
            Some(PARTICLES_WAX_OFF)
        );
    }

    #[test]
    fn dual_noise_slow_scale_uses_java_f32_coordinate_product() {
        let noise = normal_noise(
            &TransformNoiseParameters {
                first_octave: -3,
                amplitudes: vec![1.0, 1.0],
            },
            12_345,
        );
        let pos = BlockPos::new(16_777_217, 73, -16_777_217);
        let scale = 0.1_f32;

        let java_x = f64::from(pos.x() as f32 * scale);
        let wide_x = f64::from(pos.x()) * f64::from(scale);
        assert_ne!(java_x.to_bits(), wide_x.to_bits());
        assert_eq!(
            slow_noise_value(&noise, pos, scale).to_bits(),
            noise
                .get_value(
                    java_x,
                    f64::from(pos.y() as f32 * scale),
                    f64::from(pos.z() as f32 * scale),
                )
                .to_bits()
        );
    }

    #[test]
    fn transform_item_cost_matches_stack_consumption_and_durability() {
        init_vanilla_registry();

        let mut stackable = ItemStack::with_count(&vanilla_items::STICK, 3);
        consume_transform_item(&mut stackable, true, 0, false);
        assert_eq!(stackable.count(), 2);

        consume_transform_item(&mut stackable, false, 0, false);
        assert_eq!(stackable.count(), 2);

        consume_transform_item(&mut stackable, true, 0, true);
        assert_eq!(stackable.count(), 2);

        let axe_transform = vanilla_items::WOODEN_AXE
            .components
            .get_ref(BLOCK_TRANSFORMER)
            .expect("wooden axe must have a block transformer")
            .transforms
            .first()
            .expect("wooden axe must have a transform");
        let mut axe = ItemStack::new(&vanilla_items::WOODEN_AXE);
        consume_transform_item(
            &mut axe,
            axe_transform.consume_on_use,
            axe_transform.item_damage_per_use,
            false,
        );
        assert_eq!(axe.get_damage_value(), 1);

        consume_transform_item(
            &mut axe,
            axe_transform.consume_on_use,
            axe_transform.item_damage_per_use,
            true,
        );
        assert_eq!(axe.get_damage_value(), 1);
    }

    #[test]
    fn loot_context_snapshots_block_entity_type_and_container_contents() {
        init_vanilla_registry();

        let barrel: SharedBlockEntity = Arc::new(BarrelBlockEntity::new(
            Weak::<World>::new(),
            BlockPos::ZERO,
            vanilla_blocks::BARREL.default_state(),
        ));
        let container_ref = barrel
            .container_ref()
            .expect("barrel must expose its container capability");
        let mut guard = ContainerLockGuard::lock_all(&[&container_ref]);
        assert!(guard.set_item(
            container_ref.container_id(),
            0,
            ItemStack::new(&vanilla_items::STICK),
        ));
        drop(guard);

        let data = LootBlockEntityData::from_block_entity(barrel);
        let block_entity = data.as_loot_context_ref();
        assert_eq!(
            block_entity.block_entity_type,
            Some(&vanilla_block_entity_types::BARREL.key)
        );
        let inventory = block_entity
            .inventory
            .expect("barrel contents must be available to loot evaluation");
        assert_eq!(inventory.len(), BARREL_SLOTS);
        assert_eq!(inventory[0].item(), &*vanilla_items::STICK);
    }
}
