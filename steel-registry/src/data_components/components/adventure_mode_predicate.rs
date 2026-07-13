//! Data model and codecs for `minecraft:can_place_on` and `minecraft:can_break`

mod nbt;
mod network;

#[cfg(test)]
mod tests;

pub use nbt::{nbt_reader, nbt_writer};
pub use network::{network_reader, network_writer};

use nbt::{block_holder_set_nbt, partial_component_predicates_nbt, state_properties_nbt};

use simdnbt::owned::{NbtCompound, NbtTag};
use steel_utils::{
    Identifier,
    hash::{ComponentHasher, HashComponent, HashEntry, hash_nbt_tag, sort_map_entries},
    snbt::to_vanilla_snbt_compound,
};

use crate::{
    REGISTRY, RegistryExt, TaggedRegistryExt,
    blocks::BlockRef,
    data_components::{ComponentData, DataComponentCodecContext},
};

/// Vanilla `AdventureModePredicate`
#[derive(Debug, Clone, PartialEq)]
pub struct AdventureModePredicate {
    pub predicates: Vec<BlockPredicate>,
}

impl AdventureModePredicate {
    /// Returns whether this predicate has no block predicates
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }
}

impl HashComponent for AdventureModePredicate {
    fn hash_component(&self, hasher: &mut ComponentHasher) {
        let context = DataComponentCodecContext::new(&REGISTRY);
        hash_adventure_mode_predicate(hasher, &context, self);
    }
}

/// Hashes `AdventureModePredicate.CODEC` while preserving nested component codecs `HashOps` types
fn hash_adventure_mode_predicate(
    hasher: &mut ComponentHasher,
    context: &DataComponentCodecContext<'_>,
    predicate: &AdventureModePredicate,
) {
    if predicate.predicates.len() == 1 {
        hash_block_predicate(hasher, context, &predicate.predicates[0]);
        return;
    }

    hasher.start_list();
    for block_predicate in &predicate.predicates {
        hash_nested_value(hasher, |nested| {
            hash_block_predicate(nested, context, block_predicate);
        });
    }
    hasher.end_list();
}

fn hash_block_predicate(
    hasher: &mut ComponentHasher,
    context: &DataComponentCodecContext<'_>,
    predicate: &BlockPredicate,
) {
    let mut entries = Vec::with_capacity(4);
    if let Some(blocks) = &predicate.blocks {
        push_hash_nbt_entry(
            &mut entries,
            "blocks",
            block_holder_set_nbt(context, blocks),
        );
    }
    if let Some(properties) = &predicate.properties {
        push_hash_nbt_entry(
            &mut entries,
            "state",
            NbtTag::Compound(state_properties_nbt(properties)),
        );
    }
    if let Some(nbt) = &predicate.nbt {
        push_hash_entry(&mut entries, "nbt", &to_vanilla_snbt_compound(&nbt.tag));
    }
    push_hash_component_matcher_entries(&mut entries, context, &predicate.components);

    hash_map(hasher, entries);
}

fn push_hash_component_matcher_entries(
    entries: &mut Vec<HashEntry>,
    context: &DataComponentCodecContext<'_>,
    matchers: &DataComponentMatchers,
) {
    push_hash_exact_component_matcher_entry(entries, context, &matchers.exact);
    if !matchers.partial.is_empty() {
        push_hash_nbt_entry(
            entries,
            "predicates",
            NbtTag::Compound(partial_component_predicates_nbt(context, &matchers.partial)),
        );
    }
}

fn push_hash_exact_component_matcher_entry(
    entries: &mut Vec<HashEntry>,
    context: &DataComponentCodecContext<'_>,
    predicate: &DataComponentExactPredicate,
) {
    let mut value_hasher = ComponentHasher::new();
    if !hash_exact_data_components(&mut value_hasher, context, predicate) {
        return;
    }

    let mut key_hasher = ComponentHasher::new();
    key_hasher.put_string("components");
    entries.push(HashEntry::new(key_hasher, value_hasher));
}

fn hash_exact_data_components(
    hasher: &mut ComponentHasher,
    context: &DataComponentCodecContext<'_>,
    predicate: &DataComponentExactPredicate,
) -> bool {
    let mut entries = Vec::with_capacity(predicate.components.len());
    for expected in &predicate.components {
        let Some(entry) = context
            .registry()
            .data_components
            .by_key(&expected.component)
        else {
            panic!(
                "Cannot hash unknown exact data component {}",
                expected.component
            );
        };
        assert!(
            entry.validates(&expected.value),
            "Exact data component {} has an incompatible value",
            expected.component
        );
        if !entry.is_persistent() {
            continue;
        }
        push_hash_entry(
            &mut entries,
            &expected.component.to_string(),
            &expected.value,
        );
    }

    if entries.is_empty() {
        return false;
    }
    hash_map(hasher, entries);
    true
}

fn hash_map(hasher: &mut ComponentHasher, mut entries: Vec<HashEntry>) {
    sort_map_entries(&mut entries);
    hasher.start_map();
    for entry in entries {
        hasher.put_raw_bytes(&entry.key_bytes);
        hasher.put_raw_bytes(&entry.value_bytes);
    }
    hasher.end_map();
}

fn push_hash_nbt_entry(entries: &mut Vec<HashEntry>, key: &str, value: NbtTag) {
    push_hash_entry_with(entries, key, |hasher| hash_nbt_tag(hasher, &value));
}

fn push_hash_entry<T: HashComponent + ?Sized>(entries: &mut Vec<HashEntry>, key: &str, value: &T) {
    push_hash_entry_with(entries, key, |hasher| value.hash_component(hasher));
}

fn push_hash_entry_with(
    entries: &mut Vec<HashEntry>,
    key: &str,
    hash_value: impl FnOnce(&mut ComponentHasher),
) {
    let mut key_hasher = ComponentHasher::new();
    key_hasher.put_string(key);
    let mut value_hasher = ComponentHasher::new();
    hash_value(&mut value_hasher);
    entries.push(HashEntry::new(key_hasher, value_hasher));
}

fn hash_nested_value(hasher: &mut ComponentHasher, hash_value: impl FnOnce(&mut ComponentHasher)) {
    let mut nested = ComponentHasher::new();
    hash_value(&mut nested);
    hasher.put_raw_bytes(&(nested.finish() as u32).to_le_bytes());
}

/// Vanilla block predicate used by adventuremode components
#[derive(Debug, Clone, PartialEq)]
pub struct BlockPredicate {
    /// A tag-backed or direct block holder set
    pub blocks: Option<BlockHolderSet>,
    /// Required block-state properties
    pub properties: Option<StatePropertiesPredicate>,
    /// Required serialized blockentity NBT
    pub nbt: Option<NbtPredicate>,
    /// Required block-entity data components
    pub components: DataComponentMatchers,
}

/// Vanilla `HolderSet<Block>` represented without raw registry IDs
#[derive(Debug, Clone, PartialEq)]
pub enum BlockHolderSet {
    /// A named block tag
    Tag(Identifier),
    /// Direct block registry holders
    Blocks(Vec<BlockRef>),
}

impl BlockHolderSet {
    /// Returns whether this holder set contains `block` in the supplied registry
    #[must_use]
    pub fn contains(&self, context: &DataComponentCodecContext<'_>, block: BlockRef) -> bool {
        match self {
            Self::Tag(tag) => context.registry().blocks.is_in_tag(block, tag),
            Self::Blocks(blocks) => blocks.contains(&block),
        }
    }
}

/// Vanilla `StatePropertiesPredicate`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatePropertiesPredicate {
    pub properties: Vec<StatePropertyMatcher>,
}

/// Blockstate property match
#[derive(Debug, Clone, PartialEq)]
pub struct StatePropertyMatcher {
    pub name: String,
    pub value: StatePropertyValueMatcher,
}

/// Vanilla exact or ranged state property matcher
#[derive(Debug, Clone, PartialEq)]
pub enum StatePropertyValueMatcher {
    Exact(String),
    Ranged {
        min: Option<String>,
        max: Option<String>,
    },
}

/// Vanilla `NbtPredicate` for a block entity
#[derive(Debug, Clone, PartialEq)]
pub struct NbtPredicate {
    pub tag: NbtCompound,
}

/// Vanilla `DataComponentMatchers`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataComponentMatchers {
    pub exact: DataComponentExactPredicate,
    /// Stream order is retained even though Vanilla stores these in a map
    pub partial: Vec<DataComponentPredicate>,
}

impl DataComponentMatchers {
    pub const ANY: Self = Self {
        exact: DataComponentExactPredicate::EMPTY,
        partial: Vec::new(),
    };

    /// Returns whether this has neither exact nor partial component constraints
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.partial.is_empty()
    }
}

/// Vanilla `DataComponentExactPredicate`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataComponentExactPredicate {
    pub components: Vec<ExactDataComponentPredicate>,
}

impl DataComponentExactPredicate {
    pub const EMPTY: Self = Self {
        components: Vec::new(),
    };

    /// Returns whether no exact component values are required
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

/// One exact datacomponent requirement
#[derive(Debug, Clone, PartialEq)]
pub struct ExactDataComponentPredicate {
    pub component: Identifier,
    pub value: ComponentData,
}

/// Vanilla partial `DataComponentPredicate` dispatch
#[derive(Debug, Clone, PartialEq)]
pub enum DataComponentPredicate {
    /// Vanilla `AnyValue` which only requires a component to be present
    Any { component: Identifier },
    /// A concrete predicate registered in `data_component_predicate_type`
    Typed {
        predicate_type: Identifier,
        value: NbtTag,
    },
}
