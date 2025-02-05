use clap::Parser;
use env_logger;
use log::{error, info, warn};
use std::path::PathBuf;
use std::process;
use sysmon_json::{
    convert_file, 
    BatchProcessor,
    ProcessingOptions,
    ProcessingOptionsBuilder,
    merger::merge_configs,
    error::{ConversionError, PreprocessError},
    preprocessor::preprocess_config,
};
use sysmon_json::batch::ProgressReporter;

/// CLI tool for converting Sysmon configurations between XML and JSON formats
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input file or directory path
    #[arg(short, long, value_parser = clap::value_parser!(PathBuf))]
    input: PathBuf,

    /// Output file or directory path
    #[arg(short, long, value_parser = clap::value_parser!(PathBuf))]
    output: Option<PathBuf>,

    /// Process directories recursively
    #[arg(short, long)]
    recursive: bool,

    /// Process input as a directory containing multiple files
    #[arg(short, long)]
    batch: bool,

    /// Merge all Sysmon configs in the input directory into a single file
    #[arg(short, long)]
    merge: bool,

    /// Maximum file size in MB
    #[arg(long, default_value = "10")]
    max_size: u64,

    /// Maximum recursion depth
    #[arg(long, default_value = "10")]
    max_depth: u32,

    /// Number of worker threads (default: number of CPU cores)
    #[arg(long)]
    workers: Option<usize>,

    /// Verify output after conversion
    #[arg(long)]
    verify: bool,

    /// Suppress progress output
    #[arg(long)]
    silent: bool,

    /// Create backups of existing files
    #[arg(long)]
    backup: bool,

    /// Pattern to ignore (can be specified multiple times)
    #[arg(long = "ignore")]
    ignore_patterns: Vec<String>,

    #[arg(long)]
    skip_preprocessing: bool,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Err(e) = try_main() {
        error!("Error: {}", e);
        process::exit(1);
    }
}

fn try_main() -> Result<(), ConversionError> {
    let cli = Cli::parse();

    let options = ProcessingOptionsBuilder::new()
        .max_file_size(cli.max_size * 1024 * 1024)
        .max_depth(cli.max_depth)
        .workers(cli.workers)
        .verify_output(cli.verify)
        .silent(cli.silent)
        .create_backup(cli.backup)
        .ignore_patterns(if cli.ignore_patterns.is_empty() {
            None
        } else {
            Some(cli.ignore_patterns.clone())
        })
        .build();

    if !cli.input.exists() {
        return Err(ConversionError::InvalidFile(format!(
            "Input path does not exist: {}",
            cli.input.display()
        )));
    }

    if cli.merge {
        handle_merge_mode(&cli)?;
        return Ok(());
    }

    if cli.batch || cli.input.is_dir() {
        handle_batch_mode(&cli, &options)?;
        return Ok(());
    }

    handle_single_file(&cli, &options)?;
    Ok(())
}

fn handle_merge_mode(cli: &Cli) -> Result<(), ConversionError> {
    if !cli.input.is_dir() {
        return Err(ConversionError::InvalidFile(
            "Merge mode requires input to be a directory".to_string(),
        ));
    }

    let output_path = cli
        .output
        .clone()
        .unwrap_or_else(|| cli.input.join("merged.xml"));

    info!(
        "Merging configs from {} to {}",
        cli.input.display(),
        output_path.display()
    );

    merge_configs(&cli.input, &output_path, cli.recursive)?;
    info!("Merge completed successfully");

    Ok(())
}

fn handle_batch_mode(cli: &Cli, options: &ProcessingOptions) -> Result<(), ConversionError> {
    if !cli.input.is_dir() {
        return Err(ConversionError::InvalidFile(
            "Batch mode requires input to be a directory".to_string(),
        ));
    }

    let output_dir = cli.output.clone().unwrap_or_else(|| {
        let mut out = cli.input.clone();
        out.set_file_name(format!(
            "{}_converted",
            cli.input
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output")
        ));
        out
    });

    info!("Processing directory: {}", cli.input.display());
    info!("Output directory: {}", output_dir.display());

    let processor = BatchProcessor::new();
    let stats = if !cli.silent {
        process_with_progress(&cli.input, &output_dir, cli.recursive, options, &processor)?
    } else {
        processor.process_directory(&cli.input, &output_dir, cli.recursive, options)?
    };

    if stats.errors > 0 {
        warn!("Some files failed to process. Check the log for details.");
    }

    Ok(())
}

fn process_with_progress(
    input: &PathBuf,
    output: &PathBuf,
    recursive: bool,
    options: &ProcessingOptions,
    processor: &BatchProcessor,
) -> Result<sysmon_json::batch::BatchProcessingStats, ConversionError> {
    let total_files = walkdir::WalkDir::new(input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    let progress = ProgressReporter::new(total_files);
    let stats = processor.process_directory_with_progress(
        input,
        output,
        recursive,
        options,
        &progress,
    )?;
    Ok(stats)
}

fn handle_single_file(cli: &Cli, options: &ProcessingOptions) -> Result<(), ConversionError> {
    let output_path = cli.output.clone().unwrap_or_else(|| {
        let mut out = cli.input.clone();
        let new_ext = if cli.input.extension().and_then(|e| e.to_str()) == Some("xml") {
            "json"
        } else {
            "xml"
        };
        out.set_extension(new_ext);
        out
    });

    if options.create_backup && output_path.exists() {
        let backup_path = output_path.with_extension("bak");
        info!("Creating backup: {}", backup_path.display());
        std::fs::copy(&output_path, &backup_path)
            .map_err(|e| ConversionError::io_error(&output_path, e))?;
    }

    if !cli.skip_preprocessing {
        info!("Preprocessing configuration file...");
        match preprocess_config(&cli.input) {
            Ok(processed_content) => {
                let temp_dir = tempfile::tempdir()
                    .map_err(|e| ConversionError::io_error(&cli.input, e))?;
                let temp_path = temp_dir.path().join(cli.input.file_name().unwrap());
                
                std::fs::write(&temp_path, &processed_content)
                    .map_err(|e| ConversionError::io_error(&temp_path, e))?;

                info!(
                    "Converting {} to {}",
                    cli.input.display(),
                    output_path.display()
                );

                match convert_file(&temp_path, &output_path) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Conversion failed after preprocessing: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                match e {
                    PreprocessError::IoError(e) => {
                        error!("IO error during preprocessing: {}", e);
                        return Err(ConversionError::io_error(&cli.input, e));
                    }
                    PreprocessError::XmlError(e) => {
                        error!("XML parsing error during preprocessing: {}", e);
                        return Err(ConversionError::XmlParse(e.into()));
                    }
                    PreprocessError::ValidationError(e) => {
                        error!("Validation error during preprocessing: {}", e);
                        return Err(ConversionError::ValidationError(e.to_string()));
                    }
                    PreprocessError::PathError(e) => {
                        error!("Path error during preprocessing: {}", e);
                        return Err(ConversionError::InvalidFile(e));
                    }
                    PreprocessError::ParserError(e) => {
                        error!("Parser error during preprocessing: {}", e);
                        return Err(ConversionError::ParserError(e.to_string()));
                    }
                }
            }
        }
    } else {
        info!(
            "Converting {} to {}",
            cli.input.display(),
            output_path.display()
        );
        convert_file(&cli.input, &output_path)?;
    }

    info!("Conversion completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_single_file_conversion() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("test.xml");
        let output_path = temp_dir.path().join("test.json");

        fs::write(&input_path, "<root><test>value</test></root>").unwrap();

        let result = convert_file(&input_path, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_batch_processing() {
        let temp_dir = tempdir().unwrap();
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");

        fs::create_dir(&input_dir).unwrap();

        fs::write(
            input_dir.join("test1.xml"),
            "<root><test>value1</test></root>",
        )
        .unwrap();
        fs::write(
            input_dir.join("test2.xml"),
            "<root><test>value2</test></root>",
        )
        .unwrap();

        let processor = BatchProcessor::new();
        let options = ProcessingOptions::default();

        let result = processor.process_directory(&input_dir, &output_dir, false, &options);
        assert!(result.is_ok());
        assert!(output_dir.join("test1.json").exists());
        assert!(output_dir.join("test2.json").exists());
    }
}