//! This module defines command line arguments.

use clap::Parser;
use getset::{CopyGetters, Getters};
use log::LevelFilter;

/// A tool for creating consensus peaks from genomic peak data, such as ATAC- or ChIP-Seq data.
#[derive(Parser, CopyGetters, Getters, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The logging level [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    #[getset(get_copy = "pub")]
    log_level: LevelFilter,
}
