//! Vanilla's original-mob path used while a chunk reaches `ChunkStatus::Spawn`.

use glam::DVec3;
use steel_registry::{
    REGISTRY, RegistryExt as _,
    biome::{BiomeRef, SpawnerData},
    entity_type::{EntityTypeRef, MobCategory},
    vanilla_game_rules::SPAWN_MOBS,
};
use steel_utils::{
    BlockPos, ChunkPos, WorldAabb,
    random::{Random as _, legacy_random::LegacyRandom},
};

use crate::{
    entity::{
        ENTITIES, EntitySpawnReason, SpawnGroupData, next_entity_id,
        spawn_placements,
    },
    worldgen::region::WorldGenRegion,
};

pub(crate) fn spawn_mobs_for_chunk_generation(
    region: &WorldGenRegion<'_>,
    biome: BiomeRef,
    chunk_pos: ChunkPos,
    random: &mut LegacyRandom,
) {
    let Some(spawners) = biome
        .spawners
        .get(MobCategory::Creature.serialized_name())
    else {
        return;
    };
    let world = region.world();
    if spawners.is_empty() || !world.get_game_rule(&SPAWN_MOBS) {
        return;
    }

    let origin_x = chunk_pos.0.x * 16;
    let origin_z = chunk_pos.0.y * 16;
    while random.next_f32() < biome.creature_spawn_probability {
        let Some(spawner_data) = select_weighted_spawner(spawners, random) else {
            continue;
        };
        let Some(entity_type) = REGISTRY.entity_types.by_key(&spawner_data.entity_type) else {
            continue;
        };

        let count = spawner_data.min_count
            + random.next_i32_bounded(1 + spawner_data.max_count - spawner_data.min_count);
        let mut group_spawn_data: Option<SpawnGroupData> = None;
        let mut x = origin_x + random.next_i32_bounded(16);
        let mut z = origin_z + random.next_i32_bounded(16);
        let start_x = x;
        let start_z = z;

        for _ in 0..count {
            let mut success = false;
            for _ in 0..4 {
                if success {
                    break;
                }

                let pos = spawn_placements::top_non_colliding_pos(region, entity_type, x, z);
                if entity_type.summonable
                    && spawn_placements::is_spawn_position_ok(region, entity_type, pos)
                {
                    let width = f64::from(entity_type.dimensions.width);
                    let spawn_x = f64::from(x)
                        .clamp(f64::from(origin_x) + width, f64::from(origin_x + 16) - width);
                    let spawn_z = f64::from(z)
                        .clamp(f64::from(origin_z) + width, f64::from(origin_z + 16) - width);
                    let spawn_pos = DVec3::new(spawn_x, f64::from(pos.y()), spawn_z);
                    let spawn_aabb = spawn_aabb(entity_type, spawn_pos);
                    let rule_pos = BlockPos::from(spawn_pos);
                    if !spawn_placements::no_collision(region, spawn_aabb)
                        || !spawn_placements::check_spawn_rules(
                            region,
                            entity_type,
                            EntitySpawnReason::ChunkGeneration,
                            rule_pos,
                        )
                    {
                        continue;
                    }

                    let uses_raw_fallback = !ENTITIES.has_factory(entity_type);
                    let entity = ENTITIES.create_or_raw(
                        entity_type,
                        next_entity_id(),
                        spawn_pos,
                        region.weak_world(),
                    );
                    let yaw = random.next_f32() * 360.0;
                    if entity.try_set_position(spawn_pos).is_err() {
                        continue;
                    }
                    entity.set_rotation((yaw, 0.0));
                    entity.set_old_position_to_current();
                    entity.base().set_old_rotation_to_current();
                    if spawn_placements::check_spawn_obstruction(
                        region,
                        entity_type,
                        spawn_aabb,
                    ) {
                        let can_add = if let Some(mob) = entity.as_mob() {
                            group_spawn_data = mob.finalize_spawn(
                                &world,
                                EntitySpawnReason::ChunkGeneration,
                                group_spawn_data,
                            );
                            true
                        } else {
                            uses_raw_fallback
                        };
                        if can_add {
                            let _ = region.add_fresh_entity_with_passengers(entity);
                            success = true;
                        }
                    }
                }

                // Vanilla's explicit failure `continue`s above skip this walk; placement and
                // obstruction failures, as well as successful spawns, still consume it.
                move_to_next_pack_position(
                    random,
                    origin_x,
                    origin_z,
                    start_x,
                    start_z,
                    &mut x,
                    &mut z,
                );
            }
        }
    }
}

fn spawn_aabb(entity_type: EntityTypeRef, pos: DVec3) -> WorldAabb {
    WorldAabb::entity_box(
        pos.x,
        pos.y,
        pos.z,
        f64::from(entity_type.dimensions.half_width()),
        f64::from(entity_type.dimensions.height),
    )
}

fn select_weighted_spawner<'a>(
    spawners: &'a [SpawnerData],
    random: &mut LegacyRandom,
) -> Option<&'a SpawnerData> {
    let total_weight = spawners
        .iter()
        .fold(0_i32, |total, spawner| total.wrapping_add(spawner.weight));
    if total_weight <= 0 {
        return None;
    }

    let mut selected_weight = random.next_i32_bounded(total_weight);
    for spawner in spawners {
        selected_weight -= spawner.weight;
        if selected_weight < 0 {
            return Some(spawner);
        }
    }
    None
}

fn move_to_next_pack_position(
    random: &mut LegacyRandom,
    origin_x: i32,
    origin_z: i32,
    start_x: i32,
    start_z: i32,
    x: &mut i32,
    z: &mut i32,
) {
    *x += random.next_i32_bounded(5) - random.next_i32_bounded(5);
    *z += random.next_i32_bounded(5) - random.next_i32_bounded(5);
    while *x < origin_x || *x >= origin_x + 16 || *z < origin_z || *z >= origin_z + 16 {
        *x = start_x + random.next_i32_bounded(5) - random.next_i32_bounded(5);
        *z = start_z + random.next_i32_bounded(5) - random.next_i32_bounded(5);
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::biome::SpawnerData;
    use steel_utils::{
        Identifier,
        random::{Random as _, legacy_random::LegacyRandom},
    };

    use super::{move_to_next_pack_position, select_weighted_spawner};

    fn spawner(path: &'static str, weight: i32) -> SpawnerData {
        SpawnerData {
            entity_type: Identifier::vanilla_static(path),
            weight,
            min_count: 1,
            max_count: 1,
        }
    }

    #[test]
    fn weighted_selection_uses_vanilla_list_order() {
        let spawners = [
            spawner("sheep", 12),
            spawner("pig", 10),
            spawner("chicken", 10),
            spawner("cow", 8),
        ];
        let mut random = LegacyRandom::from_seed(0);
        let selected: Vec<_> = (0..6)
            .filter_map(|_| select_weighted_spawner(&spawners, &mut random))
            .map(|entry| entry.entity_type.path.as_str())
            .collect();

        assert_eq!(selected, ["sheep", "chicken", "chicken", "sheep", "cow", "pig"]);
    }

    #[test]
    fn pack_position_retry_consumes_legacy_random_in_vanilla_order() {
        let mut random = LegacyRandom::from_seed(42);
        let _ = random.next_f32();
        let mut x = 15;
        let mut z = 15;
        move_to_next_pack_position(&mut random, 0, 0, 15, 15, &mut x, &mut z);
        assert_eq!((x, z), (15, 14));
    }
}
