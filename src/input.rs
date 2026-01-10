//! This module parses input data.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{error::ApplicationError, peaks::PeakData};

/// Parses BED files according to the [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) definition.
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
            let summit = if let Some(summit_field) = fields.get(9) {
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
