pub mod editor;
pub mod error;
pub mod parser;
pub mod reader;
pub mod searcher;

pub use editor::CsvEditor;
pub use error::{MassiveCsvError, Result};
pub use parser::Delimiter;
pub use reader::CsvReader;
pub use searcher::{SearchOptions, SearchResult};

/// Search convenience function re-exported at crate root.
pub fn search(
    reader: &CsvReader,
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    searcher::search(reader, query, options)
}
