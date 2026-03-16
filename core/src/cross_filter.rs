use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MatchMode {
    And,
    Or,
}

pub struct CrossFilterInput {
    pub key_col: usize,
    pub values_col: usize,
    pub included: HashSet<String>,
    pub excluded: HashSet<String>,
    pub mode: MatchMode,
}

pub fn compute_cross_filter(
    rows: &[Vec<String>],
    input: &CrossFilterInput,
) -> HashSet<String> {
    if input.included.is_empty() && input.excluded.is_empty() {
        return HashSet::new();
    }

    // Build key -> set of values
    let mut key_values: HashMap<String, HashSet<String>> = HashMap::new();
    for row in rows {
        let key = row.get(input.key_col).cloned().unwrap_or_default();
        let val = row.get(input.values_col).cloned().unwrap_or_default();
        if !key.is_empty() && !val.is_empty() {
            key_values.entry(key).or_default().insert(val);
        }
    }

    key_values
        .into_iter()
        .filter(|(_, vals)| {
            // Exclude: key must NOT have any excluded value
            if input.excluded.iter().any(|e| vals.contains(e)) {
                return false;
            }
            // Include (if any are selected)
            if input.included.is_empty() {
                return true;
            }
            match input.mode {
                MatchMode::And => input.included.iter().all(|s| vals.contains(s)),
                MatchMode::Or => input.included.iter().any(|s| vals.contains(s)),
            }
        })
        .map(|(key, _)| key)
        .collect()
}

pub fn get_unique_values(rows: &[Vec<String>], col: usize) -> Vec<String> {
    let mut unique = HashSet::new();
    for row in rows {
        if let Some(val) = row.get(col) {
            if !val.is_empty() {
                unique.insert(val.clone());
            }
        }
    }
    let mut sorted: Vec<String> = unique.into_iter().collect();
    sorted.sort();
    sorted
}
