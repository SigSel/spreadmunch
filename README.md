# Spreadmunch

A lightweight desktop app for exploring tabular data. Open CSV or XLSX files, view them in a fast table viewer, and use the cross-column filter to find patterns across your data.

Built with [Tauri 2](https://tauri.app/) and a Rust/WASM frontend.

## Features

- **Import CSV and XLSX files** — open spreadsheets via a native file dialog
- **Fast table viewer** — sticky headers, alternating row colors, scrollable
- **Cross-column filter** — answer questions like "which customers paid with both Credit Card and Bank Transfer?" or "which customers paid by Cash but never by Bank Transfer?"
  - Pick a key column (e.g. Customer) and a values column (e.g. Payment Type)
  - **Include values** with AND/OR matching
  - **Exclude values** to filter out keys that have certain values
  - Live summary in the top bar describing the active filter
  - Table updates in real time as you adjust the filter

## Screenshot

*Coming soon*

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`
- [Tauri CLI](https://tauri.app/start/) — `cargo install tauri-cli`
- System dependencies for Tauri 2 (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

### Run in Development

```sh
cargo tauri dev
```

This starts the Trunk dev server on port 8080 and opens the Tauri window.

### Build for Production

```sh
cargo tauri build
```

The output binary/installer will be in `src-tauri/target/release/`.

## Tech Stack

| Component | Technology |
|---|---|
| Native shell | Tauri 2 |
| Frontend | Rust compiled to WASM via Trunk |
| DOM | dominator (reactive, zero-cost) |
| Styling | dwind (Tailwind-like utility classes) |
| Reactivity | futures-signals |
| CSV parsing | csv crate |
| XLSX parsing | calamine crate |

## License

This project is licensed under the GNU General Public License v3.0 or later — see the [LICENSE](LICENSE) file for details.
