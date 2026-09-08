use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::Parser;

use rayon::prelude::*;

use crate::{arguments::CommandLineArguments, output::OutputData};

fn main() -> Result<(), String> {
    let clas = CommandLineArguments::parse();

    let output_data = Arc::new(Mutex::new(Vec::new()));
    clas.input_file_peaks().par_iter().for_each(|peak_file| {
        let result_output =
            OutputData::compute::<&PathBuf, &PathBuf>(peak_file, clas.input_file_counts()).unwrap();
        output_data.lock().unwrap().push(result_output);
    });
    OutputData::save_results(&output_data.lock().unwrap(), clas.output_file())
}

mod arguments;
mod data;
mod input;
mod monotonicity;
mod output;
