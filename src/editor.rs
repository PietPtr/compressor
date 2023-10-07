use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::assets;
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

        let knob_config = ParamKnobConfiguration { label_align: LabelAlignment::Right, listener: None };

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.attack,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left, ..knob_config });
                ParamKnob::new(cx, Data::params, |p| &p.release,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left, ..knob_config });
            }).height(Pixels(200.0));

            let sineview = SineView::new(
                cx,
                Arc::clone(&params),
            ).entity;

            VStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.threshold,
                    ParamKnobConfiguration { listener: Some(sineview), ..knob_config });
                ParamKnob::new(cx, Data::params, |p| &p.ratio,
                    ParamKnobConfiguration { listener: Some(sineview), ..knob_config });
                ParamKnob::new(cx, Data::params, |p| &p.steepness,
                    ParamKnobConfiguration { listener: Some(sineview), ..knob_config });
            }).height(Pixels(300.0));
        })
        .class("main");
    })
}