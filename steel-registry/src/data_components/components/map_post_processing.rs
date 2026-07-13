//! `minecraft:map_post_processing`

use std::io::{Cursor, Error, Result};

use steel_utils::{
    codec::VarInt,
    serial::{ReadFrom, WriteTo},
};

use crate::data_components::{Component, ComponentData, DataComponentCodecContext};

/// `MapPostProcessing`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapPostProcessing {
    /// Locks map processing
    Lock,
    /// Scales map processing
    Scale,
}

static LOCK_VALUE: MapPostProcessing = MapPostProcessing::Lock;
static SCALE_VALUE: MapPostProcessing = MapPostProcessing::Scale;

impl MapPostProcessing {
    /// Stream ID
    #[must_use]
    pub const fn id(self) -> i32 {
        match self {
            Self::Lock => 0,
            Self::Scale => 1,
        }
    }

    /// Zero fallback ID
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Scale,
            _ => Self::Lock,
        }
    }
}

impl Component for MapPostProcessing {
    fn into_data(self) -> ComponentData {
        ComponentData::I32(self.id())
    }

    fn from_data(data: ComponentData) -> Option<Self> {
        match data {
            ComponentData::I32(id) => Some(Self::from_id(id)),
            _ => None,
        }
    }

    fn from_data_ref(data: &ComponentData) -> Option<&Self> {
        match data {
            ComponentData::I32(id) => match Self::from_id(*id) {
                Self::Lock => Some(&LOCK_VALUE),
                Self::Scale => Some(&SCALE_VALUE),
            },
            _ => None,
        }
    }
}

/// `MapPostProcessing.STREAM_CODEC`
pub fn network_writer(
    _context: &DataComponentCodecContext<'_>,
    data: &ComponentData,
    writer: &mut Vec<u8>,
) -> Result<()> {
    let Some(component) = MapPostProcessing::from_data_ref(data) else {
        return Err(Error::other(
            "Component type mismatch for map_post_processing",
        ));
    };
    VarInt(component.id()).write(writer)
}

/// `MapPostProcessing.STREAM_CODEC`
pub fn network_reader(
    _context: &DataComponentCodecContext<'_>,
    reader: &mut Cursor<&[u8]>,
) -> Result<ComponentData> {
    Ok(MapPostProcessing::from_id(VarInt::read(reader)?.0).into_data())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use steel_utils::{codec::VarInt, serial::WriteTo};

    use super::{MapPostProcessing, network_reader, network_writer};
    use crate::{
        REGISTRY,
        data_components::vanilla_components::MAP_POST_PROCESSING,
        data_components::{ComponentData, DataComponentCodecContext, DataComponentPatch},
        test_support::init_test_registry,
    };

    fn context() -> DataComponentCodecContext<'static> {
        init_test_registry();
        DataComponentCodecContext::new(&REGISTRY)
    }

    #[test]
    fn stream_ids_and_out_of_bounds_mapping_match_vanilla() {
        let context = context();
        for (value, expected, bytes) in [
            (MapPostProcessing::Lock, MapPostProcessing::Lock, vec![0]),
            (MapPostProcessing::Scale, MapPostProcessing::Scale, vec![1]),
        ] {
            let data = ComponentData::I32(value.id());
            let mut encoded = Vec::new();
            network_writer(&context, &data, &mut encoded)
                .expect("map post processing stream encoding must succeed");
            assert_eq!(encoded, bytes);
            assert_eq!(
                network_reader(&context, &mut Cursor::new(encoded.as_slice()))
                    .expect("map post processing stream decoding must succeed"),
                ComponentData::I32(expected.id())
            );
        }

        for id in [-1, 2, i32::MAX] {
            let mut encoded = Vec::new();
            VarInt(id)
                .write(&mut encoded)
                .expect("writing to a vec must succeed");
            assert_eq!(
                network_reader(&context, &mut Cursor::new(encoded.as_slice()))
                    .expect("out-of-bounds stream ID must decode"),
                ComponentData::I32(MapPostProcessing::Lock.id())
            );
        }
    }

    #[test]
    fn transient_component_patch_round_trips_only_on_the_network() {
        let context = context();
        let mut patch = DataComponentPatch::new();
        patch.set(MAP_POST_PROCESSING, MapPostProcessing::Scale);

        let mut network = Vec::new();
        patch
            .write_with_context(&context, &mut network)
            .expect("transient map post processing patch must encode on the network");
        assert_eq!(
            DataComponentPatch::read_with_context(&context, &mut Cursor::new(network.as_slice()))
                .expect("transient map post processing patch must decode on the network"),
            patch
        );
        assert!(patch.is_persistently_empty(&context));
    }
}
