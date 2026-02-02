//! This module contains the specifics of the simple peak merging algorithm.

use crate::{
    error::ApplicationError,
    peaks::{PeakData, gipfelkreuzer::GipfelkreuzerPeakMerger},
};

/// Merges overlapping and adjacent peaks.
/// Returns an error if the merging process fails.
/// 
/// # Parameters
/// * `peaks` - the peaks to merge
pub fn merge_peaks(peaks: Vec<PeakData>) -> Result<Vec<PeakData>, ApplicationError> {
    let complex_merger = GipfelkreuzerPeakMerger::new(peaks);
    let mut merged_peaks = Vec::with_capacity(complex_merger.bins().len());

    for (bin_index, bin) in complex_merger.bins().iter().enumerate() {
        merged_peaks.push(
            PeakData::new(bin_index, bin.start(), bin.end(), bin.start().midpoint(bin.end()))
                .map_err(|err| {
                    err.chain(format!(
                        "Failed to create a simple merge consensus peak from peak bin {}: {:?}",
                        bin_index, bin
                    ))
                })?,
        );
    }

    Ok(merged_peaks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_peaks() {
        let peaks = vec![
            PeakData::new(0, 12u64, 24u64, 18u64).unwrap(),
            PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
            PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
            PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
            PeakData::new(4, 260u64, 290u64, 270u64).unwrap(),
            PeakData::new(5, 259u64, 277u64, 270u64).unwrap(),
        ];
        let expected_consensus_peaks = vec![
            PeakData::new(0, 11u64, 29u64, 20u64).unwrap(),
            PeakData::new(1, 259u64, 290u64, 274u64).unwrap(),
        ];
        let consensus_peaks = merge_peaks(peaks).unwrap();
        assert_eq!(consensus_peaks.len(), expected_consensus_peaks.len());
        for consensus_peak in consensus_peaks {
            assert!(
                expected_consensus_peaks.contains(&consensus_peak),
                "The consensus peak {:?} was not in the list of expected peaks: {:?}",
                consensus_peak,
                expected_consensus_peaks
            )
        }
    }
}
