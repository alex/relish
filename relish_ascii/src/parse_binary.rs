use bytes::Bytes;
use relish::{
    BytesRef, ParseError, ParseErrorKind, Relish, TypeId, read_byte, read_value_for_typeid,
};

use crate::value::Value;

type Result<T> = std::result::Result<T, ParseError>;

/// Parses binary relish data into a schemaless Value.
pub fn from_bytes(data: Bytes) -> Result<Value> {
    let mut reader = BytesRef::new(&data);
    let value = parse_value(&mut reader)?;

    if !reader.is_empty() {
        return Err(ParseError::new(ParseErrorKind::ExtraData {
            bytes_remaining: reader.len(),
        }));
    }

    Ok(value)
}

/// Parses a single value from the reader (type byte + value).
fn parse_value(reader: &mut BytesRef) -> Result<Value> {
    let type_byte = read_byte(reader)?;
    let type_id = TypeId::from_byte(type_byte)
        .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTypeId(type_byte)))?;

    parse_typed_value(reader, type_id)
}

/// Parses a value given its type ID.
fn parse_typed_value(reader: &mut BytesRef, type_id: TypeId) -> Result<Value> {
    let mut content = read_value_for_typeid(reader, type_id)?;

    match type_id {
        TypeId::Null => Ok(Value::Null),
        TypeId::Bool => Ok(Value::Bool(bool::parse_value(&mut content)?)),
        TypeId::U8 => Ok(Value::U8(u8::parse_value(&mut content)?)),
        TypeId::U16 => Ok(Value::U16(u16::parse_value(&mut content)?)),
        TypeId::U32 => Ok(Value::U32(u32::parse_value(&mut content)?)),
        TypeId::U64 => Ok(Value::U64(u64::parse_value(&mut content)?)),
        TypeId::U128 => Ok(Value::U128(u128::parse_value(&mut content)?)),
        TypeId::I8 => Ok(Value::I8(i8::parse_value(&mut content)?)),
        TypeId::I16 => Ok(Value::I16(i16::parse_value(&mut content)?)),
        TypeId::I32 => Ok(Value::I32(i32::parse_value(&mut content)?)),
        TypeId::I64 => Ok(Value::I64(i64::parse_value(&mut content)?)),
        TypeId::I128 => Ok(Value::I128(i128::parse_value(&mut content)?)),
        TypeId::F32 => Ok(Value::F32(f32::parse_value(&mut content)?)),
        TypeId::F64 => Ok(Value::F64(f64::parse_value(&mut content)?)),
        TypeId::Timestamp => Ok(Value::Timestamp(u64::parse_value(&mut content)?)),
        TypeId::String => Ok(Value::String(String::parse_value(&mut content)?)),

        TypeId::Array => {
            let elem_type_byte = read_byte(&mut content)?;
            let element_type = TypeId::from_byte(elem_type_byte)
                .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTypeId(elem_type_byte)))?;

            let mut elements = Vec::new();
            while !content.is_empty() {
                elements.push(parse_typed_value(&mut content, element_type)?);
            }
            Ok(Value::Array {
                element_type,
                elements,
            })
        }

        TypeId::Map => {
            let key_type_byte = read_byte(&mut content)?;
            let key_type = TypeId::from_byte(key_type_byte)
                .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTypeId(key_type_byte)))?;
            let value_type_byte = read_byte(&mut content)?;
            let value_type = TypeId::from_byte(value_type_byte)
                .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTypeId(value_type_byte)))?;

            let mut entries = Vec::new();
            while !content.is_empty() {
                let key = parse_typed_value(&mut content, key_type)?;
                let value = parse_typed_value(&mut content, value_type)?;
                entries.push((key, value));
            }
            Ok(Value::Map {
                key_type,
                value_type,
                entries,
            })
        }

        TypeId::Struct => {
            let mut fields = Vec::new();
            while !content.is_empty() {
                let field_id = read_byte(&mut content)?;
                if field_id & 0x80 != 0 {
                    return Err(ParseError::new(ParseErrorKind::InvalidFieldId(field_id)));
                }
                fields.push((field_id, parse_value(&mut content)?));
            }
            Ok(Value::Struct { fields })
        }

        TypeId::Enum => {
            let variant_id = read_byte(&mut content)?;
            if variant_id & 0x80 != 0 {
                return Err(ParseError::new(ParseErrorKind::InvalidFieldId(variant_id)));
            }
            let value = parse_value(&mut content)?;
            if !content.is_empty() {
                return Err(ParseError::new(ParseErrorKind::ExtraData {
                    bytes_remaining: content.len(),
                }));
            }
            Ok(Value::Enum {
                variant_id,
                value: Box::new(value),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use relish::TypeId;

    use crate::parse_binary::from_bytes;
    use crate::value::Value;

    #[test]
    fn test_parse_null() {
        let data = Bytes::from(vec![0x00]);
        assert_eq!(from_bytes(data).unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(
            from_bytes(Bytes::from(vec![0x01, 0xFF])).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            from_bytes(Bytes::from(vec![0x01, 0x00])).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_parse_u32() {
        let data = Bytes::from(vec![0x04, 0x2A, 0x00, 0x00, 0x00]);
        assert_eq!(from_bytes(data).unwrap(), Value::U32(42));
    }

    #[test]
    fn test_parse_string() {
        let data = Bytes::from(vec![0x0E, 0x0A, b'H', b'e', b'l', b'l', b'o']);
        assert_eq!(
            from_bytes(data).unwrap(),
            Value::String("Hello".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let data = Bytes::from(vec![
            0x0F, 0x1A, 0x04, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
            0x00,
        ]);
        assert_eq!(
            from_bytes(data).unwrap(),
            Value::Array {
                element_type: TypeId::U32,
                elements: vec![Value::U32(1), Value::U32(2), Value::U32(3)],
            }
        );
    }

    #[test]
    fn test_parse_struct() {
        let data = Bytes::from(vec![0x11, 0x0C, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00]);
        assert_eq!(
            from_bytes(data).unwrap(),
            Value::Struct {
                fields: vec![(0, Value::U32(42))]
            }
        );
    }
}
