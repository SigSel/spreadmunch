use serde::Serialize;

#[derive(Debug, Serialize)]
struct SpreadsheetData {
    file_name: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[tauri::command]
async fn open_file() -> Result<SpreadsheetData, String> {
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
        "csv" => parse_csv(&path, file_name),
        "xlsx" => parse_xlsx(&path, file_name),
        _ => Err(format!("Unsupported file type: {ext}")),
    }
}

fn parse_csv(path: &std::path::Path, file_name: String) -> Result<SpreadsheetData, String> {
    let mut reader =
        csv::Reader::from_path(path).map_err(|e| format!("Failed to read CSV: {e}"))?;

    let headers = reader
        .headers()
        .map_err(|e| format!("Failed to read headers: {e}"))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let rows = reader
        .records()
        .map(|r| r.map(|record| record.iter().map(|f| f.to_string()).collect()))
        .collect::<Result<Vec<Vec<String>>, _>>()
        .map_err(|e| format!("Failed to read CSV records: {e}"))?;

    Ok(SpreadsheetData {
        file_name,
        headers,
        rows,
    })
}

fn parse_xlsx(path: &std::path::Path, file_name: String) -> Result<SpreadsheetData, String> {
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

    let mut row_iter = range.rows();

    let headers = row_iter
        .next()
        .map(|row| row.iter().map(|cell| data_to_string(cell)).collect())
        .unwrap_or_default();

    let rows = row_iter
        .map(|row| row.iter().map(|cell| data_to_string(cell)).collect())
        .collect();

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

#[tauri::command]
async fn set_zoom(window: tauri::WebviewWindow, factor: f64) -> Result<(), String> {
    window
        .set_zoom(factor)
        .map_err(|e| format!("Failed to set zoom: {e}"))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_file, set_zoom])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
