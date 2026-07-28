//! This module provides data types.

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
            return Err(
                format!(
                    "The peak end coordinate {} of on chromosome {} is smaller than the start coordinate {}.",
                    end, chromosome, start
                ),
            );
        }

        Ok(Self {
            chromosome,
            start,
            end,
        })
    }
}