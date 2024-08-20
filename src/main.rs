mod cli;
mod file_ops;
mod parser;

use std::fs;
use std::path::Path;
use cli::build_cli;
use file_ops::{get_files_in_directory, read_file_lines};
use parser::remove_debug_statements;

fn main() {
    let matches = build_cli().get_matches();

    if let Some(file) = matches.get_one::<String>("file") {
        process_file(Path::new(file));
    }

    if let Some(directory) = matches.get_one::<String>("directory") {
        let path = Path::new(directory);
        let extensions = matches.get_one::<String>("extensions");
        let recursive = matches.contains_id("recursive");

        if let Ok(files) = get_files_in_directory(path, extensions.map(|s| s.as_str()), recursive) {
            for file in files {
                process_file(&file);
            }
        } else {
            eprintln!("Failed to read directory: {}", directory);
        }
    }
}

fn process_file(path: &Path) {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        if let Ok(content) = fs::read_to_string(path) {
            let cleaned_content = remove_debug_statements(&content, extension);
            fs::write(path, cleaned_content).expect("Failed to write file");
            println!("Processed file: {}", path.display());
        } else {
            eprintln!("Failed to read file: {}", path.display());
        }
    } else {
        eprintln!("Skipping file with no extension: {}", path.display());
    }
}
