use std::sync::Arc;

use dominator::{clone, events, html, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::{Mutable, SignalExt};

pub struct Settings {
    pub zoom: Mutable<u32>,
    pub open: Mutable<bool>,
}

const ZOOM_LEVELS: &[u32] = &[75, 100, 125, 150, 175, 200];

impl Settings {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            zoom: Mutable::new(100),
            open: Mutable::new(false),
        })
    }
}

pub fn render_settings_modal(settings: Arc<Settings>) -> Dom {
    html!("div", {
        // Backdrop
        .style("position", "fixed")
        .style("inset", "0")
        .style("z-index", "100")
        .style("display", "flex")
        .style("align-items", "center")
        .style("justify-content", "center")
        .style("background-color", "rgba(0, 0, 0, 0.5)")
        .event(clone!(settings => move |_: events::Click| {
            settings.open.set(false);
        }))
        // Modal
        .child(html!("div", {
            .dwclass!("bg-gray-800 rounded-lg")
            .style("padding", "24px")
            .style("min-width", "320px")
            .style("border", "1px solid #374151")
            // Stop clicks inside modal from closing it
            .event(|e: events::Click| {
                e.stop_propagation();
            })
            // Title
            .child(html!("div", {
                .dwclass!("flex items-center mb-6")
                .style("justify-content", "space-between")
                .child(html!("h2", {
                    .dwclass!("text-lg font-bold text-white")
                    .text("Settings")
                }))
                .child(html!("button", {
                    .dwclass!("px-3 py-1 bg-gray-600 text-white text-xs font-medium rounded")
                    .style("cursor", "pointer")
                    .text("Close")
                    .event(clone!(settings => move |_: events::Click| {
                        settings.open.set(false);
                    }))
                }))
            }))
            // Zoom setting
            .child(html!("div", {
                .child(html!("label", {
                    .dwclass!("text-sm text-gray-400")
                    .text("Zoom Level:")
                }))
                .child(html!("div", {
                    .dwclass!("flex mt-2")
                    .style("gap", "8px")
                    .style("flex-wrap", "wrap")
                    .children(ZOOM_LEVELS.iter().map(|&level| {
                        let settings = settings.clone();
                        html!("button", {
                            .dwclass!("px-3 py-2 text-sm font-medium rounded text-white")
                            .style("cursor", "pointer")
                            .style("min-width", "56px")
                            .style_signal("background-color", settings.zoom.signal().map(move |z| {
                                if z == level { "#2563eb" } else { "#374151" }
                            }))
                            .text(&format!("{level}%"))
                            .event(move |_: events::Click| {
                                settings.zoom.set(level);
                            })
                        })
                    }).collect::<Vec<_>>())
                }))
            }))
        }))
    })
}
