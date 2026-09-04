use crate::{
    key_value_pair::{self, KeyValuePair},
    vi64,
};

pub const MESSAGE_TYPE: u64 = 0x2f00;

#[derive(Debug, PartialEq, Eq)]
pub struct Setup {
    pub options: Vec<KeyValuePair>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EncodeError {
    KeyValuePair(key_value_pair::EncodeError),
    PayloadTooLong { length: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    Vi64(vi64::DecodeError),
    UnexpectedMessageType { actual: u64 },
    UnexpectedEnd { expected: usize, actual: usize },
    KeyValuePair(key_value_pair::DecodeError),
}

pub fn encode(setup: &Setup) -> Result<Vec<u8>, EncodeError> {
    let mut payload = Vec::new();
    let mut previous_type = 0;

    for option in &setup.options {
        let encoded_option =
            key_value_pair::encode(previous_type, option).map_err(EncodeError::KeyValuePair)?;

        payload.extend(encoded_option);
        previous_type = option.type_id;
    }

    if payload.len() > usize::from(u16::MAX) {
        return Err(EncodeError::PayloadTooLong {
            length: payload.len(),
        });
    }

    let mut encoded = vi64::encode(MESSAGE_TYPE);
    encoded.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    encoded.extend(payload);

    Ok(encoded)
}

pub fn decode(input: &[u8]) -> Result<(Setup, usize), DecodeError> {
    let (message_type, type_length) = vi64::decode(input).map_err(DecodeError::Vi64)?;

    if message_type != MESSAGE_TYPE {
        return Err(DecodeError::UnexpectedMessageType {
            actual: message_type,
        });
    }

    let header_length = type_length + 2;

    if input.len() < header_length {
        return Err(DecodeError::UnexpectedEnd {
            expected: header_length,
            actual: input.len(),
        });
    }

    let payload_length = usize::from(u16::from_be_bytes([
        input[type_length],
        input[type_length + 1],
    ]));

    let expected = header_length + payload_length;

    if input.len() < expected {
        return Err(DecodeError::UnexpectedEnd {
            expected,
            actual: input.len(),
        });
    }

    let payload = &input[header_length..expected];
    let mut options = Vec::new();
    let mut offset = 0;
    let mut previous_type = 0;

    while offset < payload.len() {
        let (option, consumed) = key_value_pair::decode(previous_type, &payload[offset..])
            .map_err(DecodeError::KeyValuePair)?;

        previous_type = option.type_id;
        options.push(option);
        offset += consumed;
    }

    Ok((Setup { options }, expected))
}

#[cfg(test)]
mod tests;
