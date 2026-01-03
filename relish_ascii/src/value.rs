use relish::TypeId;

/// A schemaless representation of any relish value.
///
/// This type can represent any valid relish value without requiring
/// a schema or type information at compile time.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value (no content).
    Null,

    /// Boolean value.
    Bool(bool),

    /// Unsigned 8-bit integer.
    U8(u8),

    /// Unsigned 16-bit integer.
    U16(u16),

    /// Unsigned 32-bit integer.
    U32(u32),

    /// Unsigned 64-bit integer.
    U64(u64),

    /// Unsigned 128-bit integer.
    U128(u128),

    /// Signed 8-bit integer.
    I8(i8),

    /// Signed 16-bit integer.
    I16(i16),

    /// Signed 32-bit integer.
    I32(i32),

    /// Signed 64-bit integer.
    I64(i64),

    /// Signed 128-bit integer.
    I128(i128),

    /// 32-bit floating point.
    F32(f32),

    /// 64-bit floating point.
    F64(f64),

    /// UTF-8 string.
    String(String),

    /// Homogeneous array of values.
    Array {
        /// The element type ID.
        element_type: TypeId,
        /// The array elements.
        elements: Vec<Value>,
    },

    /// Map of key-value pairs.
    Map {
        /// The key type ID.
        key_type: TypeId,
        /// The value type ID.
        value_type: TypeId,
        /// The key-value pairs.
        entries: Vec<(Value, Value)>,
    },

    /// Struct with named fields.
    Struct {
        /// The fields as (field_id, value) pairs.
        fields: Vec<(u8, Value)>,
    },

    /// Enum variant.
    Enum {
        /// The variant ID.
        variant_id: u8,
        /// The variant value.
        value: Box<Value>,
    },

    /// Unix timestamp (seconds since epoch).
    Timestamp(u64),
}
