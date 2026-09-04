pub fn encode(value: u64) -> Vec<u8> {
    let significant_bits = (u64::BITS - value.leading_zeros()).max(1);

    let length = if significant_bits <= 56 {
        significant_bits.div_ceil(7) as usize
    } else {
        9
    };

    if length == 9 {
        let mut encoded = Vec::with_capacity(9);
        encoded.push(0xff);
        encoded.extend_from_slice(&value.to_be_bytes());
        return encoded;
    }

    let bytes = value.to_be_bytes();
    let mut encoded = bytes[8 - length..].to_vec();

    if length > 1 {
        encoded[0] |= u8::MAX << (9 - length);
    }

    encoded
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    Empty,
    UnexpectedEnd { expected: usize, actual: usize },
}

pub fn decode(input: &[u8]) -> Result<(u64, usize), DecodeError> {
    let Some(&first) = input.first() else {
        return Err(DecodeError::Empty);
    };

    let length = first.leading_ones() as usize + 1;

    if input.len() < length {
        return Err(DecodeError::UnexpectedEnd {
            expected: length,
            actual: input.len(),
        });
    }

    let mut value = if length < 8 {
        u64::from(first & (u8::MAX >> length))
    } else {
        0
    };

    for &byte in &input[1..length] {
        value = (value << 8) | u64::from(byte);
    }

    Ok((value, length))
}

#[cfg(test)]
mod tests;
