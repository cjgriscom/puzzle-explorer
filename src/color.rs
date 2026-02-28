// --- Orbit Colors (direct from JS/CSS version) ---

#[rustfmt::skip]
pub const ORBIT_COLORS: &[(&str, [f32; 3])] = &[
    ("crimson",     [0.86, 0.08, 0.24]),
    ("forestgreen", [0.13, 0.55, 0.13]),
    ("royalblue",   [0.25, 0.41, 0.88]),
    ("darkorange",  [1.00, 0.55, 0.00]),
    ("darkviolet",  [0.58, 0.00, 0.83]),
    ("deepskyblue", [0.00, 0.75, 1.00]),
    ("deeppink",    [1.00, 0.08, 0.58]),
    ("yellowgreen", [0.60, 0.80, 0.20]),
    ("lightcoral",  [0.94, 0.50, 0.50]),
    ("teal",        [0.00, 0.50, 0.50]),
    ("plum",        [0.87, 0.63, 0.87]),
    ("saddlebrown", [0.55, 0.27, 0.07]),
];
pub const SINGLETON_COLOR: (&str, [f32; 3]) = ("darkgray", [0.41, 0.41, 0.41]);

pub fn color_to_hex(c: &[f32; 3]) -> u32 {
    let r = (c[0] * 255.0) as u32;
    let g = (c[1] * 255.0) as u32;
    let b = (c[2] * 255.0) as u32;
    (r << 16) | (g << 8) | b
}
