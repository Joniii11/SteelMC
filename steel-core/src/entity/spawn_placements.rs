//! Spawn placement metadata and predicates used by chunk-generation spawning.

use steel_registry::{
    blocks::{
        block_state_ext::BlockStateExt as _, properties::Direction,
        shapes::{OffsetVoxelShape, SupportType, is_offset_shape_full_block},
    },
    entity_type::EntityTypeRef,
    vanilla_biome_tags::BiomeTag,
    vanilla_block_tags::BlockTag,
    vanilla_blocks, vanilla_entities,
    vanilla_fluid_tags::FluidTag,
};
use steel_utils::{BlockPos, WorldAabb, types::Difficulty};

use crate::{
    behavior::{
        BLOCK_BEHAVIORS, BlockBehavior as _, BlockCollisionContext,
        BlockStateBehaviorExt as _,
    },
    chunk::heightmap::HeightmapType,
    entity::{EntitySpawnReason, ai::path::PathComputationType, ai::walk::WalkPathEvaluator},
    world::SignalQueryContext,
    worldgen::region::WorldGenRegion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpawnPlacementType {
    NoRestrictions,
    InLava,
    OnGround,
}

#[derive(Clone, Copy)]
enum SpawnPredicate {
    Animal,
    Armadillo,
    Camel,
    Fox,
    Frog,
    Goat,
    Mooshroom,
    Panda,
    Parrot,
    PolarBear,
    Rabbit,
    Strider,
    Turtle,
    Wolf,
}

#[derive(Clone, Copy)]
struct SpawnPlacementData {
    heightmap: HeightmapType,
    placement: SpawnPlacementType,
    predicate: SpawnPredicate,
}

const DEFAULT_HEIGHTMAP: HeightmapType = HeightmapType::MotionBlockingNoLeaves;

const fn is_type(actual: EntityTypeRef, expected: EntityTypeRef) -> bool {
    std::ptr::eq(actual, expected)
}

fn data_for(entity_type: EntityTypeRef) -> Option<SpawnPlacementData> {
    let on_ground = |predicate| SpawnPlacementData {
        heightmap: DEFAULT_HEIGHTMAP,
        placement: SpawnPlacementType::OnGround,
        predicate,
    };

    if is_type(entity_type, &vanilla_entities::ARMADILLO) {
        Some(on_ground(SpawnPredicate::Armadillo))
    } else if is_type(entity_type, &vanilla_entities::CAMEL) {
        Some(on_ground(SpawnPredicate::Camel))
    } else if is_type(entity_type, &vanilla_entities::FOX) {
        Some(SpawnPlacementData {
            heightmap: DEFAULT_HEIGHTMAP,
            placement: SpawnPlacementType::NoRestrictions,
            predicate: SpawnPredicate::Fox,
        })
    } else if is_type(entity_type, &vanilla_entities::FROG) {
        Some(on_ground(SpawnPredicate::Frog))
    } else if is_type(entity_type, &vanilla_entities::GOAT) {
        Some(on_ground(SpawnPredicate::Goat))
    } else if is_type(entity_type, &vanilla_entities::MOOSHROOM) {
        Some(on_ground(SpawnPredicate::Mooshroom))
    } else if is_type(entity_type, &vanilla_entities::PANDA) {
        Some(SpawnPlacementData {
            heightmap: DEFAULT_HEIGHTMAP,
            placement: SpawnPlacementType::NoRestrictions,
            predicate: SpawnPredicate::Panda,
        })
    } else if is_type(entity_type, &vanilla_entities::PARROT) {
        Some(SpawnPlacementData {
            heightmap: HeightmapType::MotionBlocking,
            placement: SpawnPlacementType::OnGround,
            predicate: SpawnPredicate::Parrot,
        })
    } else if is_type(entity_type, &vanilla_entities::POLAR_BEAR) {
        Some(on_ground(SpawnPredicate::PolarBear))
    } else if is_type(entity_type, &vanilla_entities::RABBIT) {
        Some(on_ground(SpawnPredicate::Rabbit))
    } else if is_type(entity_type, &vanilla_entities::STRIDER) {
        Some(SpawnPlacementData {
            heightmap: DEFAULT_HEIGHTMAP,
            placement: SpawnPlacementType::InLava,
            predicate: SpawnPredicate::Strider,
        })
    } else if is_type(entity_type, &vanilla_entities::TURTLE) {
        Some(on_ground(SpawnPredicate::Turtle))
    } else if is_type(entity_type, &vanilla_entities::WOLF) {
        Some(on_ground(SpawnPredicate::Wolf))
    } else if [
        &vanilla_entities::CHICKEN,
        &vanilla_entities::COW,
        &vanilla_entities::DONKEY,
        &vanilla_entities::HORSE,
        &vanilla_entities::LLAMA,
        &vanilla_entities::PIG,
        &vanilla_entities::SHEEP,
    ]
    .into_iter()
    .any(|expected| is_type(entity_type, expected))
    {
        Some(on_ground(SpawnPredicate::Animal))
    } else {
        None
    }
}

#[must_use]
pub(crate) fn heightmap_type(entity_type: EntityTypeRef) -> HeightmapType {
    data_for(entity_type).map_or(DEFAULT_HEIGHTMAP, |data| data.heightmap)
}

#[must_use]
pub(crate) fn top_non_colliding_pos(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    x: i32,
    z: i32,
) -> BlockPos {
    let mut pos = BlockPos::new(x, region.height_at(heightmap_type(entity_type), x, z), z);
    if region.world().dimension_type.has_ceiling {
        loop {
            pos = pos.below();
            if region.block_state(pos).is_air() {
                break;
            }
        }

        while region.block_state(pos).is_air() && pos.y() > region.min_y() {
            pos = pos.below();
        }
    }

    adjust_spawn_position(region, entity_type, pos)
}

fn adjust_spawn_position(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    candidate: BlockPos,
) -> BlockPos {
    let placement = data_for(entity_type)
        .map_or(SpawnPlacementType::NoRestrictions, |data| data.placement);
    if placement != SpawnPlacementType::OnGround {
        return candidate;
    }

    let below = candidate.below();
    if region
        .block_state(below)
        .is_pathfindable(PathComputationType::Land)
    {
        below
    } else {
        candidate
    }
}

#[must_use]
pub(crate) fn is_spawn_position_ok(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    pos: BlockPos,
) -> bool {
    let placement = data_for(entity_type)
        .map_or(SpawnPlacementType::NoRestrictions, |data| data.placement);
    match placement {
        SpawnPlacementType::NoRestrictions => true,
        SpawnPlacementType::InLava => {
            region.world().is_block_within_world_border(pos)
                && region
                    .block_state(pos)
                    .get_fluid_state()
                    .fluid_id
                    .has_tag(&FluidTag::LAVA)
        }
        SpawnPlacementType::OnGround => {
            if !region.world().is_block_within_world_border(pos) {
                return false;
            }
            let below = pos.below();
            is_valid_spawn_surface(region, entity_type, below)
                && is_valid_empty_spawn_block(region, entity_type, pos)
                && is_valid_empty_spawn_block(region, entity_type, pos.above())
        }
    }
}

fn is_valid_spawn_surface(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    pos: BlockPos,
) -> bool {
    let state = region.block_state(pos);
    let block = state.get_block();

    if block == &vanilla_blocks::BEDROCK {
        return false;
    }
    if block.has_tag(&BlockTag::LEAVES) {
        return is_type(entity_type, &vanilla_entities::PARROT);
    }
    if block == &vanilla_blocks::ICE {
        return is_type(entity_type, &vanilla_entities::POLAR_BEAR);
    }
    if block == &vanilla_blocks::PACKED_ICE {
        return true;
    }
    if block == &vanilla_blocks::MAGMA_BLOCK {
        return entity_type.fire_immune;
    }

    state.get_light_emission() < 14
        && BLOCK_BEHAVIORS
            .get_behavior(block)
            .is_face_sturdy(state, region, pos, Direction::Up, SupportType::Full)
}

fn is_valid_empty_spawn_block(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    pos: BlockPos,
) -> bool {
    let state = region.block_state(pos);
    let behavior = BLOCK_BEHAVIORS.get_behavior(state.get_block());
    let shape = behavior.get_collision_shape(
        state,
        region,
        pos,
        BlockCollisionContext::empty(),
    );
    let offset = behavior.get_collision_shape_offset(
        state,
        region,
        pos,
        BlockCollisionContext::empty(),
    );
    if is_offset_shape_full_block(OffsetVoxelShape::new(shape, offset))
        || behavior.is_signal_source(state, SignalQueryContext::DEFAULT)
        || !state.get_fluid_state().is_empty()
        || state
            .get_block()
            .has_tag(&BlockTag::PREVENT_MOB_SPAWNING_INSIDE)
    {
        return false;
    }

    !is_block_dangerous(entity_type, state)
}

fn is_block_dangerous(
    entity_type: EntityTypeRef,
    state: steel_utils::BlockStateId,
) -> bool {
    let block = state.get_block();
    if (is_type(entity_type, &vanilla_entities::FOX)
        && block.has_tag(&BlockTag::FOX_IMMUNE_TO))
        || (is_type(entity_type, &vanilla_entities::POLAR_BEAR)
            && block.has_tag(&BlockTag::POLAR_BEAR_IMMUNE_TO))
    {
        return false;
    }
    if !entity_type.fire_immune && WalkPathEvaluator::is_burning_block(state) {
        return true;
    }

    block == &vanilla_blocks::WITHER_ROSE
        || block == &vanilla_blocks::SWEET_BERRY_BUSH
        || block == &vanilla_blocks::CACTUS
        || block == &vanilla_blocks::POWDER_SNOW
}

#[must_use]
pub(crate) fn check_spawn_rules(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    spawn_reason: EntitySpawnReason,
    pos: BlockPos,
) -> bool {
    if !entity_type.allowed_in_peaceful && region.world().difficulty() == Difficulty::Peaceful {
        return false;
    }

    let Some(data) = data_for(entity_type) else {
        return true;
    };
    check_predicate(region, data.predicate, spawn_reason, pos)
}

fn check_predicate(
    region: &WorldGenRegion<'_>,
    predicate: SpawnPredicate,
    spawn_reason: EntitySpawnReason,
    pos: BlockPos,
) -> bool {
    let below_block = region.block_state(pos.below()).get_block();
    let bright_enough = || region.spawn_raw_brightness(pos, 0) > 8;
    match predicate {
        SpawnPredicate::Animal | SpawnPredicate::Panda => {
            below_block.has_tag(&BlockTag::ANIMALS_SPAWNABLE_ON)
                && (spawn_reason.ignores_light_requirements() || bright_enough())
        }
        SpawnPredicate::Armadillo => {
            below_block.has_tag(&BlockTag::ARMADILLO_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Camel => {
            below_block.has_tag(&BlockTag::CAMELS_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Fox => {
            below_block.has_tag(&BlockTag::FOXES_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Frog => {
            below_block.has_tag(&BlockTag::FROGS_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Goat => {
            below_block.has_tag(&BlockTag::GOATS_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Mooshroom => {
            below_block.has_tag(&BlockTag::MOOSHROOMS_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Parrot => {
            below_block.has_tag(&BlockTag::PARROTS_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::PolarBear => {
            let alternate = region.biome_at(pos).is_some_and(|biome| {
                biome.has_tag(&BiomeTag::POLAR_BEARS_SPAWN_ON_ALTERNATE_BLOCKS)
            });
            if alternate {
                below_block.has_tag(&BlockTag::POLAR_BEARS_SPAWNABLE_ON_ALTERNATE)
                    && bright_enough()
            } else {
                below_block.has_tag(&BlockTag::ANIMALS_SPAWNABLE_ON)
                    && (spawn_reason.ignores_light_requirements() || bright_enough())
            }
        }
        SpawnPredicate::Rabbit => {
            below_block.has_tag(&BlockTag::RABBITS_SPAWNABLE_ON) && bright_enough()
        }
        SpawnPredicate::Strider => {
            let mut check_pos = pos;
            loop {
                check_pos = check_pos.above();
                let fluid = region.block_state(check_pos).get_fluid_state();
                if !fluid.fluid_id.has_tag(&FluidTag::LAVA) {
                    break region.block_state(check_pos).is_air();
                }
            }
        }
        SpawnPredicate::Turtle => {
            pos.y() < region.sea_level() + 4
                && below_block.has_tag(&BlockTag::SAND)
                && bright_enough()
        }
        SpawnPredicate::Wolf => {
            below_block.has_tag(&BlockTag::WOLVES_SPAWNABLE_ON) && bright_enough()
        }
    }
}

#[must_use]
pub(crate) fn no_collision(region: &WorldGenRegion<'_>, aabb: WorldAabb) -> bool {
    const EPSILON: f64 = 1.0e-7;
    let min_x = (aabb.min_x() - EPSILON).floor() as i32 - 1;
    let min_y = (aabb.min_y() - EPSILON).floor() as i32 - 1;
    let min_z = (aabb.min_z() - EPSILON).floor() as i32 - 1;
    let max_x = (aabb.max_x() + EPSILON).floor() as i32 + 1;
    let max_y = (aabb.max_y() + EPSILON).floor() as i32 + 1;
    let max_z = (aabb.max_z() + EPSILON).floor() as i32 + 1;

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let pos = BlockPos::new(x, y, z);
                let state = region.block_state(pos);
                if state.is_air() {
                    continue;
                }
                let behavior = BLOCK_BEHAVIORS.get_behavior(state.get_block());
                let boxes = behavior.get_collision_boxes(
                    state,
                    region,
                    pos,
                    BlockCollisionContext::empty(),
                );
                if boxes
                    .iter()
                    .any(|shape| aabb.intersects(shape.at_block(pos)))
                {
                    return false;
                }
            }
        }
    }

    true
}

#[must_use]
pub(crate) fn check_spawn_obstruction(
    region: &WorldGenRegion<'_>,
    entity_type: EntityTypeRef,
    aabb: WorldAabb,
) -> bool {
    if is_type(entity_type, &vanilla_entities::STRIDER) {
        return true;
    }

    let min_x = aabb.min_x().floor() as i32;
    let min_y = aabb.min_y().floor() as i32;
    let min_z = aabb.min_z().floor() as i32;
    let max_x = aabb.max_x().ceil() as i32;
    let max_y = aabb.max_y().ceil() as i32;
    let max_z = aabb.max_z().ceil() as i32;
    for y in min_y..max_y {
        for z in min_z..max_z {
            for x in min_x..max_x {
                if !region
                    .block_state(BlockPos::new(x, y, z))
                    .get_fluid_state()
                    .is_empty()
                {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use steel_registry::{REGISTRY, RegistryExt as _, init_vanilla_registry};

    use super::data_for;

    #[test]
    fn bundled_chunk_generation_creatures_have_placement_data() {
        init_vanilla_registry();
        let mut missing = Vec::new();
        for (_, biome) in REGISTRY.biomes.iter() {
            let Some(spawners) = biome.spawners.get("creature") else {
                continue;
            };
            for spawner in spawners {
                let Some(entity_type) = REGISTRY.entity_types.by_key(&spawner.entity_type) else {
                    missing.push(spawner.entity_type.to_string());
                    continue;
                };
                if data_for(entity_type).is_none() {
                    missing.push(entity_type.key.to_string());
                }
            }
        }
        missing.sort_unstable();
        missing.dedup();

        assert!(missing.is_empty(), "missing spawn placement data: {missing:?}");
    }
}
