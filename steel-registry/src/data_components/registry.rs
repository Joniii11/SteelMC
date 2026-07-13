//! Data component registry and storage types.
//!
//! This module provides:
//! - [`DataComponentRegistry`] - Registry of all component types with their serialization functions
//! - [`DataComponentMap`] - Storage for component values on items/entities
//! - [`DataComponentPatch`] - Diff representation for network/storage
//! - [`DataComponentType`] - Type-safe handle for accessing components

use rustc_hash::FxHashMap;
use simdnbt::{
    FromNbtTag, ToNbtTag,
    borrow::NbtTag as BorrowedNbtTag,
    owned::{NbtCompound, NbtTag as OwnedNbtTag},
};
use std::{
    fmt::Debug,
    io::{Cursor, Error, ErrorKind, Result, Write},
    marker::PhantomData,
};

use steel_utils::{
    Identifier,
    codec::VarInt,
    serial::{ReadFrom, WriteTo},
};

use super::component_data::{Component, ComponentData, ComponentDataDiscriminant};
use super::components::{ItemAttributeModifiers, ItemEnchantments};
use super::vanilla_components::{
    ATTRIBUTE_MODIFIERS, BREAK_SOUND, ENCHANTMENTS, LORE, MAX_STACK_SIZE, RARITY, REPAIR_COST,
    TOOLTIP_DISPLAY,
};

/// A typed handle for a data component.
///
/// This provides compile-time type safety when getting/setting components.
/// The actual storage uses [`ComponentData`] for ABI stability.
///
/// # Example
/// ```ignore
/// pub const DAMAGE: DataComponentType<Damage> =
///     DataComponentType::new(Identifier::vanilla_static("damage"));
///
/// // Type-safe access
/// let damage: Option<Damage> = components.get(DAMAGE);
/// components.set(DAMAGE, Damage(10));
/// ```
pub struct DataComponentType<T> {
    pub key: Identifier,
    _phantom: PhantomData<T>,
}

impl<T> Clone for DataComponentType<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> DataComponentType<T> {
    #[must_use]
    pub const fn new(key: Identifier) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }
}

/// Reader function for deserializing a component from network format.
pub type NetworkReader =
    for<'a> fn(&DataComponentCodecContext<'a>, &mut Cursor<&[u8]>) -> Result<ComponentData>;

/// Writer function for serializing a component to network format.
pub type NetworkWriter =
    for<'a> fn(&DataComponentCodecContext<'a>, &ComponentData, &mut Vec<u8>) -> Result<()>;

/// Reader function for deserializing a component from NBT format.
pub type NbtReader =
    for<'a> fn(&DataComponentCodecContext<'a>, BorrowedNbtTag) -> Option<ComponentData>;

/// Writer function for serializing a component to NBT format.
pub type NbtWriter = for<'a> fn(&DataComponentCodecContext<'a>, &ComponentData) -> OwnedNbtTag;

fn component_network_reader<T>(
    _context: &DataComponentCodecContext<'_>,
    cursor: &mut Cursor<&[u8]>,
) -> Result<ComponentData>
where
    T: Component + ReadFrom,
{
    Ok(T::read(cursor)?.into_data())
}

fn component_network_writer<T>(
    _context: &DataComponentCodecContext<'_>,
    data: &ComponentData,
    writer: &mut Vec<u8>,
) -> Result<()>
where
    T: Component + WriteTo,
{
    let Some(value) = T::from_data_ref(data) else {
        return Err(Error::other("Component type mismatch"));
    };
    value.write(writer)
}

fn component_nbt_reader<T>(
    _context: &DataComponentCodecContext<'_>,
    tag: BorrowedNbtTag,
) -> Option<ComponentData>
where
    T: Component + FromNbtTag,
{
    Some(T::from_nbt_tag(tag)?.into_data())
}

fn component_nbt_writer<T>(
    _context: &DataComponentCodecContext<'_>,
    data: &ComponentData,
) -> OwnedNbtTag
where
    T: Component + ToNbtTag,
{
    let Some(value) = T::from_data_ref(data) else {
        panic!("Component type mismatch");
    };
    value.clone().to_nbt_tag()
}

/// Component codec registry
pub struct DataComponentCodecContext<'a> {
    registry: &'a crate::Registry,
}

impl<'a> DataComponentCodecContext<'a> {
    #[must_use]
    pub const fn new(registry: &'a crate::Registry) -> Self {
        Self { registry }
    }

    #[must_use]
    pub const fn registry(&self) -> &'a crate::Registry {
        self.registry
    }
}

/// Component persistence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPersistence {
    Persistent,
    Transient,
}

impl ComponentPersistence {
    #[must_use]
    pub const fn is_persistent(self) -> bool {
        matches!(self, Self::Persistent)
    }
}

/// Component codecs
pub struct ComponentCodecs {
    persistence: ComponentPersistence,
    network_reader: NetworkReader,
    network_writer: NetworkWriter,
    nbt_reader: Option<NbtReader>,
    nbt_writer: Option<NbtWriter>,
}

impl ComponentCodecs {
    /// Persistent codecs
    #[must_use]
    pub const fn persistent(
        network_reader: NetworkReader,
        network_writer: NetworkWriter,
        nbt_reader: NbtReader,
        nbt_writer: NbtWriter,
    ) -> Self {
        Self {
            persistence: ComponentPersistence::Persistent,
            network_reader,
            network_writer,
            nbt_reader: Some(nbt_reader),
            nbt_writer: Some(nbt_writer),
        }
    }

    /// Transient codecs
    #[must_use]
    pub const fn transient(network_reader: NetworkReader, network_writer: NetworkWriter) -> Self {
        Self {
            persistence: ComponentPersistence::Transient,
            network_reader,
            network_writer,
            nbt_reader: None,
            nbt_writer: None,
        }
    }
}

/// Metadata for a registered component type.
///
/// Contains the component's key and all serialization functions needed
/// to read/write the component for network and persistent storage.
pub struct ComponentEntry {
    /// The component's identifier (e.g., "minecraft:damage")
    pub key: Identifier,
    /// Expected discriminant for this component type
    pub expected_discriminant: ComponentDataDiscriminant,
    /// Persistent codec
    pub persistence: ComponentPersistence,
    /// Network protocol reader
    pub network_reader: NetworkReader,
    /// Network protocol writer
    pub network_writer: NetworkWriter,
    /// Persistent NBT reader
    pub nbt_reader: Option<NbtReader>,
    /// Persistent NBT writer
    pub nbt_writer: Option<NbtWriter>,
}

/// `data_component_predicate_type` registry
pub struct DataComponentPredicateTypeRegistry {
    entries: Vec<Identifier>,
    by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl Default for DataComponentPredicateTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DataComponentPredicateTypeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    /// Registers a predicate type
    pub fn register(&mut self, key: Identifier) -> usize {
        assert!(
            self.allows_registering,
            "Cannot register data component predicate types after the registry has been frozen"
        );
        assert!(
            !self.by_key.contains_key(&key),
            "Data component predicate type already registered: {key}"
        );

        let id = self.entries.len();
        self.by_key.insert(key.clone(), id);
        self.entries.push(key);
        id
    }

    /// Predicate type ID
    #[must_use]
    pub fn id_from_key(&self, key: &Identifier) -> Option<usize> {
        self.by_key.get(key).copied()
    }

    /// Predicate type key
    #[must_use]
    pub fn get_key_by_id(&self, id: usize) -> Option<&Identifier> {
        self.entries.get(id)
    }

    /// Freezes registry
    pub const fn freeze(&mut self) {
        self.allows_registering = false;
    }
}

impl ComponentEntry {
    /// Creates a new component entry with all serialization functions.
    #[must_use]
    pub fn new(
        key: Identifier,
        expected_discriminant: ComponentDataDiscriminant,
        codecs: ComponentCodecs,
    ) -> Self {
        Self {
            key,
            expected_discriminant,
            persistence: codecs.persistence,
            network_reader: codecs.network_reader,
            network_writer: codecs.network_writer,
            nbt_reader: codecs.nbt_reader,
            nbt_writer: codecs.nbt_writer,
        }
    }

    /// Validates that a `ComponentData` value matches the expected type for this component.
    ///
    /// Returns `true` if the data is valid for this component type, `false` otherwise.
    /// This prevents plugins from setting wrong types on vanilla components.
    #[must_use]
    pub fn validates(&self, data: &ComponentData) -> bool {
        data.discriminant() == self.expected_discriminant
    }

    #[must_use]
    pub const fn is_persistent(&self) -> bool {
        self.persistence.is_persistent()
    }
}

pub type ComponentEntryRef = &'static ComponentEntry;

/// Registry of all data component types.
///
/// Stores metadata about each component type including how to serialize/deserialize
/// them for network and persistent storage.
pub struct DataComponentRegistry {
    /// Component entries indexed by network ID
    entries: Vec<ComponentEntryRef>,
    /// Map from component key to network ID
    by_key: FxHashMap<Identifier, usize>,
    /// Whether registration is still allowed
    allows_registering: bool,
}

impl Default for DataComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DataComponentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    /// Registers a vanilla component type.
    ///
    /// The component type `T` must implement the necessary serialization traits.
    /// This creates the appropriate reader/writer functions automatically.
    pub fn register<T>(
        &mut self,
        component: DataComponentType<T>,
        expected_discriminant: ComponentDataDiscriminant,
    ) where
        T: 'static + Component + WriteTo + ReadFrom + ToNbtTag + FromNbtTag,
    {
        let _ = self.register_dynamic(
            component.key,
            expected_discriminant,
            ComponentCodecs::persistent(
                component_network_reader::<T>,
                component_network_writer::<T>,
                component_nbt_reader::<T>,
                component_nbt_writer::<T>,
            ),
        );
    }

    /// Registers transient component
    pub fn register_transient<T>(
        &mut self,
        component: DataComponentType<T>,
        expected_discriminant: ComponentDataDiscriminant,
    ) where
        T: 'static + Component + WriteTo + ReadFrom,
    {
        let _ = self.register_dynamic(
            component.key,
            expected_discriminant,
            ComponentCodecs::transient(
                component_network_reader::<T>,
                component_network_writer::<T>,
            ),
        );
    }

    /// Registers a component with custom network reader/writer functions.
    ///
    /// Use this when the default `WriteTo`/`ReadFrom` implementations don't match
    /// the network encoding (e.g., VarInt-encoded i32 components).
    /// NBT serialization still uses the type's `ToNbtTag`/`FromNbtTag` impls.
    pub fn register_custom_network<T>(
        &mut self,
        component: DataComponentType<T>,
        expected_discriminant: ComponentDataDiscriminant,
        network_reader: NetworkReader,
        network_writer: NetworkWriter,
    ) where
        T: 'static + Component + ToNbtTag + FromNbtTag,
    {
        let _ = self.register_dynamic(
            component.key,
            expected_discriminant,
            ComponentCodecs::persistent(
                network_reader,
                network_writer,
                component_nbt_reader::<T>,
                component_nbt_writer::<T>,
            ),
        );
    }

    /// Registers transient stream component
    pub fn register_custom_network_transient<T>(
        &mut self,
        component: DataComponentType<T>,
        expected_discriminant: ComponentDataDiscriminant,
        network_reader: NetworkReader,
        network_writer: NetworkWriter,
    ) where
        T: Component,
    {
        let _ = self.register_dynamic(
            component.key,
            expected_discriminant,
            ComponentCodecs::transient(network_reader, network_writer),
        );
    }

    /// Registers a dynamic/plugin component type.
    ///
    /// Plugin components use the `ComponentData::Other` variant and handle
    /// their own serialization. The provided functions read/write raw bytes.
    pub fn register_dynamic(
        &mut self,
        key: Identifier,
        expected_discriminant: ComponentDataDiscriminant,
        codecs: ComponentCodecs,
    ) -> usize {
        assert!(
            self.allows_registering,
            "Cannot register data components after the registry has been frozen"
        );
        assert!(
            !self.by_key.contains_key(&key),
            "Data component already registered: {key}"
        );
        let entry = Box::leak(Box::new(ComponentEntry::new(
            key.clone(),
            expected_discriminant,
            codecs,
        )));

        let id = self.entries.len();
        self.by_key.insert(key, id);
        self.entries.push(entry);
        id
    }

    /// Registers registry-aware component
    pub fn register_custom(
        &mut self,
        key: Identifier,
        expected_discriminant: ComponentDataDiscriminant,
        network_reader: NetworkReader,
        network_writer: NetworkWriter,
        nbt_reader: NbtReader,
        nbt_writer: NbtWriter,
    ) {
        let _ = self.register_dynamic(
            key,
            expected_discriminant,
            ComponentCodecs::persistent(network_reader, network_writer, nbt_reader, nbt_writer),
        );
    }

    /// Gets the network ID for a component type.
    #[must_use]
    pub fn get_id<T>(&self, component: DataComponentType<T>) -> Option<usize> {
        self.by_key.get(&component.key).copied()
    }

    /// Gets the component key by network ID.
    #[must_use]
    pub fn get_key_by_id(&self, id: usize) -> Option<&Identifier> {
        self.entries.get(id).map(|e| &e.key)
    }
}

crate::impl_registry!(
    DataComponentRegistry,
    ComponentEntry,
    entries,
    by_key,
    data_components
);

/// Storage for component values.
///
/// Maps component keys to their values. Used on items to store their data components.
#[derive(Debug, Clone)]
pub struct DataComponentMap {
    map: FxHashMap<Identifier, ComponentData>,
}

impl Default for DataComponentMap {
    fn default() -> Self {
        Self::new()
    }
}

impl DataComponentMap {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }

    /// Creates a map with common item components pre-populated.
    #[must_use]
    pub fn common_item_components() -> Self {
        let mut map = FxHashMap::default();
        map.insert(MAX_STACK_SIZE.key.clone(), ComponentData::I32(64));
        map.insert(LORE.key.clone(), ComponentData::Todo);
        map.insert(
            ENCHANTMENTS.key.clone(),
            ComponentData::Enchantments(ItemEnchantments::empty()),
        );
        map.insert(REPAIR_COST.key.clone(), ComponentData::I32(0));
        map.insert(
            ATTRIBUTE_MODIFIERS.key.clone(),
            ComponentData::AttributeModifiers(ItemAttributeModifiers::empty()),
        );
        map.insert(RARITY.key.clone(), ComponentData::Todo);
        map.insert(BREAK_SOUND.key.clone(), ComponentData::Todo);
        map.insert(TOOLTIP_DISPLAY.key.clone(), ComponentData::Todo);
        Self { map }
    }

    /// Sets a component value (builder pattern).
    #[must_use]
    pub fn builder_set<T: Component>(
        mut self,
        component: DataComponentType<T>,
        value: Option<T>,
    ) -> Self {
        self.set(component, value);
        self
    }

    /// Sets a component value, or removes it if `None`.
    pub fn set<T: Component>(&mut self, component: DataComponentType<T>, value: Option<T>) {
        if let Some(v) = value {
            self.map.insert(component.key.clone(), v.into_data());
        } else {
            self.map.remove(&component.key);
        }
    }

    /// Gets a component value by type.
    #[must_use]
    pub fn get<T: Component>(&self, component: DataComponentType<T>) -> Option<T> {
        let data = self.map.get(&component.key)?;
        T::from_data(data.clone())
    }

    /// Gets a reference to a component value.
    #[must_use]
    pub fn get_ref<T: Component>(&self, component: DataComponentType<T>) -> Option<&T> {
        let data = self.map.get(&component.key)?;
        T::from_data_ref(data)
    }

    /// Checks if a component is present.
    #[must_use]
    pub fn has<T>(&self, component: DataComponentType<T>) -> bool {
        self.map.contains_key(&component.key)
    }

    /// Returns the number of components.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterates over component keys.
    pub fn keys(&self) -> impl Iterator<Item = &Identifier> {
        self.map.keys()
    }

    /// Gets raw component data by key (for plugin use).
    #[must_use]
    pub fn get_raw(&self, key: &Identifier) -> Option<&ComponentData> {
        self.map.get(key)
    }

    /// Sets raw component data (for plugin use).
    ///
    /// Returns `true` if the data was set successfully, `false` if the data type
    /// doesn't match the registered component type (validation failed).
    ///
    /// This prevents plugins from setting invalid types on vanilla components.
    pub fn set_raw(&mut self, key: Identifier, data: ComponentData) -> bool {
        use crate::{REGISTRY, RegistryExt};

        // Validate against registry if this component is registered
        if let Some(entry) = REGISTRY.data_components.by_key(&key)
            && !entry.validates(&data)
        {
            return false;
        }

        self.map.insert(key, data);
        true
    }

    /// Removes a component by key.
    pub fn remove(&mut self, key: &Identifier) -> Option<ComponentData> {
        self.map.remove(key)
    }
}

/// Entry in a component patch.
#[derive(Debug, Clone)]
#[expect(
    clippy::large_enum_variant,
    reason = "component patches keep set values inline to avoid changing shared item component storage semantics"
)]
pub enum ComponentPatchEntry {
    /// Component is set to this value
    Set(ComponentData),
    /// Component is explicitly removed
    Removed,
}

impl PartialEq for ComponentPatchEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Removed, Self::Removed) => true,
            (Self::Set(a), Self::Set(b)) => a == b,
            _ => false,
        }
    }
}

/// A patch representing modifications to a [`DataComponentMap`].
///
/// Stores differences from a prototype:
/// - Components that are added or overridden (`Set`)
/// - Components that are explicitly removed (`Removed`)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DataComponentPatch {
    entries: FxHashMap<Identifier, ComponentPatchEntry>,
}

impl DataComponentPatch {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Sets a component value in the patch.
    pub fn set<T: Component>(&mut self, component: DataComponentType<T>, value: T) {
        self.entries.insert(
            component.key.clone(),
            ComponentPatchEntry::Set(value.into_data()),
        );
    }

    /// Sets raw component data (for plugin use).
    ///
    /// Returns `true` if the data was set successfully, `false` if the data type
    /// doesn't match the registered component type (validation failed).
    ///
    /// This prevents plugins from setting invalid types on vanilla components.
    pub fn set_raw(&mut self, key: Identifier, data: ComponentData) -> bool {
        use crate::{REGISTRY, RegistryExt};

        // Validate against registry if this component is registered
        if let Some(entry) = REGISTRY.data_components.by_key(&key)
            && !entry.validates(&data)
        {
            return false;
        }

        self.entries.insert(key, ComponentPatchEntry::Set(data));
        true
    }

    /// Marks a component as removed.
    pub fn remove<T>(&mut self, component: DataComponentType<T>) {
        self.entries
            .insert(component.key.clone(), ComponentPatchEntry::Removed);
    }

    /// Clears any patch entry for a component.
    pub fn clear<T>(&mut self, component: DataComponentType<T>) {
        self.entries.remove(&component.key);
    }

    /// Gets the patch entry for a key.
    #[must_use]
    pub fn get_entry(&self, key: &Identifier) -> Option<&ComponentPatchEntry> {
        self.entries.get(key)
    }

    /// Checks if a component is marked as removed.
    #[must_use]
    pub fn is_removed(&self, key: &Identifier) -> bool {
        matches!(self.entries.get(key), Some(ComponentPatchEntry::Removed))
    }

    /// Counts set entries.
    #[must_use]
    pub fn count_set(&self) -> usize {
        self.entries
            .values()
            .filter(|e| matches!(e, ComponentPatchEntry::Set(_)))
            .count()
    }

    /// Counts removed entries.
    #[must_use]
    pub fn count_removed(&self) -> usize {
        self.entries
            .values()
            .filter(|e| matches!(e, ComponentPatchEntry::Removed))
            .count()
    }

    /// Iterates over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &ComponentPatchEntry)> {
        self.entries.iter()
    }

    /// Iterates over removed component keys.
    pub fn iter_removed(&self) -> impl Iterator<Item = &Identifier> {
        self.entries.iter().filter_map(|(k, v)| {
            if matches!(v, ComponentPatchEntry::Removed) {
                Some(k)
            } else {
                None
            }
        })
    }

    /// Persistent patch emptiness
    #[must_use]
    pub fn is_persistently_empty(&self, context: &DataComponentCodecContext<'_>) -> bool {
        use crate::RegistryExt;

        !self.entries.keys().any(|key| {
            context
                .registry()
                .data_components
                .by_key(key)
                .is_some_and(ComponentEntry::is_persistent)
        })
    }

    /// Converts this component patch to NBT without consuming it.
    #[must_use]
    pub fn to_nbt_tag_ref(&self) -> OwnedNbtTag {
        self.to_nbt_tag_with_context(&DataComponentCodecContext::new(&crate::REGISTRY))
    }

    /// Persistent patch NBT
    #[must_use]
    pub fn to_nbt_tag_with_context(&self, context: &DataComponentCodecContext<'_>) -> OwnedNbtTag {
        use crate::RegistryExt;

        let mut compound = NbtCompound::new();

        for (key, patch_entry) in &self.entries {
            let Some(entry) = context.registry().data_components.by_key(key) else {
                continue;
            };
            if !entry.is_persistent() {
                continue;
            }

            match patch_entry {
                ComponentPatchEntry::Set(data) => {
                    let Some(nbt_writer) = entry.nbt_writer else {
                        continue;
                    };
                    let nbt = nbt_writer(context, data);
                    compound.insert(key.to_string(), nbt);
                }
                ComponentPatchEntry::Removed => {
                    compound.insert(format!("!{key}"), NbtCompound::new());
                }
            }
        }

        OwnedNbtTag::Compound(compound)
    }
}

impl WriteTo for DataComponentPatch {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        self.write_with_context(&DataComponentCodecContext::new(&crate::REGISTRY), writer)
    }
}

impl DataComponentPatch {
    /// Contextual patch write
    pub fn write_with_context(
        &self,
        context: &DataComponentCodecContext<'_>,
        writer: &mut impl Write,
    ) -> Result<()> {
        use crate::RegistryExt;

        let mut added: Vec<(&Identifier, &ComponentData)> = Vec::new();
        let mut removed: Vec<&Identifier> = Vec::new();

        for (key, entry) in &self.entries {
            match entry {
                ComponentPatchEntry::Set(data) => added.push((key, data)),
                ComponentPatchEntry::Removed => removed.push(key),
            }
        }

        let added_count = i32::try_from(added.len())
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "too many added components"))?;
        let removed_count = i32::try_from(removed.len())
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "too many removed components"))?;
        VarInt(added_count).write(writer)?;
        VarInt(removed_count).write(writer)?;

        // Write added components
        for (key, data) in added {
            let id = context
                .registry()
                .data_components
                .id_from_key(key)
                .ok_or_else(|| std::io::Error::other(format!("Unknown component key: {key:?}")))?;

            let entry = context
                .registry()
                .data_components
                .by_id(id)
                .ok_or_else(|| std::io::Error::other(format!("No entry for component id: {id}")))?;

            VarInt(id as i32).write(writer)?;

            let mut buf = Vec::new();
            (entry.network_writer)(context, data, &mut buf)?;
            writer.write_all(&buf)?;
        }

        // Write removed component IDs
        for key in removed {
            let id = context
                .registry()
                .data_components
                .id_from_key(key)
                .ok_or_else(|| std::io::Error::other(format!("Unknown component key: {key:?}")))?;
            VarInt(id as i32).write(writer)?;
        }

        Ok(())
    }
}

impl ReadFrom for DataComponentPatch {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        Self::read_with_context(&DataComponentCodecContext::new(&crate::REGISTRY), data)
    }
}

impl DataComponentPatch {
    fn read_non_negative_varint(data: &mut Cursor<&[u8]>, field: &str) -> Result<usize> {
        let value = VarInt::read(data)?.0;
        usize::try_from(value)
            .map_err(|_| Error::new(ErrorKind::InvalidData, format!("negative {field}: {value}")))
    }

    fn remaining_bytes(data: &Cursor<&[u8]>) -> Result<usize> {
        let position = usize::try_from(data.position()).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "component patch cursor position does not fit usize",
            )
        })?;
        data.get_ref().len().checked_sub(position).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "component patch cursor position exceeds its input",
            )
        })
    }

    fn read_counts(
        data: &mut Cursor<&[u8]>,
        minimum_added_entry_bytes: usize,
    ) -> Result<(usize, usize)> {
        let added_count = Self::read_non_negative_varint(data, "added component count")?;
        let removed_count = Self::read_non_negative_varint(data, "removed component count")?;
        let minimum_bytes = added_count
            .checked_mul(minimum_added_entry_bytes)
            .and_then(|bytes| bytes.checked_add(removed_count))
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "component patch entry count overflows its minimum encoded size",
                )
            })?;

        if minimum_bytes > Self::remaining_bytes(data)? {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "component patch declares more entries than remaining input can contain",
            ));
        }

        Ok((added_count, removed_count))
    }

    /// Contextual patch read
    pub fn read_with_context(
        context: &DataComponentCodecContext<'_>,
        data: &mut Cursor<&[u8]>,
    ) -> Result<Self> {
        use crate::RegistryExt;

        let (added_count, removed_count) = Self::read_counts(data, 1)?;

        log::info!("Reading DataComponentPatch: added={added_count}, removed={removed_count}");

        let mut patch = Self::new();

        // Read added components
        for i in 0..added_count {
            let pos_before = data.position();
            let type_id = Self::read_non_negative_varint(data, "data component type ID")?;

            let key = context
                .registry()
                .data_components
                .get_key_by_id(type_id)
                .ok_or_else(|| {
                    std::io::Error::other(format!("Unknown component type ID: {type_id}"))
                })?
                .clone();

            log::info!("  [{i}] Reading component {key} (id={type_id}) at pos {pos_before}");

            let entry = context
                .registry()
                .data_components
                .by_id(type_id)
                .ok_or_else(|| std::io::Error::other(format!("No entry for component: {key}")))?;

            let component_data = (entry.network_reader)(context, data).map_err(|e| {
                log::error!("    Failed to read component {key}: {e}");
                e
            })?;

            let pos_after = data.position();
            log::info!("    Read {} bytes for {key}", pos_after - pos_before);

            patch
                .entries
                .insert(key, ComponentPatchEntry::Set(component_data));
        }

        // Read removed component IDs
        for _ in 0..removed_count {
            let type_id = Self::read_non_negative_varint(data, "data component type ID")?;

            let key = context
                .registry()
                .data_components
                .get_key_by_id(type_id)
                .ok_or_else(|| {
                    std::io::Error::other(format!("Unknown component type ID: {type_id}"))
                })?
                .clone();

            patch.entries.insert(key, ComponentPatchEntry::Removed);
        }

        Ok(patch)
    }
}

impl DataComponentPatch {
    fn read_delimited_payload<'a>(
        data: &mut Cursor<&'a [u8]>,
        byte_len: usize,
    ) -> Result<Cursor<&'a [u8]>> {
        let input = *data.get_ref();
        let start = usize::try_from(data.position()).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "component payload cursor position does not fit usize",
            )
        })?;
        let end = start.checked_add(byte_len).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "component payload length overflows input position",
            )
        })?;
        let payload = input.get(start..end).ok_or_else(|| {
            Error::new(
                ErrorKind::UnexpectedEof,
                "component payload exceeds remaining input",
            )
        })?;
        let end = u64::try_from(end).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "component payload end does not fit cursor position",
            )
        })?;
        data.set_position(end);
        Ok(Cursor::new(payload))
    }

    /// Reads a patch where each component value is prefixed with a `VarInt` byte length.
    ///
    /// Vanilla uses this for untrusted client packets (e.g., creative mode slot)
    /// via `DataComponentPatch.DELIMITED_STREAM_CODEC`.
    pub fn read_delimited(data: &mut Cursor<&[u8]>) -> Result<Self> {
        Self::read_delimited_with_context(&DataComponentCodecContext::new(&crate::REGISTRY), data)
    }

    /// Contextual delimited patch read
    pub fn read_delimited_with_context(
        context: &DataComponentCodecContext<'_>,
        data: &mut Cursor<&[u8]>,
    ) -> Result<Self> {
        use crate::RegistryExt;

        // Minimum entry size
        let (added_count, removed_count) = Self::read_counts(data, 2)?;

        let mut patch = Self::new();

        for _ in 0..added_count {
            let type_id = Self::read_non_negative_varint(data, "data component type ID")?;
            let byte_len = Self::read_non_negative_varint(data, "data component byte length")?;

            let key = context
                .registry()
                .data_components
                .get_key_by_id(type_id)
                .ok_or_else(|| {
                    std::io::Error::other(format!("Unknown component type ID: {type_id}"))
                })?
                .clone();

            let entry = context
                .registry()
                .data_components
                .by_id(type_id)
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "component ID disappeared from registry",
                    )
                })?;

            let mut payload = Self::read_delimited_payload(data, byte_len)?;
            let component_data = (entry.network_reader)(context, &mut payload)?;
            patch
                .entries
                .insert(key, ComponentPatchEntry::Set(component_data));
        }

        for _ in 0..removed_count {
            let type_id = Self::read_non_negative_varint(data, "data component type ID")?;
            let key = context
                .registry()
                .data_components
                .get_key_by_id(type_id)
                .ok_or_else(|| {
                    std::io::Error::other(format!("Unknown component type ID: {type_id}"))
                })?
                .clone();
            patch.entries.insert(key, ComponentPatchEntry::Removed);
        }

        Ok(patch)
    }
}

impl ToNbtTag for DataComponentPatch {
    fn to_nbt_tag(self) -> OwnedNbtTag {
        self.to_nbt_tag_ref()
    }
}

impl FromNbtTag for DataComponentPatch {
    fn from_nbt_tag(tag: BorrowedNbtTag) -> Option<Self> {
        Self::from_nbt_tag_with_context(&DataComponentCodecContext::new(&crate::REGISTRY), tag)
    }
}

impl DataComponentPatch {
    /// Persistent patch read
    #[must_use]
    pub fn from_nbt_tag_with_context(
        context: &DataComponentCodecContext<'_>,
        tag: BorrowedNbtTag,
    ) -> Option<Self> {
        use crate::RegistryExt;

        let compound = tag.compound()?;
        let mut patch = Self::new();

        for (key, value) in compound.iter() {
            let key_str = key.to_str();

            if let Some(stripped) = key_str.strip_prefix('!') {
                let id = stripped.parse::<Identifier>().ok()?;
                let entry = context.registry().data_components.by_key(&id)?;
                if !entry.is_persistent() {
                    return None;
                }
                patch.entries.insert(id, ComponentPatchEntry::Removed);
            } else {
                let id = key_str.parse::<Identifier>().ok()?;
                let entry = context.registry().data_components.by_key(&id)?;
                if !entry.is_persistent() {
                    return None;
                }
                let nbt_reader = entry.nbt_reader?;
                let component_data = nbt_reader(context, value)?;
                patch
                    .entries
                    .insert(id, ComponentPatchEntry::Set(component_data));
            }
        }

        Some(patch)
    }
}

/// Attempts to extract a typed component from `ComponentData`.
#[must_use]
pub fn component_try_into<T: Component>(
    data: &ComponentData,
    _component: DataComponentType<T>,
) -> Option<&T> {
    T::from_data_ref(data)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use simdnbt::owned::{NbtCompound, NbtTag};
    use steel_utils::{codec::VarInt, serial::WriteTo};

    use super::{DataComponentCodecContext, DataComponentPatch};
    use crate::{
        REGISTRY, RegistryExt,
        data_components::vanilla_components::{
            ADDITIONAL_TRADE_COST, CREATIVE_SLOT_LOCK, MAP_POST_PROCESSING, MAX_STACK_SIZE,
            MapPostProcessing,
        },
        test_support::init_test_registry,
    };

    fn context() -> DataComponentCodecContext<'static> {
        init_test_registry();
        DataComponentCodecContext::new(&REGISTRY)
    }

    fn max_stack_size_id(context: &DataComponentCodecContext<'_>) -> i32 {
        context
            .registry()
            .data_components
            .id_from_key(&MAX_STACK_SIZE.key)
            .and_then(|id| i32::try_from(id).ok())
            .expect("max_stack_size must have a protocol component ID")
    }

    fn varints(values: &[i32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for &value in values {
            VarInt(value)
                .write(&mut bytes)
                .expect("VarInt should write to a Vec");
        }
        bytes
    }

    #[test]
    fn stream_patch_rejects_negative_component_counts() {
        let context = context();
        let bytes = varints(&[-1, 0]);
        let mut cursor = Cursor::new(bytes.as_slice());

        assert!(DataComponentPatch::read_with_context(&context, &mut cursor).is_err());
    }

    #[test]
    fn delimited_patch_rejects_negative_component_payload_length() {
        let context = context();
        let bytes = varints(&[1, 0, max_stack_size_id(&context), -1]);
        let mut cursor = Cursor::new(bytes.as_slice());

        assert!(DataComponentPatch::read_delimited_with_context(&context, &mut cursor).is_err());
    }

    #[test]
    fn delimited_patch_propagates_malformed_component_errors() {
        let context = context();
        let bytes = varints(&[1, 0, max_stack_size_id(&context), 0]);
        let mut cursor = Cursor::new(bytes.as_slice());

        assert!(DataComponentPatch::read_delimited_with_context(&context, &mut cursor).is_err());
    }

    #[test]
    fn delimited_patch_decodes_a_length_bounded_component() {
        let context = context();
        let mut bytes = varints(&[1, 0, max_stack_size_id(&context), 4]);
        42_i32
            .write(&mut bytes)
            .expect("component payload should write to a Vec");
        let mut cursor = Cursor::new(bytes.as_slice());

        let patch = DataComponentPatch::read_delimited_with_context(&context, &mut cursor)
            .expect("length-bounded component should decode");

        assert_eq!(patch.count_set(), 1);
    }

    #[test]
    fn persistent_patch_filters_and_rejects_transient_components() {
        let context = context();
        let mut transient = DataComponentPatch::new();
        transient.set(CREATIVE_SLOT_LOCK, ());
        transient.set(ADDITIONAL_TRADE_COST, 12);
        transient.set(MAP_POST_PROCESSING, MapPostProcessing::Scale);
        assert!(transient.is_persistently_empty(&context));

        let NbtTag::Compound(encoded) = transient.to_nbt_tag_with_context(&context) else {
            panic!("component patch must persist as a compound");
        };
        assert!(encoded.is_empty());

        let mut removed_transient = DataComponentPatch::new();
        removed_transient.remove(CREATIVE_SLOT_LOCK);
        removed_transient.remove(ADDITIONAL_TRADE_COST);
        removed_transient.remove(MAP_POST_PROCESSING);
        let NbtTag::Compound(encoded) = removed_transient.to_nbt_tag_with_context(&context) else {
            panic!("component patch must persist as a compound");
        };
        assert!(encoded.is_empty());

        let mut mixed = DataComponentPatch::new();
        mixed.set(CREATIVE_SLOT_LOCK, ());
        mixed.set(ADDITIONAL_TRADE_COST, 12);
        mixed.set(MAP_POST_PROCESSING, MapPostProcessing::Scale);
        mixed.set(MAX_STACK_SIZE, 16);
        assert!(!mixed.is_persistently_empty(&context));
        let NbtTag::Compound(encoded) = mixed.to_nbt_tag_with_context(&context) else {
            panic!("component patch must persist as a compound");
        };
        assert!(encoded.get("minecraft:creative_slot_lock").is_none());
        assert!(encoded.get("minecraft:additional_trade_cost").is_none());
        assert!(encoded.get("minecraft:map_post_processing").is_none());
        assert!(encoded.get("minecraft:max_stack_size").is_some());

        for key in [
            CREATIVE_SLOT_LOCK.key.to_string(),
            ADDITIONAL_TRADE_COST.key.to_string(),
            MAP_POST_PROCESSING.key.to_string(),
            format!("!{}", CREATIVE_SLOT_LOCK.key),
            format!("!{}", ADDITIONAL_TRADE_COST.key),
            format!("!{}", MAP_POST_PROCESSING.key),
        ] {
            let mut invalid = NbtCompound::new();
            invalid.insert(key.as_str(), NbtCompound::new());
            let tag = NbtTag::Compound(invalid);
            let mut bytes = Vec::new();
            WriteTo::write(&tag, &mut bytes).expect("writing to a vec must succeed");
            let borrowed = simdnbt::borrow::read_tag(&mut Cursor::new(bytes.as_slice()))
                .expect("transient patch fixture must decode as NBT");

            assert!(
                DataComponentPatch::from_nbt_tag_with_context(&context, borrowed.as_tag())
                    .is_none(),
                "transient component key {key} must be rejected"
            );
        }
    }

    #[test]
    fn additional_trade_cost_transient_patch_uses_vanilla_varint_stream_codec() {
        let context = context();
        let mut patch = DataComponentPatch::new();
        patch.set(ADDITIONAL_TRADE_COST, -12);

        let mut encoded = Vec::new();
        patch
            .write_with_context(&context, &mut encoded)
            .expect("additional trade cost patch must encode");

        let mut expected = Vec::new();
        VarInt(1)
            .write(&mut expected)
            .expect("writing to a vec must succeed");
        VarInt(0)
            .write(&mut expected)
            .expect("writing to a vec must succeed");
        VarInt(41)
            .write(&mut expected)
            .expect("writing to a vec must succeed");
        VarInt(-12)
            .write(&mut expected)
            .expect("writing to a vec must succeed");
        assert_eq!(encoded, expected);
        assert_eq!(
            DataComponentPatch::read_with_context(&context, &mut Cursor::new(encoded.as_slice()))
                .expect("additional trade cost patch must decode"),
            patch
        );
    }

    #[test]
    #[should_panic(expected = "Data component already registered")]
    fn registry_rejects_duplicate_component_keys() {
        use steel_utils::Identifier;

        let component = super::DataComponentType::<()>::new(Identifier::vanilla_static("test"));
        let mut registry = crate::data_components::DataComponentRegistry::new();
        registry.register_transient(
            component.clone(),
            crate::data_components::ComponentDataDiscriminant::Empty,
        );
        registry.register_transient(
            component,
            crate::data_components::ComponentDataDiscriminant::Empty,
        );
    }
}
