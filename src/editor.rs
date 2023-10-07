use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::assets;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};

mod knob;
mod sineview;

use std::f32::consts::PI;
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

        let knob_config = ParamKnobConfiguration { label_align: LabelAlignment::Right, listeners: Vec::new() };

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.attack,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left, ..knob_config.clone() });
                ParamKnob::new(cx, Data::params, |p| &p.release,
                    ParamKnobConfiguration { label_align: LabelAlignment::Left, ..knob_config.clone() }); // TODO: don't like the clones here (not too performance impactful but there must be a nicer way)
            }).height(Pixels(200.0));

            let mut sine_view_entity = None;
            let mut rel_atk_entity = None;

            VStack::new(cx, |cx| {
                let sine_view = SineView::new(
                    cx,
                    Arc::clone(&params),
                    Box::new(|width| {
                        (0..width).map(|i| (i as f32 / (width as f32 / (2.0 * PI * 1.0))).sin()).collect()
                    }),
                ).entity;
        
                let rel_atk_view = SineView::new(
                    cx,
                    Arc::clone(&params),
                    Box::new(|width| {
                        let mut samples = Vec::new();
                        for i in 0..width {
                            samples.push((i as f32 / (width as f32 / (2.0 * PI * 16.0))).sin());
                        }
                        samples
                    }),
                ).entity;

                sine_view_entity = Some(sine_view);
                rel_atk_entity = Some(rel_atk_view)
            });

            let scope_listeners = vec![
                sine_view_entity.expect("Expect sine view to be constructed."),
                rel_atk_entity.expect("Expect release/attack view to be constructed.")
            ];

            VStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.threshold,
                    ParamKnobConfiguration { listeners: scope_listeners.clone(), ..knob_config });
                ParamKnob::new(cx, Data::params, |p| &p.ratio,
                    ParamKnobConfiguration { listeners: scope_listeners.clone(), ..knob_config });
                ParamKnob::new(cx, Data::params, |p| &p.steepness,
                    ParamKnobConfiguration { listeners: scope_listeners.clone(), ..knob_config });
            }).height(Pixels(300.0));
        })
        .class("main");
    })
}