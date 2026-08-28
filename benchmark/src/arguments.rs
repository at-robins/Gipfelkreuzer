//! This module defines command line arguments.

use std::{fmt::Debug, path::PathBuf, time::SystemTime};

use clap::Parser;
use getset::{CopyGetters, Getters};

/// A tool for benchmarking the Gipfelkreuzer software.
#[derive(Parser, CopyGetters, Getters, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The input BED files containing generated consensus peaks.
    #[arg(short = 'p', required = true)]
    #[getset(get = "pub")]
    input_file_peaks: Vec<PathBuf>,
    /// The input bigwig file containing the read distribution at single nucleotide resolution.
    #[arg(short = 'r', required = true)]
    #[getset(get = "pub")]
    input_file_counts: PathBuf,
    /// The output file path [default: the current system time and count input file with the suffix "_results.csv"]
    #[arg(short = 'o', long)]
    output_file: Option<PathBuf>,
}

impl CommandLineArguments {
    /// Returns the output file.
    /// If no file has been specified the current system time, input files and working directory are used
    /// as default output file name and directory, respectively.
    pub fn output_file(&self) -> PathBuf {
        self.output_file
            .as_ref()
            .map(|output| output.to_path_buf())
            .unwrap_or_else(|| {
                // Uses the current system time as fallback for naming the output file.
                let current_system_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|a| a.as_secs())
                    .unwrap_or(0);
                let count_name = self
                    .input_file_counts()
                    .file_stem()
                    .map(|filename| filename.to_string_lossy().to_string())
                    .unwrap_or("nofilename".to_string());
                let mut output =
                    PathBuf::from(format!("{}_{}_results", current_system_time, count_name));
                output.add_extension("csv");
                output
            })
    }
}
