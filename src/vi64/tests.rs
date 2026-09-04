use super::{DecodeError, decode, encode};

#[test]
fn encodes_one_byte_value() {
    assert_eq!(encode(37), vec![0x25]);
}

#[test]
fn encodes_largest_one_byte_value() {
    assert_eq!(encode(127), vec![0x7f]);
}

#[test]
fn encodes_smallest_two_byte_value() {
    assert_eq!(encode(128), vec![0x80, 0x80]);
}

#[test]
fn encodes_largest_two_byte_value() {
    assert_eq!(encode(16_383), vec![0xbf, 0xff]);
}

#[test]
fn encodes_setup_message_type() {
    assert_eq!(encode(0x2f00), vec![0xaf, 0x00]);
}

#[test]
fn encodes_smallest_three_byte_value() {
    assert_eq!(encode(16_384), vec![0xc0, 0x40, 0x00]);
}

#[test]
fn encodes_largest_nine_byte_value() {
    assert_eq!(
        encode(u64::MAX),
        vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,],
    );
}

#[test]
fn matches_draft_19_examples_and_five_byte_boundary() {
    let cases: &[(u64, &[u8])] = &[
        (226_442_877, &[0xed, 0x7f, 0x3e, 0x7d]),
        (268_435_456, &[0xf0, 0x10, 0x00, 0x00, 0x00]),
        (2_893_212_287_960, &[0xfa, 0xa1, 0xa0, 0xe4, 0x03, 0xd8]),
        (
            151_288_809_941_952,
            &[0xfc, 0x89, 0x98, 0xab, 0xc6, 0x6b, 0xc0],
        ),
        (
            70_423_237_261_249_041,
            &[0xfe, 0xfa, 0x31, 0x8f, 0xa8, 0xe3, 0xca, 0x11],
        ),
    ];

    for &(value, expected) in cases {
        let actual = encode(value);
        assert_eq!(actual.as_slice(), expected, "value = {value}");
    }
}

#[test]
fn decodes_one_byte_value_and_reports_consumed_length() {
    assert_eq!(decode(&[0x25, 0xaa]), Ok((37, 1)));
}

#[test]
fn rejects_empty_input() {
    assert_eq!(decode(&[]), Err(DecodeError::Empty));
}

#[test]
fn decodes_setup_message_type_and_reports_consumed_length() {
    assert_eq!(decode(&[0xaf, 0x00, 0xaa]), Ok((0x2f00, 2)));
}

#[test]
fn rejects_incomplete_two_byte_value() {
    assert_eq!(
        decode(&[0x80]),
        Err(DecodeError::UnexpectedEnd {
            expected: 2,
            actual: 1,
        }),
    );
}

#[test]
fn decodes_non_minimal_encoding() {
    assert_eq!(decode(&[0x80, 0x25, 0xaa]), Ok((37, 2)));
}

#[test]
fn round_trips_values_at_each_length_boundary() {
    for usable_bits in [7, 14, 21, 28, 35, 42, 49, 56] {
        let largest = (1_u64 << usable_bits) - 1;

        for value in [largest, largest + 1] {
            let encoded = encode(value);

            assert_eq!(
                decode(&encoded),
                Ok((value, encoded.len())),
                "value = {value}",
            );
        }
    }

    let encoded = encode(u64::MAX);
    assert_eq!(decode(&encoded), Ok((u64::MAX, encoded.len())));
}
