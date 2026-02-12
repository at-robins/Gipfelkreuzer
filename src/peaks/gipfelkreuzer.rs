//! This module contains the specifics of the Gipfelkreuzer consensus peak generation algorithm.

use crate::peaks::{PeakBin, PeakData};

/// Converts a [`PeakBin`] into its respective consensus peaks.
///
/// # Parameters
///
/// * `peak_bin` - the bin of peaks to generate consensus peaks from
/// * `max_iterations` - the maximum number of peak merging iterations to be performed
/// * `min_peaks_per_consensus` - the minimum number of raw peak that are required for the generation of a consensus peak
fn bin_to_consensus_peaks(
    peak_bin: PeakBin,
    max_iterations: usize,
    min_peaks_per_consensus: usize,
) -> Vec<PeakData> {
    let mut consensus = bin_to_consensus_peaks_internal(
        Vec::<PeakData>::from(peak_bin)
            .into_iter()
            .map(ConsensusPeakAggregator::from)
            .collect(),
    );
    // Iterativesly merges peaks until the maximum number of iterations is reached
    // or the peaks do not change anymore.
    let previous_consensus_length = consensus.len();
    for _ in 0..max_iterations {
        consensus = bin_to_consensus_peaks_internal(consensus);
        if consensus.len() == previous_consensus_length {
            break;
        }
    }
    consensus
        .into_iter()
        .filter(|peak| peak.number_aggregated_peaks() >= min_peaks_per_consensus)
        .map(PeakData::from)
        .collect()
}

/// Converts the peak bin into its respective consensus peaks.
/// Internal function logic to allow easy iterative consensus peak generation.
fn bin_to_consensus_peaks_internal(
    mut peaks: Vec<ConsensusPeakAggregator>,
) -> Vec<ConsensusPeakAggregator> {
    let mut consensus_peaks = Vec::new();
    peaks.sort_by(|a, b| a.length().cmp(&b.length()));
    let mut remaining_peaks = peaks;
    while !remaining_peaks.is_empty() {
        let mut consensus_peak_aggregator: Option<ConsensusPeakAggregator> = None;
        let mut retained_peaks = Vec::with_capacity(remaining_peaks.len());

        for peak in remaining_peaks {
            if let Some(aggregator) = &mut consensus_peak_aggregator {
                // If the peak matches the consensus defining one, adds it to the aggregator.
                if let Some(unsuitable_peak) = aggregator.try_aggregate(peak) {
                    // Otherwise retains it as an additional peak.
                    retained_peaks.push(unsuitable_peak);
                }
            } else {
                // Uses the shortest peak as initial consensus peak characteristic defining peak.
                consensus_peak_aggregator = Some(peak);
            }
        }

        consensus_peaks.push(
            consensus_peak_aggregator
                .expect("The consensus aggregator must have been created at this point."),
        );
        remaining_peaks = retained_peaks;
    }
    consensus_peaks
}

pub fn consensus_peaks(
    peaks: Vec<PeakData>,
    max_iterations: usize,
    min_peaks_per_consensus: usize,
) -> Vec<PeakData> {
    let mut consensus_peaks = Vec::new();
    for bin in PeakBin::bin_peaks(peaks) {
        consensus_peaks.extend(bin_to_consensus_peaks(
            bin,
            max_iterations,
            min_peaks_per_consensus,
        ));
    }
    consensus_peaks
}

#[derive(Clone, Debug, PartialEq)]
/// An aggregator that represents multiple raw peaks that are used for consensus peak generation.
struct ConsensusPeakAggregator {
    peaks: Vec<PeakData>,
    consensus_peak: PeakData,
}

impl ConsensusPeakAggregator {
    /// The ID of the currently aggregated consenus peak.
    fn id(&self) -> usize {
        self.consensus_peak.id()
    }

    /// Tries to merge the two peak aggregators. If they are similar based on their summit distance
    /// the passed aggregator is consumed and its peaks are merged into this aggregator, otherwise the
    /// aggregator is returned unaltered.
    ///
    /// # Parameters
    ///
    /// * `peak` - the consensus peak to merge
    pub fn try_aggregate(
        &mut self,
        peak: ConsensusPeakAggregator,
    ) -> Option<ConsensusPeakAggregator> {
        if peak.summit() <= self.consensus_peak.end()
            && peak.summit() >= self.consensus_peak.start()
        {
            self.peaks.extend(peak.peaks);
            self.update_consensus_peak();
            None
        } else {
            Some(peak)
        }
    }

    /// The summit of the currently aggregated consenus peak.
    fn summit(&self) -> u64 {
        self.consensus_peak.summit()
    }

    /// The length of the currently aggregated consenus peak.
    fn length(&self) -> u64 {
        self.consensus_peak.length()
    }

    /// Returns the number of aggregated peaks used to create this consensus peak.
    pub fn number_aggregated_peaks(&self) -> usize {
        self.peaks.len()
    }

    /// Updates the current consenus peak.
    /// Internal function that should be called after updating the raw peaks of the aggregator.
    fn update_consensus_peak(&mut self) {
        let starts: Vec<u64> = self.peaks.iter().map(PeakData::start).collect();
        let ends: Vec<u64> = self.peaks.iter().map(PeakData::end).collect();
        let summits: Vec<u64> = self.peaks.iter().map(PeakData::summit).collect();
        self.consensus_peak = PeakData::new(
            self.id(),
            u64_median(starts),
            u64_median(ends),
            u64_median(summits),
        )
        .expect(
            "The consensus peak parameters must be valid as they were derived from valid peaks.",
        );
    }
}

impl From<PeakData> for ConsensusPeakAggregator {
    fn from(peak: PeakData) -> Self {
        Self {
            peaks: vec![peak],
            consensus_peak: peak,
        }
    }
}

impl From<ConsensusPeakAggregator> for PeakData {
    fn from(value: ConsensusPeakAggregator) -> Self {
        value.consensus_peak
    }
}

/// Returns the median of the specified values.
///
/// # Parameters
///
/// * `values` - the values to calculate the median of
///
/// # Panics
///
/// If the vector of values is empty.
fn u64_median(mut values: Vec<u64>) -> u64 {
    if values.is_empty() {
        panic!("The median of an empty collection cannot be calculated.");
    }
    values.sort();
    let midpoint = values.len().div_ceil(2) - 1;
    if values.len() % 2 == 0 {
        (values[midpoint] + values[midpoint + 1]) / 2
    } else {
        values[midpoint]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_median() {
        // Central value.
        assert_eq!(8, u64_median(vec![1, 8, 56]));
        // Mean of central values.
        assert_eq!(32, u64_median(vec![1, 8, 56, 353631]));
        // Rounding of mean of central value.
        assert_eq!(32, u64_median(vec![1, 9, 56, 353631]));
    }

    #[test]
    #[should_panic]
    fn test_u64_median_empty() {
        u64_median(Vec::new());
    }

    #[test]
    fn test_consensus_peak_aggregator_from_peak_data() {
        let peak = PeakData::new(42, 42u64, 84u64, 63u64).unwrap();
        let aggregator = ConsensusPeakAggregator::from(peak);
        assert_eq!(peak.id(), aggregator.id());
        assert_eq!(peak.length(), aggregator.length());
        assert_eq!(peak.summit(), aggregator.summit());
        let consensus: PeakData = aggregator.into();
        assert_eq!(consensus, peak);
    }

    #[test]
    fn test_consensus_peak_aggregator_into_peak_data() {
        let start_peak = PeakData::new(42, 42u64, 84u64, 63u64).unwrap();
        let peaks: Vec<ConsensusPeakAggregator> = vec![
            PeakData::new(43, 44u64, 85u64, 61u64).unwrap().into(),
            PeakData::new(44, 43u64, 83u64, 62u64).unwrap().into(),
        ];
        let expected_consensus_peak = PeakData::new(42, 43u64, 84u64, 62u64).unwrap();
        let mut aggregator = ConsensusPeakAggregator::from(start_peak);
        for peak in peaks {
            assert!(aggregator.try_aggregate(peak).is_none());
        }
        let consensus: PeakData = aggregator.into();
        assert_eq!(consensus, expected_consensus_peak);
    }

    #[test]
    fn test_consensus_peak_aggregator_try_aggregate_single() {
        let start_peak = PeakData::new(42, 42u64, 84u64, 63u64).unwrap();
        let peaks: Vec<ConsensusPeakAggregator> = vec![
            PeakData::new(43, 44u64, 85u64, 61u64).unwrap().into(),
            PeakData::new(44, 43u64, 83u64, 65u64).unwrap().into(),
            PeakData::new(90, 90u64, 120u64, 100u64).unwrap().into(),
        ];
        let expected_consensus_peak = PeakData::new(42, 43u64, 84u64, 63u64).unwrap();
        let mut aggregator = ConsensusPeakAggregator::from(start_peak);
        assert_eq!(aggregator.number_aggregated_peaks(), 1);
        assert!(aggregator.try_aggregate(peaks[0].clone()).is_none());
        assert_eq!(aggregator.summit(), 62u64);
        assert_eq!(aggregator.length(), 42);
        assert_eq!(aggregator.number_aggregated_peaks(), 2);
        assert!(aggregator.try_aggregate(peaks[1].clone()).is_none());
        assert_eq!(aggregator.summit(), 63u64);
        assert_eq!(aggregator.length(), 42);
        assert_eq!(aggregator.number_aggregated_peaks(), 3);
        assert_eq!(aggregator.try_aggregate(peaks[2].clone()), Some(peaks[2].clone()));
        assert_eq!(aggregator.number_aggregated_peaks(), 3);
        assert_eq!(aggregator.summit(), 63u64);
        assert_eq!(aggregator.length(), 42);

        let consensus: PeakData = aggregator.into();
        assert_eq!(consensus, expected_consensus_peak);
    }

    #[test]
    fn test_consensus_peak_aggregator_try_aggregate_multiple() {
        let start_peak = PeakData::new(42, 42u64, 84u64, 63u64).unwrap();
        let peaks: Vec<ConsensusPeakAggregator> = vec![
            PeakData::new(43, 44u64, 85u64, 61u64).unwrap().into(),
            PeakData::new(44, 43u64, 83u64, 65u64).unwrap().into(),
        ];
        let mut aggregator = ConsensusPeakAggregator::from(start_peak);
        for peak in peaks {
            assert!(aggregator.try_aggregate(peak).is_none());
        }

        // Creates a consensus peak that should merge
        let start_peak_merge = PeakData::new(45, 39u64, 84u64, 64u64).unwrap();
        let peaks_merge: Vec<ConsensusPeakAggregator> = vec![
            PeakData::new(46, 34u64, 95u64, 64u64).unwrap().into(),
            PeakData::new(47, 40u64, 93u64, 65u64).unwrap().into(),
        ];
        let mut aggregator_merge = ConsensusPeakAggregator::from(start_peak_merge);
        for peak in peaks_merge {
            assert!(aggregator_merge.try_aggregate(peak).is_none());
        }

        // Creates a consensus peak that should not merge.
        let start_peak_no_merge = PeakData::new(420, 420u64, 840u64, 630u64).unwrap();
        let peaks_no_merge: Vec<ConsensusPeakAggregator> = vec![
            PeakData::new(430, 440u64, 850u64, 610u64).unwrap().into(),
            PeakData::new(440, 430u64, 830u64, 650u64).unwrap().into(),
        ];
        let mut aggregator_no_merge = ConsensusPeakAggregator::from(start_peak_no_merge);
        for peak in peaks_no_merge {
            assert!(aggregator_no_merge.try_aggregate(peak).is_none());
        }

        assert_eq!(aggregator.number_aggregated_peaks(), 3);
        assert_eq!(aggregator.summit(), 63u64);
        assert_eq!(aggregator.length(), 42);

        // Adds a consensus peak that consists of multiple raw peaks.
        assert!(aggregator.try_aggregate(aggregator_merge).is_none());
        assert_eq!(aggregator.number_aggregated_peaks(), 6);
        assert_eq!(aggregator.summit(), 64u64);
        assert_eq!(aggregator.length(), 44);

        // Fails to add another peak.
        assert_eq!(
            aggregator.try_aggregate(aggregator_no_merge.clone()),
            Some(aggregator_no_merge)
        );
        assert_eq!(aggregator.number_aggregated_peaks(), 6);
        assert_eq!(aggregator.summit(), 64u64);
        assert_eq!(aggregator.length(), 44);

        let expected_consensus_peak = PeakData::new(42, 41u64, 84u64, 64u64).unwrap();
        let consensus: PeakData = aggregator.into();
        assert_eq!(consensus, expected_consensus_peak);
    }
}
