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
            3 => bed_record.push_str(&format!("consensus_{}", peak.id())),
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

#[cfg(test)]
mod tests {
    use std::{
        fs::read_to_string,
        io::{BufRead, BufReader},
    };

    use crate::test_utils::test_output;

    use super::*;

    #[test]
    fn test_peak_to_bed_record_line() {
        let chromosome = "test_chromosome_42";
        let id = 21;
        let start: u64 = 42;
        let end: u64 = 84;
        let summit: u64 = 49;
        let peak = PeakData::new(id, start, end, summit).unwrap();
        assert_eq!(peak_to_bed_record_line(&peak, chromosome, 0), "");
        assert_eq!(peak_to_bed_record_line(&peak, chromosome, 1), format!("{}\n", chromosome));
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 2),
            format!("{}\t{}\n", chromosome, start)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 3),
            format!("{}\t{}\t{}\n", chromosome, start, end)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 4),
            format!("{}\t{}\t{}\tconsensus_{}\n", chromosome, start, end, id)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 5),
            format!("{}\t{}\t{}\tconsensus_{}\t0\n", chromosome, start, end, id)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 6),
            format!("{}\t{}\t{}\tconsensus_{}\t0\t.\n", chromosome, start, end, id)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 7),
            format!("{}\t{}\t{}\tconsensus_{}\t0\t.\t0\n", chromosome, start, end, id)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 8),
            format!("{}\t{}\t{}\tconsensus_{}\t0\t.\t0\t0\n", chromosome, start, end, id)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 9),
            format!("{}\t{}\t{}\tconsensus_{}\t0\t.\t0\t0\t0\n", chromosome, start, end, id)
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 10),
            format!(
                "{}\t{}\t{}\tconsensus_{}\t0\t.\t0\t0\t0\t{}\n",
                chromosome, start, end, id, summit
            )
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 11),
            format!(
                "{}\t{}\t{}\tconsensus_{}\t0\t.\t0\t0\t0\t{}\t0\n",
                chromosome, start, end, id, summit
            )
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 12),
            format!(
                "{}\t{}\t{}\tconsensus_{}\t0\t.\t0\t0\t0\t{}\t0\t0\n",
                chromosome, start, end, id, summit
            )
        );
        assert_eq!(
            peak_to_bed_record_line(&peak, chromosome, 13),
            format!(
                "{}\t{}\t{}\tconsensus_{}\t0\t.\t0\t0\t0\t{}\t0\t0\t0\n",
                chromosome, start, end, id, summit
            )
        );
    }

    #[test]
    fn test_write_peaks_to_bed_4_fields() {
        let n_fields = 4;
        let mut output_path = test_output();
        std::fs::create_dir_all(&output_path).unwrap();
        output_path.push("test_write_peaks_to_bed_4_fields.bed");
        let mut peaks = HashMap::new();
        peaks.insert("chr1".to_string(), vec![PeakData::new(0, 45u64, 98u64, 55u64).unwrap()]);
        peaks.insert("chr1".to_string(), vec![PeakData::new(10, 455u64, 983u64, 554u64).unwrap()]);
        peaks.insert("chr1".to_string(), vec![PeakData::new(11, 456u64, 986u64, 553u64).unwrap()]);
        peaks.insert(
            "chr4".to_string(),
            vec![PeakData::new(109, 4568u64, 9786u64, 5573u64).unwrap()],
        );
        write_peaks_to_bed(&output_path, &peaks, n_fields).unwrap();
        let output_file = BufReader::new(File::open(&output_path).unwrap());
        let expected_output_lines: Vec<String> = peaks
            .iter()
            .flat_map(|(chromosome, chromosome_peaks)| {
                chromosome_peaks.iter().map(move |peak| (chromosome, peak))
            })
            .map(|(chromosome, peak)| peak_to_bed_record_line(peak, chromosome, n_fields))
            .collect();
        for line in output_file.lines() {
            // Adds the new line character that was stripped during the read process.
            let line = format!("{}\n", line.unwrap());
            assert!(
                expected_output_lines.contains(&line),
                "The generated output file must contains the line \"{}\", but should only contain \"{:?}\"",
                line,
                expected_output_lines
            )
        }
        std::fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn test_write_peaks_to_bed_42_fields() {
        let n_fields = 42;
        let mut output_path = test_output();
        std::fs::create_dir_all(&output_path).unwrap();
        output_path.push("test_write_peaks_to_bed_42_fields.bed");
        let mut peaks = HashMap::new();
        peaks.insert("chr1".to_string(), vec![PeakData::new(0, 45u64, 98u64, 55u64).unwrap()]);
        peaks.insert("chr1".to_string(), vec![PeakData::new(10, 455u64, 983u64, 554u64).unwrap()]);
        peaks.insert("chr1".to_string(), vec![PeakData::new(11, 456u64, 986u64, 553u64).unwrap()]);
        peaks.insert(
            "chr4".to_string(),
            vec![PeakData::new(109, 4568u64, 9786u64, 5573u64).unwrap()],
        );
        write_peaks_to_bed(&output_path, &peaks, n_fields).unwrap();
        let output_file = BufReader::new(File::open(&output_path).unwrap());
        let expected_output_lines: Vec<String> = peaks
            .iter()
            .flat_map(|(chromosome, chromosome_peaks)| {
                chromosome_peaks.iter().map(move |peak| (chromosome, peak))
            })
            .map(|(chromosome, peak)| peak_to_bed_record_line(peak, chromosome, n_fields))
            .collect();
        for line in output_file.lines() {
            // Adds the new line character that was stripped during the read process.
            let line = format!("{}\n", line.unwrap());
            assert!(
                expected_output_lines.contains(&line),
                "The generated output file must contains the line \"{}\", but should only contain \"{:?}\"",
                line,
                expected_output_lines
            )
        }
        std::fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn test_write_peaks_to_bed_0_fields() {
        let n_fields = 0;
        let mut output_path = test_output();
        std::fs::create_dir_all(&output_path).unwrap();
        output_path.push("test_write_peaks_to_bed_0_fields.bed");
        let mut peaks = HashMap::new();
        peaks.insert("chr1".to_string(), vec![PeakData::new(0, 45u64, 98u64, 55u64).unwrap()]);
        peaks.insert("chr1".to_string(), vec![PeakData::new(10, 455u64, 983u64, 554u64).unwrap()]);
        peaks.insert("chr1".to_string(), vec![PeakData::new(11, 456u64, 986u64, 553u64).unwrap()]);
        peaks.insert(
            "chr4".to_string(),
            vec![PeakData::new(109, 4568u64, 9786u64, 5573u64).unwrap()],
        );
        write_peaks_to_bed(&output_path, &peaks, n_fields).unwrap();
        let output_content = read_to_string(&output_path).unwrap();
        assert!(
            output_content.is_empty(),
            "The output file should be empty but contains \"{}\".",
            output_content
        );
        std::fs::remove_file(output_path).unwrap();
    }
}
