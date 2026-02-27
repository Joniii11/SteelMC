//! Shared fields for all living entities.
//!
//! Mirrors the fields that vanilla defines on `LivingEntity` (and `Entity` for
//! `invulnerableTime`). Entities that implement `LivingEntity` embed this
//! struct and expose it via `LivingEntity::living_base()`, just like
//! `EntityBase` is used for core `Entity` fields.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;

/// Duration in ticks of the death animation before entity removal.
pub const DEATH_DURATION: i32 = 20;

/// Common fields shared by all living entities.
pub struct LivingEntityBase {
    /// if the entity is dead or not.
    dead: AtomicBool,
    /// The time where the entity can't be hurt and is invulnerable.
    invulnerable_time: AtomicI32,
    /// When the entity was last hurt.
    last_hurt: AtomicCell<f32>,
    /// Ticks since the entity died. Incremented each tick while dead/dying.
    death_time: AtomicI32,
}

impl LivingEntityBase {
    /// Creates a new `LivingEntityBase` with default values (alive, no invulnerability, no hurt).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dead: AtomicBool::new(false),
            invulnerable_time: AtomicI32::new(0),
            last_hurt: AtomicCell::new(0.0),
            death_time: AtomicI32::new(0),
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

    /// Ticks since the entity died. Vanilla: `LivingEntity.deathTime` (L217).
    #[inline]
    pub fn get_death_time(&self) -> i32 {
        self.death_time.load(Ordering::Relaxed)
    }

    /// Sets the death time counter directly.
    #[inline]
    pub fn set_death_time(&self, ticks: i32) {
        self.death_time.store(ticks, Ordering::Relaxed);
    }

    /// Increments `death_time` by 1 and returns the new value.
    #[inline]
    pub fn increment_death_time(&self) -> i32 {
        self.death_time.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Resets all death-related state back to alive defaults.
    #[inline]
    pub fn reset_death_state(&self) {
        self.dead.store(false, Ordering::Relaxed);
        self.death_time.store(0, Ordering::Relaxed);
        self.invulnerable_time.store(0, Ordering::Relaxed);
        self.last_hurt.store(0.0);
    }
}

impl Default for LivingEntityBase {
    fn default() -> Self {
        Self::new()
    }
}
