use gpui::{App, AppContext, Context, Div, Entity, ParentElement, Styled, Window};
use gpui_component::{input::InputState, v_flex};

use super::field;
use crate::XtceEditor;

pub(super) struct SpaceSystemDescriptionFields {
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
}

impl SpaceSystemDescriptionFields {
    pub(super) fn new(
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        Self {
            short_description_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(system.short_description.clone().unwrap_or_default())
            }),
            long_description_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .auto_grow(super::MULTILINE_MIN_ROWS, super::MULTILINE_MAX_ROWS)
                    .default_value(system.long_description.clone().unwrap_or_default())
            }),
        }
    }

    pub(super) fn load(
        &self,
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        self.short_description_input.update(cx, |input, cx| {
            input.set_value(
                system.short_description.clone().unwrap_or_default(),
                window,
                cx,
            );
        });
        self.long_description_input.update(cx, |input, cx| {
            input.set_value(
                system.long_description.clone().unwrap_or_default(),
                window,
                cx,
            );
        });
    }

    pub(super) fn apply_to(&self, system: &mut xtce::SpaceSystem, cx: &App) {
        system.short_description =
            optional_value(self.short_description_input.read(cx).value().to_string());
        system.long_description =
            optional_value(self.long_description_input.read(cx).value().to_string());
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        v_flex()
            .gap_5()
            .child(field(
                "Short description",
                "Optional",
                &self.short_description_input,
                cx,
            ))
            .child(field(
                "Long description",
                "Optional",
                &self.long_description_input,
                cx,
            ))
    }
}

fn optional_value(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}
