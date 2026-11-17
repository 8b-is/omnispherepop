use crate::ternary::Trit;
#[derive(Debug, Clone)]
pub struct Group {
    trits: Vec<Trit>,
    scale: f32,
}
impl Group {
    pub fn trits(&self) -> &[Trit] {
        &self.trits
    }
    pub fn scale(&self) -> f32 {
        self.scale
    }
}
/// Reference PTQ with strict thresholding and per-group mean nonzero scale.
pub fn quantize(
    weights: &[f32],
    group_size: usize,
    threshold: Option<f32>,
) -> Result<Vec<Group>, String> {
    if group_size == 0 {
        return Err("group size must be positive".into());
    }
    if weights.iter().any(|w| !w.is_finite()) {
        return Err("nonfinite weight".into());
    }
    if threshold.is_some_and(|t| !t.is_finite() || t < 0.) {
        return Err("invalid threshold".into());
    }
    Ok(weights
        .chunks(group_size)
        .map(|chunk| {
            let delta = threshold.map(f64::from).unwrap_or_else(|| {
                0.7 * chunk.iter().map(|w| f64::from(*w).abs()).sum::<f64>() / chunk.len() as f64
            });
            let trits: Vec<_> = chunk
                .iter()
                .map(|w| {
                    if f64::from(*w) < -delta {
                        Trit::Neg
                    } else if f64::from(*w) > delta {
                        Trit::Pos
                    } else {
                        Trit::Zero
                    }
                })
                .collect();
            let selected: Vec<_> = chunk
                .iter()
                .zip(&trits)
                .filter(|(_, t)| **t != Trit::Zero)
                .collect();
            let scale = if selected.is_empty() {
                0.
            } else {
                (selected
                    .iter()
                    .map(|(w, _)| f64::from(**w).abs())
                    .sum::<f64>()
                    / selected.len() as f64) as f32
            };
            Group { trits, scale }
        })
        .collect())
}

pub fn reconstruct(groups: &[Group]) -> Vec<f32> {
    groups
        .iter()
        .flat_map(|g| {
            g.trits.iter().map(move |t| match t {
                Trit::Neg => -g.scale,
                Trit::Zero => 0.,
                Trit::Pos => g.scale,
            })
        })
        .collect()
}

/// Fixed-order f64 reference accumulation, checked f32 output. Group scaling
/// uses multiplication; only ternary inner products use add/subtract/skip.
pub fn dot(groups: &[Group], activations: &[f32]) -> Result<f32, String> {
    let len = groups
        .iter()
        .try_fold(0usize, |n, g| n.checked_add(g.trits.len()))
        .ok_or("activation length overflow")?;
    if len != activations.len() {
        return Err("activation length mismatch".into());
    }
    if activations.iter().any(|a| !a.is_finite()) {
        return Err("nonfinite activation".into());
    }
    let mut offset = 0;
    let mut sum = 0f64;
    for group in groups {
        let mut inner = 0f64;
        for (t, a) in group.trits.iter().zip(&activations[offset..]) {
            match t {
                Trit::Neg => inner -= f64::from(*a),
                Trit::Pos => inner += f64::from(*a),
                Trit::Zero => {}
            }
        }
        sum += f64::from(group.scale) * inner;
        offset += group.trits.len();
    }
    let output = sum as f32;
    if !output.is_finite() {
        return Err("dot product overflow".into());
    }
    Ok(output)
}
