use std::collections::HashMap;

use clap::Parser;

use crate::{
    arguments::CommandLineArguments,
    error::ApplicationError,
    input::bed_to_peaks,
    output::write_peaks_to_bed,
    peaks::{PeakData, PeakMerger},
};

fn main() -> Result<(), ApplicationError> {
    // Logs any uncatched errors.
    main_internal().map_err(|err| {
        err.log_default();
        err
    })
}

fn main_internal() -> Result<(), ApplicationError> {
    // Tries to parse the command line arguments.
    let cl_args_result = CommandLineArguments::try_parse().map_err(|err| {
        ApplicationError::from(err).chain("The command line arguments could not be parsed.")
    });
    // In case of an error sets a default log level to allow logging of the error.
    let log_level = cl_args_result
        .as_ref()
        .map(|cl_args| cl_args.log_level())
        .unwrap_or(log::LevelFilter::Warn);

    // Initialises the logger.
    env_logger::builder()
        .filter_level(log_level)
        .try_init()
        .map_err(|err| ApplicationError::from(err).chain("The logger could not be initialised."))?;

    let command_line_arguments = cl_args_result?;
    let peaks_by_chromosome = bed_to_peaks(command_line_arguments.input_file()).map_err(|err| {
        err.chain(format!(
            "Failed to parse input file \"{}\".",
            command_line_arguments.input_file().display()
        ))
    })?;
    let consenus: HashMap<String, Vec<PeakData>> = peaks_by_chromosome
        .into_iter()
        .map(|(chromosome, peaks)| (chromosome, PeakMerger::new(peaks).consensus_peaks()))
        .collect();
    write_peaks_to_bed(command_line_arguments.output_file(), &consenus).map_err(|err| {
        err.chain(format!(
            "Failed to write the consensus peaks to output file \"{}\".",
            command_line_arguments.input_file().display()
        ))
    })?;
    Ok(())
}

mod arguments;
mod error;
mod input;
mod output;
mod peaks;
