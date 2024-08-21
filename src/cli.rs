use clap::{Arg, Command};
use std::path::PathBuf;

pub fn build_cli() -> Command {
    Command::new("CleanCode")
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
        .arg(Arg::new("exclude")
            .long("exclude")
            .short('x')
            .num_args(1..)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Exclude specific files or directories"))
        .arg(Arg::new("remove")
            .long("remove")
            .num_args(0)
            .help("Remove console/print statements"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .num_args(0)
            .help("Show detailed logs during processing"))
        .arg(Arg::new("dry_run")
            .short('n')
            .long("dry-run")
            .num_args(0)
            .help("Simulate the operation without making any changes"))
}

pub fn default_exclusions() -> Vec<PathBuf> {
    vec![
        PathBuf::from("node_modules"),
        PathBuf::from("target"),
        PathBuf::from("vendor"),
    ]
}
