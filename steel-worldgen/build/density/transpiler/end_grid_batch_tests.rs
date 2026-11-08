//! Generic AST safety gates, not invented Vanilla registry/data fixtures.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::density::{Axis, DensityFunction, Reference, Slice};

use super::super::TranspilerInput;
use super::{TranspileContext, raw_end_islands_leaf_count};

fn sliced_leaf(axis: Axis) -> DensityFunction {
    DensityFunction::Slice(Slice {
        axis,
        coordinate: 7,
        input: Arc::new(DensityFunction::EndIslands),
    })
}

#[test]
fn grid_batch_accepts_only_proved_raw_xz_contexts() {
    assert_eq!(raw_end_islands_leaf_count(&sliced_leaf(Axis::Y)), Some(1));
    assert_eq!(raw_end_islands_leaf_count(&sliced_leaf(Axis::X)), None);
    assert_eq!(raw_end_islands_leaf_count(&sliced_leaf(Axis::Z)), None);
    // Even a resolved reference is deliberately not a supported substitution path.
    let reference = DensityFunction::Reference(Reference {
        id: "audit:leaf".into(),
        resolved: Some(Arc::new(DensityFunction::EndIslands)),
    });
    assert_eq!(raw_end_islands_leaf_count(&reference), None);
}

#[test]
fn private_grid_mode_does_not_leak_into_original_named_functions() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let input = TranspilerInput {
            registry: BTreeMap::from([("audit:leaf".into(), sliced_leaf(axis))]),
            router_entries: BTreeMap::from([(
                "erosion".into(),
                DensityFunction::Reference(Reference {
                    id: "audit:leaf".into(),
                    resolved: None,
                }),
            )]),
            prefix: "Audit".into(),
            cell_width: 8,
            cell_height: 4,
            legacy_random_source: true,
        };
        let mut context = TranspileContext::new("Audit");
        context.analyze(&input);
        let before = context.gen_named_functions(&input).to_string();
        let column = context.gen_column_cache(&input).to_string();
        let after = context.gen_named_functions(&input).to_string();
        assert_eq!(before, after);
        assert!(!context.end_grid_batch_mode);
        assert!(before.contains("end_islands . sample"));
        assert_eq!(column.contains("sample_batch"), axis == Axis::Y);
        assert_eq!(column.contains("end_islands_grid_value"), axis == Axis::Y);
    }
}
