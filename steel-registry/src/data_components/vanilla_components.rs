//! Vanilla data component definitions and registration.
//!
//! This module defines all vanilla Minecraft data components and provides
//! the registration function to add them to the registry.
use steel_utils::Identifier;
use text_components::TextComponent;

use super::component_data::ComponentData;
pub use super::registry::DataComponentType;
use super::registry::{ComponentCodecs, DataComponentPredicateTypeRegistry, DataComponentRegistry};
pub use crate::attribute::AttributeModifierOperation;
pub use crate::equipment::{EquipmentSlot, EquipmentSlotGroup};

// Re-export component types for convenience
pub use super::components::{
    AdventureModePredicate, AttackRange, BlockHolderSet, BlockPredicate, BlockTransformData,
    BlockTransformer, DamageTypeComponent, DataComponentExactPredicate, DataComponentMatchers,
    DataComponentPredicate, DropStrategy, Equippable, EquippableAllowedEntities,
    ExactDataComponentPredicate, ItemAttributeModifierDisplay, ItemAttributeModifierEntry,
    ItemAttributeModifiers, ItemEnchantments, MapPostProcessing, NbtPredicate, PiercingWeapon,
    ProvidesPotteryPattern, StatePropertiesPredicate, StatePropertyMatcher,
    StatePropertyValueMatcher, Tool, ToolRule, TransformBlockState, TransformHolderSet,
    TransformNoiseParameters, TransformParticle, TransformPredicate, TransformStateProvider,
    TransformStateProviderRule, TransformType, UseCooldown, Weapon, WeightedTransformBlockState,
};

pub const MAX_STACK_SIZE: DataComponentType<i32> =
    DataComponentType::new(Identifier::vanilla_static("max_stack_size"));

pub const MAX_DAMAGE: DataComponentType<i32> =
    DataComponentType::new(Identifier::vanilla_static("max_damage"));

pub const CUSTOM_NAME: DataComponentType<TextComponent> =
    DataComponentType::new(Identifier::vanilla_static("custom_name"));

pub const ITEM_NAME: DataComponentType<TextComponent> =
    DataComponentType::new(Identifier::vanilla_static("item_name"));

pub const DAMAGE: DataComponentType<i32> =
    DataComponentType::new(Identifier::vanilla_static("damage"));

pub const REPAIR_COST: DataComponentType<i32> =
    DataComponentType::new(Identifier::vanilla_static("repair_cost"));

pub const UNBREAKABLE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("unbreakable"));

pub const TOOL: DataComponentType<Tool> =
    DataComponentType::new(Identifier::vanilla_static("tool"));

pub const BLOCK_TRANSFORMER: DataComponentType<BlockTransformer> =
    DataComponentType::new(Identifier::vanilla_static("block_transformer"));

pub const WEAPON: DataComponentType<Weapon> =
    DataComponentType::new(Identifier::vanilla_static("weapon"));

pub const ATTACK_RANGE: DataComponentType<AttackRange> =
    DataComponentType::new(Identifier::vanilla_static("attack_range"));

pub const EQUIPPABLE: DataComponentType<Equippable> =
    DataComponentType::new(Identifier::vanilla_static("equippable"));

pub const GLIDER: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("glider"));

pub const CREATIVE_SLOT_LOCK: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("creative_slot_lock"));

pub const INTANGIBLE_PROJECTILE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("intangible_projectile"));

pub const ENCHANTMENT_GLINT_OVERRIDE: DataComponentType<bool> =
    DataComponentType::new(Identifier::vanilla_static("enchantment_glint_override"));

pub const POTION_DURATION_SCALE: DataComponentType<f32> =
    DataComponentType::new(Identifier::vanilla_static("potion_duration_scale"));

// These components are registered but use placeholder serialization.
// They use the Todo ComponentData variant.

pub const CUSTOM_DATA: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("custom_data"));

pub const USE_EFFECTS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("use_effects"));

pub const MINIMUM_ATTACK_CHARGE: DataComponentType<f32> =
    DataComponentType::new(Identifier::vanilla_static("minimum_attack_charge"));

pub const DAMAGE_TYPE: DataComponentType<DamageTypeComponent> =
    DataComponentType::new(Identifier::vanilla_static("damage_type"));

pub const ITEM_MODEL: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("item_model"));

pub const LORE: DataComponentType<()> = DataComponentType::new(Identifier::vanilla_static("lore"));

pub const RARITY: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("rarity"));

pub const ENCHANTMENTS: DataComponentType<ItemEnchantments> =
    DataComponentType::new(Identifier::vanilla_static("enchantments"));

pub const CAN_PLACE_ON: DataComponentType<AdventureModePredicate> =
    DataComponentType::new(Identifier::vanilla_static("can_place_on"));

pub const CAN_BREAK: DataComponentType<AdventureModePredicate> =
    DataComponentType::new(Identifier::vanilla_static("can_break"));

pub const ATTRIBUTE_MODIFIERS: DataComponentType<ItemAttributeModifiers> =
    DataComponentType::new(Identifier::vanilla_static("attribute_modifiers"));

pub const CUSTOM_MODEL_DATA: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("custom_model_data"));

pub const TOOLTIP_DISPLAY: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("tooltip_display"));

pub const TOOLTIP_STYLE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("tooltip_style"));

pub const NOTE_BLOCK_SOUND: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("note_block_sound"));

pub const FOOD: DataComponentType<()> = DataComponentType::new(Identifier::vanilla_static("food"));

pub const CONSUMABLE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("consumable"));

pub const USE_REMAINDER: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("use_remainder"));

pub const USE_COOLDOWN: DataComponentType<UseCooldown> =
    DataComponentType::new(Identifier::vanilla_static("use_cooldown"));

pub const DAMAGE_RESISTANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("damage_resistant"));

pub const ENCHANTABLE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("enchantable"));

pub const REPAIRABLE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("repairable"));

pub const DEATH_PROTECTION: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("death_protection"));

pub const BLOCKS_ATTACKS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("blocks_attacks"));

pub const PIERCING_WEAPON: DataComponentType<PiercingWeapon> =
    DataComponentType::new(Identifier::vanilla_static("piercing_weapon"));

pub const KINETIC_WEAPON: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("kinetic_weapon"));

pub const SWING_ANIMATION: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("swing_animation"));

pub const ADDITIONAL_TRADE_COST: DataComponentType<i32> =
    DataComponentType::new(Identifier::vanilla_static("additional_trade_cost"));

pub const STORED_ENCHANTMENTS: DataComponentType<ItemEnchantments> =
    DataComponentType::new(Identifier::vanilla_static("stored_enchantments"));

pub const DYE: DataComponentType<()> = DataComponentType::new(Identifier::vanilla_static("dye"));

pub const DYED_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("dyed_color"));

pub const MAP_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("map_color"));

pub const MAP_ID: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("map_id"));

pub const MAP_DECORATIONS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("map_decorations"));

pub const MAP_POST_PROCESSING: DataComponentType<MapPostProcessing> =
    DataComponentType::new(Identifier::vanilla_static("map_post_processing"));

pub const CHARGED_PROJECTILES: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("charged_projectiles"));

pub const BUNDLE_CONTENTS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("bundle_contents"));

pub const POTION_CONTENTS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("potion_contents"));

pub const SUSPICIOUS_STEW_EFFECTS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("suspicious_stew_effects"));

pub const WRITABLE_BOOK_CONTENT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("writable_book_content"));

pub const WRITTEN_BOOK_CONTENT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("written_book_content"));

pub const TRIM: DataComponentType<()> = DataComponentType::new(Identifier::vanilla_static("trim"));

pub const DEBUG_STICK_STATE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("debug_stick_state"));

pub const ENTITY_DATA: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("entity_data"));

pub const BUCKET_ENTITY_DATA: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("bucket_entity_data"));

pub const BLOCK_ENTITY_DATA: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("block_entity_data"));

pub const INSTRUMENT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("instrument"));

pub const PROVIDES_TRIM_MATERIAL: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("provides_trim_material"));

pub const OMINOUS_BOTTLE_AMPLIFIER: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("ominous_bottle_amplifier"));

pub const JUKEBOX_PLAYABLE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("jukebox_playable"));

pub const PROVIDES_BANNER_PATTERNS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("provides_banner_patterns"));

pub const RECIPES: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("recipes"));

pub const LODESTONE_TRACKER: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("lodestone_tracker"));

pub const FIREWORK_EXPLOSION: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("firework_explosion"));

pub const FIREWORKS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("fireworks"));

pub const PROFILE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("profile"));

pub const BANNER_PATTERNS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("banner_patterns"));

pub const BASE_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("base_color"));

pub const POT_DECORATIONS: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("pot_decorations"));

pub const CONTAINER: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("container"));

pub const BLOCK_STATE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("block_state"));

pub const BEES: DataComponentType<()> = DataComponentType::new(Identifier::vanilla_static("bees"));

pub const SULFUR_CUBE_CONTENT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("sulfur_cube_content"));

pub const LOCK: DataComponentType<()> = DataComponentType::new(Identifier::vanilla_static("lock"));

pub const CONTAINER_LOOT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("container_loot"));

pub const BREAK_SOUND: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("break_sound"));

// Entity variant components
pub const VILLAGER_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("villager/variant"));

pub const WOLF_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("wolf/variant"));

pub const WOLF_SOUND_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("wolf/sound_variant"));

pub const WOLF_COLLAR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("wolf/collar"));

pub const FOX_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("fox/variant"));

pub const SALMON_SIZE: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("salmon/size"));

pub const PARROT_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("parrot/variant"));

pub const TROPICAL_FISH_PATTERN: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("tropical_fish/pattern"));

pub const TROPICAL_FISH_BASE_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("tropical_fish/base_color"));

pub const TROPICAL_FISH_PATTERN_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("tropical_fish/pattern_color"));

pub const MOOSHROOM_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("mooshroom/variant"));

pub const RABBIT_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("rabbit/variant"));

pub const PIG_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("pig/variant"));

pub const PIG_SOUND_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("pig/sound_variant"));

pub const COW_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("cow/variant"));

pub const COW_SOUND_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("cow/sound_variant"));

pub const CHICKEN_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("chicken/variant"));

pub const CHICKEN_SOUND_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("chicken/sound_variant"));

pub const ZOMBIE_NAUTILUS_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("zombie_nautilus/variant"));

pub const FROG_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("frog/variant"));

pub const HORSE_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("horse/variant"));

pub const PAINTING_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("painting/variant"));

pub const LLAMA_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("llama/variant"));

pub const AXOLOTL_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("axolotl/variant"));

pub const CAT_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("cat/variant"));

pub const CAT_SOUND_VARIANT: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("cat/sound_variant"));

pub const CAT_COLLAR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("cat/collar"));

pub const SHEEP_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("sheep/color"));

pub const SHULKER_COLOR: DataComponentType<()> =
    DataComponentType::new(Identifier::vanilla_static("shulker/color"));

pub const PROVIDES_POTTERY_PATTERN: DataComponentType<ProvidesPotteryPattern> =
    DataComponentType::new(Identifier::vanilla_static("provides_pottery_pattern"));

/// Helper to create stub reader/writer functions for unimplemented components.
/// These components use the Todo variant as a placeholder.
// TODO replace persistent component stubs
macro_rules! register_stub {
    ($registry:expr, $key:expr) => {{
        const fn network_reader(
            _context: &crate::data_components::DataComponentCodecContext<'_>,
            cursor: &mut std::io::Cursor<&[u8]>,
        ) -> std::io::Result<ComponentData> {
            // Stub: read nothing, return Todo
            let _ = cursor;
            Ok(ComponentData::Todo)
        }

        const fn network_writer(
            _context: &crate::data_components::DataComponentCodecContext<'_>,
            data: &ComponentData,
            _writer: &mut Vec<u8>,
        ) -> std::io::Result<()> {
            // Stub: write nothing
            let _ = data;
            Ok(())
        }

        const fn nbt_reader(
            _context: &crate::data_components::DataComponentCodecContext<'_>,
            _tag: simdnbt::borrow::NbtTag,
        ) -> Option<ComponentData> {
            Some(ComponentData::Todo)
        }

        fn nbt_writer(
            _context: &crate::data_components::DataComponentCodecContext<'_>,
            _data: &ComponentData,
        ) -> simdnbt::owned::NbtTag {
            simdnbt::owned::NbtTag::Compound(simdnbt::owned::NbtCompound::new())
        }

        $registry.register_dynamic(
            $key,
            crate::data_components::ComponentDataDiscriminant::Todo,
            ComponentCodecs::persistent(network_reader, network_writer, nbt_reader, nbt_writer),
        );
    }};
}

/// Network reader for VarInt-encoded i32 components.
fn varint_reader(
    _context: &crate::data_components::DataComponentCodecContext<'_>,
    cursor: &mut std::io::Cursor<&[u8]>,
) -> std::io::Result<ComponentData> {
    use steel_utils::{codec::VarInt, serial::ReadFrom};
    let value = VarInt::read(cursor)?;
    Ok(ComponentData::I32(value.0))
}

/// Network writer for VarInt-encoded i32 components.
fn varint_writer(
    _context: &crate::data_components::DataComponentCodecContext<'_>,
    data: &ComponentData,
    writer: &mut Vec<u8>,
) -> std::io::Result<()> {
    use steel_utils::{codec::VarInt, serial::WriteTo};
    if let ComponentData::I32(v) = data {
        VarInt(*v).write(writer)
    } else {
        Err(std::io::Error::other("Component type mismatch"))
    }
}

/// Registers all vanilla data components.
///
/// IMPORTANT: The registration order MUST match vanilla's DataComponents.java exactly,
/// as the component's network ID is determined by its registration order.
pub fn register_vanilla_data_components(registry: &mut DataComponentRegistry) {
    use crate::data_components::ComponentDataDiscriminant;

    // Order must match vanilla's DataComponents.java exactly!
    // 0: custom_data
    register_stub!(registry, CUSTOM_DATA.key.clone());
    // 1: max_stack_size
    registry.register_custom_network(
        MAX_STACK_SIZE,
        ComponentDataDiscriminant::I32,
        varint_reader,
        varint_writer,
    );
    // 2: max_damage
    registry.register_custom_network(
        MAX_DAMAGE,
        ComponentDataDiscriminant::I32,
        varint_reader,
        varint_writer,
    );
    // 3: damage
    registry.register_custom_network(
        DAMAGE,
        ComponentDataDiscriminant::I32,
        varint_reader,
        varint_writer,
    );
    // 4: unbreakable
    registry.register(UNBREAKABLE, ComponentDataDiscriminant::Empty);
    // 5: use_effects
    register_stub!(registry, USE_EFFECTS.key.clone());
    // 6: custom_name
    registry.register(CUSTOM_NAME, ComponentDataDiscriminant::TextComponent);
    // 7: minimum_attack_charge
    registry.register(MINIMUM_ATTACK_CHARGE, ComponentDataDiscriminant::Float);
    // 8: damage_type
    registry.register(DAMAGE_TYPE, ComponentDataDiscriminant::DamageType);
    // 9: item_name
    registry.register(ITEM_NAME, ComponentDataDiscriminant::TextComponent);
    // 10: item_model
    register_stub!(registry, ITEM_MODEL.key.clone());
    // 11: lore
    register_stub!(registry, LORE.key.clone());
    // 12: rarity
    register_stub!(registry, RARITY.key.clone());
    // 13: enchantments
    registry.register(ENCHANTMENTS, ComponentDataDiscriminant::Enchantments);
    // 14: can_place_on
    registry.register_custom(
        CAN_PLACE_ON.key.clone(),
        ComponentDataDiscriminant::AdventureModePredicate,
        super::components::adventure_mode_predicate_network_reader,
        super::components::adventure_mode_predicate_network_writer,
        super::components::adventure_mode_predicate_nbt_reader,
        super::components::adventure_mode_predicate_nbt_writer,
    );
    // 15: can_break
    registry.register_custom(
        CAN_BREAK.key.clone(),
        ComponentDataDiscriminant::AdventureModePredicate,
        super::components::adventure_mode_predicate_network_reader,
        super::components::adventure_mode_predicate_network_writer,
        super::components::adventure_mode_predicate_nbt_reader,
        super::components::adventure_mode_predicate_nbt_writer,
    );
    // 16: attribute_modifiers
    registry.register(
        ATTRIBUTE_MODIFIERS,
        ComponentDataDiscriminant::AttributeModifiers,
    );
    // 17: custom_model_data
    register_stub!(registry, CUSTOM_MODEL_DATA.key.clone());
    // 18: tooltip_display
    register_stub!(registry, TOOLTIP_DISPLAY.key.clone());
    // 19: repair_cost
    registry.register_custom_network(
        REPAIR_COST,
        ComponentDataDiscriminant::I32,
        varint_reader,
        varint_writer,
    );
    // 20: creative_slot_lock
    registry.register_transient(CREATIVE_SLOT_LOCK, ComponentDataDiscriminant::Empty);
    // 21: enchantment_glint_override
    registry.register(ENCHANTMENT_GLINT_OVERRIDE, ComponentDataDiscriminant::Bool);
    // 22: intangible_projectile
    registry.register(INTANGIBLE_PROJECTILE, ComponentDataDiscriminant::Empty);
    // 23: food
    register_stub!(registry, FOOD.key.clone());
    // 24: consumable
    register_stub!(registry, CONSUMABLE.key.clone());
    // 25: use_remainder
    register_stub!(registry, USE_REMAINDER.key.clone());
    // 26: use_cooldown
    registry.register(USE_COOLDOWN, ComponentDataDiscriminant::UseCooldown);
    // 27: damage_resistant
    register_stub!(registry, DAMAGE_RESISTANT.key.clone());
    // 28: tool
    registry.register(TOOL, ComponentDataDiscriminant::Tool);
    // 29: weapon
    registry.register(WEAPON, ComponentDataDiscriminant::Weapon);
    // 30: attack_range
    registry.register(ATTACK_RANGE, ComponentDataDiscriminant::AttackRange);
    // 31: enchantable
    register_stub!(registry, ENCHANTABLE.key.clone());
    // 32: equippable
    registry.register(EQUIPPABLE, ComponentDataDiscriminant::Equippable);
    // 33: repairable
    register_stub!(registry, REPAIRABLE.key.clone());
    // 34: glider
    registry.register(GLIDER, ComponentDataDiscriminant::Empty);
    // 35: tooltip_style
    register_stub!(registry, TOOLTIP_STYLE.key.clone());
    // 36: death_protection
    register_stub!(registry, DEATH_PROTECTION.key.clone());
    // 37: blocks_attacks
    register_stub!(registry, BLOCKS_ATTACKS.key.clone());
    // 38: piercing_weapon
    registry.register(PIERCING_WEAPON, ComponentDataDiscriminant::PiercingWeapon);
    // 39: kinetic_weapon
    register_stub!(registry, KINETIC_WEAPON.key.clone());
    // 40: swing_animation
    register_stub!(registry, SWING_ANIMATION.key.clone());
    // 41: additional_trade_cost
    registry.register_custom_network_transient(
        ADDITIONAL_TRADE_COST,
        ComponentDataDiscriminant::I32,
        varint_reader,
        varint_writer,
    );
    // 42: block_transformer
    registry.register_custom(
        BLOCK_TRANSFORMER.key.clone(),
        ComponentDataDiscriminant::BlockTransformer,
        super::components::block_transformer_network_reader,
        super::components::block_transformer_network_writer,
        super::components::block_transformer_nbt_reader,
        super::components::block_transformer_nbt_writer,
    );
    // 43: stored_enchantments
    registry.register(STORED_ENCHANTMENTS, ComponentDataDiscriminant::Enchantments);
    // 44: dye
    register_stub!(registry, DYE.key.clone());
    // 45: dyed_color
    register_stub!(registry, DYED_COLOR.key.clone());
    // 46: map_color
    register_stub!(registry, MAP_COLOR.key.clone());
    // 47: map_id
    register_stub!(registry, MAP_ID.key.clone());
    // 48: map_decorations
    register_stub!(registry, MAP_DECORATIONS.key.clone());
    // 49: map_post_processing
    registry.register_custom_network_transient(
        MAP_POST_PROCESSING,
        ComponentDataDiscriminant::I32,
        super::components::map_post_processing_network_reader,
        super::components::map_post_processing_network_writer,
    );
    // 50: charged_projectiles
    register_stub!(registry, CHARGED_PROJECTILES.key.clone());
    // 51: bundle_contents
    register_stub!(registry, BUNDLE_CONTENTS.key.clone());
    // 52: potion_contents
    register_stub!(registry, POTION_CONTENTS.key.clone());
    // 53: potion_duration_scale
    registry.register(POTION_DURATION_SCALE, ComponentDataDiscriminant::Float);
    // 54: suspicious_stew_effects
    register_stub!(registry, SUSPICIOUS_STEW_EFFECTS.key.clone());
    // 55: writable_book_content
    register_stub!(registry, WRITABLE_BOOK_CONTENT.key.clone());
    // 56: written_book_content
    register_stub!(registry, WRITTEN_BOOK_CONTENT.key.clone());
    // 57: trim
    register_stub!(registry, TRIM.key.clone());
    // 58: debug_stick_state
    register_stub!(registry, DEBUG_STICK_STATE.key.clone());
    // 59: entity_data
    register_stub!(registry, ENTITY_DATA.key.clone());
    // 60: bucket_entity_data
    register_stub!(registry, BUCKET_ENTITY_DATA.key.clone());
    // 61: block_entity_data
    register_stub!(registry, BLOCK_ENTITY_DATA.key.clone());
    // 62: instrument
    register_stub!(registry, INSTRUMENT.key.clone());
    // 63: provides_trim_material
    register_stub!(registry, PROVIDES_TRIM_MATERIAL.key.clone());
    // 64: ominous_bottle_amplifier
    register_stub!(registry, OMINOUS_BOTTLE_AMPLIFIER.key.clone());
    // 65: jukebox_playable
    register_stub!(registry, JUKEBOX_PLAYABLE.key.clone());
    // 66: provides_banner_patterns
    register_stub!(registry, PROVIDES_BANNER_PATTERNS.key.clone());
    // 67: recipes
    register_stub!(registry, RECIPES.key.clone());
    // 68: lodestone_tracker
    register_stub!(registry, LODESTONE_TRACKER.key.clone());
    // 69: firework_explosion
    register_stub!(registry, FIREWORK_EXPLOSION.key.clone());
    // 70: fireworks
    register_stub!(registry, FIREWORKS.key.clone());
    // 71: profile
    register_stub!(registry, PROFILE.key.clone());
    // 72: note_block_sound
    register_stub!(registry, NOTE_BLOCK_SOUND.key.clone());
    // 73: banner_patterns
    register_stub!(registry, BANNER_PATTERNS.key.clone());
    // 74: base_color
    register_stub!(registry, BASE_COLOR.key.clone());
    // 75: pot_decorations
    register_stub!(registry, POT_DECORATIONS.key.clone());
    // 76: container
    register_stub!(registry, CONTAINER.key.clone());
    // 77: block_state
    register_stub!(registry, BLOCK_STATE.key.clone());
    // 78: bees
    register_stub!(registry, BEES.key.clone());
    // 79: sulfur_cube_content
    register_stub!(registry, SULFUR_CUBE_CONTENT.key.clone());
    // 80: lock
    register_stub!(registry, LOCK.key.clone());
    // 81: container_loot
    register_stub!(registry, CONTAINER_LOOT.key.clone());
    // 82: break_sound
    register_stub!(registry, BREAK_SOUND.key.clone());
    // 83: villager/variant
    register_stub!(registry, VILLAGER_VARIANT.key.clone());
    // 84: wolf/variant
    register_stub!(registry, WOLF_VARIANT.key.clone());
    // 85: wolf/sound_variant
    register_stub!(registry, WOLF_SOUND_VARIANT.key.clone());
    // 86: wolf/collar
    register_stub!(registry, WOLF_COLLAR.key.clone());
    // 87: fox/variant
    register_stub!(registry, FOX_VARIANT.key.clone());
    // 88: salmon/size
    register_stub!(registry, SALMON_SIZE.key.clone());
    // 89: parrot/variant
    register_stub!(registry, PARROT_VARIANT.key.clone());
    // 90: tropical_fish/pattern
    register_stub!(registry, TROPICAL_FISH_PATTERN.key.clone());
    // 91: tropical_fish/base_color
    register_stub!(registry, TROPICAL_FISH_BASE_COLOR.key.clone());
    // 92: tropical_fish/pattern_color
    register_stub!(registry, TROPICAL_FISH_PATTERN_COLOR.key.clone());
    // 93: mooshroom/variant
    register_stub!(registry, MOOSHROOM_VARIANT.key.clone());
    // 94: rabbit/variant
    register_stub!(registry, RABBIT_VARIANT.key.clone());
    // 95: pig/variant
    register_stub!(registry, PIG_VARIANT.key.clone());
    // 96: pig/sound_variant
    register_stub!(registry, PIG_SOUND_VARIANT.key.clone());
    // 97: cow/variant
    register_stub!(registry, COW_VARIANT.key.clone());
    // 98: cow/sound_variant
    register_stub!(registry, COW_SOUND_VARIANT.key.clone());
    // 99: chicken/variant
    register_stub!(registry, CHICKEN_VARIANT.key.clone());
    // 100: chicken/sound_variant
    register_stub!(registry, CHICKEN_SOUND_VARIANT.key.clone());
    // 101: zombie_nautilus/variant
    register_stub!(registry, ZOMBIE_NAUTILUS_VARIANT.key.clone());
    // 102: frog/variant
    register_stub!(registry, FROG_VARIANT.key.clone());
    // 103: horse/variant
    register_stub!(registry, HORSE_VARIANT.key.clone());
    // 104: painting/variant
    register_stub!(registry, PAINTING_VARIANT.key.clone());
    // 105: llama/variant
    register_stub!(registry, LLAMA_VARIANT.key.clone());
    // 106: axolotl/variant
    register_stub!(registry, AXOLOTL_VARIANT.key.clone());
    // 107: cat/variant
    register_stub!(registry, CAT_VARIANT.key.clone());
    // 108: cat/sound_variant
    register_stub!(registry, CAT_SOUND_VARIANT.key.clone());
    // 109: cat/collar
    register_stub!(registry, CAT_COLLAR.key.clone());
    // 110: sheep/color
    register_stub!(registry, SHEEP_COLOR.key.clone());
    // 111: shulker/color
    register_stub!(registry, SHULKER_COLOR.key.clone());
    // 112: provides_pottery_pattern
    registry.register_custom(
        PROVIDES_POTTERY_PATTERN.key.clone(),
        ComponentDataDiscriminant::ProvidesPotteryPattern,
        super::components::provides_pottery_pattern_network_reader,
        super::components::provides_pottery_pattern_network_writer,
        super::components::provides_pottery_pattern_nbt_reader,
        super::components::provides_pottery_pattern_nbt_writer,
    );
}

/// Vanilla predicate types
pub fn register_vanilla_data_component_predicate_types(
    registry: &mut DataComponentPredicateTypeRegistry,
) {
    registry.register(Identifier::vanilla_static("damage"));
    registry.register(Identifier::vanilla_static("enchantments"));
    registry.register(Identifier::vanilla_static("stored_enchantments"));
    registry.register(Identifier::vanilla_static("potion_contents"));
    registry.register(Identifier::vanilla_static("custom_data"));
    registry.register(Identifier::vanilla_static("container"));
    registry.register(Identifier::vanilla_static("bundle_contents"));
    registry.register(Identifier::vanilla_static("firework_explosion"));
    registry.register(Identifier::vanilla_static("fireworks"));
    registry.register(Identifier::vanilla_static("writable_book_content"));
    registry.register(Identifier::vanilla_static("written_book_content"));
    registry.register(Identifier::vanilla_static("attribute_modifiers"));
    registry.register(Identifier::vanilla_static("trim"));
    registry.register(Identifier::vanilla_static("jukebox_playable"));
    registry.register(Identifier::vanilla_static("villager/variant"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RegistryExt;

    #[test]
    fn snapshot_2_block_transformer_keeps_vanilla_component_order() {
        let mut registry = DataComponentRegistry::new();
        register_vanilla_data_components(&mut registry);

        assert_eq!(registry.get_key_by_id(42), Some(&BLOCK_TRANSFORMER.key));
        assert_eq!(registry.get_key_by_id(43), Some(&STORED_ENCHANTMENTS.key));
        assert_eq!(registry.get_key_by_id(78), Some(&BEES.key));
        assert_eq!(registry.get_key_by_id(79), Some(&SULFUR_CUBE_CONTENT.key));
        assert_eq!(registry.get_key_by_id(80), Some(&LOCK.key));
        assert_eq!(registry.get_key_by_id(81), Some(&CONTAINER_LOOT.key));
        assert_eq!(registry.get_key_by_id(82), Some(&BREAK_SOUND.key));
        assert_eq!(registry.get_key_by_id(83), Some(&VILLAGER_VARIANT.key));
        assert_eq!(registry.get_key_by_id(111), Some(&SHULKER_COLOR.key));
        assert_eq!(
            registry.get_key_by_id(112),
            Some(&PROVIDES_POTTERY_PATTERN.key)
        );
    }

    #[test]
    fn snapshot_2_marks_exactly_the_transient_vanilla_components() {
        let mut registry = DataComponentRegistry::new();
        register_vanilla_data_components(&mut registry);

        for id in [20, 41, 49] {
            let entry = registry
                .by_id(id)
                .expect("transient component ID must be registered");
            assert!(
                !entry.is_persistent(),
                "component ID {id} must be transient"
            );
            assert!(
                entry.nbt_reader.is_none() && entry.nbt_writer.is_none(),
                "transient component ID {id} must not install persistent NBT codecs"
            );
        }
        for id in [19, 21, 40, 42, 48, 50] {
            let entry = registry
                .by_id(id)
                .expect("persistent component ID must be registered");
            assert!(
                entry.is_persistent(),
                "component ID {id} must be persistent"
            );
            assert!(
                entry.nbt_reader.is_some() && entry.nbt_writer.is_some(),
                "persistent component ID {id} must install both NBT codecs"
            );
        }
    }
}
