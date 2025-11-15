use thiserror::Error;

/// Specific kinds of parsing errors that can occur when deserializing Relish
/// data.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("invalid type ID: {0:#02x}")]
    InvalidTypeId(u8),

    #[error("invalid field ID: {0:#02x} (top bit must not be set)")]
    InvalidFieldId(u8),

    #[error("field IDs must be in strictly increasing order: got {current} after {previous}")]
    FieldOrderViolation { previous: u8, current: u8 },

    #[error("invalid UTF-8 in string")]
    InvalidUtf8,

    #[error("duplicate map key")]
    DuplicateMapKey,

    #[error(
        "enum variant value does not consume entire content length: expected {expected} bytes, got {actual}"
    )]
    EnumContentLengthMismatch { expected: u32, actual: u32 },

    #[error("type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch { expected: u8, actual: u8 },

    #[error("invalid boolean value: {0:#02x}")]
    InvalidBoolValue(u8),

    #[error("content length extends beyond available data: need {needed} bytes, have {available}")]
    InsufficientData { needed: usize, available: usize },

    #[error("missing required field")]
    MissingRequiredField,

    #[error("extra data after parsing: {bytes_remaining} bytes remaining")]
    ExtraData { bytes_remaining: usize },

    #[error("unknown variant ID: {0}")]
    UnknownVariant(u8),
}

/// Error type returned when parsing Relish binary data fails.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("parse error: {kind}")]
pub struct ParseError {
    /// The specific kind of parse error that occurred.
    kind: ParseErrorKind,
}

impl ParseError {
    /// Creates a new ParseError with the given kind.
    pub const fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the specific kind of parse error that occurred.
    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Specific kinds of writing errors that can occur when serializing Relish data.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WriteErrorKind {
    #[error("field ID {0} is too large (top bit would be set)")]
    FieldIdTooLarge(u8),

    #[error("content length {0} exceeds maximum allowed (u32::MAX >> 1)")]
    ContentTooLarge(usize),
}

/// Error type returned when serializing to Relish binary format fails.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("write error: {kind}")]
pub struct WriteError {
    /// The specific kind of write error that occurred.
    kind: WriteErrorKind,
}

impl WriteError {
    /// Creates a new WriteError with the given kind.
    pub fn new(kind: WriteErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the specific kind of write error that occurred.
    pub fn kind(&self) -> &WriteErrorKind {
        &self.kind
    }
}

/// Result type for writing operations.
pub type WriteResult<T> = Result<T, WriteError>;
