use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar};
use colored::Colorize;

pub fn create_backup(content: &str, path: &Path, pb: Option<&ProgressBar>) -> Result<(), std::io::Error> {
    // Create a unique identifier for the backup
    let unique_id = uuid::Uuid::new_v4().to_string();

    // Create a signature for the backup file
    let signature = format!(
        "/* Backup created by CleanCode tool - {}\n * Original file: {}\n * Original extension: {}\n * Unique ID: {}\n * Timestamp: {}\n */\n",
        env!("CARGO_PKG_VERSION"),
        path.display(),
        path.extension().unwrap_or_default().to_string_lossy(),
        unique_id,
        chrono::Local::now().to_rfc3339()
    );

    // Combine the signature with the original content without extra newlines
    let backup_content = format!("{}{}", signature, content.trim_end());

    // Define the backup path
    let backup_path = path.with_extension("bak");

    // Write the backup file
    if let Err(e) = fs::write(&backup_path, &backup_content) {
        if let Some(spinner) = pb {
            spinner.println(format!("Failed to create backup file: {}", e));
        } else {
            eprintln!("Failed to create backup file: {}", e);
        }
        return Err(e);
    }

    // Log backup creation
    if let Some(spinner) = pb {
        spinner.println(format!("Backup created for file: {}", path.display().to_string().green()));
    } else {
        println!("Backup created for file: {}", path.display().to_string().green());
    }

    Ok(())
}

pub fn undo_removal(path: &Path, force: bool, dry_run: bool, pb: Option<&ProgressBar>) -> Result<(), std::io::Error> {
    let backup_path = path.with_extension("bak");

    if !backup_path.exists() {
        if let Some(spinner) = pb {
            spinner.println(format!("No backup found for file: {}", path.display().to_string().red()));
        } else {
            eprintln!("No backup found for file: {}", path.display().to_string().red());
        }
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Backup not found"));
    }

    let content = fs::read_to_string(&backup_path)?;

    let signature_start = content.find("/* Backup created by CleanCode tool");
    if let Some(start) = signature_start {
        let signature_end = content.find("*/").unwrap_or(0) + 2;
        let signature = &content[start..signature_end];

        let original_extension = signature
            .lines()
            .find(|line| line.contains("Original extension:"))
            .map(|line| line.split(": ").nth(1).unwrap_or("").trim())
            .unwrap_or("");

        let restored_path = path.with_extension(original_extension);

        if restored_path.exists() && !force && !dry_run {
            if let Some(spinner) = pb {
                spinner.println(format!(
                    "File already exists: {}. Use --force to override.",
                    restored_path.display().to_string().yellow()
                ));
            } else {
                eprintln!("File already exists: {}. Use --force to override.", restored_path.display().to_string().yellow());
            }
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "File already exists"));
        }

        if dry_run {
            if let Some(spinner) = pb {
                spinner.println(format!(
                    "[Dry Run] Would restore file: {}",
                    restored_path.display().to_string().green()
                ));
            } else {
                println!("[Dry Run] Would restore file: {}", restored_path.display().to_string().green());
            }
            return Ok(());
        }

        // Restore the file without including the signature
        fs::write(&restored_path, content[signature_end..].trim_start())?;
        fs::remove_file(&backup_path)?;

        if let Some(spinner) = pb {
            spinner.println(format!("Restored file: {}", restored_path.display().to_string().green()));
        } else {
            println!("Restored file: {}", restored_path.display().to_string().green());
        }
        Ok(())
    } else {
        if let Some(spinner) = pb {
            spinner.println(format!("Invalid backup signature for file: {}", path.display().to_string().red()));
        } else {
            eprintln!("Invalid backup signature for file: {}", path.display().to_string().red());
        }
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid backup signature"))
    }
}


pub fn get_files_in_directory(
    dir: &Path,
    extensions: Option<&str>,
    recursive: bool,
    exclude_paths: &[PathBuf], 
) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if exclude_paths.iter().any(|p| path.starts_with(p)) {
            continue;
        }

        if path.is_dir() && recursive {
            files.extend(get_files_in_directory(&path, extensions, recursive, exclude_paths)?);
        } else if let Some(ext) = path.extension() {
            if let Some(exts) = extensions {
                if exts.split(',').any(|e| e == ext.to_str().unwrap_or("")) {
                    files.push(path);
                }
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}

pub fn handle_update_for_file(path: &Path, force: bool, dry_run: bool, pb: Option<&ProgressBar>) {
    if path.extension() == Some(std::ffi::OsStr::new("bak")) {
        // If the file itself is a .bak file, restore it
        undo_removal(path, force, dry_run, pb).unwrap_or_else(|err| {
            if let Some(spinner) = pb {
                spinner.println(format!("Failed to restore file: {}: {}", path.display(), err));
            } else {
                eprintln!("Failed to restore file: {}: {}", path.display(), err);
            }
        });
    } else {
        // Look for a corresponding .bak file with the same name
        let backup_path = path.with_extension("bak");
        if backup_path.exists() {
            undo_removal(&backup_path, force, dry_run, pb).unwrap_or_else(|err| {
                if let Some(spinner) = pb {
                    spinner.println(format!("Failed to restore file: {}: {}", backup_path.display(), err));
                } else {
                    eprintln!("Failed to restore file: {}: {}", backup_path.display(), err);
                }
            });
        } else {
            if let Some(spinner) = pb {
                spinner.println(format!("No backup found for file: {}", path.display().to_string().red()));
            } else {
                eprintln!("No backup found for file: {}", path.display().to_string().red());
            }
        }
    }
}
