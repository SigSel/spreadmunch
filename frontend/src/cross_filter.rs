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
    pub summary: Mutable<Option<String>>,
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
            summary: Mutable::new(None),
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
        self.summary.set(None);
    }

    pub fn clear_filter(&self) {
        self.key_column.set(None);
        self.values_column.set(None);
        self.selected_values.lock_mut().clear();
        self.results.lock_mut().clear();
        self.matching_keys.set(None);
        self.summary.set(None);
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
                self.summary.set(None);
                return;
            }
        };

        let values_col = match *self.values_column.lock_ref() {
            Some(c) => c,
            None => {
                self.results.lock_mut().clear();
                self.matching_keys.set(None);
                self.summary.set(None);
                return;
            }
        };

        let vals_ref = self.selected_values.lock_ref();

        let mut inc_list: Vec<String> = vals_ref
            .iter()
            .filter(|v| v.included.get())
            .map(|v| v.value.clone())
            .collect();
        inc_list.sort();

        let mut exc_list: Vec<String> = vals_ref
            .iter()
            .filter(|v| v.excluded.get())
            .map(|v| v.value.clone())
            .collect();
        exc_list.sort();

        let included: HashSet<String> = inc_list.iter().cloned().collect();
        let excluded: HashSet<String> = exc_list.iter().cloned().collect();

        drop(vals_ref);

        if included.is_empty() && excluded.is_empty() {
            self.results.lock_mut().clear();
            self.matching_keys.set(Some(HashSet::new()));
            self.summary.set(None);
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

        // Build summary text
        let key_name = data.headers.get(key_col).cloned().unwrap_or_default();
        let val_name = data.headers.get(values_col).cloned().unwrap_or_default();
        let mode_str = match mode {
            MatchMode::And => " and ",
            MatchMode::Or => " or ",
        };

        let mut summary = format!(
            "Filtering \"{key_name}\" by \"{val_name}\" \u{2014} ",
        );

        if !inc_list.is_empty() {
            summary.push_str("has ");
            summary.push_str(&inc_list.join(mode_str));
        }

        if !exc_list.is_empty() {
            if !inc_list.is_empty() {
                summary.push_str(", but not ");
            } else {
                summary.push_str("does not have ");
            }
            summary.push_str(&exc_list.join(" or "));
        }

        summary.push_str(&format!(" ({} match)", sorted.len()));

        self.summary.set(Some(summary));
        self.results.lock_mut().replace_cloned(sorted);
        self.matching_keys.set(Some(matching));
    }
}
