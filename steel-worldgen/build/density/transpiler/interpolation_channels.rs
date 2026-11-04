//! Share interpolation slots only for provably identical inner expressions.

use crate::density::DensityFunction;

use super::graph::unwrap_markers;

pub(super) fn intern_inner(inners: &mut Vec<DensityFunction>, inner: DensityFunction) -> usize {
    if let Some(channel) = inners
        .iter()
        .position(|previous| can_share(unwrap_markers(previous), unwrap_markers(&inner)))
    {
        return channel;
    }

    let channel = inners.len();
    inners.push(inner);
    channel
}

// Conservative equality: other shapes retain their own channels.
fn can_share(left: &DensityFunction, right: &DensityFunction) -> bool {
    match (left, right) {
        (DensityFunction::Constant(a), DensityFunction::Constant(b)) => {
            a.value.to_bits() == b.value.to_bits()
        }
        (DensityFunction::Reference(a), DensityFunction::Reference(b)) => a.id == b.id,
        (DensityFunction::Noise(a), DensityFunction::Noise(b)) => {
            a.noise_id == b.noise_id
                && a.xz_scale.to_bits() == b.xz_scale.to_bits()
                && a.y_scale.to_bits() == b.y_scale.to_bits()
        }
        (DensityFunction::RangeChoice(a), DensityFunction::RangeChoice(b)) => {
            a.min_inclusive.to_bits() == b.min_inclusive.to_bits()
                && a.max_exclusive.to_bits() == b.max_exclusive.to_bits()
                && can_share(&a.input, &b.input)
                && can_share(&a.when_in_range, &b.when_in_range)
                && can_share(&a.when_out_of_range, &b.when_out_of_range)
        }
        (DensityFunction::Marker(a), DensityFunction::Marker(b)) => {
            a.kind == b.kind
                && a.cell_size_xz == b.cell_size_xz
                && a.cell_size_y == b.cell_size_y
                && can_share(&a.wrapped, &b.wrapped)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::density::{BlendedNoise, DensityFunction, Noise, RangeChoice, Reference};

    use super::super::fingerprint::fingerprint;
    use super::intern_inner;

    fn ore_range(scale: f64, lower_bound: f64) -> DensityFunction {
        DensityFunction::RangeChoice(RangeChoice {
            input: Arc::new(DensityFunction::Reference(Reference {
                id: "minecraft:y".into(),
                resolved: None,
            })),
            min_inclusive: lower_bound,
            max_exclusive: 57.0,
            when_in_range: Arc::new(DensityFunction::Noise(Noise {
                noise_id: "minecraft:ore_veininess".into(),
                xz_scale: scale,
                y_scale: 1.5,
                noise: None,
            })),
            when_out_of_range: Arc::new(DensityFunction::constant(0.0)),
        })
    }

    #[test]
    fn shared_ore_channels_keep_scale_and_range_bits() {
        let mut inners = Vec::new();
        assert_eq!(intern_inner(&mut inners, ore_range(0.0, 0.0)), 0);
        assert_eq!(intern_inner(&mut inners, ore_range(0.0, 0.0)), 0);
        assert_eq!(intern_inner(&mut inners, ore_range(-0.0, 0.0)), 1);
        assert_eq!(intern_inner(&mut inners, ore_range(0.0, -0.0)), 2);
    }

    #[test]
    fn unsupported_inputs_do_not_merge_on_matching_fingerprints() {
        let blended = |xz_scale| {
            DensityFunction::BlendedNoise(BlendedNoise {
                xz_scale,
                y_scale: 0.125,
                xz_factor: 80.0,
                y_factor: 160.0,
                smear_scale_multiplier: 8.0,
                noise: None,
            })
        };
        let first = blended(0.25);
        let second = blended(0.5);
        assert_eq!(fingerprint(&first), fingerprint(&second));

        let mut inners = Vec::new();
        assert_eq!(intern_inner(&mut inners, first), 0);
        assert_eq!(intern_inner(&mut inners, second), 1);
    }
}
