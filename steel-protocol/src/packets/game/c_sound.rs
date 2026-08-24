use std::io::{Result, Write};

use glam::{DVec3, IVec3};
use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SOUND;
use steel_registry::sound_event::{SoundEventHolder, SoundEventRef};
use steel_utils::{codec::VarInt, serial::WriteTo};

/// Sound source categories (matches vanilla `SoundSource` enum order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SoundSource {
    Master = 0,
    Music = 1,
    Records = 2,
    Weather = 3,
    Blocks = 4,
    Hostile = 5,
    Neutral = 6,
    Players = 7,
    Ambient = 8,
    Voice = 9,
    Ui = 10,
}

impl SoundSource {
    /// Returns the `VarInt` value for the enum.
    #[must_use]
    pub const fn as_varint(self) -> i32 {
        self as i32
    }
}

/// Sent to play a sound effect at a specific position.
///
/// The position is encoded at 8x precision (divide by 8 to get actual block coordinates).
/// This allows sub-block positioning for more accurate sound placement.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_SOUND)]
pub struct CSound {
    /// `Holder<SoundEvent>`
    pub sound: SoundEventHolder,
    /// The sound source category (`VarInt`).
    pub source: i32,
    /// X position multiplied by 8 (fixed-point).
    pub pos: IVec3,
    /// Volume (1.0 = normal).
    pub volume: f32,
    /// Pitch (1.0 = normal).
    pub pitch: f32,
    /// Random seed for sound variations.
    pub seed: i64,
}

impl CSound {
    /// Creates a new sound packet.
    ///
    /// # Arguments
    /// * `sound` - Sound event to play
    /// * `source` - Sound source category
    /// * `x`, `y`, `z` - Position in block coordinates (will be scaled by 8)
    /// * `volume` - Volume multiplier (1.0 = normal)
    /// * `pitch` - Pitch multiplier (1.0 = normal)
    /// * `seed` - Random seed for sound variations
    #[must_use]
    pub fn new(
        sound: SoundEventRef,
        source: SoundSource,
        pos: DVec3,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self::new_holder(
            SoundEventHolder::registry(sound),
            source,
            pos,
            volume,
            pitch,
            seed,
        )
    }

    /// Holder sound packet
    #[must_use]
    pub fn new_holder(
        sound: SoundEventHolder,
        source: SoundSource,
        pos: DVec3,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self {
            sound,
            source: source.as_varint(),
            pos: IVec3::new(
                (pos.x * 8.0) as i32,
                (pos.y * 8.0) as i32,
                (pos.z * 8.0) as i32,
            ),
            volume,
            pitch,
            seed,
        }
    }

    /// Creates a block sound packet at the center of a block position.
    ///
    /// # Arguments
    /// * `sound` - Sound event to play
    /// * `pos` - Block position (will be centered at +0.5)
    /// * `volume` - Volume multiplier
    /// * `pitch` - Pitch multiplier
    /// * `seed` - Random seed
    #[must_use]
    pub fn block_sound(
        sound: SoundEventRef,
        pos: steel_utils::BlockPos,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self::new(
            sound,
            SoundSource::Blocks,
            pos.0.as_dvec3().map(|v| v + 0.5),
            volume,
            pitch,
            seed,
        )
    }

    /// Block holder sound packet
    #[must_use]
    pub fn block_sound_holder(
        sound: SoundEventHolder,
        pos: steel_utils::BlockPos,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self::new_holder(
            sound,
            SoundSource::Blocks,
            pos.0.as_dvec3().map(|value| value + 0.5),
            volume,
            pitch,
            seed,
        )
    }
}

impl WriteTo for CSound {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        self.sound.write(writer)?;
        VarInt(self.source).write(writer)?;
        self.pos.write(writer)?;
        self.volume.write(writer)?;
        self.pitch.write(writer)?;
        self.seed.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, sync::Once};

    use steel_registry::{
        REGISTRY, Registry, RegistryEntry, sound_event::SoundEventHolder, sound_events,
    };
    use steel_utils::{
        BlockPos, Identifier,
        serial::{ReadFrom, WriteTo},
    };

    use super::CSound;

    #[test]
    fn registered_sound_packet_uses_holder_id() {
        init_vanilla_registry();

        let packet = CSound::block_sound(
            &sound_events::BLOCK_WOODEN_BUTTON_CLICK_ON,
            BlockPos::ZERO,
            1.0,
            1.0,
            0,
        );

        let expected_holder_id = sound_events::BLOCK_WOODEN_BUTTON_CLICK_ON.id() as i32 + 1;
        assert_eq!(
            sound_events::BLOCK_WOODEN_BUTTON_CLICK_ON.packet_holder_id(),
            expected_holder_id
        );
        assert_eq!(
            packet.sound,
            SoundEventHolder::registry(&sound_events::BLOCK_WOODEN_BUTTON_CLICK_ON)
        );
    }

    #[test]
    fn direct_sound_packet_writes_a_direct_holder() {
        init_vanilla_registry();

        let packet = CSound::block_sound_holder(
            SoundEventHolder::Direct {
                sound_id: Identifier::vanilla_static("item.axe.strip"),
                fixed_range: Some(32.0),
            },
            BlockPos::ZERO,
            1.0,
            1.0,
            0,
        );
        let mut bytes = Vec::new();
        packet
            .write(&mut bytes)
            .expect("sound packet should encode");
        assert_eq!(bytes.first(), Some(&0));

        let sound = SoundEventHolder::read(&mut Cursor::new(bytes.as_slice()))
            .expect("direct sound holder should decode");
        assert_eq!(sound, packet.sound);
    }
}
