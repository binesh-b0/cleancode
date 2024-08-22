use prettytable::{Table, Row, Cell, format};
use std::collections::HashMap;

pub fn print_stats_summary(stats_summary: Vec<(String, HashMap<String, usize>)>) {
    let mut table = Table::new();

    // Set table format for better styling
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);

    // Add the header
    table.add_row(Row::new(vec![
        Cell::new("File").style_spec("bFc"),
        Cell::new("Total Lines").style_spec("bFc"),
        Cell::new("Code Lines").style_spec("bFc"),
        Cell::new("Debug Lines").style_spec("bFc"),
        Cell::new("Comment Lines").style_spec("bFc"),
        Cell::new("Empty Lines").style_spec("bFc"),
        Cell::new("Brace Lines").style_spec("bFc"),
        Cell::new("Comma Lines").style_spec("bFc"),
        Cell::new("Semicolon Lines").style_spec("bFc"),
    ]));

    // Initialize counters for totals
    let mut total_lines = 0;
    let mut total_debug_lines = 0;
    let mut total_empty_lines = 0;
    let mut total_brace_lines = 0;
    let mut total_comma_lines = 0;
    let mut total_semicolon_lines = 0;
    let mut total_comment_lines = 0;
    let mut total_real_code_lines = 0;

    // Add rows for each file and update totals
    for (file, stats) in &stats_summary {
        let file_total_lines = *stats.get("Total Lines").unwrap_or(&0);
        let file_real_code_lines = *stats.get("Real Code Lines").unwrap_or(&0);
        let file_debug_lines = *stats.get("Debug Lines").unwrap_or(&0);
        let file_comment_lines = *stats.get("Comment Lines").unwrap_or(&0);
        let file_empty_lines = *stats.get("Empty Lines").unwrap_or(&0);
        let file_brace_lines = *stats.get("Brace Lines").unwrap_or(&0);
        let file_comma_lines = *stats.get("Comma Lines").unwrap_or(&0);
        let file_semicolon_lines = *stats.get("Semicolon Lines").unwrap_or(&0);

        // Update totals
        total_lines += file_total_lines;
        total_debug_lines += file_debug_lines;
        total_comment_lines += file_comment_lines;
        total_empty_lines += file_empty_lines;
        total_brace_lines += file_brace_lines;
        total_comma_lines += file_comma_lines;
        total_semicolon_lines += file_semicolon_lines;
        total_real_code_lines += file_real_code_lines;

        table.add_row(Row::new(vec![
            Cell::new(file),
            Cell::new(&file_total_lines.to_string()),
            Cell::new(&file_real_code_lines.to_string()),
            Cell::new(&file_debug_lines.to_string()),
            Cell::new(&file_comment_lines.to_string()),
            Cell::new(&file_empty_lines.to_string()),
            Cell::new(&file_brace_lines.to_string()),
            Cell::new(&file_comma_lines.to_string()),
            Cell::new(&file_semicolon_lines.to_string()),
        ]));
    }

    // Add a summary row
    table.add_row(Row::new(vec![
        Cell::new("Total").style_spec("bFc"),  // Highlight this row
        Cell::new(&total_lines.to_string()).style_spec("b"),
        Cell::new(&total_real_code_lines.to_string()).style_spec("b"),
        Cell::new(&total_debug_lines.to_string()).style_spec("b"),
        Cell::new(&total_comment_lines.to_string()).style_spec("b"),
        Cell::new(&total_empty_lines.to_string()).style_spec("b"),
        Cell::new(&total_brace_lines.to_string()).style_spec("b"),
        Cell::new(&total_comma_lines.to_string()).style_spec("b"),
        Cell::new(&total_semicolon_lines.to_string()).style_spec("b"),
    ]));

    table.printstd();
}
