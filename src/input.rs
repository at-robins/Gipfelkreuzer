//! This module parses input data.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{error::ApplicationError, peaks::PeakData};

/// Parses BED3+ files according to the [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) definition.
/// Peak summit information will be extracted from field 10 according to the
/// [narrowPeak](https://genome.ucsc.edu/FAQ/FAQformat.html#format12) fromat definition if present and possible.
///
/// # Parameters
///
/// * `path` - the input file path
pub fn bed_to_peaks<T: AsRef<Path>>(
    path: T,
) -> Result<HashMap<String, Vec<PeakData>>, ApplicationError> {
    let file = File::open(&path).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("The input file \"{}\" could not be opened.", path.as_ref().display()))
    })?;
    let mut peak_map: HashMap<String, Vec<PeakData>> = HashMap::new();
    for (line_index, line_result) in BufReader::new(file).lines().enumerate() {
        let line = line_result.map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to parse line {} of input file \"{}\".",
                line_index + 1,
                path.as_ref().display()
            ))
        })?;
        let fields: Vec<&str> = line
            .split(&[' ', '\t'])
            .filter(|split| !split.is_empty())
            .collect();
        if fields.is_empty() {
            log::debug!(
                "Skipping blank line {} in file \"{}\".",
                line_index + 1,
                path.as_ref().display()
            );
        } else if fields[0].starts_with('#') {
            log::debug!(
                "Skipping comment line {} in file \"{}\".",
                line_index + 1,
                path.as_ref().display()
            );
        } else if fields.len() < 3 {
            return Err(ApplicationError::new(
                crate::error::ApplicationErrorType::InputDataError,
                format!(
                    "Line {} of file \"{}\" does not contain the minimally required records.",
                    line_index + 1,
                    path.as_ref().display()
                ),
            ));
        } else {
            // Tries to parse the actual values from the file.
            let chromosome = fields[0].to_string();
            let start: u64 = fields[1].parse().map_err(|err| {
                ApplicationError::from(err).chain(format!(
                    "Value \"{}\" at line {} of file \"{}\" could \
                    not be parsed as genomic start coordinates.",
                    fields[1],
                    line_index + 1,
                    path.as_ref().display()
                ))
            })?;
            let end: u64 = fields[2].parse().map_err(|err| {
                ApplicationError::from(err).chain(format!(
                    "Value \"{}\" at line {} of file \"{}\" could \
                    not be parsed as genomic end coordinates.",
                    fields[2],
                    line_index + 1,
                    path.as_ref().display()
                ))
            })?;
            let summit = if let Some(summit_field) = fields.get(9).and_then(|field_value| {
                // "-1" indicates missing peak summit information according to the narrowPeak format definition,
                // so parsing should be skipped.
                if *field_value == "-1" {
                    None
                } else {
                    Some(field_value)
                }
            }) {
                let summit_offset: u64 = summit_field.parse().map_err(|err| {
                    ApplicationError::from(err).chain(format!(
                        "Value \"{}\" at line {} of file \"{}\" could \
                        not be parsed as peak summit coordinates.",
                        summit_field,
                        line_index + 1,
                        path.as_ref().display()
                    ))
                })?;
                start + summit_offset
            } else {
                log::warn!(
                    "Line {} of file \"{}\" did not contain \
                    peak summit information. Summit is approximated.",
                    line_index + 1,
                    path.as_ref().display()
                );
                start.midpoint(end)
            };
            let peak = PeakData::new(line_index, start, end, summit).map_err(|err| {
                ApplicationError::from(err).chain(format!(
                    "Line {} of file \"{}\" contains invalid data.",
                    line_index + 1,
                    path.as_ref().display()
                ))
            })?;
            if let Some(peaks) = peak_map.get_mut(&chromosome) {
                peaks.push(peak);
            } else {
                peak_map.insert(chromosome, vec![peak]);
            }
        }
    }
    Ok(peak_map)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::test_resources;

    use super::*;

    #[test]
    fn test_bed_to_peaks() {
        let peaks =
            bed_to_peaks(test_resources().join("input_test_valid_with_summit.narrowPeak")).unwrap();
        assert_eq!(peaks.len(), 2);
        assert_eq!(peaks["chr1"].len(), 4);
        assert_eq!(peaks["chr2"].len(), 2);
        let expected_peaks_1 = vec![
            PeakData::new(0, 4470246u64, 4470509u64, 4470246u64 + 107u64).unwrap(),
            PeakData::new(1, 4496298u64, 4496749u64, 4496298u64 + 278u64).unwrap(),
            PeakData::new(2, 4547437u64, 4547657u64, 4547437u64 + 95u64).unwrap(),
            PeakData::new(3, 4671575u64, 4671768u64, 4671575u64 + 78u64).unwrap(),
        ];
        for expected_peak in expected_peaks_1 {
            assert!(
                peaks["chr1"].contains(&expected_peak),
                "Expected peak {:?} in {:?}..",
                expected_peak,
                peaks["chr1"]
            );
        }

        let expected_peaks_2 = vec![
            PeakData::new(4, 4747858u64, 4748017u64, 4747858u64 + 96u64).unwrap(),
            PeakData::new(5, 4748160u64, 4748522u64, 4748160u64 + 186u64).unwrap(),
        ];
        for expected_peak in expected_peaks_2 {
            assert!(
                peaks["chr2"].contains(&expected_peak),
                "Expected peak {:?} in {:?}..",
                expected_peak,
                peaks["chr2"]
            );
        }
    }

    #[test]
    fn test_bed_to_peaks_minimal() {
        let peaks =
            bed_to_peaks(test_resources().join("input_test_valid_minimal.narrowPeak")).unwrap();
        assert_eq!(peaks.len(), 2);
        assert_eq!(peaks["chr1"].len(), 4);
        assert_eq!(peaks["chr2"].len(), 2);
        let expected_peaks_1 = vec![
            PeakData::new(0, 4470246u64, 4470509u64, (4470246u64 + 4470509u64) / 2).unwrap(),
            PeakData::new(1, 4496298u64, 4496749u64, (4496298u64 + 4496749u64) / 2).unwrap(),
            PeakData::new(2, 4547437u64, 4547657u64, (4547437u64 + 4547657u64) / 2).unwrap(),
            PeakData::new(3, 4671575u64, 4671768u64, (4671575u64 + 4671768u64) / 2).unwrap(),
        ];
        for expected_peak in expected_peaks_1 {
            assert!(
                peaks["chr1"].contains(&expected_peak),
                "Expected peak {:?} in {:?}..",
                expected_peak,
                peaks["chr1"]
            );
        }

        let expected_peaks_2 = vec![
            PeakData::new(4, 4747858u64, 4748017u64, (4747858u64 + 4748017u64) / 2).unwrap(),
            PeakData::new(5, 4748160u64, 4748522u64, (4748160u64 + 4748522u64) / 2).unwrap(),
        ];
        for expected_peak in expected_peaks_2 {
            assert!(
                peaks["chr2"].contains(&expected_peak),
                "Expected peak {:?} in {:?}..",
                expected_peak,
                peaks["chr2"]
            );
        }
    }

    #[test]
    fn test_bed_to_peaks_additional_format_specifications() {
        let peaks = bed_to_peaks(
            test_resources().join("input_test_valid_with_summit_additional_features.narrowPeak"),
        )
        .unwrap();
        assert_eq!(peaks.len(), 2);
        assert_eq!(peaks["chr1"].len(), 4);
        assert_eq!(peaks["chr2"].len(), 2);

        let expected_peaks_1 = vec![
            PeakData::new(0, 4470246u64, 4470509u64, 4470246u64 + 107u64).unwrap(),
            PeakData::new(1, 4496298u64, 4496749u64, 4496298u64 + 278u64).unwrap(),
            PeakData::new(2, 4547437u64, 4547657u64, 4547437u64 + 95u64).unwrap(),
            PeakData::new(4, 4671575u64, 4671768u64, 4671575u64 + 78u64).unwrap(),
        ];
        for expected_peak in expected_peaks_1 {
            assert!(
                peaks["chr1"].contains(&expected_peak),
                "Expected peak {:?} in {:?}..",
                expected_peak,
                peaks["chr1"]
            );
        }

        let expected_peaks_2 = vec![
            PeakData::new(8, 4747858u64, 4748017u64, 4747858u64 + 96u64).unwrap(),
            PeakData::new(9, 4748160u64, 4748522u64, (4748160u64 + 4748522u64) / 2).unwrap(),
        ];
        for expected_peak in expected_peaks_2 {
            assert!(
                peaks["chr2"].contains(&expected_peak),
                "Expected peak {:?} in {:?}..",
                expected_peak,
                peaks["chr2"]
            );
        }
    }

    #[test]
    fn test_bed_to_peaks_file_does_not_exist() {
        let expected_error_message_content = "could not be opened.";
        let error = bed_to_peaks(test_resources().join("file_does_not_exist.error")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_encoding() {
        let expected_error_message_content = "Failed to parse line";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_utf8.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_start() {
        let expected_error_message_content = "could not be parsed as genomic start coordinates.";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_start.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_end() {
        let expected_error_message_content = "could not be parsed as genomic end coordinates.";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_end.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_summit() {
        let expected_error_message_content = "could not be parsed as peak summit coordinates.";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_summit.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_fields() {
        let expected_error_message_content = "does not contain the minimally required records.";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_not_enough_fields.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_data_start_end() {
        let expected_error_message_content = "contains invalid data.";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_data_start_end.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }

    #[test]
    fn test_bed_to_peaks_invalid_data_summit() {
        let expected_error_message_content = "contains invalid data.";
        let error =
            bed_to_peaks(test_resources().join("input_test_invalid_data_summit.narrowPeak")).unwrap_err();
        assert!(
            error
                .internal_messages()
                .last()
                .unwrap()
                .contains(expected_error_message_content),
            "The error {:?} did not contain the expected content \"{}\".",
            error,
            expected_error_message_content
        );
    }
}
