// Implements an oscilloscope like window showing what current parameters would do to a sine wave

use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_vizia::{vizia::prelude::*, widgets::param_base::ParamWidgetBase};

use crate::CompressorParams;

pub struct SineView {
    params: Arc<CompressorParams>,
}


impl SineView {
    pub fn new(
        cx: &mut Context,
        parameters: Arc<CompressorParams>
    ) -> Handle<Self> {
        Self {
            params: parameters,
        }.build(cx, |cx| {

        })
    }
}

impl View for SineView {
    fn element(&self) -> Option<&'static str> {
        Some("sineview")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        dbg!(&self.params.threshold);
    }

}
