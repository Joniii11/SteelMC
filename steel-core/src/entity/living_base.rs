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
    /// if the entity is dead or not.
    dead: bool,
    /// The time where the entity can't be hurt and is invulnerable.
    invulnerable_time: i32,
    /// When the entity was last hurt.
    last_hurt: f32,
    /// Ticks since the entity died. Incremented each tick while dead/dying.
    death_time: i32,
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

    /// Whether the entity has been killed. Vanilla: `LivingEntity.dead` (L230).
    #[inline]
    #[must_use]
    pub const fn is_dead(&self) -> bool {
        self.dead
    }

    /// Sets the dead flag.
    #[inline]
    pub const fn set_dead(&mut self, dead: bool) {
        self.dead = dead;
    }

    /// Remaining invulnerability ticks. Vanilla: `Entity.invulnerableTime` (L256).
    #[inline]
    #[must_use]
    pub const fn get_invulnerable_time(&self) -> i32 {
        self.invulnerable_time
    }

    /// Sets invulnerability ticks.
    #[inline]
    pub const fn set_invulnerable_time(&mut self, ticks: i32) {
        self.invulnerable_time = ticks;
    }

    /// Last damage amount for invulnerability-frame comparison. Vanilla: `LivingEntity.lastHurt` (L232).
    #[inline]
    #[must_use]
    pub const fn get_last_hurt(&self) -> f32 {
        self.last_hurt
    }

    /// Sets the last hurt amount.
    #[inline]
    pub const fn set_last_hurt(&mut self, amount: f32) {
        self.last_hurt = amount;
    }

    /// Ticks since the entity died. Vanilla: `LivingEntity.deathTime` (L217).
    #[inline]
    #[must_use]
    pub const fn get_death_time(&self) -> i32 {
        self.death_time
    }

    /// Sets the death time counter directly.
    #[inline]
    pub const fn set_death_time(&mut self, ticks: i32) {
        self.death_time = ticks;
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
