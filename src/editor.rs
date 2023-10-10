use nih_plug::prelude::Editor;
use nih_plug_vizia::assets;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};

mod knob;
mod scope;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use std::sync::Arc;

use crate::editor::scope::{ScopeView, SineScope, TimeConstantsScope};
use crate::CompressorParams;

use self::knob::{LabelAlignment, ParamKnob};

const STYLE: &str = include_str!("editor/stylesheet.css");

#[derive(Lens)]
struct Data {
    params: Arc<CompressorParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new_with_default_scale_factor(|| (750, 330), 1.0)
}

pub(crate) fn create(
    params: Arc<CompressorParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Builtin, move |cx, _| {
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

        let scope_listeners = Rc::new(RefCell::new(Vec::new()));

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.threshold,
                    LabelAlignment::Left,
                    Rc::clone(&scope_listeners),
                    false,
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.ratio,
                    LabelAlignment::Left,
                    Rc::clone(&scope_listeners),
                    false,
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.steepness,
                    LabelAlignment::Left,
                    Rc::clone(&scope_listeners),
                    false,
                );
            });

            VStack::new(cx, |cx| {
                let sine_view = ScopeView::new(
                    cx,
                    SineScope::new(
                        Arc::clone(&params),
                        Box::new(|width| {
                            (0..width)
                                .map(|i| (i as f32 / (width as f32 / (2.0 * PI * 1.0))).sin())
                                .collect()
                        }),
                        300,
                    ),
                    None,
                )
                .entity;

                let rel_atk_view = ScopeView::new(
                    cx,
                    TimeConstantsScope::new(
                        Arc::clone(&params),
                        Box::new(|width| {
                            let mut samples = Vec::with_capacity(width as usize);
                            for _ in 0..width / 8 {
                                samples.push(0.0);
                            }
                            for i in 0..width / 4 {
                                samples.push((i as f32 / (width as f32 / (2.0 * PI * 1024.0))).sin());
                            }
                            for _ in 0..width / 4 {
                                samples.push(0.0);
                            }
                            for i in 0..(width / 8) * 3 {
                                samples.push((i as f32 / (width as f32 / (2.0 * PI * 1024.0))).sin());
                            }
                            samples
                        }),
                        15000,
                    ),
                    None,
                )
                .entity;

                {
                    let mut listeners_ref = scope_listeners.borrow_mut();
                    listeners_ref.push(sine_view);
                    listeners_ref.push(rel_atk_view);
                }
            })
            .class("scopes");

            VStack::new(cx, |cx| {
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.attack,
                    LabelAlignment::Right,
                    Rc::clone(&scope_listeners),
                    false
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.release,
                    LabelAlignment::Right,
                    Rc::clone(&scope_listeners),
                    false,
                );
                ParamKnob::new(
                    cx,
                    Data::params,
                    |p| &p.gain,
                    LabelAlignment::Right,
                    Rc::clone(&scope_listeners),
                    true,
                );
            });
        })
        .class("main");
    })
}
