//! Persistent `AdventureModePredicate.CODEC` impl

use simdnbt::{
    borrow::NbtTag as BorrowedNbtTag,
    owned::{NbtCompound, NbtList, NbtTag},
};
use steel_utils::snbt::{parse_vanilla_snbt_compound, to_vanilla_snbt_compound};

use crate::{
    RegistryExt, TaggedRegistryExt,
    data_components::{Component, ComponentData, DataComponentCodecContext},
};

use super::network::parse_vanilla_identifier;
use super::{
    AdventureModePredicate, BlockHolderSet, BlockPredicate, DataComponentExactPredicate,
    DataComponentMatchers, DataComponentPredicate, ExactDataComponentPredicate, NbtPredicate,
    StatePropertiesPredicate, StatePropertyMatcher, StatePropertyValueMatcher,
};

/// Serializes the persistent `AdventureModePredicate.CODEC` form
#[must_use]
pub fn nbt_writer(context: &DataComponentCodecContext<'_>, data: &ComponentData) -> NbtTag {
    let Some(predicate) = AdventureModePredicate::from_data_ref(data) else {
        panic!("Component type mismatch for adventure mode predicate");
    };
    assert!(
        !predicate.is_empty(),
        "Cannot persist an empty adventure mode predicate"
    );

    adventure_mode_predicate_nbt(context, predicate)
}

fn adventure_mode_predicate_nbt(
    context: &DataComponentCodecContext<'_>,
    predicate: &AdventureModePredicate,
) -> NbtTag {
    if predicate.predicates.len() == 1 {
        return NbtTag::Compound(block_predicate_nbt(context, &predicate.predicates[0]));
    }

    NbtTag::List(NbtList::Compound(
        predicate
            .predicates
            .iter()
            .map(|predicate| block_predicate_nbt(context, predicate))
            .collect(),
    ))
}

/// Deserializes the persistent `AdventureModePredicate.CODEC` form
#[must_use]
pub fn nbt_reader(
    context: &DataComponentCodecContext<'_>,
    tag: BorrowedNbtTag,
) -> Option<ComponentData> {
    let predicates = if let Some(compound) = tag.compound() {
        vec![parse_block_predicate_nbt(context, compound)?]
    } else {
        let compounds = tag.list()?.compounds()?;
        compounds
            .into_iter()
            .map(|compound| parse_block_predicate_nbt(context, compound))
            .collect::<Option<Vec<_>>>()?
    };

    if predicates.is_empty() {
        return None;
    }

    Some(AdventureModePredicate { predicates }.into_data())
}

fn block_predicate_nbt(
    context: &DataComponentCodecContext<'_>,
    predicate: &BlockPredicate,
) -> NbtCompound {
    let mut compound = NbtCompound::new();

    if let Some(blocks) = &predicate.blocks {
        compound.insert("blocks", block_holder_set_nbt(context, blocks));
    }
    if let Some(properties) = &predicate.properties {
        compound.insert("state", state_properties_nbt(properties));
    }
    if let Some(nbt) = &predicate.nbt {
        compound.insert(
            "nbt",
            NbtTag::String(to_vanilla_snbt_compound(&nbt.tag).into()),
        );
    }
    component_matchers_nbt(context, &predicate.components, &mut compound);
    compound
}

fn parse_block_predicate_nbt(
    context: &DataComponentCodecContext<'_>,
    compound: simdnbt::borrow::NbtCompound<'_, '_>,
) -> Option<BlockPredicate> {
    let blocks = match compound.get("blocks") {
        Some(tag) => Some(parse_block_holder_set_nbt(context, tag)?),
        None => None,
    };
    let properties = match compound.get("state") {
        Some(tag) => Some(parse_state_properties_nbt(tag)?),
        None => None,
    };
    let nbt = match compound.get("nbt") {
        Some(tag) => Some(parse_nbt_predicate_nbt(tag)?),
        None => None,
    };
    let components = parse_component_matchers_nbt(context, compound)?;

    Some(BlockPredicate {
        blocks,
        properties,
        nbt,
        components,
    })
}

fn parse_nbt_predicate_nbt(tag: BorrowedNbtTag) -> Option<NbtPredicate> {
    if let Some(value) = tag.string() {
        return parse_vanilla_snbt_compound(&value.to_str())
            .ok()
            .map(|tag| NbtPredicate { tag });
    }

    tag.compound().map(|tag| NbtPredicate {
        tag: tag.to_owned(),
    })
}

pub(super) fn block_holder_set_nbt(
    context: &DataComponentCodecContext<'_>,
    blocks: &BlockHolderSet,
) -> NbtTag {
    match blocks {
        BlockHolderSet::Tag(tag) => {
            assert!(
                context.registry().blocks.get_tag(tag).is_some(),
                "Cannot persist unknown block tag {tag}"
            );
            NbtTag::String(format!("#{tag}").into())
        }
        BlockHolderSet::Blocks(blocks) => {
            for block in blocks {
                assert!(
                    context.registry().blocks.by_key(&block.key).is_some(),
                    "Cannot persist unregistered block holder {}",
                    block.key
                );
            }

            if blocks.len() == 1 {
                return NbtTag::String(blocks[0].key.to_string().into());
            }
            if blocks.is_empty() {
                return NbtTag::List(NbtList::Empty);
            }
            NbtTag::List(NbtList::String(
                blocks
                    .iter()
                    .map(|block| block.key.to_string().into())
                    .collect(),
            ))
        }
    }
}

fn parse_block_holder_set_nbt(
    context: &DataComponentCodecContext<'_>,
    tag: BorrowedNbtTag,
) -> Option<BlockHolderSet> {
    if let Some(value) = tag.string() {
        let value = value.to_str();
        if let Some(tag) = value.strip_prefix('#') {
            let tag = parse_vanilla_identifier(tag)?;
            return context
                .registry()
                .blocks
                .get_tag(&tag)
                .map(|_| BlockHolderSet::Tag(tag));
        }
        let key = parse_vanilla_identifier(&value)?;
        return context
            .registry()
            .blocks
            .by_key(&key)
            .map(|block| BlockHolderSet::Blocks(vec![block]));
    }

    let list = tag.list()?;
    if list.id() == 0 {
        return Some(BlockHolderSet::Blocks(Vec::new()));
    }
    let values = list.strings()?;
    let blocks = values
        .iter()
        .map(|value| {
            let key = parse_vanilla_identifier(&value.to_str())?;
            context.registry().blocks.by_key(&key)
        })
        .collect::<Option<Vec<_>>>()?;
    Some(BlockHolderSet::Blocks(blocks))
}

pub(super) fn state_properties_nbt(properties: &StatePropertiesPredicate) -> NbtCompound {
    let mut compound = NbtCompound::new();
    for property in &properties.properties {
        let value = match &property.value {
            StatePropertyValueMatcher::Exact(value) => NbtTag::String(value.clone().into()),
            StatePropertyValueMatcher::Ranged { min, max } => {
                let mut range = NbtCompound::new();
                if let Some(min) = min {
                    range.insert("min", min.clone());
                }
                if let Some(max) = max {
                    range.insert("max", max.clone());
                }
                NbtTag::Compound(range)
            }
        };
        compound.insert(property.name.clone(), value);
    }
    compound
}

fn parse_state_properties_nbt(tag: BorrowedNbtTag) -> Option<StatePropertiesPredicate> {
    let compound = tag.compound()?;
    let properties = compound
        .iter()
        .map(|(name, value)| {
            let name = name.to_str().to_string();
            let value = if let Some(value) = value.string() {
                StatePropertyValueMatcher::Exact(value.to_str().into_owned())
            } else {
                let range = value.compound()?;
                let min = range
                    .get("min")
                    .and_then(|value| value.string())
                    .map(|value| value.to_str().into_owned());
                let max = range
                    .get("max")
                    .and_then(|value| value.string())
                    .map(|value| value.to_str().into_owned());
                StatePropertyValueMatcher::Ranged { min, max }
            };
            Some(StatePropertyMatcher { name, value })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(StatePropertiesPredicate { properties })
}

fn component_matchers_nbt(
    context: &DataComponentCodecContext<'_>,
    matchers: &DataComponentMatchers,
    compound: &mut NbtCompound,
) {
    let exact = exact_component_values_nbt(context, &matchers.exact);
    if !exact.is_empty() {
        compound.insert("components", exact);
    }

    if !matchers.partial.is_empty() {
        compound.insert(
            "predicates",
            partial_component_predicates_nbt(context, &matchers.partial),
        );
    }
}

fn exact_component_values_nbt(
    context: &DataComponentCodecContext<'_>,
    predicate: &DataComponentExactPredicate,
) -> NbtCompound {
    let mut exact = NbtCompound::new();
    for expected in &predicate.components {
        let Some(entry) = context
            .registry()
            .data_components
            .by_key(&expected.component)
        else {
            panic!(
                "Cannot persist unknown exact data component {}",
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
        let Some(nbt_writer) = entry.nbt_writer else {
            continue;
        };
        exact.insert(
            expected.component.to_string(),
            nbt_writer(context, &expected.value),
        );
    }
    exact
}

pub(super) fn partial_component_predicates_nbt(
    context: &DataComponentCodecContext<'_>,
    predicates: &[DataComponentPredicate],
) -> NbtCompound {
    let mut partial = NbtCompound::new();
    for predicate in predicates {
        match predicate {
            DataComponentPredicate::Any { component } => {
                if context
                    .registry()
                    .data_components
                    .by_key(component)
                    .is_none()
                {
                    panic!("Cannot persist unknown data component {component}");
                }
                partial.insert(component.to_string(), NbtCompound::new());
            }
            DataComponentPredicate::Typed {
                predicate_type,
                value,
            } => {
                if context
                    .registry()
                    .data_component_predicate_types
                    .id_from_key(predicate_type)
                    .is_none()
                {
                    panic!("Cannot persist unknown data component predicate type {predicate_type}");
                }
                partial.insert(predicate_type.to_string(), value.clone());
            }
        }
    }
    partial
}

fn parse_component_matchers_nbt(
    context: &DataComponentCodecContext<'_>,
    compound: simdnbt::borrow::NbtCompound<'_, '_>,
) -> Option<DataComponentMatchers> {
    let exact = if let Some(exact) = compound.get("components") {
        let exact = exact.compound()?;
        let components = exact
            .iter()
            .map(|(key, value)| {
                let component = parse_vanilla_identifier(&key.to_str())?;
                let entry = context.registry().data_components.by_key(&component)?;
                if !entry.is_persistent() {
                    return None;
                }
                let nbt_reader = entry.nbt_reader?;
                let value = nbt_reader(context, value)?;
                Some(ExactDataComponentPredicate { component, value })
            })
            .collect::<Option<Vec<_>>>()?;
        DataComponentExactPredicate { components }
    } else {
        DataComponentExactPredicate::EMPTY
    };

    let partial = if let Some(partial) = compound.get("predicates") {
        let partial = partial.compound()?;
        partial
            .iter()
            .map(|(key, value)| {
                let key = parse_vanilla_identifier(&key.to_str())?;
                if context
                    .registry()
                    .data_component_predicate_types
                    .id_from_key(&key)
                    .is_some()
                {
                    return Some(DataComponentPredicate::Typed {
                        predicate_type: key,
                        value: value.to_owned(),
                    });
                }

                let entry = context.registry().data_components.by_key(&key)?;
                let value = value.compound()?;
                if !value.is_empty() {
                    return None;
                }
                let _ = entry;
                Some(DataComponentPredicate::Any { component: key })
            })
            .collect::<Option<Vec<_>>>()?
    } else {
        Vec::new()
    };

    Some(DataComponentMatchers { exact, partial })
}
