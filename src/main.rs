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
        path::PathBuf,
    };

    use crate::{
        output::peak_to_bed_record_line,
        test_utils::{test_output, test_resources},
    };

    use super::*;

    #[test]
    fn test_main_internal_default_with_summit_4_fields() {
        test_main_internal(
            &vec![
                "input_test_main_internal_input_01.narrowPeak",
                "input_test_main_internal_input_02.narrowPeak",
            ],
            "test_main_internal_default_with_summit_4_fields.bed",
            &vec![
                "-a".to_string(),
                "gipfelkreuzer".to_string(),
                "-m".to_string(),
                "20".to_string(),
                "-b".to_string(),
                "4".to_string(),
            ],
            vec![
                ("chr1".to_string(), PeakData::new(1, 629u64, 769u64, 698u64).unwrap()),
                ("chr1".to_string(), PeakData::new(4, 864u64, 918u64, 904u64).unwrap()),
            ],
        );
    }

    #[test]
    fn test_main_internal_default_with_summit_14_fields() {
        test_main_internal(
            &vec![
                "input_test_main_internal_input_01.narrowPeak",
                "input_test_main_internal_input_02.narrowPeak",
            ],
            "test_main_internal_default_with_summit_14_fields.bed",
            &vec![
                "-a".to_string(),
                "gipfelkreuzer".to_string(),
                "-m".to_string(),
                "20".to_string(),
                "-b".to_string(),
                "14".to_string(),
            ],
            vec![
                ("chr1".to_string(), PeakData::new(1, 629u64, 769u64, 698u64).unwrap()),
                ("chr1".to_string(), PeakData::new(4, 864u64, 918u64, 904u64).unwrap()),
            ],
        );
    }

    #[test]
    fn test_main_internal_default_with_summit_14_fields_min() {
        test_main_internal(
            &vec![
                "input_test_main_internal_input_01.narrowPeak",
                "input_test_main_internal_input_02.narrowPeak",
            ],
            "test_main_internal_default_with_summit_14_fields_min.bed",
            &vec![
                "-a".to_string(),
                "gipfelkreuzer".to_string(),
                "-m".to_string(),
                "20".to_string(),
                "-n".to_string(),
                "4".to_string(),
                "-b".to_string(),
                "14".to_string(),
            ],
            vec![("chr1".to_string(), PeakData::new(1, 629u64, 769u64, 698u64).unwrap())],
        );
    }

    #[test]
    fn test_main_internal_default_with_summit_4_fields_simple() {
        test_main_internal(
            &vec![
                "input_test_main_internal_input_01.narrowPeak",
                "input_test_main_internal_input_02.narrowPeak",
            ],
            "test_main_internal_default_with_summit_4_fields_simple.bed",
            &vec![
                "-a".to_string(),
                "simple".to_string(),
                "-b".to_string(),
                "4".to_string(),
            ],
            vec![("chr1".to_string(), PeakData::new(0, 500u64, 1000u64, 750u64).unwrap())],
        );
    }

    #[test]
    fn test_main_internal_with_summit_14_fields_simple_min() {
        test_main_internal(
            &vec![
                "input_test_main_internal_input_01_simple_min.narrowPeak",
                "input_test_main_internal_input_02.narrowPeak",
            ],
            "test_main_internal_default_with_summit_14_fields_simple_min.bed",
            &vec![
                "-a".to_string(),
                "simple".to_string(),
                "-n".to_string(),
                "4".to_string(),
                "-b".to_string(),
                "14".to_string(),
            ],
            vec![("chr1".to_string(), PeakData::new(0, 600u64, 788u64, 694u64).unwrap())],
        );
    }

    #[test]
    fn test_main_internal_default_with_summit_14_fields_simple() {
        test_main_internal(
            &vec![
                "input_test_main_internal_input_01.narrowPeak",
                "input_test_main_internal_input_02.narrowPeak",
            ],
            "test_main_internal_default_with_summit_14_fields_simple.bed",
            &vec![
                "-a".to_string(),
                "simple".to_string(),
                "-b".to_string(),
                "14".to_string(),
            ],
            vec![("chr1".to_string(), PeakData::new(0, 500u64, 1000u64, 750u64).unwrap())],
        );
    }

    #[test]
    fn test_main_internal_help() {
        let cla_short = CommandLineArguments::try_parse_from(vec!["Gipfelkreuzer", "-h"]);
        assert!(main_internal(cla_short, true).is_ok());
    }

    #[test]
    fn test_main_internal_version() {
        let cla_short = CommandLineArguments::try_parse_from(vec!["Gipfelkreuzer", "-V"]);
        assert!(main_internal(cla_short, true).is_ok());
    }

    /// Runs a standardised test for the internal ```main``` function.
    ///
    /// # Parameters
    ///
    /// * `input` - the path to the respective input files within the test directory
    /// * `output` - the path to the output file within the tmeporary test output directory
    /// * `cla` - the command line argument list
    /// * `expected_output_peaks` - a list of output peaks and their respective chromosome
    fn test_main_internal(
        input: &[&str],
        output: &str,
        cla: &[String],
        expected_output_peaks: Vec<(String, PeakData)>,
    ) {
        let input_dir = test_resources();
        let input_paths: Vec<PathBuf> = input.iter().map(|i| input_dir.join(i)).collect();
        let mut output_path = test_output();
        output_path.push(output);
        // Cleans up data from failed tests.
        if output_path.exists() {
            std::fs::remove_file(&output_path).unwrap();
        }
        assert!(!output_path.exists());
        let mut final_cla = vec!["Gipfelkreuzer".to_string()];
        final_cla.extend_from_slice(cla);
        final_cla.extend_from_slice(&["-o".to_string(), output_path.display().to_string()]);
        final_cla.extend(input_paths.iter().map(|i| i.display().to_string()));
        let cla = CommandLineArguments::try_parse_from(final_cla);
        let bed_fields = cla
            .as_ref()
            .map(|arguments| arguments.bed_output_columns())
            .unwrap();
        assert!(main_internal(cla, true).is_ok());
        assert!(output_path.exists());

        let output_file = BufReader::new(File::open(&output_path).unwrap());
        let expected_output_lines: Vec<String> = expected_output_peaks
            .iter()
            .map(|(chromosome, peak)| peak_to_bed_record_line(peak, chromosome, bed_fields))
            .collect();
        let mut number_lines_in_output = 0;
        for line in output_file.lines() {
            number_lines_in_output += 1;
            // Adds the new line character that was stripped during the read process.
            let line = format!("{}\n", line.unwrap());
            assert!(
                expected_output_lines.contains(&line),
                "The generated output file contains the line \"{}\", but should only contain \"{:?}\"",
                line,
                expected_output_lines
            )
        }
        assert_eq!(expected_output_lines.len(), number_lines_in_output);
        std::fs::remove_file(output_path).unwrap();
    }
}
