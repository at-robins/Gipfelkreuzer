//! This module defines operations on genomic peak data.

use crate::{
    arguments::CommandLineArguments,
    error::{ApplicationError, ApplicationErrorType}, peaks::gipfelkreuzer::GipfelkreuzerPeakMerger,
};
use getset::{CopyGetters};

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

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
/// A general definition of an algorithm that generates a set of consensus
/// [`PeakData`] from raw input peaks.
pub enum ConsensusPeakAlgorithm {
    Gipfelkreuzer,
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
        }
    }
}

impl std::fmt::Display for ConsensusPeakAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ConsensusPeakAlgorithm::Gipfelkreuzer => "Gipfelkreuzer",
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

#[cfg(test)]
mod tests;
