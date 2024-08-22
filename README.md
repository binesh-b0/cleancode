# CleanCode

**CleanCode** is a command-line tool designed to automate the removal of debug statements from your code files. This tool helps you clean up your projects by removing console logs, print statements, and other debug-related code before pushing to production.

## Features

- **Multi-language Support**: Automatically processes JavaScript, and Python files.
- **Recursive File Processing**: Traverse through directories and process imported modules recursively.
- **Dry Run Mode**: Preview the changes without modifying any files.
- **Verbose Mode**: Get detailed logs to see which files and statements are being processed.
- **Exclusion of Library Directories**: Automatically skips common library directories like `node_modules`, `target`, and `vendor`.
- **Customizable Exclusions**: Specify additional files or directories to exclude from processing.

## Installation

To install CleanCode, clone the repository and build the project using Rust's cargo:

```bash
git clone git@github.com:binesh-b0/cleancode.git
cd cleancode
cargo build --release
```

## Usage

### Basic Commands

- **Remove debug statements from a single file**:
  ```bash
  cleancode --file path/to/your/file.js --remove
  ```

- **Process all files in a directory recursively**:
  ```bash
  cleancode --directory path/to/your/project --recursive --remove
  ```

- **Dry run to see what changes will be made**:
  ```bash
  cleancode --file path/to/your/file.py --dry-run --verbose
  ```

### Command Line Options

- **`-f, --file <FILE>`**: Specifies a single file to process.
- **`-d, --directory <DIRECTORY>`**: Specifies a directory to process all applicable files within.
- **`-r, --recursive`**: Process files recursively in directories.
- **`-e, --extensions <EXTENSIONS>`**: Target specific file extensions, e.g., `'js,py'`.
- **`-x, --exclude <PATHS>`**: Exclude specific files or directories.
- **`--remove`**: Remove console/print statements.
- **`-v, --verbose`**: Show detailed logs during processing.
- **`-n, --dry-run`**: Simulate the operation without making any changes.

## Examples

- **Process a specific JavaScript file and remove console statements**:
  ```bash
  cleancode --file app.js --remove
  ```

- **Process a directory recursively and remove print statements from Python files**:
  ```bash
  cleancode --directory src --recursive --extensions "py" --remove
  ```

- **Dry run mode to see what will be removed without actually modifying files**:
  ```bash
  cleancode --file main.py --dry-run --verbose
  ```

- **Exclude `node_modules` and another custom directory**:
  ```bash
  cleancode --directory src --exclude node_modules custom_dir --remove
  ```

## Default Exclusions

The following directories are excluded by default:

- `node_modules`
- `target`
- `vendor`

You can customize exclusions using the `--exclude` option.

## Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests if you have suggestions, improvements, or fixes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.


