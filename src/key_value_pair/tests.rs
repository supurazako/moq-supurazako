use super::{DecodeError, EncodeError, KeyValuePair, Value, decode, encode};

#[test]
fn encodes_path_after_initial_type() {
    let path = KeyValuePair {
        type_id: 0x01,
        value: Value::Bytes(b"/demo".to_vec()),
    };

    assert_eq!(
        encode(0, &path),
        Ok(vec![0x01, 0x05, 0x2f, 0x64, 0x65, 0x6d, 0x6f]),
    );
}

#[test]
fn encodes_authority_after_path_type() {
    let authority = KeyValuePair {
        type_id: 0x05,
        value: Value::Bytes(b"localhost".to_vec()),
    };

    assert_eq!(
        encode(0x01, &authority),
        Ok(vec![
            0x04, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74,
        ]),
    );
}

#[test]
fn encodes_integer_value_for_even_type() {
    let max_cache_size = KeyValuePair {
        type_id: 0x04,
        value: Value::Integer(128),
    };

    assert_eq!(encode(0x03, &max_cache_size), Ok(vec![0x01, 0x80, 0x80]));
}

#[test]
fn rejects_decreasing_type() {
    let path = KeyValuePair {
        type_id: 0x01,
        value: Value::Bytes(b"/demo".to_vec()),
    };

    assert_eq!(
        encode(0x05, &path),
        Err(EncodeError::TypeDecreased {
            previous: 0x05,
            current: 0x01,
        }),
    );
}

#[test]
fn rejects_value_kind_mismatch() {
    let pairs = [
        KeyValuePair {
            type_id: 0x01,
            value: Value::Integer(5),
        },
        KeyValuePair {
            type_id: 0x04,
            value: Value::Bytes(b"bytes".to_vec()),
        },
    ];

    for pair in pairs {
        let type_id = pair.type_id;

        assert_eq!(
            encode(0, &pair),
            Err(EncodeError::ValueKindMismatch { type_id }),
        );
    }
}

#[test]
fn rejects_byte_value_larger_than_u16_limit() {
    let length = usize::from(u16::MAX) + 1;
    let pair = KeyValuePair {
        type_id: 0x01,
        value: Value::Bytes(vec![0; length]),
    };

    assert_eq!(encode(0, &pair), Err(EncodeError::ValueTooLong { length }));
}

#[test]
fn decodes_path_and_reports_consumed_length() {
    let input = [0x01, 0x05, 0x2f, 0x64, 0x65, 0x6d, 0x6f, 0xaa];

    assert_eq!(
        decode(0, &input),
        Ok((
            KeyValuePair {
                type_id: 0x01,
                value: Value::Bytes(b"/demo".to_vec()),
            },
            7,
        )),
    );
}

#[test]
fn decodes_authority_after_path_type() {
    let input = [
        0x04, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0xaa,
    ];

    assert_eq!(
        decode(0x01, &input),
        Ok((
            KeyValuePair {
                type_id: 0x05,
                value: Value::Bytes(b"localhost".to_vec()),
            },
            11,
        )),
    );
}

#[test]
fn decodes_integer_value_for_even_type() {
    let input = [0x01, 0x80, 0x80, 0xaa];

    assert_eq!(
        decode(0x03, &input),
        Ok((
            KeyValuePair {
                type_id: 0x04,
                value: Value::Integer(128),
            },
            3,
        )),
    );
}

#[test]
fn rejects_type_overflow_while_decoding() {
    assert_eq!(
        decode(u64::MAX, &[0x01]),
        Err(DecodeError::TypeOverflow {
            previous: u64::MAX,
            delta: 1,
        }),
    );
}

#[test]
fn rejects_incomplete_byte_value() {
    let input = [
        0x01, // Delta Type = 1
        0x05, // Length = 5
        0x2f, 0x64, // Valueは2 byteしか届いていない
    ];

    assert_eq!(
        decode(0, &input),
        Err(DecodeError::UnexpectedEnd {
            expected: 7,
            actual: 4,
        }),
    );
}

#[test]
fn rejects_byte_value_length_larger_than_u16_limit() {
    let input = [
        0x01, // Delta Type = 1
        0xc1, 0x00, 0x00, // Length = 65536
    ];

    assert_eq!(
        decode(0, &input),
        Err(DecodeError::ValueTooLong { length: 65_536 }),
    );
}

#[test]
fn reports_vi64_error_for_empty_input() {
    assert_eq!(
        decode(0, &[]),
        Err(DecodeError::Vi64(crate::vi64::DecodeError::Empty)),
    );
}
