mod aggregate_member_list;
mod alias_set;
mod ancillary_data_set;
mod argument_type;
mod array_dimension;
mod boolean_expression;
mod command_metadata;
mod container_binary_encoding;
mod container_rate;
mod context_calibrator;
mod context_significance;
pub(super) mod custom_algorithm;
pub(super) mod custom_stream;
mod data_encoding;
mod default_calibrator;
mod discrete_lookup;
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
mod parameter_alarm;
mod parameter_type;
mod reference_time;
mod rpn_operation;
mod sequence_container;
pub(super) mod service_set;
mod space_system;
mod space_system_description;
mod space_system_identity;
mod telemetry_metadata;
mod time_encoding;
mod to_string;
mod unit_set;
mod valid_range;
pub(super) mod variable_frame_stream;
mod variable_string;

use std::rc::Rc;

use gpui::{
    App, Div, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, div, prelude::FluentBuilder, uniform_list,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    form::{field as form_field, v_form},
    input::{Input, InputState},
    select::{Select, SelectItem, SelectState},
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
    let (required, description) = field_requirement(hint);
    v_flex().w_full().child(
        v_form().child(
            form_field()
                .label(label)
                .required(required)
                .when(!description.is_empty(), |field| {
                    field.description(description)
                })
                .child(Input::new(input)),
        ),
    )
}

pub(super) fn select_field<T>(
    label: &'static str,
    hint: &'static str,
    select: &Entity<SelectState<Vec<T>>>,
) -> Div
where
    T: Clone + PartialEq + SelectItem + 'static,
{
    let (required, description) = field_requirement(hint);
    v_flex().w_full().child(
        v_form().child(
            form_field()
                .label(label)
                .required(required)
                .when(!description.is_empty(), |field| {
                    field.description(description)
                })
                .child(Select::new(select).w_full()),
        ),
    )
}

pub(super) fn section_remove_button(id: impl Into<ElementId>) -> Button {
    Button::new(id)
        .small()
        .icon(IconName::Minus)
        .label("Remove")
}

pub(super) fn section_add_button(id: impl Into<ElementId>) -> Button {
    Button::new(id).small().icon(IconName::Plus).label("Add")
}

pub(super) fn count_label(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

pub(super) fn empty_list_state(message: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .w_full()
        .p_4()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(message.into())
}

pub(super) fn row_remove_button(
    id: impl Into<ElementId>,
    tooltip: impl Into<SharedString>,
) -> Button {
    Button::new(id)
        .small()
        .ghost()
        .icon(IconName::Minus)
        .tooltip(tooltip)
}

pub(super) fn detail_list_card(cx: &App) -> Div {
    v_flex()
        .w_full()
        .p_3()
        .gap_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
}

pub(super) fn compact_list_row(cx: &App) -> Div {
    gpui_component::h_flex()
        .w_full()
        .p_2()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
}

fn field_requirement(hint: &'static str) -> (bool, &'static str) {
    let Some(suffix) = hint.strip_prefix("Required") else {
        return (false, hint);
    };
    if !suffix.is_empty()
        && !suffix.starts_with(|character: char| {
            character == ',' || character == ';' || character.is_whitespace()
        })
    {
        return (false, hint);
    }
    (
        true,
        suffix.trim_start_matches(|character: char| {
            character == ',' || character == ';' || character.is_whitespace()
        }),
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
                .child(count_label(row_count, "item", "items")),
        )
        .child(if row_count == 0 {
            empty_list_state(empty_message, cx).into_any_element()
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

#[cfg(test)]
mod tests {
    use super::field_requirement;

    #[test]
    fn recognizes_required_hints_and_preserves_their_details() {
        assert_eq!(field_requirement("Required"), (true, ""));
        assert_eq!(
            field_requirement("Required, for example catalog"),
            (true, "for example catalog")
        );
        assert_eq!(
            field_requirement("Required; floating-point delta"),
            (true, "floating-point delta")
        );
        assert_eq!(
            field_requirement("Required if significance specified"),
            (true, "if significance specified")
        );
        assert_eq!(field_requirement("Optional"), (false, "Optional"));
    }
}
