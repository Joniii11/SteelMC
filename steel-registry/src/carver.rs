//! Direct world-carver registry for Minecraft 26.3 Snapshot 2.
//!
//! Snapshot 2 removed `ConfiguredWorldCarver`: biome generation now refers to
//! registered `WorldCarver` values directly. The data here deliberately has no
//! replacement or aquifer fields; those are owned by vanilla's `CarverOutput`.

use std::sync::OnceLock;

use rustc_hash::FxHashMap;
use steel_utils::Identifier;
use steel_utils::value_providers::{FloatProvider, HeightProvider, IntProvider};

/// The direct cave carver record from `CaveWorldCarver`.
#[derive(Debug, Clone)]
pub struct CaveWorldCarver {
    pub probability: f32,
    pub y: HeightProvider,
    pub count: IntProvider,
    pub thickness: FloatProvider,
    pub weird_thickness_bias: bool,
    pub room_vertical_radius_multiplier: FloatProvider,
    pub horizontal_radius_multiplier: FloatProvider,
    pub vertical_radius_multiplier: FloatProvider,
    pub start_vertical_radius_multiplier: FloatProvider,
    pub floor_level: FloatProvider,
}

/// Direct canyon shape data from `CanyonWorldCarver.Shape`.
#[derive(Debug, Clone)]
pub struct CanyonShape {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
    pub y_scale: FloatProvider,
}

/// The direct canyon carver record from `CanyonWorldCarver`.
#[derive(Debug, Clone)]
pub struct CanyonWorldCarver {
    pub probability: f32,
    pub y: HeightProvider,
    pub vertical_rotation: FloatProvider,
    pub shape: CanyonShape,
}

impl CanyonShape {
    /// Mirrors `CanyonWorldCarver.initWidthFactors`.
    #[must_use]
    pub fn init_width_factors<R: steel_utils::random::Random>(
        &self,
        gen_depth: i32,
        random: &mut R,
    ) -> Vec<f32> {
        let mut factors = vec![0.0; gen_depth as usize];
        let mut current = 1.0;
        for (index, factor) in factors.iter_mut().enumerate() {
            if index == 0 || random.next_i32_bounded(self.width_smoothness) == 0 {
                current = 1.0 + random.next_f32() * random.next_f32();
            }
            *factor = current * current;
        }
        factors
    }

    /// Mirrors `CanyonWorldCarver.updateVerticalRadius`.
    #[must_use]
    pub fn update_vertical_radius<R: steel_utils::random::Random>(
        &self,
        random: &mut R,
        vertical_radius: f64,
        distance: f32,
        current_step: f32,
    ) -> f64 {
        let vertical_multiplier = 1.0 - (0.5 - current_step / distance).abs() * 2.0;
        let factor = self.vertical_radius_default_factor
            + self.vertical_radius_center_factor * vertical_multiplier;
        f64::from(factor) * vertical_radius * f64::from(0.75 + random.next_f32() * 0.25)
    }
}

/// Algorithm selected by the direct carver codec.
#[derive(Debug, Clone)]
pub enum WorldCarverKind {
    Cave(CaveWorldCarver),
    Canyon(CanyonWorldCarver),
}

/// A direct `minecraft:worldgen/carver` registry entry.
#[derive(Debug)]
pub struct WorldCarver {
    pub key: Identifier,
    pub kind: WorldCarverKind,
    pub id: OnceLock<usize>,
}

pub type WorldCarverRef = &'static WorldCarver;

/// Registry of direct world carvers keyed by their resource location.
pub struct WorldCarverRegistry {
    carvers_by_id: Vec<WorldCarverRef>,
    carvers_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl WorldCarverRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            carvers_by_id: Vec::new(),
            carvers_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    pub fn register(&mut self, entry: WorldCarverRef) -> usize {
        assert!(
            self.allows_registering,
            "Cannot register WorldCarver after registry has been frozen"
        );
        let id = self.carvers_by_id.len();
        let cached = entry.id.get_or_init(|| id);
        assert_eq!(*cached, id, "carver registered with conflicting id");
        self.carvers_by_id.push(entry);
        self.carvers_by_key.insert(entry.key.clone(), id);
        id
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, WorldCarverRef)> + '_ {
        self.carvers_by_id
            .iter()
            .enumerate()
            .map(|(id, &entry)| (id, entry))
    }
}

impl Default for WorldCarverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

crate::impl_registry_ext!(
    WorldCarverRegistry,
    WorldCarver,
    carvers_by_id,
    carvers_by_key
);
crate::impl_registry_entry_eq!(WorldCarver);

impl crate::RegistryEntry for WorldCarver {
    fn key(&self) -> &Identifier {
        &self.key
    }

    fn try_id(&self) -> Option<usize> {
        self.id.get().copied()
    }
}
