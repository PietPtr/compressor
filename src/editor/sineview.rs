// Implements an oscilloscope like window showing what current parameters would do to a sine wave
use std::{f32::consts::PI, sync::Arc};

use nih_plug_vizia::vizia::{prelude::*, vg::{Path, Paint, Color}};

use crate::{
    compressor::{self, RawParameters},
    CompressorParams,
};

pub struct SineView {
    params: Arc<CompressorParams>,
    algo: compressor::Algo,
    width: u32,
    samples: Vec<f32>,
}

#[derive(Debug)]
pub enum ParamUpdateEvent {
    ParamUpdate,
}

impl SineView {
    pub fn new(cx: &mut Context, parameters: Arc<CompressorParams>) -> Handle<Self> {
        Self {
            params: parameters,
            algo: compressor::Algo::new(),
            width: 300,
            samples: vec![0.0; 300],
        }
        .build(cx, |cx| {})
    }

    fn recalculate(&mut self) {
        self.samples = Vec::new();

        for i in 0..self.width {
            // TODO: incorporate width as a setting?
            self.samples.push((i as f32 / (self.width as f32 / (2.0 * PI * 4.0))).sin());
        }

        self.samples.iter_mut().for_each(|sample| {
            self.algo.process_samples(
                sample,
                RawParameters {
                    threshold: self.params.threshold.value(),
                    ratio: self.params.ratio.value(),
                    steepness: self.params.steepness.value(),
                    attack: 0.0,
                    release: 10000.0,
                },
            ).expect("gaat goed toooch");
        });
    }
}

impl View for SineView {
    fn element(&self) -> Option<&'static str> {
        Some("sineview")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|param_event, _| match param_event {
            ParamUpdateEvent::ParamUpdate => self.recalculate(),
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let background_color = Color::rgb(0, 0, 0);
        let sine_color = Color::rgb(0, 255, 0);

        let bounds = cx.bounds();

        let mut path = Path::new();
        path.move_to(bounds.x, bounds.h / 2.0);
        for (x, y) in self.samples.iter().enumerate() {
            let x = bounds.x + x as f32;
            let y = bounds.y + *y * bounds.h / 2.0 + bounds.h / 2.0;
            path.line_to(x, y);
        }

        let mut paint = Paint::color(sine_color);
        paint.set_line_width(1.0);
        canvas.stroke_path(&mut path, &paint);


    }
}
