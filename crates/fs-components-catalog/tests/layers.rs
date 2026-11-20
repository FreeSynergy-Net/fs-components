//! The layer rule of the crates of this repository: `contract/layers.toml` gives every crate a layer,
//! and a crate may depend (build edges) only on crates of a strictly lower layer.
//!
//! Pattern: none — a test file; `violations` is a pure function so the rule itself can be tested.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use toml::Value;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.parse().unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The names of the path dependencies of one dependency table.
fn path_deps(table: &Value, out: &mut BTreeSet<String>) {
    let Some(table) = table.as_table() else { return };
    for (name, spec) in table {
        if spec.get("path").is_some() {
            out.insert(name.clone());
        }
    }
}

/// The build edges of a manifest: [dependencies], [build-dependencies] and the target-specific ones
/// (dev-dependencies are not edges of the build).
fn edges(manifest: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in ["dependencies", "build-dependencies"] {
        if let Some(t) = manifest.get(key) {
            path_deps(t, &mut out);
        }
    }
    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for t in targets.values() {
            for key in ["dependencies", "build-dependencies"] {
                if let Some(t) = t.get(key) {
                    path_deps(t, &mut out);
                }
            }
        }
    }
    out
}

/// Every edge that goes up or sideways and is not a named exception, as `"a -> b"`.
fn violations(
    layers: &BTreeMap<String, i64>,
    graph: &BTreeMap<String, BTreeSet<String>>,
    exceptions: &BTreeSet<String>,
) -> Vec<String> {
    let mut bad = Vec::new();
    for (from, deps) in graph {
        for to in deps {
            let edge = format!("{from} -> {to}");
            match (layers.get(from), layers.get(to)) {
                (Some(a), Some(b)) if b < a => {}
                _ if exceptions.contains(&edge) => {}
                (Some(_), Some(_)) => bad.push(edge),
                _ => bad.push(format!("{edge} (a crate has no layer)")),
            }
        }
    }
    bad
}

struct World {
    layers: BTreeMap<String, i64>,
    graph: BTreeMap<String, BTreeSet<String>>,
    exceptions: BTreeSet<String>,
    dirs: BTreeSet<String>,
}

fn world() -> World {
    let root = repo();
    let table = read(&root.join("contract/layers.toml"));
    let layers = table["layers"]
        .as_table()
        .expect("[layers]")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_integer().expect("layer number")))
        .collect();
    let exceptions = table["exceptions"]
        .as_table()
        .expect("[exceptions]")
        .iter()
        .map(|(k, v)| {
            assert!(v.as_str().is_some_and(|s| !s.is_empty()), "exception {k} needs a reason");
            k.clone()
        })
        .collect();
    let mut graph = BTreeMap::new();
    let mut dirs = BTreeSet::new();
    for entry in std::fs::read_dir(root.join("crates")).expect("crates/") {
        let dir = entry.expect("entry").path();
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        graph.insert(name.clone(), edges(&read(&manifest)));
        dirs.insert(name);
    }
    World { layers, graph, exceptions, dirs }
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

/// The real tree: every crate is listed, nothing listed is missing, no edge goes up or sideways.
#[test]
fn the_tree_follows_the_layer_table() {
    let w = world();
    let listed: BTreeSet<String> = w.layers.keys().cloned().collect();
    assert_eq!(listed, w.dirs, "contract/layers.toml and crates/ list different crates");
    let bad = violations(&w.layers, &w.graph, &w.exceptions);
    assert!(bad.is_empty(), "edges against the layer rule: {bad:#?}");
}

/// A named exception must still be a real edge (a stale exception hides nothing but must go).
#[test]
fn every_exception_is_a_real_edge() {
    let w = world();
    for e in &w.exceptions {
        let (from, to) = e.split_once(" -> ").expect("\"a -> b\"");
        assert!(w.graph.get(from).is_some_and(|d| d.contains(to)), "stale exception {e}");
    }
}

/// The rule itself: an edge up, one sideways, one to an unlisted crate are red; down and an exception are not.
#[test]
fn the_rule_is_red_on_up_sideways_and_unknown() {
    let layers: BTreeMap<String, i64> =
        [("a", 0), ("b", 10), ("c", 10)].iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
    let graph: BTreeMap<String, BTreeSet<String>> = [
        ("a".to_string(), set(&["b"])),
        ("b".to_string(), set(&["a", "c", "zz"])),
    ]
    .into_iter()
    .collect();
    let none = BTreeSet::new();
    let bad = violations(&layers, &graph, &none);
    assert_eq!(bad.len(), 3, "{bad:?}");
    assert!(bad.contains(&"a -> b".to_string()), "{bad:?}");
    assert!(bad.contains(&"b -> c".to_string()), "{bad:?}");
    let excused = violations(&layers, &graph, &set(&["a -> b", "b -> c"]));
    assert_eq!(excused.len(), 1, "{excused:?}");
}

/// Dev-dependencies are no build edges; target-specific and build dependencies are.
#[test]
fn edges_read_the_build_tables_only() {
    let m: Value = r#"
        [dependencies]
        a = { path = "../a" }
        serde = "1"
        [dev-dependencies]
        d = { path = "../d" }
        [build-dependencies]
        b = { path = "../b" }
        [target.'cfg(unix)'.dependencies]
        t = { path = "../t", optional = true }
    "#
    .parse()
    .unwrap();
    assert_eq!(edges(&m), set(&["a", "b", "t"]));
}
