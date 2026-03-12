use crate::app::SpreadsheetData;
use dominator::{html, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;

pub fn render_table(data: &SpreadsheetData) -> Dom {
    let headers = data.headers.clone();
    let rows = data.rows.clone();

    html!("div", {
        .dwclass!("flex-1 overflow-auto")
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
                .children(rows.iter().enumerate().map(|(i, row)| {
                    let bg = if i % 2 == 0 { "#111827" } else { "#1f2937" };
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
    })
}
