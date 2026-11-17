use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone)]
struct Edge {
    sources: BTreeSet<String>,
    targets: BTreeSet<String>,
}
#[derive(Debug, Clone)]
pub struct Graph {
    vertices: BTreeSet<String>,
    edges: BTreeMap<String, Edge>,
}
impl Graph {
    pub fn new(vertices: impl IntoIterator<Item = String>) -> Self {
        Self {
            vertices: vertices.into_iter().collect(),
            edges: BTreeMap::new(),
        }
    }
    pub fn add_edge(
        &mut self,
        id: String,
        sources: Vec<String>,
        targets: Vec<String>,
        recurrent: bool,
    ) -> Result<(), String> {
        if id.is_empty() || self.edges.contains_key(&id) {
            return Err("empty or duplicate edge ID".into());
        }
        let sources: BTreeSet<_> = sources.into_iter().collect();
        let targets: BTreeSet<_> = targets.into_iter().collect();
        if sources.is_empty() || targets.is_empty() {
            return Err("empty endpoint set".into());
        }
        if !sources.is_subset(&self.vertices) || !targets.is_subset(&self.vertices) {
            return Err("dangling endpoint".into());
        }
        if !recurrent && !sources.is_disjoint(&targets) {
            return Err("overlap requires recurrent edge".into());
        }
        self.edges.insert(id, Edge { sources, targets });
        Ok(())
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    /// Least fixed point: an edge fires only when all its sources are reached.
    pub fn reachable(&self, start: &str) -> Result<Vec<String>, String> {
        if !self.vertices.contains(start) {
            return Err("unknown vertex".into());
        }
        let mut reached = BTreeSet::from([start.to_owned()]);
        loop {
            let before = reached.len();
            for edge in self.edges.values() {
                if edge.sources.is_subset(&reached) {
                    reached.extend(edge.targets.iter().cloned());
                }
            }
            if before == reached.len() {
                break;
            }
        }
        Ok(reached.into_iter().collect())
    }
}
