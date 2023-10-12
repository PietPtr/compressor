use nih_plug::prelude::Editor;
use nih_plug_vizia::assets;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};

mod knob;

use std::sync::Arc;

use crate::CompressorParams;

use self::knob::{LabelAlignment, ParamKnob};

const STYLE: &str = include_str!("editor/stylesheet.css");

#[derive(Lens)]
struct Data {
    params: Arc<CompressorParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new_with_default_scale_factor(|| (750, 20), 1.0)
}

pub(crate) fn create(
    params: Arc<CompressorParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(Arc::clone(&editor_state), ViziaTheming::Builtin, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        match cx.add_stylesheet("src/editor/stylesheet.css") {
            Ok(_) => println!("Loaded stylesheet."),
            Err(_) => cx.add_theme(STYLE),
        }

        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.threshold,
                    LabelAlignment::Right,
                    false,
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.ratio,
                    LabelAlignment::Right,
                    false,
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.steepness,
                    LabelAlignment::Right,
                    false,
                );
            });

            HStack::new(cx, |cx| {
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.attack,
                    LabelAlignment::Right,
                    false
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.release,
                    LabelAlignment::Right,
                    false,
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.gain,
                    LabelAlignment::Right,
                    true,
                );
            });
        })
        .class("main");
    })
}
