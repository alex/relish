use crate::buf::BytesRef;
use crate::types::TypeLength;
use crate::{ParseError, ParseErrorKind, ParseResult, Relish, TypeId};
use bytes::Bytes;

/// Parse a value from Relish binary format.
pub fn parse<T: Relish>(data: Bytes) -> ParseResult<T> {
    let mut data = BytesRef::new(&data);
    let result = parse_tlv::<T>(&mut data)?;

    if !data.is_empty() {
        return Err(ParseError::new(ParseErrorKind::ExtraData {
            bytes_remaining: data.len(),
        }));
    }

    Ok(result)
}

#[doc(hidden)]
pub fn parse_tlv<T: Relish>(data: &mut BytesRef) -> ParseResult<T> {
    let type_id = TypeId::read_for_type::<T>(data)?;
    parse_value_for_typeid(data, type_id)
}

pub(crate) fn parse_value_for_typeid<T: Relish>(
    data: &mut BytesRef,
    type_id: TypeId,
) -> ParseResult<T> {
    let mut value_data = read_value_for_typeid(data, type_id)?;
    T::parse_value(&mut value_data)
}

fn read_value_for_typeid<'a>(
    data: &mut BytesRef<'a>,
    type_id: TypeId,
) -> ParseResult<BytesRef<'a>> {
    match type_id.length() {
        TypeLength::Fixed(size) => data.read(size),
        TypeLength::Varsize => {
            let length = read_tagged_varint_length(data)? as usize;
            data.read(length)
        }
    }
}

#[doc(hidden)]
pub fn read_byte(data: &mut BytesRef) -> ParseResult<u8> {
    let bytes = data.read(1)?;
    Ok(bytes[0])
}

pub(crate) fn read_tagged_varint_length(data: &mut BytesRef) -> ParseResult<u32> {
    let first_byte = read_byte(data)?;

    if first_byte & 0x01 == 0 {
        Ok((first_byte >> 1) as u32)
    } else {
        let remaining_bytes = data.read(3)?;
        let full_value = u32::from_le_bytes([
            first_byte,
            remaining_bytes[0],
            remaining_bytes[1],
            remaining_bytes[2],
        ]);
        Ok(full_value >> 1)
    }
}

#[doc(hidden)]
pub struct StructParser<'a, 'b> {
    data: &'a mut BytesRef<'b>,
    last_seen_field_id: Option<u8>,
}

impl<'a, 'b> StructParser<'a, 'b> {
    #[doc(hidden)]
    pub fn new(data: &'a mut BytesRef<'b>) -> Self {
        StructParser {
            data,
            last_seen_field_id: None,
        }
    }

    fn peek_field_id(&self) -> ParseResult<Option<u8>> {
        if self.data.is_empty() {
            return Ok(None);
        }

        let field_id = self.data[0];
        if field_id & 0x80 != 0 {
            return Err(ParseError::new(ParseErrorKind::InvalidFieldId(field_id)));
        }

        Ok(Some(field_id))
    }

    fn skip_current_field(&mut self) -> ParseResult<()> {
        let _field_id = read_byte(self.data)?;
        let type_byte = read_byte(self.data)?;
        let type_id = TypeId::from_byte(type_byte)
            .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidTypeId(type_byte)))?;
        let _ = read_value_for_typeid(self.data, type_id)?;
        Ok(())
    }

    #[doc(hidden)]
    pub fn read_value_for_field_id<T: crate::FieldValue>(
        &mut self,
        target_field_id: u8,
    ) -> ParseResult<Option<T::T>> {
        while let Some(field_id) = self.peek_field_id()? {
            if let Some(last_id) = self.last_seen_field_id
                && field_id <= last_id
            {
                return Err(ParseError::new(ParseErrorKind::FieldOrderViolation {
                    previous: last_id,
                    current: field_id,
                }));
            }

            if field_id < target_field_id {
                self.last_seen_field_id = Some(field_id);
                self.skip_current_field()?;
            } else if field_id == target_field_id {
                self.last_seen_field_id = Some(field_id);
                let _field_id = read_byte(self.data)?;
                let type_id = TypeId::read_for_type::<T::T>(self.data)?;
                let mut value_data = read_value_for_typeid(self.data, type_id)?;
                return Ok(Some(T::T::parse_value(&mut value_data)?));
            } else {
                return Ok(None);
            }
        }

        Ok(None)
    }

    #[doc(hidden)]
    pub fn finish(&mut self) -> ParseResult<()> {
        while let Some(field_id) = self.peek_field_id()? {
            if let Some(last_id) = self.last_seen_field_id
                && field_id <= last_id
            {
                return Err(ParseError::new(ParseErrorKind::FieldOrderViolation {
                    previous: last_id,
                    current: field_id,
                }));
            }
            self.skip_current_field()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ParseError, ParseErrorKind, parse};

    #[test]
    fn test_extra_data() {
        // Valid u32 (0x04 type ID, then 42 in LE) followed by extra bytes
        let data = &[0x04u8, 0x2A, 0x00, 0x00, 0x00, 0xFF, 0xEE];
        let result = parse::<u32>(bytes::Bytes::from(data.to_vec()));
        assert_eq!(
            result,
            Err(ParseError::new(ParseErrorKind::ExtraData {
                bytes_remaining: 2
            }))
        );
    }
}
