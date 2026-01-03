use std::fmt::Write;

use relish::TypeId;

use crate::value::Value;

/// Formats a Value as human-readable ASCII text.
pub fn to_ascii(value: &Value) -> String {
    let mut output = String::new();
    format_value(&mut output, value, 0);
    output
}

/// Returns the Rust-style type suffix for a TypeId.
fn type_suffix(type_id: TypeId) -> &'static str {
    match type_id {
        TypeId::U8 => "u8",
        TypeId::U16 => "u16",
        TypeId::U32 => "u32",
        TypeId::U64 => "u64",
        TypeId::U128 => "u128",
        TypeId::I8 => "i8",
        TypeId::I16 => "i16",
        TypeId::I32 => "i32",
        TypeId::I64 => "i64",
        TypeId::I128 => "i128",
        TypeId::F32 => "f32",
        TypeId::F64 => "f64",
        TypeId::Null => "null",
        TypeId::Bool => "bool",
        TypeId::String => "string",
        TypeId::Array => "array",
        TypeId::Map => "map",
        TypeId::Struct => "struct",
        TypeId::Enum => "enum",
        TypeId::Timestamp => "timestamp",
    }
}

/// Formats a value with the given indentation level.
fn format_value(output: &mut String, value: &Value, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(b) => output.push_str(if *b { "true" } else { "false" }),
        Value::U8(n) => write!(output, "{}u8", n).unwrap(),
        Value::U16(n) => write!(output, "{}u16", n).unwrap(),
        Value::U32(n) => write!(output, "{}u32", n).unwrap(),
        Value::U64(n) => write!(output, "{}u64", n).unwrap(),
        Value::U128(n) => write!(output, "{}u128", n).unwrap(),
        Value::I8(n) => write!(output, "{}i8", n).unwrap(),
        Value::I16(n) => write!(output, "{}i16", n).unwrap(),
        Value::I32(n) => write!(output, "{}i32", n).unwrap(),
        Value::I64(n) => write!(output, "{}i64", n).unwrap(),
        Value::I128(n) => write!(output, "{}i128", n).unwrap(),
        Value::F32(n) => write!(output, "{}f32", n).unwrap(),
        Value::F64(n) => write!(output, "{}f64", n).unwrap(),
        Value::String(s) => format_string_literal(output, s),
        Value::Timestamp(ts) => write!(output, "timestamp({})", ts).unwrap(),

        Value::Array {
            element_type,
            elements,
        } => {
            write!(output, "array<{}> {{", type_suffix(*element_type)).unwrap();
            if elements.is_empty() {
                output.push('}');
            } else {
                output.push('\n');
                for elem in elements {
                    output.push_str(&indent_str);
                    output.push_str("  ");
                    format_value(output, elem, indent + 1);
                    output.push_str(",\n");
                }
                output.push_str(&indent_str);
                output.push('}');
            }
        }

        Value::Map {
            key_type,
            value_type,
            entries,
        } => {
            write!(
                output,
                "map<{}, {}> {{",
                type_suffix(*key_type),
                type_suffix(*value_type)
            )
            .unwrap();
            if entries.is_empty() {
                output.push('}');
            } else {
                output.push('\n');
                for (k, v) in entries {
                    output.push_str(&indent_str);
                    output.push_str("  ");
                    format_value(output, k, indent + 1);
                    output.push_str(": ");
                    format_value(output, v, indent + 1);
                    output.push_str(",\n");
                }
                output.push_str(&indent_str);
                output.push('}');
            }
        }

        Value::Struct { fields } => {
            output.push_str("struct {");
            if fields.is_empty() {
                output.push('}');
            } else {
                output.push('\n');
                for (field_id, field_value) in fields {
                    output.push_str(&indent_str);
                    write!(output, "  {}: ", field_id).unwrap();
                    format_value(output, field_value, indent + 1);
                    output.push_str(",\n");
                }
                output.push_str(&indent_str);
                output.push('}');
            }
        }

        Value::Enum { variant_id, value } => {
            output.push_str("enum {\n");
            output.push_str(&indent_str);
            write!(output, "  {}: ", variant_id).unwrap();
            format_value(output, value, indent + 1);
            output.push_str(",\n");
            output.push_str(&indent_str);
            output.push('}');
        }
    }
}

/// Formats a string as a quoted literal with escapes.
fn format_string_literal(output: &mut String, s: &str) {
    output.push('"');
    for c in s.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_ascii_control() => write!(output, "\\x{:02x}", c as u32).unwrap(),
            c => output.push(c),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::{Value, to_ascii};
    use relish::TypeId;

    #[test]
    fn test_format_null() {
        assert_eq!(to_ascii(&Value::Null), "null");
    }

    #[test]
    fn test_format_bool() {
        assert_eq!(to_ascii(&Value::Bool(true)), "true");
        assert_eq!(to_ascii(&Value::Bool(false)), "false");
    }

    #[test]
    fn test_format_integers() {
        assert_eq!(to_ascii(&Value::U32(42)), "42u32");
        assert_eq!(to_ascii(&Value::I32(-42)), "-42i32");
        assert_eq!(to_ascii(&Value::U64(1234567890)), "1234567890u64");
    }

    #[test]
    fn test_format_string() {
        assert_eq!(to_ascii(&Value::String("Hello".to_string())), "\"Hello\"");
        assert_eq!(
            to_ascii(&Value::String("Hello\nWorld".to_string())),
            "\"Hello\\nWorld\""
        );
    }

    #[test]
    fn test_format_array() {
        let arr = Value::Array {
            element_type: TypeId::U32,
            elements: vec![Value::U32(1), Value::U32(2), Value::U32(3)],
        };
        assert_eq!(to_ascii(&arr), "array<u32> {\n  1u32,\n  2u32,\n  3u32,\n}");
    }

    #[test]
    fn test_format_struct() {
        let s = Value::Struct {
            fields: vec![(0, Value::U32(42)), (1, Value::String("test".to_string()))],
        };
        let expected = "struct {\n  0: 42u32,\n  1: \"test\",\n}";
        assert_eq!(to_ascii(&s), expected);
    }
}
