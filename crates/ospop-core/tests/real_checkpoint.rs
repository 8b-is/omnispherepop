//! End-to-end checks against the committed real checkpoint.
//!
//! `examples/tiny-gpt2.safetensors` is a Hugging Face safetensors file (2-layer
//! GPT-2, 64 tensors, ~454 KB). The weights are a randomly initialized
//! checkpoint, so the numbers below are real measurements of that file — not
//! perplexity, and not a quality claim about any trained model.

use ospop_core::{quantize, safetensors, snapshot::Snapshot, ultragraph, Trit};

fn model_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tiny-gpt2.safetensors")
}

fn load() -> Vec<safetensors::Tensor> {
    let bytes = std::fs::read(model_path()).expect("committed checkpoint must exist");
    safetensors::read(&bytes).expect("checkpoint must parse")
}

#[test]
fn real_checkpoint_loads_with_expected_shape() {
    let tensors = load();
    assert_eq!(tensors.len(), 64);
    assert!(tensors.iter().all(|t| t.dtype == safetensors::DType::F32));
    let attn = tensors
        .iter()
        .find(|t| t.name == "transformer.h.0.attn.c_attn.weight")
        .expect("attention projection present");
    assert_eq!(attn.shape, vec![32, 96]);
    assert_eq!(attn.values.len(), 32 * 96);
}

#[test]
fn real_weights_survive_the_snapshot_boundary() {
    let tensors = load();
    let snap = Snapshot::new(safetensors::into_map(tensors)).expect("real weights are finite");
    let ids: Vec<&String> = snap.tensors().keys().collect();
    assert_eq!(ids.len(), 64);
    // Identity is content-addressed: reloading the same bytes must match.
    let again = Snapshot::new(safetensors::into_map(load())).unwrap();
    assert_eq!(snap.id(), again.id());
}

#[test]
fn ternary_compression_is_measured_not_asserted() {
    let tensors = load();
    let snap = Snapshot::new(safetensors::into_map(tensors)).unwrap();
    let weights = &snap.tensors()["transformer.h.0.attn.c_attn.weight"];

    let groups = quantize::quantize(weights, 32, None).unwrap();
    let trits: Vec<Trit> = groups
        .iter()
        .flat_map(|g| g.trits().iter().copied())
        .collect();
    let zeros = trits.iter().filter(|t| **t == Trit::Zero).count();

    let packed4 = ospop_core::ternary::pack(&trits).len();
    let packed5 = ultragraph::pack5_trits(&trits).len();
    assert!(packed5 < packed4, "5-trit packing must be denser");
    assert!(packed4 * 8 < weights.len() * 4);
    assert!(zeros > 0 && zeros < trits.len());

    let recon = quantize::reconstruct(&groups);
    let mse = recon
        .iter()
        .zip(weights)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        / weights.len() as f32;
    assert!(mse.is_finite() && mse > 0.0, "measured error was {mse}");
}

#[test]
fn qdelta_scheme_agrees_with_the_ultragraph_kernels() {
    let weights = load()
        .into_iter()
        .find(|t| t.name == "transformer.h.0.attn.c_proj.weight")
        .unwrap()
        .values;
    let trits = ultragraph::qdelta_quantize(&weights, 6).unwrap();
    assert_eq!(trits.len(), weights.len());
    let packed = ultragraph::unpack5_trits(&ultragraph::pack5_trits(&trits), trits.len()).unwrap();
    assert_eq!(packed, trits);

    let acts: Vec<i8> = (0..trits.len().min(32))
        .map(|i| (i as i8 % 5) - 2)
        .collect();
    let a = ultragraph::dot_ternary_i8(&trits[..acts.len()], &acts).unwrap();
    let b: i64 = trits[..acts.len()]
        .iter()
        .zip(&acts)
        .map(|(t, x)| match t {
            Trit::Neg => -i64::from(*x),
            Trit::Zero => 0,
            Trit::Pos => i64::from(*x),
        })
        .sum();
    assert_eq!(a, b);
}

#[test]
fn a_truncated_checkpoint_is_refused() {
    let bytes = std::fs::read(model_path()).unwrap();
    let truncated = &bytes[..bytes.len() / 2];
    // Truncating the payload must be caught as an out-of-buffer range, naming
    // the first tensor that no longer fits.
    let err = safetensors::read(truncated).unwrap_err();
    assert!(err.contains("outside buffer"), "unexpected error: {err}");

    // Truncating the header is a different failure and must say so.
    let short = safetensors::read(&bytes[..4]).unwrap_err();
    assert!(
        short.contains("8-byte header length"),
        "unexpected error: {short}"
    );
}
