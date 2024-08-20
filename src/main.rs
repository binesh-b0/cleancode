mod cli;
mod file_ops;
mod parser;

use std::fs;
use std::path::Path;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use cli::build_cli;
use file_ops::get_files_in_directory;
use parser::remove_debug_statements;
use colored::*;

fn main() {
    let matches = build_cli().get_matches();
    let start_time = Instant::now();

    let dry_run = matches.get_flag("dry_run");
    let verbose = matches.get_flag("verbose");

    let mut total_files_processed = 0;
    let mut total_statements_removed = 0;

    if let Some(file) = matches.get_one::<String>("file") {
        let (processed, removed) = process_file(Path::new(file), dry_run, verbose);
        total_files_processed += processed;
        total_statements_removed += removed;
    }

    if let Some(directory) = matches.get_one::<String>("directory") {
        let path = Path::new(directory);
        let extensions = matches.get_one::<String>("extensions");
        let recursive = matches.get_flag("recursive");

        if let Ok(files) = get_files_in_directory(path, extensions.map(|s| s.as_str()), recursive) {
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .progress_chars("##>-")
            );

            for file in files {
                let (processed, removed) = process_file(&file, dry_run, verbose);
                total_files_processed += processed;
                total_statements_removed += removed;
                pb.inc(1);
            }
            pb.finish_with_message("Processing complete".bright_green().bold().to_string());
        } else {
            eprintln!("Failed to read directory: {}", directory.red().bold());
        }
    }

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

fn process_file(path: &Path, dry_run: bool, verbose: bool) -> (usize, usize) {
    let spinner = ProgressBar::new_spinner();
    spinner.set_draw_target(ProgressDrawTarget::stdout());
    spinner.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} Processing {wide_msg}")
    );
    spinner.enable_steady_tick(100);

    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        if let Ok(content) = fs::read_to_string(path) {
            spinner.set_message(format!("{}", path.display()));

            let cleaned_content = remove_debug_statements(&content, extension);
            let removed_statements = content.lines().count() - cleaned_content.lines().count();

            if dry_run {
                if verbose {
                    println!(
                        "Dry run for file: {}\nCleaned Content:\n{}",
                        path.display().to_string().green(),
                        cleaned_content
                    );
                }
                spinner.finish_and_clear();
                return (1, removed_statements);
            } else {
                let backup_path = path.with_extension("bak");
                if let Err(e) = fs::write(&backup_path, &content) {
                    eprintln!("Failed to create backup file: {}", e);
                    spinner.finish_and_clear();
                    return (0, 0);
                }

                if let Err(e) = fs::write(path, cleaned_content) {
                    eprintln!("Failed to write cleaned content to file: {}", e);
                    spinner.finish_and_clear();
                    return (0, 0);
                }

                if verbose {
                    println!("Processed and updated file: {}", path.display().to_string().green());
                }
                spinner.finish_and_clear();
                return (1, removed_statements);
            }
        } else {
            eprintln!("Failed to read file: {}", path.display().to_string().red());
            spinner.finish_and_clear();
            return (0, 0);
        }
    } else {
        eprintln!("Skipping file with no extension: {}", path.display().to_string().red());
        spinner.finish_and_clear();
        return (0, 0);
    }
}
