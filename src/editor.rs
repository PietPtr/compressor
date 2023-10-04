use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{assets, widgets::*};
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
mod knob;

use std::sync::Arc;

use crate::CompressorParams;

use self::knob::ParamKnob;

// TODO: should be loaded from a file (using include_str!() macro?)
const STYLE: &str = r#""#;

#[derive(Lens)]
struct Data {
    params: Arc<CompressorParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (720, 440))
}

pub(crate) fn create(
    params: Arc<CompressorParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Builtin, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        cx.add_theme(STYLE);

        Data {
            params: params.clone(),
        }.build(cx);

        ResizeHandle::new(cx);

        HStack::new(cx, |cx| {
            ParamKnob::new(cx, Data::params, |p| &p.threshold);
            ParamKnob::new(cx, Data::params, |p| &p.ratio);
            ParamKnob::new(cx, Data::params, |p| &p.attack);
            ParamKnob::new(cx, Data::params, |p| &p.release);
            ParamKnob::new(cx, Data::params, |p| &p.steepness);
        })
        .child_space(Stretch(1.0));
    })
}