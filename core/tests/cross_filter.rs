use std::collections::HashSet;
use std::path::Path;

use spreadmunch_core::cross_filter::{compute_cross_filter, get_unique_values, CrossFilterInput, MatchMode};

fn load_payments_rows() -> Vec<Vec<String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test_data/payments.csv");
    let mut reader = csv::Reader::from_path(path).expect("Failed to open payments.csv");
    reader
        .records()
        .map(|r| {
            let record = r.expect("Failed to read record");
            record.iter().map(|f| f.to_string()).collect()
        })
        .collect()
}

fn set(values: &[&str]) -> HashSet<String> {
    values.iter().map(|s| s.to_string()).collect()
}

fn assert_keys(result: &HashSet<String>, expected: &[&str]) {
    let expected_set = set(expected);
    assert_eq!(
        result, &expected_set,
        "\nExpected: {:?}\nGot: {:?}",
        expected_set, result
    );
}

// --- Include-only tests (AND mode) ---

#[test]
fn test_include_credit_card_and() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Credit Card"]),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Alice", "Charlie", "Eve"]);
}

#[test]
fn test_include_cash_and() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Cash"]),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Bob", "Charlie", "Eve", "Frank"]);
}

#[test]
fn test_include_credit_card_bank_transfer_and() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Credit Card", "Bank Transfer"]),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Alice", "Charlie"]);
}

#[test]
fn test_include_all_three_and() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Credit Card", "Bank Transfer", "Cash"]),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Charlie"]);
}

// --- Include-only tests (OR mode) ---

#[test]
fn test_include_credit_card_bank_transfer_or() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Credit Card", "Bank Transfer"]),
        excluded: HashSet::new(),
        mode: MatchMode::Or,
    });
    assert_keys(&result, &["Alice", "Charlie", "Diana", "Eve"]);
}

#[test]
fn test_include_all_three_or() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Credit Card", "Bank Transfer", "Cash"]),
        excluded: HashSet::new(),
        mode: MatchMode::Or,
    });
    assert_keys(&result, &["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"]);
}

// --- Exclude-only tests ---

#[test]
fn test_exclude_cash() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: HashSet::new(),
        excluded: set(&["Cash"]),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Alice", "Diana"]);
}

#[test]
fn test_exclude_credit_card() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: HashSet::new(),
        excluded: set(&["Credit Card"]),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Bob", "Diana", "Frank"]);
}

#[test]
fn test_exclude_bank_transfer() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: HashSet::new(),
        excluded: set(&["Bank Transfer"]),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Bob", "Eve", "Frank"]);
}

// --- Combined include + exclude tests ---

#[test]
fn test_include_credit_card_exclude_cash() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Credit Card"]),
        excluded: set(&["Cash"]),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Alice"]);
}

#[test]
fn test_include_bank_transfer_exclude_credit_card() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Bank Transfer"]),
        excluded: set(&["Credit Card"]),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Diana"]);
}

#[test]
fn test_include_cash_exclude_bank_transfer_or() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Cash"]),
        excluded: set(&["Bank Transfer"]),
        mode: MatchMode::Or,
    });
    assert_keys(&result, &["Bob", "Eve", "Frank"]);
}

// --- Edge cases ---

#[test]
fn test_no_filters() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: HashSet::new(),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert!(result.is_empty(), "No filters should return empty set");
}

#[test]
fn test_include_nonexistent_value() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Bitcoin"]),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert!(result.is_empty());
}

#[test]
fn test_exclude_nonexistent_value() {
    let rows = load_payments_rows();
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: HashSet::new(),
        excluded: set(&["Bitcoin"]),
        mode: MatchMode::And,
    });
    assert_keys(&result, &["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"]);
}

#[test]
fn test_empty_data() {
    let rows: Vec<Vec<String>> = vec![];
    let result = compute_cross_filter(&rows, &CrossFilterInput {
        key_col: 0,
        values_col: 1,
        included: set(&["Cash"]),
        excluded: HashSet::new(),
        mode: MatchMode::And,
    });
    assert!(result.is_empty());
}

// --- Unique values tests ---

#[test]
fn test_unique_values_customer_col() {
    let rows = load_payments_rows();
    let values = get_unique_values(&rows, 0);
    assert_eq!(values, vec!["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"]);
}

#[test]
fn test_unique_values_payment_type_col() {
    let rows = load_payments_rows();
    let values = get_unique_values(&rows, 1);
    assert_eq!(values, vec!["Bank Transfer", "Cash", "Credit Card"]);
}
