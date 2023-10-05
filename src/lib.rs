extern crate csv;

use csv_debugging::SampleLogger;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

pub mod csv_debugging;
mod editor;

pub struct Compressor {
    params: Arc<CompressorParams>,
    sample_rate: f32,
    envelope: f32,
    logger: SampleLogger,
}

#[derive(Params)]
struct CompressorParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "threshold"]
    pub threshold: FloatParam, // stored as gain, entered in dB
    #[id = "ratio"]
    pub ratio: FloatParam, // [1, inf)
    #[id = "attack"]
    pub attack: FloatParam, // [0, inf), milliseconds
    #[id = "release"]
    pub release: FloatParam, // [0, inf), milliseconds
    #[id = "steepness"]
    pub steepness: FloatParam, // [0, inf)

    #[cfg(feature = "detailed_debugging")]
    #[id = "logger_length"]
    pub logger_length: FloatParam,
}

impl Compressor {
    fn process_buffer(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> Result<(), &'static str> {
        #[cfg(feature = "detailed_debugging")]
        self.logger.set_quit_after_n_samples(self.params.logger_length.value());

        for channel_samples in buffer.iter_samples() {
            // TODO: can be seen from audio layout right?
            #[cfg(feature = "detailed_debugging")] {
                if channel_samples.len() > 1 {
                    panic!("Too many channels for detailed debugging to support: {:?}", channel_samples.len());
                }
            }

            // TODO: make smoothed.next() instead of value
            let threshold = self.params.threshold.smoothed.next();
            let ratio_denom = self.params.ratio.smoothed.next();
            let attack = self.params.attack.smoothed.next() / 1000.0;
            let release = self.params.release.smoothed.next() / 1000.0;
            let steepness = self.params.steepness.smoothed.next();

            let attack_slope = 1.0 / (self.sample_rate * attack);
            let release_slope = 1.0 / (self.sample_rate * release);

            let envelope_scaler = 1.0 / (1.0 - threshold);

            for sample in channel_samples {
                self.logger.write("sample", *sample)?;
                self.logger.write("sample.abs()", (*sample).abs())?;
                self.logger.write("envelope", self.envelope)?;
                self.logger.write("threshold", threshold)?;
                self.logger.write("-threshold", -threshold)?;

                let abs_sample = (*sample).abs();

                self.envelope = if abs_sample > self.envelope {
                    (self.envelope + attack_slope).min(abs_sample)
                } else if abs_sample < self.envelope {
                    (self.envelope - release_slope).max(abs_sample)
                } else {
                    self.envelope
                };
                
                let ratio = 1.0 / (((self.envelope - threshold) * envelope_scaler) * (ratio_denom - 1.0) + 1.0);

                let wet = if self.envelope > threshold && *sample > threshold {
                    threshold + (*sample - threshold) * ratio
                } else if -self.envelope < -threshold && *sample < -threshold {
                    -(threshold + (abs_sample - threshold) * ratio)
                } else {
                    *sample
                };

                let sigmoid = |x: f32| 1.0 / (1.0 + (steepness * x).exp());

                let distance_from_threshold = threshold - abs_sample;

                let mix = sigmoid(distance_from_threshold);
                *sample = *sample * (1.0 - mix) + wet * mix;

                self.logger.write("mix", mix)?;
                self.logger.write("after", *sample)?;
            }
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
            logger: SampleLogger::new(),
        }
    }
}

//TODO: apply gain after compression to normalize?
impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
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
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            ratio: FloatParam::new(
                "Ratio",
                2.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 200.0,
                    factor: FloatRange::gain_skew_factor(1.0, 40.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_value_to_string(formatters::v2s_compression_ratio(2))
            .with_string_to_value(formatters::s2v_compression_ratio()),

            attack: FloatParam::new(
                "Attack",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 200.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_value_to_string(formatters::v2s_f32_rounded(0))
            .with_unit(" ms"),

            release: FloatParam::new(
                "Release",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 200.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_value_to_string(formatters::v2s_f32_rounded(0))
            .with_unit(" ms"),

            steepness: FloatParam::new(
                "Steepness",
                30.0,
                FloatRange::Linear { min: 5.0, max: 100.0 },
            )
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            #[cfg(feature = "detailed_debugging")]
            logger_length: FloatParam::new(
                "LoggerLength",
                5000.0,
                FloatRange::Linear { min: 0.0, max: u64::MAX as f32 }
            ),
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        match self.process_buffer(buffer, _aux, _context) {
            Ok(_) => return ProcessStatus::Normal,
            Err(err) => {
                // Also ugly
                self.logger.write_debug_values().expect("Error writing CSV file");
                // Ugly, but easiest way to stop plugin right now...
                panic!("Processing aborted with: {}", err);
                // match self.logger.write_debug_values() {
                //     Ok(_) => return ProcessStatus::Error("Finished detailed debugging."),
                //     Err(_) => return ProcessStatus::Error(&err),
                // }
            }
        }
    }

    fn deactivate(&mut self) {}
}

impl Vst3Plugin for Compressor {
    const VST3_CLASS_ID: [u8; 16] = *b"Compressor      ";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(Compressor);
