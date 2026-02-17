/// Print rows as a formatted table to stdout.
///
/// `row_numbers` maps each row in `rows` to its original row number in the file.
pub fn print_table(headers: &[String], rows: &[Vec<String>], row_numbers: &[usize]) {
    if headers.is_empty() {
        return;
    }

    let max_col_width: usize = 40;
    let num_cols = headers.len();

    // "Row" label column width: at least 3 chars, or as wide as the largest row number
    let row_label_width = row_numbers
        .iter()
        .map(|n| format_number(*n).len())
        .max()
        .unwrap_or(3)
        .max(3);

    // Compute column widths from headers and data
    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, field) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(field.len());
            }
        }
    }
    // Cap each column
    for w in col_widths.iter_mut() {
        *w = (*w).min(max_col_width);
    }

    // Print header
    print!(" {:>width$} ", "Row", width = row_label_width);
    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            print!(" | ");
        } else {
            print!("| ");
        }
        print!("{:<width$}", truncate(header, col_widths[i]), width = col_widths[i]);
    }
    println!();

    // Print separator
    print!("-{:-<width$}-", "", width = row_label_width);
    for (i, w) in col_widths.iter().enumerate() {
        if i > 0 {
            print!("-+-");
        } else {
            print!("+-");
        }
        print!("{:-<width$}", "", width = w);
    }
    println!();

    // Print rows
    for (row_idx, row) in rows.iter().enumerate() {
        let row_num = row_numbers.get(row_idx).copied().unwrap_or(row_idx);
        print!(" {:>width$} ", format_number(row_num), width = row_label_width);
        for i in 0..num_cols {
            if i > 0 {
                print!(" | ");
            } else {
                print!("| ");
            }
            let field = row.get(i).map(|s| s.as_str()).unwrap_or("");
            print!("{:<width$}", truncate(field, col_widths[i]), width = col_widths[i]);
        }
        println!();
    }
}

/// Truncate a string to `max_len`, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format a number with comma separators (e.g., 1234567 -> "1,234,567").
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Format a byte count as a human-readable size (e.g., "487.3 MB").
pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

/// Get a human-readable name for a delimiter byte.
pub fn delimiter_name(delim: u8) -> &'static str {
    match delim {
        b',' => "comma",
        b'\t' => "tab",
        b';' => "semicolon",
        b'|' => "pipe",
        _ => "unknown",
    }
}
