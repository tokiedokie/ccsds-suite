use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, div, px,
};
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    discrete_lookup::DiscreteLookupListForm, dynamic_value::DynamicValueForm, field,
    impl_select_item,
};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum IntegerValueKind {
    Fixed,
    Dynamic,
    DiscreteLookup,
}
impl_select_item!(IntegerValueKind);

struct IntegerValueEditor {
    kind: IntegerValueKind,
    kind_select: Entity<SelectState<Vec<IntegerValueKind>>>,
    fixed: Entity<InputState>,
    dynamic: Entity<DynamicValueForm>,
    discrete: Entity<DiscreteLookupListForm>,
    _subscriptions: Vec<Subscription>,
}

struct DimensionRow {
    starting: Entity<IntegerValueEditor>,
    ending: Entity<IntegerValueEditor>,
}

pub(super) struct DimensionListForm {
    rows: Vec<Entity<DimensionRow>>,
}

impl DimensionListForm {
    pub(super) fn new(
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = dimension_rows(parameter_type, window, cx);
        cx.new(move |_| Self { rows })
    }

    pub(super) fn load(
        &mut self,
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = dimension_rows(parameter_type, window, cx);
        cx.notify();
    }

    pub(super) fn reset(&mut self, array: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.rows = if array {
            vec![dimension_row(None, window, cx)]
        } else {
            Vec::new()
        };
        cx.notify();
    }

    pub(super) fn apply_to(
        &self,
        parameter_type: &mut xtce::ParameterTypeSetTypeContent,
        cx: &App,
    ) {
        let xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) = parameter_type else {
            return;
        };
        let dimensions = self
            .rows
            .iter()
            .map(|row| {
                let row = row.read(cx);
                xtce::DimensionType {
                    starting_index: row.starting.read(cx).value(cx),
                    ending_index: row.ending.read(cx).value(cx),
                }
            })
            .collect::<Vec<_>>();
        if !dimensions.is_empty() {
            value.dimension_list.dimension = dimensions;
        }
    }
}

impl Render for DimensionListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Dimensions"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} dimension(s)", self.rows.len())),
                            ),
                    )
                    .child(
                        Button::new("add-parameter-type-dimension")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add dimension")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(dimension_row(None, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                h_flex()
                    .w_full()
                    .p_3()
                    .gap_3()
                    .items_center()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .w(px(110.))
                            .text_sm()
                            .font_medium()
                            .child(format!("Dimension {}", index + 1)),
                    )
                    .child(index_summary("Starting index", &row_read.starting, cx))
                    .child(index_summary("Ending index", &row_read.ending, cx))
                    .child(
                        super::row_remove_button(
                            format!("remove-parameter-type-dimension-{index}"),
                            "Remove dimension",
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if this.rows.len() > 1 {
                                this.rows.remove(index);
                                cx.notify();
                            }
                        })),
                    )
            }))
    }
}

impl IntegerValueEditor {
    fn value(&self, cx: &App) -> xtce::IntegerValueType {
        match self.kind {
            IntegerValueKind::Fixed => xtce::IntegerValueType::FixedValue(
                self.fixed
                    .read(cx)
                    .value()
                    .trim()
                    .parse()
                    .unwrap_or_default(),
            ),
            IntegerValueKind::Dynamic => {
                xtce::IntegerValueType::DynamicValue(self.dynamic.read(cx).value(cx))
            }
            IntegerValueKind::DiscreteLookup => {
                xtce::IntegerValueType::DiscreteLookupList(self.discrete.read(cx).value(cx))
            }
        }
    }

    fn summary(&self, cx: &App) -> String {
        match self.kind {
            IntegerValueKind::Fixed => self.fixed.read(cx).value().to_string(),
            IntegerValueKind::Dynamic => "Dynamic value".to_owned(),
            IntegerValueKind::DiscreteLookup => "Discrete lookup".to_owned(),
        }
    }
}

impl Render for IntegerValueEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut form = v_flex()
            .w_full()
            .gap_4()
            .child(select_field("Type", &self.kind_select));
        form = match self.kind {
            IntegerValueKind::Fixed => form.child(field("Fixed value", "Integer", &self.fixed, cx)),
            IntegerValueKind::Dynamic => form.child(self.dynamic.clone()),
            IntegerValueKind::DiscreteLookup => form.child(self.discrete.clone()),
        };
        form
    }
}

fn index_summary(
    label: &'static str,
    editor: &Entity<IntegerValueEditor>,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .flex_1()
        .gap_1()
        .child(div().text_xs().font_medium().child(label))
        .child(
            h_flex()
                .gap_2()
                .child(div().flex_1().text_sm().child(editor.read(cx).summary(cx)))
                .child(
                    Button::new(format!("edit-{label}-{}", editor.entity_id()))
                        .small()
                        .ghost()
                        .icon(IconName::Ellipsis)
                        .label("Edit")
                        .on_click({
                            let editor = editor.clone();
                            move |_, window, cx| {
                                open_integer_value(label, editor.clone(), window, cx);
                            }
                        }),
                ),
        )
}

fn open_integer_value(
    label: &'static str,
    editor: Entity<IntegerValueEditor>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title(label)
            .w(px(760.))
            .content(move |content, _, _| content.child(div().p_4().child(editor.clone())))
    });
}

fn dimension_rows(
    parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<DimensionRow>> {
    match parameter_type {
        Some(xtce::ParameterTypeSetTypeContent::ArrayParameterType(value)) => value
            .dimension_list
            .dimension
            .iter()
            .map(|value| dimension_row(Some(value), window, cx))
            .collect(),
        _ => Vec::new(),
    }
}

fn dimension_row(
    value: Option<&xtce::DimensionType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<DimensionRow> {
    let starting = integer_value_editor(value.map(|value| &value.starting_index), window, cx);
    let ending = integer_value_editor(value.map(|value| &value.ending_index), window, cx);
    cx.new(move |_| DimensionRow { starting, ending })
}

fn integer_value_editor(
    value: Option<&xtce::IntegerValueType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<IntegerValueEditor> {
    let kind = match value {
        Some(xtce::IntegerValueType::DynamicValue(_)) => IntegerValueKind::Dynamic,
        Some(xtce::IntegerValueType::DiscreteLookupList(_)) => IntegerValueKind::DiscreteLookup,
        _ => IntegerValueKind::Fixed,
    };
    let fixed_value = match value {
        Some(xtce::IntegerValueType::FixedValue(value)) => value.to_string(),
        _ => "0".to_owned(),
    };
    let dynamic = DynamicValueForm::new(
        value.and_then(|value| match value {
            xtce::IntegerValueType::DynamicValue(value) => Some(value),
            _ => None,
        }),
        window,
        cx,
    );
    let discrete = DiscreteLookupListForm::new(
        value.and_then(|value| match value {
            xtce::IntegerValueType::DiscreteLookupList(value) => Some(value),
            _ => None,
        }),
        window,
        cx,
    );
    let fixed = cx.new(|cx| InputState::new(window, cx).default_value(fixed_value));
    cx.new(move |cx| {
        let kind_select = select(IntegerValueKind::VARIANTS, kind, window, cx);
        let fixed_for_kind = fixed.clone();
        let dynamic_for_kind = dynamic.clone();
        let discrete_for_kind = discrete.clone();
        let subscription = cx.subscribe_in(
            &kind_select,
            window,
            move |this: &mut IntegerValueEditor,
                  _,
                  event: &SelectEvent<Vec<IntegerValueKind>>,
                  window,
                  cx| {
                let SelectEvent::Confirm(Some(kind)) = event else {
                    return;
                };
                if this.kind == *kind {
                    return;
                }
                this.kind = *kind;
                fixed_for_kind.update(cx, |input, cx| {
                    input.set_value("0".to_owned(), window, cx);
                });
                dynamic_for_kind.update(cx, |form, cx| {
                    form.load(None, window, cx);
                });
                discrete_for_kind.update(cx, |form, cx| {
                    form.load(None, window, cx);
                });
                cx.notify();
            },
        );
        IntegerValueEditor {
            kind,
            kind_select,
            fixed,
            dynamic,
            discrete,
            _subscriptions: vec![subscription],
        }
    })
}

fn select<T>(
    options: &'static [T],
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<T>>>
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    let index = options
        .iter()
        .position(|option| option == &selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            options.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn select_field<T>(label: &'static str, select: &Entity<SelectState<Vec<T>>>) -> gpui::Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    super::select_field(label, "", select)
}

#[cfg(test)]
mod tests {
    #[test]
    fn dimension_accepts_all_integer_value_variants() {
        let dimensions = xtce::DimensionListType {
            dimension: vec![
                xtce::DimensionType {
                    starting_index: xtce::IntegerValueType::FixedValue(0),
                    ending_index: xtce::IntegerValueType::FixedValue(3),
                },
                xtce::DimensionType {
                    starting_index: xtce::IntegerValueType::DynamicValue(xtce::DynamicValueType {
                        parameter_instance_ref: xtce::ParameterInstanceRefType {
                            parameter_ref: "Start".to_owned(),
                            instance: 0,
                            use_calibrated_value: true,
                        },
                        linear_adjustment: None,
                    }),
                    ending_index: xtce::IntegerValueType::DiscreteLookupList(
                        xtce::DiscreteLookupListType {
                            default_value: 7,
                            discrete_lookup: Vec::new(),
                        },
                    ),
                },
            ],
        };
        assert_eq!(dimensions.dimension.len(), 2);
    }
}
