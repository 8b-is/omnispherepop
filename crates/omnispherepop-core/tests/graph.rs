use omnispherepop_core::graph::Graph;
fn names(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}
#[test]
fn all_sources_required_and_cycles_terminate() {
    let mut g = Graph::new(names(&["a", "b", "c", "d"]));
    g.add_edge(
        "joint".into(),
        names(&["b", "a", "a"]),
        names(&["c", "c"]),
        false,
    )
    .unwrap();
    assert_eq!(g.reachable("a").unwrap(), names(&["a"]));
    g.add_edge("ab".into(), names(&["a"]), names(&["b"]), false)
        .unwrap();
    g.add_edge("ca".into(), names(&["c"]), names(&["a"]), false)
        .unwrap();
    assert_eq!(g.reachable("a").unwrap(), names(&["a", "b", "c"]));
    assert_eq!(g.edge_count(), 3);
}
#[test]
fn bad_edges_do_not_mutate_graph() {
    let mut g = Graph::new(names(&["a", "b"]));
    for (id, src, dst) in [
        ("x", vec![], names(&["b"])),
        ("x", names(&["a"]), vec![]),
        ("x", names(&["a"]), names(&["missing"])),
        ("x", names(&["a"]), names(&["a"])),
        ("", names(&["a"]), names(&["b"])),
    ] {
        assert!(g.add_edge(id.into(), src, dst, false).is_err());
        assert_eq!(g.edge_count(), 0);
    }
    g.add_edge("self".into(), names(&["a"]), names(&["a"]), true)
        .unwrap();
    assert!(g
        .add_edge("self".into(), names(&["a"]), names(&["b"]), false)
        .is_err());
    assert_eq!(g.edge_count(), 1);
    assert!(g.reachable("unknown").is_err());
}
