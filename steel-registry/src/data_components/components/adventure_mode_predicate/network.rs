//! Vanilla `AdventureModePredicate.STREAM_CODEC` impl

use std::io::{Cursor, Error, Read, Result, Write};

use simdnbt::owned::{NbtCompound, NbtTag};
use steel_utils::{
    Identifier,
    codec::VarInt,
    nbt::read_tag_with_default_quota,
    serial::{ReadFrom, WriteTo},
};

use crate::{
    RegistryExt, TaggedRegistryExt,
    data_components::{Component, ComponentData, DataComponentCodecContext},
};

use super::{
    AdventureModePredicate, BlockHolderSet, BlockPredicate, DataComponentExactPredicate,
    DataComponentMatchers, DataComponentPredicate, ExactDataComponentPredicate, NbtPredicate,
    StatePropertiesPredicate, StatePropertyMatcher, StatePropertyValueMatcher,
};

const MAX_PARTIAL_PREDICATES: usize = 64;

pub fn network_writer(
    context: &DataComponentCodecContext<'_>,
    data: &ComponentData,
    writer: &mut Vec<u8>,
) -> Result<()> {
    let Some(predicate) = AdventureModePredicate::from_data_ref(data) else {
        return Err(Error::other(
            "Component type mismatch for adventure mode predicate",
        ));
    };

    write_count(
        writer,
        predicate.predicates.len(),
        "adventure mode predicate",
    )?;
    for block_predicate in &predicate.predicates {
        write_block_predicate(context, block_predicate, writer)?;
    }
    Ok(())
}

/// Deserializes Vanilla `AdventureModePredicate.STREAM_CODEC`
pub fn network_reader(
    context: &DataComponentCodecContext<'_>,
    data: &mut Cursor<&[u8]>,
) -> Result<ComponentData> {
    let count = read_count(data, "adventure mode predicate")?;
    let mut predicates = Vec::with_capacity(count.min(65_536));
    for _ in 0..count {
        predicates.push(read_block_predicate(context, data)?);
    }
    Ok(AdventureModePredicate { predicates }.into_data())
}

fn write_block_predicate(
    context: &DataComponentCodecContext<'_>,
    predicate: &BlockPredicate,
    writer: &mut Vec<u8>,
) -> Result<()> {
    match &predicate.blocks {
        Some(blocks) => {
            true.write(writer)?;
            write_block_holder_set(context, blocks, writer)?;
        }
        None => false.write(writer)?,
    }

    match &predicate.properties {
        Some(properties) => {
            true.write(writer)?;
            write_state_properties(properties, writer)?;
        }
        None => false.write(writer)?,
    }

    match &predicate.nbt {
        Some(nbt) => {
            true.write(writer)?;
            WriteTo::write(&NbtTag::Compound(nbt.tag.clone()), writer)?;
        }
        None => false.write(writer)?,
    }

    write_component_matchers(context, &predicate.components, writer)
}

fn read_block_predicate(
    context: &DataComponentCodecContext<'_>,
    data: &mut Cursor<&[u8]>,
) -> Result<BlockPredicate> {
    let blocks = if bool::read(data)? {
        Some(read_block_holder_set(context, data)?)
    } else {
        None
    };
    let properties = if bool::read(data)? {
        Some(read_state_properties(data)?)
    } else {
        None
    };
    let nbt = if bool::read(data)? {
        let tag = read_nbt_tag(data)?;
        let NbtTag::Compound(tag) = tag else {
            return Err(Error::other("block predicate NBT must be a compound"));
        };
        Some(NbtPredicate { tag })
    } else {
        None
    };
    let components = read_component_matchers(context, data)?;

    Ok(BlockPredicate {
        blocks,
        properties,
        nbt,
        components,
    })
}

fn write_block_holder_set(
    context: &DataComponentCodecContext<'_>,
    blocks: &BlockHolderSet,
    writer: &mut Vec<u8>,
) -> Result<()> {
    match blocks {
        BlockHolderSet::Tag(tag) => {
            VarInt(0).write(writer)?;
            tag.write(writer)
        }
        BlockHolderSet::Blocks(blocks) => {
            let count = i32::try_from(blocks.len())
                .map_err(|_| Error::other("block holder set is too large"))?;
            VarInt(count + 1).write(writer)?;
            for block in blocks {
                let id = context
                    .registry()
                    .blocks
                    .id_from_key(&block.key)
                    .ok_or_else(|| Error::other(format!("unknown block holder {}", block.key)))?;
                VarInt(
                    i32::try_from(id)
                        .map_err(|_| Error::other(format!("block holder id out of range: {id}")))?,
                )
                .write(writer)?;
            }
            Ok(())
        }
    }
}

fn read_block_holder_set(
    context: &DataComponentCodecContext<'_>,
    data: &mut Cursor<&[u8]>,
) -> Result<BlockHolderSet> {
    let encoded_count = VarInt::read(data)?.0;
    if encoded_count == 0 {
        let tag = read_vanilla_identifier(data)?;
        if context.registry().blocks.get_tag(&tag).is_none() {
            return Err(Error::other(format!("unknown block holder tag: {tag}")));
        }
        return Ok(BlockHolderSet::Tag(tag));
    }
    if encoded_count < 0 {
        return Err(Error::other(format!(
            "negative block holder set count: {encoded_count}"
        )));
    }

    let count = usize::try_from(encoded_count - 1)
        .map_err(|_| Error::other("block holder set count out of range"))?;
    let mut blocks = Vec::with_capacity(count.min(65_536));
    for _ in 0..count {
        let id = VarInt::read(data)?.0;
        if id < 0 {
            return Err(Error::other(format!("negative block holder id: {id}")));
        }
        let block = context
            .registry()
            .blocks
            .by_id(id as usize)
            .ok_or_else(|| Error::other(format!("unknown block holder id: {id}")))?;
        blocks.push(block);
    }
    Ok(BlockHolderSet::Blocks(blocks))
}

fn write_state_properties(
    properties: &StatePropertiesPredicate,
    writer: &mut Vec<u8>,
) -> Result<()> {
    write_count(
        writer,
        properties.properties.len(),
        "state property matcher",
    )?;
    for property in &properties.properties {
        write_string(writer, &property.name)?;
        match &property.value {
            StatePropertyValueMatcher::Exact(value) => {
                true.write(writer)?;
                write_string(writer, value)?;
            }
            StatePropertyValueMatcher::Ranged { min, max } => {
                false.write(writer)?;
                write_optional_string(writer, min.as_deref())?;
                write_optional_string(writer, max.as_deref())?;
            }
        }
    }
    Ok(())
}

fn read_state_properties(data: &mut Cursor<&[u8]>) -> Result<StatePropertiesPredicate> {
    let count = read_count(data, "state property matcher")?;
    let mut properties = Vec::with_capacity(count.min(65_536));
    for _ in 0..count {
        let name = read_string(data)?;
        let value = if bool::read(data)? {
            StatePropertyValueMatcher::Exact(read_string(data)?)
        } else {
            StatePropertyValueMatcher::Ranged {
                min: read_optional_string(data)?,
                max: read_optional_string(data)?,
            }
        };
        properties.push(StatePropertyMatcher { name, value });
    }
    Ok(StatePropertiesPredicate { properties })
}

fn write_component_matchers(
    context: &DataComponentCodecContext<'_>,
    matchers: &DataComponentMatchers,
    writer: &mut Vec<u8>,
) -> Result<()> {
    write_count(
        writer,
        matchers.exact.components.len(),
        "exact data component predicate",
    )?;
    for expected in &matchers.exact.components {
        let id = context
            .registry()
            .data_components
            .id_from_key(&expected.component)
            .ok_or_else(|| {
                Error::other(format!("unknown data component {}", expected.component))
            })?;
        let entry = context
            .registry()
            .data_components
            .by_id(id)
            .ok_or_else(|| Error::other(format!("missing data component entry {id}")))?;
        VarInt(
            i32::try_from(id)
                .map_err(|_| Error::other(format!("data component id out of range: {id}")))?,
        )
        .write(writer)?;
        (entry.network_writer)(context, &expected.value, writer)?;
    }

    if matchers.partial.len() > MAX_PARTIAL_PREDICATES {
        return Err(Error::other(format!(
            "too many partial data component predicates: {}",
            matchers.partial.len()
        )));
    }
    write_count(
        writer,
        matchers.partial.len(),
        "partial data component predicate",
    )?;
    for predicate in &matchers.partial {
        match predicate {
            DataComponentPredicate::Any { component } => {
                false.write(writer)?;
                let id = context
                    .registry()
                    .data_components
                    .id_from_key(component)
                    .ok_or_else(|| Error::other(format!("unknown data component {component}")))?;
                VarInt(
                    i32::try_from(id).map_err(|_| {
                        Error::other(format!("data component id out of range: {id}"))
                    })?,
                )
                .write(writer)?;
                WriteTo::write(&NbtTag::Compound(NbtCompound::new()), writer)?;
            }
            DataComponentPredicate::Typed {
                predicate_type,
                value,
            } => {
                true.write(writer)?;
                VarInt(data_component_predicate_type_id(context, predicate_type)?).write(writer)?;
                WriteTo::write(value, writer)?;
            }
        }
    }
    Ok(())
}

fn read_component_matchers(
    context: &DataComponentCodecContext<'_>,
    data: &mut Cursor<&[u8]>,
) -> Result<DataComponentMatchers> {
    let exact_count = read_count(data, "exact data component predicate")?;
    let mut exact_components = Vec::with_capacity(exact_count.min(65_536));
    for _ in 0..exact_count {
        let id = VarInt::read(data)?.0;
        if id < 0 {
            return Err(Error::other(format!("negative data component id: {id}")));
        }
        let entry = context
            .registry()
            .data_components
            .by_id(id as usize)
            .ok_or_else(|| Error::other(format!("unknown data component id: {id}")))?;
        let value = (entry.network_reader)(context, data)?;
        exact_components.push(ExactDataComponentPredicate {
            component: entry.key.clone(),
            value,
        });
    }

    let partial_count = read_count(data, "partial data component predicate")?;
    if partial_count > MAX_PARTIAL_PREDICATES {
        return Err(Error::other(format!(
            "too many partial data component predicates: {partial_count}"
        )));
    }
    let mut partial = Vec::with_capacity(partial_count);
    for _ in 0..partial_count {
        let predicate = if bool::read(data)? {
            let id = VarInt::read(data)?.0;
            let predicate_type = data_component_predicate_type_by_id(context, id)?;
            DataComponentPredicate::Typed {
                predicate_type,
                value: read_nbt_tag(data)?,
            }
        } else {
            let id = VarInt::read(data)?.0;
            if id < 0 {
                return Err(Error::other(format!("negative data component id: {id}")));
            }
            let component = context
                .registry()
                .data_components
                .get_key_by_id(id as usize)
                .ok_or_else(|| Error::other(format!("unknown data component id: {id}")))?
                .clone();
            let value = read_nbt_tag(data)?;
            if !matches!(value, NbtTag::Compound(ref compound) if compound.is_empty()) {
                return Err(Error::other(
                    "any-value data component predicate must have an empty NBT payload",
                ));
            }
            DataComponentPredicate::Any { component }
        };

        if let DataComponentPredicate::Typed { predicate_type, .. } = &predicate
            && partial.iter().any(|existing| {
                matches!(
                    existing,
                    DataComponentPredicate::Typed {
                        predicate_type: existing_type,
                        ..
                    } if existing_type == predicate_type
                )
            })
        {
            return Err(Error::other(format!(
                "duplicate concrete data component predicate type: {predicate_type}"
            )));
        }
        partial.push(predicate);
    }

    Ok(DataComponentMatchers {
        exact: DataComponentExactPredicate {
            components: exact_components,
        },
        partial,
    })
}
fn write_count(writer: &mut Vec<u8>, count: usize, name: &str) -> Result<()> {
    let count = i32::try_from(count).map_err(|_| Error::other(format!("too many {name}s")))?;
    VarInt(count).write(writer)
}

fn read_count(data: &mut Cursor<&[u8]>, name: &str) -> Result<usize> {
    let count = VarInt::read(data)?.0;
    usize::try_from(count).map_err(|_| Error::other(format!("negative {name} count: {count}")))
}

pub(super) const MAX_STRING_UTF16_LENGTH: usize = 32_767;
pub(super) const MAX_STRING_UTF8_LENGTH: usize = MAX_STRING_UTF16_LENGTH * 3;

pub(super) fn write_string(writer: &mut Vec<u8>, value: &str) -> Result<()> {
    if value.encode_utf16().count() > MAX_STRING_UTF16_LENGTH {
        return Err(Error::other(
            "string exceeds Vanilla's 32767-character limit",
        ));
    }
    if value.len() > MAX_STRING_UTF8_LENGTH {
        return Err(Error::other(
            "string exceeds Vanilla's maximum UTF-8 encoded length",
        ));
    }

    VarInt(
        i32::try_from(value.len())
            .map_err(|_| Error::other("string byte length does not fit a VarInt"))?,
    )
    .write(writer)?;
    writer.write_all(value.as_bytes())
}

pub(super) fn read_string(data: &mut Cursor<&[u8]>) -> Result<String> {
    let byte_length = usize::try_from(VarInt::read(data)?.0)
        .map_err(|_| Error::other("negative string byte length"))?;
    if byte_length > MAX_STRING_UTF8_LENGTH {
        return Err(Error::other(
            "string exceeds Vanilla's maximum UTF-8 encoded length",
        ));
    }

    let mut bytes = vec![0; byte_length];
    data.read_exact(&mut bytes)?;
    let value = String::from_utf8(bytes).map_err(Error::other)?;
    if value.encode_utf16().count() > MAX_STRING_UTF16_LENGTH {
        return Err(Error::other(
            "string exceeds Vanilla's 32767-character limit",
        ));
    }

    Ok(value)
}

fn write_optional_string(writer: &mut Vec<u8>, value: Option<&str>) -> Result<()> {
    match value {
        Some(value) => {
            true.write(writer)?;
            write_string(writer, value)
        }
        None => false.write(writer),
    }
}

fn read_optional_string(data: &mut Cursor<&[u8]>) -> Result<Option<String>> {
    if bool::read(data)? {
        read_string(data).map(Some)
    } else {
        Ok(None)
    }
}

/// Parses identifiers with the default namespace rules used by Vanilla
pub(super) fn parse_vanilla_identifier(value: &str) -> Option<Identifier> {
    value.parse().ok()
}

fn read_vanilla_identifier(data: &mut Cursor<&[u8]>) -> Result<Identifier> {
    let value = read_string(data)?;
    parse_vanilla_identifier(&value)
        .ok_or_else(|| Error::other(format!("invalid resource location: {value}")))
}

pub(super) fn read_nbt_tag(data: &mut Cursor<&[u8]>) -> Result<NbtTag> {
    read_tag_with_default_quota(data)
}

fn data_component_predicate_type_by_id(
    context: &DataComponentCodecContext<'_>,
    id: i32,
) -> Result<Identifier> {
    let id = usize::try_from(id)
        .map_err(|_| Error::other(format!("negative data component predicate type id: {id}")))?;
    context
        .registry()
        .data_component_predicate_types
        .get_key_by_id(id)
        .cloned()
        .ok_or_else(|| Error::other(format!("unknown data component predicate type id: {id}")))
}

fn data_component_predicate_type_id(
    context: &DataComponentCodecContext<'_>,
    key: &Identifier,
) -> Result<i32> {
    let id = context
        .registry()
        .data_component_predicate_types
        .id_from_key(key)
        .ok_or_else(|| Error::other(format!("unknown data component predicate type: {key}")))?;
    i32::try_from(id).map_err(|_| {
        Error::other(format!(
            "data component predicate type id out of range: {id}"
        ))
    })
}
