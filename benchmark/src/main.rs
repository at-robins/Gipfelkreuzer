use std::path::PathBuf;

use clap::Parser;

use crate::{arguments::CommandLineArguments, output::OutputData};

fn main() -> Result<(), String> {
    let clas = CommandLineArguments::parse();

    let mut output_data = Vec::new();
    for peak_file in clas.input_file_peaks() {
        output_data
            .push(OutputData::compute::<&PathBuf, &PathBuf>(peak_file, clas.input_file_counts())?);
    }
    OutputData::save_results(&output_data, clas.output_file())
}

mod arguments;
mod data;
mod input;
mod monotonicity;
mod output;
