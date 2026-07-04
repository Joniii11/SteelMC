use crate::generator_functions::{
    generate_identifier, read_json_asset, sort_contiguous_registry_entries,
};
use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use steel_utils::Identifier;

#[derive(Deserialize, Debug)]
pub struct DecoratedPotPatternJson {
    id: usize,
    key: Identifier,
    asset_id: Identifier,
}

pub(crate) fn build() -> TokenStream {
    let path = "build_assets/decorated_pot_patterns.json";
    let mut decorated_pot_patterns: Vec<DecoratedPotPatternJson> = read_json_asset(path);
    sort_contiguous_registry_entries(&mut decorated_pot_patterns, path, |pattern| pattern.id);

    let mut stream = TokenStream::new();

    stream.extend(quote! {
        use crate::decorated_pot_pattern::{
            DecoratedPotPattern, DecoratedPotPatternRegistry,
        };
        use steel_utils::Identifier;
        use std::borrow::Cow;
    });

    let mut register_stream = TokenStream::new();
    for decorated_pot_pattern in &decorated_pot_patterns {
        let decorated_pot_pattern_name = decorated_pot_pattern.key.path.as_ref();
        let decorated_pot_pattern_ident = Ident::new(
            &decorated_pot_pattern_name.to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = generate_identifier(&decorated_pot_pattern.key);
        let asset_id = generate_identifier(&decorated_pot_pattern.asset_id);

        stream.extend(quote! {
            pub static #decorated_pot_pattern_ident: DecoratedPotPattern = DecoratedPotPattern {
                key: #key,
                asset_id: #asset_id,
            };
        });

        register_stream.extend(quote! {
            registry.register(&#decorated_pot_pattern_ident);
        });
    }

    stream.extend(quote! {
        pub fn register_decorated_pot_patterns(registry: &mut DecoratedPotPatternRegistry) {
            #register_stream
        }
    });

    stream
}
