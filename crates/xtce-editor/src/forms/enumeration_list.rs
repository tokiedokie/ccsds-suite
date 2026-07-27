use std::collections::{HashMap, VecDeque};

use gpui::{
    AnyElement, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ListAlignment,
    ListState, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div, list, px,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

const ROW_HEIGHT: f32 = 52.;
const EDITOR_CACHE_SIZE: usize = 24;

#[derive(Clone)]
struct EnumerationRowData {
    value: String,
    max_value: String,
    label: String,
    description: String,
}

impl EnumerationRowData {
    fn default_row() -> Self {
        Self {
            value: "0".to_owned(),
            max_value: String::new(),
            label: "VALUE".to_owned(),
            description: String::new(),
        }
    }
}

pub(super) struct EnumerationListForm {
    rows: Vec<EnumerationRowData>,
    editors: HashMap<usize, Entity<EnumerationRowForm>>,
    cache_order: VecDeque<usize>,
    list_state: ListState,
}

struct EnumerationRowForm {
    value_input: Entity<InputState>,
    max_value_input: Entity<InputState>,
    label_input: Entity<InputState>,
    description_input: Entity<InputState>,
}

impl EnumerationListForm {
    pub(super) fn new(
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        _: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = rows_from_parameter_type(parameter_type);
        cx.new(|_| Self::from_rows(rows))
    }

    fn from_rows(mut rows: Vec<EnumerationRowData>) -> Self {
        if rows.is_empty() {
            rows.push(EnumerationRowData::default_row());
        }
        Self {
            list_state: ListState::new(rows.len(), ListAlignment::Top, px(ROW_HEIGHT))
                .with_uniform_item_height(px(ROW_HEIGHT)),
            rows,
            editors: HashMap::new(),
            cache_order: VecDeque::new(),
        }
    }

    pub(super) fn load(
        &mut self,
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        cx: &mut Context<Self>,
    ) {
        let mut rows = rows_from_parameter_type(parameter_type);
        if rows.is_empty() {
            rows.push(EnumerationRowData::default_row());
        }
        self.rows = rows;
        self.editors.clear();
        self.cache_order.clear();
        self.list_state
            .reset_with_uniform_height(self.rows.len(), px(ROW_HEIGHT));
        cx.notify();
    }

    pub(super) fn reset_to_default(&mut self, cx: &mut Context<Self>) {
        self.rows = vec![EnumerationRowData::default_row()];
        self.editors.clear();
        self.cache_order.clear();
        self.list_state.reset_with_uniform_height(1, px(ROW_HEIGHT));
        cx.notify();
    }

    pub(super) fn apply_to(
        &self,
        parameter_type: &mut xtce::ParameterTypeSetTypeContent,
        cx: &App,
    ) {
        let xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(parameter_type) =
            parameter_type
        else {
            return;
        };
        let rows = self.current_rows(cx);
        let list = if let Some(list) =
            parameter_type
                .content
                .iter_mut()
                .find_map(|content| match content {
                    xtce::EnumeratedParameterTypeContent::EnumerationList(list) => Some(list),
                    _ => None,
                }) {
            list
        } else {
            parameter_type
                .content
                .push(xtce::EnumeratedParameterTypeContent::EnumerationList(
                    xtce::EnumerationListType {
                        enumeration: Vec::new(),
                    },
                ));
            let Some(xtce::EnumeratedParameterTypeContent::EnumerationList(list)) =
                parameter_type.content.last_mut()
            else {
                unreachable!()
            };
            list
        };
        let mut existing = std::mem::take(&mut list.enumeration).into_iter();
        list.enumeration = rows
            .into_iter()
            .filter_map(|row| {
                let value = row.value.trim().parse::<i64>().ok()?;
                let label = row.label.trim();
                if label.is_empty() {
                    return None;
                }
                let mut enumeration = existing.next().unwrap_or(xtce::ValueEnumerationType {
                    value: 0,
                    max_value: None,
                    label: String::new(),
                    short_description: None,
                });
                enumeration.value = value;
                enumeration.max_value = row.max_value.trim().parse().ok();
                enumeration.label = label.to_owned();
                enumeration.short_description =
                    (!row.description.trim().is_empty()).then(|| row.description.trim().to_owned());
                Some(enumeration)
            })
            .collect();
        if list.enumeration.is_empty() {
            list.enumeration.push(xtce::ValueEnumerationType {
                value: 0,
                max_value: None,
                label: "VALUE".to_owned(),
                short_description: None,
            });
        }
    }

    fn editor(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<EnumerationRowForm> {
        if let Some(editor) = self.editors.get(&index).cloned() {
            touch_cache(&mut self.cache_order, index);
            return editor;
        }
        let row = self.rows[index].clone();
        let editor = cx.new(|cx| EnumerationRowForm {
            value_input: input(&row.value, window, cx),
            max_value_input: input(&row.max_value, window, cx),
            label_input: input(&row.label, window, cx),
            description_input: input(&row.description, window, cx),
        });
        self.editors.insert(index, editor.clone());
        touch_cache(&mut self.cache_order, index);
        while self.editors.len() > EDITOR_CACHE_SIZE {
            let Some(evicted) = self.cache_order.pop_front() else {
                break;
            };
            if evicted == index {
                self.cache_order.push_back(evicted);
                continue;
            }
            if let Some(editor) = self.editors.remove(&evicted) {
                self.rows[evicted] = row_data(&editor, cx);
            }
        }
        editor
    }

    fn current_rows(&self, cx: &App) -> Vec<EnumerationRowData> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                self.editors
                    .get(&index)
                    .map_or_else(|| row.clone(), |editor| row_data(editor, cx))
            })
            .collect()
    }

    fn flush_editors(&mut self, cx: &App) {
        for (index, editor) in &self.editors {
            self.rows[*index] = row_data(editor, cx);
        }
    }

    fn add_row(&mut self, cx: &mut Context<Self>) {
        self.flush_editors(cx);
        let index = self.rows.len();
        self.rows.push(EnumerationRowData {
            value: next_value(&self.rows).to_string(),
            max_value: String::new(),
            label: "VALUE".to_owned(),
            description: String::new(),
        });
        self.list_state.splice(index..index, 1);
        self.list_state.scroll_to_reveal_item(index);
        cx.notify();
    }

    fn remove_row(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.rows.len() <= 1 || index >= self.rows.len() {
            return;
        }
        self.flush_editors(cx);
        self.rows.remove(index);
        self.editors.clear();
        self.cache_order.clear();
        self.list_state.splice(index..index + 1, 0);
        cx.notify();
    }

    fn move_row(&mut self, index: usize, target: usize, cx: &mut Context<Self>) {
        if index >= self.rows.len() || target >= self.rows.len() || index == target {
            return;
        }
        self.flush_editors(cx);
        self.rows.swap(index, target);
        self.editors.clear();
        self.cache_order.clear();
        cx.notify();
    }

    fn render_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editor = self.editor(index, window, cx);
        let count = self.rows.len();
        h_flex()
            .h(px(ROW_HEIGHT))
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .w(px(96.))
                    .flex_none()
                    .gap_1()
                    .child(
                        Button::new(format!("move-enumeration-up-{index}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::ArrowUp)
                            .disabled(index == 0)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(target) = index.checked_sub(1) {
                                    this.move_row(index, target, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(format!("move-enumeration-down-{index}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::ArrowDown)
                            .disabled(index + 1 >= count)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.move_row(index, index + 1, cx);
                            })),
                    )
                    .child(
                        Button::new(format!("remove-enumeration-{index}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::Minus)
                            .disabled(count <= 1)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_row(index, cx);
                            })),
                    ),
            )
            .child(editor)
            .into_any_element()
    }
}

impl Render for EnumerationListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.rows.len();
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Enumeration list"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{count} values")),
                            ),
                    )
                    .child(
                        Button::new("add-enumeration-value")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add value")
                            .on_click(cx.listener(|this, _, _, cx| this.add_row(cx))),
                    ),
            )
            .child(
                div()
                    .id("enumeration-list-scroll-boundary")
                    .w_full()
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .id("enumeration-list-horizontal-scroll")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                v_flex()
                                    .min_w(px(840.))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .child(
                                        h_flex()
                                            .h(px(34.))
                                            .px_2()
                                            .gap_2()
                                            .bg(cx.theme().muted.opacity(0.5))
                                            .text_xs()
                                            .font_medium()
                                            .child(div().w(px(96.)).child("Actions"))
                                            .child(div().w(px(90.)).child("Value"))
                                            .child(div().w(px(100.)).child("Max value"))
                                            .child(div().w(px(180.)).child("Label"))
                                            .child(
                                                div().w(px(300.)).flex_none().child("Description"),
                                            ),
                                    )
                                    .child(
                                        list(
                                            self.list_state.clone(),
                                            cx.processor(EnumerationListForm::render_row),
                                        )
                                        .w_full()
                                        .h(px((count.min(8) as f32 * ROW_HEIGHT).max(ROW_HEIGHT))),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for EnumerationRowForm {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(
                div()
                    .w(px(90.))
                    .flex_none()
                    .child(Input::new(&self.value_input)),
            )
            .child(
                div()
                    .w(px(100.))
                    .flex_none()
                    .child(Input::new(&self.max_value_input)),
            )
            .child(
                div()
                    .w(px(180.))
                    .flex_none()
                    .child(Input::new(&self.label_input)),
            )
            .child(
                div()
                    .w(px(300.))
                    .flex_none()
                    .child(Input::new(&self.description_input)),
            )
    }
}

fn rows_from_parameter_type(
    parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
) -> Vec<EnumerationRowData> {
    let Some(xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(parameter_type)) =
        parameter_type
    else {
        return Vec::new();
    };
    parameter_type
        .content
        .iter()
        .find_map(|content| match content {
            xtce::EnumeratedParameterTypeContent::EnumerationList(list) => Some(list),
            _ => None,
        })
        .into_iter()
        .flat_map(|list| &list.enumeration)
        .map(|enumeration| EnumerationRowData {
            value: enumeration.value.to_string(),
            max_value: enumeration
                .max_value
                .map(|value| value.to_string())
                .unwrap_or_default(),
            label: enumeration.label.clone(),
            description: enumeration.short_description.clone().unwrap_or_default(),
        })
        .collect()
}

fn row_data(row: &Entity<EnumerationRowForm>, cx: &App) -> EnumerationRowData {
    let row = row.read(cx);
    EnumerationRowData {
        value: value(&row.value_input, cx),
        max_value: value(&row.max_value_input, cx),
        label: value(&row.label_input, cx),
        description: value(&row.description_input, cx),
    }
}

fn next_value(rows: &[EnumerationRowData]) -> i64 {
    rows.iter()
        .filter_map(|row| {
            row.max_value
                .trim()
                .parse::<i64>()
                .ok()
                .or_else(|| row.value.trim().parse::<i64>().ok())
        })
        .max()
        .unwrap_or(-1)
        .saturating_add(1)
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn touch_cache(cache_order: &mut VecDeque<usize>, index: usize) {
    if let Some(position) = cache_order.iter().position(|candidate| *candidate == index) {
        cache_order.remove(position);
    }
    cache_order.push_back(index);
}

#[cfg(test)]
mod tests {
    use super::{EnumerationRowData, next_value};

    #[test]
    fn next_enumeration_value_follows_the_largest_value_or_range() {
        let rows = vec![
            EnumerationRowData {
                value: "0".to_owned(),
                max_value: String::new(),
                label: "OFF".to_owned(),
                description: String::new(),
            },
            EnumerationRowData {
                value: "1".to_owned(),
                max_value: "3".to_owned(),
                label: "ACTIVE".to_owned(),
                description: String::new(),
            },
        ];

        assert_eq!(next_value(&rows), 4);
    }
}
