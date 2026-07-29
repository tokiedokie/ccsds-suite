use gpui::{App, AppContext, Context, Div, Entity, ParentElement, Styled, Subscription, Window};
use gpui_component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use super::field;
use crate::XtceEditor;

pub(super) struct SpaceSystemIdentityFields {
    name_input: Entity<InputState>,
    asset_type_input: Entity<InputState>,
    status_input: Entity<InputState>,
    base_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl SpaceSystemIdentityFields {
    pub(super) fn new(
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        let name_input =
            cx.new(|cx| InputState::new(window, cx).default_value(system.name.clone()));
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        Self {
            name_input,
            asset_type_input: cx
                .new(|cx| InputState::new(window, cx).default_value(system.asset_type.clone())),
            status_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(system.operational_status.clone().unwrap_or_default())
            }),
            base_input: cx.new(|cx| {
                InputState::new(window, cx).default_value(system.base.clone().unwrap_or_default())
            }),
            _subscriptions: vec![name_subscription],
        }
    }

    pub(super) fn load(
        &self,
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        self.name_input
            .update(cx, |input, cx| input.set_value(&system.name, window, cx));
        self.asset_type_input.update(cx, |input, cx| {
            input.set_value(&system.asset_type, window, cx);
        });
        self.status_input.update(cx, |input, cx| {
            input.set_value(
                system.operational_status.clone().unwrap_or_default(),
                window,
                cx,
            );
        });
        self.base_input.update(cx, |input, cx| {
            input.set_value(system.base.clone().unwrap_or_default(), window, cx);
        });
    }

    pub(super) fn apply_to(&self, system: &mut xtce::SpaceSystem, cx: &App) {
        system.name = self.name_input.read(cx).value().to_string();
        system.asset_type = self.asset_type_input.read(cx).value().to_string();
        system.operational_status = optional_value(self.status_input.read(cx).value().to_string());
        system.base = optional_value(self.base_input.read(cx).value().to_string());
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(gpui::px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn name(&self, cx: &App) -> String {
        self.name_input.read(cx).value().to_string()
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        v_flex()
            .gap_5()
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(field(
                        "Asset type",
                        "Optional; defaults to unknown",
                        &self.asset_type_input,
                        cx,
                    ))
                    .child(field(
                        "Operational status",
                        "Optional",
                        &self.status_input,
                        cx,
                    )),
            )
            .child(field("XML base", "Optional", &self.base_input, cx))
    }
}

fn optional_value(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}
