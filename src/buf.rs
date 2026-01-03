use crate::{ParseError, ParseResult};

pub struct BytesRef<'a> {
    b: &'a bytes::Bytes,
    data: &'a [u8],
}

impl<'a> BytesRef<'a> {
    #[doc(hidden)]
    pub fn new(b: &'a bytes::Bytes) -> Self {
        BytesRef { b, data: b }
    }

    #[doc(hidden)]
    pub fn read(&mut self, amt: usize) -> ParseResult<Self> {
        if amt > self.len() {
            return Err(ParseError::new(crate::ParseErrorKind::InsufficientData {
                needed: amt,
                available: self.len(),
            }));
        }
        let (result, data) = self.data.split_at(amt);
        self.data = data;
        Ok(BytesRef {
            b: self.b,
            data: result,
        })
    }

    /// Returns the remaining data as a zero-copy `Bytes` slice.
    pub(crate) fn to_bytes(&self) -> bytes::Bytes {
        self.b.slice_ref(self.data)
    }
}

impl std::ops::Deref for BytesRef<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
