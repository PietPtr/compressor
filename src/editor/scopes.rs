use std::sync::Arc;

use nih_plug_vizia::vizia::vg::Color;
use vizia_scope::{ScopeData, ScopeLine, ConstantLine, SignalLine, AudioLine};

use crate::{
    compressor::{self, RawParameters},
    CompressorParams,
};

const SIGNAL_COLOR: Color = Color::rgbf(243.0 / 255.0, 250.0 / 255.0, 146.0 / 255.0);
const THRESHOLD_COLOR: Color = Color::rgbf(163.0 / 255.0, 144.0 / 255.0, 95.0 / 255.0);
const ENEVELOPE_COLOR: Color = Color::rgbf(255.0 / 255.0, 137.0 / 255.0, 137.0 / 255.0);

pub struct SineScope {
    params: Arc<CompressorParams>,
    algo: compressor::Algo,
    width: usize,
    samples: Vec<f32>,
    base_waveform: Box<dyn Fn(usize) -> Vec<f32>>,
}

impl SineScope {
    pub fn new(
        parameters: Arc<CompressorParams>,
        base_waveform: Box<dyn Fn(usize) -> Vec<f32>>,
        width: usize,
    ) -> Self {
        let mut scope = Self {
            params: parameters,
            algo: compressor::Algo::new(),
            width,
            samples: vec![0.0; width],
            base_waveform,
        };

        // Ensure the algorithm runs at least once to fully activate the attack during
        // the initial recalculation. After that, avoid resetting it to maintain this state.
        scope.recalculate();

        scope
    }
}

impl ScopeData for SineScope {
    fn recalculate(&mut self) {
        self.samples = (self.base_waveform)(self.width);

        self.samples.iter_mut().for_each(|sample| {
            self.algo
                .process_samples(
                    sample,
                    RawParameters {
                        threshold: self.params.threshold.value(),
                        ratio: self.params.ratio.value(),
                        steepness: self.params.steepness.value(),
                        attack: 0.0,
                        release: 10000.0,
                        gain: self.params.gain.value(),
                    },
                    None,
                )
                .expect("expect no debugging features to be enabled, so no errors either.");
        });
    }

    fn scope_lines(&self) -> Vec<ScopeLine> {
        vec![
            ScopeLine::Constant(ConstantLine::new(
                THRESHOLD_COLOR,
                self.params.threshold.value(),
            )),
            ScopeLine::Constant(ConstantLine::new(
                THRESHOLD_COLOR,
                -self.params.threshold.value(),
            )),
            ScopeLine::Signal(SignalLine::new(
                &self.samples,
                SIGNAL_COLOR,
                2.0,
            )),
        ]
    }
}

pub struct TimeConstantsScope {
    params: Arc<CompressorParams>,
    algo: compressor::Algo,
    amount_of_samples: usize,
    samples: Vec<f32>,
    envelope: Vec<f32>,
    base_waveform: Box<dyn Fn(usize) -> Vec<f32>>,
}

impl TimeConstantsScope {
    pub fn new(
        parameters: Arc<CompressorParams>,
        base_waveform: Box<dyn Fn(usize) -> Vec<f32>>,
        amount_of_samples: usize,
    ) -> Self {
        Self {
            params: parameters,
            algo: compressor::Algo::new(),
            amount_of_samples,
            samples: Vec::new(),
            envelope: Vec::new(),
            base_waveform,
        }
    }
}

impl ScopeData for TimeConstantsScope {
    fn recalculate(&mut self) {
        self.samples = (self.base_waveform)(self.amount_of_samples);
        self.envelope = Vec::with_capacity(self.amount_of_samples as usize);

        self.algo.reset();

        self.samples.iter_mut().for_each(|sample| {
            self.algo
                .process_samples(
                    sample,
                    RawParameters {
                        threshold: self.params.threshold.value(),
                        ratio: self.params.ratio.value(),
                        steepness: self.params.steepness.value(),
                        attack: self.params.attack.value() / 1000.0,
                        release: self.params.release.value() / 1000.0,
                        gain: self.params.gain.value(),
                    },
                    None
                )
                .expect("expect no debugging features to be enabled, so no errors either.");

            self.envelope.push(-self.algo.get_envelope());
        });
    }

    fn scope_lines(&self) -> Vec<ScopeLine> {
        vec![
            ScopeLine::Constant(ConstantLine::new(
                THRESHOLD_COLOR,
                self.params.threshold.value(),
            )),
            ScopeLine::Constant(ConstantLine::new(
                THRESHOLD_COLOR,
                -self.params.threshold.value(),
            )),
            ScopeLine::Audio(AudioLine::new(
                &self.samples,
                SIGNAL_COLOR,
            )),
            ScopeLine::Signal(SignalLine::new(
                &self.envelope,
                ENEVELOPE_COLOR,
                1.5,
            )),
        ]
    }
}
