
extern crate csv;
extern crate plotters;
extern crate rand;

use std::{error::Error, fs::File, collections::HashMap};
use rand::Rng;

use plotters::prelude::*;

fn random_color() -> RGBColor {
    let mut rng = rand::thread_rng();
    RGBColor(rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255))
}

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

pub fn plot_audio_data(data: HashMap<String, Vec<f32>>) -> Result<(), Box<dyn Error>> {

    let root = BitMapBackend::new("plot.png", (4000, 4000)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Audio", ("Courier", 12))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u32..44100u32, -1f64..1f64)?;

    chart.configure_mesh().draw()?;

    for header in data.keys() {
        let line_data: Vec<f64> = data.get(header)
            .expect("Expect a key in keys() to be present in map.")
            .iter().map(|&x| x as f64).collect();
        let color = random_color();
        let mut xy_data: Vec<(u32, f64)> = Vec::new();

        for (x, y) in line_data.iter().enumerate() {
            xy_data.push((x.try_into().expect("expect this usize to fit into u32"), *y));
        }

        chart.draw_series(LineSeries::new(xy_data, color.stroke_width(4)))?
            .label(header)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x+20, y)], &color));
    }

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}