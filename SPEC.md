# Relish Serialization Format Specification

## Overview

Relish is a binary serialization format designed for efficient encoding (omitting field names from structs), explicit backwards compatibility (through field tagging), and flexible type serialization.

## Binary Layout

All values are encoded as Type-[Length]-Value (T[L]V) structures. The format distinguishes between varsize and fixed-size types based on the Type ID.

### Type ID

Every value begins with a 1-byte Type ID:
- The top bit (bit 7) is reserved and must not be set
- The remaining 7 bits identify the type

### Type Categories

Types are divided into two categories:
- **Varsize types**: Include a tagged varint length prefix after the Type ID
- **Fixed-size types**: Have a predetermined size based on the Type ID

### Tagged Varint Length Encoding

Varsize types use a tagged varint encoding for length prefixes:
- If the lowest bit (bit 0) is **0**: The remaining 7 bits encode the length (0-127 bytes)
  - Format: `[0bXXXXXXX0]` where X bits represent the length
- If the lowest bit (bit 0) is **1**: A 4-byte encoding follows
  - Format: `[0bXXXXXXX1] [byte1] [byte2] [byte3]`
  - The remaining 31 bits (7 bits from first byte + 24 bits from next 3 bytes) encode the length in little-endian order
  - Maximum length: 2³¹-1 bytes (approximately 2.1 GB)

## Type IDs

| Type | Type ID | Category |
|------|---------|----------|
| Null | 0x00 | Fixed |
| Bool | 0x01 | Fixed |
| u8 | 0x02 | Fixed |
| u16 | 0x03 | Fixed |
| u32 | 0x04 | Fixed |
| u64 | 0x05 | Fixed |
| u128 | 0x06 | Fixed |
| i8 | 0x07 | Fixed |
| i16 | 0x08 | Fixed |
| i32 | 0x09 | Fixed |
| i64 | 0x0A | Fixed |
| i128 | 0x0B | Fixed |
| f32 | 0x0C | Fixed |
| f64 | 0x0D | Fixed |
| String | 0x0E | Varsize |
| Array | 0x0F | Varsize |
| Map | 0x10 | Varsize |
| Struct | 0x11 | Varsize |
| Enum | 0x12 | Varsize |
| Timestamp | 0x13 | Fixed |

## Fixed-Size Types

### Null
- Type ID only, no content bytes

### Bool
- 1 byte: `0x00` for false, `0xFF` for true

### Integer Types (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128)
- Little-endian encoded
- Sizes: 1, 2, 4, 8, or 16 bytes respectively
- Types are not interchangeable

### Floating Point Types (f32, f64)
- Little-endian encoded IEEE 754 bytes
- Sizes: 4 or 8 bytes respectively

### Timestamp
- 8 bytes: little-endian encoded unsigned 64-bit Unix timestamp
- Represents seconds since the Unix epoch (January 1, 1970, 00:00:00 UTC)
- Does not include subsecond precision

## Varsize Types

For all varsize types, the tagged varint length prefix contains the byte count of the content that follows, not including the Type ID or the length prefix itself.

### String
- Type ID: 0x0E
- Tagged varint length prefix (content byte count)
- UTF-8 encoded bytes

Format: `[0x0E] [tagged_varint_length] [UTF-8 bytes]`

### Array
- Type ID: 0x0F
- Tagged varint length prefix (total content byte count)
- 1-byte element Type ID
- Repeated element values (length-value for varsize elements, value-only for fixed-size)

Format: `[0x0F] [tagged_varint_length] [element_type_id] [element_0] [element_1] ...`

Example (array of strings):
```
[0x0F] [tagged_varint_length] [0x0E] [str0_len] [str0_bytes] [str1_len] [str1_bytes] ...
```

### Map
- Type ID: 0x10
- Tagged varint length prefix (total content byte count)
- 1-byte key Type ID
- 1-byte value Type ID
- Repeated key-value pairs (each as length-value for varsize types, value-only for fixed-size)
- Key-value pairs are unordered
- Map keys must be unique (parsing error if duplicate keys are encountered)

Format: `[0x10] [tagged_varint_length] [key_type_id] [value_type_id] [key_0] [value_0] [key_1] [value_1] ...`

### Struct
- Type ID: 0x11
- Tagged varint length prefix (total content byte count)
- Repeated field encodings:
  - 1-byte Field ID (top bit reserved, must not be set)
  - Field value as T[L]V
- Field IDs must appear in strictly increasing order
- Parsing error if Field IDs are not in order
- Fields with `Option<T>` type that are `None` are omitted entirely (Field ID not encoded)
- Fields with `Option<T>` type that are `Some(value)` are encoded normally
- Unknown Field IDs are ignored during parsing

Format: `[0x11] [tagged_varint_length] [field_id_0] [field_value_0] [field_id_1] [field_value_1] ...`

Example (struct with one u32 field at field ID 0):
```
[0x11] [0x0C] [0x00] [0x04] [u32_bytes]
```

### Enum
- Type ID: 0x12
- Tagged varint length prefix (total content byte count)
- 1-byte variant Field ID (same semantics as struct Field ID)
- Variant value as T[L]V
- The variant value must consume the entirety of the enum's content length

Format: `[0x12] [tagged_varint_length] [variant_id] [variant_value]`

## Parsing Requirements

- Top bit of Type IDs must not be set (parsing error if set)
- Top bit of Field IDs must not be set (parsing error if set)
- Struct Field IDs must be in strictly increasing order (parsing error otherwise)
- Map keys must be unique (parsing error if duplicate keys encountered)
- String contents must be valid UTF-8 (parsing error otherwise)
- Enum variant value must exactly consume the declared content length
- Unknown struct fields must be ignored (forward compatibility)
