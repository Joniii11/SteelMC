//! Clientbound update attributes packet - sent to sync entity attributes with modifiers.

use std::io::{Result, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_UPDATE_ATTRIBUTES;
use steel_utils::{Identifier, codec::VarInt, serial::WriteTo};

/// Represents a single attribute modifier within an attribute snapshot.
#[derive(Clone, Debug)]
pub struct AttributeModifierData {
    /// The resource location identifier for this modifier (e.g. `minecraft:sprinting`).
    pub id: Identifier,
    /// The modifier amount.
    pub amount: f64,
    /// The operation type for this modifier.
    pub operation: AttributeModifierOperation,
}

/// The operation type for an attribute modifier.
///
/// Matches vanilla `AttributeModifier.Operation`:
/// - `AddValue` (0): `total += amount`
/// - `AddMultipliedBase` (1): `total += base * amount`
/// - `AddMultipliedTotal` (2): `total *= 1 + amount`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AttributeModifierOperation {
    AddValue = 0,
    AddMultipliedBase = 1,
    AddMultipliedTotal = 2,
}

/// A snapshot of a single attribute's state, including its base value and active modifiers.
#[derive(Clone, Debug)]
pub struct AttributeSnapshot {
    /// The registry ID of the attribute (VarInt on the wire).
    pub attribute_id: i32,
    /// The base value of the attribute.
    pub base_value: f64,
    /// Active modifiers on this attribute.
    pub modifiers: Vec<AttributeModifierData>,
}

/// Clientbound packet sent to update entity attributes and their modifiers.
///
/// Used for things like sprint speed modifiers, potion effects on speed/health, etc.
/// Vanilla: `ClientboundUpdateAttributesPacket`
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
        // Number of attributes
        VarInt(self.attributes.len() as i32).write(writer)?;
        for attr in &self.attributes {
            // Attribute registry ID
            VarInt(attr.attribute_id).write(writer)?;
            // Base value
            attr.base_value.write(writer)?;
            // Number of modifiers
            VarInt(attr.modifiers.len() as i32).write(writer)?;
            for modifier in &attr.modifiers {
                // Modifier identifier
                modifier.id.write(writer)?;
                // Modifier amount
                modifier.amount.write(writer)?;
                // Modifier operation (VarInt enum)
                VarInt(modifier.operation as i32).write(writer)?;
            }
        }
        Ok(())
    }
}
