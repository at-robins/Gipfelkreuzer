//! This module defines operations on genomic peak data.

use crate::error::{ApplicationError, ApplicationErrorType};
use getset::{CopyGetters, Getters};

#[derive(CopyGetters, Clone, Copy, PartialEq, Eq, Debug)]
/// Data representing a peak region on genomic data.
pub struct PeakData {
    /// The unique identifier of the peak.
    #[getset(get_copy = "pub")]
    id: usize,
    /// The genomic start coordinate of the peak region.
    #[getset(get_copy = "pub")]
    start: u64,
    /// The genomic end coordinate of the peak region (inclusive).
    #[getset(get_copy = "pub")]
    end: u64,
    /// The genomic coordinate of the peak summit.
    #[getset(get_copy = "pub")]
    summit: u64,
}

impl PeakData {
    /// Creates a new genomic peak region.
    ///
    /// # Parameters
    ///
    /// * `id`: the unique identifier of the peak
    /// * `start`: the start location of the peak region
    /// * `end`: the end location of the peak region (inclusive)
    /// * `summit`: the peak summit within the peak region
    ///
    /// # Errors
    ///
    /// Returns an error if the summit is without the defined region or
    /// if the start coordinate is greater than the end coordinate.
    pub fn new<StartType: Into<u64>, EndType: Into<u64>, SummitType: Into<u64>>(
        id: usize,
        start: StartType,
        end: EndType,
        summit: SummitType,
    ) -> Result<Self, ApplicationError> {
        let start = start.into();
        let end = end.into();
        let summit = summit.into();

        if start > end {
            return Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "The end coordinate {} of peak {} is smaller than the start coordinate {}.",
                    end, id, start
                ),
            ));
        }

        if summit < start || summit > end {
            return Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "The summit {} of peak {} is not within the peak region [{}, {}].",
                    summit, id, start, end
                ),
            ));
        }

        Ok(Self {
            id,
            start,
            end,
            summit,
        })
    }

    /// Returns the length of the genomic peak region.
    pub fn length(&self) -> u64 {
        self.end() + 1 - self.start()
    }
}

#[derive(CopyGetters, Getters, PartialEq)]
/// A bin containing overlapping or adjacent peaks.
pub struct PeakBin {
    #[getset(get_copy = "pub")]
    start: u64,
    #[getset(get_copy = "pub")]
    end: u64,
    #[getset(get = "pub")]
    peaks: Vec<PeakData>,
}

impl PeakBin {
    /// Creates a new bin containing adjacent and overlapping peaks starting with a single peak.
    ///
    /// # Parameters
    ///
    /// * `peak_data` - the initial peak to start the bin with
    pub fn new(peak_data: PeakData) -> Self {
        Self {
            start: peak_data.start(),
            end: peak_data.end(),
            peaks: vec![peak_data],
        }
    }

    fn insert(&mut self, peak_data: PeakData) {
        if peak_data.start() < self.start() {
            self.start = peak_data.start();
        }
        if peak_data.end() > self.end() {
            self.end = peak_data.end();
        }
        self.peaks.push(peak_data);
    }

    /// Checks if the peak is overlapping or adjacent to the bin and inserts it by consuming it.
    /// If the peak is not, it will be returned without being inserted.
    ///
    /// # Parameters
    ///
    /// * `peak_data` - the peak that should be probed for insertion
    pub fn try_insert(&mut self, peak_data: PeakData) -> Option<PeakData> {
        if is_continuous_range(self.start(), self.end(), peak_data.start(), peak_data.end()) {
            self.insert(peak_data);
            None
        } else {
            Some(peak_data)
        }
    }

    /// Converts the peak bin into its respective consensus peaks.
    ///
    /// # Parameters
    ///
    /// * `max_iterations` - the maximum number of peak merging iterations to be performed
    fn consensus_peaks(self, max_iterations: usize) -> Vec<PeakData> {
        let mut consensus = Self::consensus_peaks_internal(self.peaks);
        // Iterativesly merges peaks until the maximum number of iterations is reached
        // or the peaks do not change anymore.
        let previous_consensus_length = consensus.len();
        for _ in 0..max_iterations {
            consensus = Self::consensus_peaks_internal(consensus);
            if consensus.len() == previous_consensus_length {
                break;
            }
        }
        consensus
    }

    /// Converts the peak bin into its respective consensus peaks.
    /// Internal function logic to allow easy iterative consensus peak generation.
    fn consensus_peaks_internal(mut peaks: Vec<PeakData>) -> Vec<PeakData> {
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
                    consensus_peak_aggregator = Some(ConsensusPeakAggregator::new(peak));
                }
            }

            consensus_peaks.push(
                consensus_peak_aggregator
                    .expect("The consensus aggregator must have been created at this point.")
                    .consensus_peak(),
            );
            remaining_peaks = retained_peaks;
        }
        consensus_peaks
    }
}

struct ConsensusPeakAggregator {
    peaks: Vec<PeakData>,
}

impl ConsensusPeakAggregator {
    fn new(peak: PeakData) -> Self {
        Self { peaks: vec![peak] }
    }

    fn defining_peak(&self) -> &PeakData {
        self.peaks
            .first()
            .expect("There must have been a peak set during initialisation.")
    }

    fn consensus_id(&self) -> usize {
        self.defining_peak().id()
    }

    fn try_aggregate(&mut self, peak: PeakData) -> Option<PeakData> {
        let defining_peak = self.defining_peak();
        if peak.summit() <= defining_peak.end() && peak.summit() >= defining_peak.start() {
            self.peaks.push(peak);
            None
        } else {
            Some(peak)
        }
    }

    fn start(&self) -> u64 {
        let starts: Vec<u64> = self.peaks.iter().map(PeakData::start).collect();
        Self::u64_median(starts)
    }

    fn end(&self) -> u64 {
        let ends: Vec<u64> = self.peaks.iter().map(PeakData::end).collect();
        Self::u64_median(ends)
    }

    fn summit(&self) -> u64 {
        let ends: Vec<u64> = self.peaks.iter().map(PeakData::summit).collect();
        Self::u64_median(ends)
    }

    fn consensus_peak(&self) -> PeakData {
        PeakData::new(self.consensus_id(), self.start(), self.end(), self.summit()).expect(
            "The consensus peak parameters must be valid as they were derived from valid peaks.",
        )
    }

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
}

pub struct PeakMerger {
    bins: Vec<PeakBin>,
}

impl PeakMerger {
    /// Merges adjacent and overlapping peaks into
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

    pub fn consensus_peaks(self, max_iterations: usize) -> Vec<PeakData> {
        let mut consensus_peaks = Vec::new();
        for bin in self.bins {
            consensus_peaks.extend(bin.consensus_peaks(max_iterations));
        }
        consensus_peaks
    }
}

/// Returns true if both ranges are either overlapping or directly adjacent.
///
/// # Panics
///
/// Panics if either start is after its respective end.
fn is_continuous_range(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> bool {
    if a_start > a_end || b_start > b_end {
        panic!(
            "Invalid ranges while comparing for continuity: A[{}, {}], B[{}, {}]",
            a_start, a_end, b_start, b_end
        )
    }
    b_start <= a_end + 1 && b_end + 1 >= a_start
}

#[cfg(test)]
mod tests;
