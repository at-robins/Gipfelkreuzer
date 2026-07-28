//! This module parses input data.

use std::{fs::File, io::BufReader, path::Path};

use crate::data::PeakData;

/// Parses BED3+ files according to the [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) definition.
/// Peak summit information will be extracted from field 10 according to the
/// [narrowPeak](https://genome.ucsc.edu/FAQ/FAQformat.html#format12) format definition if present and possible.
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
