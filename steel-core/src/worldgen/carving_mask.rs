//! Transient output of direct world carvers for one chunk-generation stage.
//!
//! Mirrors Snapshot-2's `net.minecraft.world.level.chunk.CarvingMask`.
//! Direct carvers mark geometry first; `NoiseBasedChunkGenerator` then visits
//! the marked ranges to apply aquifer and top-material behavior.

/// A `16 x height x 16` bitset of local block positions in a chunk.
#[derive(Debug, Clone)]
pub struct CarvingMask {
    min_y: i32,
    max_y: i32,
    height: i32,
    /// Vanilla orders bits by `(x, z)` column, then Y within that column.
    bits: Vec<u64>,
}

impl CarvingMask {
    /// Creates an empty mask for the inclusive Y range `[min_y, max_y]`.
    #[must_use]
    pub fn new(min_y: i32, max_y: i32) -> Self {
        assert!(min_y <= max_y, "carving mask must have a non-empty Y range");
        let height = max_y - min_y + 1;
        let total_bits = (256 * height) as usize;
        let lanes = total_bits.div_ceil(64);
        Self {
            min_y,
            max_y,
            height,
            bits: vec![0; lanes],
        }
    }

    /// Vanilla's `getIndex`: `y - minY + (z + (x << 4)) * height`.
    #[inline]
    const fn index(&self, x: i32, y: i32, z: i32) -> usize {
        ((y - self.min_y) + (z + (x << 4)) * self.height) as usize
    }

    /// Marks `(x, y, z)` as carved.
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, z: i32) {
        debug_assert!((0..16).contains(&x));
        debug_assert!((0..16).contains(&z));
        debug_assert!((self.min_y..=self.max_y).contains(&y));
        let index = self.index(x, y, z);
        self.bits[index / 64] |= 1_u64 << (index % 64);
    }

    /// Returns whether `(x, y, z)` has been marked.
    #[inline]
    #[must_use]
    pub fn get(&self, x: i32, y: i32, z: i32) -> bool {
        debug_assert!((0..16).contains(&x));
        debug_assert!((0..16).contains(&z));
        debug_assert!((self.min_y..=self.max_y).contains(&y));
        let index = self.index(x, y, z);
        (self.bits[index / 64] >> (index % 64)) & 1 != 0
    }

    /// Inclusive lower Y bound.
    #[must_use]
    pub const fn min_y(&self) -> i32 {
        self.min_y
    }

    /// Inclusive upper Y bound.
    #[must_use]
    pub const fn max_y(&self) -> i32 {
        self.max_y
    }

    /// Returns whether no carver marked any position.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&word| word == 0)
    }

    /// Visits marked ranges with the same segment splitting and column order
    /// as vanilla's `CarvingMask.visit`.
    pub fn visit(&self, mut visitor: impl FnMut(i32, i32, i32, i32)) {
        let Some(mut start_index) = self.next_set_bit(0) else {
            return;
        };

        loop {
            let end_index = self.next_clear_bit(start_index) - 1;
            self.visit_segment(&mut visitor, start_index, end_index);

            let Some(next_start) = self.next_set_bit(end_index + 1) else {
                return;
            };
            start_index = next_start;
        }
    }

    fn visit_segment(
        &self,
        visitor: &mut impl FnMut(i32, i32, i32, i32),
        start_index: usize,
        end_index: usize,
    ) {
        let height = self.height as usize;
        let start_column = start_index / height;
        let end_column = end_index / height;

        for column in start_column..=end_column {
            let column_x = ((column >> 4) & 15) as i32;
            let column_z = (column & 15) as i32;
            let column_base_index = column * height;
            let bottom_y =
                (start_index.saturating_sub(column_base_index)).min(height - 1) as i32 + self.min_y;
            let top_y =
                end_index.saturating_sub(column_base_index).min(height - 1) as i32 + self.min_y;
            visitor(column_x, column_z, bottom_y, top_y);
        }
    }

    fn next_set_bit(&self, from: usize) -> Option<usize> {
        let bit_len = self.bits.len() * 64;
        if from >= bit_len {
            return None;
        }

        let mut word_index = from / 64;
        let mut word = self.bits[word_index] & (u64::MAX << (from % 64));
        loop {
            if word != 0 {
                return Some(word_index * 64 + word.trailing_zeros() as usize);
            }
            word_index += 1;
            if word_index == self.bits.len() {
                return None;
            }
            word = self.bits[word_index];
        }
    }

    fn next_clear_bit(&self, from: usize) -> usize {
        let bit_len = self.bits.len() * 64;
        if from >= bit_len {
            return from;
        }

        let mut word_index = from / 64;
        let mut word = !self.bits[word_index] & (u64::MAX << (from % 64));
        loop {
            if word != 0 {
                return word_index * 64 + word.trailing_zeros() as usize;
            }
            word_index += 1;
            if word_index == self.bits.len() {
                return bit_len;
            }
            word = !self.bits[word_index];
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_and_get_roundtrip() {
        let mut mask = CarvingMask::new(-64, 319);
        assert!(!mask.get(5, 10, 7));
        mask.set(5, 10, 7);
        assert!(mask.get(5, 10, 7));
        assert!(!mask.get(4, 10, 7));
        assert!(!mask.get(5, 11, 7));
        assert!(!mask.get(5, 10, 8));
    }

    #[test]
    fn indexing_matches_snapshot_two_layout() {
        let mask = CarvingMask::new(-64, 319);
        assert_eq!(mask.index(0, -64, 0), 0);
        assert_eq!(mask.index(15, -64, 0), 92_160);
        assert_eq!(mask.index(0, -64, 1), 384);
        assert_eq!(mask.index(0, -63, 0), 1);
    }

    #[test]
    fn visit_matches_vanilla_segment_splitting() {
        let mut mask = CarvingMask::new(-4, 3);
        mask.set(0, 3, 0);
        mask.set(0, -4, 1);
        mask.set(1, -3, 2);
        mask.set(1, -2, 2);
        mask.set(1, 0, 2);

        let mut visited = Vec::new();
        mask.visit(|x, z, bottom, top| visited.push((x, z, bottom, top)));

        assert_eq!(
            visited,
            vec![(0, 0, 3, 3), (0, 1, -4, -4), (1, 2, -3, -2), (1, 2, 0, 0),]
        );
    }
}
