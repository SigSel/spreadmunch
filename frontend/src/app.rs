use std::sync::Arc;

use dominator::{clone, events, html, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, SignalExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::cross_filter::CrossFilter;
use crate::filter_panel::render_filter_panel;
use crate::settings::{render_settings_modal, Settings};
use crate::table::render_table;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], catch)]
    async fn listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

#[derive(Deserialize, Clone, Debug)]
pub struct SpreadsheetData {
    pub file_name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct LoadingProgress {
    loaded: usize,
    total: usize,
}

use serde::Deserialize;

pub struct App {
    pub(crate) data: Mutable<Option<SpreadsheetData>>,
    error: Mutable<Option<String>>,
    loading: Mutable<bool>,
    loading_progress: Mutable<Option<(usize, usize)>>,
    pub(crate) page: Mutable<usize>,
    pub(crate) cross_filter: Arc<CrossFilter>,
    pub(crate) settings: Arc<Settings>,
}

impl App {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            data: Mutable::new(None),
            error: Mutable::new(None),
            loading: Mutable::new(false),
            loading_progress: Mutable::new(None),
            page: Mutable::new(0),
            cross_filter: CrossFilter::new(),
            settings: Settings::new(),
        })
    }

    async fn open_file(app: Arc<Self>) {
        app.loading.set(true);
        app.error.set(None);
        app.loading_progress.set(None);

        // Set up event listener for loading progress
        let app_progress = app.clone();
        let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            if let Ok(payload) = js_sys::Reflect::get(&event, &JsValue::from_str("payload")) {
                if let Ok(progress) = serde_wasm_bindgen::from_value::<LoadingProgress>(payload) {
                    app_progress
                        .loading_progress
                        .set(Some((progress.loaded, progress.total)));
                }
            }
        });

        let unlisten_fn = listen("loading-progress", closure.as_ref().unchecked_ref()).await;

        match tauri_wasm::invoke("open_file").await {
            Ok(js_val) => match serde_wasm_bindgen::from_value::<SpreadsheetData>(js_val) {
                Ok(data) => {
                    app.cross_filter.reset();
                    app.page.set(0);
                    app.data.set(Some(data));
                }
                Err(e) => app.error.set(Some(format!("Deserialization error: {e}"))),
            },
            Err(e) => {
                let msg = format!("{:?}", e);
                if !msg.contains("No file selected") {
                    app.error.set(Some(msg));
                }
            }
        }

        // Clean up the event listener
        if let Ok(unlisten_js) = unlisten_fn {
            if let Ok(f) = unlisten_js.dyn_into::<js_sys::Function>() {
                let _ = f.call0(&JsValue::NULL);
            }
        }
        drop(closure);

        app.loading_progress.set(None);
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
                    .style("flex-shrink", "0")
                    .style("white-space", "nowrap")
                    .text("Open File")
                    .event(clone!(app => move |_: events::Click| {
                        let app = app.clone();
                        spawn_local(async move {
                            App::open_file(app).await;
                        });
                    }))
                }))
                // Cross Filter toggle button (only when data loaded)
                .child_signal(app.data.signal_cloned().map(clone!(app => move |data| {
                    data.map(|_| {
                        html!("button", {
                            .dwclass!("px-4 py-2 text-white text-sm font-medium rounded")
                            .style("cursor", "pointer")
                            .style("flex-shrink", "0")
                            .style("white-space", "nowrap")
                            .style_signal("background-color",
                                app.cross_filter.panel_open.signal().map(|open| {
                                    if open { "#7c3aed" } else { "#6d28d9" }
                                })
                            )
                            .text("Cross Filter")
                            .event(clone!(app => move |_: events::Click| {
                                let open = app.cross_filter.panel_open.get();
                                app.cross_filter.panel_open.set(!open);
                            }))
                        })
                    })
                })))
                // File name display
                .child_signal(app.data.signal_cloned().map(|data| {
                    data.map(|d| {
                        html!("span", {
                            .dwclass!("text-sm text-gray-400")
                            .style("flex-shrink", "0")
                            .style("white-space", "nowrap")
                            .text(&d.file_name)
                        })
                    })
                }))
                // Spacer to push summary + cog to the right
                .child(html!("div", {
                    .style("flex", "1")
                }))
                // Filter summary
                .child_signal(app.cross_filter.summary.signal_cloned().map(|summary| {
                    summary.map(|text| {
                        html!("span", {
                            .dwclass!("text-xs text-gray-400")
                            .style("text-align", "right")
                            .style("overflow", "hidden")
                            .style("text-overflow", "ellipsis")
                            .style("white-space", "nowrap")
                            .style("max-width", "400px")
                            .attr("title", &text)
                            .text(&text)
                        })
                    })
                }))
                // Settings cog
                .child(html!("button", {
                    .style("cursor", "pointer")
                    .style("font-size", "28px")
                    .style("background", "none")
                    .style("border", "none")
                    .style("padding", "4px 8px")
                    .style("line-height", "1")
                    .dwclass!("text-gray-400")
                    .text("\u{2699}\u{fe0f}")
                    .event(clone!(app => move |_: events::Click| {
                        app.settings.open.set(true);
                    }))
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
            // Content area - flex row with table + optional filter panel
            .child(html!("div", {
                .dwclass!("flex-1 flex")
                .style("overflow", "hidden")
                // Table container
                .child(html!("div", {
                    .style("flex", "1")
                    .style("min-width", "0")
                    .style("overflow", "auto")
                    .child_signal({
                        let page_mutable = app.page.clone();
                        map_ref! {
                            let data = app.data.signal_cloned(),
                            let keys = app.cross_filter.matching_keys.signal_cloned(),
                            let key_col = app.cross_filter.key_column.signal(),
                            let loading = app.loading.signal(),
                            let progress = app.loading_progress.signal(),
                            let page = page_mutable.signal()
                            => {
                                Some(if *loading {
                                    let text = match progress {
                                        Some((loaded, total)) if *total > 0 => {
                                            format!("Loading... {loaded} / {total} rows")
                                        }
                                        _ => "Loading...".to_string(),
                                    };
                                    html!("div", {
                                        .dwclass!("flex-1 flex items-center justify-center text-gray-400 text-lg")
                                        .text(&text)
                                    })
                                } else {
                                    match data {
                                        Some(d) => render_table(d, keys.as_ref(), *key_col, *page, page_mutable.clone()),
                                        None => html!("div", {
                                            .dwclass!("flex-1 flex items-center justify-center text-gray-500 text-lg")
                                            .text("Open a CSV or XLSX file to get started")
                                        }),
                                    }
                                })
                            }
                        }
                    })
                }))
                // Filter panel
                .child_signal(
                    app.cross_filter.panel_open.signal().map(clone!(app => move |open| {
                        if open {
                            Some(render_filter_panel(app.clone()))
                        } else {
                            None
                        }
                    }))
                )
            }))
            // Settings modal
            .child_signal(app.settings.open.signal().map(clone!(app => move |open| {
                if open {
                    Some(render_settings_modal(app.settings.clone()))
                } else {
                    None
                }
            })))
        })
    }
}
