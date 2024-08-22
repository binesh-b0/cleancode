mod cli;
mod file_ops;
mod parser;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use cli::{build_cli, default_exclusions};
use file_ops::{get_files_in_directory,create_backup,undo_removal,handle_update_for_file};
use parser::{remove_debug_statements, extract_imports};
use colored::Colorize;

fn main() {
    let matches = build_cli().get_matches();
    let start_time = Instant::now();

    // Parse command line arguments
    let dry_run = matches.get_flag("dry_run");
    let verbose = matches.get_flag("verbose");
    let recursive = matches.get_flag("recursive");
    let update = matches.get_flag("update");
    let force = matches.get_flag("force");

    // Check that both directory and file are not provided simultaneously
    if matches.get_one::<String>("file").is_some() && matches.get_one::<String>("directory").is_some() {
        eprintln!("{}", "Error: You cannot specify both a file and a directory at the same time.".red().bold());
        return;
    }
    if matches.get_one::<String>("file").is_some() && recursive && matches.get_one::<String>("update").is_some(){
        eprintln!("{}", "Error: Recursively updating file is not available, try --directory .".red().bold());
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

    if update {
        if let Some(file) = matches.get_one::<String>("file") {
            let path = Path::new(file);
            handle_update_for_file(path, force, dry_run, Some(&spinner));
        }

        if let Some(directory) = matches.get_one::<String>("directory") {
            let path = Path::new(directory);
            if let Ok(files) = get_files_in_directory(path, Some("bak"), recursive, &exclude_paths) {
                spinner.set_length(files.len() as u64);
                for file in files {
                    undo_removal(&file, force, dry_run, Some(&spinner)).unwrap_or_else(|err| {
                        spinner.println(format!("Failed to restore file: {}: {}", file.display(), err));
                    });
                    spinner.inc(1);
                }
                spinner.finish_with_message("Update complete".bright_green().bold().to_string());
            } else {
                eprintln!("Failed to read directory: {}", directory.red().bold());
            }
        }
        return;
    }

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

        if let Ok(content) = fs::read_to_string(path) {
            let cleaned_content = remove_debug_statements(&content, extension);
            let removed_statements = content.lines().count() - cleaned_content.lines().count();

            *total_files_processed += 1;
            *total_statements_removed += removed_statements;

            if verbose {
                spinner.println(format!(
                    "{} - {}",
                    path.display(),
                    removed_statements
                ));
            }

            if dry_run {
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
                            Some(&spinner)
                        );
                    }
                }
            } else {
                create_backup(&content, path, Some(&spinner)).unwrap();  // Create the backup with spinner

                if let Err(e) = fs::write(path, cleaned_content) {
                    spinner.println(format!("Failed to write cleaned content to file: {}", e));
                    spinner.finish_and_clear();
                    return;
                }

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
                            Some(&spinner)
                        );
                    }
                }
            }

            if pb.is_none() {
                spinner.finish_and_clear();
            }
        } else {
            spinner.println(format!("Failed to read file: {}", path.display().to_string().red()));
            if pb.is_none() {
                spinner.finish_and_clear();
            }
            return;
        }
    } else {
        spinner.println(format!("Skipping file with no extension: {}", path.display().to_string().red()));
        if pb.is_none() {
            spinner.finish_and_clear();
        }
        return;
    }
}
