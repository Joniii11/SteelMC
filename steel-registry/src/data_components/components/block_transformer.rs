//! `minecraft:block_transformer` codecs

use std::{
    io::{Cursor, Error, Result},
    str::FromStr,
};

use simdnbt::{
    borrow::NbtTag as BorrowedNbtTag,
    owned::{NbtCompound, NbtList, NbtTag},
};
use steel_utils::{
    Direction, Identifier,
    codec::VarInt,
    hash::{ComponentHasher, HashComponent, HashEntry, hash_nbt_tag, sort_map_entries},
    nbt::read_tag_with_default_quota,
    serial::{ReadFrom, WriteTo},
    value_providers::{IntProvider, VerticalAnchor, WeightedIntProvider},
};

use crate::{
    RegistryExt, TaggedRegistryExt,
    data_components::{Component, ComponentData, DataComponentCodecContext},
    sound_event::SoundEventHolder,
};

const MAX_TRANSFORMS: usize = 200;
const SMALL_OFFSET_LIMIT_EXCLUSIVE: u32 = 16;

/// Item block transforms
#[derive(Debug, Clone, PartialEq)]
pub struct BlockTransformer {
    /// Ordered transforms
    pub transforms: Vec<BlockTransformData>,
}

/// Block transform
#[derive(Debug, Clone, PartialEq)]
pub struct BlockTransformData {
    /// `BlockStateProvider`
    pub block_state_provider: TransformStateProvider,
    /// `Holder<SoundEvent>`
    pub sound: SoundEventHolder,
    pub particle: TransformParticle,
    pub disallowed_faces: Vec<Direction>,
    pub loot: Option<Identifier>,
    pub drop_strategy: DropStrategy,
    pub transform_type: TransformType,
    pub consume_on_use: bool,
    pub item_damage_per_use: i32,
}

/// Block state data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBlockState {
    pub block: Identifier,
    pub properties: Vec<(String, String)>,
}

/// Registry holder set
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformHolderSet {
    Tag(Identifier),
    Entries(Vec<Identifier>),
}

/// Weighted block state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedTransformBlockState {
    pub data: TransformBlockState,
    pub weight: i32,
}

/// `NormalNoise.NoiseParameters`
#[derive(Debug, Clone, PartialEq)]
pub struct TransformNoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

/// `RuleBasedStateProvider` rule
#[derive(Debug, Clone, PartialEq)]
pub struct TransformStateProviderRule {
    pub if_true: TransformPredicate,
    pub then: TransformStateProvider,
}

/// `BlockStateProvider`
#[derive(Debug, Clone, PartialEq)]
pub enum TransformStateProvider {
    Simple {
        state: TransformBlockState,
    },
    Weighted {
        entries: Vec<WeightedTransformBlockState>,
    },
    NoiseThreshold {
        seed: i64,
        noise: TransformNoiseParameters,
        scale: f32,
        threshold: f32,
        high_chance: f32,
        default_state: TransformBlockState,
        low_states: Vec<TransformBlockState>,
        high_states: Vec<TransformBlockState>,
    },
    Noise {
        seed: i64,
        noise: TransformNoiseParameters,
        scale: f32,
        states: Vec<TransformBlockState>,
    },
    DualNoise {
        variety: (i32, i32),
        slow_noise: TransformNoiseParameters,
        slow_scale: f32,
        seed: i64,
        noise: TransformNoiseParameters,
        scale: f32,
        states: Vec<TransformBlockState>,
    },
    /// Block default state
    RotatedBlock {
        block: Identifier,
    },
    RandomizedInt {
        source: Box<TransformStateProvider>,
        property: String,
        values: IntProvider,
    },
    RuleBased {
        fallback: Option<Box<TransformStateProvider>>,
        rules: Vec<TransformStateProviderRule>,
    },
    CopyProperties {
        source: Box<TransformStateProvider>,
    },
}

/// `BlockPredicate`
#[derive(Debug, Clone, PartialEq)]
pub enum TransformPredicate {
    MatchingBlocks {
        offset: (i32, i32, i32),
        blocks: TransformHolderSet,
    },
    MatchingBlockTag {
        offset: (i32, i32, i32),
        tag: Identifier,
    },
    MatchingFluids {
        offset: (i32, i32, i32),
        fluids: TransformHolderSet,
    },
    MatchingBiomes {
        biomes: TransformHolderSet,
    },
    HasSturdyFace {
        offset: (i32, i32, i32),
        direction: Direction,
    },
    Solid {
        offset: (i32, i32, i32),
    },
    Replaceable {
        offset: (i32, i32, i32),
    },
    WouldSurvive {
        offset: (i32, i32, i32),
        state: TransformBlockState,
    },
    InsideWorldBounds {
        offset: (i32, i32, i32),
    },
    Any(Vec<TransformPredicate>),
    All(Vec<TransformPredicate>),
    Not(Box<TransformPredicate>),
    True,
    /// Unused `Vec3i`
    Unobstructed {
        offset: (i32, i32, i32),
    },
    HeightRange {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformParticle {
    #[default]
    None,
    Scrape,
    WaxOn,
    WaxOff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropStrategy {
    ClickedFace,
    #[default]
    FromMiddle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformType {
    #[default]
    SingleBlock,
    CopperChest,
}

impl HashComponent for BlockTransformer {
    fn hash_component(&self, hasher: &mut ComponentHasher) {
        let context = DataComponentCodecContext::new(&crate::REGISTRY);
        hasher.start_list();
        for transform in &self.transforms {
            let mut transform_hasher = ComponentHasher::new();
            hash_transform(&mut transform_hasher, &context, transform);
            hasher.put_raw_bytes(&(transform_hasher.finish() as u32).to_le_bytes());
        }
        hasher.end_list();
    }
}

/// `BlockTransformData.CODEC` hash
fn hash_transform(
    hasher: &mut ComponentHasher,
    context: &DataComponentCodecContext<'_>,
    transform: &BlockTransformData,
) {
    let mut entries = Vec::with_capacity(9);
    push_hash_nbt_entry(
        &mut entries,
        "block_state_provider",
        provider_nbt(context, &transform.block_state_provider),
    );
    if !is_empty_sound(&transform.sound) {
        push_hash_nbt_entry(&mut entries, "sound", sound_holder_nbt(&transform.sound));
    }
    if transform.particle != TransformParticle::None {
        push_hash_entry(&mut entries, "particle", particle_name(transform.particle));
    }
    if !transform.disallowed_faces.is_empty() {
        push_hash_nbt_entry(
            &mut entries,
            "disallowed_faces",
            NbtTag::List(NbtList::String(
                transform
                    .disallowed_faces
                    .iter()
                    .map(|face| direction_name(*face).into())
                    .collect(),
            )),
        );
    }
    if let Some(loot) = &transform.loot {
        push_hash_entry(&mut entries, "loot", &loot.to_string());
    }
    if transform.drop_strategy != DropStrategy::FromMiddle {
        push_hash_entry(
            &mut entries,
            "drop_strategy",
            drop_strategy_name(transform.drop_strategy),
        );
    }
    if transform.transform_type != TransformType::SingleBlock {
        push_hash_entry(
            &mut entries,
            "transform_type",
            transform_type_name(transform.transform_type),
        );
    }
    if !transform.consume_on_use {
        push_hash_entry(&mut entries, "consume_on_use", &transform.consume_on_use);
    }
    if transform.item_damage_per_use != 0 {
        push_hash_entry(
            &mut entries,
            "item_damage_per_use",
            &transform.item_damage_per_use,
        );
    }

    sort_map_entries(&mut entries);
    hasher.start_map();
    for entry in entries {
        hasher.put_raw_bytes(&entry.key_bytes);
        hasher.put_raw_bytes(&entry.value_bytes);
    }
    hasher.end_map();
}

fn push_hash_nbt_entry(entries: &mut Vec<HashEntry>, key: &str, value: NbtTag) {
    let mut key_hasher = ComponentHasher::new();
    key_hasher.put_string(key);
    let mut value_hasher = ComponentHasher::new();
    hash_nbt_tag(&mut value_hasher, &value);
    entries.push(HashEntry::new(key_hasher, value_hasher));
}

fn push_hash_entry<T: HashComponent + ?Sized>(entries: &mut Vec<HashEntry>, key: &str, value: &T) {
    let mut key_hasher = ComponentHasher::new();
    key_hasher.put_string(key);
    let mut value_hasher = ComponentHasher::new();
    value.hash_component(&mut value_hasher);
    entries.push(HashEntry::new(key_hasher, value_hasher));
}

/// `BlockTransformer.STREAM_CODEC`
pub fn network_writer(
    context: &DataComponentCodecContext<'_>,
    data: &ComponentData,
    writer: &mut Vec<u8>,
) -> Result<()> {
    let Some(transformer) = BlockTransformer::from_data_ref(data) else {
        return Err(Error::other(
            "Component type mismatch for block_transformer",
        ));
    };

    VarInt(
        i32::try_from(transformer.transforms.len())
            .map_err(|_| Error::other("too many block transforms"))?,
    )
    .write(writer)?;
    for transform in &transformer.transforms {
        WriteTo::write(
            &provider_nbt(context, &transform.block_state_provider),
            writer,
        )?;
        write_sound_holder(context, &transform.sound, writer)?;
        VarInt(particle_id(transform.particle)).write(writer)?;
        VarInt(
            i32::try_from(transform.disallowed_faces.len())
                .map_err(|_| Error::other("too many block transformer disallowed faces"))?,
        )
        .write(writer)?;
        for face in &transform.disallowed_faces {
            VarInt(direction_id(*face)).write(writer)?;
        }
        transform.loot.write(writer)?;
        VarInt(drop_strategy_id(transform.drop_strategy)).write(writer)?;
        VarInt(transform_type_id(transform.transform_type)).write(writer)?;
        transform.consume_on_use.write(writer)?;
        VarInt(transform.item_damage_per_use).write(writer)?;
    }
    Ok(())
}

/// `BlockTransformer.STREAM_CODEC`
pub fn network_reader(
    context: &DataComponentCodecContext<'_>,
    data: &mut Cursor<&[u8]>,
) -> Result<ComponentData> {
    let transform_count = read_count(data, "block transform count")?;
    let mut transforms = Vec::with_capacity(transform_count.min(65_536) as usize);

    for _ in 0..transform_count {
        let provider = read_tag_with_default_quota(data)
            .map_err(|error| Error::other(format!("invalid block state provider NBT: {error}")))?;
        let block_state_provider = decode_provider(context, &provider)?;
        let sound = read_sound_holder(context, data)?;
        let particle = particle_from_id(VarInt::read(data)?.0);

        let face_count = read_count(data, "block transformer disallowed face count")?;
        let mut disallowed_faces = Vec::with_capacity(face_count.min(65_536) as usize);
        for _ in 0..face_count {
            // Wrapping direction ID
            disallowed_faces.push(direction_from_id(VarInt::read(data)?.0));
        }

        transforms.push(BlockTransformData {
            block_state_provider,
            sound,
            particle,
            disallowed_faces,
            loot: Option::<Identifier>::read(data)?,
            // Zero fallback enum ID
            drop_strategy: drop_strategy_from_id(VarInt::read(data)?.0),
            transform_type: transform_type_from_id(VarInt::read(data)?.0),
            consume_on_use: bool::read(data)?,
            item_damage_per_use: VarInt::read(data)?.0,
        });
    }

    Ok(ComponentData::BlockTransformer(BlockTransformer {
        transforms,
    }))
}

/// `BlockTransformer.CODEC`
#[must_use]
pub fn nbt_writer(context: &DataComponentCodecContext<'_>, data: &ComponentData) -> NbtTag {
    let Some(transformer) = BlockTransformer::from_data_ref(data) else {
        panic!("Component type mismatch for block_transformer");
    };
    block_transformer_nbt(context, transformer)
}

/// `BlockTransformer.CODEC`
pub fn nbt_reader(
    context: &DataComponentCodecContext<'_>,
    tag: BorrowedNbtTag,
) -> Option<ComponentData> {
    let tag = tag.to_owned();
    decode_transformer_nbt(context, &tag)
        .ok()
        .map(ComponentData::BlockTransformer)
}

fn read_count(data: &mut Cursor<&[u8]>, field: &str) -> Result<i32> {
    let count = VarInt::read(data)?.0;
    if count < 0 {
        return Err(Error::other(format!("negative {field}: {count}")));
    }
    Ok(count)
}

fn decode_transformer_nbt(
    context: &DataComponentCodecContext<'_>,
    tag: &NbtTag,
) -> Result<BlockTransformer> {
    let transforms = compound_list(tag, "block_transformer")?;
    if !(1..=MAX_TRANSFORMS).contains(&transforms.len()) {
        return Err(Error::other(format!(
            "block_transformer must contain 1 to {MAX_TRANSFORMS} transforms, got {}",
            transforms.len()
        )));
    }

    let transforms = transforms
        .iter()
        .map(|transform| decode_transform_nbt(context, transform))
        .collect::<Result<Vec<_>>>()?;
    Ok(BlockTransformer { transforms })
}

fn decode_transform_nbt(
    context: &DataComponentCodecContext<'_>,
    transform: &NbtCompound,
) -> Result<BlockTransformData> {
    let block_state_provider = decode_provider(
        context,
        required_tag(transform, "block_state_provider", "block transformer")?,
    )?;
    let sound = match transform.get("sound") {
        Some(tag) => read_sound_holder_nbt(context, tag)?,
        None => empty_sound_holder(context)?,
    };
    let particle = match transform.get("particle") {
        Some(tag) => particle_from_name(&required_string(tag, "block transformer particle")?)?,
        None => TransformParticle::None,
    };
    let disallowed_faces = match transform.get("disallowed_faces") {
        Some(tag) => read_directions_nbt(tag)?,
        None => Vec::new(),
    };
    let loot = transform
        .get("loot")
        .map(|tag| identifier_from_tag(tag, "block transformer loot"))
        .transpose()?;
    let drop_strategy = match transform.get("drop_strategy") {
        Some(tag) => {
            drop_strategy_from_name(&required_string(tag, "block transformer drop_strategy")?)?
        }
        None => DropStrategy::FromMiddle,
    };
    let transform_type = match transform.get("transform_type") {
        Some(tag) => {
            transform_type_from_name(&required_string(tag, "block transformer transform_type")?)?
        }
        None => TransformType::SingleBlock,
    };
    let consume_on_use = match transform.get("consume_on_use") {
        Some(tag) => bool_from_tag(tag, "block transformer consume_on_use")?,
        None => true,
    };
    let item_damage_per_use = match transform.get("item_damage_per_use") {
        Some(tag) => {
            let value = required_i32(tag, "block transformer item_damage_per_use")?;
            if value < 0 {
                return Err(Error::other(
                    "block transformer item_damage_per_use must be non-negative",
                ));
            }
            value
        }
        None => 0,
    };

    Ok(BlockTransformData {
        block_state_provider,
        sound,
        particle,
        disallowed_faces,
        loot,
        drop_strategy,
        transform_type,
        consume_on_use,
        item_damage_per_use,
    })
}

fn decode_provider(
    context: &DataComponentCodecContext<'_>,
    tag: &NbtTag,
) -> Result<TransformStateProvider> {
    let provider = required_compound(tag, "block state provider")?;
    let provider_type = required_string_field(provider, "type", "block state provider")?;
    match provider_type.as_str() {
        "minecraft:simple_state_provider" => Ok(TransformStateProvider::Simple {
            state: decode_block_state(
                context,
                required_tag(provider, "state", "simple state provider")?,
            )?,
        }),
        "minecraft:weighted_state_provider" => {
            let entries = required_compound_list(provider, "entries", "weighted state provider")?
                .iter()
                .map(|entry| decode_weighted_state(context, entry))
                .collect::<Result<Vec<_>>>()?;
            if entries.is_empty() {
                return Err(Error::other(
                    "weighted state provider must contain at least one entry",
                ));
            }
            validate_weight_total(
                entries.iter().map(|entry| entry.weight),
                "weighted state provider",
            )?;
            Ok(TransformStateProvider::Weighted { entries })
        }
        "minecraft:noise_threshold_provider" => {
            let scale = required_f32_field(provider, "scale", "noise threshold provider")?;
            validate_positive_f32(scale, "noise threshold provider scale")?;
            let threshold = required_f32_field(provider, "threshold", "noise threshold provider")?;
            if !(-1.0..=1.0).contains(&threshold) {
                return Err(Error::other(
                    "noise threshold provider threshold must be in -1.0..=1.0",
                ));
            }
            let high_chance =
                required_f32_field(provider, "high_chance", "noise threshold provider")?;
            if !(0.0..=1.0).contains(&high_chance) {
                return Err(Error::other(
                    "noise threshold provider high_chance must be in 0.0..=1.0",
                ));
            }
            let low_states = decode_block_state_list(
                context,
                required_tag(provider, "low_states", "noise threshold provider")?,
                "noise threshold provider low_states",
            )?;
            let high_states = decode_block_state_list(
                context,
                required_tag(provider, "high_states", "noise threshold provider")?,
                "noise threshold provider high_states",
            )?;
            if low_states.is_empty() || high_states.is_empty() {
                return Err(Error::other(
                    "noise threshold provider low_states and high_states must be non-empty",
                ));
            }
            Ok(TransformStateProvider::NoiseThreshold {
                seed: required_i64_field(provider, "seed", "noise threshold provider")?,
                noise: decode_noise_parameters(required_tag(
                    provider,
                    "noise",
                    "noise threshold provider",
                )?)?,
                scale,
                threshold,
                high_chance,
                default_state: decode_block_state(
                    context,
                    required_tag(provider, "default_state", "noise threshold provider")?,
                )?,
                low_states,
                high_states,
            })
        }
        "minecraft:noise_provider" => {
            let scale = required_f32_field(provider, "scale", "noise provider")?;
            validate_positive_f32(scale, "noise provider scale")?;
            let states = decode_block_state_list(
                context,
                required_tag(provider, "states", "noise provider")?,
                "noise provider states",
            )?;
            if states.is_empty() {
                return Err(Error::other("noise provider states must be non-empty"));
            }
            Ok(TransformStateProvider::Noise {
                seed: required_i64_field(provider, "seed", "noise provider")?,
                noise: decode_noise_parameters(required_tag(provider, "noise", "noise provider")?)?,
                scale,
                states,
            })
        }
        "minecraft:dual_noise_provider" => {
            let slow_scale = required_f32_field(provider, "slow_scale", "dual noise provider")?;
            validate_positive_f32(slow_scale, "dual noise provider slow_scale")?;
            let scale = required_f32_field(provider, "scale", "dual noise provider")?;
            validate_positive_f32(scale, "dual noise provider scale")?;
            let variety =
                decode_variety(required_tag(provider, "variety", "dual noise provider")?)?;
            let states = decode_block_state_list(
                context,
                required_tag(provider, "states", "dual noise provider")?,
                "dual noise provider states",
            )?;
            if states.is_empty() {
                return Err(Error::other("dual noise provider states must be non-empty"));
            }
            Ok(TransformStateProvider::DualNoise {
                variety,
                slow_noise: decode_noise_parameters(required_tag(
                    provider,
                    "slow_noise",
                    "dual noise provider",
                )?)?,
                slow_scale,
                seed: required_i64_field(provider, "seed", "dual noise provider")?,
                noise: decode_noise_parameters(required_tag(
                    provider,
                    "noise",
                    "dual noise provider",
                )?)?,
                scale,
                states,
            })
        }
        "minecraft:rotated_block_provider" => {
            let state = decode_block_state(
                context,
                required_tag(provider, "state", "rotated block provider")?,
            )?;
            Ok(TransformStateProvider::RotatedBlock { block: state.block })
        }
        "minecraft:randomized_int_state_provider" => Ok(TransformStateProvider::RandomizedInt {
            source: Box::new(decode_provider(
                context,
                required_tag(provider, "source", "randomized int state provider")?,
            )?),
            property: required_string_field(provider, "property", "randomized int state provider")?,
            values: decode_int_provider(required_tag(
                provider,
                "values",
                "randomized int state provider",
            )?)?,
        }),
        "minecraft:rule_based_state_provider" => {
            let fallback = provider
                .get("fallback")
                .map(|fallback| decode_provider(context, fallback).map(Box::new))
                .transpose()?;
            let rules = required_compound_list(provider, "rules", "rule based state provider")?
                .iter()
                .map(|rule| {
                    Ok(TransformStateProviderRule {
                        if_true: decode_predicate(
                            context,
                            required_tag(rule, "if_true", "rule based state provider rule")?,
                        )?,
                        then: decode_provider(
                            context,
                            required_tag(rule, "then", "rule based state provider rule")?,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(TransformStateProvider::RuleBased { fallback, rules })
        }
        "minecraft:copy_properties_provider" => Ok(TransformStateProvider::CopyProperties {
            source: Box::new(decode_provider(
                context,
                required_tag(
                    provider,
                    "source_block_state_provider",
                    "copy properties provider",
                )?,
            )?),
        }),
        _ => Err(Error::other(format!(
            "unsupported block transformer provider {provider_type}"
        ))),
    }
}

fn decode_weighted_state(
    context: &DataComponentCodecContext<'_>,
    entry: &NbtCompound,
) -> Result<WeightedTransformBlockState> {
    let weight = required_i32_field(entry, "weight", "weighted state provider entry")?;
    if weight < 0 {
        return Err(Error::other(
            "weighted state provider entry weight must be non-negative",
        ));
    }
    Ok(WeightedTransformBlockState {
        data: decode_block_state(
            context,
            required_tag(entry, "data", "weighted state provider entry")?,
        )?,
        weight,
    })
}

fn decode_block_state(
    context: &DataComponentCodecContext<'_>,
    tag: &NbtTag,
) -> Result<TransformBlockState> {
    let state = required_compound(tag, "block state")?;
    let block = identifier_from_tag(
        required_tag(state, "Name", "block state")?,
        "block state Name",
    )?;
    let properties = match state.get("Properties") {
        Some(properties) => decode_state_properties(properties)?,
        None => Vec::new(),
    };
    normalize_block_state(context, block, properties)
}

fn normalize_block_state(
    context: &DataComponentCodecContext<'_>,
    block: Identifier,
    properties: Vec<(String, String)>,
) -> Result<TransformBlockState> {
    let Some(block_ref) = context.registry().blocks.by_key(&block) else {
        return Err(Error::other(format!("unknown block state block {block}")));
    };
    let property_refs = properties
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()));
    let Some(state) = context
        .registry()
        .blocks
        .state_id_from_block_defaulted_properties(block_ref, property_refs)
    else {
        return Err(Error::other(format!(
            "invalid properties for block state {block}"
        )));
    };

    Ok(TransformBlockState {
        block,
        properties: context
            .registry()
            .blocks
            .get_properties(state)
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect(),
    })
}

fn decode_state_properties(tag: &NbtTag) -> Result<Vec<(String, String)>> {
    let properties = required_compound(tag, "block state Properties")?;
    let mut values = Vec::with_capacity(properties.len());
    for (name, value) in properties.iter() {
        let name = name.to_str().into_owned();
        if values.iter().any(|(existing, _)| existing == &name) {
            return Err(Error::other(format!(
                "duplicate block state property {name}"
            )));
        }
        values.push((name, required_string(value, "block state property value")?));
    }
    Ok(values)
}

fn decode_block_state_list(
    context: &DataComponentCodecContext<'_>,
    tag: &NbtTag,
    owner: &str,
) -> Result<Vec<TransformBlockState>> {
    compound_list(tag, owner)?
        .iter()
        .map(|state| decode_block_state(context, &NbtTag::Compound((*state).clone())))
        .collect()
}

fn decode_noise_parameters(tag: &NbtTag) -> Result<TransformNoiseParameters> {
    let parameters = required_compound(tag, "noise parameters")?;
    let amplitudes = required_tag(parameters, "amplitudes", "noise parameters")?;
    let amplitudes = match amplitudes {
        NbtTag::List(NbtList::Double(values)) => values.clone(),
        NbtTag::List(NbtList::Empty) => Vec::new(),
        _ => {
            return Err(Error::other(
                "noise parameters amplitudes must be a double list",
            ));
        }
    };
    Ok(TransformNoiseParameters {
        first_octave: required_i32_field(parameters, "firstOctave", "noise parameters")?,
        amplitudes,
    })
}

fn decode_variety(tag: &NbtTag) -> Result<(i32, i32)> {
    let values = match tag {
        NbtTag::List(NbtList::Int(values)) => values.as_slice(),
        _ => {
            return Err(Error::other(
                "dual noise provider variety must be an int list",
            ));
        }
    };
    let [min, max] = values else {
        return Err(Error::other(
            "dual noise provider variety must contain exactly two entries",
        ));
    };
    if !((1..=64).contains(min) && (1..=64).contains(max) && min <= max) {
        return Err(Error::other(
            "dual noise provider variety must be an inclusive range within 1..=64",
        ));
    }
    Ok((*min, *max))
}

fn decode_int_provider(tag: &NbtTag) -> Result<IntProvider> {
    if let Some(value) = number_i32(tag) {
        return Ok(IntProvider::Constant(value));
    }

    let provider = required_compound(tag, "int provider")?;
    let provider_type = required_string_field(provider, "type", "int provider")?;
    match provider_type.as_str() {
        "minecraft:constant" => Ok(IntProvider::Constant(required_i32_field(
            provider,
            "value",
            "constant int provider",
        )?)),
        "minecraft:uniform" => {
            let (min_inclusive, max_inclusive) =
                decode_int_range(provider, "uniform int provider")?;
            Ok(IntProvider::Uniform {
                min_inclusive,
                max_inclusive,
            })
        }
        "minecraft:biased_to_bottom" => {
            let (min_inclusive, max_inclusive) =
                decode_int_range(provider, "biased to bottom int provider")?;
            Ok(IntProvider::BiasedToBottom {
                min_inclusive,
                max_inclusive,
            })
        }
        "minecraft:very_biased_to_bottom" => {
            let (min_inclusive, max_inclusive) =
                decode_int_range(provider, "very biased to bottom int provider")?;
            Ok(IntProvider::VeryBiasedToBottom {
                min_inclusive,
                max_inclusive,
            })
        }
        "minecraft:trapezoid" => {
            let min = required_i32_field(provider, "min", "trapezoid int provider")?;
            let max = required_i32_field(provider, "max", "trapezoid int provider")?;
            let plateau = required_i32_field(provider, "plateau", "trapezoid int provider")?;
            if max < min || plateau > max.saturating_sub(min) {
                return Err(Error::other("invalid trapezoid int provider range"));
            }
            Ok(IntProvider::Trapezoid { min, max, plateau })
        }
        "minecraft:clamped_normal" => {
            let (min_inclusive, max_inclusive) =
                decode_int_range(provider, "clamped normal int provider")?;
            Ok(IntProvider::ClampedNormal {
                mean: required_f32_field(provider, "mean", "clamped normal int provider")?,
                deviation: required_f32_field(
                    provider,
                    "deviation",
                    "clamped normal int provider",
                )?,
                min_inclusive,
                max_inclusive,
            })
        }
        "minecraft:clamped" => {
            let (min_inclusive, max_inclusive) =
                decode_int_range(provider, "clamped int provider")?;
            Ok(IntProvider::Clamped {
                source: Box::new(decode_int_provider(required_tag(
                    provider,
                    "source",
                    "clamped int provider",
                )?)?),
                min_inclusive,
                max_inclusive,
            })
        }
        "minecraft:weighted_list" => {
            let entries =
                required_compound_list(provider, "distribution", "weighted int provider")?
                    .iter()
                    .map(|entry| {
                        let weight =
                            required_i32_field(entry, "weight", "weighted int provider entry")?;
                        if weight < 0 {
                            return Err(Error::other(
                                "weighted int provider entry weight must be non-negative",
                            ));
                        }
                        Ok(WeightedIntProvider {
                            data: decode_int_provider(required_tag(
                                entry,
                                "data",
                                "weighted int provider entry",
                            )?)?,
                            weight,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
            if entries.is_empty() {
                return Err(Error::other(
                    "weighted int provider distribution must be non-empty",
                ));
            }
            validate_weight_total(
                entries.iter().map(|entry| entry.weight),
                "weighted int provider distribution",
            )?;
            Ok(IntProvider::WeightedList {
                distribution: entries,
            })
        }
        _ => Err(Error::other(format!(
            "unsupported block transformer int provider {provider_type}"
        ))),
    }
}

fn decode_int_range(provider: &NbtCompound, owner: &str) -> Result<(i32, i32)> {
    let min_inclusive = required_i32_field(provider, "min_inclusive", owner)?;
    let max_inclusive = required_i32_field(provider, "max_inclusive", owner)?;
    if max_inclusive < min_inclusive {
        return Err(Error::other(format!("{owner} maximum is below minimum")));
    }
    Ok((min_inclusive, max_inclusive))
}

fn validate_weight_total(weights: impl IntoIterator<Item = i32>, owner: &str) -> Result<()> {
    let mut total = 0_i64;
    for weight in weights {
        total += i64::from(weight);
        if total > i64::from(i32::MAX) {
            return Err(Error::other(format!(
                "{owner} total weight must be at most {}",
                i32::MAX
            )));
        }
    }
    if total == 0 {
        return Err(Error::other(format!(
            "{owner} must contain at least one entry with non-zero weight"
        )));
    }
    Ok(())
}

fn decode_predicate(
    context: &DataComponentCodecContext<'_>,
    tag: &NbtTag,
) -> Result<TransformPredicate> {
    let predicate = required_compound(tag, "block transformer predicate")?;
    let predicate_type = required_string_field(predicate, "type", "block transformer predicate")?;
    match predicate_type.as_str() {
        "minecraft:matching_blocks" => Ok(TransformPredicate::MatchingBlocks {
            offset: decode_small_offset(predicate, "matching_blocks predicate")?,
            blocks: decode_holder_set(
                required_tag(predicate, "blocks", "matching_blocks predicate")?,
                "matching_blocks predicate blocks",
                |identifier| context.registry().blocks.by_key(identifier).is_some(),
                |identifier| context.registry().blocks.get_tag(identifier).is_some(),
            )?,
        }),
        "minecraft:matching_block_tag" => {
            let tag = identifier_from_tag(
                required_tag(predicate, "tag", "matching_block_tag predicate")?,
                "matching_block_tag predicate tag",
            )?;
            if context.registry().blocks.get_tag(&tag).is_none() {
                return Err(Error::other(format!(
                    "unknown matching_block_tag predicate tag {tag}"
                )));
            }
            Ok(TransformPredicate::MatchingBlockTag {
                offset: decode_small_offset(predicate, "matching_block_tag predicate")?,
                tag,
            })
        }
        "minecraft:matching_fluids" => Ok(TransformPredicate::MatchingFluids {
            offset: decode_small_offset(predicate, "matching_fluids predicate")?,
            fluids: decode_holder_set(
                required_tag(predicate, "fluids", "matching_fluids predicate")?,
                "matching_fluids predicate fluids",
                |identifier| context.registry().fluids.by_key(identifier).is_some(),
                |identifier| context.registry().fluids.get_tag(identifier).is_some(),
            )?,
        }),
        "minecraft:matching_biomes" => Ok(TransformPredicate::MatchingBiomes {
            biomes: decode_holder_set(
                required_tag(predicate, "biomes", "matching_biomes predicate")?,
                "matching_biomes predicate biomes",
                |identifier| context.registry().biomes.by_key(identifier).is_some(),
                |identifier| context.registry().biomes.get_tag(identifier).is_some(),
            )?,
        }),
        "minecraft:has_sturdy_face" => Ok(TransformPredicate::HasSturdyFace {
            offset: decode_small_offset(predicate, "has_sturdy_face predicate")?,
            direction: direction_from_name(&required_string_field(
                predicate,
                "direction",
                "has_sturdy_face predicate",
            )?)?,
        }),
        "minecraft:solid" => Ok(TransformPredicate::Solid {
            offset: decode_small_offset(predicate, "solid predicate")?,
        }),
        "minecraft:replaceable" => Ok(TransformPredicate::Replaceable {
            offset: decode_small_offset(predicate, "replaceable predicate")?,
        }),
        "minecraft:would_survive" => Ok(TransformPredicate::WouldSurvive {
            offset: decode_small_offset(predicate, "would_survive predicate")?,
            state: decode_block_state(
                context,
                required_tag(predicate, "state", "would_survive predicate")?,
            )?,
        }),
        "minecraft:inside_world_bounds" => Ok(TransformPredicate::InsideWorldBounds {
            offset: decode_small_offset(predicate, "inside_world_bounds predicate")?,
        }),
        "minecraft:any_of" => Ok(TransformPredicate::Any(
            required_compound_list(predicate, "predicates", "any_of predicate")?
                .iter()
                .map(|predicate| decode_predicate(context, &NbtTag::Compound((*predicate).clone())))
                .collect::<Result<Vec<_>>>()?,
        )),
        "minecraft:all_of" => Ok(TransformPredicate::All(
            required_compound_list(predicate, "predicates", "all_of predicate")?
                .iter()
                .map(|predicate| decode_predicate(context, &NbtTag::Compound((*predicate).clone())))
                .collect::<Result<Vec<_>>>()?,
        )),
        "minecraft:not" => Ok(TransformPredicate::Not(Box::new(decode_predicate(
            context,
            required_tag(predicate, "predicate", "not predicate")?,
        )?))),
        "minecraft:true" => Ok(TransformPredicate::True),
        "minecraft:unobstructed" => Ok(TransformPredicate::Unobstructed {
            offset: decode_unbounded_offset(predicate, "unobstructed predicate")?,
        }),
        "minecraft:height_range" => Ok(TransformPredicate::HeightRange {
            min_inclusive: decode_vertical_anchor(required_tag(
                predicate,
                "min_inclusive",
                "height_range predicate",
            )?)?,
            max_inclusive: decode_vertical_anchor(required_tag(
                predicate,
                "max_inclusive",
                "height_range predicate",
            )?)?,
        }),
        _ => Err(Error::other(format!(
            "unsupported block transformer predicate {predicate_type}"
        ))),
    }
}

fn decode_holder_set(
    tag: &NbtTag,
    owner: &str,
    entry_exists: impl Fn(&Identifier) -> bool,
    tag_exists: impl Fn(&Identifier) -> bool,
) -> Result<TransformHolderSet> {
    if let Some(value) = tag.string() {
        let value = value.to_str();
        if let Some(tag) = value.strip_prefix('#') {
            let tag = identifier_from_string(tag, owner)?;
            if !tag_exists(&tag) {
                return Err(Error::other(format!("unknown {owner} tag {tag}")));
            }
            return Ok(TransformHolderSet::Tag(tag));
        }
        let entry = identifier_from_string(&value, owner)?;
        if !entry_exists(&entry) {
            return Err(Error::other(format!("unknown {owner} entry {entry}")));
        }
        return Ok(TransformHolderSet::Entries(vec![entry]));
    }

    let values = match tag {
        NbtTag::List(NbtList::String(values)) => values,
        NbtTag::List(NbtList::Empty) => return Ok(TransformHolderSet::Entries(Vec::new())),
        _ => {
            return Err(Error::other(format!(
                "{owner} must be a string or string list"
            )));
        }
    };
    let entries = values
        .iter()
        .map(|value| {
            let entry = identifier_from_string(&value.to_str(), owner)?;
            if entry_exists(&entry) {
                Ok(entry)
            } else {
                Err(Error::other(format!("unknown {owner} entry {entry}")))
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(TransformHolderSet::Entries(entries))
}

fn decode_small_offset(predicate: &NbtCompound, owner: &str) -> Result<(i32, i32, i32)> {
    decode_offset(predicate, owner, Some(SMALL_OFFSET_LIMIT_EXCLUSIVE))
}

fn decode_unbounded_offset(predicate: &NbtCompound, owner: &str) -> Result<(i32, i32, i32)> {
    decode_offset(predicate, owner, None)
}

fn decode_offset(
    predicate: &NbtCompound,
    owner: &str,
    max_absolute_exclusive: Option<u32>,
) -> Result<(i32, i32, i32)> {
    let Some(offset) = predicate.get("offset") else {
        return Ok((0, 0, 0));
    };
    let values = match offset {
        // Canonical `Vec3i` NBT
        NbtTag::IntArray(values) => values.as_slice(),
        // Accepted dynamic NBT
        NbtTag::List(NbtList::Int(values)) => values.as_slice(),
        _ => {
            return Err(Error::other(format!(
                "{owner} offset must be an int stream"
            )));
        }
    };
    let [x, y, z] = values else {
        return Err(Error::other(format!(
            "{owner} offset must have exactly three entries"
        )));
    };
    if let Some(limit) = max_absolute_exclusive
        && [*x, *y, *z]
            .into_iter()
            .any(|value| value.unsigned_abs() >= limit)
    {
        return Err(Error::other(format!(
            "{owner} offset must have every axis within -{}..{}",
            limit - 1,
            limit - 1
        )));
    }
    Ok((*x, *y, *z))
}

fn decode_vertical_anchor(tag: &NbtTag) -> Result<VerticalAnchor> {
    let anchor = required_compound(tag, "vertical anchor")?;
    let candidates = [
        ("absolute", anchor.get("absolute")),
        ("above_bottom", anchor.get("above_bottom")),
        ("below_top", anchor.get("below_top")),
        ("relative_to_sea_level", anchor.get("relative_to_sea_level")),
    ];
    let present = candidates
        .iter()
        .filter_map(|(name, value)| value.map(|value| (*name, value)))
        .collect::<Vec<_>>();
    let [(kind, value)] = present.as_slice() else {
        return Err(Error::other(
            "vertical anchor must contain exactly one anchor kind",
        ));
    };
    let value = required_i32(value, "vertical anchor value")?;
    Ok(match *kind {
        "absolute" => VerticalAnchor::Absolute(value),
        "above_bottom" => VerticalAnchor::AboveBottom(value),
        "below_top" => VerticalAnchor::BelowTop(value),
        "relative_to_sea_level" => VerticalAnchor::RelativeToSeaLevel(value),
        _ => unreachable!("vertical anchor candidates are exhaustive"),
    })
}

fn block_transformer_nbt(
    context: &DataComponentCodecContext<'_>,
    transformer: &BlockTransformer,
) -> NbtTag {
    NbtTag::List(nbt_list_or_empty(
        transformer
            .transforms
            .iter()
            .map(|transform| transform_nbt(context, transform))
            .collect(),
        NbtList::Compound,
    ))
}

fn transform_nbt(
    context: &DataComponentCodecContext<'_>,
    transform: &BlockTransformData,
) -> NbtCompound {
    let mut value = NbtCompound::new();
    value.insert(
        "block_state_provider",
        provider_nbt(context, &transform.block_state_provider),
    );
    if !is_empty_sound(&transform.sound) {
        value.insert("sound", sound_holder_nbt(&transform.sound));
    }
    if transform.particle != TransformParticle::None {
        value.insert("particle", particle_name(transform.particle));
    }
    if !transform.disallowed_faces.is_empty() {
        value.insert(
            "disallowed_faces",
            nbt_list_or_empty(
                transform
                    .disallowed_faces
                    .iter()
                    .map(|face| direction_name(*face).into())
                    .collect(),
                NbtList::String,
            ),
        );
    }
    if let Some(loot) = &transform.loot {
        value.insert("loot", loot.to_string());
    }
    if transform.drop_strategy != DropStrategy::FromMiddle {
        value.insert("drop_strategy", drop_strategy_name(transform.drop_strategy));
    }
    if transform.transform_type != TransformType::SingleBlock {
        value.insert(
            "transform_type",
            transform_type_name(transform.transform_type),
        );
    }
    if !transform.consume_on_use {
        value.insert("consume_on_use", 0_i8);
    }
    if transform.item_damage_per_use != 0 {
        value.insert("item_damage_per_use", transform.item_damage_per_use);
    }
    value
}

fn provider_nbt(
    context: &DataComponentCodecContext<'_>,
    provider: &TransformStateProvider,
) -> NbtTag {
    let mut value = NbtCompound::new();
    match provider {
        TransformStateProvider::Simple { state } => {
            value.insert("state", block_state_nbt(state));
            value.insert("type", "minecraft:simple_state_provider");
        }
        TransformStateProvider::Weighted { entries } => {
            value.insert(
                "entries",
                nbt_list_or_empty(
                    entries
                        .iter()
                        .map(|entry| {
                            let mut entry_nbt = NbtCompound::new();
                            entry_nbt.insert("data", block_state_nbt(&entry.data));
                            entry_nbt.insert("weight", entry.weight);
                            entry_nbt
                        })
                        .collect(),
                    NbtList::Compound,
                ),
            );
            value.insert("type", "minecraft:weighted_state_provider");
        }
        TransformStateProvider::NoiseThreshold {
            seed,
            noise,
            scale,
            threshold,
            high_chance,
            default_state,
            low_states,
            high_states,
        } => {
            value.insert("seed", *seed);
            value.insert("noise", noise_parameters_nbt(noise));
            value.insert("scale", *scale);
            value.insert("threshold", *threshold);
            value.insert("high_chance", *high_chance);
            value.insert("default_state", block_state_nbt(default_state));
            value.insert("low_states", block_state_list_nbt(low_states));
            value.insert("high_states", block_state_list_nbt(high_states));
            value.insert("type", "minecraft:noise_threshold_provider");
        }
        TransformStateProvider::Noise {
            seed,
            noise,
            scale,
            states,
        } => {
            value.insert("seed", *seed);
            value.insert("noise", noise_parameters_nbt(noise));
            value.insert("scale", *scale);
            value.insert("states", block_state_list_nbt(states));
            value.insert("type", "minecraft:noise_provider");
        }
        TransformStateProvider::DualNoise {
            variety,
            slow_noise,
            slow_scale,
            seed,
            noise,
            scale,
            states,
        } => {
            value.insert("variety", NbtList::Int(vec![variety.0, variety.1]));
            value.insert("slow_noise", noise_parameters_nbt(slow_noise));
            value.insert("slow_scale", *slow_scale);
            value.insert("seed", *seed);
            value.insert("noise", noise_parameters_nbt(noise));
            value.insert("scale", *scale);
            value.insert("states", block_state_list_nbt(states));
            value.insert("type", "minecraft:dual_noise_provider");
        }
        TransformStateProvider::RotatedBlock { block } => {
            value.insert(
                "state",
                block_state_nbt(&default_block_state(context, block)),
            );
            value.insert("type", "minecraft:rotated_block_provider");
        }
        TransformStateProvider::RandomizedInt {
            source,
            property,
            values,
        } => {
            value.insert("source", provider_nbt(context, source));
            value.insert("property", property.as_str());
            value.insert("values", int_provider_nbt(values));
            value.insert("type", "minecraft:randomized_int_state_provider");
        }
        TransformStateProvider::RuleBased { fallback, rules } => {
            if let Some(fallback) = fallback {
                value.insert("fallback", provider_nbt(context, fallback));
            }
            value.insert(
                "rules",
                nbt_list_or_empty(
                    rules
                        .iter()
                        .map(|rule| {
                            let mut rule_nbt = NbtCompound::new();
                            rule_nbt.insert("if_true", predicate_nbt(&rule.if_true));
                            rule_nbt.insert("then", provider_nbt(context, &rule.then));
                            rule_nbt
                        })
                        .collect(),
                    NbtList::Compound,
                ),
            );
            value.insert("type", "minecraft:rule_based_state_provider");
        }
        TransformStateProvider::CopyProperties { source } => {
            value.insert("source_block_state_provider", provider_nbt(context, source));
            value.insert("type", "minecraft:copy_properties_provider");
        }
    }
    NbtTag::Compound(value)
}

fn block_state_nbt(state: &TransformBlockState) -> NbtTag {
    let mut value = NbtCompound::new();
    value.insert("Name", state.block.to_string());
    if !state.properties.is_empty() {
        let mut properties = NbtCompound::new();
        for (name, property) in &state.properties {
            properties.insert(name.as_str(), property.as_str());
        }
        value.insert("Properties", properties);
    }
    NbtTag::Compound(value)
}

fn default_block_state(
    context: &DataComponentCodecContext<'_>,
    block: &Identifier,
) -> TransformBlockState {
    let block_ref = context
        .registry()
        .blocks
        .by_key(block)
        .unwrap_or_else(|| panic!("unknown rotated block provider block {block}"));
    let state = context.registry().blocks.get_default_state_id(block_ref);
    TransformBlockState {
        block: block.clone(),
        properties: context
            .registry()
            .blocks
            .get_properties(state)
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect(),
    }
}

fn block_state_list_nbt(states: &[TransformBlockState]) -> NbtList {
    nbt_list_or_empty(
        states
            .iter()
            .map(|state| {
                let NbtTag::Compound(state) = block_state_nbt(state) else {
                    unreachable!("block state always encodes as a compound")
                };
                state
            })
            .collect(),
        NbtList::Compound,
    )
}

fn noise_parameters_nbt(parameters: &TransformNoiseParameters) -> NbtCompound {
    let mut value = NbtCompound::new();
    value.insert("firstOctave", parameters.first_octave);
    value.insert(
        "amplitudes",
        nbt_list_or_empty(parameters.amplitudes.clone(), NbtList::Double),
    );
    value
}

fn int_provider_nbt(provider: &IntProvider) -> NbtTag {
    match provider {
        IntProvider::Constant(value) => NbtTag::Int(*value),
        IntProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => int_provider_compound(
            "minecraft:uniform",
            [
                ("min_inclusive", NbtTag::Int(*min_inclusive)),
                ("max_inclusive", NbtTag::Int(*max_inclusive)),
            ],
        ),
        IntProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => int_provider_compound(
            "minecraft:biased_to_bottom",
            [
                ("min_inclusive", NbtTag::Int(*min_inclusive)),
                ("max_inclusive", NbtTag::Int(*max_inclusive)),
            ],
        ),
        IntProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => int_provider_compound(
            "minecraft:very_biased_to_bottom",
            [
                ("min_inclusive", NbtTag::Int(*min_inclusive)),
                ("max_inclusive", NbtTag::Int(*max_inclusive)),
            ],
        ),
        IntProvider::Trapezoid { min, max, plateau } => int_provider_compound(
            "minecraft:trapezoid",
            [
                ("min", NbtTag::Int(*min)),
                ("max", NbtTag::Int(*max)),
                ("plateau", NbtTag::Int(*plateau)),
            ],
        ),
        IntProvider::ClampedNormal {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        } => int_provider_compound(
            "minecraft:clamped_normal",
            [
                ("mean", NbtTag::Float(*mean)),
                ("deviation", NbtTag::Float(*deviation)),
                ("min_inclusive", NbtTag::Int(*min_inclusive)),
                ("max_inclusive", NbtTag::Int(*max_inclusive)),
            ],
        ),
        IntProvider::Clamped {
            source,
            min_inclusive,
            max_inclusive,
        } => int_provider_compound(
            "minecraft:clamped",
            [
                ("source", int_provider_nbt(source)),
                ("min_inclusive", NbtTag::Int(*min_inclusive)),
                ("max_inclusive", NbtTag::Int(*max_inclusive)),
            ],
        ),
        IntProvider::WeightedList { distribution } => int_provider_compound(
            "minecraft:weighted_list",
            [(
                "distribution",
                NbtTag::List(nbt_list_or_empty(
                    distribution
                        .iter()
                        .map(|entry| {
                            let mut value = NbtCompound::new();
                            value.insert("data", int_provider_nbt(&entry.data));
                            value.insert("weight", entry.weight);
                            value
                        })
                        .collect(),
                    NbtList::Compound,
                )),
            )],
        ),
    }
}

fn int_provider_compound<const N: usize>(
    provider_type: &str,
    entries: [(&str, NbtTag); N],
) -> NbtTag {
    let mut value = NbtCompound::new();
    for (name, entry) in entries {
        value.insert(name, entry);
    }
    value.insert("type", provider_type);
    NbtTag::Compound(value)
}

fn predicate_nbt(predicate: &TransformPredicate) -> NbtTag {
    let mut value = NbtCompound::new();
    match predicate {
        TransformPredicate::MatchingBlocks { offset, blocks } => {
            value.insert("blocks", holder_set_nbt(blocks));
            insert_offset(&mut value, *offset);
            value.insert("type", "minecraft:matching_blocks");
        }
        TransformPredicate::MatchingBlockTag { offset, tag } => {
            insert_offset(&mut value, *offset);
            value.insert("tag", tag.to_string());
            value.insert("type", "minecraft:matching_block_tag");
        }
        TransformPredicate::MatchingFluids { offset, fluids } => {
            value.insert("fluids", holder_set_nbt(fluids));
            insert_offset(&mut value, *offset);
            value.insert("type", "minecraft:matching_fluids");
        }
        TransformPredicate::MatchingBiomes { biomes } => {
            value.insert("biomes", holder_set_nbt(biomes));
            value.insert("type", "minecraft:matching_biomes");
        }
        TransformPredicate::HasSturdyFace { offset, direction } => {
            insert_offset(&mut value, *offset);
            value.insert("direction", direction_name(*direction));
            value.insert("type", "minecraft:has_sturdy_face");
        }
        TransformPredicate::Solid { offset } => {
            insert_offset(&mut value, *offset);
            value.insert("type", "minecraft:solid");
        }
        TransformPredicate::Replaceable { offset } => {
            insert_offset(&mut value, *offset);
            value.insert("type", "minecraft:replaceable");
        }
        TransformPredicate::WouldSurvive { offset, state } => {
            insert_offset(&mut value, *offset);
            value.insert("state", block_state_nbt(state));
            value.insert("type", "minecraft:would_survive");
        }
        TransformPredicate::InsideWorldBounds { offset } => {
            insert_offset(&mut value, *offset);
            value.insert("type", "minecraft:inside_world_bounds");
        }
        TransformPredicate::Any(predicates) => {
            value.insert("predicates", predicate_list_nbt(predicates));
            value.insert("type", "minecraft:any_of");
        }
        TransformPredicate::All(predicates) => {
            value.insert("predicates", predicate_list_nbt(predicates));
            value.insert("type", "minecraft:all_of");
        }
        TransformPredicate::Not(predicate) => {
            value.insert("predicate", predicate_nbt(predicate));
            value.insert("type", "minecraft:not");
        }
        TransformPredicate::True => {
            value.insert("type", "minecraft:true");
        }
        TransformPredicate::Unobstructed { offset } => {
            insert_offset(&mut value, *offset);
            value.insert("type", "minecraft:unobstructed");
        }
        TransformPredicate::HeightRange {
            min_inclusive,
            max_inclusive,
        } => {
            value.insert("min_inclusive", vertical_anchor_nbt(*min_inclusive));
            value.insert("max_inclusive", vertical_anchor_nbt(*max_inclusive));
            value.insert("type", "minecraft:height_range");
        }
    }
    NbtTag::Compound(value)
}

fn holder_set_nbt(holder_set: &TransformHolderSet) -> NbtTag {
    match holder_set {
        TransformHolderSet::Tag(tag) => NbtTag::String(format!("#{tag}").into()),
        TransformHolderSet::Entries(entries) if entries.len() == 1 => {
            NbtTag::String(entries[0].to_string().into())
        }
        TransformHolderSet::Entries(entries) => NbtTag::List(nbt_list_or_empty(
            entries
                .iter()
                .map(|entry| entry.to_string().into())
                .collect(),
            NbtList::String,
        )),
    }
}

fn insert_offset(value: &mut NbtCompound, offset: (i32, i32, i32)) {
    if offset != (0, 0, 0) {
        value.insert(
            "offset",
            NbtTag::IntArray(vec![offset.0, offset.1, offset.2]),
        );
    }
}

fn predicate_list_nbt(predicates: &[TransformPredicate]) -> NbtList {
    nbt_list_or_empty(
        predicates
            .iter()
            .map(|predicate| {
                let NbtTag::Compound(predicate) = predicate_nbt(predicate) else {
                    unreachable!("block predicate always encodes as a compound")
                };
                predicate
            })
            .collect(),
        NbtList::Compound,
    )
}

/// Empty `NbtOps` list
fn nbt_list_or_empty<T>(values: Vec<T>, make_list: impl FnOnce(Vec<T>) -> NbtList) -> NbtList {
    if values.is_empty() {
        NbtList::Empty
    } else {
        make_list(values)
    }
}

fn vertical_anchor_nbt(anchor: VerticalAnchor) -> NbtTag {
    let mut value = NbtCompound::new();
    match anchor {
        VerticalAnchor::Absolute(offset) => value.insert("absolute", offset),
        VerticalAnchor::AboveBottom(offset) => value.insert("above_bottom", offset),
        VerticalAnchor::BelowTop(offset) => value.insert("below_top", offset),
        VerticalAnchor::RelativeToSeaLevel(offset) => value.insert("relative_to_sea_level", offset),
    }
    NbtTag::Compound(value)
}

fn read_directions_nbt(tag: &NbtTag) -> Result<Vec<Direction>> {
    let faces = match tag {
        NbtTag::List(NbtList::String(faces)) => faces,
        NbtTag::List(NbtList::Empty) => return Ok(Vec::new()),
        _ => {
            return Err(Error::other(
                "block transformer disallowed_faces must be a string list",
            ));
        }
    };
    faces
        .iter()
        .map(|face| direction_from_name(&face.to_str()))
        .collect()
}

fn required_tag<'a>(compound: &'a NbtCompound, field: &str, owner: &str) -> Result<&'a NbtTag> {
    compound
        .get(field)
        .ok_or_else(|| Error::other(format!("{owner} is missing {field}")))
}

fn required_compound<'a>(tag: &'a NbtTag, owner: &str) -> Result<&'a NbtCompound> {
    tag.compound()
        .ok_or_else(|| Error::other(format!("{owner} must be an NBT compound")))
}

fn compound_list<'a>(tag: &'a NbtTag, owner: &str) -> Result<&'a [NbtCompound]> {
    match tag {
        NbtTag::List(NbtList::Compound(values)) => Ok(values),
        NbtTag::List(NbtList::Empty) => Ok(&[]),
        _ => Err(Error::other(format!("{owner} must be a compound list"))),
    }
}

fn required_compound_list<'a>(
    compound: &'a NbtCompound,
    field: &str,
    owner: &str,
) -> Result<&'a [NbtCompound]> {
    compound_list(
        required_tag(compound, field, owner)?,
        &format!("{owner} {field}"),
    )
}

fn required_string_field(compound: &NbtCompound, field: &str, owner: &str) -> Result<String> {
    required_string(
        required_tag(compound, field, owner)?,
        &format!("{owner} {field}"),
    )
}

fn required_string(tag: &NbtTag, owner: &str) -> Result<String> {
    tag.string()
        .map(|value| value.to_str().into_owned())
        .ok_or_else(|| Error::other(format!("{owner} must be a string")))
}

fn identifier_from_tag(tag: &NbtTag, owner: &str) -> Result<Identifier> {
    identifier_from_string(&required_string(tag, owner)?, owner)
}

fn identifier_from_string(value: &str, owner: &str) -> Result<Identifier> {
    Identifier::from_str(value)
        .map_err(|error| Error::other(format!("invalid {owner} identifier {value:?}: {error}")))
}

fn required_i32_field(compound: &NbtCompound, field: &str, owner: &str) -> Result<i32> {
    required_i32(
        required_tag(compound, field, owner)?,
        &format!("{owner} {field}"),
    )
}

fn required_i64_field(compound: &NbtCompound, field: &str, owner: &str) -> Result<i64> {
    required_i64(
        required_tag(compound, field, owner)?,
        &format!("{owner} {field}"),
    )
}

fn required_f32_field(compound: &NbtCompound, field: &str, owner: &str) -> Result<f32> {
    required_f32(
        required_tag(compound, field, owner)?,
        &format!("{owner} {field}"),
    )
}

fn required_i32(tag: &NbtTag, owner: &str) -> Result<i32> {
    number_i32(tag).ok_or_else(|| Error::other(format!("{owner} must be numeric")))
}

fn required_i64(tag: &NbtTag, owner: &str) -> Result<i64> {
    number_i64(tag).ok_or_else(|| Error::other(format!("{owner} must be numeric")))
}

fn required_f32(tag: &NbtTag, owner: &str) -> Result<f32> {
    number_f32(tag).ok_or_else(|| Error::other(format!("{owner} must be numeric")))
}

fn number_i32(tag: &NbtTag) -> Option<i32> {
    Some(match tag {
        NbtTag::Byte(value) => i32::from(*value),
        NbtTag::Short(value) => i32::from(*value),
        NbtTag::Int(value) => *value,
        NbtTag::Long(value) => *value as i32,
        NbtTag::Float(value) => *value as i32,
        NbtTag::Double(value) => *value as i32,
        _ => return None,
    })
}

fn number_i64(tag: &NbtTag) -> Option<i64> {
    Some(match tag {
        NbtTag::Byte(value) => i64::from(*value),
        NbtTag::Short(value) => i64::from(*value),
        NbtTag::Int(value) => i64::from(*value),
        NbtTag::Long(value) => *value,
        NbtTag::Float(value) => *value as i64,
        NbtTag::Double(value) => *value as i64,
        _ => return None,
    })
}

fn number_f32(tag: &NbtTag) -> Option<f32> {
    Some(match tag {
        NbtTag::Byte(value) => f32::from(*value),
        NbtTag::Short(value) => f32::from(*value),
        NbtTag::Int(value) => *value as f32,
        NbtTag::Long(value) => *value as f32,
        NbtTag::Float(value) => *value,
        NbtTag::Double(value) => *value as f32,
        _ => return None,
    })
}

fn bool_from_tag(tag: &NbtTag, owner: &str) -> Result<bool> {
    match tag {
        NbtTag::Byte(value) => Ok(*value != 0),
        _ => Err(Error::other(format!("{owner} must be a boolean byte"))),
    }
}

fn validate_positive_f32(value: f32, owner: &str) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::other(format!(
            "{owner} must be a positive finite float"
        )))
    }
}

fn write_sound_holder(
    context: &DataComponentCodecContext<'_>,
    sound: &SoundEventHolder,
    writer: &mut Vec<u8>,
) -> Result<()> {
    match sound {
        SoundEventHolder::Registry(sound) => {
            let Some(id) = context.registry().sound_events.id_from_key(&sound.key) else {
                return Err(Error::other(format!("unknown sound event {}", sound.key)));
            };
            VarInt(i32::try_from(id + 1).map_err(|_| Error::other("sound event id out of range"))?)
                .write(writer)
        }
        SoundEventHolder::Direct {
            sound_id,
            fixed_range,
        } => {
            VarInt(0).write(writer)?;
            sound_id.write(writer)?;
            fixed_range.write(writer)
        }
    }
}

fn read_sound_holder(
    context: &DataComponentCodecContext<'_>,
    data: &mut Cursor<&[u8]>,
) -> Result<SoundEventHolder> {
    let holder_id = VarInt::read(data)?.0;
    if holder_id == 0 {
        return Ok(SoundEventHolder::Direct {
            sound_id: Identifier::read(data)?,
            fixed_range: Option::<f32>::read(data)?,
        });
    }
    if holder_id < 0 {
        return Err(Error::other(format!(
            "negative sound event holder id: {holder_id}"
        )));
    }
    context
        .registry()
        .sound_events
        .by_id((holder_id - 1) as usize)
        .map(SoundEventHolder::Registry)
        .ok_or_else(|| Error::other(format!("unknown sound event holder id: {holder_id}")))
}

fn sound_holder_nbt(sound: &SoundEventHolder) -> NbtTag {
    match sound {
        SoundEventHolder::Registry(sound) => NbtTag::String(sound.key.to_string().into()),
        SoundEventHolder::Direct {
            sound_id,
            fixed_range,
        } => {
            let mut value = NbtCompound::new();
            value.insert("sound_id", sound_id.to_string());
            if let Some(range) = fixed_range {
                value.insert("range", *range);
            }
            NbtTag::Compound(value)
        }
    }
}

fn read_sound_holder_nbt(
    context: &DataComponentCodecContext<'_>,
    tag: &NbtTag,
) -> Result<SoundEventHolder> {
    if tag.string().is_some() {
        let id = identifier_from_tag(tag, "block transformer sound")?;
        return context
            .registry()
            .sound_events
            .by_key(&id)
            .map(SoundEventHolder::Registry)
            .ok_or_else(|| Error::other(format!("unknown sound event {id}")));
    }

    let sound = required_compound(tag, "block transformer direct sound")?;
    let sound_id = identifier_from_tag(
        required_tag(sound, "sound_id", "block transformer direct sound")?,
        "block transformer direct sound_id",
    )?;
    let fixed_range = sound
        .get("range")
        .map(|tag| required_f32(tag, "block transformer direct sound range"))
        .transpose()?;
    Ok(SoundEventHolder::Direct {
        sound_id,
        fixed_range,
    })
}

fn empty_sound_holder(context: &DataComponentCodecContext<'_>) -> Result<SoundEventHolder> {
    let key = Identifier::vanilla_static("intentionally_empty");
    context
        .registry()
        .sound_events
        .by_key(&key)
        .map(SoundEventHolder::Registry)
        .ok_or_else(|| Error::other("missing minecraft:intentionally_empty sound event"))
}

fn is_empty_sound(sound: &SoundEventHolder) -> bool {
    matches!(sound, SoundEventHolder::Registry(sound) if sound.key == Identifier::vanilla_static("intentionally_empty"))
}

const fn particle_id(value: TransformParticle) -> i32 {
    value as i32
}
const fn drop_strategy_id(value: DropStrategy) -> i32 {
    match value {
        DropStrategy::ClickedFace => 0,
        DropStrategy::FromMiddle => 1,
    }
}
const fn transform_type_id(value: TransformType) -> i32 {
    value as i32
}
const fn direction_id(value: Direction) -> i32 {
    match value {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}
const fn direction_name(value: Direction) -> &'static str {
    match value {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}
const fn particle_name(value: TransformParticle) -> &'static str {
    match value {
        TransformParticle::None => "none",
        TransformParticle::Scrape => "scrape",
        TransformParticle::WaxOn => "wax_on",
        TransformParticle::WaxOff => "wax_off",
    }
}
const fn drop_strategy_name(value: DropStrategy) -> &'static str {
    match value {
        DropStrategy::ClickedFace => "clicked_face",
        DropStrategy::FromMiddle => "from_middle",
    }
}
const fn transform_type_name(value: TransformType) -> &'static str {
    match value {
        TransformType::SingleBlock => "single_block",
        TransformType::CopperChest => "copper_chest",
    }
}
const fn particle_from_id(value: i32) -> TransformParticle {
    match value {
        1 => TransformParticle::Scrape,
        2 => TransformParticle::WaxOn,
        3 => TransformParticle::WaxOff,
        _ => TransformParticle::None,
    }
}
const fn drop_strategy_from_id(value: i32) -> DropStrategy {
    match value {
        1 => DropStrategy::FromMiddle,
        _ => DropStrategy::ClickedFace,
    }
}
const fn transform_type_from_id(value: i32) -> TransformType {
    match value {
        1 => TransformType::CopperChest,
        _ => TransformType::SingleBlock,
    }
}
const fn direction_from_id(value: i32) -> Direction {
    // `Direction.BY_ID` remainder
    match (value % 6).unsigned_abs() {
        0 => Direction::Down,
        1 => Direction::Up,
        2 => Direction::North,
        3 => Direction::South,
        4 => Direction::West,
        _ => Direction::East,
    }
}
fn direction_from_name(value: &str) -> Result<Direction> {
    match value {
        "down" => Ok(Direction::Down),
        "up" => Ok(Direction::Up),
        "north" => Ok(Direction::North),
        "south" => Ok(Direction::South),
        "west" => Ok(Direction::West),
        "east" => Ok(Direction::East),
        _ => Err(Error::other(format!(
            "invalid block transformer direction {value}"
        ))),
    }
}
fn particle_from_name(value: &str) -> Result<TransformParticle> {
    match value {
        "none" => Ok(TransformParticle::None),
        "scrape" => Ok(TransformParticle::Scrape),
        "wax_on" => Ok(TransformParticle::WaxOn),
        "wax_off" => Ok(TransformParticle::WaxOff),
        _ => Err(Error::other(format!(
            "invalid block transformer particle {value}"
        ))),
    }
}
fn drop_strategy_from_name(value: &str) -> Result<DropStrategy> {
    match value {
        "clicked_face" => Ok(DropStrategy::ClickedFace),
        "from_middle" => Ok(DropStrategy::FromMiddle),
        _ => Err(Error::other(format!(
            "invalid block transformer drop strategy {value}"
        ))),
    }
}
fn transform_type_from_name(value: &str) -> Result<TransformType> {
    match value {
        "single_block" => Ok(TransformType::SingleBlock),
        "copper_chest" => Ok(TransformType::CopperChest),
        _ => Err(Error::other(format!(
            "invalid block transformer type {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, io::Cursor, str::FromStr};

    use serde::Deserialize;
    use simdnbt::owned::{NbtList, NbtTag};
    use steel_utils::{
        Identifier,
        snbt::{parse_vanilla_snbt, to_vanilla_snbt},
    };

    use super::{
        BlockTransformer, TransformHolderSet, TransformNoiseParameters, TransformPredicate,
        TransformStateProvider, decode_small_offset, direction_from_id, holder_set_nbt, nbt_reader,
        nbt_writer, network_reader, network_writer, noise_parameters_nbt, predicate_nbt,
        provider_nbt, validate_weight_total,
    };
    use crate::{
        REGISTRY, RegistryExt,
        data_components::vanilla_components::BLOCK_TRANSFORMER,
        data_components::{
            Component, ComponentData, DataComponentCodecContext, DataComponentPatch,
        },
        test_support::init_test_registry,
    };

    #[derive(Deserialize)]
    struct AllVariantsFixture {
        hash: i32,
        snbt: String,
    }

    #[derive(Deserialize)]
    struct ComponentHashFixtures {
        block_transformer: BTreeMap<String, i32>,
        block_transformer_all_variants: AllVariantsFixture,
    }

    fn context() -> DataComponentCodecContext<'static> {
        init_test_registry();
        DataComponentCodecContext::new(&REGISTRY)
    }

    fn fixtures() -> ComponentHashFixtures {
        serde_json::from_str(include_str!("../../../test_assets/component_hashes.json"))
            .expect("component hash fixture must be valid JSON")
    }

    fn all_variants_transformer(
        context: &DataComponentCodecContext<'_>,
    ) -> (BlockTransformer, simdnbt::owned::NbtTag, i32) {
        let fixture = fixtures().block_transformer_all_variants;
        let tag = parse_vanilla_snbt(&fixture.snbt)
            .expect("Vanilla all-variants transformer SNBT fixture must parse");
        let mut bytes = Vec::new();
        tag.write(&mut bytes);
        let borrowed = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
            .expect("all-variants transformer fixture must decode as binary NBT");
        let data = nbt_reader(context, borrowed.as_tag())
            .expect("all-variants transformer fixture must decode through the component codec");
        let transformer = BlockTransformer::from_data(data)
            .expect("fixture must decode to a block transformer component");
        (transformer, tag, fixture.hash)
    }

    #[test]
    fn component_hash_matches_snapshot_2_vanilla_transformer_fixtures() {
        let context = context();
        let fixtures = fixtures();

        for (item_key, expected_hash) in fixtures.block_transformer {
            let item_key = Identifier::from_str(&item_key).expect("fixture item key must be valid");
            let item = context
                .registry()
                .items
                .by_key(&item_key)
                .expect("fixture item must be registered");
            let transformer = item
                .components
                .get_ref(BLOCK_TRANSFORMER)
                .expect("fixture item must have block_transformer");

            assert_eq!(
                ComponentData::BlockTransformer(transformer.clone()).compute_hash(),
                expected_hash,
                "block_transformer hash mismatch for {item_key}"
            );
        }
    }

    #[test]
    fn all_vanilla_provider_and_predicate_variants_round_trip_and_hash() {
        let context = context();
        let (transformer, fixture_tag, expected_hash) = all_variants_transformer(&context);
        let data = ComponentData::BlockTransformer(transformer.clone());

        assert_eq!(
            to_vanilla_snbt(&nbt_writer(&context, &data)),
            to_vanilla_snbt(&fixture_tag),
            "persistent codec must preserve every vanilla provider and predicate variant"
        );
        assert_eq!(data.compute_hash(), expected_hash);

        let mut bytes = Vec::new();
        network_writer(&context, &data, &mut bytes)
            .expect("all-variants transformer network encoding must succeed");
        let decoded = network_reader(&context, &mut Cursor::new(bytes.as_slice()))
            .expect("all-variants transformer network decoding must succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn data_component_patch_round_trips_registry_aware_transformer_codecs() {
        let context = context();
        let (transformer, _, _) = all_variants_transformer(&context);
        let mut patch = DataComponentPatch::new();
        patch.set(BLOCK_TRANSFORMER, transformer);

        let mut network = Vec::new();
        patch
            .write_with_context(&context, &mut network)
            .expect("registry-aware block_transformer patch must encode");
        let network_patch =
            DataComponentPatch::read_with_context(&context, &mut Cursor::new(network.as_slice()))
                .expect("registry-aware block_transformer patch must decode");
        assert_eq!(network_patch, patch);

        let persistent = patch.to_nbt_tag_with_context(&context);
        let mut persistent_bytes = Vec::new();
        persistent.write(&mut persistent_bytes);
        let borrowed = simdnbt::borrow::read_tag(&mut Cursor::new(persistent_bytes.as_slice()))
            .expect("persistent block_transformer patch must decode as NBT");
        let persistent_patch =
            DataComponentPatch::from_nbt_tag_with_context(&context, borrowed.as_tag())
                .expect("registry-aware block_transformer persistent patch must decode");
        assert_eq!(persistent_patch, patch);
    }

    #[test]
    fn state_testing_offsets_match_vanillas_exclusive_bound() {
        let mut predicate = simdnbt::owned::NbtCompound::new();
        predicate.insert("offset", simdnbt::owned::NbtTag::IntArray(vec![15, -15, 0]));
        assert_eq!(
            decode_small_offset(&predicate, "test predicate")
                .expect("offset within Vanilla's bound must decode"),
            (15, -15, 0)
        );

        let mut out_of_range = simdnbt::owned::NbtCompound::new();
        out_of_range.insert("offset", simdnbt::owned::NbtTag::IntArray(vec![16, 0, 0]));
        assert!(decode_small_offset(&out_of_range, "test predicate").is_err());
    }

    #[test]
    fn weighted_provider_totals_match_vanillas_non_empty_weight_limit() {
        assert!(validate_weight_total([0, 0], "test provider").is_err());
        assert!(validate_weight_total([0, 1], "test provider").is_ok());
        assert!(validate_weight_total([i32::MAX, 1], "test provider").is_err());
    }

    #[test]
    fn direction_stream_codec_uses_vanillas_absolute_remainder_for_negative_ids() {
        use steel_utils::Direction;

        assert_eq!(direction_from_id(-1), Direction::Up);
        assert_eq!(direction_from_id(-2), Direction::North);
        assert_eq!(direction_from_id(-5), Direction::East);
        assert_eq!(direction_from_id(-6), Direction::Down);
    }

    #[test]
    fn empty_nbt_lists_use_vanillas_end_element_type() {
        let context = context();

        let NbtTag::Compound(provider) = provider_nbt(
            &context,
            &TransformStateProvider::RuleBased {
                fallback: None,
                rules: Vec::new(),
            },
        ) else {
            panic!("rule-based provider must encode as a compound");
        };
        assert!(matches!(
            provider.get("rules"),
            Some(NbtTag::List(NbtList::Empty))
        ));

        for predicate in [
            TransformPredicate::Any(Vec::new()),
            TransformPredicate::All(Vec::new()),
        ] {
            let NbtTag::Compound(predicate) = predicate_nbt(&predicate) else {
                panic!("combined predicate must encode as a compound");
            };
            assert!(matches!(
                predicate.get("predicates"),
                Some(NbtTag::List(NbtList::Empty))
            ));
        }

        assert!(matches!(
            holder_set_nbt(&TransformHolderSet::Entries(Vec::new())),
            NbtTag::List(NbtList::Empty)
        ));

        let noise = noise_parameters_nbt(&TransformNoiseParameters {
            first_octave: 0,
            amplitudes: Vec::new(),
        });
        assert!(matches!(
            noise.get("amplitudes"),
            Some(NbtTag::List(NbtList::Empty))
        ));
    }
}
