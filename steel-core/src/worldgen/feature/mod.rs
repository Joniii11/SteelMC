//! Biome decoration runner for the `FEATURES` chunk stage.
//!
//! Vanilla treats biome decoration as one ordered pass over structure pieces and placed
//! features. This module builds the same per-step placed-feature ordering up front and
//! drives the per-chunk decoration seed loop. Placed-feature modifiers and selector
//! features execute normally; concrete block-mutating features are
//! added through the feature runtime registry.

#[expect(
    clippy::module_inception,
    reason = "the feature runtime implementation intentionally lives in feature.rs"
)]
mod feature;
mod features;
pub(crate) mod instrumentation;
mod placed;
mod placement;
mod predicates;
mod prelude;
mod providers;
mod runner;
mod sorter;
mod state;
mod vanilla_collections;
mod weather;

pub(crate) use runner::FeatureDecorationRunner;

#[cfg(test)]
mod tests;
