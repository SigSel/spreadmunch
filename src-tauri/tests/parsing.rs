use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use spreadmunch_tauri::{parse_csv_from_path, parse_xlsx_from_path, parse_file_from_path};

fn test_data_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test_data")
        .join(name)
}

// --- CSV parsing tests (payments.csv) ---

#[test]
fn test_csv_headers() {
    let data = parse_csv_from_path(&test_data_path("payments.csv"), None).unwrap();
    assert_eq!(data.headers, vec!["Customer", "Payment Type", "Amount", "Date"]);
}

#[test]
fn test_csv_row_count() {
    let data = parse_csv_from_path(&test_data_path("payments.csv"), None).unwrap();
    assert_eq!(data.rows.len(), 15);
}

#[test]
fn test_csv_file_name() {
    let data = parse_csv_from_path(&test_data_path("payments.csv"), None).unwrap();
    assert_eq!(data.file_name, "payments.csv");
}

#[test]
fn test_csv_first_row() {
    let data = parse_csv_from_path(&test_data_path("payments.csv"), None).unwrap();
    assert_eq!(
        data.rows[0],
        vec!["Alice", "Credit Card", "150.00", "2024-01-15"]
    );
}

#[test]
fn test_csv_last_row() {
    let data = parse_csv_from_path(&test_data_path("payments.csv"), None).unwrap();
    assert_eq!(
        data.rows[14],
        vec!["Frank", "Cash", "75.00", "2024-02-18"]
    );
}

#[test]
fn test_csv_all_customers() {
    let data = parse_csv_from_path(&test_data_path("payments.csv"), None).unwrap();
    let mut customers: Vec<String> = data.rows.iter()
        .map(|r| r[0].clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    customers.sort();
    assert_eq!(customers, vec!["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"]);
}

// --- XLSX parsing tests (company_data.xlsx) ---

#[test]
fn test_xlsx_parses_successfully() {
    let data = parse_xlsx_from_path(&test_data_path("company_data.xlsx"), None).unwrap();
    assert!(!data.headers.is_empty(), "XLSX should have headers");
    assert!(!data.rows.is_empty(), "XLSX should have rows");
}

#[test]
fn test_xlsx_file_name() {
    let data = parse_xlsx_from_path(&test_data_path("company_data.xlsx"), None).unwrap();
    assert_eq!(data.file_name, "company_data.xlsx");
}

// --- Large CSV tests (large_sales.csv) ---

#[test]
fn test_large_csv_row_count() {
    let data = parse_csv_from_path(&test_data_path("large_sales.csv"), None).unwrap();
    assert_eq!(data.rows.len(), 100_000);
}

#[test]
fn test_large_csv_headers() {
    let data = parse_csv_from_path(&test_data_path("large_sales.csv"), None).unwrap();
    assert_eq!(
        data.headers,
        vec![
            "Transaction ID", "Customer", "Payment Type", "Amount",
            "Currency", "Category", "Date", "Status", "Notes"
        ]
    );
}

// --- Error handling tests ---

#[test]
fn test_nonexistent_file() {
    let result = parse_csv_from_path(Path::new("/nonexistent/missing.csv"), None);
    assert!(result.is_err());
}

#[test]
fn test_unsupported_extension() {
    let result = parse_file_from_path(Path::new("/tmp/test.txt"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported file type"));
}

// --- Progress callback tests ---

#[test]
fn test_progress_callback_called() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = call_count.clone();

    let last_loaded = Arc::new(AtomicUsize::new(0));
    let loaded_clone = last_loaded.clone();

    let cb = move |loaded: usize, _total: usize| {
        count_clone.fetch_add(1, Ordering::SeqCst);
        loaded_clone.store(loaded, Ordering::SeqCst);
    };

    let data = parse_csv_from_path(&test_data_path("large_sales.csv"), Some(&cb)).unwrap();

    // Callback should be called: once at start (0), every 5000 rows (20 times), once at end
    assert!(call_count.load(Ordering::SeqCst) >= 3, "Progress callback should be called multiple times");
    // Final callback should report all rows loaded
    assert_eq!(last_loaded.load(Ordering::SeqCst), data.rows.len());
}
