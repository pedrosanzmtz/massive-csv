# Massive CSV - Large CSV Viewer & Editor

> A high-performance Rust-based tool for viewing and editing massive CSV files (millions of rows) with both CLI and VSCode extension interfaces.

## Project Overview

**Name:** Massive CSV  
**Brand:** Massive Data Tools  
**Tagline:** "Because your data doesn't fit in Excel anymore"  
**Author:** Pedro  
**Purpose:** View, search, and edit large CSV files (1M+ rows, multi-GB) efficiently without loading entire file into memory

**Key Use Case:**
1. Open a 2.3M row data CSV
2. Search for specific values (e.g., ID, status, metric)
3. View matching rows in context
4. Edit specific rows as needed
5. Save changes back efficiently

## Branding & Positioning

**Brand Name:** Massive Data Tools  
**First Product:** Massive CSV  
**Logo Concept:** Bold, industrial design emphasizing scale and performance  
**Color Scheme:** Deep blues and steel grays (data/engineering aesthetic)  

**Taglines:**
- Primary: "Because your data doesn't fit in Excel anymore"
- Product-specific: "Edit massive CSV files in VSCode"
- Technical: "Handle millions of rows without breaking a sweat"
- Brand: "Data engineering tools that scale"

**Brand Story:**
> "As a data engineer dealing with 2.3M row CSVs and 63GB XML files, I couldn't find tools that actually worked. So I built them. **Massive Data Tools** - practical tools for engineers working with massive datasets."

**Positioning:**
- Built by data engineers, for data engineers
- Handles what other tools can't (multi-GB files)
- Rust-powered performance, TypeScript convenience
- Open source core, sustainable through value-add features
- The only VSCode extension that can actually edit massive CSVs

**Target Users:**
- Data engineers working with large datasets
- Backend developers dealing with large exports
- DevOps engineers analyzing logs and metrics
- Analysts who outgrew Excel/Google Sheets
- Anyone frustrated with "file too large" errors

**Product Family Vision:**
```
Massive Data Tools
├─ massive-csv (this project) - "Edit massive CSV files"
├─ massive-xml (rebrand xmlshift) - "Parse GB-sized XML files"
├─ massive-json (future) - "Handle massive JSON arrays"
├─ massive-logs (future) - "Search through TB of logs"
└─ massive-parquet (future) - "Work with huge Parquet files"
```

## Competitive Landscape

| Tool | Interface | Max File Size | Editing | Virtual Scrolling | Status |
|------|-----------|---------------|---------|-------------------|--------|
| **csvlens** | Terminal (TUI) | ✅ Unlimited | ❌ View only | ✅ | Active |
| **CsvTitan** | Desktop App | ✅ 10GB+ | ❌ View only | ✅ | Commercial |
| **Edit CSV** (VSCode) | VSCode | ⚠️ ~50MB | ✅ Full | ❌ | Active |
| **Tabular Data Viewer** (VSCode) | VSCode | ✅ 1-8GB | ❌ View only | ✅ | Abandoned (2022) |
| **Excel** | Desktop | ⚠️ 1M rows | ✅ Full | ❌ | N/A |
| **Massive CSV** 👑 | VSCode + CLI | ✅ Multi-GB | ✅ Targeted edits | ✅ | **Building!** |

**Massive CSV's Unique Position:**
- **Only** VSCode extension that edits multi-GB CSVs
- Combines best of csvlens (performance) + Edit CSV (editing) + VSCode (IDE integration)
- CLI for power users, extension for visual editing
- Built for real-world data engineering workflows
- Open source with sustainable development model
- Part of the Massive Data Tools family (future-proof)

## Architecture

```
massive-csv/
├── massive-csv-core/         # Rust library (core functionality)
│   ├── src/
│   │   ├── lib.rs            # Public API re-exports
│   │   ├── reader.rs         # Memory-mapped CSV reading + line indexing
│   │   ├── searcher.rs       # Parallel text search (rayon)
│   │   ├── editor.rs         # Edit tracking & atomic save
│   │   ├── parser.rs         # CSV parsing, delimiter detection, serialization
│   │   └── error.rs          # Error types (thiserror)
│   ├── tests/
│   │   └── integration.rs    # Full workflow integration tests
│   └── Cargo.toml
│
├── massive-csv-cli/          # CLI tool
│   ├── src/
│   │   ├── main.rs           # Clap subcommands: info, view, search, edit
│   │   └── format.rs         # Table formatting, number/size display
│   └── Cargo.toml
│
└── massive-csv-vscode/       # VSCode extension (planned)
    ├── src/
    │   ├── extension.ts      # Extension entry point
    │   ├── csvProvider.ts    # Custom editor provider
    │   └── backend.ts        # Rust backend bridge (napi-rs)
    ├── webview/
    │   └── index.html        # Table UI (virtual scrolling)
    └── package.json
```

## Technology Stack

### Core (Rust)
- **memmap2**: Memory-mapped file I/O
- **csv**: CSV parsing/writing
- **rayon**: Parallel search
- **serde**: Serialization for API

### CLI
- **clap**: Command-line argument parsing (derive API)

### VSCode Extension
- **TypeScript**: Extension code
- **napi-rs**: Rust ↔ Node.js bridge
- **Handsontable** or **ag-Grid**: Virtual scrolling table UI

## Development Phases

### Phase 1: Core Library -- COMPLETE
Build the Rust core that both CLI and extension will use.

**Goals:**
- [x] Memory-mapped CSV reading
- [x] Line indexing (track byte position of each row, O(1) access)
- [x] Parallel search with rayon (column filter, case-insensitive, max results)
- [x] Read specific rows by line number + range queries
- [x] Auto delimiter detection (comma, tab, semicolon, pipe)
- [x] CSV parsing with proper quoting (csv crate)
- [x] Edit tracking (HashMap of row -> fields)
- [x] Atomic save (temp file + rename)
- [x] Error handling (thiserror)
- [x] 28 tests (23 unit + 5 integration)

**Core API:**
- `CsvReader::open(path)` — open file, build index, detect delimiter
- `CsvReader::get_row(n)` / `get_rows(start, end)` — O(1) row access
- `CsvReader::row_count()` / `headers()` / `delimiter()`
- `search(reader, query, options)` — parallel search with `SearchOptions`
- `CsvEditor::set_cell(row, col, value)` / `set_row(row, fields)`
- `CsvEditor::save()` — atomic save, re-opens reader afterward

### Phase 2: CLI Tool -- COMPLETE
Command-line interface for quick operations.

**Subcommands:**
```bash
massive-csv info data.csv                              # Row count, columns, size, delimiter
massive-csv view data.csv --rows 100-200               # View rows as formatted table
massive-csv search data.csv "error" -c status -i -n 50 # Search with filters
massive-csv edit data.csv --row 15023 --col status --value "fixed"  # Edit cell
```

**Goals:**
- [x] `info` — file metadata (rows, columns, size, delimiter, headers, load time)
- [x] `view` — formatted table output with row ranges
- [x] `search` — parallel search with column filter, case-insensitive, max results
- [x] `edit` — edit cell by column name or index, atomic save
- [x] Table formatting (column-aligned, truncation, comma-separated row numbers)
- [x] Error handling (invalid column, out of range, missing file)

### Phase 3: VSCode Extension (Week 3-5)
Visual editor with table UI.

**Features:**
- [ ] Custom editor for .csv files
- [ ] Virtual scrolling table (show 100 rows at a time)
- [ ] Search/filter UI
- [ ] Cell editing
- [ ] Save changes back to file
- [ ] Status bar (row count, file size, load time)

**Goals:**
1. Register custom editor for CSV files
2. Create webview with table UI
3. Bridge Rust backend via napi-rs
4. Implement search/filter
5. Implement editing & save

### Phase 4: Polish & Publish (Week 5-6)
- [ ] Error handling & validation
- [ ] Undo/redo for edits
- [ ] Better UI/UX
- [ ] Documentation
- [ ] Tests
- [ ] Publish CLI to crates.io
- [ ] Publish extension to VS Marketplace

## Technical Decisions

### Why Memory-Mapped I/O?
- Allows reading multi-GB files without loading into RAM
- OS handles caching automatically
- Fast random access to any row

### Why Line Indexing?
```rust
// Index maps line number → byte position
// Enables O(1) jump to any row
let index: Vec<u64> = vec![0, 45, 123, 198, ...];
```

### Edit Strategy
**For 1-10 row edits (your use case):**

Option A: Simple (MVP)
```rust
fn save_edits(file_path: &Path, edits: HashMap<usize, String>) -> Result<()> {
    // 1. Read entire file
    // 2. Apply edits to specific lines
    // 3. Write back atomically
    // ~1-2 seconds for 2.3M rows - acceptable
}
```

Option B: Optimized (future)
```rust
fn save_edits_smart(file_path: &Path, edits: HashMap<usize, String>) -> Result<()> {
    // If edit doesn't change line length:
    //   → Overwrite in-place (instant)
    // If length changes:
    //   → Rewrite from first changed line
}
```

Start with Option A, optimize later if needed.

### VSCode Extension Architecture
```
┌─────────────────────────────────┐
│  Webview (TypeScript + HTML)    │
│  - Handsontable virtual grid    │
│  - Displays 100 rows at a time  │
│  - Sends edit events to ext     │
└─────────────────────────────────┘
         ↓ postMessage
┌─────────────────────────────────┐
│  Extension (TypeScript)          │
│  - Custom editor provider       │
│  - Handles file lifecycle       │
│  - Bridges to Rust backend      │
└─────────────────────────────────┘
         ↓ napi-rs
┌─────────────────────────────────┐
│  Rust Backend (csv-lens-core)   │
│  - Memory-mapped file reading   │
│  - Line indexing                │
│  - Search/edit operations       │
└─────────────────────────────────┘
```

## Performance Targets

### File: 2.3M rows, ~500MB
- **Initial load**: < 2 seconds (build index)
- **Search**: < 500ms (full-text search)
- **Jump to row**: < 50ms
- **Save edits** (1-10 rows): < 2 seconds
- **Memory usage**: < 100MB (for index + current viewport)

## Getting Started

### 1. Create Project Structure
```bash
mkdir massive-csv && cd massive-csv
cargo new --lib massive-csv-core
cargo new massive-csv-cli
mkdir -p massive-csv-vscode/src
```

### 2. Start with Core Library
Begin with `massive-csv-core` - the foundation for both CLI and extension.

**First milestone:** Read a CSV, build line index, jump to any row.

### 3. Test with Real Data
Use an actual 2.3M row CSV to validate:
- Load time
- Memory usage
- Search speed
- Edit performance

## Code Examples

### Memory-Mapped Reader (reader.rs)
```rust
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub struct CsvReader {
    mmap: Mmap,
    line_index: Vec<u64>,  // byte position of each line
}

impl CsvReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        let line_index = build_index(&mmap);
        
        Ok(Self { mmap, line_index })
    }
    
    pub fn get_row(&self, row_num: usize) -> Option<&str> {
        let start = self.line_index.get(row_num)? as usize;
        let end = self.line_index.get(row_num + 1)
            .map(|&pos| pos as usize)
            .unwrap_or(self.mmap.len());
        
        std::str::from_utf8(&self.mmap[start..end]).ok()
    }
    
    pub fn row_count(&self) -> usize {
        self.line_index.len()
    }
}

fn build_index(data: &[u8]) -> Vec<u64> {
    let mut index = vec![0u64];
    
    for (pos, &byte) in data.iter().enumerate() {
        if byte == b'\n' {
            index.push((pos + 1) as u64);
        }
    }
    
    index
}
```

### Search (searcher.rs)
```rust
use rayon::prelude::*;

pub struct SearchResult {
    pub row_num: usize,
    pub content: String,
}

impl CsvReader {
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        (0..self.row_count())
            .into_par_iter()
            .filter_map(|row_num| {
                let content = self.get_row(row_num)?;
                if content.contains(query) {
                    Some(SearchResult {
                        row_num,
                        content: content.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
```

## Current Status

**Phase:** Phase 2 complete (Core + CLI)
**Next Steps:**
1. Phase 3: Build VSCode extension (custom editor, virtual scrolling, napi-rs bridge)
2. Test with real 2.3M+ row CSV files for performance benchmarking
3. Phase 4: Polish, undo/redo, publish to crates.io + VS Marketplace

## Notes for Claude

### When working on this project:

**For massive-csv-core:**
- Focus on performance - this will handle GB-sized files
- Keep API simple - both CLI and extension will use this
- Test with actual large files early
- Use Pedro's xmlshift learnings (similar problem domain)

**For CLI:**
- Make it useful standalone (Pedro can use this daily)
- Interactive mode is nice-to-have, not critical
- Focus on search + quick edits

**For VSCode Extension:**
- Start simple - just viewing first
- Use proven libraries (Handsontable for virtual scrolling)
- napi-rs for Rust bridge - it's well documented
- Look at existing CSV extensions for UI patterns (then improve them)

**Testing:**
- Use actual large CSV data (2.3M+ rows)
- Test on macOS (Pedro's platform)
- Verify memory usage stays low
- Benchmark search speed

**Incremental Development:**
Each piece should be usable standalone:
1. Core library → works with `cargo test`
2. CLI tool → works from terminal
3. Extension → builds on proven core

## Massive Data Tools Ecosystem

### The Vision: A Family of Data Engineering Tools

Massive CSV is the **first tool** in what will become a comprehensive suite of data engineering utilities. Each tool addresses a specific pain point when working with large datasets.

**Phase 1: Massive CSV** (Current)
- VSCode extension + CLI for editing multi-GB CSVs
- Memory-efficient viewing, searching, editing
- Target: Ship v1.0 in 6 weeks

**Phase 2: Massive XML** (3-6 months)
- Rebrand existing xmlshift project
- Add VSCode extension companion
- Unified branding with massive-csv
- Cross-promote both tools

**Phase 3: Expansion** (6-12 months)
Consider based on user feedback and Pedro's needs:
- **massive-json** - Handle massive JSON arrays
- **massive-logs** - Grep/search through TB of log files
- **massive-parquet** - Work with huge Parquet files
- **massive-db** - Database export/import utilities

### Brand Development Roadmap

**Now:**
1. Register GitHub org: `github.com/massive-data-tools`
2. Create repos: `massive-csv`, `massive-xml`
3. Reserve domains: `massivedata.tools`, `massive-csv.dev`

**After Massive CSV v1.0:**
1. Launch website: showcasing both tools
2. Blog: "Building data tools for data engineers"
3. Documentation site
4. Community Discord/discussions

**Long-term:**
1. Unified documentation across all tools
2. Shared CLI framework
3. VSCode extension pack
4. Sponsorship/sustainability model
5. Conference talks/workshops

### Why This Approach Wins

**Professional Credibility:**
- "I built the Massive Data Tools suite"
- Shows commitment beyond one-off projects
- Attractive to recruiters and companies

**Cross-Promotion:**
- Users of massive-csv discover massive-xml
- Shared community across tools
- Network effects

**Portfolio Effect:**
- Demonstrates product thinking
- Shows ability to build and maintain multiple tools
- Evidence of technical leadership

**SEO & Discovery:**
- "Massive CSV" → leads to "Massive Data Tools"
- All tools reinforce each other in search
- Brand recognition compounds

**Sustainability:**
- Multiple products spread development risk
- Community can contribute to different tools
- Future consulting/training opportunities

## Resources

### Similar Projects (for reference)
- [qsv](https://github.com/jqnatividad/qsv) - Fast CSV CLI (Rust)
- [xsv](https://github.com/BurntSushi/xsv) - CSV command line toolkit
- [csvlens](https://github.com/YS-L/csvlens) - Terminal CSV viewer (TUI)
- [CsvTitan](https://csvtitan.com) - Desktop CSV viewer (commercial)
- Pedro's own xmlshift - XML parsing for large files

### Libraries to Use
- memmap2: https://docs.rs/memmap2/
- csv: https://docs.rs/csv/
- rayon: https://docs.rs/rayon/
- napi-rs: https://napi.rs/

### VSCode Extension Guides
- Custom Editors: https://code.visualstudio.com/api/extension-guides/custom-editors
- Webview API: https://code.visualstudio.com/api/extension-guides/webview

## Success Metrics

**This project is successful if:**
1. ✅ Pedro can open his 2.3M row CSV in < 3 seconds
2. ✅ Search finds rows in < 1 second
3. ✅ Can edit any row and save in < 3 seconds
4. ✅ Uses < 100MB RAM (vs GB in current tools)
5. ✅ Actually gets used in Pedro's daily workflow

**Bonus goals:**
- Other developers find it useful
- Published to crates.io + VS Marketplace
- Portfolio piece demonstrating Rust + systems programming

---

**Let's build something useful!** 🚀
