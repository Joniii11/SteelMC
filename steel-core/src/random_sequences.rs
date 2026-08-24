//! Server global `RandomSequences`

use std::io;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use steel_registry::loot_table::VanillaLootRandomSource;
use steel_utils::{
    Identifier,
    locks::SyncMutex,
    random::{Random, name_hash::NameHash, xoroshiro::Xoroshiro},
    saved_data::{SavedDataManager, names as saved_data_names},
};

/// Persistent named random sources
pub struct RandomSequences {
    saved_data: SavedDataManager,
    world_seed: i64,
    state: SyncMutex<RandomSequencesState>,
}

struct RandomSequencesState {
    salt: i32,
    include_world_seed: bool,
    include_sequence_id: bool,
    sequences: FxHashMap<Identifier, Xoroshiro>,
    dirty: bool,
}

/// Dirty tracking sequence borrow
pub struct RandomSequence<'a> {
    source: &'a mut Xoroshiro,
    used: bool,
}

impl RandomSequence<'_> {
    fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        self.used = true;
        Random::next_i32_bounded(self.source, bound)
    }

    fn next_f32(&mut self) -> f32 {
        self.used = true;
        Random::next_f32(self.source)
    }
}

impl VanillaLootRandomSource for RandomSequence<'_> {
    fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        RandomSequence::next_i32_bounded(self, bound)
    }

    fn next_f32(&mut self) -> f32 {
        RandomSequence::next_f32(self)
    }
}

impl RandomSequences {
    /// Ephemeral sequences
    #[cfg(test)]
    #[must_use]
    fn ephemeral(world_seed: i64) -> Self {
        Self {
            saved_data: SavedDataManager::new(None),
            world_seed,
            state: SyncMutex::new(RandomSequencesState {
                salt: 0,
                include_world_seed: true,
                include_sequence_id: true,
                sequences: FxHashMap::default(),
                dirty: false,
            }),
        }
    }

    /// Loads saved sequences
    pub async fn load(saved_data: SavedDataManager, world_seed: i64) -> io::Result<Self> {
        let persisted: PersistentRandomSequences = saved_data
            .load_or_default(saved_data_names::RANDOM_SEQUENCES)
            .await?;
        let sequences = persisted
            .sequences
            .into_iter()
            .map(|(key, sequence)| {
                (
                    key,
                    Xoroshiro::from_state(sequence.source[0] as u64, sequence.source[1] as u64),
                )
            })
            .collect();

        Ok(Self {
            saved_data,
            world_seed,
            state: SyncMutex::new(RandomSequencesState {
                salt: persisted.salt,
                include_world_seed: persisted.include_world_seed,
                include_sequence_id: persisted.include_sequence_id,
                sequences,
                dirty: false,
            }),
        })
    }

    /// Uses a named sequence
    pub fn with_sequence<T>(
        &self,
        key: &Identifier,
        callback: impl FnOnce(&mut RandomSequence<'_>) -> T,
    ) -> T {
        let mut state = self.state.lock();
        let salt = state.salt;
        let include_world_seed = state.include_world_seed;
        let include_sequence_id = state.include_sequence_id;
        let source = state.sequences.entry(key.clone()).or_insert_with(|| {
            create_sequence(
                key,
                self.world_seed,
                salt,
                include_world_seed,
                include_sequence_id,
            )
        });
        let mut sequence = RandomSequence {
            source,
            used: false,
        };
        let result = callback(&mut sequence);
        if sequence.used {
            state.dirty = true;
        }
        result
    }

    /// Saves dirty sequences
    pub async fn save(&self) -> io::Result<()> {
        let persisted = {
            let mut state = self.state.lock();
            if !state.dirty {
                return Ok(());
            }
            state.dirty = false;
            PersistentRandomSequences::from(&*state)
        };

        if let Err(error) = self
            .saved_data
            .save(saved_data_names::RANDOM_SEQUENCES, &persisted)
            .await
        {
            self.state.lock().dirty = true;
            return Err(error);
        }

        Ok(())
    }
}

fn create_sequence(
    key: &Identifier,
    world_seed: i64,
    salt: i32,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Xoroshiro {
    let seed = (if include_world_seed { world_seed } else { 0 }) ^ i64::from(salt);
    if !include_sequence_id {
        return Xoroshiro::from_seed(seed as u64);
    }

    let hash = NameHash::from_name(&key.to_string());
    Xoroshiro::from_seed_and_hash(seed as u64, hash.md5[0], hash.md5[1])
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct PersistentRandomSequences {
    salt: i32,
    #[serde(default = "default_true")]
    include_world_seed: bool,
    #[serde(default = "default_true")]
    include_sequence_id: bool,
    sequences: FxHashMap<Identifier, PersistentRandomSequence>,
}

impl Default for PersistentRandomSequences {
    fn default() -> Self {
        Self {
            salt: 0,
            include_world_seed: true,
            include_sequence_id: true,
            sequences: FxHashMap::default(),
        }
    }
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize)]
struct PersistentRandomSequence {
    /// Xoroshiro state words
    source: [i64; 2],
}

impl From<&RandomSequencesState> for PersistentRandomSequences {
    fn from(state: &RandomSequencesState) -> Self {
        let sequences = state
            .sequences
            .iter()
            .map(|(key, source)| {
                let [seed_lo, seed_hi] = source.state();
                (
                    key.clone(),
                    PersistentRandomSequence {
                        source: [seed_lo as i64, seed_hi as i64],
                    },
                )
            })
            .collect();

        Self {
            salt: state.salt,
            include_world_seed: state.include_world_seed,
            include_sequence_id: state.include_sequence_id,
            sequences,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env::temp_dir,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{RandomSequences, create_sequence};
    use steel_utils::{Identifier, random::Random, saved_data::SavedDataManager};

    fn temp_world_dir(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        temp_dir().join(format!("steel-random-sequences-{test_name}-{unique}"))
    }

    #[test]
    fn sequence_key_changes_the_xoroshiro_source() {
        let mut first = create_sequence(
            &Identifier::vanilla_static("blocks/oak_log"),
            12_345,
            0,
            true,
            true,
        );
        let mut second = create_sequence(
            &Identifier::vanilla_static("blocks/birch_log"),
            12_345,
            0,
            true,
            true,
        );

        assert_ne!(Random::next_i64(&mut first), Random::next_i64(&mut second));
    }

    #[test]
    fn sequence_defaults_include_the_world_seed_and_identifier() {
        let key = Identifier::vanilla_static("blocks/oak_log");
        let mut with_defaults = create_sequence(&key, 12_345, 0, true, true);
        let mut without_world_seed = create_sequence(&key, 12_345, 0, false, true);
        let mut without_identifier = create_sequence(&key, 12_345, 0, true, false);

        assert_ne!(
            Random::next_i64(&mut with_defaults),
            Random::next_i64(&mut without_world_seed)
        );
        assert_ne!(
            Random::next_i64(&mut with_defaults),
            Random::next_i64(&mut without_identifier)
        );
    }

    #[test]
    fn only_rng_draws_mark_sequence_data_dirty() {
        let sequences = RandomSequences::ephemeral(12_345);
        let key = Identifier::vanilla_static("till/rooted_dirt");

        sequences.with_sequence(&key, |_| {});
        assert!(!sequences.state.lock().dirty);

        sequences.with_sequence(&key, |sequence| {
            let _ = sequence.next_i32_bounded(10);
        });
        assert!(sequences.state.lock().dirty);
    }

    #[tokio::test]
    async fn advanced_sequence_resumes_from_persisted_state() {
        let world_dir = temp_world_dir("round-trip");
        let key = Identifier::vanilla_static("till/rooted_dirt");
        let sequences =
            RandomSequences::load(SavedDataManager::new(Some(world_dir.as_path())), 12_345)
                .await
                .expect("random sequences should load");

        sequences.with_sequence(&key, |sequence| {
            let _ = sequence.next_i32_bounded(10_000);
            let _ = sequence.next_f32();
        });
        sequences
            .save()
            .await
            .expect("random sequences should save");

        let expected = sequences.with_sequence(&key, |sequence| sequence.next_i32_bounded(10_000));
        let resumed =
            RandomSequences::load(SavedDataManager::new(Some(world_dir.as_path())), 12_345)
                .await
                .expect("persisted random sequences should load");
        let actual = resumed.with_sequence(&key, |sequence| sequence.next_i32_bounded(10_000));

        assert_eq!(actual, expected);
    }
}
