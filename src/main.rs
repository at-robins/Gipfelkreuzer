use std::collections::HashMap;

use clap::Parser;

use crate::{
    arguments::CommandLineArguments, error::ApplicationError, input::bed_to_peaks,
    output::write_peaks_to_bed, peaks::PeakData,
};

/// Runs the application.
fn main() -> Result<(), ApplicationError> {
    // Logs any uncatched errors.
    main_internal(CommandLineArguments::try_parse(), false).map_err(|err| {
        err.log_default();
        err
    })
}

/// An internal helper function to allow easier testing and error logging.
///
/// # Parameters
///
/// * `command_line_arguments_result` - the results of parsing the command line arguments. This parameter mainly exists to allow testing.
/// * `disable_logging` - disables starting of the logger. This parameter mainly exists to allow testing.
fn main_internal(
    command_line_arguments_result: Result<CommandLineArguments, clap::Error>,
    disable_logging: bool,
) -> Result<(), ApplicationError> {
    // Tries to parse the command line arguments.
    let cl_args_result = match command_line_arguments_result {
        Ok(cl_args) => Ok(cl_args),
        Err(err) => {
            match err.kind() {
                // Returns successful after the help message has been printed
                // or an error if the printing failed.
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    if let Err(err) = err.print() {
                        Err(ApplicationError::from(err)
                            .chain("The command line arguments could not be parsed."))
                    } else {
                        return Ok(());
                    }
                },
                // On an actual error, returns the error.
                _ => Err(ApplicationError::from(err)
                    .chain("The command line arguments could not be parsed.")),
            }
        },
    };
    // In case of an error sets a default log level to allow logging of the error.
    let log_level = cl_args_result
        .as_ref()
        .map(|cl_args| cl_args.log_level())
        .unwrap_or(log::LevelFilter::Warn);

    // Initialises the logger.
    if !disable_logging {
        env_logger::builder()
            .filter_level(log_level)
            .try_init()
            .map_err(|err| {
                ApplicationError::from(err).chain("The logger could not be initialised.")
            })?;
    }

    let command_line_arguments = cl_args_result?;
    let peaks_by_chromosome =
        bed_to_peaks(command_line_arguments.input_files()).map_err(|err| {
            err.chain(format!(
                "Failed to parse input files \"{:?}\".",
                command_line_arguments.input_files()
            ))
        })?;
    let mut consenus: HashMap<String, Vec<PeakData>> = HashMap::new();
    for (chromosome, peaks) in peaks_by_chromosome {
        consenus.insert(
            chromosome,
            command_line_arguments
                .algorithm()
                .consensus_peaks(peaks, &command_line_arguments)
                .map_err(|err| err.chain("Failed to create consensus peaks."))?,
        );
    }
    write_peaks_to_bed(
        command_line_arguments.output_file(),
        &consenus,
        command_line_arguments.bed_output_columns(),
    )
    .map_err(|err| {
        err.chain(format!(
            "Failed to write the consensus peaks to output file \"{}\".",
            command_line_arguments.output_file().display(),
        ))
    })?;
    Ok(())
}

mod arguments;
mod error;
mod input;
mod output;
mod peaks;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use crate::{
        output::peak_to_bed_record_line,
        test_utils::{test_output, test_resources},
    };

    use super::*;

    #[test]
    fn test_main_internal_default_with_summit_4_fields() {
        let n_output_fields = 4;
        let input_dir = test_resources();
        let input_path_1 = input_dir.join("input_test_main_internal_input_01.narrowPeak");
        let input_path_2 = input_dir.join("input_test_main_internal_input_02.narrowPeak");
        let mut output_path = test_output();
        output_path.push("test_main_internal_input.bed");
        // assert!(!output_path.exists());
        let cla = CommandLineArguments::try_parse_from([
            "Gipfelkreuzer",
            "-a",
            "gipfelkreuzer",
            "-m",
            "20",
            "-b",
            &format!("{}", n_output_fields),
            "-o",
            &output_path.display().to_string(),
            &input_path_1.display().to_string(),
            &input_path_2.display().to_string(),
        ]);
        assert!(main_internal(cla, true).is_ok());
        assert!(output_path.exists());

        let output_file = BufReader::new(File::open(&output_path).unwrap());
        let expected_output_lines: Vec<String> = vec![
            PeakData::new(1, 658u64, 739u64, 695u64).unwrap(),
            PeakData::new(4, 864u64, 918u64, 904u64).unwrap(),
        ]
        .iter()
        .map(|peak| peak_to_bed_record_line(peak, "chr1", n_output_fields))
        .collect();
        for line in output_file.lines() {
            // Adds the new line character that was stripped during the read process.
            let line = format!("{}\n", line.unwrap());
            assert!(
                expected_output_lines.contains(&line),
                "The generated output file must contains the line \"{}\", but should only contain \"{:?}\"",
                line,
                expected_output_lines
            )
        }
        std::fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn test_main_internal_default_with_summit_4_fields_simple() {
        let n_output_fields = 4;
        let input_dir = test_resources();
        let input_path_1 = input_dir.join("input_test_main_internal_input_01.narrowPeak");
        let input_path_2 = input_dir.join("input_test_main_internal_input_02.narrowPeak");
        let mut output_path = test_output();
        output_path.push("test_main_internal_default_with_summit_4_fields_simple.bed");
        assert!(!output_path.exists());
        let cla = CommandLineArguments::try_parse_from([
            "Gipfelkreuzer",
            "-a",
            "simple",
            "-b",
            &format!("{}", n_output_fields),
            "-o",
            &output_path.display().to_string(),
            &input_path_1.display().to_string(),
            &input_path_2.display().to_string(),
        ]);
        assert!(main_internal(cla, true).is_ok());
        assert!(output_path.exists());

        let output_file = BufReader::new(File::open(&output_path).unwrap());
        let expected_output_lines: Vec<String> =
            vec![PeakData::new(0, 500u64, 1000u64, 750u64).unwrap()]
                .iter()
                .map(|peak| peak_to_bed_record_line(peak, "chr1", n_output_fields))
                .collect();
        for line in output_file.lines() {
            // Adds the new line character that was stripped during the read process.
            let line = format!("{}\n", line.unwrap());
            assert!(
                expected_output_lines.contains(&line),
                "The generated output file must contains the line \"{}\", but should only contain \"{:?}\"",
                line,
                expected_output_lines
            )
        }
        std::fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn test_main_internal_help() {
        let mut input_path = test_resources();
        input_path.push("input_test_main_internal_input.narrowPeak");
        let mut output_path = test_output();
        output_path.push("test_main_internal_input.bed");
        let cla_short = CommandLineArguments::try_parse_from(vec!["Gipfelkreuzer", "-h"]);
        assert!(main_internal(cla_short, true).is_ok());
    }

    #[test]
    fn test_main_internal_version() {
        let mut input_path = test_resources();
        input_path.push("input_test_main_internal_input.narrowPeak");
        let mut output_path = test_output();
        output_path.push("test_main_internal_input.bed");
        let cla_short = CommandLineArguments::try_parse_from(vec!["Gipfelkreuzer", "-V"]);
        assert!(main_internal(cla_short, true).is_ok());
    }
}
