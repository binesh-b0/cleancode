use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

pub fn read_file_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect()
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