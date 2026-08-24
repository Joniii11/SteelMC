#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted trim material data"
)]

use crate::generator_functions::{generate_identifier, generate_text_component, read_json_asset};
use crate::shared_structs::TextComponentJson;
use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use steel_utils::Identifier;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrimMaterialJson {
    palette_id: Identifier,
    description: TextComponentJson,
}

pub(crate) fn build() -> TokenStream {
    // `TrimMaterials.bootstrap` defines the network holder IDs.
    const VANILLA_ORDER: &[&str] = &[
        "quartz",
        "iron",
        "netherite",
        "redstone",
        "copper",
        "gold",
        "emerald",
        "diamond",
        "lapis",
        "amethyst",
        "resin",
    ];
    let trim_materials = VANILLA_ORDER.iter().map(|name| {
        let path = format!(
            "../steel-utils/build_assets/builtin_datapacks/minecraft/trim_material/{name}.json"
        );
        (*name, read_json_asset::<TrimMaterialJson>(&path))
    });

    let mut stream = TokenStream::new();

    stream.extend(quote! {
        use crate::trim_material::{
            TrimMaterial, TrimMaterialRegistry, TrimMaterialValue,
        };
        use steel_utils::Identifier;
        use std::borrow::Cow;
        use std::sync::LazyLock;
        use text_components::{TextComponent, translation::TranslatedMessage};
    });

    // Generate static trim material definitions
    let mut register_stream = TokenStream::new();
    for (trim_material_name, trim_material) in trim_materials {
        let trim_material_ident = Ident::new(
            &trim_material_name.to_shouty_snake_case(),
            Span::call_site(),
        );
        let trim_material_name_str = trim_material_name;

        let key = quote! { Identifier::vanilla_static(#trim_material_name_str) };
        let palette_id = generate_identifier(&trim_material.palette_id);
        let description = generate_text_component(&trim_material.description);

        stream.extend(quote! {
            pub static #trim_material_ident: LazyLock<TrimMaterial> = LazyLock::new(|| {
                TrimMaterial::new(#key, TrimMaterialValue::new(#palette_id, #description))
            });
        });

        register_stream.extend(quote! {
            registry.register(&#trim_material_ident);
        });
    }

    stream.extend(quote! {
        pub fn register_trim_materials(registry: &mut TrimMaterialRegistry) {
            #register_stream
        }
    });

    stream
}
