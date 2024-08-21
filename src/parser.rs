use regex::Regex;
use std::path::{Path, PathBuf};

pub fn remove_debug_statements(content: &str, file_extension: &str) -> String {
    let mut cleaned_content = String::new();

    let js_pattern = Regex::new(r"(?m)^\s*console\.log\(.*?\);?\s*(//.*)?$").unwrap();
    let py_pattern = Regex::new(r"(?m)^\s*print\(.*?\)\s*(#.*)?$").unwrap();

    let pattern = match file_extension {
        "js" => &js_pattern,
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
    let import_pattern:Regex;

    // Regex to capture imports in JavaScript/TypeScript or require in CommonJS
    match extension {
        "js" | "ts" => {
            import_pattern = Regex::new(r#"(?m)^\s*import\s*(?:(?:.*?\s+from\s+)?["'](.+?)["'])|require\(["'](.+?)["']\)"#).unwrap();
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
