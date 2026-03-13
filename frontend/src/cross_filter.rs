use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;

use crate::app::SpreadsheetData;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    And,
    Or,
}

pub struct ValueSelection {
    pub value: String,
    pub included: Mutable<bool>,
    pub excluded: Mutable<bool>,
}

pub struct CrossFilter {
    pub key_column: Mutable<Option<usize>>,
    pub values_column: Mutable<Option<usize>>,
    pub match_mode: Mutable<MatchMode>,
    pub selected_values: MutableVec<Arc<ValueSelection>>,
    pub results: MutableVec<String>,
    pub matching_keys: Mutable<Option<HashSet<String>>>,
    pub panel_open: Mutable<bool>,
}

impl CrossFilter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            key_column: Mutable::new(None),
            values_column: Mutable::new(None),
            match_mode: Mutable::new(MatchMode::And),
            selected_values: MutableVec::new(),
            results: MutableVec::new(),
            matching_keys: Mutable::new(None),
            panel_open: Mutable::new(false),
        })
    }

    pub fn reset(&self) {
        self.key_column.set(None);
        self.values_column.set(None);
        self.match_mode.set(MatchMode::And);
        self.selected_values.lock_mut().clear();
        self.results.lock_mut().clear();
        self.matching_keys.set(None);
    }

    pub fn clear_filter(&self) {
        self.key_column.set(None);
        self.values_column.set(None);
        self.results.lock_mut().clear();
        self.matching_keys.set(None);
    }

    pub fn update_unique_values(&self, data: &SpreadsheetData) {
        let values_col = match *self.values_column.lock_ref() {
            Some(c) => c,
            None => {
                self.selected_values.lock_mut().clear();
                self.results.lock_mut().clear();
                self.matching_keys.set(None);
                return;
            }
        };

        let mut unique = HashSet::new();
        for row in &data.rows {
            if let Some(val) = row.get(values_col) {
                if !val.is_empty() {
                    unique.insert(val.clone());
                }
            }
        }

        let mut sorted: Vec<String> = unique.into_iter().collect();
        sorted.sort();

        let selections: Vec<Arc<ValueSelection>> = sorted
            .into_iter()
            .map(|v| {
                Arc::new(ValueSelection {
                    value: v,
                    included: Mutable::new(true),
                    excluded: Mutable::new(false),
                })
            })
            .collect();

        self.selected_values.lock_mut().replace_cloned(selections);
        self.compute_results(data);
    }

    pub fn compute_results(&self, data: &SpreadsheetData) {
        let key_col = match *self.key_column.lock_ref() {
            Some(c) => c,
            None => {
                self.results.lock_mut().clear();
                self.matching_keys.set(None);
                return;
            }
        };

        let values_col = match *self.values_column.lock_ref() {
            Some(c) => c,
            None => {
                self.results.lock_mut().clear();
                self.matching_keys.set(None);
                return;
            }
        };

        let included: HashSet<String> = self
            .selected_values
            .lock_ref()
            .iter()
            .filter(|v| v.included.get())
            .map(|v| v.value.clone())
            .collect();

        let excluded: HashSet<String> = self
            .selected_values
            .lock_ref()
            .iter()
            .filter(|v| v.excluded.get())
            .map(|v| v.value.clone())
            .collect();

        if included.is_empty() && excluded.is_empty() {
            self.results.lock_mut().clear();
            self.matching_keys.set(Some(HashSet::new()));
            return;
        }

        // Build key -> set of values
        let mut key_values: HashMap<String, HashSet<String>> = HashMap::new();
        for row in &data.rows {
            let key = row.get(key_col).cloned().unwrap_or_default();
            let val = row.get(values_col).cloned().unwrap_or_default();
            if !key.is_empty() && !val.is_empty() {
                key_values.entry(key).or_default().insert(val);
            }
        }

        let mode = *self.match_mode.lock_ref();
        let matching: HashSet<String> = key_values
            .into_iter()
            .filter(|(_, vals)| {
                // Exclude: key must NOT have any excluded value
                if excluded.iter().any(|e| vals.contains(e)) {
                    return false;
                }
                // Include (if any are selected)
                if included.is_empty() {
                    return true;
                }
                match mode {
                    MatchMode::And => included.iter().all(|s| vals.contains(s)),
                    MatchMode::Or => included.iter().any(|s| vals.contains(s)),
                }
            })
            .map(|(key, _)| key)
            .collect();

        let mut sorted: Vec<String> = matching.iter().cloned().collect();
        sorted.sort();

        self.results.lock_mut().replace_cloned(sorted);
        self.matching_keys.set(Some(matching));
    }
}
