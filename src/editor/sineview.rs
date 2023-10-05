// Implements an oscilloscope like window showing what current parameters would do to a sine wave

use nih_plug_vizia::vizia::prelude::*;

pub struct SineView {

}

impl SineView {
    pub fn new(
        cx: &mut Context
    ) -> Handle<Self> {
        Self {

        }
        .build(cx, |cx| {
            Label::new(cx, "custom view");
        })
    }
}

impl View for SineView {
    fn element(&self) -> Option<&'static str> {
        Some("sineview")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        
    }

}

impl Handle<'_, TickKnob> {
    pub fn value<L: Lens<Target = f32>>(self, lens: L) -> Self {
        let entity = self.entity;
        Binding::new(self.cx, lens, move |cx, value| {
            let value = value.get(cx);
            if let Some(view) = cx.views.get_mut(&entity) {
                if let Some(knob) = view.downcast_mut::<TickKnob>() {
                    knob.normalized_value = value;
                    cx.style.needs_redraw = true;
                }
            }
        });

        self
    }
}