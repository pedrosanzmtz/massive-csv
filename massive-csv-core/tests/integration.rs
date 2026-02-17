use massive_csv_core::{CsvEditor, CsvReader, SearchOptions};
use std::io::Write;

fn create_test_csv(rows: usize) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();

    // Header
    writeln!(f, "id,name,status,value").unwrap();

    // Data rows
    for i in 0..rows {
        let status = if i % 3 == 0 {
            "active"
        } else if i % 3 == 1 {
            "inactive"
        } else {
            "pending"
        };
        writeln!(f, "{},user_{},{},{:.2}", i, i, status, i as f64 * 1.5).unwrap();
    }

    f.flush().unwrap();
    f
}

#[test]
fn full_workflow_open_search_edit_save_verify() {
    let f = create_test_csv(10_000);
    let path = f.path().to_path_buf();

    // 1. Open and verify basic properties
    let reader = CsvReader::open(&path).unwrap();
    assert_eq!(reader.row_count(), 10_000);
    assert_eq!(reader.headers(), &["id", "name", "status", "value"]);
    assert_eq!(reader.delimiter(), b',');

    // 2. Read specific rows
    let first_row = reader.get_row(0).unwrap();
    assert_eq!(first_row[0], "0");
    assert_eq!(first_row[1], "user_0");
    assert_eq!(first_row[2], "active");

    let last_row = reader.get_row(9999).unwrap();
    assert_eq!(last_row[0], "9999");

    // 3. Read a range of rows
    let range = reader.get_rows(100, 105).unwrap();
    assert_eq!(range.len(), 5);
    assert_eq!(range[0][0], "100");
    assert_eq!(range[4][0], "104");

    // 4. Search across all columns
    let results = massive_csv_core::search(&reader, "user_500", &SearchOptions::default()).unwrap();
    // Should match row 500, and possibly user_5000-5009 (contains "user_500")
    assert!(results.iter().any(|r| r.row_num == 500));

    // 5. Search in specific column
    let opts = SearchOptions {
        column: Some("status".to_string()),
        ..Default::default()
    };
    let active_results = massive_csv_core::search(&reader, "active", &opts).unwrap();
    // Every 3rd row is "active" (but "inactive" also contains "active")
    // With column-specific search, "inactive" also matches since it contains "active"
    assert!(!active_results.is_empty());

    // 6. Case-insensitive search
    let opts = SearchOptions {
        case_insensitive: true,
        column: Some("status".to_string()),
        ..Default::default()
    };
    let results = massive_csv_core::search(&reader, "ACTIVE", &opts).unwrap();
    assert!(!results.is_empty());

    // 7. Search with max_results
    let opts = SearchOptions {
        max_results: 5,
        ..Default::default()
    };
    let results = massive_csv_core::search(&reader, "user_", &opts).unwrap();
    assert_eq!(results.len(), 5);

    // 8. Edit and save
    drop(reader); // Release mmap before editing

    let mut editor = CsvEditor::open(&path).unwrap();

    // Edit a single cell
    editor.set_cell(0, 2, "completed".to_string()).unwrap();

    // Edit an entire row
    editor
        .set_row(
            1,
            vec![
                "1".to_string(),
                "modified_user".to_string(),
                "archived".to_string(),
                "999.99".to_string(),
            ],
        )
        .unwrap();

    assert_eq!(editor.edit_count(), 2);
    assert!(editor.has_changes());

    // Verify in-memory edits before save
    let row0 = editor.get_row(0).unwrap();
    assert_eq!(row0[2], "completed");

    let row1 = editor.get_row(1).unwrap();
    assert_eq!(row1[1], "modified_user");

    // Unedited row should still be original
    let row2 = editor.get_row(2).unwrap();
    assert_eq!(row2[1], "user_2");

    // 9. Save
    editor.save().unwrap();
    assert_eq!(editor.edit_count(), 0);
    assert!(!editor.has_changes());

    // 10. Verify saved data by re-reading
    let row0 = editor.get_row(0).unwrap();
    assert_eq!(row0[2], "completed");

    let row1 = editor.get_row(1).unwrap();
    assert_eq!(row1[1], "modified_user");
    assert_eq!(row1[3], "999.99");

    // Unedited rows preserved
    let row2 = editor.get_row(2).unwrap();
    assert_eq!(row2[1], "user_2");

    // Row count unchanged
    assert_eq!(editor.reader().row_count(), 10_000);
}

#[test]
fn revert_workflow() {
    let f = create_test_csv(100);
    let path = f.path().to_path_buf();

    let mut editor = CsvEditor::open(&path).unwrap();

    let original = editor.get_row(0).unwrap();

    editor.set_cell(0, 1, "changed".to_string()).unwrap();
    assert_eq!(editor.get_row(0).unwrap()[1], "changed");

    editor.revert_row(0);
    assert_eq!(editor.get_row(0).unwrap(), original);
    assert!(!editor.has_changes());
}

#[test]
fn revert_all_workflow() {
    let f = create_test_csv(100);
    let path = f.path().to_path_buf();

    let mut editor = CsvEditor::open(&path).unwrap();

    editor.set_cell(0, 1, "a".to_string()).unwrap();
    editor.set_cell(1, 1, "b".to_string()).unwrap();
    editor.set_cell(2, 1, "c".to_string()).unwrap();
    assert_eq!(editor.edit_count(), 3);

    editor.revert_all();
    assert_eq!(editor.edit_count(), 0);
    assert!(!editor.has_changes());
}

#[test]
fn delimiter_detection() {
    // Tab-separated
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "a\tb\tc").unwrap();
    writeln!(f, "1\t2\t3").unwrap();
    writeln!(f, "4\t5\t6").unwrap();
    f.flush().unwrap();

    let reader = CsvReader::open(f.path()).unwrap();
    assert_eq!(reader.delimiter(), b'\t');
    assert_eq!(reader.headers(), &["a", "b", "c"]);
    assert_eq!(reader.get_row(0).unwrap(), vec!["1", "2", "3"]);
}

#[test]
fn quoted_fields() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "name,description,value").unwrap();
    writeln!(f, r#"test,"hello, world",42"#).unwrap();
    writeln!(f, r#""quoted name",simple,99"#).unwrap();
    f.flush().unwrap();

    let reader = CsvReader::open(f.path()).unwrap();
    assert_eq!(reader.row_count(), 2);

    let row0 = reader.get_row(0).unwrap();
    assert_eq!(row0[0], "test");
    assert_eq!(row0[1], "hello, world");
    assert_eq!(row0[2], "42");

    let row1 = reader.get_row(1).unwrap();
    assert_eq!(row1[0], "quoted name");
}
