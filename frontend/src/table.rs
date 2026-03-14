use std::collections::HashSet;

use crate::app::SpreadsheetData;
use dominator::{clone, events, html, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::Mutable;

const ROWS_PER_PAGE: usize = 500;

pub fn render_table(
    data: &SpreadsheetData,
    matching_keys: Option<&HashSet<String>>,
    key_col: Option<usize>,
    page: usize,
    page_signal: Mutable<usize>,
) -> Dom {
    let headers = data.headers.clone();
    let rows: Vec<&Vec<String>> = match (matching_keys, key_col) {
        (Some(keys), Some(col)) => data
            .rows
            .iter()
            .filter(|row| row.get(col).map_or(false, |k| keys.contains(k)))
            .collect(),
        _ => data.rows.iter().collect(),
    };

    let total_rows = rows.len();
    let total_pages = (total_rows + ROWS_PER_PAGE - 1).max(1) / ROWS_PER_PAGE.max(1);
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * ROWS_PER_PAGE;
    let end = (start + ROWS_PER_PAGE).min(total_rows);
    let visible_rows = &rows[start..end];

    html!("div", {
        .dwclass!("flex-1 flex flex-col")
        .child(html!("div", {
            .style("flex", "1")
            .style("overflow", "auto")
            .child(html!("table", {
                .style("border-collapse", "collapse")
                .style("width", "100%")
                .child(html!("thead", {
                    .child(html!("tr", {
                        .children(headers.iter().map(|h| {
                            let h = h.clone();
                            html!("th", {
                                .style("position", "sticky")
                                .style("top", "0")
                                .style("z-index", "10")
                                .style("background-color", "#1f2937")
                                .style("padding", "8px 12px")
                                .style("text-align", "left")
                                .style("border-bottom", "2px solid #4b5563")
                                .style("white-space", "nowrap")
                                .dwclass!("text-sm font-semibold text-gray-200")
                                .text(&h)
                            })
                        }).collect::<Vec<_>>())
                    }))
                }))
                .child(html!("tbody", {
                    .children(visible_rows.iter().enumerate().map(|(i, row)| {
                        let bg = if (start + i) % 2 == 0 { "#111827" } else { "#1f2937" };
                        html!("tr", {
                            .style("background-color", bg)
                            .children(row.iter().map(|cell| {
                                let cell = cell.clone();
                                html!("td", {
                                    .style("padding", "6px 12px")
                                    .style("border-bottom", "1px solid #374151")
                                    .dwclass!("text-sm text-gray-300")
                                    .text(&cell)
                                })
                            }).collect::<Vec<_>>())
                        })
                    }).collect::<Vec<_>>())
                }))
            }))
        }))
        // Pagination bar
        .apply_if(total_pages > 1, |dom| {
            let info = format!(
                "Rows {} - {} of {}",
                if total_rows > 0 { start + 1 } else { 0 },
                end,
                total_rows
            );
            dom.child(html!("div", {
                .dwclass!("flex items-center px-4 py-2 text-sm text-gray-400")
                .style("border-top", "1px solid #374151")
                .style("gap", "12px")
                .style("background-color", "#111827")
                .style("flex-shrink", "0")
                .child(html!("button", {
                    .dwclass!("px-3 py-1 text-sm rounded")
                    .style("cursor", if page > 0 { "pointer" } else { "default" })
                    .style("background-color", if page > 0 { "#374151" } else { "#1f2937" })
                    .style("color", if page > 0 { "#d1d5db" } else { "#6b7280" })
                    .text("Prev")
                    .event(clone!(page_signal => move |_: events::Click| {
                        let p = page_signal.get();
                        if p > 0 {
                            page_signal.set(p - 1);
                        }
                    }))
                }))
                .child(html!("span", {
                    .text(&info)
                }))
                .child(html!("button", {
                    .dwclass!("px-3 py-1 text-sm rounded")
                    .style("cursor", if page + 1 < total_pages { "pointer" } else { "default" })
                    .style("background-color", if page + 1 < total_pages { "#374151" } else { "#1f2937" })
                    .style("color", if page + 1 < total_pages { "#d1d5db" } else { "#6b7280" })
                    .text("Next")
                    .event(clone!(page_signal => move |_: events::Click| {
                        let p = page_signal.get();
                        if p + 1 < total_pages {
                            page_signal.set(p + 1);
                        }
                    }))
                }))
            }))
        })
    })
}
