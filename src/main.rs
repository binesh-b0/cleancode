mod cli;
mod file_ops;

use std::path::Path;
use cli::build_cli;
use file_ops::{get_files_in_directory, read_file_lines};

fn main() {
    let matches = build_cli().get_matches();

    if let Some(file) = matches.get_one::<String>("file") {
        let path = Path::new(file);
        if let Ok(lines) = read_file_lines(path) {
            println!("Processing file: {}", file);
            for line in lines {
                println!("{}", line); // Placeholder for processing logic
            }
        } else {
            eprintln!("Failed to read file: {}", file);
        }
    }

    if let Some(directory) = matches.get_one::<String>("directory") {
        let path = Path::new(directory);
        let extensions = matches.get_one::<String>("extensions");
        let recursive = matches.contains_id("recursive");

        match get_files_in_directory(path, extensions.map(|s| s.as_str()), recursive) {
            Ok(files) => {
                for file in files {
                    println!("Found file: {}", file.display()); // Placeholder for processing logic
                }
            }
            Err(e) => eprintln!("Failed to read directory: {}", e),
        }
    }
}
