use crate::vi64;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Integer(u64),
    Bytes(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct KeyValuePair {
    pub type_id: u64,
    pub value: Value,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EncodeError {
    TypeDecreased { previous: u64, current: u64 },
    ValueKindMismatch { type_id: u64 },
    ValueTooLong { length: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    Vi64(vi64::DecodeError),
    TypeOverflow { previous: u64, delta: u64 },
    ValueTooLong { length: u64 },
    UnexpectedEnd { expected: usize, actual: usize },
}

pub fn encode(previous_type: u64, pair: &KeyValuePair) -> Result<Vec<u8>, EncodeError> {
    let Some(delta_type) = pair.type_id.checked_sub(previous_type) else {
        return Err(EncodeError::TypeDecreased {
            previous: previous_type,
            current: pair.type_id,
        });
    };

    let mut encoded = vi64::encode(delta_type);

    match (&pair.value, pair.type_id % 2) {
        (Value::Integer(value), 0) => {
            encoded.extend(vi64::encode(*value));
        }
        (Value::Bytes(value), 1) => {
            if value.len() > usize::from(u16::MAX) {
                return Err(EncodeError::ValueTooLong {
                    length: value.len(),
                });
            }

            encoded.extend(vi64::encode(value.len() as u64));
            encoded.extend_from_slice(value);
        }
        _ => {
            return Err(EncodeError::ValueKindMismatch {
                type_id: pair.type_id,
            });
        }
    }

    Ok(encoded)
}

pub fn decode(previous_type: u64, input: &[u8]) -> Result<(KeyValuePair, usize), DecodeError> {
    let (delta_type, delta_length) = vi64::decode(input).map_err(DecodeError::Vi64)?;

    let Some(type_id) = previous_type.checked_add(delta_type) else {
        return Err(DecodeError::TypeOverflow {
            previous: previous_type,
            delta: delta_type,
        });
    };

    let remaining = &input[delta_length..];

    if type_id % 2 == 0 {
        let (value, value_length) = vi64::decode(remaining).map_err(DecodeError::Vi64)?;

        return Ok((
            KeyValuePair {
                type_id,
                value: Value::Integer(value),
            },
            delta_length + value_length,
        ));
    }

    let (value_length, length_field_size) = vi64::decode(remaining).map_err(DecodeError::Vi64)?;

    if value_length > u64::from(u16::MAX) {
        return Err(DecodeError::ValueTooLong {
            length: value_length,
        });
    }

    let value_length = value_length as usize;
    let header_length = delta_length + length_field_size;
    let expected = header_length + value_length;

    if input.len() < expected {
        return Err(DecodeError::UnexpectedEnd {
            expected,
            actual: input.len(),
        });
    }

    Ok((
        KeyValuePair {
            type_id,
            value: Value::Bytes(input[header_length..expected].to_vec()),
        },
        expected,
    ))
}

#[cfg(test)]
mod tests;
