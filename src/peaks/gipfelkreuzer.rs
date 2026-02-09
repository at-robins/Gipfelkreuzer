//! This module contains the specifics of the Gipfelkreuzer consensus peak generation algorithm.

use getset::Getters;

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

#[derive(Debug)]
struct ConsensusPeakAggregator {
    peaks: Vec<PeakData>,
    consensus_peak: PeakData,
}

impl ConsensusPeakAggregator {
    fn id(&self) -> usize {
        self.consensus_peak.id()
    }

    fn try_aggregate(&mut self, peak: ConsensusPeakAggregator) -> Option<ConsensusPeakAggregator> {
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

    fn summit(&self) -> u64 {
        self.consensus_peak.summit()
    }

    fn length(&self) -> u64 {
        self.consensus_peak.length()
    }

    /// Returns the number of aggregated peaks used to create this consensus peak.
    pub fn number_aggregated_peaks(&self) -> usize {
        self.peaks.len()
    }

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

#[derive(Getters)]
pub struct GipfelkreuzerPeakMerger {
    #[getset(get = "pub")]
    bins: Vec<PeakBin>,
}

impl GipfelkreuzerPeakMerger {
    /// Merges adjacent and overlapping peaks into consensus peaks based on their summit proximity.
    pub fn new(mut peaks: Vec<PeakData>) -> Self {
        log::info!("Creating a peak merger with {} peaks.", peaks.len());
        log::debug!("Sorting peaks by start position.");
        peaks.sort_by(|a, b| a.start().cmp(&b.start()));
        let mut bins: Vec<PeakBin> = Vec::new();
        log::debug!("Inserting peaks...");
        for peak in peaks {
            log::debug!("Inserting peak {:?}...", peak);
            if let Some(current_bin) = bins.last_mut() {
                log::debug!("Checking bin [{}, {}]...", current_bin.start(), current_bin.end());
                if let Some(peak) = current_bin.try_insert(peak) {
                    // Creates a new bin if the insertion failed into the old one.
                    log::debug!("Creating new peak bin for peak {:?}.", peak);
                    bins.push(PeakBin::new(peak));
                } else {
                    log::debug!(
                        "Inserted peak into bin [{}, {}]",
                        current_bin.start(),
                        current_bin.end()
                    );
                }
            } else {
                // Creates an initial bin if there are none yet.
                log::debug!("Creating initial peak bin...");
                bins.push(PeakBin::new(peak));
            }
        }
        Self { bins }
    }

    pub fn consensus_peaks(
        self,
        max_iterations: usize,
        min_peaks_per_consensus: usize,
    ) -> Vec<PeakData> {
        let mut consensus_peaks = Vec::new();
        for bin in self.bins {
            consensus_peaks.extend(bin_to_consensus_peaks(
                bin,
                max_iterations,
                min_peaks_per_consensus,
            ));
        }
        consensus_peaks
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
}
