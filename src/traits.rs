use crate::buf::BytesRef;

/// Core trait for types that can be serialized and deserialized in the Relish format.
pub trait Relish: Sized {
    /// The TypeId that identifies this type in the Relish binary format.
    const TYPE: crate::TypeId;

    /// Parse a value from the given data buffer.
    ///
    /// The data buffer contains only the value portion of the TLV encoding,
    /// without the type ID or length prefix.
    ///
    /// Implementations should consume all data from the buffer or fail if the
    /// data is malformed.
    fn parse_value(data: &mut BytesRef) -> crate::ParseResult<Self>;

    /// Write the value to the buffer including length prefix for varsize types.
    fn write_value(&self, buffer: &mut Vec<u8>) -> crate::WriteResult<()>;

    /// Returns the total byte length of the serialized value including length prefix for varsize types.
    fn value_length(&self) -> usize;
}

pub trait FieldValue: Sized {
    type T: Relish;

    fn as_relish(&self) -> Option<&Self::T>;
    fn from_option(v: Option<Self::T>) -> crate::ParseResult<Self>;
}

impl<T: Relish> FieldValue for T {
    type T = T;

    fn as_relish(&self) -> Option<&Self::T> {
        Some(self)
    }

    fn from_option(v: Option<Self::T>) -> crate::ParseResult<Self> {
        v.ok_or_else(|| crate::ParseError::new(crate::ParseErrorKind::MissingRequiredField))
    }
}

impl<T: Relish> FieldValue for Option<T> {
    type T = T;

    fn as_relish(&self) -> Option<&Self::T> {
        self.as_ref()
    }

    fn from_option(v: Option<Self::T>) -> crate::ParseResult<Self> {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::FieldValue;

    #[test]
    fn test_field_value_trait() {
        let value: u32 = 42;
        assert_eq!(value.as_relish(), Some(&42));
        let result: u32 = FieldValue::from_option(Some(42)).unwrap();
        assert_eq!(result, 42);

        let opt_value: Option<u32> = Some(42);
        assert_eq!(opt_value.as_relish(), Some(&42));
        let result: Option<u32> = FieldValue::from_option(Some(42)).unwrap();
        assert_eq!(result, Some(42));

        let none_value: Option<u32> = None;
        assert_eq!(none_value.as_relish(), None);
        let result: Option<u32> = FieldValue::from_option(None).unwrap();
        assert_eq!(result, None);
    }
}
