//! Optional observation of horizontal motion and direct passenger occupancy.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::{mem, ptr};

use steel_utils::locks::SyncMutex;

use super::{EntityBase, EntityRelationshipState};

/// The position or relationship lock always precedes this lock. No observer
/// operation calls entity code or acquires another lock.
#[derive(Default)]
pub(super) struct SpatialQueryObserver {
    subscription: Weak<AtomicBool>,
    has_passengers: bool,
}

impl SpatialQueryObserver {
    pub(super) fn invalidate(&mut self) {
        if let Some(valid) = mem::take(&mut self.subscription).upgrade() {
            valid.store(false, Ordering::Release);
        }
    }
}

struct PassengerListUpdate<'a> {
    was_empty: bool,
    relationships: &'a mut EntityRelationshipState,
    observer: &'a SyncMutex<SpatialQueryObserver>,
}

impl Drop for PassengerListUpdate<'_> {
    fn drop(&mut self) {
        let is_empty = self.relationships.passengers.is_empty();
        if self.was_empty != is_empty {
            let mut observer = self.observer.lock();
            observer.invalidate();
            observer.has_passengers = !is_empty;
        }
    }
}

impl EntityBase {
    /// Enables observation for a concrete implementation that participates in cached queries.
    #[must_use]
    pub(in crate::entity) fn with_spatial_query_observer(mut self) -> Self {
        self.spatial_observer = Some(Box::new(SyncMutex::new(SpatialQueryObserver {
            has_passengers: !self.relationships.get_mut().passengers.is_empty(),
            ..SpatialQueryObserver::default()
        })));
        self
    }

    /// Even an occupied cart subscribes: dismounting can make a cached miss obsolete.
    pub(in crate::entity) fn observe_minecart_contacts(
        &self,
        valid: &Arc<AtomicBool>,
    ) -> Option<[u64; 2]> {
        let observer = self.spatial_observer.as_ref()?;
        let state = self.state.lock();
        let mut observer = observer.lock();
        if !ptr::eq(observer.subscription.as_ptr(), Arc::as_ptr(valid)) {
            observer.invalidate();
            observer.subscription = Arc::downgrade(valid);
        }
        (!observer.has_passengers).then(|| [state.position.x.to_bits(), state.position.z.to_bits()])
    }

    /// Passenger getters can also prune expired weak references. Publish changes
    /// in occupancy before the list unlocks, including when a callback unwinds.
    pub(super) fn with_passenger_list<T>(
        &self,
        visit: impl FnOnce(&mut EntityRelationshipState) -> T,
    ) -> T {
        let mut relationships = self.relationships.lock();
        let Some(observer) = &self.spatial_observer else {
            return visit(&mut relationships);
        };
        let update = PassengerListUpdate {
            was_empty: relationships.passengers.is_empty(),
            relationships: &mut relationships,
            observer,
        };
        visit(update.relationships)
    }
}
