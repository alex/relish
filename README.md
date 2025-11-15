# Relish

Relish is a binary serialization format and Rust library, designed for efficiency and backwards compatibility.

## Stability

This project is in early development. The format specification has not been finalized and may change in backwards-incompatible ways.

## The Relish Format

Relish is a Type-Length-Value (TLV) binary serialization format that prioritizes:

- **Efficiency**
- **Backwards Compatibility**
- **Simplicity**

Relish draws inspiration from ASN.1 DER, protobufs, and many other formats.

For detailed format specifications, see [SPEC.md](SPEC.md).

### Supported Types

#### Fixed-Size Types
- `Null` - Null value (0 bytes)
- `bool` - Boolean values (1 byte)
- `u8`, `u16`, `u32`, `u64`, `u128` - Unsigned integers
- `i8`, `i16`, `i32`, `i64`, `i128` - Signed integers
- `f32`, `f64` - Floating-point numbers

#### Variable-Size Types
- `String` - UTF-8 encoded text
- `Array` - Sequences of any Relish type
- `Map` - Key-value mappings
- `Struct` - Structured data with field IDs
- `Enum` - Tagged union types

### Schema Evolution

Relish supports backwards-compatible schema changes through field versioning:

- **Adding fields**: New optional fields can always be safely added to a struct.
- **Remove fields**: A field can always be made optional to allow it to be omitted.

## Rust Implementation

The reference implementation is a Rust library that provides a safe, idiomatic API for working with the Relish format.

### Features

- **Derive Macro** - Automatic implementation for custom structs and enums
- **Zero-Copy Parsing** - Efficient deserialization using the `bytes` crate

### Installation

```
$ cargo add relish
```

### Quick Start

#### Basic Usage

```rust
use bytes::Bytes;

// Serialize a value to bytes
let value = 42u32;
let bytes = relish::to_vec(&value).unwrap();

// Deserialize bytes back to a value
let parsed: u32 = relish::parse(Bytes::from(bytes)).unwrap();
assert_eq!(parsed, 42);
```

#### Derive (Struct)

```rust
#[derive(relish::Relish)]
struct Person {
    #[relish(field_id = 0)]
    name: String,
    #[relish(field_id = 1)]
    age: u32,
    #[relish(field_id = 2)]
    email: Option<String>,
}

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: Some("alice@example.com".to_string()),
};

let bytes = relish::to_vec(&person).unwrap();
let parsed: Person = relish::parse(bytes.into()).unwrap();
```

#### Derive (Enum)

```rust
#[derive(relish::Relish)]
enum Status {
    #[relish(variant_id = 0)]
    Active,
    #[relish(variant_id = 1)]
    Inactive(String),
    #[relish(variant_id = 2)]
    Pending(u32),
}
```
