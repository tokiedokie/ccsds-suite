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

#[derive(Clone)]
struct MemberRowData {
    name: String,
    type_ref: String,
    initial_value: String,
    description: String,
    source_index: Option<usize>,
}

impl MemberRowData {
    fn default_row() -> Self {
        Self {
            name: "member".to_owned(),
            type_ref: "MemberType".to_owned(),
            initial_value: String::new(),
            description: String::new(),
            source_index: None,
        }
    }
}

pub(super) struct AggregateMemberListForm {
    rows: Vec<MemberRowData>,
    editors: HashMap<usize, Entity<MemberRowForm>>,
    cache_order: VecDeque<usize>,
    list_state: ListState,
}

struct MemberRowForm {
    name: Entity<InputState>,
    type_ref: Entity<InputState>,
    initial_value: Entity<InputState>,
    description: Entity<InputState>,
}

impl AggregateMemberListForm {
    pub(super) fn new(
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        _: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = rows_from_type(parameter_type);
        cx.new(move |_| Self::from_rows(rows))
    }

    fn from_rows(mut rows: Vec<MemberRowData>) -> Self {
        if rows.is_empty() {
            rows.push(MemberRowData::default_row());
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
        let mut rows = rows_from_type(parameter_type);
        if rows.is_empty() {
            rows.push(MemberRowData::default_row());
        }
        self.rows = rows;
        self.editors.clear();
        self.cache_order.clear();
        self.list_state
            .reset_with_uniform_height(self.rows.len(), px(ROW_HEIGHT));
        cx.notify();
    }

    pub(super) fn reset_to_default(&mut self, cx: &mut Context<Self>) {
        self.rows = vec![MemberRowData::default_row()];
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
        let xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) = parameter_type
        else {
            return;
        };
        let rows = self.current_rows(cx);
        let mut existing = std::mem::take(&mut value.member_list.member)
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        value.member_list.member = rows
            .into_iter()
            .filter_map(|row| {
                let name = row.name.trim();
                let type_ref = row.type_ref.trim();
                if name.is_empty() || type_ref.is_empty() {
                    return None;
                }
                let mut member = row
                    .source_index
                    .and_then(|index| existing.get_mut(index))
                    .and_then(Option::take)
                    .unwrap_or(xtce::MemberType {
                        short_description: None,
                        name: String::new(),
                        type_ref: String::new(),
                        initial_value: None,
                        long_description: None,
                        alias_set: None,
                        ancillary_data_set: None,
                    });
                member.name = name.to_owned();
                member.type_ref = type_ref.to_owned();
                member.initial_value = optional(&row.initial_value);
                member.short_description = optional(&row.description);
                Some(member)
            })
            .collect();
        if value.member_list.member.is_empty() {
            value.member_list.member.push(default_member());
        }
    }

    fn editor(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<MemberRowForm> {
        if let Some(editor) = self.editors.get(&index).cloned() {
            touch(&mut self.cache_order, index);
            return editor;
        }
        let row = self.rows[index].clone();
        let editor = cx.new(|cx| MemberRowForm {
            name: input(&row.name, window, cx),
            type_ref: input(&row.type_ref, window, cx),
            initial_value: input(&row.initial_value, window, cx),
            description: input(&row.description, window, cx),
        });
        self.editors.insert(index, editor.clone());
        touch(&mut self.cache_order, index);
        while self.editors.len() > 24 {
            let Some(evicted) = self.cache_order.pop_front() else {
                break;
            };
            if evicted == index {
                self.cache_order.push_back(evicted);
                continue;
            }
            if let Some(editor) = self.editors.remove(&evicted) {
                self.rows[evicted] = row_data(&editor, &self.rows[evicted], cx);
            }
        }
        editor
    }

    fn current_rows(&self, cx: &App) -> Vec<MemberRowData> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                self.editors
                    .get(&index)
                    .map_or_else(|| row.clone(), |editor| row_data(editor, row, cx))
            })
            .collect()
    }

    fn flush(&mut self, cx: &App) {
        for (index, editor) in &self.editors {
            self.rows[*index] = row_data(editor, &self.rows[*index], cx);
        }
    }

    fn add(&mut self, cx: &mut Context<Self>) {
        self.flush(cx);
        let index = self.rows.len();
        let mut row = MemberRowData::default_row();
        row.name = format!("member{}", index + 1);
        self.rows.push(row);
        self.list_state.splice(index..index, 1);
        self.list_state.scroll_to_reveal_item(index);
        cx.notify();
    }

    fn remove(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.rows.len() <= 1 || index >= self.rows.len() {
            return;
        }
        self.flush(cx);
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
        self.flush(cx);
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
                        Button::new(format!("move-member-up-{index}"))
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
                        Button::new(format!("move-member-down-{index}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::ArrowDown)
                            .disabled(index + 1 >= count)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.move_row(index, index + 1, cx);
                            })),
                    )
                    .child(
                        Button::new(format!("remove-member-{index}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::Minus)
                            .disabled(count <= 1)
                            .on_click(cx.listener(move |this, _, _, cx| this.remove(index, cx))),
                    ),
            )
            .child(editor)
            .into_any_element()
    }
}

impl Render for AggregateMemberListForm {
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
                            .child(div().text_sm().font_medium().child("Member list"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{count} members · packed in this order")),
                            ),
                    )
                    .child(
                        Button::new("add-aggregate-member")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add member")
                            .on_click(cx.listener(|this, _, _, cx| this.add(cx))),
                    ),
            )
            .child(
                div()
                    .id("aggregate-member-scroll-boundary")
                    .w_full()
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .id("aggregate-member-horizontal-scroll")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                v_flex()
                                    .min_w(px(900.))
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
                                            .child(div().w(px(180.)).child("Name"))
                                            .child(div().w(px(220.)).child("Type reference"))
                                            .child(div().w(px(160.)).child("Initial value"))
                                            .child(div().w(px(220.)).child("Description")),
                                    )
                                    .child(
                                        list(
                                            self.list_state.clone(),
                                            cx.processor(AggregateMemberListForm::render_row),
                                        )
                                        .w_full()
                                        .h(px((count.min(8) as f32 * ROW_HEIGHT).max(ROW_HEIGHT))),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for MemberRowForm {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .gap_2()
            .child(div().w(px(180.)).flex_none().child(Input::new(&self.name)))
            .child(
                div()
                    .w(px(220.))
                    .flex_none()
                    .child(Input::new(&self.type_ref)),
            )
            .child(
                div()
                    .w(px(160.))
                    .flex_none()
                    .child(Input::new(&self.initial_value)),
            )
            .child(
                div()
                    .w(px(220.))
                    .flex_none()
                    .child(Input::new(&self.description)),
            )
    }
}

fn rows_from_type(
    parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
) -> Vec<MemberRowData> {
    let Some(xtce::ParameterTypeSetTypeContent::AggregateParameterType(value)) = parameter_type
    else {
        return Vec::new();
    };
    value
        .member_list
        .member
        .iter()
        .enumerate()
        .map(|(index, member)| MemberRowData {
            name: member.name.clone(),
            type_ref: member.type_ref.clone(),
            initial_value: member.initial_value.clone().unwrap_or_default(),
            description: member.short_description.clone().unwrap_or_default(),
            source_index: Some(index),
        })
        .collect()
}

fn row_data(row: &Entity<MemberRowForm>, original: &MemberRowData, cx: &App) -> MemberRowData {
    let row = row.read(cx);
    MemberRowData {
        name: value(&row.name, cx),
        type_ref: value(&row.type_ref, cx),
        initial_value: value(&row.initial_value, cx),
        description: value(&row.description, cx),
        source_index: original.source_index,
    }
}

fn default_member() -> xtce::MemberType {
    xtce::MemberType {
        short_description: None,
        name: "member".to_owned(),
        type_ref: "MemberType".to_owned(),
        initial_value: None,
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
    }
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn optional(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.trim().to_owned())
}

fn touch(order: &mut VecDeque<usize>, index: usize) {
    if let Some(position) = order.iter().position(|item| *item == index) {
        order.remove(position);
    }
    order.push_back(index);
}
