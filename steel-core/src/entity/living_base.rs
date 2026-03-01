//! Shared fields for all living entities.
//!
//! Mirrors the fields that vanilla defines on `LivingEntity` (and `Entity` for
//! `invulnerableTime`). Entities that implement `LivingEntity` embed this
//! struct and expose it via `LivingEntity::living_base()`, just like
//! `EntityBase` is used for core `Entity` fields.

/// Duration in ticks of the death animation before entity removal.
pub const DEATH_DURATION: i32 = 20;

/// Common fields shared by all living entities.
pub struct LivingEntityBase {
    /// Whether the entity has been killed.
    pub dead: bool,
    /// Remaining invulnerability ticks.
    pub invulnerable_time: i32,
    /// Last damage amount for invulnerability-frame comparison.
    pub last_hurt: f32,
    /// Ticks since the entity died. Incremented each tick while dead/dying.
    pub death_time: i32,
}

impl LivingEntityBase {
    /// Creates a new `LivingEntityBase` with default values (alive, no invulnerability, no hurt).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dead: false,
            invulnerable_time: 0,
            last_hurt: 0.0,
            death_time: 0,
        }
    }

    /// Increments `death_time` by 1 and returns the new value.
    #[inline]
    pub const fn increment_death_time(&mut self) -> i32 {
        self.death_time += 1;
        self.death_time
    }

    /// Resets all death-related state back to alive defaults.
    #[inline]
    pub const fn reset_death_state(&mut self) {
        self.dead = false;
        self.death_time = 0;
        self.invulnerable_time = 0;
        self.last_hurt = 0.0;
    }
}

impl Default for LivingEntityBase {
    fn default() -> Self {
        Self::new()
    }
}
