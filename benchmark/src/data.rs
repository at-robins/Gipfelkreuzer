//! This module provides data types.

use std::collections::HashMap;

use getset::{CopyGetters, Getters};

#[derive(CopyGetters, Getters, Clone, PartialEq, Eq, Debug)]
/// Data representing a peak region on genomic data.
pub struct PeakData {
    /// The chromosome the peak is located on.
    #[getset(get = "pub")]
    chromosome: String,
    /// The genomic start coordinate of the peak region.
    #[getset(get_copy = "pub")]
    start: usize,
    /// The genomic end coordinate of the peak region (inclusive).
    #[getset(get_copy = "pub")]
    end: usize,
}

impl PeakData {
    /// Creates a new genomic peak region.
    ///
    /// # Parameters
    ///
    /// * `chromosome`: the chromosome the peak is located on
    /// * `start`: the start location of the peak region
    /// * `end`: the end location of the peak region (inclusive)
    ///
    /// # Errors
    ///
    /// Returns an error if the start coordinate is greater than the end coordinate.
    pub fn new<StartType: Into<usize>, EndType: Into<usize>, ChromosomeType: Into<String>>(
        chromosome: ChromosomeType,
        start: StartType,
        end: EndType,
    ) -> Result<Self, String> {
        let start = start.into();
        let end = end.into();
        let chromosome = chromosome.into();

        if start > end {
            return Err(format!(
                "The peak end coordinate {} of on chromosome {} is smaller than the start coordinate {}.",
                end, chromosome, start
            ));
        }

        Ok(Self {
            chromosome,
            start,
            end,
        })
    }

    /// Merges peaks into non-overlapping peak regions.
    /// 
    /// # Parameters
    /// 
    /// * `peaks` - the peaks to merge
    pub fn non_overlapping(peaks: &[Self]) -> Vec<Self> {
        let mut non_overlapping = Vec::new();
        let mut peak_map: HashMap<&String, Vec<&Self>> = HashMap::new();
        for peak in peaks {
            if let Some(chromosome_peaks) = peak_map.get_mut(peak.chromosome()) {
                chromosome_peaks.push(peak);
            } else {
                peak_map.insert(peak.chromosome(), vec![peak]);
            }
        }
        for (_, peaks_by_chromosome) in peak_map {
            // println!("{:?}", Self::non_overlapping_internal(peaks_by_chromosome.clone()));
            non_overlapping.extend(Self::non_overlapping_internal(peaks_by_chromosome));
        }
        non_overlapping
    }

    /// Merges peaks into non-overlapping peak regions within a chromosome (ignores chromosome information).
    /// 
    /// # Parameters
    /// 
    /// * `peaks` - the peaks to merge
    fn non_overlapping_internal(mut peaks: Vec<&Self>) -> Vec<Self> {
        peaks.sort_by(|a, b| a.start().cmp(&b.start()));
        // println!("{:?}", peaks);
        let mut non_overlapping: Vec<Self> = Vec::new();
        for peak in peaks {
            if let Some(last_peak) = non_overlapping.last_mut()
                && last_peak.end() >= peak.start()
            {
                if last_peak.end() < peak.end() {
                    last_peak.end = peak.end();
                }
            } else {
                non_overlapping.push(peak.clone());
            }
        }
        non_overlapping
    }
}

#[cfg(test)]
mod tests {

    use std::borrow::Borrow;

use super::*;

    #[test]
    fn test_non_overlapping_internal() {
        let test_chromosome = "test";
        let test_peaks = vec![
            PeakData::new(test_chromosome, 15usize, 25usize).unwrap(),
            PeakData::new(test_chromosome, 452usize, 455usize).unwrap(),
            PeakData::new(test_chromosome, 555usize, 5555usize).unwrap(),
            PeakData::new(test_chromosome, 20usize, 24usize).unwrap(),
            PeakData::new(test_chromosome, 21usize, 28usize).unwrap(),
            PeakData::new(test_chromosome, 28usize, 34usize).unwrap(),
            PeakData::new(test_chromosome, 0usize, 5usize).unwrap(),
            PeakData::new(test_chromosome, 451usize, 453usize).unwrap(),
        ];
        let expected_output_peaks = vec![
            PeakData::new(test_chromosome, 0usize, 5usize).unwrap(),
            PeakData::new(test_chromosome, 15usize, 34usize).unwrap(),
            PeakData::new(test_chromosome, 451usize, 455usize).unwrap(),
            PeakData::new(test_chromosome, 555usize, 5555usize).unwrap(),
        ];
        let test_peak_refs: Vec<&PeakData> = test_peaks.iter().map(Borrow::borrow).collect();
        assert_eq!(expected_output_peaks, PeakData::non_overlapping_internal(test_peak_refs));
    }

    #[test]
    fn test_non_overlapping() {
        let test_chromosome_1 = "test1";
        let test_chromosome_2 = "test2";
        let test_peaks = vec![
            PeakData::new(test_chromosome_1, 15usize, 25usize).unwrap(),
            PeakData::new(test_chromosome_1, 452usize, 455usize).unwrap(),
            PeakData::new(test_chromosome_1, 555usize, 5555usize).unwrap(),
            PeakData::new(test_chromosome_1, 20usize, 24usize).unwrap(),
            PeakData::new(test_chromosome_1, 21usize, 28usize).unwrap(),
            PeakData::new(test_chromosome_1, 28usize, 34usize).unwrap(),
            PeakData::new(test_chromosome_1, 0usize, 5usize).unwrap(),
            PeakData::new(test_chromosome_1, 451usize, 453usize).unwrap(),

            PeakData::new(test_chromosome_2, 15usize, 25usize).unwrap(),
            PeakData::new(test_chromosome_2, 452usize, 455usize).unwrap(),
            PeakData::new(test_chromosome_2, 555usize, 5555usize).unwrap(),
            PeakData::new(test_chromosome_2, 20usize, 24usize).unwrap(),
            PeakData::new(test_chromosome_2, 21usize, 28usize).unwrap(),
            PeakData::new(test_chromosome_2, 28usize, 34usize).unwrap(),
            PeakData::new(test_chromosome_2, 0usize, 5usize).unwrap(),
            PeakData::new(test_chromosome_2, 451usize, 453usize).unwrap(),
        ];
        let expected_output_peaks = vec![
            PeakData::new(test_chromosome_1, 0usize, 5usize).unwrap(),
            PeakData::new(test_chromosome_1, 15usize, 34usize).unwrap(),
            PeakData::new(test_chromosome_1, 451usize, 455usize).unwrap(),
            PeakData::new(test_chromosome_1, 555usize, 5555usize).unwrap(),
            PeakData::new(test_chromosome_2, 0usize, 5usize).unwrap(),
            PeakData::new(test_chromosome_2, 15usize, 34usize).unwrap(),
            PeakData::new(test_chromosome_2, 451usize, 455usize).unwrap(),
            PeakData::new(test_chromosome_2, 555usize, 5555usize).unwrap(),
        ];
        let output_peaks = PeakData::non_overlapping(&test_peaks);
        assert_eq!(expected_output_peaks.len(), output_peaks.len());
        for expected_output_peak in expected_output_peaks {
            assert!(output_peaks.contains(&expected_output_peak));
        }
    }
}
