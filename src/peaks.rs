//! This module defines operations on genomic peak data.

use crate::{
    arguments::CommandLineArguments,
    error::{ApplicationError, ApplicationErrorType},
    peaks::gipfelkreuzer::GipfelkreuzerPeakMerger,
};
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

#[derive(CopyGetters, Getters, PartialEq, Debug)]
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

    /// Internal function that inserts a peak into the bin without bound checks
    /// and updates the respective start and end of the peak bin.
    /// 
    /// # Parameters
    /// 
    /// * `peak_data` - the peak to insert into the bin
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
}

impl From<PeakBin> for Vec<PeakData> {
    fn from(value: PeakBin) -> Self {
        value.peaks
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
/// A general definition of an algorithm that generates a set of consensus
/// [`PeakData`] from raw input peaks.
pub enum ConsensusPeakAlgorithm {
    /// The Gipfelkreuzer algorithm.
    /// Creates consensus peaks by iteratively merging smaller peaks with larger peaks
    /// when their summits are in close proximity.
    Gipfelkreuzer,
    /// A simple merge algorithm.
    /// Overlapping and adjacent peaks are merged to create consensus peaks.
    Simple,
}

impl ConsensusPeakAlgorithm {
    /// Creates consensus peaks from the specified raw input peaks.
    /// Returns and error if the consensus finding failed.
    ///
    /// `peaks` - the raw input peaks to create consensus peaks from
    /// `algorithm_arguments` - the passed [`CommandLineArguments`] to customise the algorithm
    pub fn consensus_peaks(
        &self,
        peaks: Vec<PeakData>,
        algorithm_arguments: &CommandLineArguments,
    ) -> Result<Vec<PeakData>, ApplicationError> {
        match self {
            ConsensusPeakAlgorithm::Gipfelkreuzer => Ok(GipfelkreuzerPeakMerger::new(peaks)
                .consensus_peaks(algorithm_arguments.max_merge_iterations())),
            ConsensusPeakAlgorithm::Simple => simple::merge_peaks(peaks),
        }
    }
}

impl std::fmt::Display for ConsensusPeakAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ConsensusPeakAlgorithm::Gipfelkreuzer => "gipfelkreuzer",
            ConsensusPeakAlgorithm::Simple => "simple",
        };
        write!(f, "{}", name)
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

pub mod gipfelkreuzer;
pub mod simple;

#[cfg(test)]
mod tests;
