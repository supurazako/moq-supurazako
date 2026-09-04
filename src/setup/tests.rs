use super::{DecodeError, EncodeError, Setup, decode, encode};
use crate::key_value_pair::{EncodeError as KeyValueEncodeError, KeyValuePair, Value};

#[test]
fn encodes_client_setup_for_native_quic_uri() {
    let setup = Setup {
        options: vec![
            KeyValuePair {
                type_id: 0x01,
                value: Value::Bytes(b"/demo".to_vec()),
            },
            KeyValuePair {
                type_id: 0x05,
                value: Value::Bytes(b"localhost".to_vec()),
            },
        ],
    };

    assert_eq!(
        encode(&setup),
        Ok(vec![
            0xaf, 0x00, // SETUP Type
            0x00, 0x12, // Payload Length = 18
            0x01, 0x05, 0x2f, 0x64, 0x65, 0x6d, 0x6f, // PATH
            0x04, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, // AUTHORITY
        ]),
    );
}

#[test]
fn encodes_setup_without_options() {
    let setup = Setup {
        options: Vec::new(),
    };

    assert_eq!(
        encode(&setup),
        Ok(vec![
            0xaf, 0x00, // SETUP Type
            0x00, 0x00, // Payload Length = 0
        ]),
    );
}

#[test]
fn rejects_decreasing_setup_option_types() {
    let setup = Setup {
        options: vec![
            KeyValuePair {
                type_id: 0x05,
                value: Value::Bytes(b"localhost".to_vec()),
            },
            KeyValuePair {
                type_id: 0x01,
                value: Value::Bytes(b"/demo".to_vec()),
            },
        ],
    };

    assert_eq!(
        encode(&setup),
        Err(EncodeError::KeyValuePair(
            KeyValueEncodeError::TypeDecreased {
                previous: 0x05,
                current: 0x01,
            },
        )),
    );
}

#[test]
fn rejects_payload_larger_than_u16_length_field() {
    let setup = Setup {
        options: vec![KeyValuePair {
            type_id: 0x01,
            value: Value::Bytes(vec![0; usize::from(u16::MAX)]),
        }],
    };

    assert_eq!(
        encode(&setup),
        Err(EncodeError::PayloadTooLong { length: 65_539 }),
    );
}

#[test]
fn decodes_client_setup_and_reports_consumed_length() {
    let input = [
        0xaf, 0x00, // SETUP Type
        0x00, 0x12, // Payload Length = 18
        0x01, 0x05, 0x2f, 0x64, 0x65, 0x6d, 0x6f, // PATH
        0x04, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, // AUTHORITY
        0xaa, // 次のcontrol messageを想定
    ];

    assert_eq!(
        decode(&input),
        Ok((
            Setup {
                options: vec![
                    KeyValuePair {
                        type_id: 0x01,
                        value: Value::Bytes(b"/demo".to_vec()),
                    },
                    KeyValuePair {
                        type_id: 0x05,
                        value: Value::Bytes(b"localhost".to_vec()),
                    },
                ],
            },
            22,
        )),
    );
}

#[test]
fn decodes_setup_without_options() {
    let input = [0xaf, 0x00, 0x00, 0x00, 0xaa];

    assert_eq!(
        decode(&input),
        Ok((
            Setup {
                options: Vec::new(),
            },
            4,
        ))
    );
}

#[test]
fn rejects_unexpected_message_type() {
    let input = [0x01];

    assert_eq!(
        decode(&input),
        Err(DecodeError::UnexpectedMessageType { actual: 1 })
    );
}

#[test]
fn rejects_incomplete_setup_length() {
    let input = [0xaf, 0x00, 0x00];

    assert_eq!(
        decode(&input),
        Err(DecodeError::UnexpectedEnd {
            expected: 4,
            actual: 3,
        })
    );
}

#[test]
fn rejects_incomplete_setup_payload() {
    let input = [
        0xaf, 0x00, // SETUP Type
        0x00, 0x01, // Payload Length = 1
    ];

    assert_eq!(
        decode(&input),
        Err(DecodeError::UnexpectedEnd {
            expected: 5,
            actual: 4,
        })
    );
}

#[test]
fn reports_malformed_option() {
    let input = [
        0xaf, 0x00, // SETUP Type
        0x00, 0x01, // Payload Length = 1
        0x80, // 2バイトvi64の先頭だが、続きがない
        0x00, // 次のcontrol messageを想定
    ];

    assert_eq!(
        decode(&input),
        Err(DecodeError::KeyValuePair(
            crate::key_value_pair::DecodeError::Vi64(crate::vi64::DecodeError::UnexpectedEnd {
                expected: 2,
                actual: 1,
            })
        ))
    );
}
