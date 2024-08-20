use regex::Regex;

pub fn remove_debug_statements(content: &str, file_extension: &str) -> String {
    let mut cleaned_content = String::new();

    // Refined regex pattern for JavaScript
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
