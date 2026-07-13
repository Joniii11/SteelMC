//! Canonical Vanilla SNBT formatting
//!
//! Minecraft's `StringTagVisitor` emits a canonical SNBT representation of NBT values This module provides a formatter that produces the same output and a parser that accepts the same inputhe formatter is used to persist `NbtPredicate` values in the stable `TagParser.LENIENT_CODEC` representation and the parser is used to read those values back into memory

use std::{cmp::Ordering, fmt::Write as _};

use simdnbt::owned::{NbtCompound, NbtList, NbtTag};

pub use crate::nbt::{
    SnbtError, parse_snbt, parse_snbt_argument, parse_snbt_compound, parse_snbt_compound_argument,
};

/// Parses one complete Vanilla SNBT value
pub fn parse_vanilla_snbt(input: &str) -> Result<NbtTag, SnbtError> {
    parse_snbt(input)
}

/// Parses one complete Vanilla SNBT compound
pub fn parse_vanilla_snbt_compound(input: &str) -> Result<NbtCompound, SnbtError> {
    parse_snbt_compound(input)
}

/// Formats an NBT tag with Vanilla's canonical `StringTagVisitor` layout
#[must_use]
pub fn to_vanilla_snbt(tag: &NbtTag) -> String {
    let mut output = String::new();
    write_tag(&mut output, tag);
    output
}

/// Formats an NBT compound with Vanilla's canonical `StringTagVisitor` layout
#[must_use]
pub fn to_vanilla_snbt_compound(compound: &NbtCompound) -> String {
    let mut output = String::new();
    write_compound(&mut output, compound);
    output
}

/// Returns the logical value of an NBT list element
///
/// Vanilla stores heterogeneous binary lists as compounds with an empty key
/// `ListTag` unwraps those compounds when it reads them. `simdnbt` exposes the
/// wire representation, so callers operating on logical NBT lists must apply
/// the same unwrapping step
#[must_use]
pub fn unwrap_vanilla_list_element(tag: &NbtTag) -> &NbtTag {
    let Some(compound) = tag.compound() else {
        return tag;
    };
    if compound.len() != 1 {
        return tag;
    }

    compound.get("").unwrap_or(tag)
}

fn write_tag(output: &mut String, tag: &NbtTag) {
    match tag {
        NbtTag::Byte(value) => {
            let _ = write!(output, "{value}b");
        }
        NbtTag::Short(value) => {
            let _ = write!(output, "{value}s");
        }
        NbtTag::Int(value) => {
            let _ = write!(output, "{value}");
        }
        NbtTag::Long(value) => {
            let _ = write!(output, "{value}L");
        }
        NbtTag::Float(value) => {
            output.push_str(&format_java_float(*value));
            output.push('f');
        }
        NbtTag::Double(value) => {
            output.push_str(&format_java_double(*value));
            output.push('d');
        }
        NbtTag::ByteArray(values) => {
            output.push_str("[B;");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                let _ = write!(output, "{}B", *value as i8);
            }
            output.push(']');
        }
        NbtTag::String(value) => write_quoted_string(output, &value.to_str()),
        NbtTag::List(list) => write_list(output, list),
        NbtTag::Compound(compound) => write_compound(output, compound),
        NbtTag::IntArray(values) => {
            output.push_str("[I;");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                let _ = write!(output, "{value}");
            }
            output.push(']');
        }
        NbtTag::LongArray(values) => {
            output.push_str("[L;");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                let _ = write!(output, "{value}L");
            }
            output.push(']');
        }
    }
}

fn write_list(output: &mut String, list: &NbtList) {
    output.push('[');
    let values = list.as_nbt_tags();
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        write_tag(output, unwrap_vanilla_list_element(value));
    }
    output.push(']');
}

fn write_compound(output: &mut String, compound: &NbtCompound) {
    let mut entries = Vec::with_capacity(compound.len());
    for (key, value) in compound.iter() {
        let key = key.to_str().into_owned();
        if let Some((_, previous)) = entries.iter_mut().find(|(previous, _)| previous == &key) {
            *previous = value;
        } else {
            entries.push((key, value));
        }
    }
    entries.sort_by(|(left, _), (right, _)| compare_java_strings(left, right));

    output.push('{');
    for (index, (key, value)) in entries.into_iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        if is_unquoted_key(&key) {
            output.push_str(&key);
        } else {
            write_quoted_string(output, &key);
        }
        output.push(':');
        write_tag(output, value);
    }
    output.push('}');
}

fn compare_java_strings(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

fn is_unquoted_key(value: &str) -> bool {
    if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        return false;
    }

    let bytes = value.as_bytes();
    let Some((&first, rest)) = bytes.split_first() else {
        return false;
    };
    if !matches!(first, b'A'..=b'Z' | b'a'..=b'z' | b'.' | b'_') {
        return false;
    }
    rest.iter().all(
        |byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'+' | b'-'),
    )
}

fn write_quoted_string(output: &mut String, value: &str) {
    let quote = value
        .chars()
        .find_map(|character| match character {
            '"' => Some('\''),
            '\'' => Some('"'),
            _ => None,
        })
        .unwrap_or('"');
    output.push(quote);

    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\"' | '\'' if character == quote => {
                output.push('\\');
                output.push(character);
            }
            '\u{0008}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{000C}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            character if character < ' ' => {
                let _ = write!(output, "\\x{:02X}", character as u32);
            }
            _ => output.push(character),
        }
    }

    output.push(quote);
}

fn format_java_float(value: f32) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f32::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f32::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    format_java_finite(&format!("{value:?}"))
}

fn format_java_double(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    format_java_finite(&format!("{value:?}"))
}

/// Converts Rust's shortest finite representation to Java's
/// `FloatingDecimal.toJavaFormatString` layout. Both runtimes use shortest
/// round-trippable digits; Java switches to scientific notation below 1E-3
/// and at 1E7, and always retains a decimal digit for integral floats.
fn format_java_finite(shortest: &str) -> String {
    let (negative, shortest) = shortest
        .strip_prefix('-')
        .map_or((false, shortest), |value| (true, value));
    if shortest == "0.0" || shortest == "0" {
        return if negative {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        };
    }

    let exponent_marker = shortest.find(['e', 'E']);
    let (mantissa, source_exponent) = match exponent_marker {
        Some(index) => {
            let Ok(exponent) = shortest[index + 1..].parse::<i32>() else {
                unreachable!("Rust's finite float formatter must emit an integer exponent after E");
            };
            (&shortest[..index], exponent)
        }
        None => (shortest, 0),
    };
    let decimal_position = mantissa.find('.').unwrap_or(mantissa.len());
    let mut digits = mantissa.replace('.', "");
    let leading_zeroes = digits.bytes().take_while(|byte| *byte == b'0').count();
    digits.drain(..leading_zeroes);
    while digits.ends_with('0') {
        digits.pop();
    }
    if digits.is_empty() {
        return if negative {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        };
    }

    let exponent = source_exponent + decimal_position as i32 - 1 - leading_zeroes as i32;
    let mut output = String::new();
    if negative {
        output.push('-');
    }

    if !(-3..7).contains(&exponent) {
        output.push(digits.as_bytes()[0] as char);
        output.push('.');
        if digits.len() == 1 {
            output.push('0');
        } else {
            output.push_str(&digits[1..]);
        }
        let _ = write!(output, "E{exponent}");
        return output;
    }

    let decimal_position = (exponent + 1) as usize;
    if exponent < 0 {
        output.push_str("0.");
        for _ in 0..(-exponent - 1) {
            output.push('0');
        }
        output.push_str(&digits);
    } else if decimal_position >= digits.len() {
        output.push_str(&digits);
        for _ in 0..decimal_position - digits.len() {
            output.push('0');
        }
        output.push_str(".0");
    } else {
        output.push_str(&digits[..decimal_position]);
        output.push('.');
        output.push_str(&digits[decimal_position..]);
    }
    output
}

#[cfg(test)]
mod tests {
    use simdnbt::owned::{NbtCompound, NbtList, NbtTag};

    use super::{parse_vanilla_snbt_compound, to_vanilla_snbt_compound};

    #[test]
    fn formats_like_vanilla_string_tag_visitor() {
        let mut compound = NbtCompound::new();
        compound.insert("z", NbtTag::Float(10_000_000.0));
        compound.insert("true", NbtTag::String("a'\"b\\\n".into()));
        compound.insert("a", NbtTag::ByteArray(vec![255, 1]));
        compound.insert("list", NbtTag::List(NbtList::Long(vec![1, -2])));

        assert_eq!(
            to_vanilla_snbt_compound(&compound),
            r#"{a:[B;-1B,1B],list:[1L,-2L],"true":"a'\"b\\\n",z:1.0E7f}"#
        );
    }

    #[test]
    fn parses_vanilla_byte_suffixes_before_binary_prefixes() {
        let compound =
            parse_vanilla_snbt_compound("{byte:0b,binary:0b10}").expect("valid Vanilla SNBT");

        assert_eq!(compound.byte("byte"), Some(0));
        assert_eq!(compound.int("binary"), Some(2));
    }
}
