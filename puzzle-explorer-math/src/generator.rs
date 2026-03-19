use std::collections::{HashMap, HashSet};

pub trait Generator: Sized {
    /// Renumber the generator such that it's n-indexed with no missing labels.
    /// Returns the renumbered generator and the number of vertices
    fn renumber(&self, start_idx: usize) -> (Self, usize);

    /// Get generator's unique vertex labels in order of appearance
    fn get_unique_vertices(&self) -> Vec<usize>;

    fn to_gap_string(&self) -> String;
}

impl Generator for Vec<Vec<Vec<usize>>> {
    fn renumber(&self, start_idx: usize) -> (Vec<Vec<Vec<usize>>>, usize) {
        let labels = self.get_unique_vertices();
        let label_to_index: HashMap<usize, usize> = labels
            .iter()
            .copied()
            .enumerate()
            .map(|(index, label)| (label, index + start_idx))
            .collect();

        let gen_renumbered: Vec<Vec<Vec<usize>>> = self
            .iter()
            .map(|generator| {
                generator
                    .iter()
                    .map(|cycle| cycle.iter().map(|vertex| label_to_index[vertex]).collect())
                    .collect()
            })
            .collect();

        (gen_renumbered, labels.len())
    }

    fn get_unique_vertices(&self) -> Vec<usize> {
        let mut seen = HashSet::new();
        let mut labels = Vec::new();
        for generator in self {
            for cycle in generator {
                for &vertex in cycle {
                    if seen.insert(vertex) {
                        labels.push(vertex);
                    }
                }
            }
        }
        labels
    }

    fn to_gap_string(&self) -> String {
        generator_to_gap_string(0, self)
    }
}

pub fn generator_to_gap_string(offset: usize, generator: &[Vec<Vec<usize>>]) -> String {
    if generator.is_empty() {
        return "[]".to_string();
    }
    format!(
        "[({})]",
        generator
            .iter()
            .map(|generator| {
                generator
                    .iter()
                    .map(|cycle| {
                        cycle
                            .iter()
                            .map(|vertex| (vertex + offset).to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .collect::<Vec<_>>()
                    .join(")(")
            })
            .collect::<Vec<_>>()
            .join("),(")
    )
}

/// Parses GAP generator notation like `[(1,2)(3,4),(5,6)]`
/// Each operation is a list of cycles, each cycle is a list of element indices
pub fn parse_gap_string(generator_string: &str) -> Option<Vec<Vec<Vec<usize>>>> {
    let s = generator_string
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    if s.len() < 2 || !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }
    let inner = &s[1..s.len() - 1];
    if inner.is_empty() {
        return Some(vec![]);
    }

    let mut operations = Vec::new();
    for part in inner.split("),(") {
        let mut operation = Vec::new();
        for cycle_str in part.split(")(") {
            let cleaned: String = cycle_str
                .chars()
                .filter(|c| *c != '(' && *c != ')')
                .collect();
            let cycle: Vec<usize> = if cleaned.is_empty() {
                vec![]
            } else {
                cleaned
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse().ok())
                    .collect::<Option<Vec<_>>>()?
            };
            operation.push(cycle);
        }
        operations.push(operation);
    }
    if operations.is_empty() {
        operations.push(vec![]);
    }
    Some(operations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_renumber_generator() {
        let gen_raw = vec![vec![vec![1, 9, 3], vec![4, 3, 6]]];
        let (gen_renumbered, num_vertices) = gen_raw.renumber(0);
        assert_eq!(gen_renumbered, vec![vec![vec![0, 1, 2], vec![3, 2, 4]]]);
        assert_eq!(num_vertices, 5);

        let gen_raw = vec![vec![vec![], vec![5, 9, 2]]];
        let (gen_renumbered, num_vertices) = gen_raw.renumber(1);
        assert_eq!(gen_renumbered, vec![vec![vec![], vec![1, 2, 3]]]);
        assert_eq!(num_vertices, 3);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_parse_gap_string() {
        let result = parse_gap_string("[(1,2,5)(3,4)]");
        assert_eq!(result, Some(vec![vec![vec![1, 2, 5], vec![3, 4]]]));

        let result = parse_gap_string("[(1,2)(3,4),(5,6,9)]");
        assert_eq!(
            result,
            Some(vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6, 9]]])
        );

        let result = parse_gap_string("[(),(1,2)]");
        assert_eq!(result, Some(vec![vec![vec![]], vec![vec![1, 2]]]));

        let result = parse_gap_string("[( ),(1,2)]");
        assert_eq!(result, Some(vec![vec![vec![]], vec![vec![1, 2]]]));

        let result = parse_gap_string("[( ),( 1, 2)]");
        assert_eq!(result, Some(vec![vec![vec![]], vec![vec![1, 2]]]));

        let result = parse_gap_string("[(), (1,2 )]");
        assert_eq!(result, Some(vec![vec![vec![]], vec![vec![1, 2]]]));

        assert_eq!(parse_gap_string("[]"), Some(vec![]));
        assert_eq!(parse_gap_string("[ ]"), Some(vec![]));

        assert_eq!(parse_gap_string(""), None);
        assert_eq!(parse_gap_string("foo"), None);
        assert_eq!(parse_gap_string("("), None);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_to_gap_string() {
        assert_eq!(vec![].to_gap_string(), "[]");

        // Undefined, ignore
        //assert_eq!(vec![vec![]].to_gap_string(), "[()]");

        assert_eq!(vec![vec![vec![]]].to_gap_string(), "[()]");

        assert_eq!(
            vec![vec![vec![]], vec![vec![1, 2]]].to_gap_string(),
            "[(),(1,2)]"
        );

        assert_eq!(
            vec![vec![vec![1, 2, 5], vec![3, 4]]].to_gap_string(),
            "[(1,2,5)(3,4)]"
        );

        assert_eq!(
            vec![vec![vec![]], vec![vec![1, 2]]].to_gap_string(),
            "[(),(1,2)]"
        );
    }
}
