//! Canonization of puzzles
//!
//! This module interfaces with dreadnaut to canonize orbit graphs
//! as well as complete puzzles.

/// Construct a Dreadnaut script to get the canon hash for an orbit generator
pub fn orbit_graph_hash_script(
    start_idx: usize,
    combined_gen: &[Vec<Vec<usize>>],
    n_vertices: usize,
) -> Result<String, String> {
    // Ported from DreadnautInterface.java in GroupExplorer

    let mut adj: Vec<std::collections::BTreeSet<usize>> =
        vec![std::collections::BTreeSet::new(); n_vertices];
    for generator in combined_gen {
        for cycle in generator {
            if cycle.len() < 2 {
                continue;
            }
            for i in 0..cycle.len() {
                let (x, y) = match (cycle.get(i), cycle.get((i + 1) % cycle.len())) {
                    (Some(x), Some(y)) => (x, y),
                    (_, _) => return Err("Vertex index out of bounds".to_string()),
                };
                let (u, v) = match (x.checked_sub(start_idx), y.checked_sub(start_idx)) {
                    (Some(u), Some(v)) => (u, v),
                    (_, _) => return Err("Vertex index out of bounds".to_string()),
                };
                adj[u].insert(v);
            }
        }
    }

    let mut script = String::new();
    script.push_str("l=0\n-m\n");
    script.push_str("Ad\nd\n");
    script.push_str(&format!("n={} g\n", n_vertices));
    (0..n_vertices).for_each(|i| {
        script.push_str(&format!("{}:", i));
        let neigh = &adj[i];
        if !neigh.is_empty() {
            for &j in neigh {
                script.push_str(&format!(" {}", j));
            }
        }
        if i == n_vertices - 1 {
            script.push_str(".\n");
        } else {
            script.push_str(";\n");
        }
    });
    script.push_str("c -a\nx\nz\n");
    Ok(script)
}
