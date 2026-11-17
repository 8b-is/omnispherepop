use omnispherepop_core::ternary::{pack, unpack, Trit::*};

#[test]
fn known_bytes() {
    let values = [Neg, Zero, Pos, Neg, Pos];
    assert_eq!(pack(&values), vec![0x92, 0x01]);
    assert_eq!(unpack(&[0x92, 1], 5).unwrap(), values);
}

#[test]
fn exhaustive_short_sequences() {
    for len in 0..=7u32 {
        for mut n in 0..3usize.pow(len) {
            let values: Vec<_> = (0..len)
                .map(|_| {
                    let t = [Neg, Zero, Pos][n % 3];
                    n /= 3;
                    t
                })
                .collect();
            assert_eq!(unpack(&pack(&values), values.len()).unwrap(), values);
        }
    }
}

#[test]
fn malformed_encoding_rejected() {
    for (bytes, len) in [
        (&[3][..], 1),
        (&[4][..], 1),
        (&[0][..], 0),
        (&[][..], 1),
        (&[][..], usize::MAX),
    ] {
        assert!(unpack(bytes, len).is_err());
    }
}
