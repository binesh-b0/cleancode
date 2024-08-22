use colored::Colorize;
use indicatif::ProgressBar;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn remove_debug_statements(content: &str, file_extension: &str) -> String {
    let mut cleaned_content = String::new();

    let js_pattern = Regex::new(r"(?m)^\s*console\.log\(.*?\);?\s*(//.*)?$").unwrap();
    let py_pattern = Regex::new(r"(?m)^\s*print\(.*?\)\s*(#.*)?$").unwrap();

    let pattern = match file_extension {
        "js" => &js_pattern,
        "ts" => &js_pattern,
        "py" => &py_pattern,
        _ => return content.to_string(),
    };

    for line in content.lines() {
        if !pattern.is_match(line) {
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
        }
    }

    cleaned_content
}

pub fn extract_imports(content: &str, base_path: &Path, extension: &str) -> Vec<PathBuf> {
    let mut imports = Vec::new();
    let import_pattern: Regex;

    // Regex to capture imports in JavaScript/TypeScript or require in CommonJS
    match extension {
        "js" | "ts" => {
            import_pattern = Regex::new(
                r#"(?m)^\s*import\s*(?:(?:.*?\s+from\s+)?["'](.+?)["'])|require\(["'](.+?)["']\)"#,
            )
            .unwrap();
        }
        "py" => {
            import_pattern = Regex::new(r#"(?m)^\s*(?:from\s+([a-zA-Z_][a-zA-Z0-9_\.]*)\s+import\s+[a-zA-Z_][a-zA-Z0-9_\.]*)|(?:import\s+([a-zA-Z_][a-zA-Z0-9_\.]*))"#).unwrap();
        }
        _ => {
            println!("No import pattern available for extension: {}", extension);
            return vec![];
        }
    }

    for cap in import_pattern.captures_iter(content) {
        if let Some(import_path) = cap.get(1).or_else(|| cap.get(2)) {
            let mut path = base_path.to_path_buf();
            path.push(import_path.as_str());

            // Adjust for relative paths and missing extensions
            if path.extension().is_none() {
                path.set_extension(extension); // Assuming .js extension for now
            }

            if path.exists() {
                imports.push(path);
            } else {
                eprintln!("Warning: Import not found at path: {}", path.display());
            }
        }
    }

    imports
}

// logic for multi line comments is wrong.
pub fn calculate_stats(
    path: &Path,
    pb: Option<&ProgressBar>,
    verbose: bool,
) -> HashMap<String, usize> {
    let mut stats = HashMap::new();

    if let Ok(content) = fs::read_to_string(path) {
        let mut total_lines = 0;
        let mut debug_lines = 0;
        let mut empty_lines = 0;
        let mut brace_lines = 0;
        let mut comma_lines = 0;
        let mut semicolon_lines = 0;
        let mut real_code_lines = 0;
        let mut comment_lines = 0;

        let mut multiline = false;

        for line in content.lines() {
            total_lines += 1;

            let trimmed_line = line.trim();

            if trimmed_line.is_empty() {
                empty_lines += 1;
            } else if trimmed_line.starts_with("console.log") || trimmed_line.starts_with("print") {
                debug_lines += 1;
            } else if trimmed_line == "{" || trimmed_line == "}" {
                brace_lines += 1;
            } else if trimmed_line == "," {
                comma_lines += 1;
            } else if trimmed_line == ";" {
                semicolon_lines += 1;
            } else if trimmed_line.starts_with("//") {
                comment_lines += 1;
            } else if trimmed_line.starts_with("/*") {
                multiline = true;
                comment_lines += 1;
            } else if trimmed_line.starts_with("*/") {
                comment_lines += 1;
                multiline = false;
            } else if multiline {
                comment_lines += 1;
                continue;
            } else {
                real_code_lines += 1;
            }
        }

        stats.insert("Total Lines".to_string(), total_lines);
        stats.insert("Real Code Lines".to_string(), real_code_lines);
        stats.insert("Debug Lines".to_string(), debug_lines);
        stats.insert("Empty Lines".to_string(), empty_lines);
        stats.insert("Brace Lines".to_string(), brace_lines);
        stats.insert("Comma Lines".to_string(), comma_lines);
        stats.insert("Semicolon Lines".to_string(), semicolon_lines);
        stats.insert("Comment Lines".to_string(), comment_lines);

        if let Some(spinner) = pb {
            if verbose {
                spinner.println(format!(
                    "File: {} | Total: {} | Real Code: {} | Debug: {} | Empty: {} | Braces: {} | Commas: {} | Semicolons: {} | Comments: {}",
                    path.display(),
                    total_lines,
                    real_code_lines,
                    debug_lines,
                    empty_lines,
                    brace_lines,
                    comma_lines,
                    semicolon_lines,
                    comment_lines,
                ));
            }
        }
    } else {
        if let Some(spinner) = pb {
            spinner.println(format!(
                "Failed to read file: {}",
                path.display().to_string().red()
            ));
        }
    }

    stats
}
