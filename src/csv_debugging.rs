
extern crate csv;

use std::{error::Error, fs::File, collections::HashMap};

pub fn read_csv_as_audio_data(filename: String) -> Result<HashMap<String, Vec<f32>>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(File::open(filename.as_str())?);
    let headers = reader.headers()?.clone();
    let mut data = HashMap::new();

    for record in reader.records() {
        let record = record?;
        let row: Vec<f64> = record.iter().map(|x| x.parse().expect("parse error on reading csv")).collect();

        for (i, header) in headers.iter().enumerate() {
            data.entry(String::from(header))
                .or_insert(Vec::new())
                .push(row[i] as f32);
        }
    }

    Ok(data)
}
