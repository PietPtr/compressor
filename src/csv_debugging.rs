
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

/// Logs the operation of an audio plugin to a CSV file. Requires that every line in the CSV is at the same time/sample.
pub struct SampleLogger {
    debug_values: HashMap<String, Vec<f32>>,
    samples_seen: u64,
    quit_after_n_samples: u64,
}

impl SampleLogger {
    pub fn new(quit_after_n_samples: u64) -> Self {
        Self {
            debug_values: HashMap::new(),
            samples_seen: 0,
            quit_after_n_samples,
        }
    }

    pub fn write(&mut self, key: &str, value: f32) -> Result<(), &'static str>{
        self.debug_values.entry(String::from(key))
            .or_insert(Vec::new())
            .push(value);

        if key == "sample" {
            self.samples_seen += 1;
        }

        if self.samples_seen > self.quit_after_n_samples {
            return Err("Seen enough samples.");
        }

        self.is_logged_correctly()
    }

    fn is_logged_correctly(&self) -> Result<(), &'static str> {
        let lengths: Vec<usize> = self.debug_values.values().into_iter()
            .map(|vec| vec.len()).collect();

        // Checks whether all lists have n or n+1 elements.
        let n = lengths.iter().sum::<usize>() / lengths.len();
        if !lengths.iter().all(|&elem| elem == n || elem == n + 1) {
            return Err("Element added to list caused imbalance.");
        }

        if n > 1 && !self.debug_values.keys().any(|elem| elem.eq("sample")) {
            dbg!(self.debug_values.keys());
            return Err("First sample iteration has been added but no key 'sample' is present.");
        }

        Ok(())
    }

    pub fn write_debug_values(&mut self) -> Result<(), Box<dyn Error>> {
        let max_len = self
            .debug_values
            .values()
            .map(|v| v.len())
            .max()
            .unwrap_or(0);

        let file = File::create("debug.csv")?;
        let mut writer = csv::Writer::from_writer(file);

        writer.write_record(self.debug_values.keys())?;

        for i in 0..max_len {
            let mut record = csv::StringRecord::new();
            for value in self.debug_values.values() {
                let entry = value.get(i).map(|v| v.to_string()).unwrap_or(String::new());
                record.push_field(entry.as_str());
            }
            writer.write_record(&record)?;
        }

        Ok(())
    }
}