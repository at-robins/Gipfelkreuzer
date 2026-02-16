//! This module contains the specifics of the consensus peak generation algorithm
//! supposed by [Cherchame 2025](https://www.protocols.io/view/atac-seq-methods-for-consensus-peak-generation-to-36wgq326olk5/v1).

use std::u64;

use crate::{
    error::ApplicationError,
    peaks::{PeakBin, PeakData},
};

/// Creates harmonised consensus peaks from raw peaks based on summit proximity.
///
/// # Parameters
///
/// * `peaks` - the raw input peaks
/// * `harmonising_distance` - the distance from the summit that is considered a harmonised peak region
/// * `min_peaks_per_consensus` - the minimum number of raw peaks required to form a consensus peak
///
/// # Error
///
/// Returns an error if the creation of a consensus peaks fails.
pub fn harmonised_consensus_peaks(
    peaks: Vec<PeakData>,
    harmonising_distance: u64,
    min_peaks_per_consensus: usize,
) -> Result<Vec<PeakData>, ApplicationError> {
    let mut consensus_peaks = Vec::new();
    let peaks = peaks
        .into_iter()
        .map(|peak| harmonise_peak(peak, harmonising_distance))
        .collect();

    for (bin_index, bin) in PeakBin::bin_peaks(peaks)
        .into_iter()
        .filter(|bin| bin.peaks().len() >= min_peaks_per_consensus)
        .enumerate()
    {
        consensus_peaks.push(
            PeakData::new(bin_index, bin.start(), bin.end(), bin.start().midpoint(bin.end()))
                .map_err(|err| {
                    err.chain(format!(
                        "Failed to create a harmonised consensus peak from peak bin {}: {:?}",
                        bin_index, bin
                    ))
                })?,
        );
    }
    Ok(consensus_peaks)
}

/// Harmonises the [`PeakData`] by setting start and end coordinates
/// based on a fixed distance from the summit.
///
/// # Parameters
///
/// * `peak` - the peak to harmonise
/// * `distance` - the fixed distance of start and end coordinate from the peak summit
fn harmonise_peak(peak: PeakData, distance: u64) -> PeakData {
    let summit = peak.summit();
    let start = summit.checked_sub(distance).unwrap_or(0);
    let end = summit.checked_add(distance).unwrap_or(u64::MAX);
    PeakData {
        id: peak.id(),
        start,
        end,
        summit,
    }
}

#[cfg(test)]
mod tests {
    use std::u64;

    use super::*;

    #[test]
    fn test_harmonise_peak() {
        let peak_normal = PeakData::new(1, 1000u64, 2000u64, 1500u64).unwrap();
        let peak_harmonised = harmonise_peak(peak_normal, 250);
        let peak_expected = PeakData::new(1, 1250u64, 1750u64, 1500u64).unwrap();
        assert_eq!(peak_harmonised, peak_expected);
    }

    #[test]
    fn test_harmonise_peak_0() {
        let peak_normal = PeakData::new(1, 42u64, 176u64, 175u64).unwrap();
        let peak_harmonised = harmonise_peak(peak_normal, 250);
        let peak_expected = PeakData::new(1, 0u64, 425u64, 175u64).unwrap();
        assert_eq!(peak_harmonised, peak_expected);
    }

    #[test]
    fn test_harmonise_peak_max() {
        let peak_normal = PeakData::new(1, u64::MAX - 133, u64::MAX - 100, u64::MAX - 120).unwrap();
        let peak_harmonised = harmonise_peak(peak_normal, 250);
        let peak_expected = PeakData::new(1, u64::MAX - 370, u64::MAX, u64::MAX - 120).unwrap();
        assert_eq!(peak_harmonised, peak_expected);
    }

    #[test]
    fn test_harmonised_consensus_peaks() {
        let harmonising_distance = 250;
        let peaks = vec![
            PeakData::new(0, 12u64, 22u64, 18u64).unwrap(),
            PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
            PeakData::new(7, 13u64, 22u64, 16u64).unwrap(),
            PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
            PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
            PeakData::new(4, 270u64, 290u64, 277u64).unwrap(),
            PeakData::new(5, 271u64, 291u64, 276u64).unwrap(),
            PeakData::new(6, 2700u64, 2900u64, 2770u64).unwrap(),
        ];
        let consensus = harmonised_consensus_peaks(peaks, harmonising_distance, 0).unwrap();

        let expected_consensus_peaks = vec![
            PeakData::new(0, 0u64, 527u64, 263u64).unwrap(),
            PeakData::new(
                1,
                2770u64 - harmonising_distance,
                2770u64 + harmonising_distance,
                2770u64,
            )
            .unwrap(),
        ];
        assert_eq!(consensus, expected_consensus_peaks);
    }

    #[test]
    fn test_harmonised_consensus_filter() {
        let harmonising_distance = 250;
        let peaks = vec![
            PeakData::new(0, 12u64, 22u64, 18u64).unwrap(),
            PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
            PeakData::new(7, 13u64, 22u64, 16u64).unwrap(),
            PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
            PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
            PeakData::new(4, 270u64, 290u64, 277u64).unwrap(),
            PeakData::new(5, 271u64, 291u64, 276u64).unwrap(),
            PeakData::new(6, 2700u64, 2900u64, 2770u64).unwrap(),
        ];
        {
            let consensus =
                harmonised_consensus_peaks(peaks.clone(), harmonising_distance, 0).unwrap();

            let expected_consensus_peaks = vec![
                PeakData::new(0, 0u64, 527u64, 263u64).unwrap(),
                PeakData::new(
                    1,
                    2770u64 - harmonising_distance,
                    2770u64 + harmonising_distance,
                    2770u64,
                )
                .unwrap(),
            ];
            assert_eq!(consensus, expected_consensus_peaks);
        }
        {
            let consensus = harmonised_consensus_peaks(peaks, harmonising_distance, 2).unwrap();

            let expected_consensus_peaks = vec![PeakData::new(0, 0u64, 527u64, 263u64).unwrap()];
            assert_eq!(consensus, expected_consensus_peaks);
        }
    }

    #[test]
    fn test_harmonised_consensus_distance() {
        let peaks = vec![
            PeakData::new(0, 100u64, 200u64, 150u64).unwrap(),
            PeakData::new(1, 300u64, 400u64, 350u64).unwrap(),
        ];
        {
            let consensus = harmonised_consensus_peaks(peaks.clone(), 75, 0).unwrap();

            let expected_consensus_peaks = vec![
                PeakData::new(0, 75u64, 225u64, 150u64).unwrap(),
                PeakData::new(1, 275u64, 425u64, 350u64).unwrap(),
            ];
            assert_eq!(consensus, expected_consensus_peaks);
        }
        {
            let consensus = harmonised_consensus_peaks(peaks, 110, 0).unwrap();

            let expected_consensus_peaks = vec![PeakData::new(0, 40u64, 460u64, 250u64).unwrap()];
            assert_eq!(consensus, expected_consensus_peaks);
        }
    }
}
