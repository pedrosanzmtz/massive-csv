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

## Usage

### CLI

```bash
# File info — row count, columns, size, delimiter
massive-csv info data.csv

# View rows as a formatted table
massive-csv view data.csv                    # first 20 rows
massive-csv view data.csv --rows 100-200     # specific range
massive-csv view data.csv --rows 5000        # single row

# Search across all columns
massive-csv search data.csv "error"

# Search with filters
massive-csv search data.csv "error" --column status   # specific column
massive-csv search data.csv "alice" -i                 # case-insensitive
massive-csv search data.csv "error" -n 50              # limit results

# Edit a specific cell
massive-csv edit data.csv --row 15023 --col status --value "fixed"
massive-csv edit data.csv --row 0 --col 3 --value "new"   # column by index
```

### VSCode Extension (Planned)

Open any `.csv` file — Massive CSV takes over with a fast, scrollable table view. Search, filter, edit cells, and save.

## Development Status

**Phase:** CLI Tool (Phase 2 complete)

- [x] Project setup
- [x] Memory-mapped CSV reading
- [x] Line position indexing (O(1) row access)
- [x] Parallel text search (rayon)
- [x] Edit tracking & atomic save
- [x] Auto delimiter detection (comma, tab, semicolon, pipe)
- [x] CLI tool (`info`, `view`, `search`, `edit`)
- [ ] VSCode extension

## Building from Source

```bash
# Clone
git clone https://github.com/pedrosanzmtz/massive-csv.git
cd massive-csv

# Build everything
cargo build --release

# Or just the CLI
cargo build -p massive-csv-cli --release

# Run tests
cargo test
```

The binary is at `target/release/massive-csv`.

## Who Is This For?

- Data engineers working with large exports
- Backend developers dealing with large CSV dumps
- DevOps engineers analyzing logs and metrics
- Anyone frustrated with "file too large" errors

## License

MIT

---

Built by [@pedrosanzmtz](https://github.com/pedrosanzmtz) — practical tools for engineers working with massive datasets.
