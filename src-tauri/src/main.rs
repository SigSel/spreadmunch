use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Serialize)]
struct SpreadsheetData {
    file_name: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Serialize)]
struct LoadingProgress {
    loaded: usize,
    total: usize,
}

#[tauri::command]
async fn open_file(app: tauri::AppHandle) -> Result<SpreadsheetData, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Spreadsheets", &["csv", "xlsx"])
        .pick_file()
        .await
        .ok_or_else(|| "No file selected".to_string())?;

    let path = file.path().to_path_buf();
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    match ext.as_str() {
        "csv" => parse_csv(&path, file_name, &app),
        "xlsx" => parse_xlsx(&path, file_name, &app),
        _ => Err(format!("Unsupported file type: {ext}")),
    }
}

fn parse_csv(
    path: &std::path::Path,
    file_name: String,
    app: &tauri::AppHandle,
) -> Result<SpreadsheetData, String> {
    // Quick line count by scanning for newlines
    let content = std::fs::read(path).map_err(|e| format!("Failed to read file: {e}"))?;
    let total = content.iter().filter(|&&b| b == b'\n').count().saturating_sub(1); // subtract header

    let _ = app.emit("loading-progress", LoadingProgress { loaded: 0, total });

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
            let _ = app.emit("loading-progress", LoadingProgress { loaded, total });
        }
    }

    let _ = app.emit(
        "loading-progress",
        LoadingProgress {
            loaded,
            total: loaded,
        },
    );

    Ok(SpreadsheetData {
        file_name,
        headers,
        rows,
    })
}

fn parse_xlsx(
    path: &std::path::Path,
    file_name: String,
    app: &tauri::AppHandle,
) -> Result<SpreadsheetData, String> {
    use calamine::{open_workbook_auto, Reader};

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

    let total = range.height().saturating_sub(1); // subtract header row
    let _ = app.emit("loading-progress", LoadingProgress { loaded: 0, total });

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
            let _ = app.emit("loading-progress", LoadingProgress { loaded, total });
        }
    }

    let _ = app.emit(
        "loading-progress",
        LoadingProgress {
            loaded,
            total: loaded,
        },
    );

    Ok(SpreadsheetData {
        file_name,
        headers,
        rows,
    })
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
