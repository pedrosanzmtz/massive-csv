use std::path::Path;
use std::sync::Mutex;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use massive_csv_core::{CsvEditor, SearchOptions};

/// Info about an opened CSV file.
#[napi(object)]
pub struct CsvInfo {
    pub row_count: u32,
    pub headers: Vec<String>,
    pub delimiter: String,
    pub file_path: String,
}

/// A single search result returned to JS.
#[napi(object)]
pub struct JsSearchResult {
    pub row_num: u32,
    pub fields: Vec<String>,
}

/// Options for searching.
#[napi(object)]
pub struct JsSearchOptions {
    pub column: Option<String>,
    pub case_insensitive: Option<bool>,
    pub max_results: Option<u32>,
}

/// A CSV document backed by the massive-csv-core engine.
///
/// Wraps CsvEditor which itself wraps CsvReader, providing
/// memory-mapped reading, parallel search, edit tracking, and atomic save.
#[napi]
pub struct CsvDocument {
    editor: Mutex<CsvEditor>,
}

#[napi]
impl CsvDocument {
    /// Open a CSV file and return a CsvDocument.
    #[napi(factory)]
    pub fn open(path: String) -> Result<CsvDocument> {
        let editor = CsvEditor::open(Path::new(&path))
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(CsvDocument {
            editor: Mutex::new(editor),
        })
    }

    /// Get file metadata.
    #[napi]
    pub fn get_info(&self) -> Result<CsvInfo> {
        let editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        let reader = editor.reader();
        Ok(CsvInfo {
            row_count: reader.row_count() as u32,
            headers: reader.headers().to_vec(),
            delimiter: String::from(reader.delimiter() as char),
            file_path: reader.path().to_string_lossy().into_owned(),
        })
    }

    /// Get a single row (returns edited version if modified).
    #[napi]
    pub fn get_row(&self, row: u32) -> Result<Vec<String>> {
        let editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        editor
            .get_row(row as usize)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Get a range of rows [start, end). Returns edited versions where applicable.
    #[napi]
    pub fn get_rows(&self, start: u32, end: u32) -> Result<Vec<Vec<String>>> {
        let editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        let end = (end as usize).min(editor.reader().row_count());
        let mut rows = Vec::with_capacity(end.saturating_sub(start as usize));
        for i in (start as usize)..end {
            rows.push(
                editor
                    .get_row(i)
                    .map_err(|e| Error::from_reason(e.to_string()))?,
            );
        }
        Ok(rows)
    }

    /// Search for rows matching a query.
    #[napi]
    pub fn search(
        &self,
        query: String,
        options: Option<JsSearchOptions>,
    ) -> Result<Vec<JsSearchResult>> {
        let editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        let opts = match options {
            Some(o) => SearchOptions {
                column: o.column,
                case_insensitive: o.case_insensitive.unwrap_or(false),
                max_results: o.max_results.unwrap_or(0) as usize,
            },
            None => SearchOptions::default(),
        };
        let results = massive_csv_core::search(editor.reader(), &query, &opts)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| JsSearchResult {
                row_num: r.row_num as u32,
                fields: r.fields,
            })
            .collect())
    }

    /// Edit a single cell.
    #[napi]
    pub fn set_cell(&self, row: u32, col: u32, value: String) -> Result<()> {
        let mut editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        editor
            .set_cell(row as usize, col as usize, value)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Replace an entire row.
    #[napi]
    pub fn set_row(&self, row: u32, fields: Vec<String>) -> Result<()> {
        let mut editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        editor
            .set_row(row as usize, fields)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Revert a single row to its original state.
    #[napi]
    pub fn revert_row(&self, row: u32) -> Result<()> {
        let mut editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        editor.revert_row(row as usize);
        Ok(())
    }

    /// Revert all pending edits.
    #[napi]
    pub fn revert_all(&self) -> Result<()> {
        let mut editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        editor.revert_all();
        Ok(())
    }

    /// Save all pending edits atomically.
    #[napi]
    pub fn save(&self) -> Result<()> {
        let mut editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        editor
            .save()
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Number of pending edits.
    #[napi(getter)]
    pub fn edit_count(&self) -> Result<u32> {
        let editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(editor.edit_count() as u32)
    }

    /// Whether there are unsaved changes.
    #[napi(getter)]
    pub fn has_changes(&self) -> Result<bool> {
        let editor = self.editor.lock().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(editor.has_changes())
    }
}
