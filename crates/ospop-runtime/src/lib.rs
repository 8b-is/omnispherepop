//! Typed program surface: one operation set, many adapters.
//! CLI, MCP, and WASM all construct a `Program` and call `Runtime::execute`.

use ospop_academy::{self, Lesson};
use ospop_core::{
    quantize,
    snapshot::{Edit, Snapshot},
    ternary::{self, Trit},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scheme {
    Ternary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Threshold {
    Auto,
    Fixed(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Program {
    /// Quantize a named tensor slice of the session snapshot.
    Quantize {
        tensor: String,
        group_size: usize,
        threshold: Threshold,
    },
    /// Structural view of one tensor slice.
    Lens { tensor: String, view: LensView },
    /// Diff the session snapshot against an overlay of one tensor.
    Diff { tensor: String, scale: f32 },
    /// Prune ultra-edges of the snapshot graph by reachability from a root.
    Prune { root: String },
    /// Curriculum listing or a single lesson lab.
    Learn { lesson: Option<String> },
    /// Explain one topic in plain language.
    Explain { topic: String },
    /// Snapshot identity and tensor summary.
    Inspect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LensView {
    Ternary,
    Heatmap,
    Graph,
    Diff,
    SparsityHistogram,
}

impl LensView {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ternary" => Some(Self::Ternary),
            "heatmap" => Some(Self::Heatmap),
            "graph" => Some(Self::Graph),
            "diff" => Some(Self::Diff),
            "sparsity-histogram" => Some(Self::SparsityHistogram),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Ternary => "ternary",
            Self::Heatmap => "heatmap",
            Self::Graph => "graph",
            Self::Diff => "diff",
            Self::SparsityHistogram => "sparsity-histogram",
        }
    }
}

/// Runtime holds session state; operations are pure functions of it.
#[derive(Debug, Default)]
pub struct Runtime {
    snapshot: Option<Snapshot>,
}

impl Runtime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the bundled demo snapshot. Model file IO is a separate adapter.
    pub fn load_demo(&mut self) -> Result<(), String> {
        self.snapshot = Some(ospop_academy::demo_snapshot()?);
        Ok(())
    }

    pub fn load_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshot = Some(snapshot);
    }

    fn snapshot(&self) -> Result<&Snapshot, String> {
        self.snapshot
            .as_ref()
            .ok_or_else(|| "no model loaded".to_string())
    }

    fn tensor(&self, name: &str) -> Result<&[f32], String> {
        self.snapshot()?
            .tensors()
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("unknown tensor {name}"))
    }

    pub fn execute(&mut self, program: &Program) -> Result<Value, String> {
        match program {
            Program::Inspect => {
                let s = self.snapshot()?;
                let tensors: BTreeMap<&str, usize> = s
                    .tensors()
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.len()))
                    .collect();
                Ok(json!({
                    "snapshot_id": s.id(),
                    "tensor_count": s.tensors().len(),
                    "tensors": tensors,
                    "format": "ospop.snapshot.v1",
                }))
            }
            Program::Quantize {
                tensor,
                group_size,
                threshold,
            } => {
                let weights = self.tensor(tensor)?;
                let threshold = match threshold {
                    Threshold::Auto => None,
                    Threshold::Fixed(t) => Some(*t),
                };
                let groups = quantize::quantize(weights, *group_size, threshold)?;
                let total: usize = groups.iter().map(|g| g.trits().len()).sum();
                let zeros = groups
                    .iter()
                    .flat_map(|g| g.trits())
                    .filter(|t| **t == Trit::Zero)
                    .count();
                let recon = quantize::reconstruct(&groups);
                let mse = recon
                    .iter()
                    .zip(weights)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f32>()
                    / total as f32;
                let scales: Vec<f32> = groups.iter().map(|g| g.scale()).collect();
                Ok(json!({
                    "op": "quantize",
                    "scheme": "ternary",
                    "tensor": tensor,
                    "group_size": group_size,
                    "values": total,
                    "groups": groups.len(),
                    "packed_bytes": ternary::pack(&groups.iter().flat_map(|g| g.trits()).copied().collect::<Vec<_>>()).len(),
                    "float_bytes": total * 4,
                    "sparsity": zeros as f32 / total as f32,
                    "mse": mse,
                    "min_scale": scales.iter().copied().fold(f32::INFINITY, f32::min),
                    "max_scale": scales.iter().copied().fold(0.0f32, f32::max),
                }))
            }
            Program::Lens { tensor, view } => {
                let weights = self.tensor(tensor)?;
                match view {
                    LensView::Ternary => {
                        let groups = quantize::quantize(weights, 128, None)?;
                        let row: Vec<i8> = groups
                            .iter()
                            .flat_map(|g| g.trits())
                            .map(|t| match t {
                                Trit::Neg => -1,
                                Trit::Zero => 0,
                                Trit::Pos => 1,
                            })
                            .collect();
                        Ok(
                            json!({ "op": "lens", "view": "ternary", "tensor": tensor, "trits": row }),
                        )
                    }
                    LensView::SparsityHistogram => {
                        let groups = quantize::quantize(weights, 128, None)?;
                        let mut buckets = [0usize; 10];
                        for g in &groups {
                            let zeros = g.trits().iter().filter(|t| **t == Trit::Zero).count();
                            let frac = zeros as f32 / g.trits().len() as f32;
                            buckets[((frac * 9.0) as usize).min(9)] += 1;
                        }
                        Ok(
                            json!({ "op": "lens", "view": "sparsity-histogram", "tensor": tensor, "buckets": buckets }),
                        )
                    }
                    LensView::Heatmap => {
                        let width = 16usize;
                        let rows: Vec<Vec<f32>> =
                            weights.chunks(width).map(|c| c.to_vec()).collect();
                        Ok(
                            json!({ "op": "lens", "view": "heatmap", "tensor": tensor, "width": width, "rows": rows }),
                        )
                    }
                    LensView::Graph => {
                        let s = self.snapshot()?;
                        let g = s.graph();
                        Ok(
                            json!({ "op": "lens", "view": "graph", "vertices": s.tensors().keys().collect::<Vec<_>>(), "edges": g.edge_count() }),
                        )
                    }
                    LensView::Diff => {
                        let s = self.snapshot()?;
                        let mut deltas = BTreeMap::new();
                        deltas.insert(
                            (*tensor).clone(),
                            self.tensor(tensor)?
                                .iter()
                                .map(|w| w * 0.5 + 0.125)
                                .collect::<Vec<f32>>(),
                        );
                        let other = s.overlay(&deltas, 1.0)?;
                        let patch = s.diff(&other);
                        let applied = s.apply(&patch)?;
                        let reversed = applied.apply(&s.reverse(&patch)?)?;
                        Ok(json!({
                            "op": "lens",
                            "view": "diff",
                            "tensor": tensor,
                            "base_id": s.id(),
                            "target_id": other.id(),
                            "edited_tensors": patch.edits.len(),
                            "round_trip_ok": reversed.id() == s.id(),
                        }))
                    }
                }
            }
            Program::Diff { tensor, scale } => {
                let s = self.snapshot()?;
                let mut deltas = BTreeMap::new();
                deltas.insert(
                    tensor.clone(),
                    self.tensor(tensor)?
                        .iter()
                        .map(|w| w * scale + 0.125)
                        .collect::<Vec<f32>>(),
                );
                let other = s.overlay(&deltas, 1.0)?;
                let patch = s.diff(&other);
                let applied = s.apply(&patch)?;
                let reversed = applied.apply(&s.reverse(&patch)?)?;
                let mut out = applied.to_bytes();
                out.insert(0, b'\n');
                Ok(json!({
                    "op": "diff",
                    "base_id": s.id(),
                    "target_id": other.id(),
                    "edited_tensors": patch.edits.len(),
                    "round_trip_ok": reversed.id() == s.id(),
                    "artifact_bytes": out.len(),
                }))
            }
            Program::Prune { root } => {
                let s = self.snapshot()?;
                let g = s.graph();
                let reached = g.reachable(root)?;
                let kept = s
                    .tensors()
                    .keys()
                    .filter(|k| reached.contains(k))
                    .cloned()
                    .collect::<Vec<_>>();
                let dropped = s.tensors().len() - kept.len();
                // Pruning is a real transform: stage the pruned snapshot, then
                // prove the operation is reversible via its own patch.
                let pruned = Snapshot::new(
                    s.tensors()
                        .iter()
                        .filter(|(k, _)| reached.contains(k))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                )?;
                let patch = s.diff(&pruned);
                let restored = pruned.apply(&s.reverse(&patch)?)?;
                Ok(json!({
                    "op": "prune",
                    "root": root,
                    "kept": kept.len(),
                    "dropped": dropped,
                    "edges": g.edge_count(),
                    "pruned_id": pruned.id(),
                    "restore_ok": restored.id() == s.id(),
                }))
            }
            Program::Learn { lesson } => match lesson {
                None => Ok(json!({
                    "op": "learn",
                    "lessons": ospop_academy::lessons()
                        .iter()
                        .map(|l: &Lesson| {
                            json!({
                                "id": l.id,
                                "title": l.title,
                                "track": l.track.id(),
                                "prerequisites": l.prerequisites,
                                "lab_view": match l.lab_view {
                                    ospop_academy::View::Ternary => "ternary",
                                    ospop_academy::View::Heatmap => "heatmap",
                                    ospop_academy::View::Graph => "graph",
                                    ospop_academy::View::Diff => "diff",
                                    ospop_academy::View::SparsityHistogram => "sparsity-histogram",
                                },
                            })
                        })
                        .collect::<Vec<_>>(),
                })),
                Some(id) => {
                    let l = ospop_academy::lesson(id).ok_or("unknown lesson")?;
                    let obs = ospop_academy::run_lab(id)?;
                    let verdict = ospop_academy::verify(&l.checkpoint, &obs);
                    Ok(json!({
                        "op": "learn",
                        "lesson": l.id,
                        "title": l.title,
                        "track": l.track.id(),
                        "concept": l.concept,
                        "eli5": l.eli5,
                        "lab_target": l.lab_target,
                        "checkpoint_passed": verdict.is_ok(),
                        "checkpoint_error": verdict.err(),
                        "observation": {
                            "sparsity": obs.sparsity,
                            "min_scale": obs.min_scale,
                            "reconstruction_error": obs.reconstruction_error,
                            "patch_round_trip": obs.patch_round_trip,
                            "graph_fully_reachable": obs.graph_fully_reachable,
                        },
                    }))
                }
            },
            Program::Explain { topic } => {
                let aliases = match topic.as_str() {
                    "ternary" | "trits" => Some("wp.ternary"),
                    "quantization" | "ptq" => Some("wp.quantization"),
                    "ultragraph" | "ultra-edges" => Some("dl.attention"),
                    "omnigraph" | "omnigraph-ir" => Some("dl.layers"),
                    "delta" | "diff" => Some("wp.diff"),
                    "prune" | "pruning" | "sparsity" => Some("wp.pruning"),
                    "academy" | "eli5" | "why" => Some("found.tensor"),
                    "agents" | "mcp" => Some("sys.agents"),
                    _ => ospop_academy::lesson(topic).map(|l| l.id),
                };
                let id = aliases.ok_or_else(|| format!("unknown topic {topic}"))?;
                let l = ospop_academy::lesson(id).ok_or("unknown lesson")?;
                Ok(json!({
                    "op": "explain",
                    "topic": topic,
                    "lesson": l.id,
                    "title": l.title,
                    "eli5": l.eli5,
                    "concept": l.concept,
                    "lab": { "target": l.lab_target, "view": l.lab_view_id() },
                }))
            }
        }
    }
}

/// Re-exported so adapters can report edits without depending on core types.
pub type PatchEdit = Edit;

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> Runtime {
        let mut r = Runtime::new();
        r.load_demo().unwrap();
        r
    }

    #[test]
    fn inspect_reports_snapshot_identity() {
        let v = rt().execute(&Program::Inspect).unwrap();
        assert_eq!(v["format"], "ospop.snapshot.v1");
        assert_eq!(v["tensor_count"], 2);
        assert!(v["snapshot_id"].as_str().unwrap().len() == 64);
    }

    #[test]
    fn quantize_reports_real_compression() {
        let v = rt()
            .execute(&Program::Quantize {
                tensor: "layer.weight".into(),
                group_size: 128,
                threshold: Threshold::Auto,
            })
            .unwrap();
        assert!(v["sparsity"].as_f64().unwrap() >= 0.4);
        assert!(v["packed_bytes"].as_u64().unwrap() * 8 < v["float_bytes"].as_u64().unwrap());
        assert!(v["min_scale"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn diff_and_prune_and_learn_and_explain_work() {
        let mut r = rt();
        let d = r
            .execute(&Program::Diff {
                tensor: "layer.weight".into(),
                scale: 0.5,
            })
            .unwrap();
        assert_eq!(d["round_trip_ok"], true);
        assert_eq!(d["edited_tensors"], 1);
        let p = r
            .execute(&Program::Prune {
                root: "layer.weight".into(),
            })
            .unwrap();
        assert_eq!(p["kept"], 1);
        assert_eq!(p["dropped"], 1);
        assert_eq!(p["restore_ok"], true);
        assert!(r
            .execute(&Program::Prune {
                root: "nope".into()
            })
            .is_err());
        let all = r.execute(&Program::Learn { lesson: None }).unwrap();
        assert_eq!(all["lessons"].as_array().unwrap().len(), 17);
        for l in all["lessons"].as_array().unwrap() {
            let id = l["id"].as_str().unwrap();
            let one = r
                .execute(&Program::Learn {
                    lesson: Some(id.into()),
                })
                .unwrap();
            assert_eq!(one["checkpoint_passed"], true, "{id}");
        }
        let e = r
            .execute(&Program::Explain {
                topic: "ternary".into(),
            })
            .unwrap();
        assert_eq!(e["lesson"], "wp.ternary");
        assert!(r
            .execute(&Program::Explain {
                topic: "nope".into()
            })
            .is_err());
    }

    #[test]
    fn no_model_loaded_is_an_error() {
        let mut r = Runtime::new();
        assert!(r.execute(&Program::Inspect).is_err());
        assert!(r
            .execute(&Program::Quantize {
                tensor: "layer.weight".into(),
                group_size: 128,
                threshold: Threshold::Auto,
            })
            .is_err());
    }
}
