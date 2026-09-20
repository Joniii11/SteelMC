//! Compile generated expressions so lexical scope and channel selection are exercised.

use super::*;
use crate::density::{Constant, IntervalSelect, Marker, RangeChoice, Reference, YClampedGradient};
use std::collections::BTreeMap;
use std::env::temp_dir;
use std::fs::remove_file;
use std::io::Write;
use std::process::{Command, Stdio, id};
use std::sync::atomic::{AtomicUsize, Ordering};

fn input() -> TranspilerInput {
    TranspilerInput {
        registry: BTreeMap::new(),
        router_entries: BTreeMap::new(),
        prefix: "Test".into(),
        cell_width: 4,
        cell_height: 8,
        legacy_random_source: false,
    }
}

fn reference(id: &str) -> Arc<DensityFunction> {
    Arc::new(DensityFunction::Reference(Reference {
        id: id.into(),
        resolved: None,
    }))
}

fn range(
    input: Arc<DensityFunction>,
    when_in_range: Arc<DensityFunction>,
    when_out_of_range: Arc<DensityFunction>,
) -> Arc<DensityFunction> {
    Arc::new(DensityFunction::RangeChoice(RangeChoice {
        input,
        min_inclusive: 0.0,
        max_exclusive: 10.0,
        when_in_range,
        when_out_of_range,
    }))
}

fn compile_and_run(source: TokenStream) {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let path = temp_dir().join(format!(
        "steel-density-codegen-{}-{}",
        id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut compiler = Command::new("rustc")
        .args(["--edition=2024", "-Copt-level=2", "-o"])
        .arg(&path)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Rust compiler for generated expressions");
    compiler
        .stdin
        .take()
        .expect("compiler stdin")
        .write_all(source.to_string().as_bytes())
        .expect("write generated expressions");
    let output = compiler
        .wait_with_output()
        .expect("compile generated expressions");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&path)
        .output()
        .expect("evaluate generated expressions");
    remove_file(&path).expect("remove generated test executable");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn nested_branches_preserve_outer_inputs_in_scalar_and_simd() {
    let input = input();
    let mut context = TranspileContext::new("Test");
    context.flat_cached.extend(["a".into(), "b".into()]);
    let nested_range = range(
        reference("a"),
        range(reference("b"), reference("a"), reference("b")),
        reference("b"),
    );
    let nested_interval = range(
        reference("a"),
        Arc::new(DensityFunction::IntervalSelect(IntervalSelect {
            input: reference("b"),
            thresholds: vec![0.0],
            functions: vec![reference("a"), reference("b")],
        })),
        reference("b"),
    );
    let range_scalar = context.gen_expr(&nested_range, &input, false);
    let range_simd = context.gen_expr_simd(&nested_range, &input, false);
    let interval_scalar = context.gen_expr(&nested_interval, &input, false);
    let interval_simd = context.gen_expr_simd(&nested_interval, &input, false);
    compile_and_run(quote! {
        #![feature(portable_simd)]
        use std::simd::{Simd, Select, cmp::SimdPartialOrd};
        struct Cache { df_a: f32, df_b: f32 }
        fn main() {
            const N: usize = 4;
            let ys = Simd::<f64, N>::from_array([-1.0, 0.0, 1.0, 12.0]);
            for a in [-1.0_f32, 0.0, 3.0, 9.0, 10.0] {
                for b in [-2.0_f32, 0.0, 4.0, 11.0] {
                    let cache = &Cache { df_a: a, df_b: b };
                    let expected = if (0.0..10.0).contains(&a) && (0.0..10.0).contains(&b) { a } else { b };
                    assert_eq!((#range_scalar).to_bits(), expected.to_bits());
                    assert_eq!((#range_simd).to_array().map(f32::to_bits), [expected.to_bits(); N]);
                    let expected = if (0.0..10.0).contains(&a) && b < 0.0 { a } else { b };
                    assert_eq!((#interval_scalar).to_bits(), expected.to_bits());
                    assert_eq!((#interval_simd).to_array().map(f32::to_bits), [expected.to_bits(); N]);
                }
            }
        }
    });
}

#[test]
fn reused_interpolated_input_keeps_later_channels_aligned() {
    let mut input = input();
    for (name, value) in [("a", 3.0), ("b", 7.0)] {
        input.registry.insert(
            name.into(),
            DensityFunction::Marker(Marker {
                kind: MarkerType::Interpolated,
                wrapped: Arc::new(if name == "a" {
                    DensityFunction::YClampedGradient(YClampedGradient {
                        from_y: -20,
                        to_y: 20,
                        from_value: -20.0,
                        to_value: 20.0,
                    })
                } else {
                    DensityFunction::Constant(Constant { value })
                }),
                cell_size_xz: 4,
                cell_size_y: 8,
            }),
        );
    }
    let expression = range(reference("a"), reference("a"), reference("b"));
    let mut context = TranspileContext::new("Test");
    context.interpolated_refs.extend(["a".into(), "b".into()]);
    context.interpolated_param_mode = true;
    context.interpolated_param_channels = vec![0, 0, 1];
    let expression = context.gen_expr(&expression, &input, false);
    assert_eq!(context.interpolated_param_counter, 3);
    compile_and_run(quote! {
        fn main() {
            for (interpolated, expected) in [([3.0_f32, 7.0], 3.0_f32), ([-3.0, 7.0], 7.0)] {
                assert_eq!((#expression).to_bits(), expected.to_bits());
            }
        }
    });
}

#[test]
fn material_cache_analysis_retains_non_interpolated_dependencies() {
    let mut input = input();
    let mut context = TranspileContext::new("Test");
    let flat = DensityFunction::Reference(Reference {
        id: "flat".into(),
        resolved: None,
    });
    input.registry.insert(
        "flat".into(),
        DensityFunction::Constant(Constant { value: 1.0 }),
    );
    context.flat_cached.insert("flat".into());
    assert!(context.combine_needs_column_cache(&flat, &input));
    let interpolated = DensityFunction::Marker(Marker {
        kind: MarkerType::Interpolated,
        wrapped: Arc::new(flat),
        cell_size_xz: 4,
        cell_size_y: 8,
    });
    assert!(!context.combine_needs_column_cache(&interpolated, &input));
}

#[test]
fn shared_channels_preserve_material_combine_results() {
    let mut input = input();
    for (name, value) in [("a", 3.0), ("b", 7.0)] {
        input.registry.insert(
            name.into(),
            DensityFunction::Marker(Marker {
                kind: MarkerType::Interpolated,
                wrapped: Arc::new(if name == "a" {
                    DensityFunction::YClampedGradient(YClampedGradient {
                        from_y: -20,
                        to_y: 20,
                        from_value: -20.0,
                        to_value: 20.0,
                    })
                } else {
                    DensityFunction::Constant(Constant { value })
                }),
                cell_size_xz: 4,
                cell_size_y: 8,
            }),
        );
    }
    input
        .router_entries
        .insert("final_density".into(), (*reference("a")).clone());
    input.router_entries.insert(
        "material_ore_vein_0_density".into(),
        (*range(reference("a"), reference("a"), reference("b"))).clone(),
    );
    let mut context = TranspileContext::new("Test");
    context.analyze(&input);
    let functions = context.gen_all_interpolation_functions(&input);
    compile_and_run(quote! {
        #![feature(portable_simd)]
        use std::simd::{Simd, cmp::SimdOrd, num::{SimdFloat, SimdInt}};
        struct TestNoises;
        struct TestColumnCache { x: i32, z: i32 }
        #functions
        fn main() {
            let noises = TestNoises;
            let mut cache = TestColumnCache { x: 0, z: 0 };
            let mut values = [0.0_f32; INTERPOLATED_COUNT];
            fill_cell_corner_densities(&noises, &cache, 0, 3, 0, 0.0, &mut values);
            assert_eq!(&values[..], &[3.0, 7.0]);
            let mut simd = [0.0; 4 * INTERPOLATED_COUNT];
            fill_cell_corner_densities_y_simd(&noises, &cache, 0, [3, 3, 3, 3], 0, [0.0; 4], &mut simd);
            for lane in simd.chunks_exact(INTERPOLATED_COUNT) { assert_eq!(lane, values); }
            assert_eq!(combine_material_ore_vein_0_density(&noises, &mut cache, &values, 0, 0, 0), 3.0);
            values[0] = -3.0;
            assert_eq!(combine_material_ore_vein_0_density(&noises, &mut cache, &values, 0, 0, 0), 7.0);
        }
    });
}
