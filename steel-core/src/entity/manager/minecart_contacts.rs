//! A synchronous broad phase for the zero-distance minecart push.

use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use steel_utils::{Downcast as _, WorldAabb};

use super::{
    EntityCollisionCandidates, EntitySpatialCellBounds, MAX_DIRECT_SPATIAL_CELL_PROBES,
    ManagerState, WorldEntityManager,
};
use crate::entity::Entity;
use crate::entity::entities::{MinecartEntity, TntMinecartEntity};

// Both concrete implementations delegate their push callback to AbstractMinecart.
fn uses_native_push(entity: &dyn Entity) -> bool {
    entity.is::<MinecartEntity>() || entity.is::<TntMinecartEntity>()
}

pub(super) struct CoincidentCartQuery {
    bounds: EntitySpatialCellBounds,
    /// None caches a mixed or empty region until a relevant input changes.
    common_xz: Option<[u64; 2]>,
    valid: Arc<AtomicBool>,
}

impl Drop for CoincidentCartQuery {
    fn drop(&mut self) {
        // Readers may still hold a token while checking the world's multipart index.
        self.valid.store(false, Ordering::Release);
    }
}

impl CoincidentCartQuery {
    fn inspect(state: &ManagerState, hitbox: &WorldAabb, bounds: EntitySpatialCellBounds) -> Self {
        let valid = Arc::new(AtomicBool::new(true));
        // Include every entry in the broad-phase cells regardless of accessibility or
        // exact bounds. Motion within those cells cannot introduce an unobserved candidate.
        let entries =
            WorldEntityManager::query_entries(state, hitbox, EntityCollisionCandidates::All, None);
        let mut positions = entries.map(|entry| {
            let entity = entry.entity.as_ref();
            if !uses_native_push(entity) {
                return None;
            }
            entity.base().observe_minecart_contacts(&valid)
        });
        let common_xz = positions
            .next()
            .flatten()
            .filter(|xz| positions.all(|other| other == Some(*xz)));
        Self {
            bounds,
            common_xz,
            valid,
        }
    }

    fn for_source(&self, state: &ManagerState, source: &dyn Entity) -> Option<Arc<AtomicBool>> {
        let xz = self.common_xz?;
        let live = state.live_by_id.get(source.id())?.entity.as_ref();
        if !ptr::addr_eq(live, source)
            || source.base().observe_minecart_contacts(&self.valid) != Some(xz)
        {
            return None;
        }
        self.valid
            .load(Ordering::Acquire)
            .then(|| Arc::clone(&self.valid))
    }
}

impl ManagerState {
    fn current_cart_query(&self, bounds: EntitySpatialCellBounds) -> Option<&CoincidentCartQuery> {
        self.coincident_cart_query
            .as_ref()
            .filter(|query| query.bounds == bounds && query.valid.load(Ordering::Acquire))
    }
}

impl WorldEntityManager {
    /// On success all indexed candidates share the source's exact X/Z and use the
    /// native minecart push. Recheck the returned token after consulting any other index.
    pub(crate) fn coincident_minecarts(
        &self,
        hitbox: &WorldAabb,
        source: &dyn Entity,
    ) -> Option<Arc<AtomicBool>> {
        if !uses_native_push(source) {
            return None;
        }
        let bounds = EntitySpatialCellBounds::from_aabb(hitbox);
        if bounds.direct_probe_count() > MAX_DIRECT_SPATIAL_CELL_PROBES {
            return None;
        }
        {
            let state = self.state.read();
            if let Some(query) = state.current_cart_query(bounds) {
                return query.for_source(&state, source);
            }
            // An isolated cart doesn't benefit from a cache allocation or a manager write lock.
            if !bounds.cells().iter().any(|cell| {
                state
                    .by_spatial_cell
                    .get(cell)
                    .is_some_and(|entries| entries.entities.len() > 1)
            }) {
                return None;
            }
        }
        let mut state = self.state.write();
        if state.current_cart_query(bounds).is_none() {
            state.coincident_cart_query =
                Some(CoincidentCartQuery::inspect(&state, hitbox, bounds));
        }
        state
            .coincident_cart_query
            .as_ref()?
            .for_source(&state, source)
    }
}
