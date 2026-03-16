use serde::Serialize;
use tauri::Emitter;

pub use spreadmunch_core::types::SpreadsheetData;

#[derive(Clone, Serialize)]
struct LoadingProgress {
    loaded: usize,
    total: usize,
}

/// Parse a CSV file into SpreadsheetData.
/// Optionally accepts a progress callback called every 5000 rows with (loaded, total).
pub fn parse_csv_from_path(
    path: &std::path::Path,
    progress_cb: Option<&dyn Fn(usize, usize)>,
) -> Result<SpreadsheetData, String> {
    let content = std::fs::read(path).map_err(|e| format!("Failed to read file: {e}"))?;
    let total = content.iter().filter(|&&b| b == b'\n').count().saturating_sub(1);

    if let Some(cb) = progress_cb {
        cb(0, total);
    }

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut reader =
        csv::Reader::from_path(path).map_err(|e| format!("Failed to read CSV: {e}"))?;

    let headers = reader
        .headers()
        .map_err(|e| format!("Failed to read headers: {e}"))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut rows = Vec::with_capacity(total);
    let mut loaded = 0;
    for result in reader.records() {
        let record = result.map_err(|e| format!("Failed to read CSV records: {e}"))?;
        rows.push(record.iter().map(|f| f.to_string()).collect());
        loaded += 1;
        if loaded % 5000 == 0 {
            if let Some(cb) = progress_cb {
                cb(loaded, total);
            }
        }
    }

    if let Some(cb) = progress_cb {
        cb(loaded, loaded);
    }

    Ok(SpreadsheetData {
        file_name,
        headers,
        rows,
    })
}

/// Parse an XLSX file into SpreadsheetData.
/// Optionally accepts a progress callback called every 5000 rows with (loaded, total).
pub fn parse_xlsx_from_path(
    path: &std::path::Path,
    progress_cb: Option<&dyn Fn(usize, usize)>,
) -> Result<SpreadsheetData, String> {
    use calamine::{open_workbook_auto, Reader};

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut workbook =
        open_workbook_auto(path).map_err(|e| format!("Failed to open workbook: {e}"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .ok_or_else(|| "No sheets found".to_string())?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Failed to read sheet: {e}"))?;

    let total = range.height().saturating_sub(1);
    if let Some(cb) = progress_cb {
        cb(0, total);
    }

    let mut row_iter = range.rows();

    let headers = row_iter
        .next()
        .map(|row| row.iter().map(|cell| data_to_string(cell)).collect())
        .unwrap_or_default();

    let mut rows = Vec::with_capacity(total);
    let mut loaded = 0;
    for row in row_iter {
        rows.push(row.iter().map(|cell| data_to_string(cell)).collect());
        loaded += 1;
        if loaded % 5000 == 0 {
            if let Some(cb) = progress_cb {
                cb(loaded, total);
            }
        }
    }

    if let Some(cb) = progress_cb {
        cb(loaded, loaded);
    }

    Ok(SpreadsheetData {
        file_name,
        headers,
        rows,
    })
}

/// Parse a file by path, detecting format from extension.
pub fn parse_file_from_path(
    path: &std::path::Path,
    progress_cb: Option<&dyn Fn(usize, usize)>,
) -> Result<SpreadsheetData, String> {
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    match ext.as_str() {
        "csv" => parse_csv_from_path(path, progress_cb),
        "xlsx" => parse_xlsx_from_path(path, progress_cb),
        _ => Err(format!("Unsupported file type: {ext}")),
    }
}

fn data_to_string(data: &calamine::Data) -> String {
    use calamine::Data;
    match data {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::Error(e) => format!("#ERR:{e:?}"),
        _ => String::new(),
    }
}

// --- Tauri commands ---

#[tauri::command]
async fn open_file(app: tauri::AppHandle) -> Result<SpreadsheetData, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Spreadsheets", &["csv", "xlsx"])
        .pick_file()
        .await
        .ok_or_else(|| "No file selected".to_string())?;

    let path = file.path().to_path_buf();

    let progress_app = app.clone();
    let progress_cb = move |loaded: usize, total: usize| {
        let _ = progress_app.emit(
            "loading-progress",
            LoadingProgress { loaded, total },
        );
    };

    parse_file_from_path(&path, Some(&progress_cb))
}

#[cfg(debug_assertions)]
#[tauri::command]
async fn load_file_from_path(
    path: String,
    app: tauri::AppHandle,
) -> Result<SpreadsheetData, String> {
    let path = std::path::PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let progress_app = app.clone();
    let progress_cb = move |loaded: usize, total: usize| {
        let _ = progress_app.emit(
            "loading-progress",
            LoadingProgress { loaded, total },
        );
    };

    parse_file_from_path(&path, Some(&progress_cb))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_file,
            #[cfg(debug_assertions)]
            load_file_from_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
