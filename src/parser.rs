use regex::Regex;

pub fn remove_debug_statements(content: &str, file_extension: &str) -> String {
    let mut cleaned_content = String::new();

    // Define regex patterns for JavaScript and Python
    let js_pattern = Regex::new(r"(?m)^\s*console\.log\(.*\);?\s*$").unwrap();
    let py_pattern = Regex::new(r"(?m)^\s*print\(.*\)\s*$").unwrap();

    // Select the appropriate pattern based on file extension
    let pattern = match file_extension {
        "js" => &js_pattern,
        "py" => &py_pattern,
        _ => return content.to_string(),
    };

    // Iterate over each line and filter out debug statements
    for line in content.lines() {
        if !pattern.is_match(line) {
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
        }
    }

    cleaned_content
}
