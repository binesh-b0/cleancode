mod cli;
mod file_ops;
mod parser;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use cli::{build_cli, default_exclusions};
use file_ops::get_files_in_directory;
use parser::{remove_debug_statements, extract_imports};
use colored::Colorize;

fn main() {
    let matches = build_cli().get_matches();
    let start_time = Instant::now();

    // Parse command line arguments
    let dry_run = matches.get_flag("dry_run");
    let verbose = matches.get_flag("verbose");
    let recursive = matches.get_flag("recursive");

    // Check that both directory and file are not provided simultaneously
    if matches.get_one::<String>("file").is_some() && matches.get_one::<String>("directory").is_some() {
        eprintln!("{}", "Error: You cannot specify both a file and a directory at the same time.".red().bold());
        return;
    }

    // Gather paths to exclude from processing
    let mut exclude_paths = default_exclusions();
    if let Some(excludes) = matches.get_many::<PathBuf>("exclude") {
        exclude_paths.extend(excludes.cloned());
    }

    let mut processed_files = HashSet::new();
    let mut total_files_processed = 0;
    let mut total_statements_removed = 0;

    // Create a spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        );
    spinner.enable_steady_tick(100); // Tick every 100ms
    spinner.set_message("Starting process...");

    // Process a single file if specified
    if let Some(file) = matches.get_one::<String>("file") {
        process_file(
            Path::new(file), 
            dry_run, 
            verbose, 
            &mut processed_files, 
            recursive, 
            &mut total_files_processed, 
            &mut total_statements_removed,
            Some(&spinner)
        );
    }

    // Process all files in a directory if specified
    if let Some(directory) = matches.get_one::<String>("directory") {
        let path = Path::new(directory);
        let extensions = matches.get_one::<String>("extensions");

        if let Ok(files) = get_files_in_directory(path, extensions.map(|s| s.as_str()), recursive, &exclude_paths) {
            spinner.set_length(files.len() as u64);

            // Process each file found in the directory
            for file in files {
                process_file(
                    &file, 
                    dry_run, 
                    verbose, 
                    &mut processed_files, 
                    recursive, 
                    &mut total_files_processed, 
                    &mut total_statements_removed,
                    Some(&spinner)
                );
                spinner.inc(1);
            }
            spinner.finish_with_message("Processing complete".bright_green().bold().to_string());
        } else {
            eprintln!("Failed to read directory: {}", directory.red().bold());
        }
    }

    // Display summary of the processing
    let duration = start_time.elapsed();
    spinner.finish_and_clear();
    println!("\n{}", "Summary".bold().underline());
    println!(
        "Total files processed: {}",
        total_files_processed.to_string().cyan().bold()
    );
    println!(
        "Total debug statements removed: {}",
        total_statements_removed.to_string().cyan().bold()
    );
    println!(
        "Time taken: {}",
        format!("{:.2?}", duration).cyan().bold()
    );

    if dry_run {
        println!("{}", "This was a dry run, no files were modified.".yellow().bold());
    }

    if verbose {
        if let Some(directory) = matches.get_one::<String>("directory") {
            println!(
                "Processed directory: {}",
                directory.bright_yellow().bold()
            );
        }
        if let Some(file) = matches.get_one::<String>("file") {
            println!("Processed file: {}", file.bright_yellow().bold());
        }
    }
}

fn process_file(
    path: &Path, 
    dry_run: bool, 
    verbose: bool, 
    processed_files: &mut HashSet<PathBuf>, 
    recursive: bool,
    total_files_processed: &mut usize, 
    total_statements_removed: &mut usize,
    pb: Option<&ProgressBar>  // Option to handle ProgressBar reference
) {
    // Check if the file has already been processed
    if !processed_files.insert(path.to_path_buf()) {
        if verbose {
            // Clear the spinner, print the message, and restart the spinner
            if let Some(spinner) = pb {
                spinner.println(format!(
                    "File already processed: {}",
                    path.display().to_string().yellow()
                ));
            } else {
                println!("File already processed: {}", path.display().to_string().yellow());
            }
        }
        return;
    }

    // Create a spinner if one isn't passed in
    let spinner: Box<ProgressBar> = pb.map(|p| Box::new(p.clone())).unwrap_or_else(|| {
        let sp = ProgressBar::new_spinner();
        sp.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .tick_chars("/|\\- ")
        );
        sp.enable_steady_tick(100);
        Box::new(sp)
    });

    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        spinner.set_message(format!("Processing file: {}", path.display()));

        // Read the file content
        if let Ok(content) = fs::read_to_string(path) {
            // Remove debug statements from the content
            let cleaned_content = remove_debug_statements(&content, extension);
            let removed_statements = content.lines().count() - cleaned_content.lines().count();

            *total_files_processed += 1;
            *total_statements_removed += removed_statements;

            // Print verbose message above the spinner
            if verbose {
                spinner.println(format!(
                    "{} - {}",
                    path.display(),
                    removed_statements
                ));
            }

            if dry_run {
                // Continue to process imports even in dry run mode
                if recursive && (extension == "js" || extension == "py" || extension == "ts") {
                    let imports = extract_imports(&content, path.parent().unwrap_or(Path::new("")), extension);

                    for import in imports {
                        process_file(
                            &import, 
                            dry_run, 
                            verbose, 
                            processed_files, 
                            recursive, 
                            total_files_processed, 
                            total_statements_removed,
                            Some(&spinner)  // Pass the spinner reference to recursive calls
                        );
                    }
                }
            } else {
                // Create a backup and write the cleaned content to the file
                let backup_path = path.with_extension("bak");
                if let Err(e) = fs::write(&backup_path, &content) {
                    spinner.println(format!("Failed to create backup file: {}", e));
                    spinner.finish_and_clear();
                    return;
                }

                if let Err(e) = fs::write(path, cleaned_content) {
                    spinner.println(format!("Failed to write cleaned content to file: {}", e));
                    spinner.finish_and_clear();
                    return;
                }

                // Process imports as usual if recursive flag is set
                if recursive && (extension == "js" || extension == "py" || extension == "ts") {
                    let imports = extract_imports(&content, path.parent().unwrap_or(Path::new("")), extension);

                    for import in imports {
                        process_file(
                            &import, 
                            dry_run, 
                            verbose, 
                            processed_files, 
                            recursive, 
                            total_files_processed, 
                            total_statements_removed,
                            Some(&spinner)  // Pass the spinner reference to recursive calls
                        );
                    }
                }
            }

            if pb.is_none() {
                spinner.finish_and_clear(); // Finish the spinner only if it was created locally
            }
        } else {
            spinner.println(format!("Failed to read file: {}", path.display().to_string().red()));
            if pb.is_none() {
                spinner.finish_and_clear(); // Finish the spinner only if it was created locally
            }
            return;
        }
    } else {
        spinner.println(format!("Skipping file with no extension: {}", path.display().to_string().red()));
        if pb.is_none() {
            spinner.finish_and_clear(); // Finish the spinner only if it was created locally
        }
        return;
    }
}
