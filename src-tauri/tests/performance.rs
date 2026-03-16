use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use spreadmunch_tauri::parse_csv_from_path;
use spreadmunch_core::cross_filter::{compute_cross_filter, get_unique_values, CrossFilterInput, MatchMode};

fn test_data_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test_data")
        .join(name)
}

#[test]
#[ignore]
fn test_large_csv_parse_time() {
    let start = Instant::now();
    let data = parse_csv_from_path(&test_data_path("large_sales.csv"), None).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(data.rows.len(), 100_000);
    assert!(
        elapsed.as_secs() < 2,
        "Parsing 100K rows took {:?}, expected < 2s",
        elapsed
    );
    eprintln!("Large CSV parse time: {:?}", elapsed);
}

#[test]
#[ignore]
fn test_large_cross_filter_time() {
    let data = parse_csv_from_path(&test_data_path("large_sales.csv"), None).unwrap();

    // Find the Payment Type column index
    let payment_col = data.headers.iter().position(|h| h == "Payment Type").unwrap();
    let customer_col = data.headers.iter().position(|h| h == "Customer").unwrap();

    let start = Instant::now();
    let result = compute_cross_filter(&data.rows, &CrossFilterInput {
        key_col: customer_col,
        values_col: payment_col,
        included: HashSet::from(["Credit Card".to_string(), "Bank Transfer".to_string()]),
        excluded: HashSet::from(["Cash".to_string()]),
        mode: MatchMode::And,
    });
    let elapsed = start.elapsed();

    assert!(!result.is_empty(), "Should have matching results");
    assert!(
        elapsed.as_millis() < 500,
        "Cross-filter on 100K rows took {:?}, expected < 500ms",
        elapsed
    );
    eprintln!("Large cross-filter time: {:?} ({} results)", elapsed, result.len());
}

#[test]
#[ignore]
fn test_large_unique_values_time() {
    let data = parse_csv_from_path(&test_data_path("large_sales.csv"), None).unwrap();
    let payment_col = data.headers.iter().position(|h| h == "Payment Type").unwrap();

    let start = Instant::now();
    let values = get_unique_values(&data.rows, payment_col);
    let elapsed = start.elapsed();

    assert!(!values.is_empty(), "Should have unique values");
    assert!(
        elapsed.as_millis() < 200,
        "Unique values on 100K rows took {:?}, expected < 200ms",
        elapsed
    );
    eprintln!("Large unique values time: {:?} ({} values)", elapsed, values.len());
}
