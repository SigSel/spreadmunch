use std::sync::Arc;

use dominator::{clone, events, html, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::{Mutable, SignalExt};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

use crate::table::render_table;

#[derive(Deserialize, Clone, Debug)]
pub struct SpreadsheetData {
    pub file_name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct App {
    data: Mutable<Option<SpreadsheetData>>,
    error: Mutable<Option<String>>,
    loading: Mutable<bool>,
}

impl App {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            data: Mutable::new(None),
            error: Mutable::new(None),
            loading: Mutable::new(false),
        })
    }

    async fn open_file(app: Arc<Self>) {
        app.loading.set(true);
        app.error.set(None);

        match tauri_wasm::invoke("open_file").await {
            Ok(js_val) => {
                match serde_wasm_bindgen::from_value::<SpreadsheetData>(js_val) {
                    Ok(data) => app.data.set(Some(data)),
                    Err(e) => app.error.set(Some(format!("Deserialization error: {e}"))),
                }
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                if !msg.contains("No file selected") {
                    app.error.set(Some(msg));
                }
            }
        }

        app.loading.set(false);
    }

    pub fn render(app: Arc<Self>) -> Dom {
        html!("div", {
            .dwclass!("w-full h-screen flex flex-col bg-gray-900")
            // Top bar
            .child(html!("div", {
                .dwclass!("flex items-center px-4 py-3")
                .style("border-bottom", "1px solid #374151")
                .style("gap", "16px")
                .child(html!("h1", {
                    .dwclass!("text-xl font-bold text-white")
                    .text("Spreadmunch")
                }))
                .child(html!("button", {
                    .dwclass!("px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded")
                    .style("cursor", "pointer")
                    .text("Open File")
                    .event(clone!(app => move |_: events::Click| {
                        let app = app.clone();
                        spawn_local(async move {
                            App::open_file(app).await;
                        });
                    }))
                }))
                // File name display
                .child_signal(app.data.signal_cloned().map(|data| {
                    data.map(|d| {
                        html!("span", {
                            .dwclass!("text-sm text-gray-400")
                            .text(&d.file_name)
                        })
                    })
                }))
            }))
            // Error banner
            .child_signal(app.error.signal_cloned().map(|error| {
                error.map(|msg| {
                    html!("div", {
                        .dwclass!("px-4 py-2 text-sm text-red-400")
                        .style("background-color", "#7f1d1d")
                        .text(&msg)
                    })
                })
            }))
            // Content area
            .child_signal(app.data.signal_cloned().map(|data| {
                Some(match data {
                    Some(d) => render_table(&d),
                    None => html!("div", {
                        .dwclass!("flex-1 flex items-center justify-center text-gray-500 text-lg")
                        .text("Open a CSV or XLSX file to get started")
                    }),
                })
            }))
        })
    }
}
