//! Relish is a binary serialization format library designed for efficient encoding,
//! explicit backwards compatibility through field tagging, and flexible type serialization.
//!
//! The format uses Type-Length-Value (TLV) encoding with support for both fixed-size
//! and variable-size types. See the SPEC.md file for the complete format specification.
//!
//! # Examples
//!
//! ```
//! use relish::{Relish, parse, to_vec};
//! use bytes::Bytes;
//!
//! // Serialize a value to bytes
//! let value = 42u32;
//! let bytes = to_vec(&value).unwrap();
//!
//! // Deserialize bytes back to a value
//! let parsed: u32 = parse(Bytes::from(bytes)).unwrap();
//! assert_eq!(parsed, 42);
//! ```

mod buf;
mod error;
mod parse;
mod traits;
mod types;
mod write;

pub use crate::buf::BytesRef;
pub use crate::error::{
    ParseError, ParseErrorKind, ParseResult, WriteError, WriteErrorKind, WriteResult,
};
pub use crate::parse::parse;
pub use crate::traits::{FieldValue, Relish};
pub use crate::types::{Null, TypeId};
pub use crate::write::to_vec;

#[doc(hidden)]
pub use crate::parse::StructParser;

#[doc(hidden)]
pub use crate::parse::{parse_tlv, read_byte};

#[doc(hidden)]
pub use crate::types::{tagged_varint_length_size, write_tagged_varint_length};

/// Re-export of the derive macro for implementing the Relish trait on custom types.
///
/// # Example
///
/// ```
/// use relish::Relish;
///
/// #[derive(Relish)]
/// struct MyStruct {
///     #[relish(field_id = 0)]
///     name: String,
///     #[relish(field_id = 1)]
///     age: u32,
/// }
/// ```
pub use relish_derive::Relish;
