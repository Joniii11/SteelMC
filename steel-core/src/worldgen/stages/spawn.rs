use std::sync::Arc;

use crate::chunk::{
    chunk_generation_task::StaticCache2D, chunk_holder::ChunkHolder, chunk_pyramid::ChunkStep,
};
use crate::worldgen::generator::ChunkGenerator as _;
use crate::worldgen::generator::context::WorldGenContext;
use crate::worldgen::region::WorldGenRegion;

pub(crate) fn generate(
    context: Arc<WorldGenContext>,
    step: &ChunkStep,
    cache: &Arc<StaticCache2D<Arc<ChunkHolder>>>,
    holder: Arc<ChunkHolder>,
) {
    let center = holder.get_pos();
    let world_seed = context.world().seed();
    let region_random = context
        .generator
        .create_worldgen_region_random(world_seed, center);
    let region = WorldGenRegion::new(context.as_ref(), step, cache, center, region_random);
    context.generator.spawn_original_mobs(&region);
}
