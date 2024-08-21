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

    // Process a single file if specified
    if let Some(file) = matches.get_one::<String>("file") {
        process_file(
            Path::new(file), 
            dry_run, 
            verbose, 
            &mut processed_files, 
            recursive, 
            &mut total_files_processed, 
            &mut total_statements_removed
        );
    }

    // Process all files in a directory if specified
    if let Some(directory) = matches.get_one::<String>("directory") {
        let path = Path::new(directory);
        let extensions = matches.get_one::<String>("extensions");

        if let Ok(files) = get_files_in_directory(path, extensions.map(|s| s.as_str()), recursive, &exclude_paths) {
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .progress_chars("##>-")
            );

            // Process each file found in the directory
            for file in files {
                process_file(
                    &file, 
                    dry_run, 
                    verbose, 
                    &mut processed_files, 
                    recursive, 
                    &mut total_files_processed, 
                    &mut total_statements_removed
                );
                pb.inc(1);
            }
            pb.finish_with_message("Processing complete".bright_green().bold().to_string());
        } else {
            eprintln!("Failed to read directory: {}", directory.red().bold());
        }
    }

    // Display summary of the processing
    let duration = start_time.elapsed();
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
    total_files_processed: &mut usize,  // Passed by mutable reference
    total_statements_removed: &mut usize,  // Passed by mutable reference
) {
    // Check if the file has already been processed
    if !processed_files.insert(path.to_path_buf()) {
        if verbose {
            println!("File already processed: {}", path.display().to_string().yellow());
        }
        return;
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_draw_target(ProgressDrawTarget::stdout());
    spinner.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}"));
    spinner.enable_steady_tick(100);

    spinner.set_message(format!("Processing file: {}", path.display()));

    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        // Read the file content
        if let Ok(content) = fs::read_to_string(path) {
            // Remove debug statements from the content
            let cleaned_content = remove_debug_statements(&content, extension);
            let removed_statements = content.lines().count() - cleaned_content.lines().count();

            *total_files_processed += 1;
            *total_statements_removed += removed_statements;

            if dry_run {
                // Set spinner message for dry run
                if verbose {
                    spinner.set_message(format!(
                        "Dry run for file: {}",
                        path.display().to_string().green(),
                    ));
                }
                // Continue to process imports even in dry run mode
                if recursive && (extension == "js" || extension == "py") {
                    let imports = extract_imports(&content, path.parent().unwrap_or(Path::new("")),extension);
                    
                    // Set spinner message for imports
                    if verbose {
                        spinner.set_message(format!("Extracted imports from {}: {:?}", path.display(), imports));
                    }

                    for import in imports {
                        process_file(
                            &import, 
                            dry_run, 
                            verbose, 
                            processed_files, 
                            recursive, 
                            total_files_processed, 
                            total_statements_removed
                        );
                    }
                }
                spinner.finish_and_clear();
                return;
            } else {
                // Create a backup and write the cleaned content to the file
                let backup_path = path.with_extension("bak");
                if let Err(e) = fs::write(&backup_path, &content) {
                    eprintln!("Failed to create backup file: {}", e);
                    spinner.finish_and_clear();
                    return;
                }

                if let Err(e) = fs::write(path, cleaned_content) {
                    eprintln!("Failed to write cleaned content to file: {}", e);
                    spinner.finish_and_clear();
                    return;
                }

                if verbose {
                    spinner.set_message(format!("Processed and updated file: {}", path.display().to_string().green()));
                }
                spinner.finish_and_clear();

                // Process imports as usual if recursive flag is set
                if recursive && (extension == "js" || extension == "py") {
                    let imports = extract_imports(&content, path.parent().unwrap_or(Path::new("")),extension);
                    
                    // Set spinner message for imports
                    if verbose {
                        spinner.set_message(format!("Extracted imports from {}: {:?}", path.display(), imports));
                    }

                    for import in imports {
                        process_file(
                            &import, 
                            dry_run, 
                            verbose, 
                            processed_files, 
                            recursive, 
                            total_files_processed, 
                            total_statements_removed
                        );
                    }
                }

                return;
            }
        } else {
            eprintln!("Failed to read file: {}", path.display().to_string().red());
            spinner.finish_and_clear();
            return;
        }
    } else {
        eprintln!("Skipping file with no extension: {}", path.display().to_string().red());
        spinner.finish_and_clear();
        return;
    }
}
