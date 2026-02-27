//! Shared fields for all living entities.
//!
//! Mirrors the fields that vanilla defines on `LivingEntity` (and `Entity` for
//! `invulnerableTime`). Entities that implement `LivingEntity` embed this
//! struct and expose it via `LivingEntity::living_base()`, just like
//! `EntityBase` is used for core `Entity` fields.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;

/// Common fields shared by all living entities.
///
/// Vanilla locations:
/// - `LivingEntity.dead` (L230)
/// - `LivingEntity.lastHurt` (L232)
/// - `Entity.invulnerableTime` (L256)
pub struct LivingEntityBase {
    dead: AtomicBool,
    invulnerable_time: AtomicI32,
    last_hurt: AtomicCell<f32>,
}

impl LivingEntityBase {
    /// Creates a new `LivingEntityBase` with default values (alive, no invulnerability, no hurt).
    #[must_use]
    pub fn new() -> Self {
        Self {
            dead: AtomicBool::new(false),
            invulnerable_time: AtomicI32::new(0),
            last_hurt: AtomicCell::new(0.0),
        }
    }

    /// Whether the entity has been killed. Vanilla: `LivingEntity.dead` (L230).
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }

    /// Sets the dead flag.
    #[inline]
    pub fn set_dead(&self, dead: bool) {
        self.dead.store(dead, Ordering::Relaxed);
    }

    /// Remaining invulnerability ticks. Vanilla: `Entity.invulnerableTime` (L256).
    #[inline]
    pub fn get_invulnerable_time(&self) -> i32 {
        self.invulnerable_time.load(Ordering::Relaxed)
    }

    /// Sets invulnerability ticks.
    #[inline]
    pub fn set_invulnerable_time(&self, ticks: i32) {
        self.invulnerable_time.store(ticks, Ordering::Relaxed);
    }

    /// Last damage amount for invulnerability-frame comparison. Vanilla: `LivingEntity.lastHurt` (L232).
    #[inline]
    pub fn get_last_hurt(&self) -> f32 {
        self.last_hurt.load()
    }

    /// Sets the last hurt amount.
    #[inline]
    pub fn set_last_hurt(&self, amount: f32) {
        self.last_hurt.store(amount);
    }
}

impl Default for LivingEntityBase {
    fn default() -> Self {
        Self::new()
    }
}
