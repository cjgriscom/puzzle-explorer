use crate::circle::{Arc, Circle};
use crate::geometry::rotate_v;
use crate::math::TAU;
use crate::polygon::{PolygonOptions, get_poly_centroids};
use glam::DVec3;
use std::collections::HashSet;

// --- Orbit Analysis ---

pub struct OrbitAnalysis {
    pub degenerate_faces: HashSet<usize>,
    pub face_positions: Vec<DVec3>,
    pub orbits: Vec<Vec<usize>>,
    pub generators: Vec<Vec<Vec<Vec<usize>>>>,
}

pub struct OrbitAnalysisInput<'a> {
    pub circles: &'a [Circle],
    pub arcs: &'a [Arc],
    pub n_a: u32,
    pub n_b: u32,
    pub axis_angle_rad: f64,
    pub colat_a: f64,
    pub colat_b: f64,
    pub options: PolygonOptions,
}

pub fn compute_orbit_analysis(input: OrbitAnalysisInput<'_>) -> Result<OrbitAnalysis, String> {
    let OrbitAnalysisInput {
        circles,
        arcs,
        n_a,
        n_b,
        axis_angle_rad,
        colat_a,
        colat_b,
        options,
    } = input;
    let faces = get_poly_centroids(circles, arcs, options)?;
    let n_faces = faces.len();

    let fudged_mode = matches!(options, PolygonOptions::FudgedMode { .. });

    if n_faces == 0 {
        return Ok(OrbitAnalysis {
            degenerate_faces: HashSet::new(),
            face_positions: vec![],
            orbits: vec![],
            generators: vec![],
        });
    }

    let axis_a = DVec3::new(0.0, 0.0, 1.0);
    let axis_b = DVec3::new(axis_angle_rad.sin(), 0.0, axis_angle_rad.cos());

    let base_pos: Vec<DVec3> = faces.iter().map(|f| f.center).collect();

    /* // For ignoring orbits and displaying debug points
    let orbits_all: Vec<Vec<usize>> = (0..n_faces).map(|i| vec![i]).collect();

    if true {
        return Ok(OrbitAnalysis {
            face_positions: base_pos,
            orbits: orbits_all,
            generators: vec![],
        });
    } */

    struct Move {
        name: &'static str,
        axis: DVec3,
        angle: f64,
        colat: f64,
    }
    let moves = [
        Move {
            name: "A",
            axis: axis_a,
            angle: TAU / n_a as f64,
            colat: colat_a,
        },
        Move {
            name: "Ai",
            axis: axis_a,
            angle: -TAU / n_a as f64,
            colat: colat_a,
        },
        Move {
            name: "B",
            axis: axis_b,
            angle: TAU / n_b as f64,
            colat: colat_b,
        },
        Move {
            name: "Bi",
            axis: axis_b,
            angle: -TAU / n_b as f64,
            colat: colat_b,
        },
    ];

    let find_match = |p_rot: DVec3| -> Option<usize> {
        let mut best_d = f64::MAX;
        let mut best_idx = None;
        for (i, bp) in base_pos.iter().enumerate() {
            let d = p_rot.distance(*bp);
            if d < best_d {
                best_d = d;
                best_idx = Some(i);
            }
        }
        if best_d < 0.4 { best_idx } else { None }
    };

    let mut adj: Vec<Vec<usize>> = vec![vec![]; n_faces];
    let mut perm_a: Vec<usize> = (0..n_faces).collect();
    let mut perm_b: Vec<usize> = (0..n_faces).collect();

    for m in &moves {
        let cos_colat = m.colat.cos();
        for i in 0..n_faces {
            let p0 = base_pos[i];
            let dot = p0.normalize().dot(m.axis);
            let p_rot = if dot > cos_colat + 1e-4 {
                rotate_v(p0, m.axis, m.angle)
            } else {
                p0
            };
            if let Some(idx) = find_match(p_rot) {
                if !adj[i].contains(&idx) {
                    adj[i].push(idx);
                }
                if !adj[idx].contains(&i) {
                    adj[idx].push(i);
                }
                if m.name == "A" {
                    perm_a[i] = idx;
                }
                if m.name == "B" {
                    perm_b[i] = idx;
                }
            }
        }
    }

    // BFS connected components
    let mut visited = vec![false; n_faces];
    let mut orbits: Vec<Vec<usize>> = Vec::new();
    for i in 0..n_faces {
        if visited[i] {
            continue;
        }
        let mut queue = vec![i];
        visited[i] = true;
        let mut members = Vec::new();
        while let Some(u) = queue.pop() {
            members.push(u);
            for &v in &adj[u] {
                if !visited[v] {
                    visited[v] = true;
                    queue.push(v);
                }
            }
        }
        members.sort();
        orbits.push(members);
    }

    let perm_to_0_indexed_cycles = |perm: &[usize], subset: &[usize]| -> Vec<Vec<usize>> {
        let mut in_set = std::collections::HashMap::new();
        for (i, &v) in subset.iter().enumerate() {
            in_set.insert(v, i);
        }
        let mut seen = HashSet::new();
        let mut cycles = Vec::new();
        for &start in subset {
            if seen.contains(&start) {
                continue;
            }
            let mut cycle = Vec::new();
            let mut cur = start;
            while !seen.contains(&cur) && in_set.contains_key(&cur) {
                seen.insert(cur);
                cycle.push(in_set[&cur]);
                cur = perm[cur];
            }
            if cycle.len() > 1 {
                cycles.push(cycle);
            }
        }
        cycles
    };

    let mut generators = Vec::new();
    let mut orbits_final = Vec::new();
    let mut degenerate_faces = HashSet::new();

    for members in orbits.iter() {
        if members.len() == 1 {
            generators.push(vec![]);
            orbits_final.push(members.clone());
        } else {
            let gen_a = perm_to_0_indexed_cycles(&perm_a, members);
            let gen_b = perm_to_0_indexed_cycles(&perm_b, members);
            let gen_a_cycle_length_mismatch = gen_a.iter().any(|c| c.len() != n_a as usize);
            let gen_b_cycle_length_mismatch = gen_b.iter().any(|c| c.len() != n_b as usize);
            if gen_a_cycle_length_mismatch && !fudged_mode {
                return Err(format!(
                    "Orbit Cycle Length mismatch: expected cycle length of {} for move A.",
                    n_a
                ));
            }
            if gen_b_cycle_length_mismatch && !fudged_mode {
                return Err(format!(
                    "Orbit Cycle Length mismatch: expected cycle length of {} for move B.",
                    n_b
                ));
            }

            if fudged_mode {
                if gen_a_cycle_length_mismatch || gen_b_cycle_length_mismatch {
                    for &m in members {
                        degenerate_faces.insert(m);
                    }
                    continue;
                }

                /*
                let int_scale_factor = 100f32; // Cvt to int to sort and set tolerance.
                let mut face_perimeters = Vec::new();
                for &m in members {
                    face_perimeters.push((faces[m].perimeter * int_scale_factor) as i32);
                }

                face_perimeters.sort();
                if face_perimeters[0] != face_perimeters[face_perimeters.len() - 1] {
                    for &m in members {
                        degenerate_faces.insert(m);
                    }
                    continue;
                }
                */

                orbits_final.push(members.clone());
                generators.push([gen_a, gen_b].to_vec());
            } else {
                let mut gens_for_orbit = Vec::new();
                if !gen_a.is_empty() {
                    gens_for_orbit.push(gen_a);
                }
                if !gen_b.is_empty() {
                    gens_for_orbit.push(gen_b);
                }
                generators.push(gens_for_orbit);
                orbits_final.push(members.clone());
            }
        }
    }

    Ok(OrbitAnalysis {
        degenerate_faces,
        face_positions: base_pos,
        orbits: orbits_final,
        generators,
    })
}
