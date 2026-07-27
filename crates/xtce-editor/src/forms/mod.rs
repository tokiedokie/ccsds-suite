mod aggregate_member_list;
mod alias_set;
mod ancillary_data_set;
mod argument_type;
mod boolean_expression;
mod command_metadata;
mod container_binary_encoding;
mod container_rate;
pub(super) mod custom_algorithm;
pub(super) mod custom_stream;
mod data_encoding;
mod default_calibrator;
mod dynamic_value;
mod element_forms;
mod enumeration_list;
mod error_detect_correct;
pub(super) mod fixed_frame_stream;
mod header;
mod input_algorithm;
pub(super) mod math_algorithm;
mod message;
pub(super) mod message_set;
mod meta_command;
mod parameter;
mod parameter_type;
mod rpn_operation;
mod sequence_container;
mod service_set;
mod space_system;
mod space_system_description;
mod space_system_identity;
mod telemetry_metadata;
pub(super) mod variable_frame_stream;

use std::rc::Rc;

use gpui::{
    App, Div, Entity, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, WeakEntity, div, prelude::FluentBuilder, uniform_list,
};
use gpui_component::{
    ActiveTheme, StyledExt,
    form::{field as form_field, v_form},
    input::{Input, InputState},
    v_flex,
};

use crate::{ElementKind, XtceEditor};

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
    _cx: &App,
) -> Div {
    let required = hint == "Required";
    v_flex().w_full().child(
        v_form().child(
            form_field()
                .label(label)
                .required(required)
                .when(!required && !hint.is_empty(), |field| {
                    field.description(hint)
                })
                .child(Input::new(input)),
        ),
    )
}

pub(super) fn optional_value(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

pub(super) fn collection_summary(
    headers: &[&'static str],
    rows: Vec<Vec<String>>,
    targets: Vec<Option<ElementKind>>,
    editor: WeakEntity<XtceEditor>,
    empty_message: &'static str,
    cx: &App,
) -> Div {
    let row_count = rows.len();
    let table = v_flex()
        .w_full()
        .min_w(gpui::px(headers.len() as f32 * 180.))
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .child(
            gpui_component::h_flex()
                .h(gpui::px(34.))
                .px_2()
                .gap_2()
                .items_center()
                .bg(cx.theme().muted.opacity(0.5))
                .text_xs()
                .font_medium()
                .children(
                    headers
                        .iter()
                        .map(|header| div().flex_1().min_w_0().truncate().child(*header)),
                ),
        )
        .when(row_count > 0, |table| {
            let rows = Rc::new(rows);
            let targets = Rc::new(targets);
            table.child(
                uniform_list(
                    "collection-summary-rows",
                    row_count,
                    move |visible_range, _, cx| {
                        visible_range
                            .map(|index| {
                                let target = targets.get(index).copied().flatten();
                                let editor = editor.clone();
                                let hover_color = cx.theme().sidebar_accent;
                                gpui_component::h_flex()
                                    .id(format!("collection-summary-row-{index}"))
                                    .w_full()
                                    .h(gpui::px(42.))
                                    .px_2()
                                    .gap_2()
                                    .items_center()
                                    .border_b_1()
                                    .border_color(cx.theme().border)
                                    .text_sm()
                                    .children(rows[index].iter().map(|value| {
                                        div().flex_1().min_w_0().truncate().child(value.clone())
                                    }))
                                    .when_some(target, move |row, target| {
                                        row.cursor_pointer()
                                            .hover(move |row| row.bg(hover_color))
                                            .on_click(move |_, window, cx| {
                                                _ = editor.update(cx, |this, cx| {
                                                    this.select_summary_element(target, window, cx);
                                                });
                                            })
                                    })
                            })
                            .collect::<Vec<_>>()
                    },
                )
                .w_full()
                .h(gpui::px(row_count.min(12) as f32 * 42.)),
            )
        });

    v_flex()
        .w_full()
        .gap_3()
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{row_count} items")),
        )
        .child(if row_count == 0 {
            div()
                .p_4()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(empty_message)
                .into_any_element()
        } else {
            div()
                .id("collection-summary-scroll-boundary")
                .w_full()
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .child(
                    div()
                        .id("collection-summary-horizontal-scroll")
                        .w_full()
                        .overflow_x_scroll()
                        .child(table),
                )
                .into_any_element()
        })
}
