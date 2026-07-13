//! Generates direct Snapshot-2 `worldgen/carver` registry entries.

use std::fs;

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use steel_utils::value_providers::{FloatProvider, HeightProvider, IntProvider, VerticalAnchor};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum CarverJson {
    #[serde(rename = "minecraft:cave")]
    Cave(CaveJson),
    #[serde(rename = "minecraft:canyon")]
    Canyon(CanyonJson),
}

#[derive(Deserialize)]
struct CaveJson {
    probability: f32,
    y: HeightProvider,
    count: IntProvider,
    thickness: FloatProvider,
    #[serde(default)]
    weird_thickness_bias: bool,
    room_vertical_radius_multiplier: FloatProvider,
    horizontal_radius_multiplier: FloatProvider,
    vertical_radius_multiplier: FloatProvider,
    #[serde(default = "one_float")]
    start_vertical_radius_multiplier: FloatProvider,
    floor_level: FloatProvider,
}

const fn one_float() -> FloatProvider {
    FloatProvider::Constant(1.0)
}

#[derive(Deserialize)]
struct CanyonJson {
    probability: f32,
    y: HeightProvider,
    vertical_rotation: FloatProvider,
    shape: CanyonShapeJson,
}

#[derive(Deserialize)]
struct CanyonShapeJson {
    distance_factor: FloatProvider,
    thickness: FloatProvider,
    width_smoothness: i32,
    horizontal_radius_factor: FloatProvider,
    vertical_radius_default_factor: f32,
    vertical_radius_center_factor: f32,
    y_scale: FloatProvider,
}

fn vertical_anchor(value: VerticalAnchor) -> TokenStream {
    match value {
        VerticalAnchor::Absolute(value) => quote! { VerticalAnchor::Absolute(#value) },
        VerticalAnchor::AboveBottom(value) => quote! { VerticalAnchor::AboveBottom(#value) },
        VerticalAnchor::BelowTop(value) => quote! { VerticalAnchor::BelowTop(#value) },
        VerticalAnchor::RelativeToSeaLevel(value) => {
            quote! { VerticalAnchor::RelativeToSeaLevel(#value) }
        }
    }
}

fn height_provider(value: HeightProvider) -> TokenStream {
    match value {
        HeightProvider::Constant(anchor) => {
            let anchor = vertical_anchor(anchor);
            quote! { HeightProvider::Constant(#anchor) }
        }
        HeightProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            let min = vertical_anchor(min_inclusive);
            let max = vertical_anchor(max_inclusive);
            quote! { HeightProvider::Uniform { min_inclusive: #min, max_inclusive: #max } }
        }
        HeightProvider::Trapezoid {
            min_inclusive,
            max_inclusive,
            plateau,
        } => {
            let min = vertical_anchor(min_inclusive);
            let max = vertical_anchor(max_inclusive);
            quote! { HeightProvider::Trapezoid { min_inclusive: #min, max_inclusive: #max, plateau: #plateau } }
        }
        HeightProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = vertical_anchor(min_inclusive);
            let max = vertical_anchor(max_inclusive);
            quote! { HeightProvider::BiasedToBottom { min_inclusive: #min, max_inclusive: #max, inner: #inner } }
        }
        HeightProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = vertical_anchor(min_inclusive);
            let max = vertical_anchor(max_inclusive);
            quote! { HeightProvider::VeryBiasedToBottom { min_inclusive: #min, max_inclusive: #max, inner: #inner } }
        }
    }
}

fn float_provider(value: FloatProvider) -> TokenStream {
    match value {
        FloatProvider::Constant(value) => quote! { FloatProvider::Constant(#value) },
        FloatProvider::Uniform {
            min_inclusive,
            max_exclusive,
        } => {
            quote! { FloatProvider::Uniform { min_inclusive: #min_inclusive, max_exclusive: #max_exclusive } }
        }
        FloatProvider::Trapezoid { min, max, plateau } => {
            quote! { FloatProvider::Trapezoid { min: #min, max: #max, plateau: #plateau } }
        }
        FloatProvider::ClampedNormal {
            mean,
            deviation,
            min,
            max,
        } => {
            quote! { FloatProvider::ClampedNormal { mean: #mean, deviation: #deviation, min: #min, max: #max } }
        }
    }
}

fn int_provider(value: IntProvider) -> TokenStream {
    match value {
        IntProvider::Constant(value) => quote! { IntProvider::Constant(#value) },
        IntProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            quote! { IntProvider::Uniform { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive } }
        }
        IntProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => {
            quote! { IntProvider::BiasedToBottom { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive } }
        }
        IntProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => {
            quote! { IntProvider::VeryBiasedToBottom { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive } }
        }
        IntProvider::Trapezoid { min, max, plateau } => {
            quote! { IntProvider::Trapezoid { min: #min, max: #max, plateau: #plateau } }
        }
        IntProvider::ClampedNormal {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        } => {
            quote! { IntProvider::ClampedNormal { mean: #mean, deviation: #deviation, min_inclusive: #min_inclusive, max_inclusive: #max_inclusive } }
        }
        IntProvider::Clamped {
            source,
            min_inclusive,
            max_inclusive,
        } => {
            let source = int_provider(*source);
            quote! { IntProvider::Clamped { source: Box::new(#source), min_inclusive: #min_inclusive, max_inclusive: #max_inclusive } }
        }
        IntProvider::WeightedList { distribution } => {
            let entries = distribution.into_iter().map(|entry| {
                let data = int_provider(entry.data);
                let weight = entry.weight;
                quote! { steel_utils::value_providers::WeightedIntProvider { data: #data, weight: #weight } }
            });
            quote! { IntProvider::WeightedList { distribution: vec![#(#entries),*] } }
        }
    }
}

fn cave_kind(value: CaveJson) -> TokenStream {
    let CaveJson {
        probability,
        y,
        count,
        thickness,
        weird_thickness_bias,
        room_vertical_radius_multiplier,
        horizontal_radius_multiplier,
        vertical_radius_multiplier,
        start_vertical_radius_multiplier,
        floor_level,
    } = value;
    let y = height_provider(y);
    let count = int_provider(count);
    let thickness = float_provider(thickness);
    let room = float_provider(room_vertical_radius_multiplier);
    let horizontal = float_provider(horizontal_radius_multiplier);
    let vertical = float_provider(vertical_radius_multiplier);
    let start_vertical = float_provider(start_vertical_radius_multiplier);
    let floor = float_provider(floor_level);
    quote! { WorldCarverKind::Cave(CaveWorldCarver { probability: #probability, y: #y, count: #count, thickness: #thickness, weird_thickness_bias: #weird_thickness_bias, room_vertical_radius_multiplier: #room, horizontal_radius_multiplier: #horizontal, vertical_radius_multiplier: #vertical, start_vertical_radius_multiplier: #start_vertical, floor_level: #floor }) }
}

fn canyon_kind(value: CanyonJson) -> TokenStream {
    let CanyonJson {
        probability,
        y,
        vertical_rotation,
        shape,
    } = value;
    let y = height_provider(y);
    let rotation = float_provider(vertical_rotation);
    let distance = float_provider(shape.distance_factor);
    let thickness = float_provider(shape.thickness);
    let horizontal = float_provider(shape.horizontal_radius_factor);
    let y_scale = float_provider(shape.y_scale);
    let width = shape.width_smoothness;
    let default_factor = shape.vertical_radius_default_factor;
    let center_factor = shape.vertical_radius_center_factor;
    quote! { WorldCarverKind::Canyon(CanyonWorldCarver { probability: #probability, y: #y, vertical_rotation: #rotation, shape: CanyonShape { distance_factor: #distance, thickness: #thickness, width_smoothness: #width, horizontal_radius_factor: #horizontal, vertical_radius_default_factor: #default_factor, vertical_radius_center_factor: #center_factor, y_scale: #y_scale } }) }
}

pub(crate) fn build() -> TokenStream {
    let dir = "../steel-utils/build_assets/builtin_datapacks/minecraft/worldgen/carver";
    println!("cargo:rerun-if-changed={dir}");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("worldgen/carver dir missing")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort_by_key(std::fs::DirEntry::file_name);

    let entries = files
        .into_iter()
        .map(|entry| {
            let path = entry.path();
            let name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .expect("invalid carver file name")
                .to_owned();
            let content = fs::read_to_string(path).expect("failed to read direct carver data");
            let value: CarverJson = serde_json::from_str(&content)
                .unwrap_or_else(|error| panic!("failed to parse direct carver {name}: {error}"));
            let kind = match value {
                CarverJson::Cave(value) => cave_kind(value),
                CarverJson::Canyon(value) => canyon_kind(value),
            };
            (name, kind)
        })
        .collect::<Vec<_>>();

    let declarations = entries.iter().map(|(name, kind)| {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        quote! { pub static #ident: LazyLock<WorldCarver> = LazyLock::new(|| WorldCarver { key: Identifier::vanilla_static(#name), kind: #kind, id: OnceLock::new() }); }
    });
    let registrations = entries.iter().map(|(name, _)| {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        quote! { registry.register(&#ident); }
    });

    quote! {
        use std::sync::{LazyLock, OnceLock};
        use steel_utils::Identifier;
        use steel_utils::value_providers::{FloatProvider, HeightProvider, IntProvider, VerticalAnchor};
        use crate::carver::{CanyonShape, CanyonWorldCarver, CaveWorldCarver, WorldCarver, WorldCarverKind, WorldCarverRegistry};
        #(#declarations)*
        pub fn register_world_carvers(registry: &mut WorldCarverRegistry) { #(#registrations)* }
    }
}
