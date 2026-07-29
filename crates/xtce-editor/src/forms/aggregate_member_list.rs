use std::collections::{HashMap, VecDeque};

use gpui::{
    AnyElement, App, AppContext, Context, Entity, IntoElement, ListAlignment, ListState,
    ParentElement, Render, Styled, Window, div, list, px,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    v_flex,
};

const ROW_HEIGHT: f32 = 70.;
const NAME_WIDTH: f32 = 220.;
const ACTIONS_WIDTH: f32 = 132.;

#[derive(Clone)]
pub(super) struct MemberRowData {
    pub(super) name: String,
    pub(super) type_ref: String,
    pub(super) initial_value: String,
    pub(super) description: String,
    pub(super) source_index: Option<usize>,
}

impl MemberRowData {
    fn default_row() -> Self {
        Self {
            name: String::new(),
            type_ref: String::new(),
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

    pub(super) fn new_argument_type(
        argument_type: Option<&xtce::ArgumentTypeSetTypeContent>,
        _: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = rows_from_argument_type(argument_type);
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

    pub(super) fn load_argument_type(
        &mut self,
        argument_type: Option<&xtce::ArgumentTypeSetTypeContent>,
        cx: &mut Context<Self>,
    ) {
        let mut rows = rows_from_argument_type(argument_type);
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
                    .and_then(|index| existing.get_mut(index)?.take())
                    .unwrap_or_else(default_member);
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

    pub(super) fn apply_to_argument_type(
        &self,
        argument_type: &mut xtce::ArgumentTypeSetTypeContent,
        cx: &App,
    ) {
        let xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) = argument_type else {
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
                    .and_then(|index| existing.get_mut(index)?.take())
                    .unwrap_or_else(default_member);
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
        let row = MemberRowData::default_row();
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
        let (name_field, type_ref_field) = {
            let row = editor.read(cx);
            (
                div()
                    .w(px(NAME_WIDTH))
                    .flex_none()
                    .child(super::required_input(&row.name, cx)),
                div()
                    .flex_1()
                    .min_w_0()
                    .child(super::required_input(&row.type_ref, cx)),
            )
        };
        h_flex()
            .w_full()
            .h(px(ROW_HEIGHT))
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(name_field)
            .child(type_ref_field)
            .child(
                h_flex()
                    .w(px(ACTIONS_WIDTH))
                    .flex_none()
                    .gap_1()
                    .child(
                        Button::new(format!("aggregate-member-options-{index}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::Ellipsis)
                            .tooltip("Member options")
                            .on_click(move |_, window, cx| {
                                open_member_options(editor.clone(), window, cx);
                            }),
                    )
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
                        super::row_remove_button(
                            format!("remove-member-{index}"),
                            "Remove aggregate member",
                        )
                        .disabled(count <= 1)
                        .on_click(cx.listener(move |this, _, _, cx| this.remove(index, cx))),
                    ),
            )
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
                                    .child(format!(
                                        "{} · packed in this order",
                                        super::count_label(count, "member", "members")
                                    )),
                            ),
                    )
                    .child(
                        super::collection_add_button("add-aggregate-member", "Add member")
                            .on_click(cx.listener(|this, _, _, cx| this.add(cx))),
                    ),
            )
            .child(
                super::list_table_frame(cx)
                    .child(
                        h_flex()
                            .h(px(34.))
                            .px_2()
                            .gap_2()
                            .bg(cx.theme().muted.opacity(0.5))
                            .text_xs()
                            .font_medium()
                            .child(
                                div()
                                    .w(px(NAME_WIDTH))
                                    .flex_none()
                                    .child(super::required_label("Name", cx)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(super::required_label("Type reference", cx)),
                            )
                            .child(div().w(px(ACTIONS_WIDTH)).flex_none().child("Actions")),
                    )
                    .child(
                        list(
                            self.list_state.clone(),
                            cx.processor(AggregateMemberListForm::render_row),
                        )
                        .w_full()
                        .h(px((count.min(8) as f32 * ROW_HEIGHT).max(ROW_HEIGHT))),
                    ),
            )
    }
}

fn open_member_options(editor: Entity<MemberRowForm>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title("Member options")
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                let row = editor.read(cx);
                content.child(
                    super::form_dialog_content()
                        .child(super::field(
                            "Initial value",
                            "Optional",
                            &row.initial_value,
                            cx,
                        ))
                        .child(super::field(
                            "Description",
                            "Optional",
                            &row.description,
                            cx,
                        )),
                )
            })
    });
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

pub(super) fn rows_from_argument_type(
    argument_type: Option<&xtce::ArgumentTypeSetTypeContent>,
) -> Vec<MemberRowData> {
    let Some(xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value)) = argument_type else {
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
        name: String::new(),
        type_ref: String::new(),
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
