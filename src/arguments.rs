//! This module defines command line arguments.

use std::{fmt::Debug, path::PathBuf, time::SystemTime};

use clap::Parser;
use getset::{CopyGetters, Getters};
use log::LevelFilter;

use crate::peaks::ConsensusPeakAlgorithm;

/// A tool for creating consensus peaks from genomic peak data, such as ATAC- or ChIP-Seq data.
#[derive(Parser, CopyGetters, Getters, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The paths to the GA4GH BED v1.0 complient BED3+ input peak files.
    /// The peak summit offset from the start is expected at column 10
    /// as defined in the narrowPeak file format definition.
    #[arg(required = true)]
    #[getset(get = "pub")]
    input_files: Vec<PathBuf>,
    /// The output file path [default: the input file path with the suffix "_consensus_peaks.bed"]
    #[arg(short, long)]
    output_file: Option<PathBuf>,
    /// The logging level. Extensive logging might slow down software execution [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    #[getset(get_copy = "pub")]
    log_level: LevelFilter,
    /// The number of fields / columns to output. If 10 or more columns are specified,
    /// column 10 is filled with the consensus peak coordinate [minimum to generate a valid BED file: 3]
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
    /// The algorithm to use for creating consensus peaks.
    #[arg(short, long, default_value_t = ConsensusPeakAlgorithm::Gipfelkreuzer)]
    #[getset(get_copy = "pub")]
    algorithm: ConsensusPeakAlgorithm,
}
impl CommandLineArguments {
    /// Returns the output file.
    /// If no file has been specified the current system time and working directory are used
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
                let mut output = PathBuf::from(format!("{}_consensus_peaks", current_system_time));
                output.add_extension("bed");
                output
            })
    }
}
