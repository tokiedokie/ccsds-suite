mod alias_set;
mod ancillary_data_set;
mod command_metadata;
mod data_encoding;
mod element_forms;
mod header;
mod parameter;
mod parameter_set;
mod parameter_type;
mod service_set;
mod space_system;
mod space_system_description;
mod space_system_identity;
mod telemetry_metadata;

use gpui::{App, Div, Entity, ParentElement, Styled, div};
use gpui_component::{
    ActiveTheme, StyledExt,
    input::{Input, InputState},
    v_flex,
};

pub(crate) use element_forms::ElementForms;

macro_rules! impl_select_item {
    ($type:ty) => {
        impl gpui_component::select::SelectItem for $type {
            type Value = Self;

            fn title(&self) -> gpui::SharedString {
                self.to_string().into()
            }

            fn value(&self) -> &Self::Value {
                self
            }
        }
    };
}

pub(super) use impl_select_item;

pub(super) fn field(
    label: &'static str,
    hint: &'static str,
    input: &Entity<InputState>,
    cx: &App,
) -> Div {
    v_flex()
        .gap_2()
        .w_full()
        .child(
            gpui_component::h_flex()
                .justify_between()
                .child(div().text_sm().font_medium().child(label))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(hint),
                ),
        )
        .child(Input::new(input))
}

pub(super) fn optional_value(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

pub(super) fn property_rows(rows: Vec<(&'static str, String)>, cx: &App) -> Div {
    let mut content = v_flex().gap_3();
    for (label, value) in rows {
        content = content.child(
            gpui_component::h_flex()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(div().text_sm().font_medium().child(value)),
        );
    }
    content
}
