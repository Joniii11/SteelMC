use super::super::prelude::*;
use super::super::runner::FeatureDecorationRunner;

impl FeatureDecorationRunner {
    pub(in crate::worldgen::feature) fn place_end_podium_feature(
        region: &mut WorldGenRegion<'_>,
        config: &EndPodiumConfiguration,
        origin: BlockPos,
    ) -> bool {
        let bedrock = vanilla_blocks::BEDROCK.default_state();
        let end_stone = vanilla_blocks::END_STONE.default_state();
        let air = vanilla_blocks::AIR.default_state();
        let end_portal = vanilla_blocks::END_PORTAL.default_state();

        for x in origin.x() - 4..=origin.x() + 4 {
            for y in origin.y() - 1..=origin.y() + 32 {
                for z in origin.z() - 4..=origin.z() + 4 {
                    let pos = BlockPos::new(x, y, z);
                    let inside_rim = closer_than(pos, origin, 2.5);
                    if !inside_rim && !closer_than(pos, origin, 3.5) {
                        continue;
                    }

                    let state = if y < origin.y() {
                        if inside_rim { bedrock } else { end_stone }
                    } else if y > origin.y() {
                        air
                    } else if inside_rim {
                        if config.active { end_portal } else { air }
                    } else {
                        bedrock
                    };
                    let _ = region.set_block_state(pos, state, UpdateFlags::UPDATE_CLIENTS);
                }
            }
        }

        for y in 0..4 {
            let _ = region.set_block_state(origin.above_n(y), bedrock, UpdateFlags::UPDATE_CLIENTS);
        }

        let center_of_pillar = origin.above_n(2);
        for facing in [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            let state = vanilla_blocks::WALL_TORCH
                .default_state()
                .set_value(&BlockStateProperties::HORIZONTAL_FACING, facing);
            let offset = facing.offset_vec();
            let pos = center_of_pillar.offset(offset.x, offset.y, offset.z);
            let _ = region.set_block_state(pos, state, UpdateFlags::UPDATE_CLIENTS);
        }

        true
    }
}

fn closer_than(pos: BlockPos, origin: BlockPos, distance: f64) -> bool {
    let dx = f64::from(pos.x() - origin.x());
    let dy = f64::from(pos.y() - origin.y());
    let dz = f64::from(pos.z() - origin.z());
    dx * dx + dy * dy + dz * dz < distance * distance
}
