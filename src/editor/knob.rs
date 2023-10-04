use nih_plug::prelude::Param;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Debug)]
pub enum ParamEvent {
    BeginSetParam,
    SetParam(f32),
    EndSetParam,
}

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase
}

impl ParamKnob {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone + Copy,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params.clone(), params_to_param),
        }.build(
            cx, 
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                VStack::new(cx, |cx| {
                    Knob::new(cx, 0.5, 
                        params.map(move |params| {
                            params_to_param(params).unmodulated_normalized_value()
                        }), 
                        true);

                    
                });
            }))
    }
}

impl View for ParamKnob {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|param_change_event, _| {
            match param_change_event {
                ParamEvent::BeginSetParam => {
                    self.param_base.begin_set_parameter(cx);
                }
                ParamEvent::SetParam(val) => {
                    self.param_base.set_normalized_value(cx, *val);
                }
                ParamEvent::EndSetParam => {
                    self.param_base.end_set_parameter(cx);
                }
            }
        });
    }
}