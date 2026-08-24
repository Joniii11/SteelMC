pub use crate::{DyeColor, equipment::EquipmentSlotGroup};
use crate::{
    REGISTRY, RegistryExt, TaggedRegistryExt, blocks::block_state_ext::BlockStateExt,
    instrument::InstrumentRef, item_stack::ItemStack,
};
use rand::RngExt;
use rustc_hash::FxHashMap;
use steel_utils::{BlockStateId, Identifier, random::Random as VanillaRandom};

mod conditions;
mod context;
mod entries;
mod functions;
mod registry;

/// Source of random values used by loot evaluation.
pub trait LootRandom {
    /// `RandomSource.nextInt`
    fn next_i32_bounded(&mut self, bound: i32) -> i32;

    /// `RandomSource.nextFloat`
    fn next_f32(&mut self) -> f32;
}

impl<R: rand::Rng + ?Sized> LootRandom for R {
    fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        self.random_range(0..bound)
    }

    fn next_f32(&mut self) -> f32 {
        self.random()
    }
}

/// Vanilla random source accepted by the loot adapter.
pub trait VanillaLootRandomSource {
    /// `RandomSource.nextInt`
    fn next_i32_bounded(&mut self, bound: i32) -> i32;

    /// `RandomSource.nextFloat`
    fn next_f32(&mut self) -> f32;
}

impl<R: VanillaRandom + ?Sized> VanillaLootRandomSource for R {
    fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        VanillaRandom::next_i32_bounded(self, bound)
    }

    fn next_f32(&mut self) -> f32 {
        VanillaRandom::next_f32(self)
    }
}

/// Adapter over a vanilla random source for legacy loot callers.
pub struct VanillaLootRandom<'a, R: VanillaLootRandomSource + ?Sized> {
    source: &'a mut R,
}

impl<'a, R: VanillaLootRandomSource + ?Sized> VanillaLootRandom<'a, R> {
    /// Wraps a vanilla random source.
    #[must_use]
    pub const fn new(source: &'a mut R) -> Self {
        Self { source }
    }
}

impl<R: VanillaLootRandomSource + ?Sized> LootRandom for VanillaLootRandom<'_, R> {
    fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        self.source.next_i32_bounded(bound)
    }

    fn next_f32(&mut self) -> f32 {
        self.source.next_f32()
    }
}

pub use conditions::*;
pub use context::*;
pub use entries::*;
pub use functions::*;
pub use registry::*;

#[cfg(test)]
mod tests;
