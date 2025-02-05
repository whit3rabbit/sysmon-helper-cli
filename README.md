# Sysmon Helper CLI

A command-line tool for converting, merging, and managing Sysmon configuration files between XML and JSON formats.

## Features

- Convert between XML and JSON formats
- Batch processing of multiple files
- Merge multiple Sysmon configurations
- Progress tracking for batch operations
- File preprocessing and validation
- Configurable backup creation
- Recursive directory processing

## Prerequisites

- Rust (latest stable version)
- Git

## Installation

### From Source

1. Clone the repository:
```bash
git clone https://github.com/whit3rabbit/sysmon-helper-cli.git
cd sysmon-helper-cli
```

2. Initialize submodules:
```bash
git submodule init
git submodule update
```

3. Build the project:
```bash
cargo build --release
```

The binary will be available at `target/release/sysmon_cli`

## Usage

### Basic Conversion

Convert a single file between XML and JSON:

```bash
# XML to JSON
sysmon_cli -i config.xml -o output.json

# JSON to XML
sysmon_cli -i config.json -o output.xml

# Automatic output filename
sysmon_cli -i config.xml
```

### Batch Processing

Process multiple files in a directory:

```bash
# Convert all files in a directory
sysmon_cli -i input_dir -o output_dir --batch

# Process directory recursively
sysmon_cli -i input_dir -o output_dir --batch --recursive

# With automatic backup creation
sysmon_cli -i input_dir -o output_dir --batch --backup
```

### Configuration Merging

Merge multiple Sysmon configurations:

```bash
# Merge configs with default output
sysmon_cli -i configs/ --merge

# Merge with custom output
sysmon_cli -i configs/ -o combined.xml --merge

# Merge recursively with verification
sysmon_cli -i configs/ -o combined.xml --merge --recursive --verify
```

## Options

```bash
Options:
  -i, --input <PATH>           Input file or directory path
  -o, --output <PATH>          Output file or directory path [optional]
  -r, --recursive              Process directories recursively
  -b, --batch                  Process input as a directory containing multiple files
  -m, --merge                  Merge all Sysmon configs in the input directory
      --max-size <MB>          Maximum file size in MB [default: 10]
      --max-depth <DEPTH>      Maximum recursion depth [default: 10]
      --workers <NUM>          Number of worker threads (default: CPU cores)
      --verify                 Verify output after conversion
      --silent                 Suppress progress output
      --backup                 Create backups of existing files
      --ignore <PATTERN>       Pattern to ignore (can be specified multiple times)
      --skip-preprocessing     Skip preprocessing phase
  -h, --help                   Print help
  -V, --version                Print version
```

## Environment Variables

The tool uses env_logger for logging. Control log levels using:

```bash
# Set logging level
export RUST_LOG=info    # Default
export RUST_LOG=debug   # More detailed logging
export RUST_LOG=warn    # Warnings and errors only
```

## Testing

### Setting up Test Fixtures

This project uses git submodules for test fixtures. The Sysmon Modular configurations are included as a submodule under `tests/fixtures`. To properly set up the test environment, follow these steps:

```bash
# If cloning the repository for the first time
git clone --recursive https://github.com/yourusername/sysmon_json.git

# If you've already cloned the repository
git submodule init
git submodule update
```

### Updating Test Fixtures

To update the test fixtures to their latest version:

```bash
git submodule update --remote tests/fixtures
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests with parallel execution
cargo test -- --test-threads=num_threads
```

### Test Coverage

For generating test coverage reports, you'll need to install cargo-tarpaulin:

```bash
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

The coverage report will be available in `tarpaulin-report.html`.

### Troubleshooting Tests

If you encounter issues with the test fixtures:

1. Ensure all submodules are properly initialized:
   ```bash
   git submodule status
   ```

2. Reset the submodule if needed:
   ```bash
   git submodule deinit -f tests/fixtures
   git submodule update --init
   ```

3. Check if the test fixtures are up to date:
   ```bash
   cd tests/fixtures
   git fetch
   git status
   ```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
