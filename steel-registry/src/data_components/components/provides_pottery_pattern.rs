//! Codecs for `minecraft:provides_pottery_pattern`.
//!
//! Vanilla stores a registered `Holder<DecoratedPotPattern>`: persistent data
//! uses its registry key, while the synchronized stream uses its numeric holder
//! ID. Keeping the reference typed avoids leaking registry IDs into item data.

use std::{
    io::{Cursor, Error, Result},
    str::FromStr,
};

use simdnbt::{borrow::NbtTag as BorrowedNbtTag, owned::NbtTag};
use steel_utils::{
    Identifier,
    codec::VarInt,
    hash::{ComponentHasher, HashComponent},
    serial::{ReadFrom, WriteTo},
};

use crate::{
    RegistryExt,
    data_components::{Component, ComponentData, DataComponentCodecContext},
    decorated_pot_pattern::DecoratedPotPatternRef,
};

/// A registered pottery pattern supplied by an item to a decorated pot
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProvidesPotteryPattern {
    pub pattern: DecoratedPotPatternRef,
}

impl HashComponent for ProvidesPotteryPattern {
    fn hash_component(&self, hasher: &mut ComponentHasher) {
        hasher.put_string(&self.pattern.key.to_string());
    }
}

/// Writes Vanilla `DecoratedPotPatterns.STREAM_CODEC` holderregistry ID
pub fn network_writer(
    context: &DataComponentCodecContext<'_>,
    data: &ComponentData,
    writer: &mut Vec<u8>,
) -> Result<()> {
    let Some(component) = ProvidesPotteryPattern::from_data_ref(data) else {
        return Err(Error::other(
            "Component type mismatch for provides_pottery_pattern",
        ));
    };

    let id = context
        .registry()
        .decorated_pot_patterns
        .id_from_key(&component.pattern.key)
        .ok_or_else(|| {
            Error::other(format!(
                "unknown decorated pot pattern {}",
                component.pattern.key
            ))
        })?;
    let id = i32::try_from(id).map_err(|_| {
        Error::other(format!(
            "decorated pot pattern id out of protocol range: {id}"
        ))
    })?;
    VarInt(id).write(writer)
}

/// Reads Vanilla `DecoratedPotPatterns.STREAM_CODEC` holderegistry ID
pub fn network_reader(
    context: &DataComponentCodecContext<'_>,
    reader: &mut Cursor<&[u8]>,
) -> Result<ComponentData> {
    let id = VarInt::read(reader)?.0;
    let id = usize::try_from(id)
        .map_err(|_| Error::other(format!("negative decorated pot pattern id: {id}")))?;
    let pattern = context
        .registry()
        .decorated_pot_patterns
        .by_id(id)
        .ok_or_else(|| Error::other(format!("unknown decorated pot pattern id: {id}")))?;
    Ok(ComponentData::ProvidesPotteryPattern(
        ProvidesPotteryPattern { pattern },
    ))
}

/// Writes Vanilla registry fixed persistent holder representation
#[must_use]
pub fn nbt_writer(context: &DataComponentCodecContext<'_>, data: &ComponentData) -> NbtTag {
    let Some(component) = ProvidesPotteryPattern::from_data_ref(data) else {
        panic!("Component type mismatch for provides_pottery_pattern");
    };

    if context
        .registry()
        .decorated_pot_patterns
        .by_key(&component.pattern.key)
        .is_none()
    {
        panic!(
            "provides_pottery_pattern references unregistered pattern {}",
            component.pattern.key
        );
    }

    NbtTag::String(component.pattern.key.to_string().into())
}

/// Reads Vanilla registry fixed persistent holder representation
#[must_use]
pub fn nbt_reader(
    context: &DataComponentCodecContext<'_>,
    tag: BorrowedNbtTag,
) -> Option<ComponentData> {
    let key = Identifier::from_str(&tag.string()?.to_str()).ok()?;
    let pattern = context.registry().decorated_pot_patterns.by_key(&key)?;
    Some(ComponentData::ProvidesPotteryPattern(
        ProvidesPotteryPattern { pattern },
    ))
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, io::Cursor, str::FromStr};

    use serde::Deserialize;
    use steel_utils::Identifier;

    use super::{nbt_reader, nbt_writer, network_reader, network_writer};
    use crate::{
        REGISTRY, RegistryExt,
        data_components::vanilla_components::PROVIDES_POTTERY_PATTERN,
        data_components::{ComponentData, DataComponentCodecContext, DataComponentPatch},
        test_support::init_test_registry,
    };

    #[derive(Deserialize)]
    struct ComponentHashFixtures {
        provides_pottery_pattern: BTreeMap<String, i32>,
    }

    fn context() -> DataComponentCodecContext<'static> {
        init_test_registry();
        DataComponentCodecContext::new(&REGISTRY)
    }

    fn fixtures() -> ComponentHashFixtures {
        serde_json::from_str(include_str!("../../../test_assets/component_hashes.json"))
            .expect("component hash fixture must be valid JSON")
    }

    #[test]
    fn generated_pottery_sherd_components_match_snapshot_2_hashes() {
        let context = context();
        let fixtures = fixtures();
        assert_eq!(fixtures.provides_pottery_pattern.len(), 23);

        for (item_key, expected_hash) in fixtures.provides_pottery_pattern {
            let item_key = Identifier::from_str(&item_key).expect("fixture item key must be valid");
            let item = context
                .registry()
                .items
                .by_key(&item_key)
                .expect("fixture item must be registered");
            let pattern = item
                .components
                .get_ref(PROVIDES_POTTERY_PATTERN)
                .expect("fixture item must provide a pottery pattern");

            assert_eq!(
                ComponentData::ProvidesPotteryPattern(*pattern).compute_hash(),
                expected_hash,
                "provides_pottery_pattern hash mismatch for {item_key}"
            );
        }
    }

    #[test]
    fn codecs_use_vanillas_registered_holder_forms() {
        let context = context();
        let snort = Identifier::vanilla_static("snort_pottery_sherd");
        let item = context
            .registry()
            .items
            .by_key(&snort)
            .expect("snort pottery sherd must be registered");
        let component = *item
            .components
            .get_ref(PROVIDES_POTTERY_PATTERN)
            .expect("snort pottery sherd must provide a pattern");
        let data = ComponentData::ProvidesPotteryPattern(component);

        let mut network = Vec::new();
        network_writer(&context, &data, &mut network)
            .expect("pottery pattern network encoding must succeed");
        assert_eq!(network, vec![22]);
        assert_eq!(
            network_reader(&context, &mut Cursor::new(network.as_slice()))
                .expect("pottery pattern network decoding must succeed"),
            data
        );

        let persistent = nbt_writer(&context, &data);
        let mut persistent_bytes = Vec::new();
        persistent.write(&mut persistent_bytes);
        let borrowed = simdnbt::borrow::read_tag(&mut Cursor::new(persistent_bytes.as_slice()))
            .expect("pottery pattern persistent data must be binary NBT");
        assert_eq!(nbt_reader(&context, borrowed.as_tag()), Some(data.clone()));

        let mut patch = DataComponentPatch::new();
        patch.set(PROVIDES_POTTERY_PATTERN, component);
        let mut patch_network = Vec::new();
        patch
            .write_with_context(&context, &mut patch_network)
            .expect("pottery pattern patch network encoding must succeed");
        assert_eq!(
            DataComponentPatch::read_with_context(
                &context,
                &mut Cursor::new(patch_network.as_slice()),
            )
            .expect("pottery pattern patch network decoding must succeed"),
            patch
        );

        let persistent_patch = patch.to_nbt_tag_with_context(&context);
        let mut persistent_patch_bytes = Vec::new();
        persistent_patch.write(&mut persistent_patch_bytes);
        let borrowed =
            simdnbt::borrow::read_tag(&mut Cursor::new(persistent_patch_bytes.as_slice()))
                .expect("pottery pattern patch persistent data must be binary NBT");
        assert_eq!(
            DataComponentPatch::from_nbt_tag_with_context(&context, borrowed.as_tag())
                .expect("pottery pattern patch persistent decoding must succeed"),
            patch
        );
    }
}
