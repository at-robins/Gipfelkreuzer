//! This module defines command line arguments.

use std::{borrow::Cow, fmt::Debug, path::PathBuf, time::SystemTime};

use clap::Parser;
use getset::{CopyGetters, Getters};
use log::LevelFilter;

/// A tool for creating consensus peaks from genomic peak data, such as ATAC- or ChIP-Seq data.
#[derive(Parser, CopyGetters, Getters, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The path to the GA4GH BED v1.0 complient input peak file.
    /// The peak summit offset from the start is expected at column 10.
    #[getset(get = "pub")]
    input_file: PathBuf,
    /// The output file path [default: the input file path with the suffix "_consensus_peaks.bed"]
    #[arg(short, long)]
    output_file: Option<PathBuf>,
    /// The logging level. Extensive logging might slow down software execution [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    #[getset(get_copy = "pub")]
    log_level: LevelFilter,
    /// The number of fields / columns to output. If 10 or more columns are specified,
    /// column 10 is filled with the consensus peak offset information [minimum to generate a valid BED file: 3]
    #[arg(short, long, default_value_t = 4)]
    #[getset(get_copy = "pub")]
    bed_output_columns: usize,
    /// The maximum number of consensus peak merging iterations.
    /// A value of "0" means consensus peaks are only called once and not
    /// iteratively merged. This will yield the highest sensitivity, but
    /// also poteintially result in multiple slight variations of the same peaks
    /// being present in the output.
    #[arg(short, long, default_value_t = 20)]
    #[getset(get_copy = "pub")]
    max_merge_iterations: usize,
}
impl CommandLineArguments {
    /// Returns the output directory.
    /// If no directory has been specified the parent directory of the input file is returned.
    pub fn output_file(&self) -> PathBuf {
        self.output_file
            .as_ref()
            .map(|output| output.to_path_buf())
            .unwrap_or_else(|| {
                // Uses the current system time as fallback for naming the output file.
                let current_system_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|a| a.as_secs())
                    .unwrap_or(0)
                    .to_string();
                // Tries to use the input file name as first fallback, then the system time.
                let input_file_name = self
                    .input_file()
                    .file_prefix()
                    .map(|name| name.to_string_lossy())
                    .unwrap_or(Cow::Borrowed(&current_system_time.as_str()));
                let mut output = self
                    .input_file()
                    .with_file_name(format!("{}_consensus_peaks", input_file_name));
                output.add_extension("bed");
                output
            })
    }
}
