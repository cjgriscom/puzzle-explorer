use std::collections::{BTreeSet, HashMap};

use glam::DVec3;
use serde::{Deserialize, Serialize};

/// Closable window states
#[derive(Clone, Debug, PartialEq)]
pub struct WindowState {
    pub show_controls: bool,
    pub show_measure_axis_angle: bool,
    pub show_invert_axis_angle: bool,
    pub show_gap_console: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            show_controls: false,
            show_measure_axis_angle: false,
            show_invert_axis_angle: false,
            show_gap_console: true,
        }
    }
}

/// Colatitude of a cut, or a link to the axis above.
///
/// Serialized as a number of degrees, or the string "MergeUp".
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColatitudeDeg {
    Degrees(f32),
    MergeUp,
}

impl ColatitudeDeg {
    pub const DEFAULT_DEGREES: f32 = 89.0;

    pub fn degrees(self) -> Option<f32> {
        match self {
            ColatitudeDeg::Degrees(v) => Some(v),
            ColatitudeDeg::MergeUp => None,
        }
    }

    pub fn is_merge_up(self) -> bool {
        matches!(self, ColatitudeDeg::MergeUp)
    }
}

impl Serialize for ColatitudeDeg {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            ColatitudeDeg::Degrees(v) => serializer.serialize_f32(v),
            ColatitudeDeg::MergeUp => serializer.serialize_str("MergeUp"),
        }
    }
}

impl<'de> Deserialize<'de> for ColatitudeDeg {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Degrees(f32),
            Tag(String),
        }
        match Raw::deserialize(deserializer)? {
            Raw::Degrees(v) => Ok(ColatitudeDeg::Degrees(v)),
            Raw::Tag(s) if s == "MergeUp" => Ok(ColatitudeDeg::MergeUp),
            Raw::Tag(s) => Err(serde::de::Error::custom(format!(
                "expected a number or \"MergeUp\", got {s:?}"
            ))),
        }
    }
}

/// Axis entry in puzzle params
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisEntry {
    pub axis_name: String, // references an axis definition (or X/Y/Z)
    pub n: u32,
    pub colatitude_deg: ColatitudeDeg,
    pub n_match: bool, // when true, n syncs from CosineRule definition
    pub enabled: bool, // when false, axis is skipped during geometry build
}

impl AxisEntry {
    /// Concrete colatitude, walking MergeUp links toward lower indices.
    ///
    /// A MergeUp at index 0 is invalid and resolves to [`ColatitudeDeg::DEFAULT_DEGREES`].
    pub fn resolved_colatitude_deg(axes: &[Self], idx: usize) -> f32 {
        let mut i = idx;
        loop {
            match axes.get(i).map(|a| a.colatitude_deg) {
                Some(ColatitudeDeg::Degrees(v)) => return v,
                Some(ColatitudeDeg::MergeUp) if i > 0 => i -= 1,
                _ => return ColatitudeDeg::DEFAULT_DEGREES,
            }
        }
    }

    /// Replace a MergeUp entry with the concrete degrees it currently resolves to.
    pub fn materialize_colatitude(axes: &mut [Self], idx: usize) {
        if idx >= axes.len() || !axes[idx].colatitude_deg.is_merge_up() {
            return;
        }
        let degrees = Self::resolved_colatitude_deg(axes, idx);
        axes[idx].colatitude_deg = ColatitudeDeg::Degrees(degrees);
    }

    /// Copy the first axis's concrete colatitude onto every non-MergeUp entry.
    pub fn unify_concrete_colatitudes(axes: &mut [Self]) {
        let Some(first) = axes.first().and_then(|a| a.colatitude_deg.degrees()) else {
            return;
        };
        let unified = ColatitudeDeg::Degrees(first);
        for entry in axes.iter_mut() {
            if !entry.colatitude_deg.is_merge_up() {
                entry.colatitude_deg = unified;
            }
        }
    }

    /// Swap `idx` with the axis above it.
    ///
    /// When a MergeUp axis swaps with the top axis, their colatitudes are exchanged.
    /// The new top keeps the concrete degrees, and the axis that moved down becomes MergeUp.
    pub fn move_up(axes: &mut [Self], idx: usize) {
        if idx == 0 || idx >= axes.len() {
            return;
        }
        let moving_is_merge_up = axes[idx].colatitude_deg.is_merge_up();
        axes.swap(idx - 1, idx);
        if idx == 1 && moving_is_merge_up {
            let moved_down = axes[idx].colatitude_deg;
            axes[idx].colatitude_deg = axes[idx - 1].colatitude_deg;
            axes[idx - 1].colatitude_deg = moved_down;
        }
    }

    /// Remove `idx`.
    ///
    /// If that axis is the concrete head of a MergeUp chain, the next axis keeps
    /// that concrete colatitude instead of linking to whatever is now above it.
    pub fn remove_at(axes: &mut Vec<Self>, idx: usize) {
        if idx >= axes.len() {
            return;
        }
        let next = idx + 1;
        if next < axes.len()
            && !axes[idx].colatitude_deg.is_merge_up()
            && axes[next].colatitude_deg.is_merge_up()
        {
            Self::materialize_colatitude(axes, next);
        }
        axes.remove(idx);
    }
}

impl Default for AxisEntry {
    fn default() -> Self {
        Self {
            axis_name: String::new(),
            n: 3,
            colatitude_deg: ColatitudeDeg::Degrees(ColatitudeDeg::DEFAULT_DEGREES),
            n_match: false,
            enabled: true,
        }
    }
}

/// Puzzle parameters window state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PuzzleParams {
    #[serde(skip)]
    pub show_axes: bool,
    pub max_iterations: u32,
    pub lock_cuts: bool,
    pub axes: Vec<AxisEntry>,
}

impl PuzzleParams {
    /// Apply imported data, preserving transient UI state
    pub fn apply_imported(&mut self, imported: &Self) {
        self.max_iterations = imported.max_iterations;
        self.lock_cuts = imported.lock_cuts;
        self.axes = imported.axes.clone();
        AxisEntry::materialize_colatitude(&mut self.axes, 0);
    }
}

impl Default for PuzzleParams {
    fn default() -> Self {
        Self {
            show_axes: true,
            max_iterations: 30,
            lock_cuts: true,
            axes: Vec::new(),
        }
    }
}

/// Fudged mode settings
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FudgedModeSettings {
    pub min_piece_angle_deg: f32,
    pub min_piece_perimeter: f64,
}

impl Default for FudgedModeSettings {
    fn default() -> Self {
        Self {
            min_piece_angle_deg: 5.0,
            min_piece_perimeter: 0.05,
        }
    }
}

/// Orbit analysis window state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrbitAnalysisState {
    #[serde(skip)]
    pub annotate_pieces: bool,
    #[serde(skip)]
    pub number_pieces: bool,
    #[serde(skip)]
    pub auto_update_orbits: bool,
    #[serde(skip)]
    pub auto_update_groups: bool,
    #[serde(skip)]
    pub orbits_stale: bool,
    #[serde(skip)]
    pub groups_stale: bool,

    pub fudged_mode: bool,
    pub fudged_mode_settings: FudgedModeSettings,
}

impl OrbitAnalysisState {
    /// Apply imported data, preserving transient UI state
    pub fn apply_imported(&mut self, imported: &Self) {
        self.fudged_mode = imported.fudged_mode;
        self.fudged_mode_settings = imported.fudged_mode_settings.clone();
    }
}

impl Default for OrbitAnalysisState {
    fn default() -> Self {
        Self {
            annotate_pieces: true,
            number_pieces: false,
            auto_update_orbits: false,
            auto_update_groups: false,
            orbits_stale: false,
            groups_stale: false,
            fudged_mode: false,
            fudged_mode_settings: FudgedModeSettings::default(),
        }
    }
}

/// Measure axis angle window state
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MeasureAxisAngleWindowState {
    pub axis_a: String,
    pub axis_b: String,
}

/// Invert axis angle window state
#[derive(Clone, Debug, PartialEq)]
pub struct InvertAxisAngleWindowState {
    pub angle_deg_input: f64,
    pub epsilon_decimal_places: f64,
    pub output_lines: Vec<String>,
}

impl Default for InvertAxisAngleWindowState {
    fn default() -> Self {
        Self {
            angle_deg_input: 138.1897f64,
            epsilon_decimal_places: 1.0,
            output_lines: Vec::new(),
        }
    }
}

/// Axis definitions window state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AxisDefinitions {
    /// User-defined axes in insertion order
    pub definitions: Vec<AxisDefinition>,
    /// Currently visible axes
    pub visible_axes: BTreeSet<String>,

    /// Resolved vectors results (includes built-in & sub-indexed)
    #[serde(skip)]
    pub resolved: HashMap<String, Result<Vec<DVec3>, String>>,
    /// When Some, rename dialog is active: (original_name, text_buffer)
    #[serde(skip)]
    pub rename_state: Option<(String, String)>,
    /// When Some, delete confirmation is pending for this axis
    #[serde(skip)]
    pub pending_delete: Option<String>,
}

impl AxisDefinitions {
    /// Apply imported data, preserving transient UI state
    pub fn apply_imported(&mut self, imported: &Self) {
        self.definitions = imported.definitions.clone();
        self.visible_axes = imported.visible_axes.clone();
        self.resolve_all();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisDefinition {
    pub name: String,
    pub axis: DerivedAxis,
}

/// Axis definition formats
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DerivedAxis {
    // Raw vector definition
    Vector {
        x: f64,
        y: f64,
        z: f64,
    },
    // Defines a normalized vector based on pitch and yaw
    Euler {
        pitch: f64,
        yaw: f64,
    },
    // Makes a copy of an axis for renaming or inverting it
    Copy {
        axis: String,
        invert: bool,
    },
    // Cross product of the normalized references
    CrossProduct {
        a0: String,
        a1: String,
    },
    // Normalizes and averages the referenced axes
    Average {
        axes: Vec<String>,
    },
    // Spherical law of cosines (puzzle-explorer-math::geometry::derive_axis_angle)
    CosineRule {
        p: u32,
        q: u32,
        n_a: u32,
        n_b: u32,
        a_axis: String,             // Axis A
        perpendicular_axis: String, // axis A is copied and rotated around this axis by the derived angle
        manual_axis_angle_deg: Option<f64>,
    },
    // Derive a third axis C from two existing axes A and B using two cosine rules
    ThirdAxis {
        a_axis: String,
        b_axis: String,
        n_a: u32,
        n_b: u32,
        n_c: u32,
        p_bc: u32,
        q_bc: u32,
        p_ca: u32,
        q_ca: u32,
        manual_angle_bc_deg: Option<f64>,
        manual_angle_ca_deg: Option<f64>,
        mirror: bool,
    },
    // Copy target_axis n times (2+) around pattern_axis along the angle range
    CircularPattern {
        pattern_axis: String,
        target_axis: String,
        n: u32,
        angle_range_deg: f64,
        invert_range: bool,
    },
    // Mirror target_axis across the plane defined by plane_axis_1 and plane_axis_2
    Mirror {
        target_axis: String,
        plane_axis_1: String,
        plane_axis_2: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(colatitude_deg: ColatitudeDeg) -> AxisEntry {
        AxisEntry {
            colatitude_deg,
            ..AxisEntry::default()
        }
    }

    #[test]
    fn colatitude_serializes_as_number_or_merge_up() {
        let degrees = ColatitudeDeg::Degrees(90.0);
        let degrees_yaml = serde_yaml_ng::to_string(&degrees).unwrap();
        assert!(degrees_yaml.contains("90"));
        assert!(!degrees_yaml.contains("Degrees"));
        let degrees_back: ColatitudeDeg = serde_yaml_ng::from_str(&degrees_yaml).unwrap();
        assert_eq!(degrees_back, degrees);

        let merge_yaml = serde_yaml_ng::to_string(&ColatitudeDeg::MergeUp).unwrap();
        assert!(merge_yaml.contains("MergeUp"));
        let merge_back: ColatitudeDeg = serde_yaml_ng::from_str(&merge_yaml).unwrap();
        assert_eq!(merge_back, ColatitudeDeg::MergeUp);

        let from_int: ColatitudeDeg = serde_yaml_ng::from_str("90").unwrap();
        assert_eq!(from_int, ColatitudeDeg::Degrees(90.0));
    }

    #[test]
    fn axis_entry_reads_merge_up_string() {
        let yaml = r#"
axis_name: A
n: 3
colatitude_deg: MergeUp
n_match: false
enabled: true
"#;
        let entry: AxisEntry = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(entry.colatitude_deg.is_merge_up());

        let written = serde_yaml_ng::to_string(&entry).unwrap();
        assert!(written.contains("colatitude_deg: MergeUp"));
    }

    #[test]
    fn example_puzzle_colatitudes_still_load() {
        let yaml = include_str!("examples/Biaxe.yml");
        let export = crate::puzzle_io::PuzzleExport::from_yaml(yaml).unwrap();
        assert_eq!(
            export.params.axes[0].colatitude_deg,
            ColatitudeDeg::Degrees(90.0)
        );
        assert_eq!(
            export.params.axes[1].colatitude_deg,
            ColatitudeDeg::Degrees(90.0)
        );
    }

    #[test]
    fn merge_up_chain_resolves_to_topmost_concrete_axis() {
        let axes = vec![
            entry(ColatitudeDeg::Degrees(12.5)),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::Degrees(40.0)),
            entry(ColatitudeDeg::MergeUp),
        ];
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 0), 12.5);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 1), 12.5);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 2), 12.5);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 3), 40.0);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 4), 40.0);
    }

    #[test]
    fn moving_merge_up_to_top_transfers_the_lock_downward() {
        let mut axes = vec![
            AxisEntry {
                axis_name: "top".into(),
                colatitude_deg: ColatitudeDeg::Degrees(33.0),
                ..AxisEntry::default()
            },
            AxisEntry {
                axis_name: "linked".into(),
                colatitude_deg: ColatitudeDeg::MergeUp,
                ..AxisEntry::default()
            },
            entry(ColatitudeDeg::MergeUp),
        ];
        AxisEntry::move_up(&mut axes, 1);
        assert_eq!(axes[0].axis_name, "linked");
        assert_eq!(axes[0].colatitude_deg, ColatitudeDeg::Degrees(33.0));
        assert_eq!(axes[1].axis_name, "top");
        assert!(axes[1].colatitude_deg.is_merge_up());
        assert!(axes[2].colatitude_deg.is_merge_up());
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 1), 33.0);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 2), 33.0);
    }

    #[test]
    fn removing_top_axis_materializes_the_new_top() {
        let mut axes = vec![
            entry(ColatitudeDeg::Degrees(33.0)),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::MergeUp),
        ];
        AxisEntry::remove_at(&mut axes, 0);
        assert_eq!(axes[0].colatitude_deg, ColatitudeDeg::Degrees(33.0));
        assert!(axes[1].colatitude_deg.is_merge_up());
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 1), 33.0);
    }

    #[test]
    fn deleting_a_chain_head_gives_its_value_to_the_next_axis() {
        let mut axes = vec![
            entry(ColatitudeDeg::Degrees(33.0)),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::Degrees(50.0)),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::MergeUp),
        ];
        AxisEntry::remove_at(&mut axes, 2);
        assert_eq!(axes[0].colatitude_deg, ColatitudeDeg::Degrees(33.0));
        assert!(axes[1].colatitude_deg.is_merge_up());
        assert_eq!(axes[2].colatitude_deg, ColatitudeDeg::Degrees(50.0));
        assert!(axes[3].colatitude_deg.is_merge_up());
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 1), 33.0);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 2), 50.0);
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 3), 50.0);
    }

    #[test]
    fn deleting_the_middle_of_a_chain_keeps_the_link() {
        let mut axes = vec![
            entry(ColatitudeDeg::Degrees(33.0)),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::MergeUp),
        ];
        AxisEntry::remove_at(&mut axes, 1);
        assert_eq!(axes[0].colatitude_deg, ColatitudeDeg::Degrees(33.0));
        assert!(axes[1].colatitude_deg.is_merge_up());
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 1), 33.0);
    }

    #[test]
    fn lock_cuts_unifies_concrete_values_and_keeps_merge_up() {
        let mut axes = vec![
            entry(ColatitudeDeg::Degrees(10.0)),
            entry(ColatitudeDeg::Degrees(20.0)),
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::Degrees(30.0)),
        ];
        AxisEntry::unify_concrete_colatitudes(&mut axes);
        assert_eq!(axes[0].colatitude_deg, ColatitudeDeg::Degrees(10.0));
        assert_eq!(axes[1].colatitude_deg, ColatitudeDeg::Degrees(10.0));
        assert!(axes[2].colatitude_deg.is_merge_up());
        assert_eq!(axes[3].colatitude_deg, ColatitudeDeg::Degrees(10.0));
        assert_eq!(AxisEntry::resolved_colatitude_deg(&axes, 2), 10.0);
    }

    #[test]
    fn imported_merge_up_on_top_axis_is_replaced() {
        let mut params = PuzzleParams::default();
        let mut imported = PuzzleParams::default();
        imported.axes = vec![
            entry(ColatitudeDeg::MergeUp),
            entry(ColatitudeDeg::Degrees(50.0)),
        ];
        params.apply_imported(&imported);
        assert_eq!(
            params.axes[0].colatitude_deg,
            ColatitudeDeg::Degrees(ColatitudeDeg::DEFAULT_DEGREES)
        );
        assert_eq!(params.axes[1].colatitude_deg, ColatitudeDeg::Degrees(50.0));
    }
}
