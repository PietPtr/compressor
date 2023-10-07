// Implements an oscilloscope like window showing what current parameters would do to a sine wave
use std::sync::Arc;

use nih_plug_vizia::vizia::{
    prelude::*,
    vg::{Color, Paint, Path},
};

use crate::{
    compressor::{self, RawParameters},
    CompressorParams,
};

pub struct SineView {
    params: Arc<CompressorParams>,
    algo: compressor::Algo,
    width: u32,
    samples: Vec<f32>,
    base_waveform: Box<dyn Fn(u32) -> Vec<f32>>,
}

#[derive(Debug)]
pub enum ParamUpdateEvent {
    ParamUpdate,
}

impl SineView {
    pub fn new(
        cx: &mut Context,
        parameters: Arc<CompressorParams>,
        base_waveform: Box<dyn Fn(u32) -> Vec<f32>>,
    ) -> Handle<Self> {
        let mut view = Self {
            params: parameters,
            algo: compressor::Algo::new(),
            width: 300, // TODO: bad
            samples: vec![0.0; 300],
            base_waveform,
        };

        view.recalculate();
        view.recalculate(); // TODO: ugliest hack in the west

        view.build(cx, |_| {})
    }

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
                    },
                )
                .expect("gaat goed toooch");
        });
    }
}

impl View for SineView {
    fn element(&self) -> Option<&'static str> {
        Some("sineview")
    }

    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|param_event, _| match param_event {
            ParamUpdateEvent::ParamUpdate => self.recalculate(),
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let background_color = Color::rgb(0, 0, 0);
        let sine_color = cx.background_color().cloned().unwrap_or_default().into();

        let bounds = cx.bounds();

        canvas.clear_rect(
            bounds.x as u32,
            bounds.y as u32,
            bounds.w as u32,
            bounds.h as u32,
            background_color,
        );

        // Generate grid
        // TODO: check if the path can be transformed in one go by a matrix so we don't have to add bounds to it all the time
        const AMT_GRID_DIVS: u32 = 10;
        let grid_paint = Paint::color(Color::rgb(50, 50, 40));
        let mut grid_path = Path::new();

        for x in 0..AMT_GRID_DIVS + 1 {
            let x_pos = bounds.x + (x as f32 / AMT_GRID_DIVS as f32) * bounds.w;
            grid_path.move_to(x_pos, bounds.y);
            grid_path.line_to(x_pos, bounds.y + bounds.h);
        }
        for y in 0..AMT_GRID_DIVS + 1 {
            let y_pos = bounds.y + (y as f32 / AMT_GRID_DIVS as f32) * bounds.h;
            grid_path.move_to(bounds.x, y_pos);
            grid_path.line_to(bounds.x + bounds.w, y_pos);
        }

        canvas.stroke_path(&mut grid_path, &grid_paint);

        // Show thresholds
        let mut threshold_path = Path::new();
        let threshold_paint = Paint::color(Color::rgb(163, 144, 95));

        let threshold_y = self.params.threshold.value() * bounds.h / 2.0;
        let base_y = bounds.y + bounds.h / 2.0;
        threshold_path.move_to(bounds.x, base_y + threshold_y);
        threshold_path.line_to(bounds.x + bounds.w, base_y + threshold_y);

        threshold_path.move_to(bounds.x, base_y - threshold_y);
        threshold_path.line_to(bounds.x + bounds.w, base_y - threshold_y);

        canvas.stroke_path(&mut threshold_path, &threshold_paint);

        // Render sound waveform
        let mut path = Path::new();
        path.move_to(bounds.x, bounds.y + bounds.h / 2.0);
        for (x, y) in self.samples.iter().enumerate() {
            let x = bounds.x + x as f32;
            let y = bounds.y + *y * bounds.h / 2.0 + bounds.h / 2.0;
            path.line_to(x, y);
        }

        let mut paint = Paint::color(sine_color);
        paint.set_line_width(2.0);
        canvas.stroke_path(&mut path, &paint);
    }
}


pub struct TimeConstantsView {
    params: Arc<CompressorParams>,
    algo: compressor::Algo,
    width: u32,
    samples: Vec<f32>,
    base_waveform: Box<dyn Fn(u32) -> Vec<f32>>,
}

impl TimeConstantsView {
    pub fn new(
        cx: &mut Context,
        parameters: Arc<CompressorParams>,
        base_waveform: Box<dyn Fn(u32) -> Vec<f32>>,
    ) -> Handle<Self> {
        let mut view = Self {
            params: parameters,
            algo: compressor::Algo::new(),
            width: 300,
            samples: vec![0.0; 300],
            base_waveform,
        };

        view.recalculate();

        view.build(cx, |_| {})
    }

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
                        attack: self.params.attack.value(),
                        release: self.params.release.value(),
                    },
                )
                .expect("gaat goed toooch");
        });
    }
}

// TODO: reconcile with SineView
impl View for TimeConstantsView {
    fn element(&self) -> Option<&'static str> {
        Some("timeconstants")
    }

    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|param_event, _| match param_event {
            ParamUpdateEvent::ParamUpdate => self.recalculate(),
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let background_color = Color::rgb(0, 0, 0);
        let sine_color = cx.background_color().cloned().unwrap_or_default().into();

        let bounds = cx.bounds();

        canvas.clear_rect(
            bounds.x as u32,
            bounds.y as u32,
            bounds.w as u32,
            bounds.h as u32,
            background_color,
        );

        // Generate grid
        // TODO: check if the path can be transformed in one go by a matrix so we don't have to add bounds to it all the time
        const AMT_GRID_DIVS: u32 = 10;
        let grid_paint = Paint::color(Color::rgb(50, 50, 40));
        let mut grid_path = Path::new();

        for x in 0..AMT_GRID_DIVS + 1 {
            let x_pos = bounds.x + (x as f32 / AMT_GRID_DIVS as f32) * bounds.w;
            grid_path.move_to(x_pos, bounds.y);
            grid_path.line_to(x_pos, bounds.y + bounds.h);
        }
        for y in 0..AMT_GRID_DIVS + 1 {
            let y_pos = bounds.y + (y as f32 / AMT_GRID_DIVS as f32) * bounds.h;
            grid_path.move_to(bounds.x, y_pos);
            grid_path.line_to(bounds.x + bounds.w, y_pos);
        }

        canvas.stroke_path(&mut grid_path, &grid_paint);

        // Show thresholds
        let mut threshold_path = Path::new();
        let threshold_paint = Paint::color(Color::rgb(163, 144, 95));

        let threshold_y = self.params.threshold.value() * bounds.h / 2.0;
        let base_y = bounds.y + bounds.h / 2.0;
        threshold_path.move_to(bounds.x, base_y + threshold_y);
        threshold_path.line_to(bounds.x + bounds.w, base_y + threshold_y);

        threshold_path.move_to(bounds.x, base_y - threshold_y);
        threshold_path.line_to(bounds.x + bounds.w, base_y - threshold_y);

        canvas.stroke_path(&mut threshold_path, &threshold_paint);

        // Render sound waveform
        let mut path = Path::new();
        path.move_to(bounds.x, bounds.y + bounds.h / 2.0);
        for (x, y) in self.samples.iter().enumerate() {
            let x = bounds.x + x as f32;
            let y = bounds.y + *y * bounds.h / 2.0 + bounds.h / 2.0;
            path.line_to(x, y);
        }

        let mut paint = Paint::color(sine_color);
        paint.set_line_width(2.0);
        canvas.stroke_path(&mut path, &paint);
    }
}
