//! Vanilla-accounted binary NBT readers for protocol payloads
//!
//! `simdnbt` is deliberately optimized for trusted data. Its owned parser
//! preserves duplicate compound entries and raw MUTF-8 bytes, while Vanilla
//! replaces duplicate keys and materializes Java strings. This module decodes
//! directly into the owned representation while applying `NbtAccounter`

use std::io::{Cursor, Error, ErrorKind, Result};

use rustc_hash::FxHashMap;
use simdnbt::{
    Mutf8String,
    owned::{NbtCompound, NbtList, NbtTag},
};

/// Vanilla's `NbtAccounter.DEFAULT_NBT_QUOTA`
pub const DEFAULT_NBT_QUOTA: u64 = 2_097_152;

/// Vanilla's `NbtAccounter.MAX_STACK_DEPTH`
pub const MAX_NBT_DEPTH: usize = 512;

const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_SHORT: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_LONG: u8 = 4;
const TAG_FLOAT: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_BYTE_ARRAY: u8 = 7;
const TAG_STRING: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_COMPOUND: u8 = 10;
const TAG_INT_ARRAY: u8 = 11;
const TAG_LONG_ARRAY: u8 = 12;

/// Reads a protocol NBT root with Vanilla default accounting quota
pub fn read_optional_tag_with_default_quota(data: &mut Cursor<&[u8]>) -> Result<Option<NbtTag>> {
    read_optional_tag_with_quota(data, DEFAULT_NBT_QUOTA)
}

/// Reads a required protocol NBT root with Vanilla default accounting quota
pub fn read_tag_with_default_quota(data: &mut Cursor<&[u8]>) -> Result<NbtTag> {
    read_optional_tag_with_default_quota(data)?.ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "expected a non-null NBT tag, found root TAG_End",
        )
    })
}

fn read_optional_tag_with_quota(data: &mut Cursor<&[u8]>, quota: u64) -> Result<Option<NbtTag>> {
    let input = *data.get_ref();
    let start = usize::try_from(data.position()).map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            "NBT cursor position does not fit usize",
        )
    })?;
    let remaining = input.get(start..).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "NBT cursor position exceeds its input",
        )
    })?;

    let mut decoder = NbtDecoder::new(remaining, quota);
    let tag = decoder.read_root()?;
    let consumed = decoder.position();
    let end = start.checked_add(consumed).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "NBT payload length overflows input position",
        )
    })?;

    data.set_position(u64::try_from(end).map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            "NBT payload end does not fit cursor position",
        )
    })?);
    Ok(tag)
}

struct NbtDecoder<'a> {
    input: &'a [u8],
    position: usize,
    usage: u64,
    quota: u64,
}

enum DecodeNode {
    Tag(NbtTag),
    Frame(NbtFrame),
}

enum NbtFrame {
    List(NbtListFrame),
    Compound(NbtCompoundFrame),
}

struct NbtListFrame {
    element_type: u8,
    depth: usize,
    remaining: usize,
    values: NbtListValues,
}

enum NbtListValues {
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    ByteArray(Vec<Vec<u8>>),
    String(Vec<Mutf8String>),
    List(Vec<NbtList>),
    Compound(Vec<NbtCompound>),
    IntArray(Vec<Vec<i32>>),
    LongArray(Vec<Vec<i64>>),
}

struct NbtCompoundFrame {
    depth: usize,
    key_indices: FxHashMap<Vec<u16>, usize>,
    values: Vec<(Mutf8String, NbtTag)>,
    pending_key: Option<(Vec<u16>, Mutf8String)>,
}

enum FrameAdvance {
    Child { tag_type: u8, depth: usize },
    Complete,
}

impl NbtFrame {
    fn next_child(&mut self, decoder: &mut NbtDecoder<'_>) -> Result<FrameAdvance> {
        match self {
            Self::List(frame) => {
                if frame.remaining == 0 {
                    return Ok(FrameAdvance::Complete);
                }

                frame.remaining -= 1;
                Ok(FrameAdvance::Child {
                    tag_type: frame.element_type,
                    depth: frame.depth + 1,
                })
            }
            Self::Compound(frame) => {
                if frame.pending_key.is_some() {
                    return Err(invalid_data(
                        "NBT decoder lost a compound entry before its value was decoded",
                    ));
                }

                let tag_type = decoder.read_u8()?;
                if tag_type == TAG_END {
                    return Ok(FrameAdvance::Complete);
                }

                let key_code_units = decoder.read_modified_utf8()?;
                decoder.account(28)?;
                decoder.account_bytes_per_entry(2, key_code_units.len())?;
                let key = canonical_mutf8(&key_code_units);
                frame.pending_key = Some((key_code_units, key));
                Ok(FrameAdvance::Child {
                    tag_type,
                    depth: frame.depth + 1,
                })
            }
        }
    }

    fn push_child(&mut self, decoder: &mut NbtDecoder<'_>, tag: NbtTag) -> Result<()> {
        match self {
            Self::List(frame) => frame.values.push(tag),
            Self::Compound(frame) => {
                let Some((key_code_units, key)) = frame.pending_key.take() else {
                    return Err(invalid_data(
                        "NBT decoder received a compound value without its key",
                    ));
                };

                if let Some(&index) = frame.key_indices.get(&key_code_units) {
                    frame.values[index] = (key, tag);
                } else {
                    decoder.account(36)?;
                    frame.key_indices.insert(key_code_units, frame.values.len());
                    frame.values.push((key, tag));
                }
                Ok(())
            }
        }
    }

    fn finish(self) -> Result<NbtTag> {
        match self {
            Self::List(frame) => Ok(NbtTag::List(frame.values.finish())),
            Self::Compound(frame) => {
                if frame.pending_key.is_some() {
                    return Err(invalid_data(
                        "NBT decoder completed a compound with a missing value",
                    ));
                }
                Ok(NbtTag::Compound(NbtCompound::from_values(frame.values)))
            }
        }
    }
}

impl NbtListValues {
    fn new(element_type: u8, count: usize) -> Result<Self> {
        match element_type {
            TAG_BYTE => Ok(Self::Byte(Vec::with_capacity(count))),
            TAG_SHORT => Ok(Self::Short(Vec::with_capacity(count))),
            TAG_INT => Ok(Self::Int(Vec::with_capacity(count))),
            TAG_LONG => Ok(Self::Long(Vec::with_capacity(count))),
            TAG_FLOAT => Ok(Self::Float(Vec::with_capacity(count))),
            TAG_DOUBLE => Ok(Self::Double(Vec::with_capacity(count))),
            TAG_BYTE_ARRAY => Ok(Self::ByteArray(Vec::with_capacity(count))),
            TAG_STRING => Ok(Self::String(Vec::with_capacity(count))),
            TAG_LIST => Ok(Self::List(Vec::with_capacity(count))),
            TAG_COMPOUND => Ok(Self::Compound(Vec::with_capacity(count))),
            TAG_INT_ARRAY => Ok(Self::IntArray(Vec::with_capacity(count))),
            TAG_LONG_ARRAY => Ok(Self::LongArray(Vec::with_capacity(count))),
            _ => Err(invalid_data(format!(
                "unknown NBT list element type: {element_type}"
            ))),
        }
    }

    fn push(&mut self, tag: NbtTag) -> Result<()> {
        match (self, tag) {
            (Self::Byte(values), NbtTag::Byte(value)) => values.push(value),
            (Self::Short(values), NbtTag::Short(value)) => values.push(value),
            (Self::Int(values), NbtTag::Int(value)) => values.push(value),
            (Self::Long(values), NbtTag::Long(value)) => values.push(value),
            (Self::Float(values), NbtTag::Float(value)) => values.push(value),
            (Self::Double(values), NbtTag::Double(value)) => values.push(value),
            (Self::ByteArray(values), NbtTag::ByteArray(value)) => values.push(value),
            (Self::String(values), NbtTag::String(value)) => values.push(value),
            (Self::List(values), NbtTag::List(value)) => values.push(value),
            (Self::Compound(values), NbtTag::Compound(value)) => values.push(value),
            (Self::IntArray(values), NbtTag::IntArray(value)) => values.push(value),
            (Self::LongArray(values), NbtTag::LongArray(value)) => values.push(value),
            _ => {
                return Err(invalid_data(
                    "NBT list element type changed during decoding",
                ));
            }
        }
        Ok(())
    }

    fn finish(self) -> NbtList {
        match self {
            Self::Byte(values) => NbtList::Byte(values),
            Self::Short(values) => NbtList::Short(values),
            Self::Int(values) => NbtList::Int(values),
            Self::Long(values) => NbtList::Long(values),
            Self::Float(values) => NbtList::Float(values),
            Self::Double(values) => NbtList::Double(values),
            Self::ByteArray(values) => NbtList::ByteArray(values),
            Self::String(values) => NbtList::String(values),
            Self::List(values) => NbtList::List(values),
            Self::Compound(values) => NbtList::Compound(values),
            Self::IntArray(values) => NbtList::IntArray(values),
            Self::LongArray(values) => NbtList::LongArray(values),
        }
    }
}

impl<'a> NbtDecoder<'a> {
    const fn new(input: &'a [u8], quota: u64) -> Self {
        Self {
            input,
            position: 0,
            usage: 0,
            quota,
        }
    }

    const fn position(&self) -> usize {
        self.position
    }

    fn read_root(&mut self) -> Result<Option<NbtTag>> {
        let tag_type = self.read_u8()?;
        if tag_type == TAG_END {
            return Ok(None);
        }

        self.read_tag(tag_type, 0).map(Some)
    }

    fn read_tag(&mut self, tag_type: u8, depth: usize) -> Result<NbtTag> {
        let mut frames: Vec<NbtFrame> = Vec::new();
        let mut next = Some((tag_type, depth));
        let mut completed = None;

        loop {
            if let Some(tag) = completed.take() {
                let Some(mut parent) = frames.pop() else {
                    return Ok(tag);
                };
                parent.push_child(self, tag)?;
                frames.push(parent);
                continue;
            }

            if let Some((tag_type, depth)) = next.take() {
                match self.start_tag(tag_type, depth)? {
                    DecodeNode::Tag(tag) => completed = Some(tag),
                    DecodeNode::Frame(frame) => frames.push(frame),
                }
                continue;
            }

            let Some(mut frame) = frames.pop() else {
                return Err(invalid_data("NBT decoder lost its root value"));
            };
            match frame.next_child(self)? {
                FrameAdvance::Child { tag_type, depth } => {
                    frames.push(frame);
                    next = Some((tag_type, depth));
                }
                FrameAdvance::Complete => completed = Some(frame.finish()?),
            }
        }
    }

    fn start_tag(&mut self, tag_type: u8, depth: usize) -> Result<DecodeNode> {
        match tag_type {
            TAG_BYTE => {
                self.account(9)?;
                Ok(DecodeNode::Tag(NbtTag::Byte(self.read_i8()?)))
            }
            TAG_SHORT => {
                self.account(10)?;
                Ok(DecodeNode::Tag(NbtTag::Short(self.read_i16()?)))
            }
            TAG_INT => {
                self.account(12)?;
                Ok(DecodeNode::Tag(NbtTag::Int(self.read_i32()?)))
            }
            TAG_LONG => {
                self.account(16)?;
                Ok(DecodeNode::Tag(NbtTag::Long(self.read_i64()?)))
            }
            TAG_FLOAT => {
                self.account(12)?;
                let value = f32::from_bits(self.read_u32()?);
                // `FloatTag.valueOf` folds both IEEE zero encodings into `ZERO`
                Ok(DecodeNode::Tag(NbtTag::Float(if value == 0.0 {
                    0.0
                } else {
                    value
                })))
            }
            TAG_DOUBLE => {
                self.account(16)?;
                let value = f64::from_bits(self.read_u64()?);
                // `DoubleTag.valueOf` folds both IEEE zero encodings into `ZERO`
                Ok(DecodeNode::Tag(NbtTag::Double(if value == 0.0 {
                    0.0
                } else {
                    value
                })))
            }
            TAG_BYTE_ARRAY => self.read_byte_array().map(DecodeNode::Tag),
            TAG_STRING => self.read_string().map(DecodeNode::Tag),
            TAG_LIST => self.start_list(depth),
            TAG_COMPOUND => self.start_compound(depth),
            TAG_INT_ARRAY => self.read_int_array().map(DecodeNode::Tag),
            TAG_LONG_ARRAY => self.read_long_array().map(DecodeNode::Tag),
            TAG_END => Err(invalid_data(
                "TAG_End is only valid as a root or compound terminator",
            )),
            _ => Err(invalid_data(format!("unknown NBT tag type: {tag_type}"))),
        }
    }

    fn read_string(&mut self) -> Result<NbtTag> {
        self.account(36)?;
        let code_units = self.read_modified_utf8()?;
        self.account_bytes_per_entry(2, code_units.len())?;
        Ok(NbtTag::String(canonical_mutf8(&code_units)))
    }

    fn read_byte_array(&mut self) -> Result<NbtTag> {
        let count = self.read_array_len(1)?;
        Ok(NbtTag::ByteArray(self.take(count)?.to_vec()))
    }

    fn read_int_array(&mut self) -> Result<NbtTag> {
        let count = self.read_array_len(4)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_i32()?);
        }
        Ok(NbtTag::IntArray(values))
    }

    fn read_long_array(&mut self) -> Result<NbtTag> {
        let count = self.read_array_len(8)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_i64()?);
        }
        Ok(NbtTag::LongArray(values))
    }

    fn read_array_len(&mut self, width: u64) -> Result<usize> {
        self.account(24)?;
        let count = self.read_i32()?;
        self.account_signed_count(width, count)?;
        usize::try_from(count).map_err(|_| invalid_data("negative NBT array length"))
    }

    fn start_list(&mut self, depth: usize) -> Result<DecodeNode> {
        Self::enter_container(depth)?;
        self.account(36)?;

        let element_type = self.read_u8()?;
        let count = self.read_i32()?;
        if count < 0 {
            return Err(invalid_data("NBT list length cannot be negative"));
        }
        if element_type == TAG_END && count > 0 {
            return Err(invalid_data("NBT list with TAG_End elements must be empty"));
        }
        self.account_signed_count(4, count)?;

        if count == 0 {
            return Ok(DecodeNode::Tag(NbtTag::List(NbtList::Empty)));
        }
        if !is_known_tag_type(element_type) {
            return Err(invalid_data(format!(
                "unknown NBT list element type: {element_type}"
            )));
        }

        let count = usize::try_from(count).map_err(|_| invalid_data("negative NBT list length"))?;
        Ok(DecodeNode::Frame(NbtFrame::List(NbtListFrame {
            element_type,
            depth,
            remaining: count,
            values: NbtListValues::new(element_type, count)?,
        })))
    }

    fn start_compound(&mut self, depth: usize) -> Result<DecodeNode> {
        Self::enter_container(depth)?;
        self.account(48)?;
        Ok(DecodeNode::Frame(NbtFrame::Compound(NbtCompoundFrame {
            depth,
            key_indices: FxHashMap::default(),
            values: Vec::new(),
            pending_key: None,
        })))
    }

    fn enter_container(depth: usize) -> Result<()> {
        if depth >= MAX_NBT_DEPTH {
            return Err(invalid_data(format!(
                "NBT nesting depth exceeds Vanilla limit of {MAX_NBT_DEPTH}"
            )));
        }
        Ok(())
    }

    fn account_signed_count(&mut self, bytes_per_entry: u64, count: i32) -> Result<()> {
        let count =
            u64::try_from(count).map_err(|_| invalid_data("NBT length must not be negative"))?;
        self.account(
            bytes_per_entry
                .checked_mul(count)
                .ok_or_else(|| invalid_data("NBT accounting size overflows u64"))?,
        )
    }

    fn account_bytes_per_entry(&mut self, bytes_per_entry: u64, count: usize) -> Result<()> {
        let count = u64::try_from(count)
            .map_err(|_| invalid_data("NBT accounting count does not fit u64"))?;
        self.account(
            bytes_per_entry
                .checked_mul(count)
                .ok_or_else(|| invalid_data("NBT accounting size overflows u64"))?,
        )
    }

    fn account(&mut self, size: u64) -> Result<()> {
        let usage = self
            .usage
            .checked_add(size)
            .ok_or_else(|| invalid_data("NBT accounting usage overflows u64"))?;
        if usage > self.quota {
            return Err(invalid_data(format!(
                "NBT payload exceeds Vanilla quota: {} + {size} > {}",
                self.usage, self.quota
            )));
        }
        self.usage = usage;
        Ok(())
    }

    fn read_modified_utf8(&mut self) -> Result<Vec<u16>> {
        let byte_len = usize::from(self.read_u16()?);
        let bytes = self.take(byte_len)?;
        let mut code_units = Vec::with_capacity(byte_len);
        let mut index = 0;

        while index < bytes.len() {
            let first = bytes[index];
            match first {
                0x00..=0x7F => {
                    code_units.push(u16::from(first));
                    index += 1;
                }
                0xC0..=0xDF => {
                    let second = *bytes.get(index + 1).ok_or_else(|| {
                        invalid_data("truncated two-byte modified UTF-8 sequence")
                    })?;
                    if second & 0xC0 != 0x80 {
                        return Err(invalid_data("invalid two-byte modified UTF-8 sequence"));
                    }
                    code_units.push((u16::from(first & 0x1F) << 6) | u16::from(second & 0x3F));
                    index += 2;
                }
                0xE0..=0xEF => {
                    let second = *bytes.get(index + 1).ok_or_else(|| {
                        invalid_data("truncated three-byte modified UTF-8 sequence")
                    })?;
                    let third = *bytes.get(index + 2).ok_or_else(|| {
                        invalid_data("truncated three-byte modified UTF-8 sequence")
                    })?;
                    if second & 0xC0 != 0x80 || third & 0xC0 != 0x80 {
                        return Err(invalid_data("invalid three-byte modified UTF-8 sequence"));
                    }
                    code_units.push(
                        (u16::from(first & 0x0F) << 12)
                            | (u16::from(second & 0x3F) << 6)
                            | u16::from(third & 0x3F),
                    );
                    index += 3;
                }
                _ => return Err(invalid_data("invalid modified UTF-8 leading byte")),
            }
        }

        Ok(code_units)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let bytes = self.take(2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.take(4)?;
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.take(8)?;
        Ok(i64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.take(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(len)
            .ok_or_else(|| invalid_data("NBT input position overflows usize"))?;
        let bytes = self.input.get(self.position..end).ok_or_else(|| {
            Error::new(
                ErrorKind::UnexpectedEof,
                "NBT payload ends before the declared value",
            )
        })?;
        self.position = end;
        Ok(bytes)
    }
}

fn canonical_mutf8(code_units: &[u16]) -> Mutf8String {
    let mut bytes = Vec::with_capacity(code_units.len());
    for &code_unit in code_units {
        match code_unit {
            0 => bytes.extend_from_slice(&[0xC0, 0x80]),
            0x0001..=0x007F => bytes.push(code_unit as u8),
            0x0080..=0x07FF => bytes.extend_from_slice(&[
                0xC0 | ((code_unit >> 6) as u8),
                0x80 | ((code_unit & 0x3F) as u8),
            ]),
            _ => bytes.extend_from_slice(&[
                0xE0 | ((code_unit >> 12) as u8),
                0x80 | (((code_unit >> 6) & 0x3F) as u8),
                0x80 | ((code_unit & 0x3F) as u8),
            ]),
        }
    }
    Mutf8String::from_vec(bytes)
}

const fn is_known_tag_type(tag_type: u8) -> bool {
    matches!(
        tag_type,
        TAG_BYTE
            | TAG_SHORT
            | TAG_INT
            | TAG_LONG
            | TAG_FLOAT
            | TAG_DOUBLE
            | TAG_BYTE_ARRAY
            | TAG_STRING
            | TAG_LIST
            | TAG_COMPOUND
            | TAG_INT_ARRAY
            | TAG_LONG_ARRAY
    )
}

fn invalid_data(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use simdnbt::owned::{NbtCompound, NbtList, NbtTag};

    use super::{
        DEFAULT_NBT_QUOTA, MAX_NBT_DEPTH, read_optional_tag_with_default_quota,
        read_optional_tag_with_quota, read_tag_with_default_quota,
    };

    fn encode(tag: &NbtTag) -> Vec<u8> {
        let mut bytes = Vec::new();
        tag.write(&mut bytes);
        bytes
    }

    fn nested_list(depth: usize) -> Vec<u8> {
        let mut bytes = vec![9];
        for _ in 1..depth {
            bytes.extend_from_slice(&[9, 0, 0, 0, 1]);
        }
        bytes.extend_from_slice(&[1, 0, 0, 0, 1, 0]);
        bytes
    }

    #[test]
    fn optional_reader_consumes_root_end_and_required_reader_rejects_it() {
        let bytes = [0];
        let mut optional = Cursor::new(bytes.as_slice());
        assert_eq!(
            read_optional_tag_with_default_quota(&mut optional)
                .expect("root end must decode as null NBT"),
            None
        );
        assert_eq!(optional.position(), 1);

        let mut required = Cursor::new(bytes.as_slice());
        assert!(read_tag_with_default_quota(&mut required).is_err());
        assert_eq!(required.position(), 1);
    }

    #[test]
    fn reader_rejects_invalid_signed_list_lengths_before_parsing() {
        let negative = [9, 1, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(
            read_optional_tag_with_default_quota(&mut Cursor::new(negative.as_slice())).is_err()
        );

        let end_with_values = [9, 0, 0, 0, 0, 1];
        assert!(
            read_optional_tag_with_default_quota(&mut Cursor::new(end_with_values.as_slice()))
                .is_err()
        );
    }

    #[test]
    fn reader_matches_vanilla_container_depth_limit() {
        let within_limit = nested_list(MAX_NBT_DEPTH);
        assert!(
            read_optional_tag_with_default_quota(&mut Cursor::new(within_limit.as_slice())).is_ok()
        );

        let over_limit = nested_list(MAX_NBT_DEPTH + 1);
        assert!(
            read_optional_tag_with_default_quota(&mut Cursor::new(over_limit.as_slice())).is_err()
        );
    }

    #[test]
    fn reader_accepts_the_quota_boundary_and_rejects_one_byte_more() {
        let boundary = NbtTag::ByteArray(vec![0; (DEFAULT_NBT_QUOTA - 24) as usize]);
        let boundary = encode(&boundary);
        assert!(
            read_optional_tag_with_default_quota(&mut Cursor::new(boundary.as_slice())).is_ok()
        );

        let over_limit = NbtTag::ByteArray(vec![0; (DEFAULT_NBT_QUOTA - 23) as usize]);
        let over_limit = encode(&over_limit);
        assert!(
            read_optional_tag_with_default_quota(&mut Cursor::new(over_limit.as_slice())).is_err()
        );
    }

    #[test]
    fn reader_accounts_supplementary_strings_as_two_java_code_units() {
        let bytes = encode(&NbtTag::String("\u{1F600}".into()));
        assert!(read_optional_tag_with_quota(&mut Cursor::new(bytes.as_slice()), 40).is_ok());
        assert!(read_optional_tag_with_quota(&mut Cursor::new(bytes.as_slice()), 39).is_err());
    }

    #[test]
    fn reader_accounts_duplicate_compound_keys_like_vanilla_maps() {
        let mut compound = NbtCompound::new();
        compound.insert("a", NbtTag::Int(1));
        compound.insert("a", NbtTag::Int(2));
        let bytes = encode(&NbtTag::Compound(compound));

        assert!(read_optional_tag_with_quota(&mut Cursor::new(bytes.as_slice()), 168).is_ok());
        assert!(read_optional_tag_with_quota(&mut Cursor::new(bytes.as_slice()), 167).is_err());
    }

    #[test]
    fn reader_replaces_duplicate_keys_by_java_string_value() {
        let bytes = [
            10, // compound
            3, 0, 2, 0xC1, 0xA1, 0, 0, 0, 1, // non-canonical MUTF-8 key "a"
            3, 0, 1, b'a', 0, 0, 0, 2, // canonical MUTF-8 key "a"
            0,
        ];
        let tag = read_optional_tag_with_default_quota(&mut Cursor::new(bytes.as_slice()))
            .expect("compound must decode")
            .expect("compound must not be null");
        let NbtTag::Compound(compound) = tag else {
            panic!("expected compound tag");
        };

        assert_eq!(compound.len(), 1);
        assert_eq!(compound.int("a"), Some(2));
    }

    #[test]
    fn reader_canonicalizes_noncanonical_modified_utf8_strings() {
        let bytes = [8, 0, 2, 0xC1, 0xA1];
        let tag = read_optional_tag_with_default_quota(&mut Cursor::new(bytes.as_slice()))
            .expect("string must decode")
            .expect("string must not be null");
        let NbtTag::String(value) = tag else {
            panic!("expected string tag");
        };

        assert_eq!(value.to_str(), "a");
        assert_eq!(value.as_bytes(), b"a");
    }

    #[test]
    fn reader_canonicalizes_empty_lists_with_unknown_element_types() {
        let bytes = [9, 13, 0, 0, 0, 0];
        let tag = read_optional_tag_with_default_quota(&mut Cursor::new(bytes.as_slice()))
            .expect("empty list must decode")
            .expect("list must not be null");

        assert!(matches!(tag, NbtTag::List(NbtList::Empty)));
    }

    #[test]
    fn reader_matches_vanilla_float_and_double_value_of_zero_canonicalization() {
        let float = [5, 0x80, 0, 0, 0];
        let float = read_optional_tag_with_default_quota(&mut Cursor::new(float.as_slice()))
            .expect("float must decode")
            .expect("float must not be null");
        let NbtTag::Float(float) = float else {
            panic!("expected float tag");
        };
        assert_eq!(float.to_bits(), 0.0_f32.to_bits());

        let double = [6, 0x80, 0, 0, 0, 0, 0, 0, 0];
        let double = read_optional_tag_with_default_quota(&mut Cursor::new(double.as_slice()))
            .expect("double must decode")
            .expect("double must not be null");
        let NbtTag::Double(double) = double else {
            panic!("expected double tag");
        };
        assert_eq!(double.to_bits(), 0.0_f64.to_bits());

        let float_nan = [5, 0x7F, 0xA0, 0, 1];
        let float_nan =
            read_optional_tag_with_default_quota(&mut Cursor::new(float_nan.as_slice()))
                .expect("float NaN must decode")
                .expect("float NaN must not be null");
        let NbtTag::Float(float_nan) = float_nan else {
            panic!("expected float tag");
        };
        assert_eq!(float_nan.to_bits(), 0x7FA0_0001);

        let double_nan = [6, 0x7F, 0xF4, 0, 0, 0, 0, 0, 1];
        let double_nan =
            read_optional_tag_with_default_quota(&mut Cursor::new(double_nan.as_slice()))
                .expect("double NaN must decode")
                .expect("double NaN must not be null");
        let NbtTag::Double(double_nan) = double_nan else {
            panic!("expected double tag");
        };
        assert_eq!(double_nan.to_bits(), 0x7FF4_0000_0000_0001);
    }

    #[test]
    fn reader_attaches_nested_compound_list_values_without_recursion() {
        let bytes = [
            10, // root compound
            9, 0, 4, b'l', b'i', b's', b't', // list entry
            10, 0, 0, 0, 1, // one compound list element
            3, 0, 1, b'x', 0, 0, 0, 7, // compound's int entry
            0, // nested compound end
            0, // root compound end
        ];
        let tag = read_optional_tag_with_default_quota(&mut Cursor::new(bytes.as_slice()))
            .expect("nested value must decode")
            .expect("nested value must not be null");
        let NbtTag::Compound(root) = tag else {
            panic!("expected root compound");
        };
        let Some(NbtTag::List(NbtList::Compound(values))) = root.get("list") else {
            panic!("expected compound list");
        };

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].int("x"), Some(7));
    }
}
