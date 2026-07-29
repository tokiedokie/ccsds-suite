use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, IndexPath, Selectable, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{InputEvent, InputState},
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item};

const MATH_OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "^", "y^x", "ln", "log", "e^x", "1/x", "x!", "tan", "cos", "sin",
    "atan", "atan2", "acos", "asin", "tanh", "cosh", "sinh", "atanh", "acosh", "asinh", "swap",
    "drop", "dup", "over", "<<", ">>", "&", "|", "&&", "||", "!", "abs", "div", "int", ">", ">=",
    "<", "<=", "==", "!=", "min", "max", "xor", "~",
];

#[derive(Clone, Debug, PartialEq)]
pub(super) enum RpnOperationEntry {
    Value(String),
    ThisParameter(String),
    Operator(String),
    ParameterInstance {
        parameter_ref: String,
        instance: i64,
        use_calibrated_value: bool,
    },
    ArgumentInstance {
        argument_ref: String,
        use_calibrated_value: bool,
    },
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum EntryKind {
    #[strum(serialize = "Value")]
    ValueOperand,
    #[strum(serialize = "This parameter")]
    ThisParameterOperand,
    Operator,
    #[strum(serialize = "Parameter reference")]
    ParameterInstanceRef,
    #[strum(serialize = "Argument reference")]
    ArgumentInstanceRef,
}
impl_select_item!(EntryKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

pub(super) struct RpnOperationForm {
    rows: Vec<RpnOperationRow>,
    selected: usize,
    allow_argument_instances: bool,
}

struct RpnOperationRow {
    kind: Entity<SelectState<Vec<EntryKind>>>,
    value: Entity<InputState>,
    instance: Entity<InputState>,
    calibrated: Entity<SelectState<Vec<BooleanChoice>>>,
    _subscriptions: Vec<Subscription>,
}

impl RpnOperationForm {
    pub(super) fn new(
        entries: Vec<RpnOperationEntry>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let entries = normalized(entries);
        cx.new(move |cx| Self {
            rows: row_values(entries, false, window, cx),
            selected: 0,
            allow_argument_instances: false,
        })
    }

    pub(super) fn new_argument(
        entries: Vec<RpnOperationEntry>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let entries = normalized(entries);
        cx.new(move |cx| Self {
            rows: row_values(entries, true, window, cx),
            selected: 0,
            allow_argument_instances: true,
        })
    }

    pub(super) fn load(
        &mut self,
        entries: Vec<RpnOperationEntry>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = row_values(
            normalized(entries),
            self.allow_argument_instances,
            window,
            cx,
        );
        self.selected = 0;
        cx.notify();
    }

    pub(super) fn entries(&self, cx: &App) -> Vec<RpnOperationEntry> {
        self.rows.iter().map(|row| row.entry(cx)).collect()
    }

    fn add_entry(&mut self, entry: RpnOperationEntry, window: &mut Window, cx: &mut Context<Self>) {
        self.rows.push(RpnOperationRow::new(
            entry,
            self.allow_argument_instances,
            window,
            cx,
        ));
        self.selected = self.rows.len() - 1;
        cx.notify();
    }

    fn render_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let index = self.selected.min(self.rows.len() - 1);
        let row = &self.rows[index];
        let kind = selected_value(&row.kind, EntryKind::ValueOperand, cx);
        let operator = input_value(&row.value, cx);
        let row_count = self.rows.len();

        v_flex()
            .w_full()
            .p_3()
            .gap_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .child(format!("Edit token {}", index + 1)),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("move-selected-rpn-entry-left")
                                    .xsmall()
                                    .ghost()
                                    .icon(IconName::ArrowLeft)
                                    .disabled(index == 0)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index > 0 && index < this.rows.len() {
                                            this.rows.swap(index, index - 1);
                                            this.selected = index - 1;
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("move-selected-rpn-entry-right")
                                    .xsmall()
                                    .ghost()
                                    .icon(IconName::ArrowRight)
                                    .disabled(index + 1 >= row_count)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index + 1 < this.rows.len() {
                                            this.rows.swap(index, index + 1);
                                            this.selected = index + 1;
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                super::row_remove_button(
                                    "remove-selected-rpn-entry",
                                    "Remove selected operation",
                                )
                                .disabled(row_count <= 1)
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        if this.rows.len() > 1 && index < this.rows.len() {
                                            this.rows.remove(index);
                                            this.selected =
                                                index.min(this.rows.len().saturating_sub(1));
                                            cx.notify();
                                        }
                                    },
                                )),
                            ),
                    ),
            )
            .child(super::select_field("Token type", "Required", &row.kind))
            .when(kind == EntryKind::ValueOperand, |form| {
                form.child(field("Value", "Required", &row.value, cx))
            })
            .when(kind == EntryKind::ThisParameterOperand, |form| {
                form.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Uses this parameter's calibrated value."),
                )
            })
            .when(kind == EntryKind::Operator, |form| {
                form.child(
                    v_flex()
                        .gap_2()
                        .child(div().text_sm().font_medium().child("Operator"))
                        .child(
                            h_flex()
                                .flex_wrap()
                                .gap_1()
                                .children(MATH_OPERATORS.iter().map(|candidate| {
                                    let candidate = *candidate;
                                    let selected = operator == candidate;
                                    let input = row.value.clone();
                                    Button::new(format!("select-rpn-operator-{index}-{candidate}"))
                                        .xsmall()
                                        .selected(selected)
                                        .label(operator_label(candidate))
                                        .on_click(move |_, window, cx| {
                                            input.update(cx, |input, cx| {
                                                input.set_value(candidate, window, cx);
                                            });
                                        })
                                })),
                        ),
                )
            })
            .when(kind == EntryKind::ParameterInstanceRef, |form| {
                form.child(field("Parameter reference", "Required", &row.value, cx))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Instance",
                                "Optional; defaults to 0",
                                &row.instance,
                                cx,
                            ))
                            .child(super::select_field(
                                "Use calibrated value",
                                "Required",
                                &row.calibrated,
                            )),
                    )
            })
            .when(kind == EntryKind::ArgumentInstanceRef, |form| {
                form.child(field("Argument reference", "Required", &row.value, cx))
                    .child(super::select_field(
                        "Use calibrated value",
                        "Required",
                        &row.calibrated,
                    ))
            })
    }
}

impl RpnOperationRow {
    fn new(
        entry: RpnOperationEntry,
        allow_argument_instances: bool,
        window: &mut Window,
        cx: &mut Context<RpnOperationForm>,
    ) -> Self {
        let (kind, value, instance, calibrated) = entry_values(entry);
        let kind = select(entry_kinds(allow_argument_instances), kind, window, cx);
        let value = input(&value, window, cx);
        let instance = input(&instance, window, cx);
        let calibrated = select(BooleanChoice::VARIANTS, calibrated, window, cx);
        let subscriptions = vec![
            cx.subscribe(&kind, |_, _, _: &SelectEvent<Vec<EntryKind>>, cx| {
                cx.notify()
            }),
            cx.subscribe(&value, |_, _, _: &InputEvent, cx| cx.notify()),
            cx.subscribe(&instance, |_, _, _: &InputEvent, cx| cx.notify()),
            cx.subscribe(
                &calibrated,
                |_, _, _: &SelectEvent<Vec<BooleanChoice>>, cx| cx.notify(),
            ),
        ];
        Self {
            kind,
            value,
            instance,
            calibrated,
            _subscriptions: subscriptions,
        }
    }

    fn entry(&self, cx: &App) -> RpnOperationEntry {
        let value = input_value(&self.value, cx);
        match selected_value(&self.kind, EntryKind::ValueOperand, cx) {
            EntryKind::ValueOperand => RpnOperationEntry::Value(value),
            EntryKind::ThisParameterOperand => RpnOperationEntry::ThisParameter(String::new()),
            EntryKind::Operator => RpnOperationEntry::Operator(value),
            EntryKind::ParameterInstanceRef => RpnOperationEntry::ParameterInstance {
                parameter_ref: value,
                instance: input_value(&self.instance, cx).trim().parse().unwrap_or(0),
                use_calibrated_value: selected_value(&self.calibrated, BooleanChoice::True, cx)
                    == BooleanChoice::True,
            },
            EntryKind::ArgumentInstanceRef => RpnOperationEntry::ArgumentInstance {
                argument_ref: value,
                use_calibrated_value: selected_value(&self.calibrated, BooleanChoice::True, cx)
                    == BooleanChoice::True,
            },
        }
    }
}

impl Render for RpnOperationForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self.entries(cx);
        let preview = expression_preview(&entries);

        v_flex()
            .w_full()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(div().text_sm().font_medium().child("RPN operation"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Tokens are evaluated from left to right."),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .flex_wrap()
                    .gap_2()
                    .children(entries.iter().enumerate().map(|(index, entry)| {
                        Button::new(format!("select-rpn-token-{index}"))
                            .small()
                            .selected(index == self.selected)
                            .label(token_label(entry))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected = index;
                                cx.notify();
                            }))
                    })),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_1()
                    .child(
                        Button::new("add-rpn-value")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Plus)
                            .label("Value")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_entry(
                                    RpnOperationEntry::Value("0".to_owned()),
                                    window,
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new("add-rpn-this-parameter")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Plus)
                            .label("This parameter")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_entry(
                                    RpnOperationEntry::ThisParameter(String::new()),
                                    window,
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new("add-rpn-parameter")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Plus)
                            .label("Parameter")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_entry(
                                    RpnOperationEntry::ParameterInstance {
                                        parameter_ref: String::new(),
                                        instance: 0,
                                        use_calibrated_value: true,
                                    },
                                    window,
                                    cx,
                                );
                            })),
                    )
                    .when(self.allow_argument_instances, |toolbar| {
                        toolbar.child(
                            Button::new("add-rpn-argument")
                                .xsmall()
                                .ghost()
                                .icon(IconName::Plus)
                                .label("Argument")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.add_entry(
                                        RpnOperationEntry::ArgumentInstance {
                                            argument_ref: String::new(),
                                            use_calibrated_value: true,
                                        },
                                        window,
                                        cx,
                                    );
                                })),
                        )
                    })
                    .child(
                        Button::new("add-rpn-operator")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Plus)
                            .label("Operator")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_entry(
                                    RpnOperationEntry::Operator("+".to_owned()),
                                    window,
                                    cx,
                                );
                            })),
                    ),
            )
            .child(self.render_editor(cx))
            .child(
                v_flex()
                    .p_3()
                    .gap_1()
                    .rounded_md()
                    .bg(cx.theme().secondary)
                    .child(div().text_xs().font_medium().child("Expression preview"))
                    .child(
                        div()
                            .text_sm()
                            .when(preview.is_err(), |preview| {
                                preview.text_color(cx.theme().danger)
                            })
                            .child(preview.unwrap_or_else(|message| message)),
                    ),
            )
    }
}

fn normalized(mut entries: Vec<RpnOperationEntry>) -> Vec<RpnOperationEntry> {
    if entries.is_empty() {
        entries.push(RpnOperationEntry::Value("0".to_owned()));
    }
    entries
}

fn row_values(
    entries: Vec<RpnOperationEntry>,
    allow_argument_instances: bool,
    window: &mut Window,
    cx: &mut Context<RpnOperationForm>,
) -> Vec<RpnOperationRow> {
    entries
        .into_iter()
        .map(|entry| RpnOperationRow::new(entry, allow_argument_instances, window, cx))
        .collect()
}

fn entry_kinds(allow_argument_instances: bool) -> &'static [EntryKind] {
    if allow_argument_instances {
        EntryKind::VARIANTS
    } else {
        &[
            EntryKind::ValueOperand,
            EntryKind::ThisParameterOperand,
            EntryKind::Operator,
            EntryKind::ParameterInstanceRef,
        ]
    }
}

fn entry_values(entry: RpnOperationEntry) -> (EntryKind, String, String, BooleanChoice) {
    match entry {
        RpnOperationEntry::Value(value) => (
            EntryKind::ValueOperand,
            value,
            "0".to_owned(),
            BooleanChoice::True,
        ),
        RpnOperationEntry::ThisParameter(value) => (
            EntryKind::ThisParameterOperand,
            value,
            "0".to_owned(),
            BooleanChoice::True,
        ),
        RpnOperationEntry::Operator(value) => (
            EntryKind::Operator,
            value,
            "0".to_owned(),
            BooleanChoice::True,
        ),
        RpnOperationEntry::ParameterInstance {
            parameter_ref,
            instance,
            use_calibrated_value,
        } => (
            EntryKind::ParameterInstanceRef,
            parameter_ref,
            instance.to_string(),
            if use_calibrated_value {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
        ),
        RpnOperationEntry::ArgumentInstance {
            argument_ref,
            use_calibrated_value,
        } => (
            EntryKind::ArgumentInstanceRef,
            argument_ref,
            "0".to_owned(),
            if use_calibrated_value {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
        ),
    }
}

fn token_label(entry: &RpnOperationEntry) -> String {
    match entry {
        RpnOperationEntry::Value(value) if value.trim().is_empty() => "Value".to_owned(),
        RpnOperationEntry::Value(value) => value.clone(),
        RpnOperationEntry::ThisParameter(_) => "This parameter".to_owned(),
        RpnOperationEntry::Operator(operator) => operator_label(operator).to_owned(),
        RpnOperationEntry::ParameterInstance { parameter_ref, .. }
            if parameter_ref.trim().is_empty() =>
        {
            "Parameter".to_owned()
        }
        RpnOperationEntry::ParameterInstance {
            parameter_ref,
            instance,
            ..
        } => format!("{parameter_ref}[{instance}]"),
        RpnOperationEntry::ArgumentInstance { argument_ref, .. }
            if argument_ref.trim().is_empty() =>
        {
            "Argument".to_owned()
        }
        RpnOperationEntry::ArgumentInstance { argument_ref, .. } => argument_ref.clone(),
    }
}

fn operator_label(operator: &str) -> &str {
    match operator {
        "*" => "×",
        "/" => "÷",
        _ => operator,
    }
}

fn expression_preview(entries: &[RpnOperationEntry]) -> Result<String, String> {
    let mut stack = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        match entry {
            RpnOperationEntry::Value(value) => {
                stack.push(if value.trim().is_empty() {
                    "value".to_owned()
                } else {
                    value.clone()
                });
            }
            RpnOperationEntry::ThisParameter(_) => stack.push("this parameter".to_owned()),
            RpnOperationEntry::ParameterInstance { parameter_ref, .. } => {
                stack.push(if parameter_ref.trim().is_empty() {
                    "parameter".to_owned()
                } else {
                    parameter_ref.clone()
                });
            }
            RpnOperationEntry::ArgumentInstance { argument_ref, .. } => {
                stack.push(if argument_ref.trim().is_empty() {
                    "argument".to_owned()
                } else {
                    argument_ref.clone()
                });
            }
            RpnOperationEntry::Operator(operator) => {
                apply_preview_operator(operator, &mut stack).map_err(|required| {
                    format!(
                        "Token {} ({}) needs {required} operand(s).",
                        index + 1,
                        operator_label(operator)
                    )
                })?;
            }
        }
    }
    match stack.len() {
        1 => Ok(stack.pop().unwrap_or_default()),
        count => Err(format!(
            "The operation leaves {count} values on the stack; one is required."
        )),
    }
}

fn apply_preview_operator(operator: &str, stack: &mut Vec<String>) -> Result<(), usize> {
    match operator {
        "swap" => {
            require_stack(stack, 2)?;
            let len = stack.len();
            stack.swap(len - 1, len - 2);
        }
        "drop" => {
            require_stack(stack, 1)?;
            stack.pop();
        }
        "dup" => {
            require_stack(stack, 1)?;
            stack.push(stack.last().cloned().unwrap_or_default());
        }
        "over" => {
            require_stack(stack, 2)?;
            stack.push(stack[stack.len() - 2].clone());
        }
        operator if is_unary_operator(operator) => {
            require_stack(stack, 1)?;
            let value = stack.pop().unwrap_or_default();
            stack.push(unary_preview(operator, value));
        }
        operator => {
            require_stack(stack, 2)?;
            let right = stack.pop().unwrap_or_default();
            let left = stack.pop().unwrap_or_default();
            stack.push(binary_preview(operator, left, right));
        }
    }
    Ok(())
}

fn require_stack(stack: &[String], required: usize) -> Result<(), usize> {
    if stack.len() < required {
        Err(required)
    } else {
        Ok(())
    }
}

fn is_unary_operator(operator: &str) -> bool {
    matches!(
        operator,
        "ln" | "log"
            | "e^x"
            | "1/x"
            | "x!"
            | "tan"
            | "cos"
            | "sin"
            | "atan"
            | "acos"
            | "asin"
            | "tanh"
            | "cosh"
            | "sinh"
            | "atanh"
            | "acosh"
            | "asinh"
            | "abs"
            | "div"
            | "int"
    )
}

fn unary_preview(operator: &str, value: String) -> String {
    match operator {
        "1/x" => format!("(1 / {value})"),
        "x!" => format!("({value}!)"),
        "e^x" => format!("exp({value})"),
        _ => format!("{operator}({value})"),
    }
}

fn binary_preview(operator: &str, left: String, right: String) -> String {
    match operator {
        "min" | "max" | "atan2" => format!("{operator}({left}, {right})"),
        "y^x" => format!("({right} ^ {left})"),
        _ => format!("({left} {} {right})", operator_label(operator)),
    }
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn input_value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn select<T>(
    choices: &[T],
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<T>>>
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    let selected_index = choices
        .iter()
        .position(|choice| *choice == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            choices.to_vec(),
            Some(IndexPath::default().row(selected_index)),
            window,
            cx,
        )
    })
}

fn selected_value<T>(select: &Entity<SelectState<Vec<T>>>, fallback: T, cx: &App) -> T
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{RpnOperationEntry, entry_values, expression_preview, normalized};

    #[test]
    fn an_empty_operation_gets_a_zero_operand() {
        assert_eq!(
            normalized(Vec::new()),
            vec![RpnOperationEntry::Value("0".to_owned())]
        );
    }

    #[test]
    fn parameter_entry_fields_are_preserved() {
        let entry = RpnOperationEntry::ParameterInstance {
            parameter_ref: "P1".to_owned(),
            instance: 2,
            use_calibrated_value: false,
        };
        let (_, reference, instance, calibrated) = entry_values(entry);
        assert_eq!(reference, "P1");
        assert_eq!(instance, "2");
        assert_eq!(calibrated.to_string(), "false");
    }

    #[test]
    fn rpn_entries_are_previewed_as_an_infix_expression() {
        let entries = vec![
            RpnOperationEntry::ThisParameter(String::new()),
            RpnOperationEntry::Value("2".to_owned()),
            RpnOperationEntry::Operator("*".to_owned()),
            RpnOperationEntry::Value("1".to_owned()),
            RpnOperationEntry::Operator("+".to_owned()),
        ];
        assert_eq!(
            expression_preview(&entries),
            Ok("((this parameter × 2) + 1)".to_owned())
        );
    }

    #[test]
    fn preview_reports_an_operator_with_missing_operands() {
        let entries = vec![RpnOperationEntry::Operator("+".to_owned())];
        assert_eq!(
            expression_preview(&entries),
            Err("Token 1 (+) needs 2 operand(s).".to_owned())
        );
    }
}
