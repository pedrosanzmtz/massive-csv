use thiserror::Error;

pub type Result<T> = std::result::Result<T, MassiveCsvError>;

#[derive(Debug, Error)]
pub enum MassiveCsvError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Row {0} is out of range (file has {1} rows)")]
    RowOutOfRange(usize, usize),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("File is empty")]
    EmptyFile,

    #[error("Invalid UTF-8 at byte position {0}")]
    InvalidUtf8(usize),
}
