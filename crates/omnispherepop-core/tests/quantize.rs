use omnispherepop_core::{
    quantize::{dot, quantize, reconstruct},
    ternary::Trit::*,
};

#[test]
fn reference_quantization_and_dot() {
    let q = quantize(&[-2., 0., 2.], 128, None).unwrap();
    assert_eq!(q[0].trits(), &[Neg, Zero, Pos]);
    assert_eq!(q[0].scale(), 2.);
    assert_eq!(reconstruct(&q), vec![-2., 0., 2.]);
    assert_eq!(dot(&q, &[3., 9., 4.]).unwrap(), 2.);
}
#[test]
fn zero_tails_and_strict_threshold() {
    let q = quantize(&[0., 0., 0.], 2, None).unwrap();
    assert_eq!(q.len(), 2);
    assert_eq!(q[1].trits(), &[Zero]);
    assert_eq!(q[0].scale(), 0.);
    let q = quantize(&[-1., 1., 2.], 3, Some(1.)).unwrap();
    assert_eq!(q[0].trits(), &[Zero, Zero, Pos]);
    assert!(quantize(&[], 128, None).unwrap().is_empty());
    assert_eq!(dot(&[], &[]).unwrap(), 0.);
}
#[test]
fn invalid_inputs_and_overflow_rejected() {
    assert!(quantize(&[1.], 0, None).is_err());
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(quantize(&[bad], 1, None).is_err());
        assert!(quantize(&[1.], 1, Some(bad)).is_err());
    }
    assert!(quantize(&[1.], 1, Some(-1.)).is_err());
    let q = quantize(&[f32::MAX, f32::MAX], 2, None).unwrap();
    assert!(q[0].scale().is_finite());
    assert!(dot(&q, &[2., 2.]).is_err());
    assert!(dot(&q, &[1.]).is_err());
    assert!(dot(&q, &[f32::NAN, 1.]).is_err());
}
