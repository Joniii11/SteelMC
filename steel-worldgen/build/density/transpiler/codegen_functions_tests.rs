use std::collections::BTreeMap;
use std::sync::Arc;

use proc_macro2::{Delimiter, TokenStream, TokenTree};

use crate::density::{
    CubicSpline, DensityFunction, Marker, MarkerType, Noise, RangeChoice, Reference, Spline,
    SplinePoint, SplineValue, TwoArgType, TwoArgumentSimple,
};

use super::super::TranspilerInput;
use super::TranspileContext;

fn interpolated(name: &str) -> DensityFunction {
    DensityFunction::Marker(Marker {
        kind: MarkerType::Interpolated,
        wrapped: Arc::new(DensityFunction::Noise(Noise {
            noise_id: name.into(),
            xz_scale: 1.0,
            y_scale: 1.0,
            noise: None,
        })),
        cell_size_xz: 4,
        cell_size_y: 8,
    })
}

fn binary(op: TwoArgType, a: DensityFunction, b: DensityFunction) -> DensityFunction {
    DensityFunction::TwoArgumentSimple(TwoArgumentSimple {
        op,
        argument1: Arc::new(a),
        argument2: Arc::new(b),
    })
}

fn function_body(tokens: &TokenStream, name: &str) -> String {
    tokens
        .clone()
        .into_iter()
        .skip_while(|token| !matches!(token, TokenTree::Ident(ident) if ident == name))
        .skip(1)
        .find_map(|token| match token {
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                Some(group.stream().to_string())
            }
            _ => None,
        })
        .expect("generated combine function must have a body")
}

#[test]
fn shared_channels_remap_router_material_and_spline_occurrences() {
    let final_density = binary(
        TwoArgType::Min,
        binary(TwoArgType::Add, interpolated("a"), interpolated("a")),
        interpolated("b"),
    );
    let richness = DensityFunction::Spline(Spline {
        spline: Arc::new(CubicSpline::new(
            Arc::new(interpolated("c")),
            vec![SplinePoint {
                location: 0.0,
                derivative: 1.0,
                value: SplineValue::Constant(2.0),
            }],
        )),
    });
    let pair = || {
        DensityFunction::Reference(Reference {
            id: "pair".into(),
            resolved: None,
        })
    };
    // branch-input CSE must skip both logical a occurrences, not one shared slot
    let material_density = binary(
        TwoArgType::Add,
        DensityFunction::RangeChoice(RangeChoice {
            input: Arc::new(pair()),
            min_inclusive: -1.0,
            max_exclusive: 1.0,
            when_in_range: Arc::new(pair()),
            when_out_of_range: Arc::new(interpolated("b")),
        }),
        interpolated("c"),
    );
    let input = TranspilerInput {
        registry: BTreeMap::from([(
            "pair".into(),
            binary(TwoArgType::Add, interpolated("a"), interpolated("a")),
        )]),
        router_entries: BTreeMap::from([
            ("final_density".into(), final_density),
            ("vein_toggle".into(), interpolated("a")),
            ("vein_ridged".into(), interpolated("c")),
            ("material_ore_vein_0_density".into(), material_density),
            (
                "material_ore_vein_0_filler_gap".into(),
                DensityFunction::constant(0.0),
            ),
            ("material_ore_vein_0_richness".into(), richness),
        ]),
        prefix: "Test".into(),
        cell_width: 4,
        cell_height: 8,
        legacy_random_source: false,
    };
    let mut context = TranspileContext::new("Test");
    context.analyze(&input);
    let tokens = context.gen_all_interpolation_functions(&input);
    let code = tokens.to_string();
    assert!(code.contains("INTERPOLATED_COUNT : usize = 3"));
    assert!(
        code.contains("FINAL_DENSITY_NONPOSITIVE_CHANNELS : Option < & [usize] > = Some (& [1")
    );

    let final_body = function_body(&tokens, "combine_interpolated");
    assert_eq!(final_body.matches("interpolated [0]").count(), 2);
    assert_eq!(final_body.matches("interpolated [1]").count(), 1);
    assert!(function_body(&tokens, "combine_vein_toggle").contains("interpolated [0]"));
    assert!(function_body(&tokens, "combine_vein_ridged").contains("interpolated [2]"));

    let material_body = function_body(&tokens, "combine_material_ore_vein_0_density");
    assert_eq!(material_body.matches("interpolated [1]").count(), 1);
    assert_eq!(material_body.matches("interpolated [0]").count(), 2);
    assert_eq!(material_body.matches("interpolated [2]").count(), 1);
    assert!(
        function_body(&tokens, "combine_material_ore_vein_0_richness").contains("interpolated [2]")
    );
}
