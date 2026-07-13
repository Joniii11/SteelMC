//! Vanilla compatible NBT helpers

mod snbt;
mod stream;

use simdnbt::owned::{NbtCompound, NbtList, NbtTag};

pub use snbt::{
    SnbtError, parse_snbt, parse_snbt_argument, parse_snbt_compound, parse_snbt_compound_argument,
};
pub use stream::{
    DEFAULT_NBT_QUOTA, MAX_NBT_DEPTH, read_optional_tag_with_default_quota,
    read_tag_with_default_quota,
};

/// Compares two NBT tags with vanilla partial compound and list semantics
#[must_use]
pub fn compare_nbt(
    expected: Option<&NbtTag>,
    actual: Option<&NbtTag>,
    partial_list_matches: bool,
) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    let Some(actual) = actual else {
        return false;
    };

    match (expected, actual) {
        (NbtTag::Compound(expected), NbtTag::Compound(actual)) => {
            compare_nbt_compounds(expected, actual, partial_list_matches)
        }
        (NbtTag::List(expected), NbtTag::List(actual)) if partial_list_matches => {
            compare_lists_partially(expected, actual)
        }
        _ => tags_equal_like_vanilla(expected, actual),
    }
}

/// Compares two compounds with vanilla partial compound semantics
#[must_use]
pub fn compare_nbt_compounds(
    expected: &NbtCompound,
    actual: &NbtCompound,
    partial_list_matches: bool,
) -> bool {
    compound_last_entries(expected)
        .into_iter()
        .all(|(key, expected_tag)| {
            compare_nbt(
                Some(expected_tag),
                compound_last_value(actual, key),
                partial_list_matches,
            )
        })
}

fn compound_last_entries(compound: &NbtCompound) -> Vec<(&simdnbt::Mutf8Str, &NbtTag)> {
    let entries = compound.iter().collect::<Vec<_>>();
    let mut last_entries = Vec::with_capacity(entries.len());

    for (index, entry) in entries.iter().enumerate() {
        if !entries[index + 1..]
            .iter()
            .any(|(later_key, _)| *later_key == entry.0)
        {
            last_entries.push(*entry);
        }
    }

    last_entries
}

fn compound_last_value<'a>(
    compound: &'a NbtCompound,
    key: &simdnbt::Mutf8Str,
) -> Option<&'a NbtTag> {
    let mut result = None;

    for (candidate, value) in compound.iter() {
        if candidate == key {
            result = Some(value);
        }
    }

    result
}

fn tags_equal_like_vanilla(expected: &NbtTag, actual: &NbtTag) -> bool {
    match (expected, actual) {
        (NbtTag::Float(expected), NbtTag::Float(actual)) => {
            (expected.is_nan() && actual.is_nan()) || expected.to_bits() == actual.to_bits()
        }
        (NbtTag::Double(expected), NbtTag::Double(actual)) => {
            (expected.is_nan() && actual.is_nan()) || expected.to_bits() == actual.to_bits()
        }
        (NbtTag::List(expected), NbtTag::List(actual)) => {
            let expected = list_as_tags(expected);
            let actual = list_as_tags(actual);
            expected.len() == actual.len()
                && expected
                    .iter()
                    .zip(&actual)
                    .all(|(expected, actual)| compare_nbt(Some(expected), Some(actual), false))
        }
        (NbtTag::Compound(expected), NbtTag::Compound(actual)) => {
            compare_nbt_compounds(expected, actual, false)
        }
        _ => expected == actual,
    }
}

fn compare_lists_partially(expected: &NbtList, actual: &NbtList) -> bool {
    let expected = list_as_tags(expected);
    let actual = list_as_tags(actual);
    if expected.is_empty() {
        return actual.is_empty();
    }
    if actual.len() < expected.len() {
        return false;
    }

    expected.iter().all(|expected_tag| {
        actual
            .iter()
            .any(|actual_tag| compare_nbt(Some(expected_tag), Some(actual_tag), true))
    })
}

fn list_as_tags(list: &NbtList) -> Vec<NbtTag> {
    list.as_nbt_tags()
        .into_iter()
        .map(unwrap_list_wrapper)
        .collect()
}

fn unwrap_list_wrapper(tag: NbtTag) -> NbtTag {
    match tag {
        NbtTag::Compound(mut compound) if compound.len() == 1 && compound.contains("") => {
            let Some(value) = compound.take("") else {
                return NbtTag::Compound(compound);
            };
            value
        }
        tag => tag,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use simdnbt::owned::read_tag;

    use super::*;

    fn compound(entries: impl IntoIterator<Item = (&'static str, NbtTag)>) -> NbtTag {
        let mut compound = NbtCompound::new();
        for (key, tag) in entries {
            compound.insert(key, tag);
        }
        NbtTag::Compound(compound)
    }

    fn list(entries: impl IntoIterator<Item = NbtTag>) -> NbtTag {
        NbtTag::List(NbtList::from(entries.into_iter().collect::<Vec<_>>()))
    }

    #[test]
    fn compounds_and_lists_match_partially() {
        let expected = compound([(
            "values",
            list([compound([("name", NbtTag::String("second".into()))])]),
        )]);
        let actual = compound([(
            "values",
            list([
                compound([("name", NbtTag::String("first".into()))]),
                compound([
                    ("name", NbtTag::String("second".into())),
                    ("extra", NbtTag::Byte(1)),
                ]),
            ]),
        )]);

        assert!(compare_nbt(Some(&expected), Some(&actual), true));
        assert!(!compare_nbt(Some(&expected), Some(&actual), false));
    }

    #[test]
    fn empty_partial_list_only_matches_an_empty_list() {
        let empty = list([]);
        let non_empty = list([NbtTag::Int(1)]);

        assert!(compare_nbt(Some(&empty), Some(&empty), true));
        assert!(!compare_nbt(Some(&empty), Some(&non_empty), true));
    }

    #[test]
    fn partial_lists_match_heterogeneous_values() {
        let expected = list([NbtTag::String("two".into())]);
        let actual = list([NbtTag::Int(1), NbtTag::String("two".into())]);

        assert!(compare_nbt(Some(&expected), Some(&actual), true));
    }

    #[test]
    fn scalar_tags_require_the_same_nbt_type() {
        assert!(compare_nbt(
            Some(&NbtTag::Int(1)),
            Some(&NbtTag::Int(1)),
            true
        ));
        assert!(!compare_nbt(
            Some(&NbtTag::Int(1)),
            Some(&NbtTag::Long(1)),
            true
        ));
    }

    #[test]
    fn floating_tags_use_java_record_equality() {
        let float_nan = NbtTag::Float(f32::from_bits(0x7FC0_0000));
        let alternate_float_nan = NbtTag::Float(f32::from_bits(0x7FA0_0001));
        assert!(compare_nbt(
            Some(&float_nan),
            Some(&alternate_float_nan),
            false
        ));
        assert!(!compare_nbt(
            Some(&NbtTag::Float(-0.0)),
            Some(&NbtTag::Float(0.0)),
            false
        ));

        let double_nan = NbtTag::Double(f64::from_bits(0x7FF8_0000_0000_0000));
        let alternate_double_nan = NbtTag::Double(f64::from_bits(0x7FF0_0000_0000_0001));
        assert!(compare_nbt(
            Some(&double_nan),
            Some(&alternate_double_nan),
            false
        ));
        assert!(!compare_nbt(
            Some(&NbtTag::Double(-0.0)),
            Some(&NbtTag::Double(0.0)),
            false
        ));
    }

    #[test]
    fn binary_compound_duplicate_keys_use_the_last_value_like_vanilla() {
        let mut expected_compound = NbtCompound::new();
        expected_compound.insert("foo", NbtTag::Int(1));
        expected_compound.insert("foo", NbtTag::Int(2));
        let expected = NbtTag::Compound(expected_compound);

        let mut encoded = Vec::new();
        expected.write(&mut encoded);
        let expected = read_tag(&mut Cursor::new(encoded.as_slice()))
            .expect("duplicate-key fixture must decode as binary NBT");
        let NbtTag::Compound(parsed_expected) = &expected else {
            panic!("duplicate-key fixture must remain a compound");
        };
        assert_eq!(
            parsed_expected.len(),
            2,
            "simdnbt preserves wire duplicates"
        );

        let mut actual_compound = NbtCompound::new();
        actual_compound.insert("foo", NbtTag::Int(2));
        let actual = NbtTag::Compound(actual_compound);

        assert!(compare_nbt(Some(&expected), Some(&actual), false));
    }
}
