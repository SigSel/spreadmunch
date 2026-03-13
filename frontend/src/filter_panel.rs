use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::SignalVecExt;
use web_sys::{HtmlInputElement, HtmlSelectElement};

use crate::app::App;
use crate::cross_filter::MatchMode;

pub fn render_filter_panel(app: Arc<App>) -> Dom {
    let headers: Vec<String> = match app.data.lock_ref().as_ref() {
        Some(d) => d.headers.clone(),
        None => return html!("div"),
    };


    html!("div", {
        .style("width", "35%")
        .style("flex-shrink", "0")
        .style("border-left", "1px solid #374151")
        .style("overflow-y", "auto")
        .dwclass!("bg-gray-800 flex flex-col")

        // Header
        .child(html!("div", {
            .dwclass!("flex items-center px-4 py-3")
            .style("border-bottom", "1px solid #374151")
            .style("justify-content", "space-between")
            .child(html!("span", {
                .dwclass!("text-sm font-semibold text-white")
                .text("Cross-Column Filter")
            }))
            .child(html!("div", {
                .dwclass!("flex")
                .style("gap", "8px")
                .child(html!("button", {
                    .dwclass!("px-3 py-1 bg-red-700 text-white text-xs font-medium rounded")
                    .style("cursor", "pointer")
                    .text("Clear Filter")
                    .event(clone!(app => move |_: events::Click| {
                        app.cross_filter.clear_filter();
                    }))
                }))
                .child(html!("button", {
                    .dwclass!("px-3 py-1 bg-gray-600 text-white text-xs font-medium rounded")
                    .style("cursor", "pointer")
                    .text("Close")
                    .event(clone!(app => move |_: events::Click| {
                        app.cross_filter.panel_open.set(false);
                    }))
                }))
            }))
        }))

        // Key Column dropdown
        .child(html!("div", {
            .dwclass!("px-4 py-3")
            .style("border-bottom", "1px solid #374151")
            .child(html!("label", {
                .dwclass!("text-xs text-gray-400")
                .text("Key Column:")
            }))
            .child(html!("select" => HtmlSelectElement, {
                .dwclass!("w-full mt-1 px-2 py-1 text-sm text-white rounded")
                .style("background-color", "#374151")
                .style("border", "1px solid #4b5563")
                .child(html!("option", {
                    .attr("value", "")
                    .text("-- Select --")
                }))
                .children(headers.iter().enumerate().map(|(i, h)| {
                    html!("option", {
                        .attr("value", &i.to_string())
                        .text(h)
                    })
                }).collect::<Vec<_>>())
                .prop_signal("value", app.cross_filter.key_column.signal().map(|col| {
                    col.map_or(String::new(), |i| i.to_string())
                }))
                .with_node!(element => {
                    .event(clone!(app => move |_: events::Change| {
                        let val = element.value();
                        let col = if val.is_empty() { None } else { val.parse().ok() };
                        app.cross_filter.key_column.set(col);
                        if let Some(data) = app.data.lock_ref().as_ref() {
                            app.cross_filter.compute_results(data);
                        }
                    }))
                })
            }))
        }))

        // Values Column dropdown
        .child(html!("div", {
            .dwclass!("px-4 py-3")
            .style("border-bottom", "1px solid #374151")
            .child(html!("label", {
                .dwclass!("text-xs text-gray-400")
                .text("Values Column:")
            }))
            .child(html!("select" => HtmlSelectElement, {
                .dwclass!("w-full mt-1 px-2 py-1 text-sm text-white rounded")
                .style("background-color", "#374151")
                .style("border", "1px solid #4b5563")
                .child(html!("option", {
                    .attr("value", "")
                    .text("-- Select --")
                }))
                .children(headers.iter().enumerate().map(|(i, h)| {
                    html!("option", {
                        .attr("value", &i.to_string())
                        .text(h)
                    })
                }).collect::<Vec<_>>())
                .prop_signal("value", app.cross_filter.values_column.signal().map(|col| {
                    col.map_or(String::new(), |i| i.to_string())
                }))
                .with_node!(element => {
                    .event(clone!(app => move |_: events::Change| {
                        let val = element.value();
                        let col = if val.is_empty() { None } else { val.parse().ok() };
                        app.cross_filter.values_column.set(col);
                        if let Some(data) = app.data.lock_ref().as_ref() {
                            app.cross_filter.update_unique_values(data);
                        }
                    }))
                })
            }))
        }))

        // Match Mode toggle
        .child(html!("div", {
            .dwclass!("px-4 py-3")
            .style("border-bottom", "1px solid #374151")
            .child(html!("label", {
                .dwclass!("text-xs text-gray-400")
                .text("Match Mode:")
            }))
            .child(html!("div", {
                .dwclass!("flex mt-1")
                .style("gap", "8px")
                .child(html!("button", {
                    .dwclass!("px-3 py-1 text-sm font-medium rounded text-white")
                    .style("cursor", "pointer")
                    .style_signal("background-color", app.cross_filter.match_mode.signal().map(|m| {
                        if m == MatchMode::And { "#2563eb" } else { "#374151" }
                    }))
                    .text("AND")
                    .event(clone!(app => move |_: events::Click| {
                        app.cross_filter.match_mode.set(MatchMode::And);
                        if let Some(data) = app.data.lock_ref().as_ref() {
                            app.cross_filter.compute_results(data);
                        }
                    }))
                }))
                .child(html!("button", {
                    .dwclass!("px-3 py-1 text-sm font-medium rounded text-white")
                    .style("cursor", "pointer")
                    .style_signal("background-color", app.cross_filter.match_mode.signal().map(|m| {
                        if m == MatchMode::Or { "#2563eb" } else { "#374151" }
                    }))
                    .text("OR")
                    .event(clone!(app => move |_: events::Click| {
                        app.cross_filter.match_mode.set(MatchMode::Or);
                        if let Some(data) = app.data.lock_ref().as_ref() {
                            app.cross_filter.compute_results(data);
                        }
                    }))
                }))
            }))
        }))

        // Include Values section
        .child(html!("div", {
            .dwclass!("px-4 py-3")
            .style("border-bottom", "1px solid #374151")
            .child(html!("div", {
                .dwclass!("flex items-center mb-2")
                .style("justify-content", "space-between")
                .child(html!("label", {
                    .dwclass!("text-xs text-gray-400")
                    .text("Include Values:")
                }))
                .child(html!("div", {
                    .dwclass!("flex")
                    .style("gap", "8px")
                    .child(html!("button", {
                        .dwclass!("px-3 py-1 bg-blue-600 text-white text-xs font-medium rounded")
                        .style("cursor", "pointer")
                        .text("Select All")
                        .event(clone!(app => move |_: events::Click| {
                            for v in app.cross_filter.selected_values.lock_ref().iter() {
                                if !v.excluded.get() {
                                    v.included.set(true);
                                }
                            }
                            if let Some(data) = app.data.lock_ref().as_ref() {
                                app.cross_filter.compute_results(data);
                            }
                        }))
                    }))
                    .child(html!("button", {
                        .dwclass!("px-3 py-1 bg-gray-600 text-white text-xs font-medium rounded")
                        .style("cursor", "pointer")
                        .text("Clear All")
                        .event(clone!(app => move |_: events::Click| {
                            for v in app.cross_filter.selected_values.lock_ref().iter() {
                                v.included.set(false);
                            }
                            if let Some(data) = app.data.lock_ref().as_ref() {
                                app.cross_filter.compute_results(data);
                            }
                        }))
                    }))
                }))
            }))
            .child(html!("div", {
                .style("max-height", "200px")
                .style("overflow-y", "auto")
                .children_signal_vec(
                    app.cross_filter.selected_values.signal_vec_cloned()
                        .map(clone!(app => move |vs| {
                            let label = vs.value.clone();
                            html!("label", {
                                .dwclass!("flex items-center py-1 text-sm")
                                .style("gap", "8px")
                                .style("cursor", "pointer")
                                // Grey out if this value is excluded
                                .style_signal("opacity", vs.excluded.signal().map(|ex| {
                                    if ex { "0.35" } else { "1" }
                                }))
                                .child(html!("input" => HtmlInputElement, {
                                    .attr("type", "checkbox")
                                    .prop_signal("checked", vs.included.signal())
                                    .prop_signal("disabled", vs.excluded.signal())
                                    .with_node!(element => {
                                        .event(clone!(app, vs => move |_: events::Change| {
                                            vs.included.set(element.checked());
                                            if let Some(data) = app.data.lock_ref().as_ref() {
                                                app.cross_filter.compute_results(data);
                                            }
                                        }))
                                    })
                                }))
                                .child(html!("span", {
                                    .dwclass!("text-gray-300")
                                    .text(&label)
                                }))
                            })
                        }))
                )
            }))
        }))

        // Exclude Values section
        .child(html!("div", {
            .dwclass!("px-4 py-3")
            .style("border-bottom", "1px solid #374151")
            .child(html!("div", {
                .dwclass!("flex items-center mb-2")
                .style("justify-content", "space-between")
                .child(html!("label", {
                    .dwclass!("text-xs text-gray-400")
                    .text("Exclude Values (NOT):")
                }))
                .child(html!("button", {
                    .dwclass!("px-3 py-1 bg-gray-600 text-white text-xs font-medium rounded")
                    .style("cursor", "pointer")
                    .text("Clear All")
                    .event(clone!(app => move |_: events::Click| {
                        for v in app.cross_filter.selected_values.lock_ref().iter() {
                            v.excluded.set(false);
                        }
                        if let Some(data) = app.data.lock_ref().as_ref() {
                            app.cross_filter.compute_results(data);
                        }
                    }))
                }))
            }))
            .child(html!("div", {
                .style("max-height", "200px")
                .style("overflow-y", "auto")
                .children_signal_vec(
                    app.cross_filter.selected_values.signal_vec_cloned()
                        .map(clone!(app => move |vs| {
                            let label = vs.value.clone();
                            html!("label", {
                                .dwclass!("flex items-center py-1 text-sm")
                                .style("gap", "8px")
                                .style("cursor", "pointer")
                                // Grey out if this value is included
                                .style_signal("opacity", vs.included.signal().map(|inc| {
                                    if inc { "0.35" } else { "1" }
                                }))
                                .child(html!("input" => HtmlInputElement, {
                                    .attr("type", "checkbox")
                                    .prop_signal("checked", vs.excluded.signal())
                                    .prop_signal("disabled", vs.included.signal())
                                    .with_node!(element => {
                                        .event(clone!(app, vs => move |_: events::Change| {
                                            vs.excluded.set(element.checked());
                                            if let Some(data) = app.data.lock_ref().as_ref() {
                                                app.cross_filter.compute_results(data);
                                            }
                                        }))
                                    })
                                }))
                                .child(html!("span", {
                                    .dwclass!("text-gray-300")
                                    .text(&label)
                                }))
                            })
                        }))
                )
            }))
        }))

        // Results section
        .child(html!("div", {
            .dwclass!("px-4 py-3 flex-1")
            .child(html!("div", {
                .dwclass!("text-xs text-gray-400 mb-2")
                .text_signal(
                    app.cross_filter.results.signal_vec_cloned()
                        .len()
                        .map(|len| format!("Matching Keys ({len}):"))
                )
            }))
            .child(html!("div", {
                .style("max-height", "300px")
                .style("overflow-y", "auto")
                .children_signal_vec(
                    app.cross_filter.results.signal_vec_cloned()
                        .map(|key| {
                            html!("div", {
                                .dwclass!("text-sm text-gray-300 py-1")
                                .text(&format!("\u{2022} {key}"))
                            })
                        })
                )
            }))
        }))
    })
}
