extern crate csv;

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod compressor;
pub mod csv_debugging;
mod editor;

pub struct Compressor {
    params: Arc<CompressorParams>,
    algo: compressor::Algo,
}

#[derive(Params, Debug)]
pub struct CompressorParams {
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
    #[id = "gain"]
    pub gain: FloatParam, // stored as gain, entered in dB

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
        self.algo
            .logger
            .set_quit_after_n_samples(self.params.logger_length.value());

        for channel_samples in buffer.iter_samples() {
            // TODO: can be seen from audio layout right?
            #[cfg(feature = "detailed_debugging")]
            {
                if channel_samples.len() > 1 {
                    panic!(
                        "Too many channels for detailed debugging to support: {:?}",
                        channel_samples.len()
                    );
                }
            }

            let threshold = self.params.threshold.smoothed.next();
            let ratio = self.params.ratio.smoothed.next();
            let attack = self.params.attack.smoothed.next() / 1000.0;
            let release = self.params.release.smoothed.next() / 1000.0;
            let steepness = self.params.steepness.smoothed.next();
            let gain = self.params.gain.smoothed.next();

            self.algo.process_samples(
                channel_samples
                    .into_iter()
                    .next()
                    .expect("Expect at least one channel"),
                compressor::RawParameters {
                    threshold,
                    ratio,
                    steepness,
                    attack,
                    release,
                    gain,
                },
            )?;
        }

        Ok(())
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            params: Arc::new(CompressorParams::default()),
            algo: compressor::Algo::new(),
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
                util::db_to_gain(-10.0),
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
                3.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 200.0,
                    factor: FloatRange::gain_skew_factor(1.0, 40.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_value_to_string(formatters::v2s_compression_ratio(1))
            .with_string_to_value(formatters::s2v_compression_ratio()),

            attack: FloatParam::new(
                "Attack",
                20.0,
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
                140.0,
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
                8.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 300.0,
                    factor: FloatRange::gain_skew_factor(1.0, 40.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_value_to_string(formatters::v2s_f32_rounded(0)),

            gain: FloatParam::new(
                "Gain",
                1.0,
                FloatRange::Skewed {
                    min: util::db_to_gain(-6.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-6.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            #[cfg(feature = "detailed_debugging")]
            logger_length: FloatParam::new(
                "LoggerLength",
                5000.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: u64::MAX as f32,
                },
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
                self.algo
                    .logger
                    .write_debug_values()
                    .expect("Error writing CSV file");
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
