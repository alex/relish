use crate::{Relish, WriteResult};

/// Serialize a value to Relish binary format as a Vec<u8>.
pub fn to_vec<T: Relish>(value: &T) -> WriteResult<Vec<u8>> {
    let mut buffer = Vec::new();

    buffer.push(T::TYPE as u8);

    value.write_value(&mut buffer)?;

    Ok(buffer)
}
