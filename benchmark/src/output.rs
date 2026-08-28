//! This module computes and writes output data.

use std::{io::BufWriter, path::Path};

use serde::Serialize;

use crate::input::{parse_count_file, parse_peak_file, peak_counts_for_region};

/// Computed output data.
#[derive(Debug, Serialize)]
pub struct OutputData {
    #[serde(rename = "peak file")]
    peak_file_path: String,
    #[serde(rename = "count file")]
    count_file_path: String,
    #[serde(rename = "number of peaks")]
    number_of_peaks: usize,
    #[serde(rename = "median peak length")]
    median_peak_length: f64,
    #[serde(rename = "mean peak length")]
    mean_peak_length: f64,
    #[serde(rename = "mean monotonicity deviation")]
    mean_monotonicity_deviation: f64,
    #[serde(rename = "total counts")]
    total_counts: u64,
}

impl OutputData {
    /// Parses a peak and count file and computes output data.
    ///
    /// # Parameters
    ///
    /// * `peak_file_path` - path to the peak file
    /// * `count file_path` - the path to the count file
    pub fn compute<PeakFilePathType: AsRef<Path>, CountFilePathType: AsRef<Path>>(
        peak_file_path: PeakFilePathType,
        count_file_path: PeakFilePathType,
    ) -> Result<Self, String> {
        let mut count_file = parse_count_file(&count_file_path)?;
        let peak_file = parse_peak_file(&peak_file_path)?;

        let number_of_peaks = peak_file.len();
        let peak_lengths: Vec<usize> = peak_file
            .iter()
            .map(|peak| peak.end() - peak.start())
            .collect();
        let mean_peak_length = mean(&peak_lengths)?;
        let median_peak_length = median(peak_lengths)?;

        let mut monotonicity_values = Vec::with_capacity(peak_file.len());
        let mut total_count_sum: u64 = 0;
        for peak_data in peak_file {
            let a = peak_counts_for_region(&mut count_file, &peak_data)?;
            // monotonicity_dev.push(a.monotonicity_deviation());
            monotonicity_values.push(a.monotonicity_deviation());
            total_count_sum += a.total_counts();
        }

        Ok(Self {
            peak_file_path: peak_file_path.as_ref().display().to_string(),
            count_file_path: count_file_path.as_ref().display().to_string(),
            number_of_peaks,
            median_peak_length,
            mean_peak_length,
            mean_monotonicity_deviation: mean_f64(&monotonicity_values)?,
            total_counts: total_count_sum,
        })
    }

    pub fn save_results<SaveFilePathType: AsRef<Path>>(
        results: &[Self],
        save_file_path: SaveFilePathType,
    ) -> Result<(), String> {
        // Creates the specified output path.
        let parent_directory = save_file_path.as_ref().parent().ok_or(format!(
            "The output file path \"{}\" is invalid.",
            save_file_path.as_ref().display(),
        ))?;
        std::fs::create_dir_all(parent_directory).map_err(|err| {
            format!(
                "The output directory \"{}\" could not be created: {}",
                parent_directory.display(),
                err.to_string()
            )
        })?;

        // Creates the output file.
        let mut output_file = csv::Writer::from_writer(BufWriter::new(
            std::fs::File::create(&save_file_path).map_err(|err| {
                format!(
                    "The output file \"{}\" could not be created: {}",
                    save_file_path.as_ref().display(),
                    err.to_string()
                )
            })?,
        ));
        for result in results {
            output_file.serialize(result).map_err(|err| {
                format!(
                    "The record {:?} could not be written to output file \"{}\": {}",
                    result,
                    save_file_path.as_ref().display(),
                    err.to_string()
                )
            })?;
        }
        Ok(())
    }
}

/// Calculates the mean if possible.
///
/// # Parameters
///
/// * `values` - the values to calculate the mean for
pub fn mean(values: &[usize]) -> Result<f64, String> {
    if values.is_empty() {
        Err("Cannot compute mean of an empty vector.".to_string())
    } else {
        let n_elements = values.len() as f64;
        // Overflow should never be a problem, but just in case.
        let mut mean = 0.0;
        for value in values {
            mean += (*value as f64) / n_elements;
        }
        Ok(mean)
    }
}

/// Calculates the mean if possible.
///
/// # Parameters
///
/// * `values` - the values to calculate the mean for
pub fn mean_f64(values: &[f64]) -> Result<f64, String> {
    if values.is_empty() {
        Err("Cannot compute mean of an empty vector.".to_string())
    } else {
        let n_elements = values.len() as f64;
        // Overflow should never be a problem, but just in case.
        let mut mean = 0.0;
        for value in values {
            mean += (*value as f64) / n_elements;
        }
        Ok(mean)
    }
}

/// Calculates the median if possible.
///
/// # Parameters
///
/// * `values` - the values to calculate the mean for
pub fn median(mut values: Vec<usize>) -> Result<f64, String> {
    if values.is_empty() {
        Err("Cannot compute median of an empty vector.".to_string())
    } else {
        values.sort();
        let central_index = values.len() / 2;
        if values.len() % 2 == 0 {
            Ok(((values[central_index] as f64) / 2.0) + ((values[central_index - 1] as f64) / 2.0))
        } else {
            Ok(values[central_index] as f64)
        }
    }
}

#[cfg(test)]
mod tests {

    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn test_mean() {
        assert_ulps_eq!(mean(&vec![0, 1, 2, 1, 3, 6, 5, 4, 3, 0]).unwrap(), 2.5);
        assert_ulps_eq!(mean(&vec![0, 1, 2, 1, 3, 6, 0, 4, 3, 0]).unwrap(), 2.0);
        assert_ulps_eq!(mean(&vec![0, 0, 0, 0, 0, 0]).unwrap(), 0.0);
        assert_ulps_eq!(mean(&vec![201]).unwrap(), 201.0);
        assert!(mean(&Vec::new()).is_err());
    }

    #[test]
    fn test_mean_f64() {
        assert_ulps_eq!(
            mean_f64(&vec![0.0, 1.0, 2.0, 1.0, 3.0, 6.0, 5.0, 4.0, 3.0, 0.0]).unwrap(),
            2.5
        );
        assert_ulps_eq!(
            mean_f64(&vec![0.0, 1.0, 2.0, 1.0, 3.0, 6.0, 0.0, 4.0, 3.0, 0.0]).unwrap(),
            2.0
        );
        assert_ulps_eq!(mean_f64(&vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap(), 0.0);
        assert_ulps_eq!(mean_f64(&vec![201.0]).unwrap(), 201.0);
        assert!(mean(&Vec::new()).is_err());
    }

    #[test]
    fn test_median() {
        assert_ulps_eq!(median(vec![0, 0, 0, 0, 0, 0]).unwrap(), 0.0);
        assert_ulps_eq!(median(vec![201]).unwrap(), 201.0);
        assert_ulps_eq!(median(vec![201, 1]).unwrap(), 101.0);
        assert_ulps_eq!(median(vec![201, 1, 4]).unwrap(), 4.0);
        assert_ulps_eq!(median(vec![201, 1, 4, 5]).unwrap(), 4.5);
        assert_ulps_eq!(median(vec![0, 1, 2, 1, 3, 6, 5, 4, 3, 0]).unwrap(), 2.5);
        assert_ulps_eq!(median(vec![0, 0, 1, 1, 2, 3, 3, 4, 5, 6]).unwrap(), 2.5);
        assert!(median(Vec::new()).is_err());
    }
}
