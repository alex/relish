use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::mem;
use std::sync::Arc;

use crate::Relish;
use crate::buf::BytesRef;
use crate::parse::{parse_value_for_typeid, read_byte};
use crate::{ParseError, ParseErrorKind, ParseResult};

#[doc(hidden)]
pub fn write_tagged_varint_length(buffer: &mut Vec<u8>, length: usize) -> crate::WriteResult<()> {
    if length > (1 << 31) - 1 {
        return Err(crate::WriteError::new(
            crate::WriteErrorKind::ContentTooLarge(length),
        ));
    }

    if length < 128 {
        buffer.push((length << 1) as u8);
    } else {
        let tagged_value = ((length as u32) << 1) | 1;
        buffer.extend_from_slice(&tagged_value.to_le_bytes());
    }
    Ok(())
}

#[doc(hidden)]
pub fn tagged_varint_length_size(length: usize) -> usize {
    if length < 128 { 1 } else { 4 }
}

/// Type identifiers used in the Relish binary format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TypeId {
    Null = 0x00,
    Bool = 0x01,
    U8 = 0x02,
    U16 = 0x03,
    U32 = 0x04,
    U64 = 0x05,
    U128 = 0x06,
    I8 = 0x07,
    I16 = 0x08,
    I32 = 0x09,
    I64 = 0x0A,
    I128 = 0x0B,
    F32 = 0x0C,
    F64 = 0x0D,
    String = 0x0E,
    Array = 0x0F,
    Map = 0x10,
    Struct = 0x11,
    Enum = 0x12,
    Timestamp = 0x13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeLength {
    Fixed(usize),
    Varsize,
}

impl TypeId {
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        if byte & 0x80 != 0 {
            return None;
        }

        match byte {
            0x00 => Some(TypeId::Null),
            0x01 => Some(TypeId::Bool),
            0x02 => Some(TypeId::U8),
            0x03 => Some(TypeId::U16),
            0x04 => Some(TypeId::U32),
            0x05 => Some(TypeId::U64),
            0x06 => Some(TypeId::U128),
            0x07 => Some(TypeId::I8),
            0x08 => Some(TypeId::I16),
            0x09 => Some(TypeId::I32),
            0x0A => Some(TypeId::I64),
            0x0B => Some(TypeId::I128),
            0x0C => Some(TypeId::F32),
            0x0D => Some(TypeId::F64),
            0x0E => Some(TypeId::String),
            0x0F => Some(TypeId::Array),
            0x10 => Some(TypeId::Map),
            0x11 => Some(TypeId::Struct),
            0x12 => Some(TypeId::Enum),
            0x13 => Some(TypeId::Timestamp),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn read_for_type<T: Relish>(data: &mut BytesRef) -> ParseResult<TypeId> {
        let type_byte = read_byte(data)?;
        let type_id = TypeId::from_byte(type_byte)
            .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTypeId(type_byte)))?;

        if type_id != T::TYPE {
            return Err(ParseError::new(ParseErrorKind::TypeMismatch {
                expected: T::TYPE as u8,
                actual: type_byte,
            }));
        }

        Ok(type_id)
    }

    pub(crate) fn length(self) -> TypeLength {
        match self {
            TypeId::Null => TypeLength::Fixed(0),
            TypeId::Bool => TypeLength::Fixed(1),
            TypeId::U8 => TypeLength::Fixed(1),
            TypeId::U16 => TypeLength::Fixed(2),
            TypeId::U32 => TypeLength::Fixed(4),
            TypeId::U64 => TypeLength::Fixed(8),
            TypeId::U128 => TypeLength::Fixed(16),
            TypeId::I8 => TypeLength::Fixed(1),
            TypeId::I16 => TypeLength::Fixed(2),
            TypeId::I32 => TypeLength::Fixed(4),
            TypeId::I64 => TypeLength::Fixed(8),
            TypeId::I128 => TypeLength::Fixed(16),
            TypeId::F32 => TypeLength::Fixed(4),
            TypeId::F64 => TypeLength::Fixed(8),
            TypeId::String => TypeLength::Varsize,
            TypeId::Array => TypeLength::Varsize,
            TypeId::Map => TypeLength::Varsize,
            TypeId::Struct => TypeLength::Varsize,
            TypeId::Enum => TypeLength::Varsize,
            TypeId::Timestamp => TypeLength::Fixed(8),
        }
    }
}

impl From<TypeId> for u8 {
    fn from(type_id: TypeId) -> u8 {
        type_id as u8
    }
}

macro_rules! impl_relish_integer {
    ($ty:ty, $type_id:expr) => {
        impl Relish for $ty {
            const TYPE: TypeId = $type_id;

            fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
                const SIZE: usize = mem::size_of::<$ty>();
                let bytes: [u8; SIZE] = data.read(SIZE)?.as_ref().try_into().unwrap();
                Ok(<$ty>::from_le_bytes(bytes))
            }

            fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
                buffer.extend_from_slice(&self.to_le_bytes());
                Ok(())
            }

            fn value_length(&self) -> usize {
                mem::size_of::<Self>()
            }
        }
    };
}

impl_relish_integer!(u8, TypeId::U8);
impl_relish_integer!(u16, TypeId::U16);
impl_relish_integer!(u32, TypeId::U32);
impl_relish_integer!(u64, TypeId::U64);
impl_relish_integer!(u128, TypeId::U128);
impl_relish_integer!(i8, TypeId::I8);
impl_relish_integer!(i16, TypeId::I16);
impl_relish_integer!(i32, TypeId::I32);
impl_relish_integer!(i64, TypeId::I64);
impl_relish_integer!(i128, TypeId::I128);
impl_relish_integer!(f32, TypeId::F32);
impl_relish_integer!(f64, TypeId::F64);

impl Relish for bool {
    const TYPE: TypeId = TypeId::Bool;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        match data[0] {
            0x00 => Ok(false),
            0xFF => Ok(true),
            b => Err(ParseError::new(ParseErrorKind::InvalidBoolValue(b))),
        }
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        buffer.push(if *self { 0xFF } else { 0x00 });
        Ok(())
    }

    fn value_length(&self) -> usize {
        1
    }
}

/// A unit type representing the null value in Relish serialization.
#[derive(Debug, PartialEq)]
pub struct Null;

impl Relish for Null {
    const TYPE: TypeId = TypeId::Null;

    fn parse_value(_data: &mut BytesRef) -> ParseResult<Self> {
        Ok(Null)
    }

    fn write_value(&self, _buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        Ok(())
    }

    fn value_length(&self) -> usize {
        0
    }
}

#[cfg(feature = "chrono")]
impl Relish for chrono::DateTime<chrono::Utc> {
    const TYPE: TypeId = TypeId::Timestamp;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        const SIZE: usize = mem::size_of::<u64>();
        let bytes: [u8; SIZE] = data.read(SIZE)?.as_ref().try_into().unwrap();
        let timestamp = u64::from_le_bytes(bytes);
        let timestamp_i64 = i64::try_from(timestamp)
            .map_err(|_| ParseError::new(ParseErrorKind::InvalidTimestamp(timestamp)))?;
        chrono::DateTime::from_timestamp(timestamp_i64, 0)
            .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTimestamp(timestamp)))
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        let timestamp = self.timestamp();
        let timestamp_u64 = u64::try_from(timestamp)
            .map_err(|_| crate::WriteError::new(crate::WriteErrorKind::InvalidTimestamp))?;
        buffer.extend_from_slice(&timestamp_u64.to_le_bytes());
        Ok(())
    }

    fn value_length(&self) -> usize {
        mem::size_of::<u64>()
    }
}

impl Relish for String {
    const TYPE: TypeId = TypeId::String;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        let result = String::from_utf8(data.as_ref().to_vec())
            .map_err(|_| ParseError::new(ParseErrorKind::InvalidUtf8))?;
        Ok(result)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        let bytes = self.as_bytes();
        let len = bytes.len();
        let prefix_len = tagged_varint_length_size(len);
        buffer.reserve(prefix_len + len);
        write_tagged_varint_length(buffer, len)?;
        buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn value_length(&self) -> usize {
        let len = self.len();
        tagged_varint_length_size(len) + len
    }
}

impl Relish for Arc<str> {
    const TYPE: TypeId = TypeId::String;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        let s = std::str::from_utf8(data.as_ref())
            .map_err(|_| ParseError::new(ParseErrorKind::InvalidUtf8))?;
        Ok(Arc::from(s))
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        let bytes = self.as_bytes();
        let len = bytes.len();
        let prefix_len = tagged_varint_length_size(len);
        buffer.reserve(prefix_len + len);
        write_tagged_varint_length(buffer, len)?;
        buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn value_length(&self) -> usize {
        let len = self.len();
        tagged_varint_length_size(len) + len
    }
}

impl<T: Relish> Relish for Vec<T> {
    const TYPE: TypeId = TypeId::Array;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        let elem_type = TypeId::read_for_type::<T>(data)?;

        let mut elements = Vec::new();
        if let TypeLength::Fixed(size) = elem_type.length() {
            elements.reserve(data.len() / size);
        }

        while !data.is_empty() {
            elements.push(parse_value_for_typeid::<T>(data, elem_type)?);
        }

        Ok(elements)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        let mut content_len = 1;
        for elem in self {
            content_len += elem.value_length();
        }

        let prefix_len = tagged_varint_length_size(content_len);
        buffer.reserve(prefix_len + content_len);
        write_tagged_varint_length(buffer, content_len)?;
        buffer.push(T::TYPE as u8);

        for elem in self {
            elem.write_value(buffer)?;
        }

        Ok(())
    }

    fn value_length(&self) -> usize {
        let mut content_size = 1;
        for elem in self {
            content_size += elem.value_length();
        }
        tagged_varint_length_size(content_size) + content_size
    }
}

impl Relish for bytes::Bytes {
    const TYPE: TypeId = TypeId::Array;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        TypeId::read_for_type::<u8>(data)?;

        Ok(data.to_bytes())
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        let content_len = 1 + self.len();

        let prefix_len = tagged_varint_length_size(content_len);
        buffer.reserve(prefix_len + content_len);
        write_tagged_varint_length(buffer, content_len)?;
        buffer.push(TypeId::U8 as u8);
        buffer.extend_from_slice(self);

        Ok(())
    }

    fn value_length(&self) -> usize {
        let content_size = 1 + self.len();
        tagged_varint_length_size(content_size) + content_size
    }
}

impl<K: Relish + Eq + Hash, V: Relish, S: BuildHasher + Default> Relish for HashMap<K, V, S> {
    const TYPE: TypeId = TypeId::Map;

    fn parse_value(data: &mut BytesRef) -> ParseResult<Self> {
        let key_type = TypeId::read_for_type::<K>(data)?;
        let value_type = TypeId::read_for_type::<V>(data)?;

        let mut map = HashMap::default();
        while !data.is_empty() {
            let key = parse_value_for_typeid::<K>(data, key_type)?;
            let value = parse_value_for_typeid::<V>(data, value_type)?;
            if map.insert(key, value).is_some() {
                return Err(ParseError::new(ParseErrorKind::DuplicateMapKey));
            }
        }

        Ok(map)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()> {
        let mut content_len = 2;
        for (k, v) in self {
            content_len += k.value_length() + v.value_length();
        }

        let prefix_len = tagged_varint_length_size(content_len);
        buffer.reserve(prefix_len + content_len);
        write_tagged_varint_length(buffer, content_len)?;
        buffer.push(K::TYPE as u8);
        buffer.push(V::TYPE as u8);

        for (k, v) in self {
            k.write_value(buffer)?;
            v.write_value(buffer)?;
        }

        Ok(())
    }

    fn value_length(&self) -> usize {
        let mut content_size = 2;
        for (k, v) in self {
            content_size += k.value_length() + v.value_length();
        }
        tagged_varint_length_size(content_size) + content_size
    }
}

#[cfg(test)]
mod tests {
    use super::{tagged_varint_length_size, write_tagged_varint_length};
    use crate::buf::BytesRef;
    use crate::parse::read_tagged_varint_length;
    use crate::{Null, ParseError, ParseErrorKind, ParseResult, Relish, TypeId, parse, to_vec};
    use bytes::Bytes;

    #[test]
    fn test_type_id_from_byte() {
        assert_eq!(TypeId::from_byte(0x00), Some(TypeId::Null));
        assert_eq!(TypeId::from_byte(0x01), Some(TypeId::Bool));
        assert_eq!(TypeId::from_byte(0x0E), Some(TypeId::String));
        assert_eq!(TypeId::from_byte(0x12), Some(TypeId::Enum));
        assert_eq!(TypeId::from_byte(0x13), Some(TypeId::Timestamp));
        assert_eq!(TypeId::from_byte(0x80), None);
        assert_eq!(TypeId::from_byte(0xFF), None);
        assert_eq!(TypeId::from_byte(0x14), None);
    }

    #[test]
    fn test_tagged_varint_roundtrip() {
        for length in 0..=u16::MAX as usize {
            let mut buffer = Vec::new();
            write_tagged_varint_length(&mut buffer, length).unwrap();

            let expected_size = tagged_varint_length_size(length);
            assert_eq!(buffer.len(), expected_size);

            let bytes = Bytes::from(buffer);
            let mut bytes_ref = BytesRef::new(&bytes);
            let decoded = read_tagged_varint_length(&mut bytes_ref).unwrap();
            assert_eq!(decoded as usize, length);
            assert!(bytes_ref.is_empty());
        }
    }

    fn assert_roundtrips<T: Relish + PartialEq + std::fmt::Debug>(
        cases: &[(ParseResult<T>, &[u8])],
    ) {
        for (expected, bytes) in cases {
            let parse_result = parse::<T>(Bytes::from(bytes.to_vec()));
            assert_eq!(expected, &parse_result);
            if let Ok(val) = expected {
                assert_eq!(to_vec(val).as_deref(), Ok(*bytes));
            }
        }
    }

    #[test]
    fn test_null() {
        assert_roundtrips(&[(Ok(Null), &[0x00u8])]);
    }

    #[test]
    fn test_bool() {
        assert_roundtrips(&[
            (Ok(true), &[0x01u8, 0xFF]),
            (Ok(false), &[0x01, 0x00]),
            (
                Err(ParseError::new(ParseErrorKind::InvalidBoolValue(0x42))),
                &[0x01, 0x42],
            ),
            (
                Err(ParseError::new(ParseErrorKind::TypeMismatch {
                    expected: 0x01,
                    actual: 0x04,
                })),
                &[0x04, 0x2A, 0x00, 0x00, 0x00],
            ),
            (
                Err(ParseError::new(ParseErrorKind::InsufficientData {
                    needed: 1,
                    available: 0,
                })),
                &[0x01],
            ),
        ]);
    }

    #[test]
    fn test_integers_u8() {
        assert_roundtrips(&[
            (Ok(42u8), &[0x02u8, 0x2A]),
            (Ok(255u8), &[0x02, 0xFF]),
            (
                Err(ParseError::new(ParseErrorKind::InsufficientData {
                    needed: 1,
                    available: 0,
                })),
                &[0x02],
            ),
            (
                Err(ParseError::new(ParseErrorKind::InvalidTypeId(0x80))),
                &[0x80],
            ),
        ]);
    }

    #[test]
    fn test_integers_u32() {
        assert_roundtrips(&[
            (Ok(42u32), &[0x04u8, 0x2A, 0x00, 0x00, 0x00]),
            (
                Err(ParseError::new(ParseErrorKind::InsufficientData {
                    needed: 4,
                    available: 1,
                })),
                &[0x04, 0x2A],
            ),
        ]);
    }

    #[test]
    fn test_integers_i32() {
        assert_roundtrips(&[(Ok(-42i32), &[0x09u8, 0xD6, 0xFF, 0xFF, 0xFF])]);
    }

    #[test]
    fn test_integers_u128() {
        assert_roundtrips(&[(
            Ok(u128::MAX),
            &[
                0x06u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF,
            ],
        )]);
    }

    #[test]
    fn test_integers_i64() {
        assert_roundtrips(&[(
            Ok(-1234567890i64),
            &[0x0Au8, 0x2E, 0xFD, 0x69, 0xB6, 0xFF, 0xFF, 0xFF, 0xFF],
        )]);
    }

    #[test]
    fn test_floats() {
        assert_roundtrips(&[(Ok(std::f32::consts::PI), &[0x0Cu8, 0xdb, 0x0F, 0x49, 0x40])]);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_timestamp() {
        use chrono::DateTime;

        assert_roundtrips(&[
            (
                // Test a known timestamp: 2009-02-13 23:31:30 UTC
                Ok(DateTime::from_timestamp(1234567890, 0).unwrap()),
                &[0x13u8, 0xD2, 0x02, 0x96, 0x49, 0x00, 0x00, 0x00, 0x00],
            ),
            (
                // Test Unix epoch
                Ok(DateTime::from_timestamp(0, 0).unwrap()),
                &[0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ),
        ]);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_timestamp_out_of_range() {
        use chrono::DateTime;

        // Test parsing a u64 value that exceeds i64::MAX
        // This should fail because chrono can't represent timestamps > i64::MAX
        let invalid_bytes = &[0x13u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let result = parse::<DateTime<chrono::Utc>>(Bytes::from(invalid_bytes.to_vec()));
        assert_eq!(
            result.unwrap_err().kind(),
            &ParseErrorKind::InvalidTimestamp(u64::MAX)
        );

        // Test serializing a DateTime before Unix epoch (negative timestamp)
        // This should fail because we can't represent negative timestamps as u64
        let before_epoch = DateTime::from_timestamp(-1, 0).unwrap();
        let result = to_vec(&before_epoch);
        assert_eq!(
            result.unwrap_err().kind(),
            &crate::WriteErrorKind::InvalidTimestamp
        );
    }

    #[test]
    fn test_string() {
        assert_roundtrips(&[
            (
                Ok("Hello, Relish!".to_string()),
                &[
                    0x0Eu8, 0x1C, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'R', b'e', b'l', b'i',
                    b's', b'h', b'!',
                ],
            ),
            (
                Err(ParseError::new(ParseErrorKind::InvalidUtf8)),
                &[0x0E, 0x08, 0xFF, 0xFE, 0xFD, 0xFC],
            ),
        ]);
    }

    #[test]
    fn test_arc_str() {
        use std::sync::Arc;

        assert_roundtrips(&[
            (
                Ok(Arc::<str>::from("Hello, Relish!")),
                &[
                    0x0Eu8, 0x1C, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'R', b'e', b'l', b'i',
                    b's', b'h', b'!',
                ],
            ),
            (
                Err(ParseError::new(ParseErrorKind::InvalidUtf8)),
                &[0x0E, 0x08, 0xFF, 0xFE, 0xFD, 0xFC],
            ),
        ]);

        // Verify roundtrip with String produces identical bytes
        let arc_str: Arc<str> = Arc::from("test string");
        let string_data = "test string".to_string();
        assert_eq!(to_vec(&arc_str).unwrap(), to_vec(&string_data).unwrap());
    }

    #[test]
    fn test_vec() {
        assert_roundtrips(&[
            (
                Ok(vec![1u32, 2, 3, 4]),
                &[
                    0x0Fu8, 0x22, 0x04, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00,
                    0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                ],
            ),
            (Ok(vec![]), &[0x0F, 0x02, 0x04]),
        ]);
    }

    #[test]
    fn test_vec_string() {
        assert_roundtrips(&[(
            Ok(vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string(),
            ]),
            &[
                0x0Fu8, 0x1A, 0x0E, 0x06, b'f', b'o', b'o', 0x06, b'b', b'a', b'r', 0x06, b'b',
                b'a', b'z',
            ],
        )]);
    }

    #[test]
    fn test_bytes() {
        assert_roundtrips(&[
            (
                Ok(Bytes::from(vec![1u8, 2, 3, 4])),
                &[0x0Fu8, 0x0A, 0x02, 0x01, 0x02, 0x03, 0x04],
            ),
            (Ok(Bytes::from(vec![])), &[0x0F, 0x02, 0x02]),
            (
                Err(ParseError::new(ParseErrorKind::TypeMismatch {
                    expected: 0x02,
                    actual: 0x04,
                })),
                &[0x0Fu8, 0x0A, 0x04, 0x01, 0x00, 0x00, 0x00],
            ),
        ]);

        // Verify roundtrip with Vec<u8> produces identical bytes
        let bytes_data = Bytes::from(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let vec_data: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(to_vec(&bytes_data).unwrap(), to_vec(&vec_data).unwrap());
    }

    #[test]
    fn test_map() {
        use std::collections::HashMap;

        let mut map1 = HashMap::new();
        map1.insert(1u32, 10u32);

        let map_empty: HashMap<u32, u32> = HashMap::new();

        assert_roundtrips(&[
            (
                Ok(map1),
                &[
                    0x10u8, 0x14, 0x04, 0x04, 0x01, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00,
                ],
            ),
            (Ok(map_empty), &[0x10, 0x04, 0x04, 0x04]),
            (
                Err(ParseError::new(ParseErrorKind::DuplicateMapKey)),
                &[
                    0x10u8, 0x24, 0x04, 0x04, 0x01, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00,
                ],
            ),
        ]);

        let mut map2 = HashMap::new();
        map2.insert(1u32, "foo".to_string());

        assert_roundtrips(&[(
            Ok(map2),
            &[
                0x10u8, 0x14, 0x04, 0x0E, 0x01, 0x00, 0x00, 0x00, 0x06, b'f', b'o', b'o',
            ],
        )]);
    }
}
