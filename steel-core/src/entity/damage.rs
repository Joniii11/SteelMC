//! Damage source system - represents how an entity was damaged.
//!
//! Based on vanilla's `DamageSource` class. A `DamageSource` describes the type
//! of damage, who caused it, and optionally where it originated.

use steel_registry::damage_type::DamageType;
use steel_utils::math::Vector3;

/// Represents the source of damage dealt to an entity.
///
/// This is used to determine:
/// - What type of damage was dealt (for death messages, armor calculations, etc.)
/// - Who caused the damage (for kill attribution)
/// - Where the damage came from (for knockback direction)
#[derive(Debug, Clone)]
pub struct DamageSource {
    /// The damage type from the registry (e.g., out_of_world, fall, lava).
    pub damage_type: &'static DamageType,
    /// The entity ID of the entity ultimately responsible for the damage.
    /// For projectiles, this would be the shooter. For direct attacks, the attacker.
    pub causing_entity_id: Option<i32>,
    /// The entity ID of the entity that directly dealt the damage.
    /// For projectiles, this is the projectile. For direct attacks, same as causing.
    pub direct_entity_id: Option<i32>,
    /// The source position of the damage (used for explosions, etc.).
    pub source_position: Option<Vector3<f64>>,
}

impl DamageSource {
    /// Creates a damage source with just a damage type and no entity/position context.
    /// Used for environmental damage like void, starvation, etc.
    #[must_use]
    pub const fn environment(damage_type: &'static DamageType) -> Self {
        Self {
            damage_type,
            causing_entity_id: None,
            direct_entity_id: None,
            source_position: None,
        }
    }

    /// Checks if this damage type bypasses invulnerability (creative mode protection).
    ///
    /// In vanilla, this is determined by the `bypasses_invulnerability` damage type tag,
    /// which contains `out_of_world` and `generic_kill`.
    ///
    /// TODO: Replace with proper damage type tag system when implemented.
    #[must_use]
    pub fn bypasses_invulnerability(&self) -> bool {
        let key = &*self.damage_type.key.path;

        matches!(key, "out_of_world" | "generic_kill")
    }
}
