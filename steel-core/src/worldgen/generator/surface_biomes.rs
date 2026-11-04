//! fuzzed biome lookup for the surface stage

use glam::IVec3;
use rustc_hash::FxHashMap;
use steel_registry::REGISTRY;
use steel_registry::biome::TemperatureModifier;
use steel_worldgen::density::{DimensionNoises, NoiseSettings};
use steel_worldgen::surface::SurfaceBiomeProvider;
use steel_worldgen::surface_partial::PartialSurfaceOutcome;

use super::vanilla::{get_fiddle, lcg_next};

/// partition of biome ids into classes
pub(super) struct SurfaceBiomeClasses {
    class_of: Box<[u16]>,
    below_preliminary: Box<[PartialSurfaceOutcome]>,
    min_y: i32,
    height: i32,
}

impl SurfaceBiomeClasses {
    /// group by rule membership and temperature none if unused or too many sets
    pub(super) fn new<N: DimensionNoises>() -> Option<Self> {
        let sets = N::surface_rule_biome_sets();
        if !N::surface_rule_uses_biome() || sets.len() > u64::BITS as usize {
            return None;
        }

        let mut classes = FxHashMap::<(u64, u32, bool), u16>::default();
        let mut class_masks = Vec::new();
        let class_of = REGISTRY
            .biomes
            .iter()
            .map(|(id, biome)| {
                let mask = sets
                    .iter()
                    .enumerate()
                    .filter(|(_, set)| set.contains(&(id as u16)))
                    .fold(0u64, |mask, (bit, _)| mask | 1 << bit);

                let key = (
                    mask,
                    biome.temperature.to_bits(),
                    matches!(biome.temperature_modifier, TemperatureModifier::Frozen),
                );

                let next = classes.len() as u16;
                *classes.entry(key).or_insert_with(|| {
                    class_masks.push(mask);
                    next
                })
            })
            .collect();

        let min_y = N::Settings::MIN_Y;
        let height = N::Settings::HEIGHT;
        let below_preliminary = if N::surface_rule_uses_preliminary_surface() {
            let rule = N::surface_rule_below_preliminary_surface();
            class_masks
                .iter()
                .flat_map(|&mask| (min_y..min_y + height).map(move |y| rule.resolve(y, mask)))
                .collect()
        } else {
            Box::default()
        };

        Some(Self {
            class_of,
            below_preliminary,
            min_y,
            height,
        })
    }

    fn class(&self, biome_id: u16) -> Option<u16> {
        self.class_of.get(usize::from(biome_id)).copied()
    }

    fn below_preliminary_outcome(&self, class: u16, block_y: i32) -> PartialSurfaceOutcome {
        let index = usize::from(class) * self.height as usize + (block_y - self.min_y) as usize;
        self.below_preliminary
            .get(index)
            .copied()
            .unwrap_or(PartialSurfaceOutcome::Evaluate)
    }
}

/// Quart biome reads for one chunk and the one-quart ring around it.
pub(super) struct SurfaceQuartBiomes<'a> {
    biome_data: &'a [u16],
    classes: Option<&'a SurfaceBiomeClasses>,
    uniform_groups: Box<[UniformBiomeGroup]>,
    chunk_quart_x: i32,
    chunk_quart_z: i32,
    min_qy: i32,
    total_quarts_y: i32,
    neighbor_biomes: &'a dyn Fn(IVec3) -> u16,
    ring: Box<[u16]>,
}

const QUARTS_PER_SECTION_SIDE: i32 = 4;
const QUARTS_PER_SECTION_PLANE: i32 = QUARTS_PER_SECTION_SIDE * QUARTS_PER_SECTION_SIDE;
const QUARTS_PER_SECTION: i32 = QUARTS_PER_SECTION_PLANE * QUARTS_PER_SECTION_SIDE;
const RING_SIDE: i32 = QUARTS_PER_SECTION_SIDE + 2;
const GROUP_SIDE: i32 = QUARTS_PER_SECTION_SIDE + 1;
const UNREAD: u16 = u16::MAX;

#[derive(Clone, Copy)]
enum UniformBiomeGroup {
    Unread,
    Mixed,
    Uniform(u16),
}

impl<'a> SurfaceQuartBiomes<'a> {
    pub(super) fn new(
        biome_data: &'a [u16],
        classes: Option<&'a SurfaceBiomeClasses>,
        section_count: usize,
        min_y: i32,
        chunk_quart_x: i32,
        chunk_quart_z: i32,
        neighbor_biomes: &'a dyn Fn(IVec3) -> u16,
    ) -> Self {
        let total_quarts_y = (section_count * QUARTS_PER_SECTION_SIDE as usize) as i32;
        let group_count = if classes.is_some() {
            (GROUP_SIDE * GROUP_SIDE * (total_quarts_y + 1)) as usize
        } else {
            0
        };

        Self {
            biome_data,
            classes,
            uniform_groups: vec![UniformBiomeGroup::Unread; group_count].into(),
            chunk_quart_x,
            chunk_quart_z,
            min_qy: min_y >> 2,
            total_quarts_y,
            neighbor_biomes,
            ring: vec![UNREAD; (RING_SIDE * RING_SIDE * total_quarts_y) as usize].into(),
        }
    }

    #[expect(
        clippy::similar_names,
        reason = "quart coordinate names mirror the x/y/z axes"
    )]
    fn biome_at(&mut self, quart: IVec3) -> u16 {
        let qy = (quart.y - self.min_qy).clamp(0, self.total_quarts_y - 1);
        let local_qx = quart.x - self.chunk_quart_x;
        let local_qz = quart.z - self.chunk_quart_z;

        if (0..QUARTS_PER_SECTION_SIDE).contains(&local_qx)
            && (0..QUARTS_PER_SECTION_SIDE).contains(&local_qz)
        {
            let (section_idx, local_qy) =
                (qy / QUARTS_PER_SECTION_SIDE, qy % QUARTS_PER_SECTION_SIDE);

            return self.biome_data[(section_idx * QUARTS_PER_SECTION
                + local_qy * QUARTS_PER_SECTION_PLANE
                + local_qz * QUARTS_PER_SECTION_SIDE
                + local_qx) as usize];
        }

        let (rx, rz) = (local_qx + 1, local_qz + 1);
        if !(0..RING_SIDE).contains(&rx) || !(0..RING_SIDE).contains(&rz) {
            return (self.neighbor_biomes)(quart);
        }

        let slot = &mut self.ring[((qy * RING_SIDE + rz) * RING_SIDE + rx) as usize];
        if *slot == UNREAD {
            *slot = (self.neighbor_biomes)(quart);
        }

        *slot
    }

    /// nearby block columns share the same eight quart candidates
    fn uniform_group_biome(&mut self, parent: IVec3) -> Option<u16> {
        let classes = self.classes?;
        let x = parent.x - self.chunk_quart_x;
        let y = parent.y - self.min_qy;
        let z = parent.z - self.chunk_quart_z;

        // keep the bottom group separate: both of its y corners clamp to min_qy
        if !(-1..QUARTS_PER_SECTION_SIDE).contains(&x)
            || !(-1..QUARTS_PER_SECTION_SIDE).contains(&z)
            || !(-1..self.total_quarts_y).contains(&y)
        {
            return self.classify_group(classes, parent);
        }

        let index = (((y + 1) * GROUP_SIDE + z + 1) * GROUP_SIDE + x + 1) as usize;
        match self.uniform_groups[index] {
            UniformBiomeGroup::Uniform(biome) => return Some(biome),
            UniformBiomeGroup::Mixed => return None,
            UniformBiomeGroup::Unread => {}
        }

        let biome = self.classify_group(classes, parent);
        self.uniform_groups[index] = match biome {
            Some(biome) => UniformBiomeGroup::Uniform(biome),
            None => UniformBiomeGroup::Mixed,
        };

        biome
    }

    fn classify_group(&mut self, classes: &SurfaceBiomeClasses, parent: IVec3) -> Option<u16> {
        let first = self.biome_at(parent);
        let class = classes.class(first)?;

        for i in 1..8 {
            let biome = self.biome_at(
                parent
                    + IVec3::new(
                        i32::from(i & 4 != 0),
                        i32::from(i & 2 != 0),
                        i32::from(i & 1 != 0),
                    ),
            );
            if classes.class(biome) != Some(class) {
                return None;
            }
        }

        Some(first)
    }
}

#[derive(Clone, Copy, Default)]
struct BiomeCandidate {
    y_fiddle: f64,
    x_distance_sq: f64,
    z_distance_sq: f64,
}

impl BiomeCandidate {
    fn squared_distance(self, dy: f64) -> f64 {
        self.z_distance_sq + (dy + self.y_fiddle) * (dy + self.y_fiddle) + self.x_distance_sq
    }
}

/// cache fuzzed biome lookups for one column
pub(super) struct FuzzedBiomeColumn<'a, 'b> {
    quarts: &'a mut SurfaceQuartBiomes<'b>,
    biome_zoom_seed: i64,
    parent_x: i32,
    parent_z: i32,
    fract_x: f64,
    fract_z: f64,
    cached_parent_y: i32,
    candidates: [BiomeCandidate; 8],
    rval_after_cx: [i64; 2],
    class_parent_y: i32,
    uniform_biome: Option<u16>,
}

impl<'a, 'b> FuzzedBiomeColumn<'a, 'b> {
    pub(super) fn new(
        quarts: &'a mut SurfaceQuartBiomes<'b>,
        biome_zoom_seed: i64,
        block_x: i32,
        block_z: i32,
    ) -> Self {
        let abs_x = block_x - 2;
        let abs_z = block_z - 2;
        let parent_x = abs_x >> 2;
        let parent_z = abs_z >> 2;
        Self {
            quarts,
            biome_zoom_seed,
            parent_x,
            parent_z,
            fract_x: f64::from(abs_x & 3) / 4.0,
            fract_z: f64::from(abs_z & 3) / 4.0,
            cached_parent_y: i32::MIN,
            candidates: [BiomeCandidate::default(); 8],
            rval_after_cx: [
                lcg_next(biome_zoom_seed, i64::from(parent_x)),
                lcg_next(biome_zoom_seed, i64::from(parent_x + 1)),
            ],
            class_parent_y: i32::MIN,
            uniform_biome: None,
        }
    }

    /// fill the low or high y slots for this candidate group
    #[inline]
    fn compute_cy_group(&mut self, cy: i32, high: bool) {
        let base_idx = if high { 2 } else { 0 };
        for cx_idx in 0..2usize {
            let cx = self.parent_x + cx_idx as i32;
            let dx = if cx_idx == 0 {
                self.fract_x
            } else {
                self.fract_x - 1.0
            };

            let rval_cy = lcg_next(self.rval_after_cx[cx_idx], i64::from(cy));
            for cz_off in 0..2usize {
                let cz = self.parent_z + cz_off as i32;
                let dz = if cz_off == 0 {
                    self.fract_z
                } else {
                    self.fract_z - 1.0
                };

                let mut rval = lcg_next(rval_cy, i64::from(cz));
                rval = lcg_next(rval, i64::from(cx));
                rval = lcg_next(rval, i64::from(cy));
                rval = lcg_next(rval, i64::from(cz));
                let fx = get_fiddle(rval);
                rval = lcg_next(rval, self.biome_zoom_seed);
                let fy = get_fiddle(rval);
                rval = lcg_next(rval, self.biome_zoom_seed);
                let fz = get_fiddle(rval);

                self.candidates[cx_idx * 4 + base_idx + cz_off] = BiomeCandidate {
                    y_fiddle: fy,
                    x_distance_sq: (dx + fx) * (dx + fx),
                    z_distance_sq: (dz + fz) * (dz + fz),
                };
            }
        }
    }

    /// refresh candidates for this parent y
    fn recompute_candidates(&mut self, parent_y: i32) {
        if self.cached_parent_y != i32::MIN && parent_y == self.cached_parent_y - 1 {
            // moving down: old low y becomes new high y
            self.candidates[2] = self.candidates[0];
            self.candidates[3] = self.candidates[1];
            self.candidates[6] = self.candidates[4];
            self.candidates[7] = self.candidates[5];
            self.compute_cy_group(parent_y, false);
        } else {
            self.compute_cy_group(parent_y, false);
            self.compute_cy_group(parent_y + 1, true);
        }

        self.cached_parent_y = parent_y;
    }

    /// exact fuzzed biome at this y
    #[inline]
    pub(super) fn get(&mut self, block_y: i32) -> u16 {
        let abs_y = block_y - 2;
        let parent_y = abs_y >> 2;
        let fract_y = f64::from(abs_y & 3) / 4.0;

        if parent_y != self.cached_parent_y {
            self.recompute_candidates(parent_y);
        }

        let mut min_i = 0usize;
        let mut min_dist = f64::INFINITY;
        for (i, candidate) in self.candidates.iter().enumerate() {
            let dy = if (i & 2) == 0 { fract_y } else { fract_y - 1.0 };
            let dist = candidate.squared_distance(dy);
            if min_dist > dist {
                min_i = i;
                min_dist = dist;
            }
        }

        self.quarts.biome_at(IVec3::new(
            self.parent_x + i32::from(min_i & 4 != 0),
            parent_y + i32::from(min_i & 2 != 0),
            self.parent_z + i32::from(min_i & 1 != 0),
        ))
    }

    /// A biome the surface rule cannot distinguish from [`Self::get`]'s.
    pub(super) fn get_for_surface_rule(&mut self, block_y: i32) -> u16 {
        match self.uniform_group_biome(block_y) {
            Some(biome) => biome,
            None => self.get(block_y),
        }
    }

    /// resolve below the preliminary surface using y and a shared biome class
    pub(super) fn below_preliminary_outcome(&mut self, block_y: i32) -> PartialSurfaceOutcome {
        let Some(classes) = self.quarts.classes else {
            return PartialSurfaceOutcome::Evaluate;
        };
        self.uniform_group_biome(block_y)
            .and_then(|biome| classes.class(biome))
            .map_or(PartialSurfaceOutcome::Evaluate, |class| {
                classes.below_preliminary_outcome(class, block_y)
            })
    }

    /// The group's shared-class candidate biome, if its 8 candidates agree.
    fn uniform_group_biome(&mut self, block_y: i32) -> Option<u16> {
        self.quarts.classes?;
        let parent_y = (block_y - 2) >> 2;
        if parent_y != self.class_parent_y {
            self.class_parent_y = parent_y;
            self.uniform_biome =
                self.quarts
                    .uniform_group_biome(IVec3::new(self.parent_x, parent_y, self.parent_z));
        }

        self.uniform_biome
    }
}

impl SurfaceBiomeProvider for FuzzedBiomeColumn<'_, '_> {
    #[inline]
    fn biome_id(&mut self, block_y: i32) -> u16 {
        self.get_for_surface_rule(block_y)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    fn classes() -> SurfaceBiomeClasses {
        SurfaceBiomeClasses {
            class_of: Box::from([0, 0, 1]),
            below_preliminary: Box::default(),
            min_y: 0,
            height: 0,
        }
    }

    fn reference_group(quarts: &mut SurfaceQuartBiomes<'_>, parent: IVec3) -> Option<u16> {
        let classes = quarts.classes?;
        let first = quarts.biome_at(parent);
        let class = *classes.class_of.get(usize::from(first))?;

        for corner in 1..8 {
            let offset = IVec3::new((corner >> 2) & 1, (corner >> 1) & 1, corner & 1);
            let biome = quarts.biome_at(parent + offset);
            if classes.class_of.get(usize::from(biome)) != Some(&class) {
                return None;
            }
        }

        Some(first)
    }

    #[test]
    fn shared_groups_match_uncached_queries_and_neighbor_reads() {
        let classes = classes();
        let data: Vec<_> = (0..QUARTS_PER_SECTION)
            .map(|index| match index / QUARTS_PER_SECTION_PLANE {
                0 => 0,
                1 => 1,
                _ => 2,
            })
            .collect();
        let cached_reads = RefCell::new(Vec::new());
        let reference_reads = RefCell::new(Vec::new());
        let biome = |pos: IVec3| {
            if pos.y >= -2 {
                2
            } else {
                (pos.x + pos.z).rem_euclid(2) as u16
            }
        };
        let cached_neighbor = |pos| {
            cached_reads.borrow_mut().push(pos);
            biome(pos)
        };
        let reference_neighbor = |pos| {
            reference_reads.borrow_mut().push(pos);
            biome(pos)
        };
        let mut cached =
            SurfaceQuartBiomes::new(&data, Some(&classes), 1, -16, -4, -8, &cached_neighbor);
        let mut reference =
            SurfaceQuartBiomes::new(&data, Some(&classes), 1, -16, -4, -8, &reference_neighbor);

        for _ in 0..2 {
            for y in -6..=1 {
                for z in -10..=-4 {
                    for x in -6..=0 {
                        let parent = IVec3::new(x, y, z);
                        assert_eq!(
                            cached.uniform_group_biome(parent),
                            reference_group(&mut reference, parent)
                        );
                    }
                }
            }
        }

        assert!(!cached_reads.borrow().is_empty());
        assert_eq!(*cached_reads.borrow(), *reference_reads.borrow());
        assert_eq!(cached.uniform_group_biome(IVec3::new(-4, -4, -8)), Some(0));
    }

    #[test]
    fn bottom_clamped_group_stays_distinct_and_class_maps_stay_local() {
        let classes = classes();
        let other_classes = SurfaceBiomeClasses {
            class_of: Box::from([0, 0, 0]),
            ..self::classes()
        };
        let data: Vec<_> = (0..QUARTS_PER_SECTION)
            .map(|index| {
                if index < QUARTS_PER_SECTION_PLANE {
                    0
                } else {
                    2
                }
            })
            .collect();
        let neighbor = |_| 2;
        let mut cached = SurfaceQuartBiomes::new(&data, Some(&classes), 1, -16, 0, 0, &neighbor);
        let mut other =
            SurfaceQuartBiomes::new(&data, Some(&other_classes), 1, -16, 0, 0, &neighbor);

        assert_eq!(cached.uniform_group_biome(IVec3::new(0, -5, 0)), Some(0));
        assert_eq!(cached.uniform_group_biome(IVec3::new(0, -4, 0)), None);
        assert_eq!(cached.uniform_group_biome(IVec3::new(0, -1, 0)), Some(2));
        assert_eq!(other.uniform_group_biome(IVec3::new(0, -4, 0)), Some(0));
    }
}
