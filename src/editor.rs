use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{assets, widgets::*};
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};

mod knob;
mod sineview;

use std::sync::Arc;

use crate::CompressorParams;

use self::knob::{ParamKnob, ParamKnobConfiguration, LabelAlignment};
use self::sineview::SineView;

#[cfg(not(feature = "external_stylesheet"))]
const STYLE: &str = include_str!("editor/stylesheet.css");

#[derive(Lens)]
struct Data {
    params: Arc<CompressorParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (720, 300))
}

pub(crate) fn create(
    params: Arc<CompressorParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Builtin, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        #[cfg(feature = "external_stylesheet")]
        cx.add_stylesheet("src/editor/stylesheet.css")
            .expect("Expect stylesheet to exist in debug mode");

        #[cfg(not(feature = "external_stylesheet"))]
        cx.add_theme(STYLE);

        Data {
            params: params.clone(),
        }.build(cx);

        ResizeHandle::new(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.threshold,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left });
                ParamKnob::new(cx, Data::params, |p| &p.ratio,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left });
                ParamKnob::new(cx, Data::params, |p| &p.steepness,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left });
            }).height(Pixels(300.0));
            SineView::new(
                cx,
                Arc::clone(&params),
            );
            VStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.attack,
                    ParamKnobConfiguration { label_align: LabelAlignment::Right });
                ParamKnob::new(cx, Data::params, |p| &p.release,
                    ParamKnobConfiguration { label_align: LabelAlignment::Right });
            }).height(Pixels(200.0));
        })
        .top(Pixels(0.))
        .class("main");
    })
}