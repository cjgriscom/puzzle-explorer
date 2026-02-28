// --- App State ---

#[derive(Clone, Debug, PartialEq)]
pub struct PuzzleParams {
    pub n_a: u32,
    pub n_b: u32,
    pub p: u32,
    pub q: u32,
    pub colat_a: f32,
    pub colat_b: f32,
    pub lock_cuts: bool,
    pub show_pieces: bool,
}

impl Default for PuzzleParams {
    fn default() -> Self {
        Self {
            n_a: 3,
            n_b: 2,
            p: 1,
            q: 3,
            colat_a: 119.4,
            colat_b: 119.4,
            lock_cuts: true,
            show_pieces: true,
        }
    }
}
