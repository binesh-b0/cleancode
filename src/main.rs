mod cli;
mod file_ops;
mod parser;

use std::fs;
use std::path::Path;
use cli::build_cli;
use file_ops::get_files_in_directory;
use parser::remove_debug_statements;

fn main() {
    let matches = build_cli().get_matches();

    // Print the arguments for debugging
    println!("Arguments passed: {:?}", matches);

    // Correctly check if --dry-run is passed
    let dry_run = matches.get_flag("dry_run");

    if let Some(file) = matches.get_one::<String>("file") {
        process_file(Path::new(file), dry_run);
    }

    if let Some(directory) = matches.get_one::<String>("directory") {
        let path = Path::new(directory);
        let extensions = matches.get_one::<String>("extensions");
        let recursive = matches.get_flag("recursive");

        if let Ok(files) = get_files_in_directory(path, extensions.map(|s| s.as_str()), recursive) {
            for file in files {
                process_file(&file, dry_run);
            }
        } else {
            eprintln!("Failed to read directory: {}", directory);
        }
    }
}

fn process_file(path: &Path, dry_run: bool) {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        if let Ok(content) = fs::read_to_string(path) {
            let cleaned_content = remove_debug_statements(&content, extension);

            if dry_run {
                println!("Dry run for file: {}", path.display());
                println!("Cleaned Content:\n{}", cleaned_content);
            } else {
                // Backup the original file before making changes
                let backup_path = path.with_extension("bak");
                if let Err(e) = fs::write(&backup_path, &content) {
                    eprintln!("Failed to create backup file: {}", e);
                    return;
                }

                // Write the cleaned content to the original file
                if let Err(e) = fs::write(path, cleaned_content) {
                    eprintln!("Failed to write cleaned content to file: {}", e);
                    return;
                }

                println!("Processed and updated file: {}", path.display());
            }
        } else {
            eprintln!("Failed to read file: {}", path.display());
        }
    } else {
        eprintln!("Skipping file with no extension: {}", path.display());
    }
}
