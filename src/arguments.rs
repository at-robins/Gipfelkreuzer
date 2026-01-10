//! This module defines command line arguments.

use std::path::PathBuf;

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
    /// The logging level [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    #[getset(get_copy = "pub")]
    log_level: LevelFilter,
}
