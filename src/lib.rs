//! VST3 plugin implementing an audio compressor with six parameters:
//! * Threshold: Level above which to start compressing.
//! * Ratio: Amount of compression
//! * Steepness: measure of quickly the compression engages. A low steepness means that as the actual audio
//!     level approaches the threshold the compressor will gradually already engage. A high steepness will be
//!     more like the compressor engaging not at all under the threshold and immediately after it goes over
//!     the threshold. It is a measure of how smooth the knee is.
//! * Attack: Time in ms until the compressor fully engages.
//! * Release: Time in ms until the compressor is fully disengaged.
//! * Gain: gain to apply after compression.

extern crate csv;

use compressor::Algo;
#[cfg(feature = "detailed_debugging")]
use llad::SampleLogger;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod compressor;
mod editor;

/// Parameters for the compressor.
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

/// Struct implementing [`nih_plug::prelude::Plugin`].
pub struct Compressor {
    params: Arc<CompressorParams>,
    algos: Vec<Algo>,
    #[cfg(feature = "detailed_debugging")]
    logger: SampleLogger,
}

impl Compressor {
    // TODO: move back into process?
    fn process_buffer(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> Result<(), &'static str> {
        #[cfg(feature = "detailed_debugging")]
        self.logger
            .set_quit_after_n_samples(self.params.logger_length.value() as u64);

        for channel_samples in buffer.iter_samples() {
            let threshold = self.params.threshold.smoothed.next();
            let ratio = self.params.ratio.smoothed.next();
            let attack = self.params.attack.smoothed.next() / 1000.0;
            let release = self.params.release.smoothed.next() / 1000.0;
            let steepness = self.params.steepness.smoothed.next();
            let gain = self.params.gain.smoothed.next();

            while self.algos.len() < channel_samples.len() {
                self.algos.push(Algo::new());
            }

            assert!(channel_samples.len() == self.algos.len());

            let mut algo_id = 0;
            for sample in channel_samples {
                self.algos
                    .get_mut(algo_id)
                    .expect(format!("Expect algo id {algo_id} to be present.").as_str())
                    .process_samples(
                        sample,
                        compressor::RawParameters {
                            threshold,
                            ratio,
                            steepness,
                            attack,
                            release,
                            gain,
                        },
                        #[cfg(feature = "detailed_debugging")]
                        if algo_id == 0 {
                            Some(&mut self.logger)
                        } else {
                            None
                        },
                        #[cfg(not(feature = "detailed_debugging"))]
                        None
                    )?;
                algo_id += 1;
            }
        }

        Ok(())
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            params: Arc::new(CompressorParams::default()),
            algos: Vec::new(),
            #[cfg(feature = "detailed_debugging")]
            logger: SampleLogger::new(String::from("debug.csv")),
        }
    }
}

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
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: u64::MAX as f32,
                },
            ),
        }
    }
}

impl Plugin for Compressor {
    #[cfg(feature = "detailed_debugging")]
    const NAME: &'static str = "Compressor (debug)";
    #[cfg(not(feature = "detailed_debugging"))]
    const NAME: &'static str = "Compressor";
    const VENDOR: &'static str = "Staal";
    const URL: &'static str = "example.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

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
            Err(err) => return ProcessStatus::Error(&err),
        }
    }

    fn deactivate(&mut self) {
        #[cfg(feature = "detailed_debugging")]
        self.logger
            .write_debug_values()
            .expect("Expect CSV writing to be succesful.");
    }
}

impl Vst3Plugin for Compressor {
    const VST3_CLASS_ID: [u8; 16] = *b"Compressor      ";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(Compressor);
