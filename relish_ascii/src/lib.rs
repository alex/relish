//! relish_ascii provides a human-readable ASCII format for relish binary data.
//!
//! This crate is primarily used as a binary (`relish2ascii`) for pretty-printing
//! arbitrary relish binary data without a schema.
//!
//! This is inspired by Google's der-ascii project for DER/BER encodings.

mod format;
mod parse_binary;
mod value;

/// Parses binary relish data and formats it as ASCII.
pub fn relish2ascii(data: &[u8]) -> Result<String, relish::ParseError> {
    let bytes = bytes::Bytes::from(data.to_vec());
    let value = parse_binary::from_bytes(bytes)?;
    Ok(format::to_ascii(&value))
}

#[cfg(test)]
mod tests {
    use relish::{Relish, to_vec};

    use crate::format::to_ascii;
    use crate::parse_binary::from_bytes;
    use crate::relish2ascii;
    use crate::value::Value;

    #[derive(Debug, Relish)]
    struct Person {
        #[relish(field_id = 0)]
        name: String,
        #[relish(field_id = 1)]
        age: u32,
        #[relish(field_id = 2)]
        active: bool,
    }

    #[test]
    fn test_derived_struct() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        };
        let serialized = to_vec(&person).unwrap();
        let ascii = relish2ascii(&serialized).unwrap();
        assert_eq!(
            ascii,
            "struct {\n  0: \"Alice\",\n  1: 30u32,\n  2: true,\n}"
        );
    }

    #[derive(Debug, Relish)]
    struct Outer {
        #[relish(field_id = 0)]
        inner: Inner,
    }

    #[derive(Debug, Relish)]
    struct Inner {
        #[relish(field_id = 0)]
        value: u32,
    }

    #[test]
    fn test_nested_struct() {
        let outer = Outer {
            inner: Inner { value: 42 },
        };
        let serialized = to_vec(&outer).unwrap();
        let ascii = relish2ascii(&serialized).unwrap();
        assert_eq!(ascii, "struct {\n  0: struct {\n    0: 42u32,\n  },\n}");
    }

    #[derive(Debug, Relish)]
    enum Status {
        #[relish(field_id = 0)]
        Active(u32),
        #[relish(field_id = 1)]
        Inactive(String),
    }

    #[test]
    fn test_derived_enum() {
        let active = Status::Active(100);
        let serialized = to_vec(&active).unwrap();
        let ascii = relish2ascii(&serialized).unwrap();
        assert_eq!(ascii, "enum {\n  0: 100u32,\n}");

        let inactive = Status::Inactive("retired".to_string());
        let serialized = to_vec(&inactive).unwrap();
        let ascii = relish2ascii(&serialized).unwrap();
        assert_eq!(ascii, "enum {\n  1: \"retired\",\n}");
    }

    #[derive(Debug, Relish)]
    struct WithOptional {
        #[relish(field_id = 0)]
        required: u32,
        #[relish(field_id = 1)]
        optional: Option<String>,
    }

    #[test]
    fn test_optional_fields() {
        let with_value = WithOptional {
            required: 1,
            optional: Some("present".to_string()),
        };
        let ascii = relish2ascii(&to_vec(&with_value).unwrap()).unwrap();
        assert_eq!(ascii, "struct {\n  0: 1u32,\n  1: \"present\",\n}");

        let without_value = WithOptional {
            required: 1,
            optional: None,
        };
        let ascii = relish2ascii(&to_vec(&without_value).unwrap()).unwrap();
        assert_eq!(ascii, "struct {\n  0: 1u32,\n}");
    }

    #[test]
    fn test_array() {
        let numbers: Vec<u32> = vec![1, 2, 3];
        let serialized = to_vec(&numbers).unwrap();
        let ascii = relish2ascii(&serialized).unwrap();
        assert_eq!(ascii, "array<u32> {\n  1u32,\n  2u32,\n  3u32,\n}");
    }

    #[test]
    fn test_map() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("key".to_string(), 42u32);
        let serialized = to_vec(&map).unwrap();
        let ascii = relish2ascii(&serialized).unwrap();
        assert_eq!(ascii, "map<string, u32> {\n  \"key\": 42u32,\n}");
    }

    #[test]
    fn test_invalid_type_id() {
        let data = bytes::Bytes::from(vec![0x80u8]);
        assert!(from_bytes(data).is_err());
    }

    #[test]
    fn test_insufficient_data() {
        let data = bytes::Bytes::from(vec![0x04u8, 0x2A, 0x00]);
        assert!(from_bytes(data).is_err());
    }

    #[test]
    fn test_float_special_values() {
        let nan_data =
            bytes::Bytes::from(vec![0x0Du8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x7F]);
        let value = from_bytes(nan_data).unwrap();
        assert!(matches!(value, Value::F64(f) if f.is_nan()));
        assert_eq!(to_ascii(&value), "NaNf64");

        let inf_data =
            bytes::Bytes::from(vec![0x0Du8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x7F]);
        let value = from_bytes(inf_data).unwrap();
        assert_eq!(value, Value::F64(f64::INFINITY));
        assert_eq!(to_ascii(&value), "inff64");
    }
}
