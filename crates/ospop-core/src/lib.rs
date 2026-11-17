#![forbid(unsafe_code)]
//! Omnispherepop reference primitives: trits, PTQ, ultragraphs, snapshots.
pub mod graph;
pub mod quantize;
pub mod safetensors;
pub mod snapshot;
pub mod ternary;

pub use ternary::Trit;
pub mod ultragraph;
