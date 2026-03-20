//! Dreadnaut script macros
//!

#![allow(unused_macros)]

/// Set line length for output (0 = no limit)
macro_rules! dn_linelength {
    ($n:expr) => {
        format!("l={}\n", $n)
    };
}

/// Switch to sparse mode (adjacency list; uses sparse nauty)
macro_rules! dn_sparse_mode {
    () => {
        "As\n"
    };
}

/// Set level markers mode -m or m
macro_rules! dn_set_level_markers {
    ($on:expr) => {
        if $on { "-m\n" } else { "m\n" }
    };
}

/// Output a literal string (newline not included)
macro_rules! dn_print_literal {
    ($n:expr) => {
        format!("\"{}\"\n", $n)
    };
}

/// Mark graph as directed (digraph)
macro_rules! dn_digraph {
    () => {
        "d\n"
    };
}

/// Set number of vertices and begin graph input
macro_rules! dn_begin_graph {
    ($n:expr) => {
        format!("n={} g\n", $n)
    };
}

/// Enable canonical labeling if true
macro_rules! dn_set_canonical_labeling {
    ($on:expr) => {
        if $on { "c\n" } else { "-c\n" }
    };
}

/// Enable automorphism output if true
macro_rules! dn_set_automorphism_output {
    ($on:expr) => {
        if $on { "a\n" } else { "-a\n" }
    };
}

/// Execute nauty/Traces
macro_rules! dn_execute {
    () => {
        "x\n"
    };
}

/// Output canonical label and canonically labelled graph
macro_rules! dn_canonical_label {
    () => {
        "b\n"
    };
}

/// Output hash (three 8-digit hex numbers) for quick isomorphism comparison
macro_rules! dn_hash {
    () => {
        "z\n"
    };
}
