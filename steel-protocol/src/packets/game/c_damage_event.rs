//! Clientbound damage event packet - tells the client an entity took damage.

use std::io::{Result, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DAMAGE_EVENT;
use steel_utils::{codec::VarInt, serial::WriteTo};

/// Sent when an entity takes damage. The client uses this to play animations
/// and show the damage direction indicator.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_DAMAGE_EVENT)]
pub struct CDamageEvent {
    /// The entity ID of the entity taking damage.
    pub entity_id: i32,
    /// The damage type ID in the `minecraft:damage_type` registry.
    pub source_type_id: i32,
    /// The entity ID + 1 of the entity responsible for the damage, or 0 if none.
    pub source_cause_id: i32,
    /// The entity ID + 1 of the entity that directly dealt the damage, or 0 if none.
    pub source_direct_id: i32,
    /// Optional source position (e.g. for explosions).
    pub source_position: Option<(f64, f64, f64)>,
}

impl WriteTo for CDamageEvent {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(self.entity_id).write(writer)?;
        VarInt(self.source_type_id).write(writer)?;
        VarInt(self.source_cause_id).write(writer)?;
        VarInt(self.source_direct_id).write(writer)?;

        match &self.source_position {
            Some((x, y, z)) => {
                true.write(writer)?;
                x.write(writer)?;
                y.write(writer)?;
                z.write(writer)?;
            }
            None => {
                false.write(writer)?;
            }
        }

        Ok(())
    }
}
