use std::error::Error;

use nih_plug::prelude::*;

use compressor::{Compressor, csv_debugging};

fn main() -> Result<(), Box<dyn Error>> {
    if cfg!(feature = "detailed_debugging") {
        let csv_audio_data = csv_debugging::read_csv_as_audio_data(String::from("debug.csv"))?;
        csv_debugging::plot_audio_data(csv_audio_data)?;
    } else {
        nih_export_standalone::<Compressor>();
    }

    Ok(())
}
