//! Clientbound update attributes packet - syncs entity attribute values and modifiers.
//!
//! Sent when an entity's attributes change (e.g., movement speed modifier from sprinting).
//! Matches vanilla `ClientboundUpdateAttributesPacket`.

use std::io::{Result, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_UPDATE_ATTRIBUTES;
use steel_utils::{Identifier, codec::VarInt, serial::WriteTo};

/// An attribute modifier operation type.
///
/// Matches vanilla `AttributeModifier.Operation` ordinal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AttributeModifierOperation {
    /// Adds the value directly to the base: `base + value`
    AddValue = 0,
    /// Adds `base * value` to the result (additive with other multiplied_base modifiers)
    AddMultipliedBase = 1,
    /// Multiplies the current result by `(1 + value)` (multiplicative with others)
    AddMultipliedTotal = 2,
}

/// A single attribute modifier sent over the network.
#[derive(Clone, Debug)]
pub struct AttributeModifierData {
    /// The modifier's unique resource location ID (e.g., `minecraft:sprinting`).
    pub id: Identifier,
    /// The modifier amount.
    pub amount: f64,
    /// The modifier operation.
    pub operation: AttributeModifierOperation,
}

impl WriteTo for AttributeModifierData {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        self.id.write(writer)?;
        self.amount.write(writer)?;
        (self.operation as u8).write(writer)?;
        Ok(())
    }
}

/// A single attribute snapshot sent in the packet.
#[derive(Clone, Debug)]
pub struct AttributeSnapshot {
    /// The attribute's registry ID (e.g., 22 for `minecraft:movement_speed`).
    pub attribute_id: i32,
    /// The base value of the attribute (before modifiers).
    pub base_value: f64,
    /// Active modifiers on this attribute.
    pub modifiers: Vec<AttributeModifierData>,
}

impl WriteTo for AttributeSnapshot {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(self.attribute_id).write(writer)?;
        self.base_value.write(writer)?;
        VarInt(self.modifiers.len() as i32).write(writer)?;
        for modifier in &self.modifiers {
            modifier.write(writer)?;
        }
        Ok(())
    }
}

/// Sent to synchronize entity attribute values and their modifiers with the client.
///
/// Used for movement speed (sprint modifier), max health, attack damage, etc.
/// The client uses this to update its local attribute instances.
///
/// Matches vanilla `ClientboundUpdateAttributesPacket`.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_UPDATE_ATTRIBUTES)]
pub struct CUpdateAttributes {
    /// The entity ID whose attributes are being updated.
    pub entity_id: i32,
    /// The attribute snapshots to sync.
    pub attributes: Vec<AttributeSnapshot>,
}

impl CUpdateAttributes {
    /// Creates a new update attributes packet.
    #[must_use]
    pub fn new(entity_id: i32, attributes: Vec<AttributeSnapshot>) -> Self {
        Self {
            entity_id,
            attributes,
        }
    }
}

impl WriteTo for CUpdateAttributes {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(self.entity_id).write(writer)?;
        VarInt(self.attributes.len() as i32).write(writer)?;
        for attr in &self.attributes {
            attr.write(writer)?;
        }
        Ok(())
    }
}
