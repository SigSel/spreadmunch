use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetData {
    pub file_name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
