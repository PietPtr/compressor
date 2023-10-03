extern crate csv;

use nih_plug::prelude::*;
use std::{collections::HashMap, error::Error, fs::File, sync::Arc};

pub mod csv_debugging;

pub struct Compressor {
    params: Arc<CompressorParams>,
    sample_rate: f32,
    envelope: f32,

    #[cfg(feature = "detailed_debugging")]
    debug_values: HashMap<String, Vec<f32>>,
    #[cfg(feature = "detailed_debugging")]
    samples_seen: u64,
}

#[derive(Params)]
struct CompressorParams {
    #[id = "threshold"]
    pub threshold: FloatParam, // stored as gain, entered in dB

    #[id = "ratio"]
    pub ratio: FloatParam, // [1, inf)

    #[id = "attack"]
    pub attack: FloatParam, // [0, inf), milliseconds

    #[id = "release"]
    pub release: FloatParam, // [0, inf), milliseconds
}

impl Compressor {
    #[cfg(feature = "detailed_debugging")]
    fn write_debug_values(&mut self) -> Result<(), Box<dyn Error>> {
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

impl Default for Compressor {
    fn default() -> Self {
        Self {
            params: Arc::new(CompressorParams::default()),
            sample_rate: 48000.0,
            envelope: 0.0,

            #[cfg(feature = "detailed_debugging")]
            debug_values: HashMap::new(),
            #[cfg(feature = "detailed_debugging")]
            samples_seen: 0,
        }
    }
}

//TODO: apply gain after compression to normalize?
impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                util::db_to_gain(-7.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(0.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 0.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            ratio: FloatParam::new(
                "Ratio",
                4.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 200.0,
                    factor: FloatRange::gain_skew_factor(1.0, 200.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            attack: FloatParam::new(
                "Attack",
                10.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_unit(" ms"),

            release: FloatParam::new(
                "Release",
                10.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_unit(" ms"),
        }
    }
}

impl Plugin for Compressor {
    const NAME: &'static str = "Compressor";
    const VENDOR: &'static str = "Staal";
    const URL: &'static str = "example.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // TODO: not the right place for this
        let mut add_to_debug_values = |key: &str, value: f32| {
            self.debug_values.entry(String::from(key))
                .or_insert(Vec::new())
                .push(value);
        };

        for channel_samples in buffer.iter_samples() {
            #[cfg(feature = "detailed_debugging")]
            {
                self.samples_seen += channel_samples.len() as u64; // TODO: broken when several channels present
                if self.samples_seen > 50000 {
                    self.write_debug_values();
                    return ProcessStatus::Normal; // TODO: error?
                }
            }

            let threshold = self.params.threshold.smoothed.next();
            let ratio = 1.0 / self.params.ratio.smoothed.next();
            let attack = self.params.attack.smoothed.next() / 1000.0;
            let release = self.params.release.smoothed.next() / 1000.0;

            let attack_slope = 1.0 / (self.sample_rate * attack);
            let release_slope = 1.0 / (self.sample_rate * release);

            for sample in channel_samples {
                add_to_debug_values("before", *sample);

                let abs_sample = (*sample).abs();
                if abs_sample > self.envelope {
                    self.envelope += attack_slope;
                } else if abs_sample < self.envelope {
                    self.envelope -= release_slope;
                }

                if self.envelope > threshold && *sample > threshold {
                    *sample = threshold + (*sample - threshold) * ratio;
                } else if -self.envelope < -threshold && *sample < -threshold {
                    *sample = -(threshold + (abs_sample - threshold) * ratio);
                }

                add_to_debug_values("after", *sample);
            }
        }

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

impl Vst3Plugin for Compressor {
    const VST3_CLASS_ID: [u8; 16] = *b"Compressor      ";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(Compressor);
