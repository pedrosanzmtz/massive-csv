mod format;

use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::{Parser, Subcommand};
use massive_csv_core::{CsvEditor, CsvReader, SearchOptions};

#[derive(Parser)]
#[command(name = "massive-csv")]
#[command(about = "View, search, and edit massive CSV files")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show file metadata: row count, columns, size, delimiter
    Info {
        /// Path to the CSV file
        file: PathBuf,
    },

    /// View rows from a CSV file as a formatted table
    View {
        /// Path to the CSV file
        file: PathBuf,

        /// Row range to display, e.g. "100-200" or "100" (default: first 20 rows)
        #[arg(short, long)]
        rows: Option<String>,
    },

    /// Search for rows matching a query
    Search {
        /// Path to the CSV file
        file: PathBuf,

        /// Text to search for
        query: String,

        /// Restrict search to a specific column name
        #[arg(short, long)]
        column: Option<String>,

        /// Case-insensitive matching
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Maximum number of results (default: 100)
        #[arg(short = 'n', long, default_value_t = 100)]
        max_results: usize,
    },

    /// Edit a specific cell and save
    Edit {
        /// Path to the CSV file
        file: PathBuf,

        /// Row number to edit (0-indexed)
        #[arg(long)]
        row: usize,

        /// Column name or 0-indexed column number
        #[arg(long)]
        col: String,

        /// New value for the cell
        #[arg(long)]
        value: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info { file } => cmd_info(&file),
        Commands::View { file, rows } => cmd_view(&file, rows.as_deref()),
        Commands::Search {
            file,
            query,
            column,
            ignore_case,
            max_results,
        } => cmd_search(&file, &query, column.as_deref(), ignore_case, max_results),
        Commands::Edit {
            file,
            row,
            col,
            value,
        } => cmd_edit(&file, row, &col, &value),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn cmd_info(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let reader = CsvReader::open(path)?;
    let elapsed = start.elapsed();

    let metadata = std::fs::metadata(path)?;
    let headers = reader.headers();

    // Show first 10 headers, abbreviate if more
    let header_display = if headers.len() <= 10 {
        headers.join(", ")
    } else {
        format!(
            "{}, ... (+{} more)",
            headers[..10].join(", "),
            headers.len() - 10
        )
    };

    println!("File:       {}", path.display());
    println!("Size:       {}", format::format_size(metadata.len()));
    println!("Rows:       {}", format::format_number(reader.row_count()));
    println!("Columns:    {}", headers.len());
    println!("Delimiter:  {}", format::delimiter_name(reader.delimiter()));
    println!("Headers:    {header_display}");
    println!("Load time:  {:.2?}", elapsed);

    Ok(())
}

fn cmd_view(path: &PathBuf, rows_arg: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let reader = CsvReader::open(path)?;
    let row_count = reader.row_count();

    let (start, end) = parse_row_range(rows_arg, row_count)?;

    if start >= row_count {
        eprintln!("Row {start} is out of range (file has {row_count} rows)");
        process::exit(1);
    }

    let rows = reader.get_rows(start, end)?;
    let row_numbers: Vec<usize> = (start..start + rows.len()).collect();

    format::print_table(reader.headers(), &rows, &row_numbers);

    Ok(())
}

fn cmd_search(
    path: &PathBuf,
    query: &str,
    column: Option<&str>,
    ignore_case: bool,
    max_results: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = CsvReader::open(path)?;

    let options = SearchOptions {
        column: column.map(|s| s.to_string()),
        case_insensitive: ignore_case,
        max_results,
    };

    let start = Instant::now();
    let results = massive_csv_core::search(&reader, query, &options)?;
    let elapsed = start.elapsed();

    let total = results.len();
    println!(
        "Found {} match{} (searched {} rows in {:.2?}):\n",
        format::format_number(total),
        if total == 1 { "" } else { "es" },
        format::format_number(reader.row_count()),
        elapsed,
    );

    if results.is_empty() {
        return Ok(());
    }

    let row_numbers: Vec<usize> = results.iter().map(|r| r.row_num).collect();
    let rows: Vec<Vec<String>> = results.into_iter().map(|r| r.fields).collect();

    format::print_table(reader.headers(), &rows, &row_numbers);

    Ok(())
}

fn cmd_edit(
    path: &PathBuf,
    row: usize,
    col: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = CsvEditor::open(path)?;
    let headers: Vec<String> = editor.reader().headers().to_vec();

    // Resolve column: try name first, then numeric index
    let col_idx = headers
        .iter()
        .position(|h| h == col)
        .or_else(|| col.parse::<usize>().ok().filter(|&i| i < headers.len()))
        .ok_or_else(|| {
            format!(
                "Column '{}' not found. Available: {}",
                col,
                headers.join(", ")
            )
        })?;

    let col_name = &headers[col_idx];

    // Get old value for display
    let old_row = editor.reader().get_row(row)?;
    let old_value = old_row
        .get(col_idx)
        .map(|s| s.as_str())
        .unwrap_or("<missing>");

    editor.set_cell(row, col_idx, value.to_string())?;
    editor.save()?;

    println!(
        "Updated row {}, column \"{}\": \"{}\" -> \"{}\"",
        format::format_number(row),
        col_name,
        old_value,
        value
    );
    println!("Saved.");

    Ok(())
}

/// Parse a row range string like "100-200" or "100" into (start, end).
/// Returns (start, end) where end is exclusive.
fn parse_row_range(
    arg: Option<&str>,
    row_count: usize,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    match arg {
        None => Ok((0, 20.min(row_count))),
        Some(s) => {
            if let Some((left, right)) = s.split_once('-') {
                let start: usize = left.trim().parse()?;
                let end: usize = right.trim().parse::<usize>()? + 1; // inclusive -> exclusive
                Ok((start, end.min(row_count)))
            } else {
                let n: usize = s.trim().parse()?;
                Ok((n, (n + 1).min(row_count)))
            }
        }
    }
}
