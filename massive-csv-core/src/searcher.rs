use rayon::prelude::*;

use crate::error::Result;
use crate::parser::parse_row;
use crate::reader::CsvReader;

/// A single search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub row_num: usize,
    pub fields: Vec<String>,
}

/// Options controlling how search is performed.
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// If set, only search within this column name.
    pub column: Option<String>,
    /// Case-insensitive matching.
    pub case_insensitive: bool,
    /// Stop after finding this many results (0 = unlimited).
    pub max_results: usize,
}

/// Search the CSV for rows matching the query string.
///
/// Strategy: pre-filter on raw text (fast) before parsing fields (slow).
/// For column-specific searches, we still pre-filter on raw text, then
/// verify the match is in the target column after parsing.
pub fn search(
    reader: &CsvReader,
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    let column_index = if let Some(ref col_name) = options.column {
        let idx = reader
            .headers()
            .iter()
            .position(|h| h == col_name)
            .ok_or_else(|| crate::error::MassiveCsvError::ColumnNotFound(col_name.clone()))?;
        Some(idx)
    } else {
        None
    };

    let query_lower = if options.case_insensitive {
        query.to_lowercase()
    } else {
        query.to_string()
    };

    let row_count = reader.row_count();

    // Collect raw rows with indices so we can use rayon
    // For very large files, we process in chunks to allow early termination
    let results: Vec<SearchResult> = (0..row_count)
        .into_par_iter()
        .filter_map(|row_num| {
            let raw = reader.get_row_raw(row_num).ok()?;

            // Pre-filter: quick check if query appears in the raw line at all
            let matches_raw = if options.case_insensitive {
                raw.to_lowercase().contains(&query_lower)
            } else {
                raw.contains(query)
            };

            if !matches_raw {
                return None;
            }

            // Parse fields for column-specific check or to return
            let fields = parse_row(raw, reader.delimiter()).ok()?;

            if let Some(col_idx) = column_index {
                let field = fields.get(col_idx)?;
                let matches_field = if options.case_insensitive {
                    field.to_lowercase().contains(&query_lower)
                } else {
                    field.contains(query)
                };
                if !matches_field {
                    return None;
                }
            }

            Some(SearchResult { row_num, fields })
        })
        .collect();

    // Apply max_results after parallel collection (rayon doesn't support early exit cleanly)
    if options.max_results > 0 && results.len() > options.max_results {
        Ok(results.into_iter().take(options.max_results).collect())
    } else {
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_csv(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn search_all_columns() {
        let f = make_csv("name,city\nAlice,NYC\nBob,LA\nCarol,NYC\n");
        let reader = CsvReader::open(f.path()).unwrap();

        let results = search(&reader, "NYC", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].row_num, 0);
        assert_eq!(results[1].row_num, 2);
    }

    #[test]
    fn search_specific_column() {
        let f = make_csv("name,city\nAlice,NYC\nNYC,LA\n");
        let reader = CsvReader::open(f.path()).unwrap();

        let opts = SearchOptions {
            column: Some("city".to_string()),
            ..Default::default()
        };
        let results = search(&reader, "NYC", &opts).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].row_num, 0);
    }

    #[test]
    fn search_case_insensitive() {
        let f = make_csv("name\nAlice\nBOB\ncarol\n");
        let reader = CsvReader::open(f.path()).unwrap();

        let opts = SearchOptions {
            case_insensitive: true,
            ..Default::default()
        };
        let results = search(&reader, "bob", &opts).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fields, vec!["BOB"]);
    }

    #[test]
    fn search_max_results() {
        let f = make_csv("v\na\na\na\na\na\n");
        let reader = CsvReader::open(f.path()).unwrap();

        let opts = SearchOptions {
            max_results: 2,
            ..Default::default()
        };
        let results = search(&reader, "a", &opts).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_column_not_found() {
        let f = make_csv("name\nAlice\n");
        let reader = CsvReader::open(f.path()).unwrap();

        let opts = SearchOptions {
            column: Some("nonexistent".to_string()),
            ..Default::default()
        };
        let result = search(&reader, "x", &opts);
        assert!(result.is_err());
    }
}
