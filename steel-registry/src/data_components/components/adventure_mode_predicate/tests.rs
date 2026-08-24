use std::{collections::BTreeMap, io::Cursor};

use super::network::{
    MAX_STRING_UTF8_LENGTH, MAX_STRING_UTF16_LENGTH, parse_vanilla_identifier, read_string,
    write_string,
};
use super::*;
use crate::{
    REGISTRY, RegistryExt,
    data_components::{
        Component,
        vanilla_components::{
            CAN_BREAK, CAN_PLACE_ON, CREATIVE_SLOT_LOCK, DAMAGE, ENCHANTMENT_GLINT_OVERRIDE,
            MAX_DAMAGE, MAX_STACK_SIZE,
        },
    },
    init_vanilla_registry,
};
use serde::Deserialize;
use simdnbt::owned::NbtList;
use steel_utils::{codec::VarInt, serial::WriteTo};

fn codec_context() -> DataComponentCodecContext<'static> {
    init_vanilla_registry();
    DataComponentCodecContext::new(&REGISTRY)
}

fn predicate(context: &DataComponentCodecContext<'_>) -> AdventureModePredicate {
    let oak_log = context
        .registry()
        .blocks
        .by_key(&Identifier::vanilla_static("oak_log"))
        .expect("oak log must be registered");

    let mut block_entity_nbt = NbtCompound::new();
    block_entity_nbt.insert("id", "minecraft:chest");

    AdventureModePredicate {
        predicates: vec![
            BlockPredicate {
                blocks: Some(BlockHolderSet::Blocks(vec![oak_log])),
                properties: Some(StatePropertiesPredicate {
                    properties: vec![
                        StatePropertyMatcher {
                            name: "axis".to_string(),
                            value: StatePropertyValueMatcher::Exact("y".to_string()),
                        },
                        StatePropertyMatcher {
                            name: "distance".to_string(),
                            value: StatePropertyValueMatcher::Ranged {
                                min: Some("1".to_string()),
                                max: Some("7".to_string()),
                            },
                        },
                    ],
                }),
                nbt: Some(NbtPredicate {
                    tag: block_entity_nbt,
                }),
                components: DataComponentMatchers {
                    exact: DataComponentExactPredicate {
                        components: vec![ExactDataComponentPredicate {
                            component: MAX_DAMAGE.key.clone(),
                            value: ComponentData::I32(1561),
                        }],
                    },
                    partial: vec![
                        DataComponentPredicate::Any {
                            component: MAX_STACK_SIZE.key.clone(),
                        },
                        DataComponentPredicate::Typed {
                            predicate_type: Identifier::vanilla_static("damage"),
                            value: NbtTag::Compound(NbtCompound::new()),
                        },
                    ],
                },
            },
            BlockPredicate {
                blocks: Some(BlockHolderSet::Tag(Identifier::vanilla_static(
                    "mineable/axe",
                ))),
                properties: None,
                nbt: None,
                components: DataComponentMatchers::ANY,
            },
        ],
    }
}

#[test]
fn stream_codec_round_trips_direct_and_tag_block_holders() {
    let context = codec_context();
    let predicate = predicate(&context);
    let data = predicate.clone().into_data();

    let mut bytes = Vec::new();
    network_writer(&context, &data, &mut bytes).expect("stream encoding must succeed");
    let decoded = network_reader(&context, &mut Cursor::new(bytes.as_slice()))
        .expect("stream decoding must succeed");

    assert_eq!(AdventureModePredicate::from_data(decoded), Some(predicate));
}

#[test]
fn persistent_codec_round_trips_compact_predicate_list() {
    let context = codec_context();
    let predicate = predicate(&context);
    let data = predicate.clone().into_data();
    let tag = nbt_writer(&context, &data);

    let mut bytes = Vec::new();
    tag.write(&mut bytes);
    let decoded_tag = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
        .expect("persistent NBT must decode");
    let decoded =
        nbt_reader(&context, (&decoded_tag).into()).expect("persistent codec must decode");

    assert_eq!(AdventureModePredicate::from_data(decoded), Some(predicate));
}

#[test]
fn persistent_nbt_predicate_uses_canonical_snbt_and_accepts_lenient_compounds() {
    let context = codec_context();
    let predicate = predicate(&context);
    let data = predicate.clone().into_data();
    let tag = nbt_writer(&context, &data);

    let NbtTag::List(NbtList::Compound(predicates)) = tag else {
        panic!("multiple predicates must persist as a compound list");
    };
    let persisted_nbt = predicates[0]
        .get("nbt")
        .expect("fixture predicate must persist an NBT condition");
    assert_eq!(
        persisted_nbt
            .string()
            .map(|value| value.to_str().into_owned()),
        Some("{id:\"minecraft:chest\"}".to_owned())
    );

    let mut raw_predicate = predicates[0].clone();
    let expected_nbt = predicate.predicates[0]
        .nbt
        .as_ref()
        .expect("fixture predicate must have NBT")
        .tag
        .clone();
    raw_predicate.insert("nbt", NbtTag::Compound(expected_nbt));
    let raw = NbtTag::Compound(raw_predicate);
    let mut bytes = Vec::new();
    raw.write(&mut bytes);
    let raw = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
        .expect("raw persistent NBT must parse");

    let decoded =
        nbt_reader(&context, raw.as_tag()).expect("lenient raw compound NBT predicate must decode");
    assert_eq!(
        AdventureModePredicate::from_data(decoded),
        Some(AdventureModePredicate {
            predicates: vec![predicate.predicates[0].clone()],
        })
    );
}

#[test]
fn persistent_exact_components_filter_and_reject_transient_values() {
    let context = codec_context();
    let transient_exact = AdventureModePredicate {
        predicates: vec![BlockPredicate {
            blocks: None,
            properties: None,
            nbt: None,
            components: DataComponentMatchers {
                exact: DataComponentExactPredicate {
                    components: vec![ExactDataComponentPredicate {
                        component: CREATIVE_SLOT_LOCK.key.clone(),
                        value: ComponentData::Empty,
                    }],
                },
                partial: Vec::new(),
            },
        }],
    };
    let transient_data = ComponentData::AdventureModePredicate(transient_exact);
    let tag = nbt_writer(&context, &transient_data);
    let NbtTag::Compound(compound) = tag else {
        panic!("single predicate must persist as a compound");
    };
    assert!(compound.get("components").is_none());

    let unconstrained = AdventureModePredicate {
        predicates: vec![BlockPredicate {
            blocks: None,
            properties: None,
            nbt: None,
            components: DataComponentMatchers::ANY,
        }],
    };
    assert_eq!(
        transient_data.compute_hash(),
        ComponentData::AdventureModePredicate(unconstrained).compute_hash()
    );

    let mut exact = NbtCompound::new();
    exact.insert("minecraft:creative_slot_lock", NbtCompound::new());
    let mut invalid = NbtCompound::new();
    invalid.insert("components", exact);
    let mut bytes = Vec::new();
    WriteTo::write(&NbtTag::Compound(invalid), &mut bytes).expect("writing to a vec must succeed");
    let borrowed = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
        .expect("transient exact component fixture must decode as NBT");

    assert!(nbt_reader(&context, borrowed.as_tag()).is_none());
}

#[derive(Deserialize)]
struct ComponentHashFixtures {
    adventure_mode_predicate: BTreeMap<String, i32>,
}

#[test]
fn component_hash_matches_vanilla_complex_predicate_fixture() {
    let context = codec_context();
    let predicate = predicate(&context);
    let fixtures: ComponentHashFixtures = serde_json::from_str(include_str!(
        "../../../../test_assets/component_hashes.json"
    ))
    .expect("component hash fixture must be valid JSON");
    let expected = fixtures
        .adventure_mode_predicate
        .get("complex")
        .expect("fixture must contain the complex adventure predicate");

    assert_eq!(
        ComponentData::AdventureModePredicate(predicate).compute_hash(),
        *expected
    );
}

#[test]
fn component_hash_matches_vanilla_boolean_exact_component_fixture() {
    let _context = codec_context();
    let predicate = AdventureModePredicate {
        predicates: vec![BlockPredicate {
            blocks: None,
            properties: None,
            nbt: None,
            components: DataComponentMatchers {
                exact: DataComponentExactPredicate {
                    components: vec![ExactDataComponentPredicate {
                        component: ENCHANTMENT_GLINT_OVERRIDE.key.clone(),
                        value: ComponentData::Bool(false),
                    }],
                },
                partial: Vec::new(),
            },
        }],
    };
    let fixtures: ComponentHashFixtures = serde_json::from_str(include_str!(
        "../../../../test_assets/component_hashes.json"
    ))
    .expect("component hash fixture must be valid JSON");
    let expected = fixtures
        .adventure_mode_predicate
        .get("boolean_component")
        .expect("fixture must contain the boolean adventure predicate");

    assert_eq!(
        ComponentData::AdventureModePredicate(predicate).compute_hash(),
        *expected
    );
}

#[test]
fn stream_uses_raw_holder_registry_ids() {
    let context = codec_context();
    let oak_log = context
        .registry()
        .blocks
        .by_key(&Identifier::vanilla_static("oak_log"))
        .expect("oak log must be registered");
    let predicate = AdventureModePredicate {
        predicates: vec![BlockPredicate {
            blocks: Some(BlockHolderSet::Blocks(vec![oak_log])),
            properties: None,
            nbt: None,
            components: DataComponentMatchers::ANY,
        }],
    };

    let mut bytes = Vec::new();
    network_writer(&context, &predicate.into_data(), &mut bytes)
        .expect("stream encoding must succeed");

    let block_id = context
        .registry()
        .blocks
        .id_from_key(&oak_log.key)
        .expect("oak log must have a registry id");
    let mut expected = vec![1, 1, 2];
    VarInt(i32::try_from(block_id).expect("block ID must fit a VarInt"))
        .write(&mut expected)
        .expect("writing to a vec must succeed");
    expected.extend([0, 0, 0, 0]);

    assert_eq!(bytes, expected);
}

#[test]
fn stream_defaults_unqualified_holder_tag_identifiers_to_minecraft() {
    let context = codec_context();
    let mut bytes = Vec::new();
    VarInt(1)
        .write(&mut bytes)
        .expect("writing predicate count to a vec must succeed");
    true.write(&mut bytes)
        .expect("writing block holder presence to a vec must succeed");
    VarInt(0)
        .write(&mut bytes)
        .expect("writing tag holder marker to a vec must succeed");
    write_string(&mut bytes, "mineable/axe")
        .expect("writing an unqualified holder tag must succeed");
    false
        .write(&mut bytes)
        .expect("writing state presence to a vec must succeed");
    false
        .write(&mut bytes)
        .expect("writing NBT presence to a vec must succeed");
    VarInt(0)
        .write(&mut bytes)
        .expect("writing exact component count to a vec must succeed");
    VarInt(0)
        .write(&mut bytes)
        .expect("writing partial component count to a vec must succeed");

    let decoded = network_reader(&context, &mut Cursor::new(bytes.as_slice()))
        .expect("unqualified holder tag must decode");
    assert_eq!(
        AdventureModePredicate::from_data(decoded),
        Some(AdventureModePredicate {
            predicates: vec![BlockPredicate {
                blocks: Some(BlockHolderSet::Tag(Identifier::vanilla_static(
                    "mineable/axe"
                ))),
                properties: None,
                nbt: None,
                components: DataComponentMatchers::ANY,
            }],
        })
    );
}

#[test]
fn persistent_codec_defaults_unqualified_holder_and_component_identifiers() {
    let context = codec_context();
    let oak_log = context
        .registry()
        .blocks
        .by_key(&Identifier::vanilla_static("oak_log"))
        .expect("oak log must be registered");

    let mut exact = NbtCompound::new();
    exact.insert("max_damage", 1561_i32);
    let mut partial = NbtCompound::new();
    partial.insert("max_stack_size", NbtCompound::new());
    let mut block = NbtCompound::new();
    block.insert("blocks", "oak_log");
    block.insert("components", exact);
    block.insert("predicates", partial);

    let mut bytes = Vec::new();
    WriteTo::write(&NbtTag::Compound(block), &mut bytes)
        .expect("writing persistent predicate NBT to a vec must succeed");
    let tag = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
        .expect("persistent predicate NBT must decode");
    let decoded =
        nbt_reader(&context, tag.as_tag()).expect("unqualified persistent identifiers must decode");

    assert_eq!(
        AdventureModePredicate::from_data(decoded),
        Some(AdventureModePredicate {
            predicates: vec![BlockPredicate {
                blocks: Some(BlockHolderSet::Blocks(vec![oak_log])),
                properties: None,
                nbt: None,
                components: DataComponentMatchers {
                    exact: DataComponentExactPredicate {
                        components: vec![ExactDataComponentPredicate {
                            component: MAX_DAMAGE.key.clone(),
                            value: ComponentData::I32(1561),
                        }],
                    },
                    partial: vec![DataComponentPredicate::Any {
                        component: MAX_STACK_SIZE.key.clone(),
                    }],
                },
            }],
        })
    );
}

#[test]
fn persistent_empty_direct_holder_set_uses_and_accepts_an_end_list() {
    let context = codec_context();
    let predicate = AdventureModePredicate {
        predicates: vec![BlockPredicate {
            blocks: Some(BlockHolderSet::Blocks(Vec::new())),
            properties: None,
            nbt: None,
            components: DataComponentMatchers::ANY,
        }],
    };
    let tag = nbt_writer(&context, &predicate.clone().into_data());

    let NbtTag::Compound(block_predicate) = &tag else {
        panic!("a single predicate must persist as a compound");
    };
    assert!(matches!(
        block_predicate.get("blocks"),
        Some(NbtTag::List(NbtList::Empty))
    ));

    let mut bytes = Vec::new();
    WriteTo::write(&tag, &mut bytes)
        .expect("writing persistent predicate NBT to a vec must succeed");
    let tag = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
        .expect("persistent predicate NBT must decode");
    let decoded =
        nbt_reader(&context, tag.as_tag()).expect("an end-list direct holder set must decode");
    assert_eq!(AdventureModePredicate::from_data(decoded), Some(predicate));
}

#[test]
fn vanilla_identifier_parser_defaults_an_empty_namespace() {
    let expected = Identifier::vanilla_static("oak_log");
    assert_eq!(parse_vanilla_identifier("oak_log"), Some(expected.clone()));
    assert_eq!(parse_vanilla_identifier(":oak_log"), Some(expected));
    assert!(parse_vanilla_identifier("..:oak_log").is_none());
}

#[test]
fn stream_rejects_unknown_block_holder_tags() {
    let context = codec_context();
    let mut bytes = vec![1, 1, 0];
    Identifier::vanilla_static("missing_tag")
        .write(&mut bytes)
        .expect("writing identifier must succeed");
    bytes.extend([0, 0, 0, 0]);

    let error = network_reader(&context, &mut Cursor::new(bytes.as_slice()))
        .expect_err("unknown block holder tag must be rejected");
    assert!(error.to_string().contains("unknown block holder tag"));
}

#[test]
fn stream_rejects_duplicate_concrete_partial_predicate_types() {
    let context = codec_context();
    let damage_type_id = context
        .registry()
        .data_component_predicate_types
        .id_from_key(&Identifier::vanilla_static("damage"))
        .expect("damage predicate type must be registered");
    let damage_type_id =
        i32::try_from(damage_type_id).expect("damage predicate type ID must fit a VarInt");

    let mut bytes = Vec::new();
    VarInt(1)
        .write(&mut bytes)
        .expect("writing to a vec must succeed");
    false
        .write(&mut bytes)
        .expect("writing to a vec must succeed");
    false
        .write(&mut bytes)
        .expect("writing to a vec must succeed");
    false
        .write(&mut bytes)
        .expect("writing to a vec must succeed");
    VarInt(0)
        .write(&mut bytes)
        .expect("writing to a vec must succeed");
    VarInt(2)
        .write(&mut bytes)
        .expect("writing to a vec must succeed");

    for _ in 0..2 {
        true.write(&mut bytes)
            .expect("writing to a vec must succeed");
        VarInt(damage_type_id)
            .write(&mut bytes)
            .expect("writing to a vec must succeed");
        WriteTo::write(&NbtTag::Compound(NbtCompound::new()), &mut bytes)
            .expect("writing to a vec must succeed");
    }

    assert!(network_reader(&context, &mut Cursor::new(bytes.as_slice())).is_err());
}

#[test]
fn stream_keeps_concrete_and_any_partial_predicate_types_independent() {
    let context = codec_context();
    let predicate = AdventureModePredicate {
        predicates: vec![BlockPredicate {
            blocks: None,
            properties: None,
            nbt: None,
            components: DataComponentMatchers {
                exact: DataComponentExactPredicate::EMPTY,
                partial: vec![
                    DataComponentPredicate::Typed {
                        predicate_type: Identifier::vanilla_static("damage"),
                        value: NbtTag::Compound(NbtCompound::new()),
                    },
                    DataComponentPredicate::Any {
                        component: DAMAGE.key.clone(),
                    },
                ],
            },
        }],
    };
    let data = predicate.clone().into_data();

    let mut bytes = Vec::new();
    network_writer(&context, &data, &mut bytes).expect("stream encoding must succeed");
    let decoded = network_reader(&context, &mut Cursor::new(bytes.as_slice()))
        .expect("stream decoding must succeed");

    assert_eq!(AdventureModePredicate::from_data(decoded), Some(predicate));
}

#[test]
fn state_property_stream_strings_use_vanilla_utf16_limits() {
    let value = format!("{}a", "\u{1f600}".repeat(16_383));
    assert_eq!(value.encode_utf16().count(), MAX_STRING_UTF16_LENGTH);

    let mut bytes = Vec::new();
    write_string(&mut bytes, &value).expect("maximum valid UTF-16 string must encode");
    assert_eq!(
        read_string(&mut Cursor::new(bytes.as_slice()))
            .expect("maximum valid UTF-16 string must decode"),
        value
    );

    let too_long = "\u{1f600}".repeat(16_384);
    assert!(write_string(&mut Vec::new(), &too_long).is_err());
}

#[test]
fn state_property_stream_strings_reject_excessive_utf8_payloads() {
    let mut bytes = Vec::new();
    VarInt(
        i32::try_from(MAX_STRING_UTF8_LENGTH + 1).expect("test string length must fit a VarInt"),
    )
    .write(&mut bytes)
    .expect("writing to a vec must succeed");

    assert!(read_string(&mut Cursor::new(bytes.as_slice())).is_err());
}

#[test]
fn vanilla_partial_predicate_type_ids_match_bootstrap_order() {
    let context = codec_context();
    let types = &context.registry().data_component_predicate_types;

    assert_eq!(
        types.id_from_key(&Identifier::vanilla_static("damage")),
        Some(0)
    );
    assert_eq!(
        types.id_from_key(&Identifier::vanilla_static("attribute_modifiers")),
        Some(11)
    );
    assert_eq!(
        types.id_from_key(&Identifier::vanilla_static("villager/variant")),
        Some(14)
    );
    assert_eq!(
        context
            .registry()
            .data_components
            .id_from_key(&CAN_PLACE_ON.key),
        Some(14)
    );
    assert_eq!(
        context
            .registry()
            .data_components
            .id_from_key(&CAN_BREAK.key),
        Some(15)
    );
}
