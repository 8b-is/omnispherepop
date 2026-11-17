//! Bridge to the [ultra-graph](https://github.com/8b-is/ultra-graph) kernels,
//! vendored as a git submodule at `third_party/ultra-graph`.
//!
//! Two things come from `zerocpu`:
//!
//! * **5-trit packing** — `3^5 = 243 < 256`, so five trits fit one byte (1.6 bits
//!   per value) instead of the 4-per-byte codec in [`crate::ternary`].
//! * **A6P Q-delta thresholding** — integer thresholds over Q16.16 values, which
//!   is a genuinely different calibration from Δ = 0.7·mean|w|.
//!
//! Both are used by `Program::Quantize` with `--scheme qdelta` and by
//! `quantize::dot_i8`, so the submodule is on the live path rather than vendored
//! for show.

use crate::ternary::Trit;
use zerocpu::ternary::{pack5, unpack5};
use zerocpu::trit;

/// Trits per byte for the ultra-graph packing (1.6 bits per value).
pub const TRITS_PER_BYTE: usize = zerocpu::ternary::TRITS_PER_BYTE;

/// Number of A6P detail levels.
pub const QDELTA_LEVELS: usize = zerocpu::trit::A6P_LEVELS;

/// Q16.16 fixed-point scale.
const Q: f64 = 65_536.0;

fn to_trit(v: i8) -> Result<Trit, String> {
    match v {
        -1 => Ok(Trit::Neg),
        0 => Ok(Trit::Zero),
        1 => Ok(Trit::Pos),
        other => Err(format!("non-ternary code from zerocpu: {other}")),
    }
}

fn to_code(t: Trit) -> i8 {
    match t {
        Trit::Neg => -1,
        Trit::Zero => 0,
        Trit::Pos => 1,
    }
}

/// Pack trits 5-per-byte via zerocpu. Panics inside zerocpu on non-ternary input,
/// so the range check happens here first.
pub fn pack5_trits(trits: &[Trit]) -> Vec<u8> {
    let codes: Vec<i8> = trits.iter().map(|t| to_code(*t)).collect();
    pack5(&codes)
}

/// Unpack 5-per-byte, rejecting wrong lengths and reserved codes.
pub fn unpack5_trits(bytes: &[u8], len: usize) -> Result<Vec<Trit>, String> {
    let expected = len.div_ceil(TRITS_PER_BYTE);
    if bytes.len() != expected {
        return Err(format!(
            "packed length mismatch: {} bytes for {len} trits",
            bytes.len()
        ));
    }
    unpack5(bytes, len).iter().map(|v| to_trit(*v)).collect()
}

/// Fixed-point Q16.16 threshold for an A6P detail level, as an `f32`.
pub fn qdelta_threshold(level: usize) -> Result<f32, String> {
    if level >= QDELTA_LEVELS {
        return Err(format!("level must be < {QDELTA_LEVELS}, got {level}"));
    }
    Ok(trit::delta_for_level(level).map_err(|e| e.to_string())? as f32 / Q as f32)
}

/// Quantize with the ultra-graph A6P rule: values are converted to Q16.16, then
/// thresholded at `delta_for_level`. Inclusive comparisons, unlike the core's
/// strict Δ rule, so a value exactly at the threshold survives.
pub fn qdelta_quantize(weights: &[f32], level: usize) -> Result<Vec<Trit>, String> {
    let delta = trit::delta_for_level(level).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(weights.len());
    for w in weights {
        if !w.is_finite() {
            return Err("nonfinite weight".into());
        }
        let fixed = (f64::from(*w) * Q).round();
        if fixed.abs() > i64::MAX as f64 / 2.0 {
            return Err("weight out of Q16.16 range".into());
        }
        out.push(to_trit(
            trit::quantize(fixed as i64, delta).map_err(|e| e.to_string())?,
        )?);
    }
    Ok(out)
}

/// Ternary dot product over int8 activations, accumulated in i64 by zerocpu.
pub fn dot_ternary_i8(weights: &[Trit], activations: &[i8]) -> Result<i64, String> {
    if weights.len() != activations.len() {
        return Err("ternary dot length mismatch".into());
    }
    let w: Vec<i8> = weights.iter().map(|t| to_code(*t)).collect();
    Ok(zerocpu::ternary::dot_ternary(&w, activations))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack5_round_trips_and_rejects_bad_input() {
        let trits: Vec<Trit> = (0..37)
            .map(|i| match i % 3 {
                0 => Trit::Neg,
                1 => Trit::Zero,
                _ => Trit::Pos,
            })
            .collect();
        let packed = pack5_trits(&trits);
        assert_eq!(packed.len(), 8, "5 trits per byte");
        assert_eq!(unpack5_trits(&packed, trits.len()).unwrap(), trits);
        assert!(unpack5_trits(&packed[..7], trits.len()).is_err());
        assert!(unpack5_trits(&packed, trits.len() + TRITS_PER_BYTE).is_err());
        // 37 trits need 8 bytes; 38 still fits, 42 does not.
        assert!(unpack5_trits(&packed, 38).is_ok());
    }

    #[test]
    fn pack5_is_denser_than_the_core_codec() {
        let trits = vec![Trit::Pos; 512];
        assert!(pack5_trits(&trits).len() < crate::ternary::pack(&trits).len());
    }

    #[test]
    fn qdelta_thresholds_follow_the_ultragraph_table() {
        assert_eq!(qdelta_threshold(0).unwrap(), 0.25);
        assert_eq!(qdelta_threshold(14).unwrap(), 1.0 / Q as f32);
        assert!(qdelta_threshold(QDELTA_LEVELS).is_err());
    }

    #[test]
    fn qdelta_quantizes_inclusive_at_the_threshold() {
        // Level 0 has delta 0.25; exactly 0.25 must survive as +1.
        let t = qdelta_quantize(&[0.25, 0.2499, -0.25, 0.0], 0).unwrap();
        assert_eq!(t, vec![Trit::Pos, Trit::Zero, Trit::Neg, Trit::Zero]);
        assert!(qdelta_quantize(&[f32::NAN], 0).is_err());
        assert!(qdelta_quantize(&[1.0], QDELTA_LEVELS).is_err());
    }

    #[test]
    fn dot_ternary_matches_scalar_arithmetic() {
        let w = vec![Trit::Pos, Trit::Neg, Trit::Zero, Trit::Pos];
        let x = [2i8, 3, 9, -5];
        // 1*2 + -1*3 + 0*9 + 1*-5
        assert_eq!(dot_ternary_i8(&w, &x).unwrap(), 2 - 3 - 5);
        assert!(dot_ternary_i8(&w, &x[..2]).is_err());
    }
}
