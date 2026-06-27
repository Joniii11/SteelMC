use super::prelude::*;
use super::runner::FeatureDecorationRunner;
use crate::worldgen::template::{
    StructurePlaceSettings, StructureProcessorRandom, StructureTemplate,
};
use glam::IVec3;
use steel_registry::structure::LiquidSettingsData;
use steel_utils::BoundingBox;
use steel_worldgen::structure::{StructureBlockIgnore, StructureMirror};

struct FeaturePlaceContext<'a, 'region> {
    region: &'a mut WorldGenRegion<'region>,
    registry: &'a Registry,
    random: &'a mut WorldgenRandom,
    origin: BlockPos,
    biome_zoom_seed: i64,
}

type FeaturePlacer =
    for<'a, 'region> fn(&mut FeaturePlaceContext<'a, 'region>, &FeatureKind) -> bool;

impl FeatureDecorationRunner {
    pub(super) fn place_feature(
        region: &mut WorldGenRegion<'_>,
        registry: &Registry,
        random: &mut WorldgenRandom,
        feature: &FeatureRef,
        origin: BlockPos,
        biome_zoom_seed: i64,
    ) -> bool {
        let kind = Self::feature_kind(feature);
        Self::place_feature_kind(region, registry, random, kind, origin, biome_zoom_seed)
    }

    pub(super) fn place_feature_kind(
        region: &mut WorldGenRegion<'_>,
        registry: &Registry,
        random: &mut WorldgenRandom,
        kind: &FeatureKind,
        origin: BlockPos,
        biome_zoom_seed: i64,
    ) -> bool {
        if !region.can_write_to_chunk(
            SectionPos::block_to_section_coord(origin.x()),
            SectionPos::block_to_section_coord(origin.z()),
        ) {
            return false;
        }

        let placer = Self::feature_placer(kind);
        let mut context = FeaturePlaceContext {
            region,
            registry,
            random,
            origin,
            biome_zoom_seed,
        };
        placer(&mut context, kind)
    }

    pub(super) fn feature_kind(feature: &FeatureRef) -> &FeatureKind {
        match feature {
            FeatureRef::Reference(feature) => &feature.kind,
            FeatureRef::Inline(feature) => feature,
        }
    }

    fn feature_placer(kind: &FeatureKind) -> FeaturePlacer {
        match kind {
            FeatureKind::Bamboo(_) => place_bamboo,
            FeatureKind::BasaltColumns(_) => place_basalt_columns,
            FeatureKind::BasaltPillar => place_basalt_pillar,
            FeatureKind::BlockBlob(_) => place_block_blob,
            FeatureKind::BlockColumn(_) => place_block_column,
            FeatureKind::BlockPile(_) => place_block_pile,
            FeatureKind::BlueIce => place_blue_ice,
            FeatureKind::BonusChest => place_bonus_chest,
            FeatureKind::ChorusPlant => place_chorus_plant,
            FeatureKind::CoralClaw => place_coral_claw,
            FeatureKind::CoralMushroom => place_coral_mushroom,
            FeatureKind::CoralTree => place_coral_tree,
            FeatureKind::DeltaFeature(_) => place_delta_feature,
            FeatureKind::DesertWell => place_desert_well,
            FeatureKind::Disk(_) => place_disk,
            FeatureKind::DripstoneCluster(_) => place_dripstone_cluster,
            FeatureKind::EndGateway(_) => place_end_gateway,
            FeatureKind::EndIsland => place_end_island,
            FeatureKind::EndPlatform => place_end_platform,
            FeatureKind::EndPodium(_) => place_end_podium,
            FeatureKind::EndSpike(_) => place_end_spike,
            FeatureKind::FallenTree(_) => place_fallen_tree,
            FeatureKind::Fossil(_) => place_fossil,
            FeatureKind::FreezeTopLayer => place_freeze_top_layer,
            FeatureKind::Geode(_) => place_geode,
            FeatureKind::GlowstoneBlob => place_glowstone_blob,
            FeatureKind::HugeBrownMushroom(_) => place_huge_brown_mushroom,
            FeatureKind::HugeFungus(_) => place_huge_fungus,
            FeatureKind::HugeRedMushroom(_) => place_huge_red_mushroom,
            FeatureKind::Iceberg(_) => place_iceberg,
            FeatureKind::Kelp => place_kelp,
            FeatureKind::Lake(_) => place_lake,
            FeatureKind::LargeDripstone(_) => place_large_dripstone,
            FeatureKind::MonsterRoom => place_monster_room,
            FeatureKind::MultifaceGrowth(_) => place_multiface_growth,
            FeatureKind::NetherForestVegetation(_) => place_nether_forest_vegetation,
            FeatureKind::NetherrackReplaceBlobs(_) => place_netherrack_replace_blobs,
            FeatureKind::Ore(_) => place_ore,
            FeatureKind::PointedDripstone(_) => place_pointed_dripstone,
            FeatureKind::RandomBooleanSelector(_) => place_random_boolean_selector,
            FeatureKind::RandomSelector(_) => place_random_selector,
            FeatureKind::WeightedRandomSelector(_) => place_weighted_random_selector,
            FeatureKind::RootSystem(_) => place_root_system,
            FeatureKind::ScatteredOre(_) => place_scattered_ore,
            FeatureKind::SculkPatch(_) => place_sculk_patch,
            FeatureKind::SeaPickle(_) => place_sea_pickle,
            FeatureKind::Seagrass(_) => place_seagrass,
            FeatureKind::Sequence(_) => place_sequence,
            FeatureKind::SimpleBlock(_) => place_simple_block,
            FeatureKind::SimpleRandomSelector(_) => place_simple_random_selector,
            FeatureKind::Speleothem(_) => place_speleothem,
            FeatureKind::SpeleothemCluster(_) => place_speleothem_cluster,
            FeatureKind::Spike(_) => place_spike,
            FeatureKind::SpringFeature(_) => place_spring_feature,
            FeatureKind::Template(_) => place_template,
            FeatureKind::Tree(_) => place_tree,
            FeatureKind::TwistingVines(_) => place_twisting_vines,
            FeatureKind::UnderwaterMagma(_) => place_underwater_magma,
            FeatureKind::VegetationPatch(_) => place_vegetation_patch,
            FeatureKind::Vines => place_vines,
            FeatureKind::VoidStartPlatform => place_void_start_platform,
            FeatureKind::WaterloggedVegetationPatch(_) => place_waterlogged_vegetation_patch,
            FeatureKind::WeepingVines => place_weeping_vines,
        }
    }
}

fn place_random_boolean_selector(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::RandomBooleanSelector(config) = kind else {
        panic!("random_boolean_selector placer received wrong feature kind");
    };
    let selected_feature = if context.random.next_bool() {
        &config.feature_true
    } else {
        &config.feature_false
    };
    FeatureDecorationRunner::place_placed_feature_ref(
        context.region,
        context.registry,
        context.random,
        context.origin,
        selected_feature,
        context.biome_zoom_seed,
    )
}

fn place_random_selector(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::RandomSelector(config) = kind else {
        panic!("random_selector placer received wrong feature kind");
    };
    for weighted_feature in &config.features {
        let roll = context.random.next_f32();
        if roll < weighted_feature.chance {
            return FeatureDecorationRunner::place_placed_feature_ref(
                context.region,
                context.registry,
                context.random,
                context.origin,
                &weighted_feature.feature,
                context.biome_zoom_seed,
            );
        }
    }

    FeatureDecorationRunner::place_placed_feature_ref(
        context.region,
        context.registry,
        context.random,
        context.origin,
        &config.default,
        context.biome_zoom_seed,
    )
}

fn place_weighted_random_selector(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::WeightedRandomSelector(config) = kind else {
        panic!("weighted_random_selector placer received wrong feature kind");
    };
    let Some(feature_index) = weighted_index(
        config.features.len(),
        |index| config.features[index].weight,
        context.random,
    ) else {
        return false;
    };
    FeatureDecorationRunner::place_placed_feature_ref(
        context.region,
        context.registry,
        context.random,
        context.origin,
        &config.features[feature_index].data,
        context.biome_zoom_seed,
    )
}

fn place_simple_random_selector(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::SimpleRandomSelector(config) = kind else {
        panic!("simple_random_selector placer received wrong feature kind");
    };
    assert!(
        !config.features.is_empty(),
        "simple random selector feature list must not be empty"
    );
    let Ok(feature_count) = i32::try_from(config.features.len()) else {
        panic!(
            "simple random selector feature count {} exceeds i32 range",
            config.features.len()
        );
    };
    let feature_index = context.random.next_i32_bounded(feature_count) as usize;
    FeatureDecorationRunner::place_placed_feature_ref(
        context.region,
        context.registry,
        context.random,
        context.origin,
        &config.features[feature_index],
        context.biome_zoom_seed,
    )
}

fn place_sequence(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Sequence(config) = kind else {
        panic!("sequence placer received wrong feature kind");
    };
    for feature in &config.features {
        if !FeatureDecorationRunner::place_placed_feature_ref(
            context.region,
            context.registry,
            context.random,
            context.origin,
            feature,
            context.biome_zoom_seed,
        ) {
            return false;
        }
    }
    true
}

fn place_template(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Template(config) = kind else {
        panic!("template placer received wrong feature kind");
    };
    let Some(template_index) = weighted_index(
        config.templates.len(),
        |index| config.templates[index].weight,
        context.random,
    ) else {
        panic!("template feature has no selectable templates");
    };
    let entry = &config.templates[template_index].data;
    let Ok(rotation_count) = i32::try_from(entry.rotations.len()) else {
        panic!(
            "template feature rotation count {} exceeds i32 range",
            entry.rotations.len()
        );
    };
    assert!(
        rotation_count != 0,
        "template feature entry {} has no rotations",
        entry.id
    );
    let rotation = entry.rotations[context.random.next_i32_bounded(rotation_count) as usize];
    let template = match StructureTemplate::load_vanilla(context.registry, &entry.id) {
        Ok(template) => template,
        Err(err) => panic!("{err}"),
    };
    let size = template.size(Rotation::None);
    let position = template_feature_position(context.origin, rotation, size);
    let settings = StructurePlaceSettings {
        mirror: StructureMirror::None,
        rotation,
        rotation_pivot: BlockPos::ZERO,
        bounding_box: template_feature_bounding_box(context.region),
        processors: &[],
        block_ignore: StructureBlockIgnore::None,
        late_block_ignore: StructureBlockIgnore::None,
        replace_jigsaws: false,
        projection: None,
        processor_random: StructureProcessorRandom::Placement,
        liquid_settings: LiquidSettingsData::ApplyWaterlogging,
    };

    template.place_in_world(
        context.region,
        context.registry,
        position,
        position,
        &settings,
        context.random,
        UpdateFlags::UPDATE_ALL,
    )
}

fn weighted_index(
    len: usize,
    weight_at: impl Fn(usize) -> u32,
    random: &mut WorldgenRandom,
) -> Option<usize> {
    let total_weight = (0..len)
        .map(|index| u64::from(weight_at(index)))
        .sum::<u64>();
    if total_weight == 0 {
        return None;
    }
    let Ok(total_weight_i32) = i32::try_from(total_weight) else {
        panic!("weighted feature total weight {total_weight} exceeds i32 range");
    };
    let mut selection = random.next_i32_bounded(total_weight_i32) as u64;
    for index in 0..len {
        let weight = u64::from(weight_at(index));
        if selection < weight {
            return Some(index);
        }
        selection -= weight;
    }
    None
}

const fn template_feature_position(origin: BlockPos, rotation: Rotation, size: IVec3) -> BlockPos {
    let west_offset = rotation.rotate(Direction::West).offset();
    let north_offset = rotation.rotate(Direction::North).offset();
    origin.offset(
        west_offset.0 * (size.x / 2) + north_offset.0 * (size.z / 2),
        0,
        west_offset.2 * (size.x / 2) + north_offset.2 * (size.z / 2),
    )
}

const fn template_feature_bounding_box(region: &WorldGenRegion<'_>) -> BoundingBox {
    BoundingBox::new(
        IVec3::new(i32::MIN, region.min_y(), i32::MIN),
        IVec3::new(i32::MAX, region.max_y_exclusive() - 1, i32::MAX),
    )
}

fn place_bamboo(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Bamboo(config) = kind else {
        panic!("bamboo placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_bamboo_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_simple_block(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::SimpleBlock(config) = kind else {
        panic!("simple_block placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_simple_block_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_block_blob(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BlockBlob(config) = kind else {
        panic!("block_blob placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_block_blob_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_vegetation_patch(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::VegetationPatch(config) = kind else {
        panic!("vegetation_patch placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_vegetation_patch_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
        context.biome_zoom_seed,
    )
}

fn place_waterlogged_vegetation_patch(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::WaterloggedVegetationPatch(config) = kind else {
        panic!("waterlogged_vegetation_patch placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_waterlogged_vegetation_patch_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
        context.biome_zoom_seed,
    )
}

fn place_block_column(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BlockColumn(config) = kind else {
        panic!("block_column placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_block_column_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_block_pile(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BlockPile(config) = kind else {
        panic!("block_pile placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_block_pile_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_disk(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Disk(config) = kind else {
        panic!("disk placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_disk_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_basalt_pillar(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BasaltPillar = kind else {
        panic!("basalt_pillar placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_basalt_pillar_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_basalt_columns(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BasaltColumns(config) = kind else {
        panic!("basalt_columns placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_basalt_columns_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_blue_ice(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BlueIce = kind else {
        panic!("blue_ice placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_blue_ice_feature(context.region, context.random, context.origin)
}

fn place_bonus_chest(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::BonusChest = kind else {
        panic!("bonus_chest placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_bonus_chest_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_chorus_plant(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::ChorusPlant = kind else {
        panic!("chorus_plant placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_chorus_plant_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_coral_claw(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::CoralClaw = kind else {
        panic!("coral_claw placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_coral_claw_feature(
        context.region,
        context.registry,
        context.random,
        context.origin,
    )
}

fn place_coral_mushroom(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::CoralMushroom = kind else {
        panic!("coral_mushroom placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_coral_mushroom_feature(
        context.region,
        context.registry,
        context.random,
        context.origin,
    )
}

fn place_coral_tree(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::CoralTree = kind else {
        panic!("coral_tree placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_coral_tree_feature(
        context.region,
        context.registry,
        context.random,
        context.origin,
    )
}

fn place_delta_feature(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::DeltaFeature(config) = kind else {
        panic!("delta_feature placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_delta_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_desert_well(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::DesertWell = kind else {
        panic!("desert_well placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_desert_well_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_end_gateway(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::EndGateway(config) = kind else {
        panic!("end_gateway placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_end_gateway_feature(context.region, config, context.origin)
}

fn place_end_island(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::EndIsland = kind else {
        panic!("end_island placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_end_island_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_end_platform(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::EndPlatform = kind else {
        panic!("end_platform placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_end_platform_feature(context.region, context.origin)
}

fn place_end_podium(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::EndPodium(config) = kind else {
        panic!("end_podium placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_end_podium_feature(context.region, config, context.origin)
}

fn place_end_spike(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::EndSpike(config) = kind else {
        panic!("end_spike placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_end_spike_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_geode(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Geode(config) = kind else {
        panic!("geode placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_geode_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_glowstone_blob(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::GlowstoneBlob = kind else {
        panic!("glowstone_blob placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_glowstone_blob_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_huge_brown_mushroom(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::HugeBrownMushroom(config) = kind else {
        panic!("huge_brown_mushroom placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_huge_brown_mushroom_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_huge_red_mushroom(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::HugeRedMushroom(config) = kind else {
        panic!("huge_red_mushroom placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_huge_red_mushroom_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_huge_fungus(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::HugeFungus(config) = kind else {
        panic!("huge_fungus placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_huge_fungus_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_iceberg(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Iceberg(config) = kind else {
        panic!("iceberg placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_iceberg_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_netherrack_replace_blobs(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::NetherrackReplaceBlobs(config) = kind else {
        panic!("netherrack_replace_blobs placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_netherrack_replace_blobs_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_nether_forest_vegetation(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::NetherForestVegetation(config) = kind else {
        panic!("nether_forest_vegetation placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_nether_forest_vegetation_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_twisting_vines(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::TwistingVines(config) = kind else {
        panic!("twisting_vines placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_twisting_vines_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_vines(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Vines = kind else {
        panic!("vines placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_vines_feature(context.region, context.origin)
}

fn place_void_start_platform(
    context: &mut FeaturePlaceContext<'_, '_>,
    kind: &FeatureKind,
) -> bool {
    let FeatureKind::VoidStartPlatform = kind else {
        panic!("void_start_platform placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_void_start_platform_feature(context.region, context.origin)
}

fn place_weeping_vines(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::WeepingVines = kind else {
        panic!("weeping_vines placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_weeping_vines_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_spring_feature(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::SpringFeature(config) = kind else {
        panic!("spring_feature placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_spring_feature(
        context.region,
        context.registry,
        config,
        context.origin,
    )
}

fn place_kelp(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Kelp = kind else {
        panic!("kelp placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_kelp_feature(context.region, context.random, context.origin)
}

fn place_lake(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Lake(config) = kind else {
        panic!("lake placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_lake_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
        context.biome_zoom_seed,
    )
}

fn place_monster_room(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::MonsterRoom = kind else {
        panic!("monster_room placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_monster_room_feature(
        context.region,
        context.random,
        context.origin,
    )
}

fn place_freeze_top_layer(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::FreezeTopLayer = kind else {
        panic!("freeze_top_layer placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_freeze_top_layer_feature(
        context.region,
        context.registry,
        context.origin,
        context.biome_zoom_seed,
    )
}

fn place_multiface_growth(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::MultifaceGrowth(config) = kind else {
        panic!("multiface_growth placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_multiface_growth_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_sea_pickle(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::SeaPickle(config) = kind else {
        panic!("sea_pickle placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_sea_pickle_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_seagrass(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Seagrass(config) = kind else {
        panic!("seagrass placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_seagrass_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_underwater_magma(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::UnderwaterMagma(config) = kind else {
        panic!("underwater_magma placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_underwater_magma_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_pointed_dripstone(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::PointedDripstone(config) = kind else {
        panic!("pointed_dripstone placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_pointed_dripstone_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_dripstone_cluster(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::DripstoneCluster(config) = kind else {
        panic!("dripstone_cluster placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_dripstone_cluster_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_speleothem(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Speleothem(config) = kind else {
        panic!("speleothem placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_speleothem_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_speleothem_cluster(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::SpeleothemCluster(config) = kind else {
        panic!("speleothem_cluster placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_speleothem_cluster_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_large_dripstone(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::LargeDripstone(config) = kind else {
        panic!("large_dripstone placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_large_dripstone_feature(
        context.region,
        context.random,
        config,
        context.origin,
    )
}

fn place_spike(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Spike(config) = kind else {
        panic!("spike placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_spike_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_ore(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Ore(config) = kind else {
        panic!("ore placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_ore_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_scattered_ore(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::ScatteredOre(config) = kind else {
        panic!("scattered_ore placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_scattered_ore_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_sculk_patch(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::SculkPatch(config) = kind else {
        panic!("sculk_patch placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_sculk_patch_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_tree(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Tree(config) = kind else {
        panic!("tree placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_tree_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
        context.biome_zoom_seed,
    )
}

fn place_fallen_tree(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::FallenTree(config) = kind else {
        panic!("fallen_tree placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_fallen_tree_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
        context.biome_zoom_seed,
    )
}

fn place_fossil(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::Fossil(config) = kind else {
        panic!("fossil placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_fossil_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
    )
}

fn place_root_system(context: &mut FeaturePlaceContext<'_, '_>, kind: &FeatureKind) -> bool {
    let FeatureKind::RootSystem(config) = kind else {
        panic!("root_system placer received wrong feature kind");
    };
    FeatureDecorationRunner::place_root_system_feature(
        context.region,
        context.registry,
        context.random,
        config,
        context.origin,
        context.biome_zoom_seed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_feature_position_uses_template_depth_for_north_offset() {
        assert_eq!(
            template_feature_position(
                BlockPos::new(100, 64, 200),
                Rotation::None,
                IVec3::new(10, 30, 6),
            ),
            BlockPos::new(95, 64, 197)
        );
    }
}
