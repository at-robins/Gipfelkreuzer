//! This module writes output data.

use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    peaks::PeakData,
};

/// Creates a BED record line according to the
/// [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) standard
/// from [`PeakData`] and the respective chromosome name.
///
/// # Parameters
///
/// * `peak` - the peak data
/// * `chromosome`- the name of the chromosome the peak belongs to
/// * `fields`- the number of fields / columns to generate
pub fn peak_to_bed_record_line(peak: &PeakData, chromosome: &str, fields: usize) -> String {
    let mut bed_record = String::new();
    for field_index in 0..fields {
        match field_index {
            // Chromosome
            0 => bed_record.push_str(chromosome),
            // Start
            1 => bed_record.push_str(&peak.start().to_string()),
            // End
            2 => bed_record.push_str(&peak.end().to_string()),
            // Name
            3 => bed_record.push_str(&peak.id().to_string()),
            // Strand
            5 => bed_record.push('.'),
            9 => bed_record.push_str(&peak.summit().to_string()),
            _ => bed_record.push('0'),
        };
        if field_index < fields - 1 {
            bed_record.push('\t');
        } else {
            bed_record.push('\n');
        }
    }
    bed_record
}

///  Writes all peaks to the specified file using the
/// [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf)
/// standard.
///
/// # Parameters
/// * `path`- the path of the output file
/// * `peaks` - all peaks sorted by chromosome
/// * `fields`- the number of fields / columns to generate
///
/// # Errors
/// Returns an error if the output file path is invalid or if
/// creation of the output file failed.
pub fn write_peaks_to_bed<T: AsRef<Path>>(
    path: T,
    peaks: &HashMap<String, Vec<PeakData>>,
    fields: usize,
) -> Result<(), ApplicationError> {
    // Creates the specified output path.
    let parent_directory = path.as_ref().parent().ok_or(ApplicationError::new(
        ApplicationErrorType::OutputOperationError,
        format!("The output file path \"{}\" is invalid.", path.as_ref().display()),
    ))?;
    std::fs::create_dir_all(parent_directory).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "The output directory \"{}\" could not be created.",
            parent_directory.display()
        ))
    })?;

    // Creates the output file.
    let mut file = File::create(&path).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("The output file \"{}\" could not created.", path.as_ref().display()))
    })?;

    // Writes the records to the file.
    for (chromosome, chromosome_peaks) in peaks {
        for peak in chromosome_peaks {
            let peak_record = peak_to_bed_record_line(peak, chromosome, fields);
            file.write_all(peak_record.as_bytes()).map_err(|err| {
                ApplicationError::from(err).chain(format!(
                    "Writing record \"{}\" to output file \"{}\" failed.",
                    peak_record,
                    path.as_ref().display()
                ))
            })?;
        }
    }
    Ok(())
}
