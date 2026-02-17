use memmap2::Mmap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::error::{MassiveCsvError, Result};
use crate::parser::{detect_delimiter, parse_headers, parse_row};

/// A memory-mapped CSV reader with O(1) row access via line indexing.
pub struct CsvReader {
    mmap: Mmap,
    /// Byte offset of the start of each data row (row 0 = first row after header).
    line_index: Vec<u64>,
    headers: Vec<String>,
    delimiter: u8,
    path: PathBuf,
}

impl CsvReader {
    /// Open a CSV file, build the line index, and detect delimiter/headers.
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        if metadata.len() == 0 {
            return Err(MassiveCsvError::EmptyFile);
        }

        // SAFETY: We only read from the mmap. The file should not be modified externally
        // while we hold this mapping (standard mmap caveat).
        let mmap = unsafe { Mmap::map(&file)? };

        let delimiter = detect_delimiter(&mmap).as_byte();
        let headers = parse_headers(&mmap, delimiter)?;

        // Find where the header line ends
        let header_end = mmap
            .iter()
            .position(|&b| b == b'\n')
            .map(|pos| pos + 1)
            .unwrap_or(mmap.len());

        let line_index = build_index(&mmap, header_end);

        Ok(Self {
            mmap,
            line_index,
            headers,
            delimiter,
            path: path.to_path_buf(),
        })
    }

    /// Number of data rows (excluding header).
    pub fn row_count(&self) -> usize {
        self.line_index.len()
    }

    /// Column headers.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// The detected delimiter byte.
    pub fn delimiter(&self) -> u8 {
        self.delimiter
    }

    /// File path this reader was opened from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get a raw line as &str (zero-copy from mmap). Does not include the trailing newline.
    pub fn get_row_raw(&self, row: usize) -> Result<&str> {
        let count = self.row_count();
        if row >= count {
            return Err(MassiveCsvError::RowOutOfRange(row, count));
        }

        let start = self.line_index[row] as usize;
        let end = if row + 1 < count {
            self.line_index[row + 1] as usize
        } else {
            self.mmap.len()
        };

        let slice = &self.mmap[start..end];

        // Trim trailing \n and \r\n
        let slice = strip_line_ending(slice);

        std::str::from_utf8(slice).map_err(|_| MassiveCsvError::InvalidUtf8(start))
    }

    /// Get a row parsed into fields.
    pub fn get_row(&self, row: usize) -> Result<Vec<String>> {
        let raw = self.get_row_raw(row)?;
        parse_row(raw, self.delimiter)
    }

    /// Get a range of rows parsed into fields.
    pub fn get_rows(&self, start: usize, end: usize) -> Result<Vec<Vec<String>>> {
        let end = end.min(self.row_count());
        let mut rows = Vec::with_capacity(end.saturating_sub(start));
        for i in start..end {
            rows.push(self.get_row(i)?);
        }
        Ok(rows)
    }

    /// Re-open the file (e.g., after save). Returns a new CsvReader.
    pub fn reopen(&self) -> Result<Self> {
        Self::open(&self.path)
    }
}

/// Build a line index starting from `data_start` (byte position after the header line).
fn build_index(data: &[u8], data_start: usize) -> Vec<u64> {
    if data_start >= data.len() {
        return vec![];
    }

    let mut index = vec![data_start as u64];

    for pos in data_start..data.len() {
        if data[pos] == b'\n' && pos + 1 < data.len() {
            index.push((pos + 1) as u64);
        }
    }

    // If the last "row" is empty (file ends with \n), remove it
    if let Some(&last_offset) = index.last() {
        let last = last_offset as usize;
        if last >= data.len()
            || strip_line_ending(&data[last..])
                .iter()
                .all(|b| b.is_ascii_whitespace())
        {
            index.pop();
        }
    }

    index
}

fn strip_line_ending(data: &[u8]) -> &[u8] {
    let mut end = data.len();
    if end > 0 && data[end - 1] == b'\n' {
        end -= 1;
    }
    if end > 0 && data[end - 1] == b'\r' {
        end -= 1;
    }
    &data[..end]
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
    fn basic_read() {
        let f = make_csv("name,age\nAlice,30\nBob,25\n");
        let reader = CsvReader::open(f.path()).unwrap();

        assert_eq!(reader.headers(), &["name", "age"]);
        assert_eq!(reader.row_count(), 2);
        assert_eq!(reader.get_row(0).unwrap(), vec!["Alice", "30"]);
        assert_eq!(reader.get_row(1).unwrap(), vec!["Bob", "25"]);
    }

    #[test]
    fn no_trailing_newline() {
        let f = make_csv("x,y\n1,2\n3,4");
        let reader = CsvReader::open(f.path()).unwrap();
        assert_eq!(reader.row_count(), 2);
        assert_eq!(reader.get_row(1).unwrap(), vec!["3", "4"]);
    }

    #[test]
    fn out_of_range() {
        let f = make_csv("a\n1\n");
        let reader = CsvReader::open(f.path()).unwrap();
        assert!(reader.get_row(5).is_err());
    }

    #[test]
    fn empty_file() {
        let f = make_csv("");
        let result = CsvReader::open(f.path());
        assert!(result.is_err());
    }

    #[test]
    fn get_rows_range() {
        let f = make_csv("h\na\nb\nc\nd\ne\n");
        let reader = CsvReader::open(f.path()).unwrap();
        let rows = reader.get_rows(1, 3).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["b"]);
        assert_eq!(rows[1], vec!["c"]);
    }

    #[test]
    fn crlf_line_endings() {
        let f = make_csv("name,age\r\nAlice,30\r\nBob,25\r\n");
        let reader = CsvReader::open(f.path()).unwrap();
        assert_eq!(reader.row_count(), 2);
        assert_eq!(reader.get_row(0).unwrap(), vec!["Alice", "30"]);
    }
}
