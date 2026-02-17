use crate::error::{MassiveCsvError, Result};

/// Supported CSV delimiters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Comma,
    Tab,
    Semicolon,
    Pipe,
}

impl Delimiter {
    pub fn as_byte(self) -> u8 {
        match self {
            Delimiter::Comma => b',',
            Delimiter::Tab => b'\t',
            Delimiter::Semicolon => b';',
            Delimiter::Pipe => b'|',
        }
    }

    fn all() -> &'static [Delimiter] {
        &[
            Delimiter::Comma,
            Delimiter::Tab,
            Delimiter::Semicolon,
            Delimiter::Pipe,
        ]
    }
}

/// Detect the delimiter by sampling the first lines of the file.
///
/// Strategy: for each candidate delimiter, count how many fields each line produces.
/// The best delimiter is the one where most lines produce a consistent (>1) field count.
pub fn detect_delimiter(data: &[u8]) -> Delimiter {
    let sample = first_n_lines(data, 20);
    if sample.is_empty() {
        return Delimiter::Comma;
    }

    let mut best = Delimiter::Comma;
    let mut best_score: usize = 0;

    for &delim in Delimiter::all() {
        let counts: Vec<usize> = sample
            .iter()
            .map(|line| count_fields(line, delim.as_byte()))
            .collect();

        // Skip if first line only has 1 field (delimiter not present)
        if counts.first().copied().unwrap_or(0) <= 1 {
            continue;
        }

        let mode = counts[0];
        let consistent = counts.iter().filter(|&&c| c == mode).count();

        // Score = consistency * field_count (prefer more fields when tied)
        let score = consistent * mode;
        if score > best_score {
            best_score = score;
            best = delim;
        }
    }

    best
}

/// Parse a raw line into fields using the csv crate (handles quoting properly).
pub fn parse_row(line: &str, delimiter: u8) -> Result<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_reader(line.as_bytes());

    let mut record = csv::StringRecord::new();
    if reader
        .read_record(&mut record)
        .map_err(MassiveCsvError::Csv)?
    {
        Ok(record.iter().map(|f| f.to_string()).collect())
    } else {
        Ok(vec![])
    }
}

/// Parse the first line of data as headers.
pub fn parse_headers(data: &[u8], delimiter: u8) -> Result<Vec<String>> {
    let first_line = first_line(data).ok_or(MassiveCsvError::EmptyFile)?;
    let line_str = std::str::from_utf8(first_line).map_err(|_| MassiveCsvError::InvalidUtf8(0))?;
    parse_row(line_str, delimiter)
}

/// Serialize fields back into a CSV line (with proper quoting).
pub fn serialize_row(fields: &[String], delimiter: u8) -> String {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_writer(Vec::new());

    writer
        .write_record(fields)
        .expect("write to Vec cannot fail");
    writer.flush().expect("flush to Vec cannot fail");

    let mut output = String::from_utf8(writer.into_inner().expect("flush already called"))
        .expect("csv crate produces valid utf-8");

    // Remove trailing newline that the csv writer adds
    if output.ends_with('\n') {
        output.pop();
        if output.ends_with('\r') {
            output.pop();
        }
    }

    output
}

fn first_line(data: &[u8]) -> Option<&[u8]> {
    if data.is_empty() {
        return None;
    }
    let end = data.iter().position(|&b| b == b'\n').unwrap_or(data.len());
    let line = &data[..end];
    // Strip trailing \r
    if line.last() == Some(&b'\r') {
        Some(&line[..line.len() - 1])
    } else {
        Some(line)
    }
}

fn first_n_lines(data: &[u8], n: usize) -> Vec<&[u8]> {
    let mut lines = Vec::with_capacity(n);
    let mut start = 0;

    for _ in 0..n {
        if start >= data.len() {
            break;
        }
        let remaining = &data[start..];
        let end = remaining
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(remaining.len());
        let line = &remaining[..end];
        // Strip trailing \r
        let line = if line.last() == Some(&b'\r') {
            &line[..line.len() - 1]
        } else {
            line
        };
        lines.push(line);
        start += end + 1;
    }

    lines
}

/// Count fields by counting unquoted delimiters + 1.
fn count_fields(line: &[u8], delimiter: u8) -> usize {
    let mut count = 1usize;
    let mut in_quotes = false;

    for &b in line {
        if b == b'"' {
            in_quotes = !in_quotes;
        } else if b == delimiter && !in_quotes {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_comma() {
        let data = b"a,b,c\n1,2,3\n4,5,6\n";
        assert_eq!(detect_delimiter(data), Delimiter::Comma);
    }

    #[test]
    fn detect_tab() {
        let data = b"a\tb\tc\n1\t2\t3\n4\t5\t6\n";
        assert_eq!(detect_delimiter(data), Delimiter::Tab);
    }

    #[test]
    fn detect_semicolon() {
        let data = b"a;b;c\n1;2;3\n4;5;6\n";
        assert_eq!(detect_delimiter(data), Delimiter::Semicolon);
    }

    #[test]
    fn detect_pipe() {
        let data = b"a|b|c\n1|2|3\n4|5|6\n";
        assert_eq!(detect_delimiter(data), Delimiter::Pipe);
    }

    #[test]
    fn parse_and_serialize_round_trip() {
        let line = r#"hello,"world, ok",test"#;
        let fields = parse_row(line, b',').unwrap();
        assert_eq!(fields, vec!["hello", "world, ok", "test"]);

        let serialized = serialize_row(&fields, b',');
        assert_eq!(serialized, r#"hello,"world, ok",test"#);
    }

    #[test]
    fn parse_headers_works() {
        let data = b"name,age,city\nAlice,30,NYC\n";
        let headers = parse_headers(data, b',').unwrap();
        assert_eq!(headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn empty_data_returns_comma() {
        assert_eq!(detect_delimiter(b""), Delimiter::Comma);
    }
}
