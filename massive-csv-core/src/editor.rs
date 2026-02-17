use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};

use tempfile::NamedTempFile;

use crate::error::{MassiveCsvError, Result};
use crate::parser::serialize_row;
use crate::reader::CsvReader;

/// A CSV editor that tracks changes in memory and saves atomically.
pub struct CsvEditor {
    reader: CsvReader,
    /// Pending edits: row_num -> edited fields
    edits: HashMap<usize, Vec<String>>,
}

impl CsvEditor {
    /// Create an editor from an existing reader.
    pub fn new(reader: CsvReader) -> Self {
        Self {
            reader,
            edits: HashMap::new(),
        }
    }

    /// Open a file for editing.
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let reader = CsvReader::open(path)?;
        Ok(Self::new(reader))
    }

    /// Access the underlying reader.
    pub fn reader(&self) -> &CsvReader {
        &self.reader
    }

    /// Number of pending edits.
    pub fn edit_count(&self) -> usize {
        self.edits.len()
    }

    /// Check if there are any unsaved changes.
    pub fn has_changes(&self) -> bool {
        !self.edits.is_empty()
    }

    /// Get the current state of a row (edited version if modified, otherwise from file).
    pub fn get_row(&self, row: usize) -> Result<Vec<String>> {
        if let Some(edited) = self.edits.get(&row) {
            Ok(edited.clone())
        } else {
            self.reader.get_row(row)
        }
    }

    /// Replace an entire row with new fields.
    pub fn set_row(&mut self, row: usize, fields: Vec<String>) -> Result<()> {
        let count = self.reader.row_count();
        if row >= count {
            return Err(MassiveCsvError::RowOutOfRange(row, count));
        }
        self.edits.insert(row, fields);
        Ok(())
    }

    /// Edit a single cell (row, column_index).
    pub fn set_cell(&mut self, row: usize, col: usize, value: String) -> Result<()> {
        let mut fields = self.get_row(row)?;

        if col >= fields.len() {
            return Err(MassiveCsvError::ColumnNotFound(format!("index {col}")));
        }

        fields[col] = value;
        self.edits.insert(row, fields);
        Ok(())
    }

    /// Revert a row to its original state.
    pub fn revert_row(&mut self, row: usize) {
        self.edits.remove(&row);
    }

    /// Revert all pending edits.
    pub fn revert_all(&mut self) {
        self.edits.clear();
    }

    /// Save all changes atomically.
    ///
    /// Strategy: write all rows to a temp file in the same directory,
    /// then atomically rename it over the original file.
    /// After save, re-opens the reader to reflect the new file contents.
    pub fn save(&mut self) -> Result<()> {
        if self.edits.is_empty() {
            return Ok(());
        }

        let path = self.reader.path().to_path_buf();
        let parent = path.parent().unwrap_or(std::path::Path::new("."));
        let delimiter = self.reader.delimiter();

        // Create temp file in the same directory (required for atomic rename)
        let temp = NamedTempFile::new_in(parent)?;
        let mut writer = BufWriter::new(&temp);

        // Write header
        let header_line = serialize_row(self.reader.headers(), delimiter);
        writer.write_all(header_line.as_bytes())?;
        writer.write_all(b"\n")?;

        // Write all rows, substituting edits
        let row_count = self.reader.row_count();
        for i in 0..row_count {
            if let Some(edited_fields) = self.edits.get(&i) {
                let line = serialize_row(edited_fields, delimiter);
                writer.write_all(line.as_bytes())?;
            } else {
                let raw = self.reader.get_row_raw(i)?;
                writer.write_all(raw.as_bytes())?;
            }
            writer.write_all(b"\n")?;
        }

        writer.flush()?;
        drop(writer);

        // Atomic rename
        // On Unix, persist does rename(2). On Windows, it falls back to copy+delete.
        temp.persist(&path).map_err(|e| e.error)?;

        // Ensure filesystem has flushed the directory entry
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }

        // Re-open reader with new file contents
        self.reader = CsvReader::open(&path)?;
        self.edits.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    fn make_csv(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn edit_and_save() {
        let f = make_csv("name,age\nAlice,30\nBob,25\n");
        let path = f.path().to_path_buf();

        let mut editor = CsvEditor::open(&path).unwrap();
        assert_eq!(editor.edit_count(), 0);

        editor.set_cell(0, 1, "31".to_string()).unwrap();
        assert_eq!(editor.edit_count(), 1);
        assert!(editor.has_changes());

        editor.save().unwrap();
        assert_eq!(editor.edit_count(), 0);

        // Verify the save
        let row = editor.get_row(0).unwrap();
        assert_eq!(row, vec!["Alice", "31"]);

        // Original row should be unchanged
        let row = editor.get_row(1).unwrap();
        assert_eq!(row, vec!["Bob", "25"]);
    }

    #[test]
    fn set_row_and_revert() {
        let f = make_csv("a,b\n1,2\n3,4\n");
        let path = f.path().to_path_buf();

        let mut editor = CsvEditor::open(&path).unwrap();

        editor
            .set_row(0, vec!["x".to_string(), "y".to_string()])
            .unwrap();
        assert_eq!(editor.get_row(0).unwrap(), vec!["x", "y"]);

        editor.revert_row(0);
        assert_eq!(editor.get_row(0).unwrap(), vec!["1", "2"]);
        assert!(!editor.has_changes());
    }

    #[test]
    fn out_of_range_edit() {
        let f = make_csv("h\n1\n");
        let path = f.path().to_path_buf();

        let mut editor = CsvEditor::open(&path).unwrap();
        let result = editor.set_row(99, vec!["x".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn save_no_changes_is_noop() {
        let f = make_csv("h\n1\n");
        let path = f.path().to_path_buf();

        let mut editor = CsvEditor::open(&path).unwrap();
        editor.save().unwrap(); // should be a no-op
    }

    #[test]
    fn multiple_edits_save() {
        let f = make_csv("x\na\nb\nc\nd\n");
        let path = f.path().to_path_buf();

        let mut editor = CsvEditor::open(&path).unwrap();

        editor.set_row(0, vec!["A".to_string()]).unwrap();
        editor.set_row(2, vec!["C".to_string()]).unwrap();
        editor.set_row(3, vec!["D".to_string()]).unwrap();

        editor.save().unwrap();

        assert_eq!(editor.get_row(0).unwrap(), vec!["A"]);
        assert_eq!(editor.get_row(1).unwrap(), vec!["b"]);
        assert_eq!(editor.get_row(2).unwrap(), vec!["C"]);
        assert_eq!(editor.get_row(3).unwrap(), vec!["D"]);
    }
}
