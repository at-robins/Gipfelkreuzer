//! This module parses input data.

use std::{borrow::Borrow, fs::File, io::BufReader, path::Path};

use bigtools::{BigWigRead, utils::reopen::ReopenableFile};

use crate::{data::PeakData, monotonicity::PeakCounts};

/// Parses BED3+ files according to the [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) definition.
///
/// # Parameters
///
/// * `path` - the input file path
pub fn parse_peak_file<T: AsRef<Path>>(path: T) -> Result<Vec<PeakData>, String> {
    let file = File::open(&path).map_err(|err| {
        format!(
            "The peak input file \"{}\" could not be opened: {}",
            path.as_ref().display(),
            err.to_string()
        )
    })?;
    let mut peaks = Vec::new();
    let mut record = noodles::bed::Record::default();
    let mut reader = noodles::bed::io::Reader::<3, BufReader<File>>::new(BufReader::new(file));
    while reader.read_record(&mut record).map_err(|err| {
        format!(
            "The peak input file \"{}\" could not be opened: {}",
            path.as_ref().display(),
            err.to_string()
        )
    })? > 0
    {
        peaks.push(PeakData::new(
            String::try_from(record.reference_sequence_name()).map_err(|err| {
                format!(
                    "Peak chromosome of peak input file \"{}\" could not be parsed: {}",
                    path.as_ref().display(),
                    err.to_string()
                )
            })?,
            // Start coordinate of record is 1 based.
            record
                .feature_start()
                .map(|pos| pos.get() - 1)
                .map_err(|err| {
                    format!(
                        "Peak start coordinate of peak input file \"{}\" could not be parsed: {}",
                        path.as_ref().display(),
                        err.to_string()
                    )
                })?,
            record
                .feature_end()
                .ok_or(format!(
                    "No end coordinate present for peak input file \"{}\".",
                    path.as_ref().display(),
                ))?
                .map(|pos| pos.get())
                .map_err(|err| {
                    format!(
                        "Peak start coordinate of peak input file \"{}\" could not be parsed: {}",
                        path.as_ref().display(),
                        err.to_string()
                    )
                })?,
        )?);
    }
    Ok(peaks)
}

/// Parses BED3+ files according to the [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) definition.
///
/// # Parameters
///
/// * `path` - the input file path
pub fn parse_count_file<T: AsRef<Path>>(path: T) -> Result<BigWigRead<ReopenableFile>, String> {
    let file = BigWigRead::open_file(&path).map_err(|err| {
        format!(
            "The count input file \"{}\" could not be opened: {}",
            path.as_ref().display(),
            err.to_string()
        )
    })?;

    Ok(file)
}

/// Reads [`PeakCounts`] form a count input file.
///
/// # Parameters
///
/// * `count_file` - the count input file
/// * `peak_region` - the region to get counts for
pub fn peak_counts_for_region<T: Borrow<PeakData>>(
    count_file: &mut BigWigRead<ReopenableFile>,
    peak_region: T,
) -> Result<PeakCounts, String> {
    let peak_data = peak_region.borrow();
    let count_file_path = count_file.inner_read().path.display().to_string();
    let interval = count_file
        .get_interval(peak_data.chromosome(), peak_data.start() as u32, peak_data.end() as u32)
        .map_err(|err| {
            format!(
                "The peak region {:?} could not be obtained from count input file \"{}\": {}",
                peak_data,
                count_file_path,
                err.to_string()
            )
        })?;

    let mut interval_values = Vec::new();
    for interval_value in interval {
        interval_values.push(interval_value.map_err(|err| format!("{}", err))?);
    }

    Ok(PeakCounts::try_from(interval_values)?)
}
