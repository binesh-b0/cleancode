use clap::{Arg, Command};

fn main() {
    let matches = Command::new("CleanCode")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("CLI tool to remove debug/console statements from code files")
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .num_args(1)
            .help("Specifies a single file to process"))
        .arg(Arg::new("directory")
            .short('d')
            .long("directory")
            .num_args(1)
            .help("Specifies a directory to process all applicable files within"))
        .arg(Arg::new("recursive")
            .short('r')
            .long("recursive")
            .num_args(0)
            .help("Process files recursively in directories"))
        .arg(Arg::new("extensions")
            .short('e')
            .long("extensions")
            .num_args(1)
            .help("File extensions to target, e.g., 'js,py'"))
        .arg(Arg::new("remove")
            .long("remove")
            .num_args(0) // This ensures no value is expected
            .help("Remove console/print statements"))
        .arg(Arg::new("dry_run")
            .short('n')
            .long("dry-run")
            .num_args(0) // This ensures no value is expected
            .help("Simulate the operation without making any changes"))
        .get_matches();

    // Example of handling arguments
    if let Some(file) = matches.get_one::<String>("file") {
        println!("File to process: {}", file);
    }

    if let Some(directory) = matches.get_one::<String>("directory") {
        println!("Directory to process: {}", directory);
    }

    if matches.contains_id("recursive") {
        println!("Processing recursively...");
    }

    if let Some(exts) = matches.get_one::<String>("extensions") {
        println!("Target extensions: {}", exts);
    }

    if matches.contains_id("remove") {
        println!("Removing debug/console statements...");
    }

    if matches.contains_id("dry_run") {
        println!("Dry run mode enabled...");
    }
}
