use std::collections::HashSet;
use std::sync::Arc;

use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;
use spreadmunch_core::cross_filter::{
    self as core_filter, CrossFilterInput, MatchMode as CoreMatchMode,
};
use spreadmunch_core::types::SpreadsheetData;

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

pub const MAX_DISPLAY_VALUES: usize = 20;

pub struct CrossFilter {
    pub key_column: Mutable<Option<usize>>,
    pub values_column: Mutable<Option<usize>>,
    pub match_mode: Mutable<MatchMode>,
    pub selected_values: MutableVec<Arc<ValueSelection>>,
    pub results: MutableVec<String>,
    pub matching_keys: Mutable<Option<HashSet<String>>>,
    pub summary: Mutable<Option<String>>,
    pub panel_open: Mutable<bool>,
    pub total_unique_count: Mutable<Option<usize>>,
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
            total_unique_count: Mutable::new(None),
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
        self.total_unique_count.set(None);
    }

    pub fn clear_filter(&self) {
        self.key_column.set(None);
        self.values_column.set(None);
        self.selected_values.lock_mut().clear();
        self.results.lock_mut().clear();
        self.matching_keys.set(None);
        self.summary.set(None);
        self.total_unique_count.set(None);
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

        let mut sorted = core_filter::get_unique_values(&data.rows, values_col);

        let total = sorted.len();
        if total > MAX_DISPLAY_VALUES {
            self.total_unique_count.set(Some(total));
            sorted.truncate(MAX_DISPLAY_VALUES);
        } else {
            self.total_unique_count.set(None);
        }

        let selections: Vec<Arc<ValueSelection>> = sorted
            .into_iter()
            .map(|v| {
                Arc::new(ValueSelection {
                    value: v,
                    included: Mutable::new(false),
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
            self.matching_keys.set(None);
            self.summary.set(None);
            return;
        }

        let mode = *self.match_mode.lock_ref();
        let core_mode = match mode {
            MatchMode::And => CoreMatchMode::And,
            MatchMode::Or => CoreMatchMode::Or,
        };

        let matching = core_filter::compute_cross_filter(
            &data.rows,
            &CrossFilterInput {
                key_col,
                values_col,
                included,
                excluded,
                mode: core_mode,
            },
        );

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
