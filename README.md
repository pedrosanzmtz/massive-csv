# Massive CSV

**Because your data doesn't fit in Excel anymore.**

A high-performance Rust-based tool for viewing and editing massive CSV files (millions of rows) with both CLI and VSCode extension interfaces.

Part of the [Massive Data Tools](https://github.com/pedrosanzmtz) family.

## The Problem

You have a 2.3M row CSV file. Excel can't open it. VSCode chokes on it. Online tools time out. You just need to find one row and change a value.

**Massive CSV** handles multi-GB CSV files without breaking a sweat.

## Features

- **Memory-mapped I/O** — open multi-GB files without loading them into RAM
- **Fast search** — find rows across millions of entries in under a second
- **Targeted editing** — modify specific rows and save efficiently
- **CLI + VSCode** — use from the terminal or as a visual editor in VSCode
- **Virtual scrolling** — browse millions of rows smoothly in the UI

## Performance Targets

| Operation | Target (2.3M rows, ~500MB) |
|-----------|---------------------------|
| Open file | < 2 seconds |
| Search | < 500ms |
| Jump to row | < 50ms |
| Save edits | < 2 seconds |
| Memory usage | < 100MB |

## Architecture

```
massive-csv/
├── massive-csv-core/      # Rust library — memory-mapped reading, indexing, search, editing
├── massive-csv-cli/       # CLI tool — view, search, edit from the terminal
└── massive-csv-vscode/    # VSCode extension — visual table editor with virtual scrolling
```

## Tech Stack

- **Rust** — core engine (memmap2, csv, rayon)
- **TypeScript** — VSCode extension
- **napi-rs** — Rust ↔ Node.js bridge

## Usage (Planned)

### CLI

```bash
# View rows
massive-csv view data.csv --rows 100-200

# Search
massive-csv search data.csv "error" --column status

# File info
massive-csv info data.csv

# Edit interactively
massive-csv edit data.csv
```

### VSCode Extension

Open any `.csv` file — Massive CSV takes over with a fast, scrollable table view. Search, filter, edit cells, and save.

## Development Status

**Phase:** Core Library (Phase 1)

- [x] Project setup
- [ ] Memory-mapped CSV reading
- [ ] Line position indexing
- [ ] Fast text search
- [ ] Edit tracking & smart save
- [ ] CLI tool
- [ ] VSCode extension

## Building from Source

```bash
# Clone
git clone https://github.com/pedrosanzmtz/massive-csv.git
cd massive-csv

# Build core library
cd massive-csv-core
cargo build --release

# Build CLI
cd ../massive-csv-cli
cargo build --release
```

## Who Is This For?

- Data engineers working with large exports
- Backend developers dealing with large CSV dumps
- DevOps engineers analyzing logs and metrics
- Anyone frustrated with "file too large" errors

## License

MIT

---

Built by [@pedrosanzmtz](https://github.com/pedrosanzmtz) — practical tools for engineers working with massive datasets.
