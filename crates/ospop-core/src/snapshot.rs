use crate::graph::Graph;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Immutable, validated demo snapshot. Production chunked manifests are a
/// separate format; this identity is domain-separated and versioned.
#[derive(Debug, Clone)]
pub struct Snapshot {
    id: String,
    tensors: BTreeMap<String, Vec<f32>>,
}

/// Structural difference between two snapshots.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Set { values: Vec<f32> },
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub base_id: String,
    pub target_id: String,
    pub edits: BTreeMap<String, Edit>,
}

fn canonical(tensors: &BTreeMap<String, Vec<f32>>) -> String {
    let mut out = String::from("ospop.snapshot.v1\n");
    for (name, values) in tensors {
        out.push_str(name);
        out.push('\n');
        for v in values {
            let v = if *v == 0.0 { 0.0 } else { *v };
            out.push_str(&v.to_bits().to_string());
            out.push(' ');
        }
        out.push('\n');
    }
    out
}

fn digest(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b"ospop.snapshot.v1\0");
    h.update(bytes);
    format!("{:x}", h.finalize())
}

impl Snapshot {
    pub fn new(tensors: BTreeMap<String, Vec<f32>>) -> Result<Self, String> {
        for (name, values) in &tensors {
            if name.is_empty() || name.contains('\n') {
                return Err("invalid tensor name".into());
            }
            if values.iter().any(|v| !v.is_finite()) {
                return Err("nonfinite tensor value".into());
            }
        }
        let id = digest(canonical(&tensors).as_bytes());
        Ok(Self { id, tensors })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn tensors(&self) -> &BTreeMap<String, Vec<f32>> {
        &self.tensors
    }

    pub fn graph(&self) -> Graph {
        Graph::new(self.tensors.keys().cloned())
    }

    pub fn diff(&self, other: &Self) -> Patch {
        let mut edits = BTreeMap::new();
        for (name, values) in &self.tensors {
            match other.tensors.get(name) {
                Some(new) if new == values => {}
                Some(new) => {
                    edits.insert(
                        name.clone(),
                        Edit::Set {
                            values: new.clone(),
                        },
                    );
                }
                None => {
                    edits.insert(name.clone(), Edit::Remove);
                }
            }
        }
        for (name, values) in &other.tensors {
            if !self.tensors.contains_key(name) {
                edits.insert(
                    name.clone(),
                    Edit::Set {
                        values: values.clone(),
                    },
                );
            }
        }
        Patch {
            base_id: self.id.clone(),
            target_id: other.id.clone(),
            edits,
        }
    }

    pub fn apply(&self, patch: &Patch) -> Result<Self, String> {
        if patch.base_id != self.id {
            return Err("patch base mismatch".into());
        }
        let mut next = self.tensors.clone();
        for (name, edit) in &patch.edits {
            match edit {
                Edit::Remove => {
                    if next.remove(name).is_none() {
                        return Err("patch removes missing tensor".into());
                    }
                }
                Edit::Set { values } => {
                    if name.is_empty() || name.contains('\n') {
                        return Err("invalid tensor name".into());
                    }
                    if values.iter().any(|v| !v.is_finite()) {
                        return Err("nonfinite tensor value".into());
                    }
                    if values.len() != self.tensors.get(name).map_or(values.len(), Vec::len) {
                        return Err("patch changes tensor shape".into());
                    }
                    next.insert(name.clone(), values.clone());
                }
            }
        }
        Self::new(next)
    }

    pub fn reverse(&self, patch: &Patch) -> Result<Patch, String> {
        if patch.base_id != self.id {
            return Err("patch base mismatch".into());
        }
        let mut edits = BTreeMap::new();
        for (name, edit) in &patch.edits {
            edits.insert(
                name.clone(),
                match edit {
                    Edit::Set { .. } => match self.tensors.get(name) {
                        Some(values) => Edit::Set {
                            values: values.clone(),
                        },
                        None => Edit::Remove,
                    },
                    Edit::Remove => match self.tensors.get(name) {
                        Some(values) => Edit::Set {
                            values: values.clone(),
                        },
                        None => return Err("reverse removes missing tensor".into()),
                    },
                },
            );
        }
        Ok(Patch {
            base_id: patch.target_id.clone(),
            target_id: patch.base_id.clone(),
            edits,
        })
    }

    /// Blended overlay; result is f32, so it is not representable as packed trits.
    pub fn overlay(&self, deltas: &BTreeMap<String, Vec<f32>>, beta: f32) -> Result<Self, String> {
        if !beta.is_finite() {
            return Err("nonfinite blend coefficient".into());
        }
        let mut next = self.tensors.clone();
        for (name, delta) in deltas {
            let target = next.get_mut(name).ok_or("overlay target missing")?;
            if target.len() != delta.len() {
                return Err("overlay shape mismatch".into());
            }
            for (w, d) in target.iter_mut().zip(delta) {
                let blended = f64::from(*w) + f64::from(beta) * f64::from(*d);
                if !blended.is_finite() {
                    return Err("overlay overflow".into());
                }
                *w = blended as f32;
            }
        }
        Self::new(next)
    }

    /// Canonical serialization of the demo format, for CLI artifacts.
    pub fn to_bytes(&self) -> Vec<u8> {
        canonical(&self.tensors).into_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "invalid utf8")?;
        let mut lines = text.lines();
        if lines.next() != Some("ospop.snapshot.v1") {
            return Err("unsupported snapshot format".into());
        }
        let mut tensors = BTreeMap::new();
        while let Some(name) = lines.next() {
            let row = lines.next().ok_or("missing tensor row")?;
            let values = row
                .split_whitespace()
                .map(|t| {
                    let bits: u32 = t.parse().map_err(|_| "invalid tensor value")?;
                    Ok(f32::from_bits(bits))
                })
                .collect::<Result<Vec<_>, String>>()?;
            tensors.insert(name.to_string(), values);
        }
        Self::new(tensors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(pairs: &[(&str, &[f32])]) -> Snapshot {
        Snapshot::new(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_vec()))
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn identity_is_order_independent() {
        let a = snap(&[("b", &[1.0]), ("a", &[2.0])]);
        let b = snap(&[("a", &[2.0]), ("b", &[1.0])]);
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn patch_round_trips_and_reverse_restores() {
        let a = snap(&[("a", &[1.0, 2.0]), ("gone", &[3.0])]);
        let b = snap(&[("a", &[1.0, 5.0])]);
        let p = a.diff(&b);
        let applied = a.apply(&p).unwrap();
        assert_eq!(applied.id(), b.id());
        let rev = a.reverse(&p).unwrap();
        assert_eq!(applied.apply(&rev).unwrap().id(), a.id());
        assert_eq!(a.id(), snap(&[("a", &[1.0, 2.0]), ("gone", &[3.0])]).id());
    }

    #[test]
    fn bad_base_shape_and_overlay_rejected() {
        let a = snap(&[("a", &[1.0, 2.0])]);
        let b = snap(&[("a", &[3.0, 4.0])]);
        let p = a.diff(&b);
        assert!(b.apply(&p).is_err());
        assert!(a
            .apply(&Patch {
                base_id: "nope".into(),
                ..p
            })
            .is_err());
        let shaped = snap(&[("a", &[1.0, 2.0])]);
        assert!(shaped
            .overlay(
                &{
                    let mut m = BTreeMap::new();
                    m.insert("a".to_string(), vec![1.0]);
                    m
                },
                0.5
            )
            .is_err());
        assert!(shaped
            .overlay(
                &{
                    let mut m = BTreeMap::new();
                    m.insert("missing".to_string(), vec![1.0, 2.0]);
                    m
                },
                0.5
            )
            .is_err());
        assert!(snap(&[("a", &[1.0])])
            .overlay(
                &{
                    let mut m = BTreeMap::new();
                    m.insert("a".to_string(), vec![1.0]);
                    m
                },
                f32::NAN
            )
            .is_err());
    }

    #[test]
    fn invalid_snapshots_rejected() {
        assert!(Snapshot::new(BTreeMap::from([("a".into(), vec![f32::NAN])])).is_err());
        assert!(Snapshot::new(BTreeMap::from([("".into(), vec![1.0])])).is_err());
    }

    #[test]
    fn equal_snapshots_have_empty_patch() {
        let a = snap(&[("a", &[1.0])]);
        assert!(a.diff(&a).edits.is_empty());
    }

    #[test]
    fn serialization_round_trips() {
        let a = snap(&[("a", &[-0.0, 1.5]), ("b", &[])]);
        let bytes = a.to_bytes();
        assert_eq!(Snapshot::from_bytes(&bytes).unwrap().id(), a.id());
        assert!(Snapshot::from_bytes(b"other\n").is_err());
    }
}
