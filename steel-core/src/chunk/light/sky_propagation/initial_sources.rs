use super::{LightAxisDirection, LightDirectionSet};
use crate::chunk::light::{CHUNK_COLUMN_COUNT, CHUNK_EDGE, LightSectionRange};

/// only records sources actually written by the initial scan
pub(super) struct InitialSkySources {
    range: LightSectionRange,
    start_y: i32,
    pub(super) stop_ys: [i32; CHUNK_COLUMN_COUNT],
    marked_sections: Vec<bool>,
}

pub(super) struct SourceFrontier {
    pub(super) min_y: i32,
    pub(super) max_y: i32,
    horizontal_max_ys: [Option<i32>; LightAxisDirection::HORIZONTAL.len()],
    downward: bool,
}

impl InitialSkySources {
    pub(super) fn new(range: LightSectionRange, start_y: i32) -> Self {
        Self {
            range,
            start_y,
            stop_ys: [start_y; CHUNK_COLUMN_COUNT],
            marked_sections: vec![false; CHUNK_COLUMN_COUNT * range.section_count()],
        }
    }

    pub(super) fn mark_section(&mut self, column: usize, section_y: i32) {
        if let Some(section) = self.range.section_index(section_y) {
            self.marked_sections[column * self.range.section_count() + section] = true;
        }
    }

    fn marked(&self, column: usize, section: usize) -> bool {
        self.marked_sections[column * self.range.section_count() + section]
    }

    fn contains(&self, column: usize, y: i32) -> bool {
        y > self.stop_ys[column]
            && y <= self.start_y
            && self
                .range
                .section_index(y >> 4)
                .is_some_and(|section| self.marked(column, section))
    }

    pub(super) fn section_frontier(
        &self,
        x: usize,
        z: usize,
        section: usize,
    ) -> Option<SourceFrontier> {
        let column = z * CHUNK_EDGE + x;
        if !self.marked(column, section) {
            return None;
        }

        let section_min_y = (self.range.min_section_y() + section as i32) << 4;
        let min_y = section_min_y.max(self.stop_ys[column] + 1);
        let max_y = (section_min_y | 15).min(self.start_y);
        if min_y > max_y {
            return None;
        }

        let mut horizontal_max_ys = [None; LightAxisDirection::HORIZONTAL.len()];
        for (limit, direction) in horizontal_max_ys
            .iter_mut()
            .zip(LightAxisDirection::HORIZONTAL)
        {
            let (dx, _, dz) = direction.offset();
            let neighbor_x = x as i32 + dx;
            let neighbor_z = z as i32 + dz;
            let bound = if (0..CHUNK_EDGE as i32).contains(&neighbor_x)
                && (0..CHUNK_EDGE as i32).contains(&neighbor_z)
            {
                let neighbor = neighbor_z as usize * CHUNK_EDGE + neighbor_x as usize;
                if self.marked(neighbor, section) {
                    max_y.min(self.stop_ys[neighbor])
                } else {
                    max_y
                }
            } else {
                max_y
            };

            if bound >= min_y {
                *limit = Some(bound);
            }
        }

        // higher downward targets are sources in this same section
        let downward = !self.contains(column, min_y - 1);
        let max_y = horizontal_max_ys
            .iter()
            .flatten()
            .copied()
            .chain(downward.then_some(min_y))
            .max()?;

        Some(SourceFrontier {
            min_y,
            max_y,
            horizontal_max_ys,
            downward,
        })
    }
}

impl SourceFrontier {
    pub(super) fn directions(&self, y: i32) -> LightDirectionSet {
        let mut directions = LightDirectionSet::empty();
        for (direction, max_y) in LightAxisDirection::HORIZONTAL
            .into_iter()
            .zip(self.horizontal_max_ys)
        {
            if max_y.is_some_and(|max_y| y <= max_y) {
                directions = directions.with(direction);
            }
        }

        if self.downward && y == self.min_y {
            directions = directions.with(LightAxisDirection::NegativeY);
        }

        directions
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashSet;

    use super::*;

    #[test]
    fn frontier_matches_source_membership_with_slopes_holes_and_borders() {
        for (min_y, height) in [(-64, 384), (-64, 1104)] {
            let range = LightSectionRange::from_world_height(min_y, height)
                .expect("valid test light range");
            for start_y in [
                (range.max_section_y_exclusive() << 4) - 1,
                (range.max_section_y_exclusive() - 1) << 4,
            ] {
                let mut sources = InitialSkySources::new(range, start_y);
                let mut points = FxHashSet::default();
                let mut ordered_points = Vec::new();

                for z in 0..CHUNK_EDGE {
                    for x in 0..CHUNK_EDGE {
                        let column = z * CHUNK_EDGE + x;
                        let stop_y = if (x, z) == (6, 8) {
                            start_y
                        } else if (x, z) == (9, 6) {
                            (range.min_section_y() << 4) - 1
                        } else {
                            start_y - 32 - (x * 3 + z * 5) as i32 % 57
                        };
                        sources.stop_ys[column] = stop_y;

                        for y in (stop_y + 1..=start_y).rev() {
                            let section = range
                                .section_index(y >> 4)
                                .expect("source stays within the light range");
                            if (section + x + z).is_multiple_of(7) {
                                continue;
                            }

                            sources.mark_section(column, y >> 4);
                            let point = (x as i32, y, z as i32);
                            points.insert(point);
                            ordered_points.push(point);
                        }
                    }
                }

                let expected: Vec<_> = ordered_points
                    .into_iter()
                    .filter_map(|(x, y, z)| {
                        let mut directions = LightDirectionSet::empty();
                        for direction in LightAxisDirection::ALL {
                            if direction == LightAxisDirection::PositiveY {
                                continue;
                            }

                            let (dx, dy, dz) = direction.offset();
                            if !points.contains(&(x + dx, y + dy, z + dz)) {
                                directions = directions.with(direction);
                            }
                        }

                        (directions != LightDirectionSet::empty())
                            .then_some(((x, y, z), directions))
                    })
                    .collect();
                let mut actual = Vec::new();
                for z in 0..CHUNK_EDGE {
                    for x in 0..CHUNK_EDGE {
                        for section in (0..range.section_count()).rev() {
                            let Some(frontier) = sources.section_frontier(x, z, section) else {
                                continue;
                            };

                            for y in (frontier.min_y..=frontier.max_y).rev() {
                                actual.push(((x as i32, y, z as i32), frontier.directions(y)));
                            }
                        }
                    }
                }

                assert_eq!(actual, expected, "height {height}, start {start_y}");
            }
        }
    }
}
