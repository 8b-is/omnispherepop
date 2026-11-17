//! Curriculum, lessons, and checkpoint verification for Omnispherepop Academy.

use ospop_core::{graph::Graph, quantize, snapshot::Snapshot, ternary::Trit};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    Foundations,
    DeepLearning,
    Llm,
    WeightProgramming,
    AiSystems,
}

impl Track {
    pub fn id(self) -> &'static str {
        match self {
            Track::Foundations => "foundations",
            Track::DeepLearning => "deep-learning",
            Track::Llm => "llms",
            Track::WeightProgramming => "weight-programming",
            Track::AiSystems => "ai-systems",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Ternary,
    Heatmap,
    Graph,
    Diff,
    SparsityHistogram,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Checkpoint {
    /// Fraction of zero trits at a quantization threshold must be >= bound.
    SparsityAtLeast {
        threshold: f32,
        group_size: usize,
        bound: f32,
    },
    /// Quantized groups must carry a strictly positive scale.
    ScalesPositive { group_size: usize },
    /// Reconstruction error against source weights must stay under bound.
    ReconstructionErrorAtMost { group_size: usize, bound: f32 },
    /// A patch must apply and reverse back to the source identity.
    PatchRoundTrip,
    /// Every tensor name must be reachable from the first tensor.
    GraphFullyReachable,
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: &'static str,
    pub title: &'static str,
    pub track: Track,
    pub prerequisites: &'static [&'static str],
    pub concept: &'static str,
    pub eli5: &'static str,
    /// Slice of the demo snapshot the lab binds to.
    pub lab_target: &'static str,
    pub lab_view: View,
    pub checkpoint: Checkpoint,
}

impl View {
    pub fn id(self) -> &'static str {
        match self {
            View::Ternary => "ternary",
            View::Heatmap => "heatmap",
            View::Graph => "graph",
            View::Diff => "diff",
            View::SparsityHistogram => "sparsity-histogram",
        }
    }
}

impl Lesson {
    pub fn lab_view_id(&self) -> &'static str {
        self.lab_view.id()
    }

    /// Stable identifier for the lesson's checkpoint rule, for cross-surface parity.
    pub fn checkpoint_id(&self) -> &'static str {
        match self.checkpoint {
            Checkpoint::SparsityAtLeast { .. } => "sparsity-at-least",
            Checkpoint::ScalesPositive { .. } => "scales-positive",
            Checkpoint::ReconstructionErrorAtMost { .. } => "reconstruction-error-at-most",
            Checkpoint::PatchRoundTrip => "patch-round-trip",
            Checkpoint::GraphFullyReachable => "graph-fully-reachable",
        }
    }
}

/// Lesson content. Every lab binds to a real slice of the demo snapshot.
pub const LESSONS: &[Lesson] = &[
    Lesson {
        id: "found.tensor",
        title: "What is a weight?",
        track: Track::Foundations,
        prerequisites: &[],
        concept: "A weight is one number in a matrix that scales a signal.",
        eli5: "A weight is a volume knob on one wire. Programming weights means turning knobs on purpose instead of hoping.",
        lab_target: "layer.weight",
        lab_view: View::Heatmap,
        checkpoint: Checkpoint::ScalesPositive { group_size: 128 },
    },
    Lesson {
        id: "found.forward-pass",
        title: "Weights times activations, by hand",
        track: Track::Foundations,
        prerequisites: &["found.tensor"],
        concept: "A forward pass is dot products between rows of weights and an activation vector.",
        eli5: "Each output is a shopping receipt: multiply what came in by each price, then add it all up.",
        lab_target: "layer.weight",
        lab_view: View::Ternary,
        checkpoint: Checkpoint::ReconstructionErrorAtMost { group_size: 128, bound: 1.0 },
    },
    Lesson {
        id: "found.training",
        title: "Loss, gradients, and why weights move",
        track: Track::Foundations,
        prerequisites: &["found.forward-pass"],
        concept: "Training nudges weights opposite the gradient of a loss.",
        eli5: "Gradient descent is walking downhill in fog: feel the slope under one foot, take a small step.",
        lab_target: "layer.weight",
        lab_view: View::Diff,
        checkpoint: Checkpoint::PatchRoundTrip,
    },
    Lesson {
        id: "dl.layers",
        title: "Layers as weight blocks",
        track: Track::DeepLearning,
        prerequisites: &["found.forward-pass"],
        concept: "A layer is a named block of weights with a fixed shape.",
        eli5: "A layer is a shelf in a warehouse: one address, many boxes.",
        lab_target: "layer.weight",
        lab_view: View::Heatmap,
        checkpoint: Checkpoint::GraphFullyReachable,
    },
    Lesson {
        id: "dl.attention",
        title: "Attention heads as ultra-edges",
        track: Track::DeepLearning,
        prerequisites: &["dl.layers"],
        concept: "Attention routes information from sets of positions to sets of positions, which is an ultra-edge.",
        eli5: "A neighborhood mail route delivers to a whole street at once, not one house.",
        lab_target: "layer.weight",
        lab_view: View::Graph,
        checkpoint: Checkpoint::GraphFullyReachable,
    },
    Lesson {
        id: "dl.embeddings",
        title: "Geometry of embedding weights",
        track: Track::DeepLearning,
        prerequisites: &["dl.layers"],
        concept: "Distances between embedding rows encode similarity.",
        eli5: "Words are cities on a map; similar meanings sit closer together.",
        lab_target: "embed.weight",
        lab_view: View::Heatmap,
        checkpoint: Checkpoint::ScalesPositive { group_size: 128 },
    },
    Lesson {
        id: "dl.overfitting",
        title: "More weights is not better",
        track: Track::DeepLearning,
        prerequisites: &["dl.layers"],
        concept: "Sparsity trades capacity for generalization; measure both.",
        eli5: "A backpack with fewer tools is easier to carry, as long as you keep the ones you need.",
        lab_target: "layer.weight",
        lab_view: View::SparsityHistogram,
        checkpoint: Checkpoint::SparsityAtLeast { threshold: 0.7, group_size: 128, bound: 0.4 },
    },
    Lesson {
        id: "llm.tokens",
        title: "From bytes to tokens",
        track: Track::Llm,
        prerequisites: &["dl.embeddings"],
        concept: "Token ids index embedding rows; weights are selected by data.",
        eli5: "Tokens are page numbers; the embedding table is the book.",
        lab_target: "embed.weight",
        lab_view: View::Heatmap,
        checkpoint: Checkpoint::ScalesPositive { group_size: 128 },
    },
    Lesson {
        id: "llm.attention-heads",
        title: "Attention heads in a tiny model",
        track: Track::Llm,
        prerequisites: &["dl.attention"],
        concept: "Each head owns a slice of the projection weights.",
        eli5: "A head is one reader in a study group, each skimming a different chapter.",
        lab_target: "layer.weight",
        lab_view: View::Ternary,
        checkpoint: Checkpoint::ReconstructionErrorAtMost { group_size: 128, bound: 1.0 },
    },
    Lesson {
        id: "llm.moe",
        title: "Mixture of experts as routing edges",
        track: Track::Llm,
        prerequisites: &["llm.attention-heads"],
        concept: "Routing selects expert blocks; the selection pattern is a directed graph.",
        eli5: "A hospital receptionist sends you to the right specialist, not all of them.",
        lab_target: "layer.weight",
        lab_view: View::Graph,
        checkpoint: Checkpoint::GraphFullyReachable,
    },
    Lesson {
        id: "llm.context",
        title: "How weights shape context",
        track: Track::Llm,
        prerequisites: &["llm.tokens"],
        concept: "Context changes activations, not weights; the two must not be confused.",
        eli5: "Weights are the recipe, context is the ingredients you happen to have.",
        lab_target: "layer.weight",
        lab_view: View::Heatmap,
        checkpoint: Checkpoint::PatchRoundTrip,
    },
    Lesson {
        id: "wp.ternary",
        title: "Ternary weights: the light-switch model",
        track: Track::WeightProgramming,
        prerequisites: &["found.tensor"],
        concept: "Each weight becomes -1, 0, or +1; two bits per value.",
        eli5: "Dimmer switches become light switches: dim, off, bright. Cheaper and much faster.",
        lab_target: "layer.weight",
        lab_view: View::Ternary,
        checkpoint: Checkpoint::SparsityAtLeast { threshold: 0.7, group_size: 128, bound: 0.4 },
    },
    Lesson {
        id: "wp.quantization",
        title: "Thresholds and scales",
        track: Track::WeightProgramming,
        prerequisites: &["wp.ternary"],
        concept: "A threshold decides the trit; a per-group scale restores magnitude.",
        eli5: "Round the pennies for the pocket, keep the dollars for the bank.",
        lab_target: "layer.weight",
        lab_view: View::SparsityHistogram,
        checkpoint: Checkpoint::ReconstructionErrorAtMost { group_size: 128, bound: 1.0 },
    },
    Lesson {
        id: "wp.pruning",
        title: "Sparsity and ultra-edge cuts",
        track: Track::WeightProgramming,
        prerequisites: &["dl.attention", "wp.ternary"],
        concept: "Pruning removes coordinates and edges while preserving invariants.",
        eli5: "Cutting roads only helps if the towns you care about stay connected.",
        lab_target: "layer.weight",
        lab_view: View::Graph,
        checkpoint: Checkpoint::GraphFullyReachable,
    },
    Lesson {
        id: "wp.diff",
        title: "Versioning weights with receipts",
        track: Track::WeightProgramming,
        prerequisites: &["found.training"],
        concept: "A diff is a reversible edit set bound to a base snapshot identity.",
        eli5: "Keep the receipt, not the whole grocery store; you can always reconstruct the trip.",
        lab_target: "layer.weight",
        lab_view: View::Diff,
        checkpoint: Checkpoint::PatchRoundTrip,
    },
    Lesson {
        id: "sys.inference",
        title: "What happens at query time",
        track: Track::AiSystems,
        prerequisites: &["llm.tokens"],
        concept: "Inference reads weights and writes activations; ternary kernels trade accuracy for speed.",
        eli5: "Cooking dinner uses the recipe and the pantry; you do not rewrite the recipe each time.",
        lab_target: "layer.weight",
        lab_view: View::Ternary,
        checkpoint: Checkpoint::ReconstructionErrorAtMost { group_size: 128, bound: 1.0 },
    },
    Lesson {
        id: "sys.agents",
        title: "Let an agent program your weights",
        track: Track::AiSystems,
        prerequisites: &["wp.diff"],
        concept: "Agents drive the same operations through MCP tools, so results are identical.",
        eli5: "A robot arm using your own tools is still your workshop.",
        lab_target: "layer.weight",
        lab_view: View::Diff,
        checkpoint: Checkpoint::PatchRoundTrip,
    },
];

pub fn lesson(id: &str) -> Option<&'static Lesson> {
    LESSONS.iter().find(|l| l.id == id)
}

pub fn lessons() -> &'static [Lesson] {
    LESSONS
}

/// Curriculum as an ultragraph: prerequisite sets point at a lesson.
pub fn curriculum_graph() -> Result<Graph, String> {
    let mut vertices: Vec<String> = LESSONS.iter().map(|l| l.id.to_string()).collect();
    vertices.sort();
    let mut g = Graph::new(vertices);
    for (i, l) in LESSONS.iter().enumerate() {
        if l.prerequisites.is_empty() {
            continue;
        }
        g.add_edge(
            format!("prereq-{}", i),
            l.prerequisites.iter().map(|s| s.to_string()).collect(),
            vec![l.id.to_string()],
            false,
        )?;
    }
    for id in LESSONS.iter().flat_map(|l| l.prerequisites) {
        if lesson(id).is_none() {
            return Err(format!("unknown prerequisite {id}"));
        }
    }
    Ok(g)
}

/// Observed results from a lab run. Checkpoints read these, never the model.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub sparsity: f32,
    pub min_scale: f32,
    pub reconstruction_error: f32,
    pub patch_round_trip: bool,
    pub graph_fully_reachable: bool,
}

pub fn verify(checkpoint: &Checkpoint, obs: &Observation) -> Result<(), String> {
    match *checkpoint {
        Checkpoint::SparsityAtLeast { bound, .. } => {
            if obs.sparsity + 1e-6 < bound {
                return Err(format!(
                    "sparsity {:.3} below required {:.3}",
                    obs.sparsity, bound
                ));
            }
        }
        Checkpoint::ScalesPositive { .. } => {
            if obs.min_scale <= 0.0 {
                return Err("expected at least one strictly positive group scale".into());
            }
        }
        Checkpoint::ReconstructionErrorAtMost { bound, .. } => {
            if obs.reconstruction_error > bound {
                return Err(format!(
                    "reconstruction error {:.4} above bound {bound}",
                    obs.reconstruction_error
                ));
            }
        }
        Checkpoint::PatchRoundTrip => {
            if !obs.patch_round_trip {
                return Err("patch did not reverse to the base identity".into());
            }
        }
        Checkpoint::GraphFullyReachable => {
            if !obs.graph_fully_reachable {
                return Err("curriculum graph is not fully reachable".into());
            }
        }
    }
    Ok(())
}

/// Toy demo snapshot used by the labs: real numbers, tiny shapes.
pub fn demo_snapshot() -> Result<Snapshot, String> {
    let mut layer = Vec::new();
    let mut state = 0x9e37_79b9u32;
    for i in 0..512 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let unit = (state >> 8) as f32 / (1u32 << 24) as f32;
        // Peaky, zero-centred: real trained tensors cluster near zero, which is
        // what makes ternary quantization interesting.
        let centred = unit * 4.0 - 2.0;
        layer.push(centred * centred.abs() * (1.0 - (i % 7) as f32 * 0.1));
    }
    let embed = (0..256).map(|i| ((i % 16) as f32 - 7.5) * 0.25).collect();
    Snapshot::new(BTreeMap::from([
        ("layer.weight".to_string(), layer),
        ("embed.weight".to_string(), embed),
    ]))
}

fn weights_of<'a>(snap: &'a Snapshot, name: &str) -> Result<&'a [f32], String> {
    snap.tensors()
        .get(name)
        .map(Vec::as_slice)
        .ok_or_else(|| "lab target missing".to_string())
}

/// A deterministic perturbation, so the diff lab has real edits to reverse.
fn scaled(weights: &[f32]) -> Vec<f32> {
    weights.iter().map(|w| w * 0.5 + 0.125).collect()
}

/// Run a lesson's lab against the demo snapshot and verify its checkpoint.
pub fn run_lab(id: &str) -> Result<Observation, String> {
    let l = lesson(id).ok_or("unknown lesson")?;
    let snap = demo_snapshot()?;
    let weights = weights_of(&snap, l.lab_target)?;
    let (group_size, threshold) = match l.checkpoint {
        Checkpoint::SparsityAtLeast {
            threshold,
            group_size,
            ..
        } => (group_size, Some(threshold)),
        _ => (128, None),
    };
    let groups = quantize::quantize(weights, group_size, threshold)?;
    let total: usize = groups.iter().map(|g| g.trits().len()).sum();
    let zeros = groups
        .iter()
        .flat_map(|g| g.trits())
        .filter(|t| **t == Trit::Zero)
        .count();
    let min_scale = groups
        .iter()
        .map(|g| g.scale())
        .fold(f32::INFINITY, f32::min);
    let recon = quantize::reconstruct(&groups);
    let error = (recon
        .iter()
        .zip(weights)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        / total as f32)
        .sqrt();
    let snap = demo_snapshot()?;
    let other = snap
        .overlay(
            &BTreeMap::from([(l.lab_target.to_string(), scaled(weights))]),
            0.25,
        )
        .unwrap_or_else(|_| snap.clone());
    let patch = snap.diff(&other);
    let round_trip = snap
        .apply(&patch)
        .and_then(|applied| {
            let rev = snap.reverse(&patch)?;
            applied.apply(&rev)
        })
        .map(|back| back.id() == snap.id())
        .unwrap_or(false);
    let graph = curriculum_graph()?;
    let first = lessons()[0].id;
    let reached = graph.reachable(first)?;
    Ok(Observation {
        sparsity: zeros as f32 / total as f32,
        min_scale: if min_scale.is_infinite() {
            0.0
        } else {
            min_scale
        },
        reconstruction_error: error,
        patch_round_trip: round_trip,
        graph_fully_reachable: reached.len() == lessons().len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curriculum_covers_all_lessons() {
        let g = curriculum_graph().unwrap();
        assert_eq!(g.reachable(lessons()[0].id).unwrap().len(), lessons().len());
        assert_eq!(lessons().len(), 17);
    }

    #[test]
    fn every_lesson_passes_its_own_checkpoint() {
        for l in lessons() {
            let obs = run_lab(l.id).unwrap_or_else(|e| panic!("{}: {e}", l.id));
            verify(&l.checkpoint, &obs).unwrap_or_else(|e| panic!("{}: {e}", l.id));
        }
    }

    #[test]
    fn checkpoint_rejects_missing_evidence() {
        let obs = Observation {
            sparsity: 0.0,
            min_scale: 0.0,
            reconstruction_error: 9.0,
            patch_round_trip: false,
            graph_fully_reachable: false,
        };
        for c in [
            Checkpoint::SparsityAtLeast {
                threshold: 0.0,
                group_size: 8,
                bound: 0.5,
            },
            Checkpoint::ScalesPositive { group_size: 8 },
            Checkpoint::ReconstructionErrorAtMost {
                group_size: 8,
                bound: 0.1,
            },
            Checkpoint::PatchRoundTrip,
            Checkpoint::GraphFullyReachable,
        ] {
            assert!(verify(&c, &obs).is_err());
        }
    }

    #[test]
    fn unknown_lesson_and_target_rejected() {
        assert!(run_lab("nope").is_err());
        assert!(lesson("nope").is_none());
    }
}
