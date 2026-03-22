use crate::circle::{Arc, Circle};
use crate::math::{PI, TAU, norm_ang};
use glam::DVec3;
use std::collections::{BTreeMap, HashSet};

const AUTO_MAX_ITERS: usize = 35;
pub struct Interval {
    pub s: f64,
    pub l: f64,
}

pub fn rotate_v(v: DVec3, axis: DVec3, angle: f64) -> DVec3 {
    let c = angle.cos();
    let s = angle.sin();
    let d = v.dot(axis);
    v * c + axis.cross(v) * s + axis * d * (1.0 - c)
}

pub fn cap_range(c: &Circle, axis: DVec3, cap_colat: f64) -> Option<Interval> {
    let sc = c.colat.sin();
    let cc = c.colat.cos();
    let a_val = sc * c.u.dot(axis);
    let b_val = sc * c.w.dot(axis);
    let c_val = cc * c.pole.dot(axis);
    let amp = (a_val * a_val + b_val * b_val).sqrt();
    let cos_cap = cap_colat.cos();
    let eps = 1e-9;

    if c_val + amp < cos_cap - eps {
        return None;
    }
    if c_val - amp >= cos_cap - eps {
        return Some(Interval { s: 0.0, l: TAU });
    }

    let phi = b_val.atan2(a_val);
    let ratio = ((cos_cap - c_val) / amp).clamp(-1.0, 1.0);
    let delta = ratio.acos();
    Some(Interval {
        s: norm_ang(phi - delta),
        l: 2.0 * delta,
    })
}

pub fn isect_iv(a: &Interval, b: &Interval) -> Vec<Interval> {
    fn segs(iv: &Interval) -> Vec<(f64, f64)> {
        let e = iv.s + iv.l;
        if e <= TAU + 1e-10 {
            vec![(iv.s, e.min(TAU))]
        } else {
            vec![(iv.s, TAU), (0.0, e - TAU)]
        }
    }
    let sa = segs(a);
    let sb = segs(b);
    let mut res = Vec::new();
    for (as_start, as_end) in &sa {
        for (bs, be) in &sb {
            let lo = as_start.max(*bs);
            let hi = as_end.min(*be);
            if hi > lo + 1e-10 {
                res.push(Interval { s: lo, l: hi - lo });
            }
        }
    }
    res
}

pub fn subtract_iv(base: &Interval, removals: &[Interval]) -> Vec<Interval> {
    if base.l < 1e-10 {
        return vec![];
    }
    let bl = base.l;
    let mut segs: Vec<(f64, f64)> = Vec::new();

    for r in removals {
        let rs = norm_ang(r.s - base.s);
        let rl = r.l;
        if rs < bl {
            segs.push((rs, (rs + rl).min(bl)));
        }
        if rs + rl > TAU {
            let we = rs + rl - TAU;
            if we > 0.0 {
                segs.push((0.0, we.min(bl)));
            }
        }
        if rs > bl && rs + rl > TAU {
            let we2 = rs + rl - TAU;
            if we2 > 0.0 {
                segs.push((0.0, we2.min(bl)));
            }
        }
    }

    segs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for seg in segs {
        if let Some(last) = merged.last_mut() {
            if seg.0 <= last.1 + 1e-10 {
                last.1 = last.1.max(seg.1);
            } else {
                merged.push(seg);
            }
        } else {
            merged.push(seg);
        }
    }

    let mut res = Vec::new();
    let mut pos = 0.0;
    for (start, end) in merged {
        if start > pos + 1e-10 {
            res.push(Interval {
                s: norm_ang(pos + base.s),
                l: start - pos,
            });
        }
        pos = end;
    }
    if bl > pos + 1e-10 {
        res.push(Interval {
            s: norm_ang(pos + base.s),
            l: bl - pos,
        });
    }
    res
}

pub fn map_arc_to_rotated(
    src_c: &Circle,
    dst_c: &Circle,
    iv: &Interval,
    axis: DVec3,
    angle: f64,
) -> Interval {
    let r0 = rotate_v(src_c.circ_pt(iv.s), axis, angle).normalize();
    Interval {
        s: norm_ang(dst_c.pt_ang(r0)),
        l: iv.l,
    }
}

pub fn same_circle(c1: &Circle, c2: &Circle) -> bool {
    c1.pole.dot(c2.pole) > 1.0 - 1e-6 && (c1.colat - c2.colat).abs() < 1e-6
}

pub fn find_circ(list: &[Circle], circ: &Circle) -> Option<usize> {
    list.iter().position(|c| same_circle(c, circ))
}

/// Attempt to derive the geometric parameters used to generate a dihedral
/// angle given the angle and a small epsilon to allow for rounding errors
///
/// Alternatively this can be used to find fudged axis angles by specifying a
/// reasonably large epsilon.
///
/// cos(t) = (cos(pi*a)*cos(pi*b) - cos(pi*p/q)) / (sin(pi*a)*sin(pi*b))
/// p/q = acos((cos(pi*a)*cos(pi*b) - sin(pi*a)*sin(pi*b)*cos(t)) / (sin(pi*a)*sin(pi*b))) / pi
pub fn invert_axis_angle(axis_angle_rad: f64, epsilon: f64) -> Vec<(u32, u32, u32, u32, f64)> {
    // sweep a and b [2,8], then solve for p and q and check for close integer fractions
    let mut results = Vec::new();
    for a in 2..=8 {
        for b in a..=8 {
            let cos_t = axis_angle_rad.cos();
            let sin_pi_a = (PI / a as f64).sin();
            let sin_pi_b = (PI / b as f64).sin();
            let cos_pi_a = (PI / a as f64).cos();
            let cos_pi_b = (PI / b as f64).cos();
            let cos_p_q = cos_pi_a * cos_pi_b - sin_pi_a * sin_pi_b * cos_t;
            if !(-1.0 - 1e-9..=1.0 + 1e-9).contains(&cos_p_q) {
                continue;
            }
            let p_q = cos_p_q.clamp(-1.0, 1.0).acos() / PI;
            let mut pq_checked = HashSet::new();

            // sweep denominators [2,15] and round the numerator, feed back into equation
            for q in 2..=15 {
                // if the result is within epsilon of the original axis angle, add to results
                let p = (p_q * q as f64).round() as u32;
                if !(1..=15).contains(&p) {
                    continue;
                }
                let derived_axis_angle = derive_axis_angle(a, b, p, q);
                if let Some(derived_axis_angle) = derived_axis_angle
                    && pq_checked.insert(((p as f64 / q as f64) * 100000f64).round() as u32)
                {
                    let diff = (derived_axis_angle - axis_angle_rad).abs();
                    if diff < epsilon {
                        results.push((a, b, p, q, diff));
                    }
                }
            }
        }
    }
    // Sort by epsilon
    results.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap());
    results
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_invert_axis_angle() {
    let axis_angle_rad = 63.356f64.to_radians();
    let epsilon = 1e-2;
    let result = invert_axis_angle(axis_angle_rad, epsilon);
    println!("result: {:?}", result);
}

/// Derive the dihedral angle between two faces for a given set of
/// geometric parameters using the law of cosines
pub fn derive_axis_angle(n_a: u32, n_b: u32, p: u32, q: u32) -> Option<f64> {
    let c_a = (PI / n_a as f64).cos();
    let s_a = (PI / n_a as f64).sin();
    let c_b = (PI / n_b as f64).cos();
    let s_b = (PI / n_b as f64).sin();
    let c_g = (PI * p as f64 / q as f64).cos();
    let denom = s_a * s_b;
    if denom.abs() < 1e-12 {
        return None;
    }
    let cos_t = (c_a * c_b - c_g) / denom;
    if !(-1.0 - 1e-9..=1.0 + 1e-9).contains(&cos_t) {
        return None;
    }
    Some(cos_t.clamp(-1.0, 1.0).acos())
}

/// Each axis is (direction_unit_vec, colat_radians, rotational_symmetry_n)
pub fn compute_arcs(
    axes: &[(DVec3, f64, u32)],
    max_iterations_cap: Option<usize>,
) -> (Vec<Circle>, Vec<Arc>) {
    let mut circles = Vec::new();
    let mut covered: Vec<Vec<Interval>> = Vec::new();
    let mut disp_arcs = Vec::new();

    // Seed one full circle per axis
    for &(axis, colat, _n) in axes {
        let ci = circles.len();
        circles.push(Circle::new(axis, colat));
        covered.push(vec![Interval { s: 0.0, l: TAU }]);
        disp_arcs.push(Arc {
            circ_idx: ci,
            s: 0.0,
            l: TAU,
        });
    }

    let mut step_start = 0;
    let max_iterations = max_iterations_cap.unwrap_or(AUTO_MAX_ITERS);
    for _ in 0..max_iterations {
        let before = disp_arcs.len();
        let mut bailout = false;

        for &(axis, cap_colat, n) in axes {
            if bailout {
                break;
            }

            for ai in step_start..before {
                if bailout {
                    break;
                }
                let arc = disp_arcs[ai];
                let src_c = circles[arc.circ_idx];
                let cr = match cap_range(&src_c, axis, cap_colat) {
                    Some(v) => v,
                    None => continue,
                };
                let clipped = isect_iv(&Interval { s: arc.s, l: arc.l }, &cr);
                if clipped.is_empty() {
                    continue;
                }

                for k in 1..n {
                    if bailout {
                        break;
                    }
                    let rot_ang = k as f64 * TAU / n as f64;
                    let rot_pole = rotate_v(src_c.pole, axis, rot_ang).normalize();
                    let rot_c = Circle::new(rot_pole, src_c.colat);
                    let rci = match find_circ(&circles, &rot_c) {
                        Some(idx) => idx,
                        None => {
                            let idx = circles.len();
                            circles.push(rot_c);
                            covered.push(Vec::new());
                            idx
                        }
                    };
                    let dst_c = circles[rci];
                    for clip in &clipped {
                        let rot_iv = map_arc_to_rotated(&src_c, &dst_c, clip, axis, rot_ang);
                        let remaining = subtract_iv(&rot_iv, &covered[rci]);
                        for r in remaining {
                            if r.l > 1e-6 {
                                disp_arcs.push(Arc {
                                    circ_idx: rci,
                                    s: r.s,
                                    l: r.l,
                                });
                                covered[rci].push(Interval { s: r.s, l: r.l });
                                if disp_arcs.len() - before > 1000 {
                                    bailout = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if disp_arcs.len() == before {
            break;
        }
        step_start = before;
    }
    (circles, disp_arcs)
}

/// Given two existing axis unit vectors A and B, plus two dihedral angles
/// (B-to-C and C-to-A), solve for the third axis C.
///
/// Returns the two mirror solutions (c_positive, c_negative) sitting on
/// opposite sides of the A-B great-circle plane.
pub fn derive_third_axis(
    a_vec: DVec3,
    b_vec: DVec3,
    angle_bc: f64,
    angle_ca: f64,
) -> Result<(DVec3, DVec3), &'static str> {
    let a = a_vec.normalize();
    let b = b_vec.normalize();

    let cos_ab = a.dot(b);
    let cross_ab = a.cross(b);
    let sin_ab = cross_ab.length();

    if sin_ab < 1e-10 {
        return Err("A and B axes are parallel");
    }

    let e1 = a;
    let e2 = (b - cos_ab * a) / sin_ab;
    let e3 = cross_ab / sin_ab;

    let x = angle_ca.cos();
    let y = (angle_bc.cos() - x * cos_ab) / sin_ab;
    let z_sq = 1.0 - x * x - y * y;

    if z_sq < -1e-9 {
        return Err("No solution: angle constraints are geometrically inconsistent");
    }

    let z = z_sq.max(0.0).sqrt();

    let c_pos = (x * e1 + y * e2 + z * e3).normalize();
    let c_neg = (x * e1 + y * e2 - z * e3).normalize();

    Ok((c_pos, c_neg))
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_derive_third_axis() {
    let a_vec = DVec3::new(1.0, 0.0, 0.0);
    let perp = DVec3::new(0.0, 1.0, 0.0);

    let angle_ab = derive_axis_angle(3, 5, 2, 5).unwrap();
    let b_vec = rotate_v(a_vec, perp, angle_ab).normalize();

    let angle_bc = derive_axis_angle(5, 5, 1, 3).unwrap();
    let angle_ca = derive_axis_angle(5, 3, 1, 2).unwrap();

    println!("A: {:?}", a_vec);
    println!("B: {:?}", b_vec);
    println!("Angle A-B: {:.4}°", angle_ab.to_degrees());
    println!("Angle B-C: {:.4}°", angle_bc.to_degrees());
    println!("Angle C-A: {:.4}°", angle_ca.to_degrees());

    let (c1, c2) =
        derive_third_axis(a_vec, b_vec, angle_bc, angle_ca).expect("Failed to derive third axis");

    println!("C (solution +): {:?}", c1);
    println!("C (solution -): {:?}", c2);

    let verify = |label: &str, c: DVec3| {
        let actual_bc = b_vec.dot(c).clamp(-1.0, 1.0).acos();
        let actual_ca = c.dot(a_vec).clamp(-1.0, 1.0).acos();
        println!(
            "  {} => B-C: {:.4}° (expect {:.4}°), C-A: {:.4}° (expect {:.4}°)",
            label,
            actual_bc.to_degrees(),
            angle_bc.to_degrees(),
            actual_ca.to_degrees(),
            angle_ca.to_degrees()
        );
        assert!(
            (actual_bc - angle_bc).abs() < 1e-10,
            "B-C mismatch for {}",
            label
        );
        assert!(
            (actual_ca - angle_ca).abs() < 1e-10,
            "C-A mismatch for {}",
            label
        );
    };

    verify("C+", c1);
    verify("C-", c2);
}

pub fn merge_arcs(arcs: &[Arc]) -> Vec<Arc> {
    let mut by_circ: BTreeMap<usize, Vec<(f64, f64)>> = BTreeMap::new();
    for a in arcs {
        by_circ.entry(a.circ_idx).or_default().push((a.s, a.l));
    }
    let mut merged = Vec::new();
    for (ci, mut segs) in by_circ {
        segs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut result: Vec<(f64, f64)> = vec![segs[0]];
        for seg in segs.iter().skip(1) {
            let (prev_s, prev_l) = *result.last().unwrap();
            let end = norm_ang(prev_s + prev_l);
            let gap = norm_ang(seg.0 - end);
            if gap < 1e-4 {
                result.last_mut().unwrap().1 += gap + seg.1;
            } else {
                result.push(*seg);
            }
        }
        if result.len() > 1 {
            let (last_s, last_l) = result.last().unwrap();
            let (first_s, first_l) = result[0];
            let end = norm_ang(last_s + last_l);
            let gap = norm_ang(first_s - end);
            if gap < 1e-4 {
                result[0] = (*last_s, *last_l + gap + first_l);
                result.pop();
            }
        }
        for (s, l) in result {
            merged.push(Arc {
                circ_idx: ci,
                s,
                l: l.min(TAU),
            });
        }
    }
    merged
}
