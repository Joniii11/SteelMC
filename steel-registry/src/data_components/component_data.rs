//! ABI-stable component data storage.
//!
//! This module provides the core types for storing component values in an ABI-stable way.
//! Vanilla components get dedicated enum variants for zero-cost access, while plugin
//! components use the `Other` variant with opaque bytes.
use super::components::{
    AdventureModePredicate, AttackRange, BlockTransformer, DamageTypeComponent, Equippable,
    ItemAttributeModifiers, ItemEnchantments, PiercingWeapon, ProvidesPotteryPattern, Tool,
    UseCooldown, Weapon,
};
use steel_utils::hash::{ComponentHasher, HashComponent};
use text_components::TextComponent;

/// Discriminant for [`ComponentData`] variants.
///
/// Used for runtime type validation to ensure plugins don't
/// set wrong types on vanilla components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentDataDiscriminant {
    Empty,
    Bool,
    I32,
    Float,
    DamageType,
    AdventureModePredicate,
    BlockTransformer,
    ProvidesPotteryPattern,
    Tool,
    Weapon,
    AttackRange,
    UseCooldown,
    PiercingWeapon,
    Equippable,
    AttributeModifiers,
    Enchantments,
    TextComponent,
    Todo,
    Other,
}

/// ABI-stable component value storage.
///
/// Each vanilla component type gets its own variant for type-safe, zero-cost access.
/// Plugin-defined components use the `Other` variant with serialized bytes that
/// the plugin is responsible for interpreting.
///
/// # Example (vanilla code)
/// ```ignore
/// let data = ComponentData::I32(10);
/// if let ComponentData::I32(d) = data {
///     println!("Value: {}", d);
/// }
/// ```
///
/// # Example (plugin code)
/// ```ignore
/// // Plugin stores its own serialized data
/// let my_bytes = my_energy.serialize();
/// let data = ComponentData::Other(my_bytes);
///
/// // Plugin retrieves and deserializes
/// if let ComponentData::Other(bytes) = data {
///     let energy = MyEnergy::deserialize(&bytes)?;
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ComponentData {
    /// Component with no data (e.g., Unbreakable, Glider, `CreativeSlotLock`)
    Empty,

    /// Boolean component (e.g., `EnchantmentGlintOverride`)
    Bool(bool),
    /// i32 component (e.g., `MaxStackSize`, `MaxDamage`, Damage, `RepairCost`)
    /// Stored as `VarInt` on network.
    I32(i32),
    /// Float component (e.g., `PotionDurationScale`)
    Float(f32),
    /// `minecraft:damage_type`
    DamageType(DamageTypeComponent),
    /// `minecraft:can_place_on` / `minecraft:can_break`
    AdventureModePredicate(AdventureModePredicate),
    /// `minecraft:block_transformer`
    BlockTransformer(BlockTransformer),
    /// `minecraft:provides_pottery_pattern`
    ProvidesPotteryPattern(ProvidesPotteryPattern),

    /// minecraft:tool
    Tool(Tool),
    /// minecraft:weapon
    Weapon(Weapon),
    /// `minecraft:attack_range`
    AttackRange(AttackRange),
    /// `minecraft:use_cooldown`
    UseCooldown(UseCooldown),
    /// `minecraft:piercing_weapon`
    PiercingWeapon(PiercingWeapon),
    /// minecraft:equippable
    Equippable(Equippable),
    /// `minecraft:attribute_modifiers`
    AttributeModifiers(ItemAttributeModifiers),
    /// minecraft:enchantments / `minecraft:stored_enchantments`
    Enchantments(ItemEnchantments),
    /// `TextComponent` component (e.g., `CustomName`, `ItemName`)
    TextComponent(Box<TextComponent>),

    /// Placeholder for components that aren't implemented yet.
    Todo,

    /// Opaque bytes for plugin-defined components.
    /// The plugin is responsible for serialization/deserialization.
    Other(Vec<u8>),
}

impl ComponentData {
    /// Returns true if this is the empty/unit variant.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns the raw bytes if this is an `Other` variant.
    #[must_use]
    pub fn as_other(&self) -> Option<&[u8]> {
        match self {
            Self::Other(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns the discriminant of this component data variant.
    /// Used for runtime type validation.
    #[must_use]
    pub const fn discriminant(&self) -> ComponentDataDiscriminant {
        match self {
            Self::Empty => ComponentDataDiscriminant::Empty,
            Self::Bool(_) => ComponentDataDiscriminant::Bool,
            Self::I32(_) => ComponentDataDiscriminant::I32,
            Self::Float(_) => ComponentDataDiscriminant::Float,
            Self::DamageType(_) => ComponentDataDiscriminant::DamageType,
            Self::AdventureModePredicate(_) => ComponentDataDiscriminant::AdventureModePredicate,
            Self::BlockTransformer(_) => ComponentDataDiscriminant::BlockTransformer,
            Self::ProvidesPotteryPattern(_) => ComponentDataDiscriminant::ProvidesPotteryPattern,
            Self::Tool(_) => ComponentDataDiscriminant::Tool,
            Self::Weapon(_) => ComponentDataDiscriminant::Weapon,
            Self::AttackRange(_) => ComponentDataDiscriminant::AttackRange,
            Self::UseCooldown(_) => ComponentDataDiscriminant::UseCooldown,
            Self::PiercingWeapon(_) => ComponentDataDiscriminant::PiercingWeapon,
            Self::Equippable(_) => ComponentDataDiscriminant::Equippable,
            Self::AttributeModifiers(_) => ComponentDataDiscriminant::AttributeModifiers,
            Self::Enchantments(_) => ComponentDataDiscriminant::Enchantments,
            Self::TextComponent(_) => ComponentDataDiscriminant::TextComponent,
            Self::Todo => ComponentDataDiscriminant::Todo,
            Self::Other(_) => ComponentDataDiscriminant::Other,
        }
    }

    /// Persistent `HashOps` hash
    #[must_use]
    pub fn compute_hash(&self) -> i32 {
        let mut hasher = ComponentHasher::new();
        self.hash_component(&mut hasher);
        hasher.finish()
    }
}

impl HashComponent for ComponentData {
    fn hash_component(&self, hasher: &mut ComponentHasher) {
        match self {
            // Primitives
            Self::Empty => hasher.put_empty(),
            Self::Bool(v) => v.hash_component(hasher),
            Self::I32(v) => v.hash_component(hasher),
            Self::Float(v) => v.hash_component(hasher),
            Self::DamageType(v) => v.hash_component(hasher),
            Self::AdventureModePredicate(v) => v.hash_component(hasher),
            Self::BlockTransformer(v) => v.hash_component(hasher),
            Self::ProvidesPotteryPattern(v) => v.hash_component(hasher),

            // Complex types
            Self::Tool(v) => v.hash_component(hasher),
            Self::Weapon(v) => v.hash_component(hasher),
            Self::AttackRange(v) => v.hash_component(hasher),
            Self::UseCooldown(v) => v.hash_component(hasher),
            Self::PiercingWeapon(v) => v.hash_component(hasher),
            Self::Equippable(v) => v.hash_component(hasher),
            Self::AttributeModifiers(v) => v.hash_component(hasher),
            Self::Enchantments(v) => v.hash_component(hasher),
            Self::TextComponent(v) => v.hash_component(hasher),

            // Stub/plugin types - hash as empty map for now
            // TODO: Implement proper hashing when these types are implemented
            Self::Todo | Self::Other(_) => {
                hasher.start_map();
                hasher.end_map();
            }
        }
    }
}

/// Trait for types that can be converted to/from [`ComponentData`].
///
/// This provides compile-time type safety for vanilla components while
/// the actual storage uses the ABI-stable `ComponentData` enum.
///
/// # Example
/// ```ignore
/// impl Component for Damage {
///     fn into_data(self) -> ComponentData {
///         ComponentData::Damage(self)
///     }
///
///     fn from_data(data: ComponentData) -> Option<Self> {
///         match data {
///             ComponentData::Damage(d) => Some(d),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait Component: Sized + Clone {
    /// Converts this component value into `ComponentData`.
    fn into_data(self) -> ComponentData;

    /// Attempts to extract this component type from `ComponentData`.
    /// Returns `None` if the data is a different variant.
    fn from_data(data: ComponentData) -> Option<Self>;

    /// Attempts to get a reference to this component type from `ComponentData`.
    /// Returns `None` if the data is a different variant or if the type
    /// cannot be referenced directly (e.g., needs conversion).
    fn from_data_ref(data: &ComponentData) -> Option<&Self>;
}

// Unit type for marker components
impl Component for () {
    fn into_data(self) -> ComponentData {
        ComponentData::Empty
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Empty => Some(()),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Empty => Some(&()),
            _ => None,
        }
    }
}

impl Component for bool {
    fn into_data(self) -> ComponentData {
        ComponentData::Bool(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Bool(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Bool(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for i32 {
    fn into_data(self) -> ComponentData {
        ComponentData::I32(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::I32(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::I32(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for f32 {
    fn into_data(self) -> ComponentData {
        ComponentData::Float(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Float(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Float(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for DamageTypeComponent {
    fn into_data(self) -> ComponentData {
        ComponentData::DamageType(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::DamageType(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::DamageType(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for BlockTransformer {
    fn into_data(self) -> ComponentData {
        ComponentData::BlockTransformer(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::BlockTransformer(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::BlockTransformer(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for ProvidesPotteryPattern {
    fn into_data(self) -> ComponentData {
        ComponentData::ProvidesPotteryPattern(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::ProvidesPotteryPattern(value) => Some(value),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::ProvidesPotteryPattern(value) => Some(value),
            _ => None,
        }
    }
}

impl Component for AdventureModePredicate {
    fn into_data(self) -> ComponentData {
        ComponentData::AdventureModePredicate(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::AdventureModePredicate(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::AdventureModePredicate(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for Tool {
    fn into_data(self) -> ComponentData {
        ComponentData::Tool(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Tool(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Tool(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for Weapon {
    fn into_data(self) -> ComponentData {
        ComponentData::Weapon(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Weapon(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Weapon(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for AttackRange {
    fn into_data(self) -> ComponentData {
        ComponentData::AttackRange(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::AttackRange(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::AttackRange(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for PiercingWeapon {
    fn into_data(self) -> ComponentData {
        ComponentData::PiercingWeapon(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::PiercingWeapon(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::PiercingWeapon(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for UseCooldown {
    fn into_data(self) -> ComponentData {
        ComponentData::UseCooldown(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::UseCooldown(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::UseCooldown(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for ItemEnchantments {
    fn into_data(self) -> ComponentData {
        ComponentData::Enchantments(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Enchantments(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Enchantments(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for Equippable {
    fn into_data(self) -> ComponentData {
        ComponentData::Equippable(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::Equippable(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::Equippable(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for ItemAttributeModifiers {
    fn into_data(self) -> ComponentData {
        ComponentData::AttributeModifiers(self)
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::AttributeModifiers(v) => Some(v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::AttributeModifiers(v) => Some(v),
            _ => None,
        }
    }
}

impl Component for TextComponent {
    fn into_data(self) -> ComponentData {
        ComponentData::TextComponent(Box::new(self))
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::TextComponent(v) => Some(*v),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::TextComponent(v) => Some(v),
            _ => None,
        }
    }
}

// TextComponent and Identifier need special handling since they're used
// for multiple component types. We'll handle these through the DataComponentType
// registration rather than a blanket Component impl.
