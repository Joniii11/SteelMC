//! Conservative init-grid-only substitution of raw EndIslands leaves.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::density::{Axis, DensityFunction, MarkerType};

use super::TranspilerInput;
use super::context::TranspileContext;
use super::graph::unwrap_markers;
use super::naming::named_fn_ident;

/// A deliberately small grammar: every accepted path preserves raw X and Z.
/// References and unknown/coordinate-changing nodes decline the whole helper.
fn raw_end_islands_leaf_count(df: &DensityFunction) -> Option<usize> {
    match df {
        DensityFunction::Constant(_) | DensityFunction::DistanceToPoint(_) => Some(0),
        DensityFunction::EndIslands => Some(1),
        DensityFunction::TwoArgumentSimple(binary) => {
            let first = raw_end_islands_leaf_count(&binary.argument1)?;
            first.checked_add(raw_end_islands_leaf_count(&binary.argument2)?)
        }
        DensityFunction::Clamp(clamp) => raw_end_islands_leaf_count(&clamp.input),
        DensityFunction::Slice(slice) if slice.axis == Axis::Y => {
            raw_end_islands_leaf_count(&slice.input)
        }
        DensityFunction::Marker(marker)
            if matches!(
                marker.kind,
                MarkerType::FlatCache
                    | MarkerType::Cache2D
                    | MarkerType::CacheOnce
                    | MarkerType::CacheAllInCell
            ) =>
        {
            raw_end_islands_leaf_count(&marker.wrapped)
        }
        _ => None,
    }
}

impl TranspileContext {
    /// Emit additional helpers only; original named/router functions keep scalar codegen.
    pub(super) fn gen_end_grid_batch_helpers<'a>(
        &mut self,
        input: &'a TranspilerInput,
    ) -> (TokenStream, BTreeSet<&'a str>) {
        let mut helpers = Vec::new();
        let mut names = BTreeSet::new();

        for (name, df) in &input.registry {
            if !self.flat_cached.contains(name)
                || !raw_end_islands_leaf_count(df).is_some_and(|count| count > 0)
            {
                continue;
            }

            // This accepted grammar has no Reference/Noise/Spline CSE nodes.
            // gen_expr therefore preserves both the original expression tree
            // and its f32 rounding boundaries without carrying bindings out.
            let previous_mode = self.end_grid_batch_mode;
            self.end_grid_batch_mode = true;
            let body = self.gen_expr(unwrap_markers(df), input, true);
            self.end_grid_batch_mode = previous_mode;

            let params = self.fn_params(true);
            let helper = format_ident!("{}_grid_end_islands", named_fn_ident(name));
            helpers.push(quote! {
                #[inline]
                fn #helper(#params, end_islands_grid_value: f64) -> f32 {
                    #body
                }
            });
            names.insert(name.as_str());
        }

        let output = if helpers.is_empty() {
            TokenStream::new()
        } else {
            let cache = &self.cache_ident;
            // Keep helper names out of the free named-DF function namespace.
            quote! { impl #cache { #(#helpers)* } }
        };
        (output, names)
    }
}

#[cfg(test)]
#[path = "end_grid_batch_tests.rs"]
mod tests;
