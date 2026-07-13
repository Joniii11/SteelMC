//! Individual component type definitions.

mod adventure_mode_predicate;
mod attribute_modifiers;
mod block_transformer;
mod combat;
mod enchantments;
mod equippable;
mod map_post_processing;
mod provides_pottery_pattern;
mod tool;
mod use_cooldown;

pub use adventure_mode_predicate::{
    AdventureModePredicate, BlockHolderSet, BlockPredicate, DataComponentExactPredicate,
    DataComponentMatchers, DataComponentPredicate, ExactDataComponentPredicate, NbtPredicate,
    StatePropertiesPredicate, StatePropertyMatcher, StatePropertyValueMatcher,
    nbt_reader as adventure_mode_predicate_nbt_reader,
    nbt_writer as adventure_mode_predicate_nbt_writer,
    network_reader as adventure_mode_predicate_network_reader,
    network_writer as adventure_mode_predicate_network_writer,
};
pub use attribute_modifiers::{
    ItemAttributeModifierDisplay, ItemAttributeModifierEntry, ItemAttributeModifiers,
};
pub use block_transformer::{
    BlockTransformData, BlockTransformer, DropStrategy, TransformBlockState, TransformHolderSet,
    TransformNoiseParameters, TransformParticle, TransformPredicate, TransformStateProvider,
    TransformStateProviderRule, TransformType, WeightedTransformBlockState,
    nbt_reader as block_transformer_nbt_reader, nbt_writer as block_transformer_nbt_writer,
    network_reader as block_transformer_network_reader,
    network_writer as block_transformer_network_writer,
};
pub use combat::{AttackRange, DamageTypeComponent, PiercingWeapon, Weapon};
pub use enchantments::ItemEnchantments;
pub use equippable::{Equippable, EquippableAllowedEntities};
pub use map_post_processing::{
    MapPostProcessing, network_reader as map_post_processing_network_reader,
    network_writer as map_post_processing_network_writer,
};
pub use provides_pottery_pattern::{
    ProvidesPotteryPattern, nbt_reader as provides_pottery_pattern_nbt_reader,
    nbt_writer as provides_pottery_pattern_nbt_writer,
    network_reader as provides_pottery_pattern_network_reader,
    network_writer as provides_pottery_pattern_network_writer,
};
pub use tool::{Tool, ToolRule};
pub use use_cooldown::UseCooldown;
