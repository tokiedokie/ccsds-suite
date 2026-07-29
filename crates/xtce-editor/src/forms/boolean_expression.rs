use gpui::{
    App, AppContext, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, IndexPath, Selectable, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ComparisonOperator {
    #[strum(serialize = "==")]
    Equal,
    #[strum(serialize = "!=")]
    NotEqual,
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = "<=")]
    LessOrEqual,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = ">=")]
    GreaterOrEqual,
}
impl_select_item!(ComparisonOperator);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum RightOperandKind {
    #[strum(serialize = "Value")]
    LiteralValue,
    #[strum(serialize = "Parameter")]
    ParameterInstance,
}
impl_select_item!(RightOperandKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogicalOperator {
    And,
    Or,
}

pub(super) struct BooleanExpressionForm {
    root: BooleanNode,
}

enum BooleanNode {
    Condition(Box<ConditionFields>),
    Group {
        operator: LogicalOperator,
        children: Vec<BooleanNode>,
    },
}

struct ConditionFields {
    left_parameter_ref: Entity<InputState>,
    left_instance: Entity<InputState>,
    left_calibrated: Entity<SelectState<Vec<BooleanChoice>>>,
    operator: Entity<SelectState<Vec<ComparisonOperator>>>,
    right_kind: Entity<SelectState<Vec<RightOperandKind>>>,
    right_value: Entity<InputState>,
    right_parameter_ref: Entity<InputState>,
    right_instance: Entity<InputState>,
    right_calibrated: Entity<SelectState<Vec<BooleanChoice>>>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct ConditionValues {
    left_parameter_ref: String,
    left_instance: i64,
    left_calibrated: bool,
    operator: ComparisonOperator,
    right: RightOperand,
}

#[derive(Clone)]
enum RightOperand {
    Value(String),
    Parameter {
        parameter_ref: String,
        instance: i64,
        calibrated: bool,
    },
}

impl BooleanExpressionForm {
    pub(super) fn new(
        expression: Option<&xtce::BooleanExpressionType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let model = expression
            .map(BooleanNodeModel::from_expression)
            .unwrap_or_else(BooleanNodeModel::default_condition);
        cx.new(move |cx| Self {
            root: BooleanNode::from_model(model, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        expression: Option<&xtce::BooleanExpressionType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let model = expression
            .map(BooleanNodeModel::from_expression)
            .unwrap_or_else(BooleanNodeModel::default_condition);
        self.root = BooleanNode::from_model(model, window, cx);
        cx.notify();
    }

    pub(super) fn expression(&self, cx: &App) -> xtce::BooleanExpressionType {
        self.root.to_expression(cx)
    }

    fn wrap_root(
        &mut self,
        operator: LogicalOperator,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous =
            std::mem::replace(&mut self.root, BooleanNode::default_condition(window, cx));
        self.root = BooleanNode::Group {
            operator,
            children: vec![previous, BooleanNode::default_condition(window, cx)],
        };
        cx.notify();
    }

    fn add_condition(&mut self, path: &[usize], window: &mut Window, cx: &mut Context<Self>) {
        if let Some(BooleanNode::Group { children, .. }) = self.root.node_mut(path) {
            children.push(BooleanNode::default_condition(window, cx));
            cx.notify();
        }
    }

    fn add_group(
        &mut self,
        path: &[usize],
        operator: LogicalOperator,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(BooleanNode::Group { children, .. }) = self.root.node_mut(path) {
            children.push(BooleanNode::Group {
                operator,
                children: vec![
                    BooleanNode::default_condition(window, cx),
                    BooleanNode::default_condition(window, cx),
                ],
            });
            cx.notify();
        }
    }

    fn remove_node(&mut self, path: &[usize], cx: &mut Context<Self>) {
        let Some((&index, parent_path)) = path.split_last() else {
            return;
        };
        if let Some(BooleanNode::Group { children, .. }) = self.root.node_mut(parent_path)
            && children.len() > 2
            && index < children.len()
        {
            children.remove(index);
            cx.notify();
        }
    }

    fn set_group_operator(
        &mut self,
        path: &[usize],
        operator: LogicalOperator,
        cx: &mut Context<Self>,
    ) {
        if let Some(BooleanNode::Group {
            operator: current, ..
        }) = self.root.node_mut(path)
        {
            *current = operator;
            cx.notify();
        }
    }

    fn render_node(
        &self,
        node: &BooleanNode,
        path: Vec<usize>,
        parent_count: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Div {
        match node {
            BooleanNode::Condition(condition) => {
                self.render_condition(condition, path, parent_count, cx)
            }
            BooleanNode::Group { operator, children } => {
                self.render_group(*operator, children, path, parent_count, cx)
            }
        }
    }

    fn render_group(
        &self,
        operator: LogicalOperator,
        children: &[BooleanNode],
        path: Vec<usize>,
        parent_count: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Div {
        let remove_path = path.clone();
        let and_path = path.clone();
        let or_path = path.clone();
        let add_condition_path = path.clone();
        let add_group_path = path.clone();
        let nested_operator = match operator {
            LogicalOperator::And => LogicalOperator::Or,
            LogicalOperator::Or => LogicalOperator::And,
        };
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
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new(format!("boolean-group-and-{path:?}"))
                                    .xsmall()
                                    .selected(operator == LogicalOperator::And)
                                    .label("AND")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_group_operator(
                                            &and_path,
                                            LogicalOperator::And,
                                            cx,
                                        );
                                    })),
                            )
                            .child(
                                Button::new(format!("boolean-group-or-{path:?}"))
                                    .xsmall()
                                    .selected(operator == LogicalOperator::Or)
                                    .label("OR")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_group_operator(&or_path, LogicalOperator::Or, cx);
                                    })),
                            ),
                    )
                    .when_some(parent_count, |header, count| {
                        header.child(
                            super::row_remove_button(
                                format!("remove-boolean-group-{path:?}"),
                                "Remove condition group",
                            )
                            .disabled(count <= 2)
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.remove_node(&remove_path, cx);
                                },
                            )),
                        )
                    }),
            )
            .children(children.iter().enumerate().map(|(index, child)| {
                let mut child_path = path.clone();
                child_path.push(index);
                self.render_node(child, child_path, Some(children.len()), cx)
            }))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new(format!("add-boolean-condition-{path:?}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::Plus)
                            .label("Condition")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.add_condition(&add_condition_path, window, cx);
                            })),
                    )
                    .child(
                        Button::new(format!("add-boolean-group-{path:?}"))
                            .xsmall()
                            .ghost()
                            .icon(IconName::Plus)
                            .label(match nested_operator {
                                LogicalOperator::And => "AND group",
                                LogicalOperator::Or => "OR group",
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.add_group(&add_group_path, nested_operator, window, cx);
                            })),
                    ),
            )
    }

    fn render_condition(
        &self,
        condition: &ConditionFields,
        path: Vec<usize>,
        parent_count: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Div {
        let right_kind = selected_value(&condition.right_kind, RightOperandKind::LiteralValue, cx);
        let remove_path = path.clone();
        v_flex()
            .w_full()
            .p_3()
            .gap_3()
            .rounded_md()
            .bg(cx.theme().muted.opacity(0.35))
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_sm().font_medium().child("Condition"))
                    .when_some(parent_count, |header, count| {
                        header.child(
                            super::row_remove_button(
                                format!("remove-boolean-condition-{path:?}"),
                                "Remove condition",
                            )
                                .disabled(count <= 2)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.remove_node(&remove_path, cx);
                                })),
                        )
                    }),
            )
            .child(
                div()
                    .id(format!("boolean-condition-scroll-{path:?}"))
                    .w_full()
                    .overflow_x_scroll()
                    .child(
                        h_flex()
                            .min_w(px(820.))
                            .gap_3()
                            .items_start()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w(px(300.))
                                    .p_3()
                                    .gap_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .child("Left operand"),
                                    )
                                    .child(field(
                                        "Parameter reference",
                                        "Required",
                                        &condition.left_parameter_ref,
                                        cx,
                                    ))
                                    .child(
                                        h_flex()
                                            .gap_3()
                                            .items_start()
                                            .child(
                                                div().w(px(120.)).flex_none().child(field(
                                                    "Instance",
                                                    "Optional",
                                                    &condition.left_instance,
                                                    cx,
                                                )),
                                            )
                                            .child(
                                                super::select_field(
                                                    "Use calibrated value",
                                                    "",
                                                    &condition.left_calibrated,
                                                )
                                                .w(px(150.))
                                                .flex_none(),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(110.))
                                    .flex_none()
                                    .pt_3()
                                    .child(
                                        super::select_field(
                                            "Operator",
                                            "Required",
                                            &condition.operator,
                                        ),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w(px(300.))
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
                                                    .text_xs()
                                                    .font_medium()
                                                    .child("Right operand"),
                                            )
                                            .child(
                                                Select::new(&condition.right_kind).w(px(150.)),
                                            ),
                                    )
                                    .when(
                                        right_kind == RightOperandKind::LiteralValue,
                                        |operand| {
                                            operand
                                                .child(field(
                                                    "Comparison value",
                                                    "Required",
                                                    &condition.right_value,
                                                    cx,
                                                ))
                                                .child(
                                                    div()
                                                        .h(px(58.))
                                                        .text_xs()
                                                        .text_color(
                                                            cx.theme().muted_foreground,
                                                        )
                                                        .child(
                                                            "The value is interpreted using the left parameter's type.",
                                                        ),
                                                )
                                        },
                                    )
                                    .when(
                                        right_kind == RightOperandKind::ParameterInstance,
                                        |operand| {
                                            operand
                                                .child(field(
                                                    "Parameter reference",
                                                    "Required",
                                                    &condition.right_parameter_ref,
                                                    cx,
                                                ))
                                                .child(
                                                    h_flex()
                                                        .gap_3()
                                                        .items_start()
                                                        .child(
                                                            div()
                                                                .w(px(120.))
                                                                .flex_none()
                                                                .child(field(
                                                                    "Instance",
                                                                    "Optional",
                                                                    &condition.right_instance,
                                                                    cx,
                                                                )),
                                                        )
                                                        .child(
                                                            super::select_field(
                                                                "Use calibrated value",
                                                                "",
                                                                &condition.right_calibrated,
                                                            )
                                                            .w(px(150.))
                                                            .flex_none(),
                                                        ),
                                                )
                                        },
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for BooleanExpressionForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Build nested AND/OR groups. Every group contains at least two items."),
            )
            .child(self.render_node(&self.root, Vec::new(), None, cx))
            .when(matches!(self.root, BooleanNode::Condition(_)), |form| {
                form.child(
                    h_flex()
                        .gap_1()
                        .child(
                            Button::new("wrap-boolean-root-and")
                                .small()
                                .ghost()
                                .icon(IconName::Plus)
                                .label("AND condition")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.wrap_root(LogicalOperator::And, window, cx);
                                })),
                        )
                        .child(
                            Button::new("wrap-boolean-root-or")
                                .small()
                                .ghost()
                                .icon(IconName::Plus)
                                .label("OR condition")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.wrap_root(LogicalOperator::Or, window, cx);
                                })),
                        ),
                )
            })
    }
}

impl BooleanNode {
    fn from_model(
        model: BooleanNodeModel,
        window: &mut Window,
        cx: &mut Context<BooleanExpressionForm>,
    ) -> Self {
        match model {
            BooleanNodeModel::Condition(values) => {
                Self::Condition(Box::new(ConditionFields::new(values, window, cx)))
            }
            BooleanNodeModel::Group { operator, children } => Self::Group {
                operator,
                children: children
                    .into_iter()
                    .map(|child| Self::from_model(child, window, cx))
                    .collect(),
            },
        }
    }

    fn default_condition(window: &mut Window, cx: &mut Context<BooleanExpressionForm>) -> Self {
        Self::from_model(BooleanNodeModel::default_condition(), window, cx)
    }

    fn node_mut(&mut self, path: &[usize]) -> Option<&mut Self> {
        let Some((&first, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Self::Group { children, .. } => children.get_mut(first)?.node_mut(rest),
            Self::Condition(_) => None,
        }
    }

    fn to_expression(&self, cx: &App) -> xtce::BooleanExpressionType {
        match self {
            Self::Condition(condition) => {
                xtce::BooleanExpressionType::Condition(condition.to_check(cx))
            }
            Self::Group {
                operator: LogicalOperator::And,
                children,
            } => xtce::BooleanExpressionType::AnDedConditions(xtce::AnDedConditionsType {
                content: and_content(children, cx),
            }),
            Self::Group {
                operator: LogicalOperator::Or,
                children,
            } => xtce::BooleanExpressionType::ORedConditions(xtce::ORedConditionsType {
                content: or_content(children, cx),
            }),
        }
    }
}

impl ConditionFields {
    fn new(
        values: ConditionValues,
        window: &mut Window,
        cx: &mut Context<BooleanExpressionForm>,
    ) -> Self {
        let (right_kind, right_value, right_parameter_ref, right_instance, right_calibrated) =
            match values.right {
                RightOperand::Value(value) => (
                    RightOperandKind::LiteralValue,
                    value,
                    String::new(),
                    0,
                    true,
                ),
                RightOperand::Parameter {
                    parameter_ref,
                    instance,
                    calibrated,
                } => (
                    RightOperandKind::ParameterInstance,
                    String::new(),
                    parameter_ref,
                    instance,
                    calibrated,
                ),
            };
        let right_kind = select(RightOperandKind::VARIANTS, right_kind, window, cx);
        let right_kind_subscription = cx.subscribe(
            &right_kind,
            |_, _, _: &SelectEvent<Vec<RightOperandKind>>, cx| cx.notify(),
        );
        Self {
            left_parameter_ref: input(&values.left_parameter_ref, window, cx),
            left_instance: input(&values.left_instance.to_string(), window, cx),
            left_calibrated: boolean_select(values.left_calibrated, window, cx),
            operator: select(ComparisonOperator::VARIANTS, values.operator, window, cx),
            right_kind,
            right_value: input(&right_value, window, cx),
            right_parameter_ref: input(&right_parameter_ref, window, cx),
            right_instance: input(&right_instance.to_string(), window, cx),
            right_calibrated: boolean_select(right_calibrated, window, cx),
            _subscriptions: vec![right_kind_subscription],
        }
    }

    fn to_check(&self, cx: &App) -> xtce::ComparisonCheckType {
        let mut content = vec![
            xtce::ComparisonCheckTypeContent::ParameterInstanceRef(
                xtce::ParameterInstanceRefType {
                    parameter_ref: value(&self.left_parameter_ref, cx),
                    instance: integer_value(&self.left_instance, cx),
                    use_calibrated_value: boolean_value(&self.left_calibrated, cx),
                },
            ),
            xtce::ComparisonCheckTypeContent::ComparisonOperator(
                selected_value(&self.operator, ComparisonOperator::Equal, cx).to_string(),
            ),
        ];
        match selected_value(&self.right_kind, RightOperandKind::LiteralValue, cx) {
            RightOperandKind::LiteralValue => {
                content.push(xtce::ComparisonCheckTypeContent::Value(value(
                    &self.right_value,
                    cx,
                )));
            }
            RightOperandKind::ParameterInstance => {
                content.push(xtce::ComparisonCheckTypeContent::ParameterInstanceRef(
                    xtce::ParameterInstanceRefType {
                        parameter_ref: value(&self.right_parameter_ref, cx),
                        instance: integer_value(&self.right_instance, cx),
                        use_calibrated_value: boolean_value(&self.right_calibrated, cx),
                    },
                ));
            }
        }
        xtce::ComparisonCheckType { content }
    }
}

enum BooleanNodeModel {
    Condition(ConditionValues),
    Group {
        operator: LogicalOperator,
        children: Vec<BooleanNodeModel>,
    },
}

impl BooleanNodeModel {
    fn default_condition() -> Self {
        Self::Condition(ConditionValues {
            left_parameter_ref: String::new(),
            left_instance: 0,
            left_calibrated: true,
            operator: ComparisonOperator::Equal,
            right: RightOperand::Value(String::new()),
        })
    }

    fn from_expression(expression: &xtce::BooleanExpressionType) -> Self {
        match expression {
            xtce::BooleanExpressionType::Condition(condition) => {
                Self::Condition(ConditionValues::from_check(condition))
            }
            xtce::BooleanExpressionType::AnDedConditions(group) => Self::Group {
                operator: LogicalOperator::And,
                children: group
                    .content
                    .iter()
                    .map(|child| match child {
                        xtce::AnDedConditionsTypeContent::Condition(condition) => {
                            Self::Condition(ConditionValues::from_check(condition))
                        }
                        xtce::AnDedConditionsTypeContent::ORedConditions(group) => {
                            Self::from_or_group(group)
                        }
                    })
                    .collect(),
            },
            xtce::BooleanExpressionType::ORedConditions(group) => Self::from_or_group(group),
        }
    }

    fn from_or_group(group: &xtce::ORedConditionsType) -> Self {
        Self::Group {
            operator: LogicalOperator::Or,
            children: group
                .content
                .iter()
                .map(|child| match child {
                    xtce::ORedConditionsTypeContent::Condition(condition) => {
                        Self::Condition(ConditionValues::from_check(condition))
                    }
                    xtce::ORedConditionsTypeContent::AnDedConditions(group) => Self::Group {
                        operator: LogicalOperator::And,
                        children: group
                            .content
                            .iter()
                            .map(|child| match child {
                                xtce::AnDedConditionsTypeContent::Condition(condition) => {
                                    Self::Condition(ConditionValues::from_check(condition))
                                }
                                xtce::AnDedConditionsTypeContent::ORedConditions(group) => {
                                    Self::from_or_group(group)
                                }
                            })
                            .collect(),
                    },
                })
                .collect(),
        }
    }
}

impl ConditionValues {
    fn from_check(check: &xtce::ComparisonCheckType) -> Self {
        let mut parameters = check.content.iter().filter_map(|content| match content {
            xtce::ComparisonCheckTypeContent::ParameterInstanceRef(parameter) => Some(parameter),
            _ => None,
        });
        let left = parameters.next();
        let right_parameter = parameters.next();
        let right_value = check.content.iter().find_map(|content| match content {
            xtce::ComparisonCheckTypeContent::Value(value) => Some(value.clone()),
            _ => None,
        });
        let operator = check
            .content
            .iter()
            .find_map(|content| match content {
                xtce::ComparisonCheckTypeContent::ComparisonOperator(operator) => {
                    operator.parse().ok()
                }
                _ => None,
            })
            .unwrap_or(ComparisonOperator::Equal);
        Self {
            left_parameter_ref: left
                .map(|parameter| parameter.parameter_ref.clone())
                .unwrap_or_default(),
            left_instance: left.map(|parameter| parameter.instance).unwrap_or_default(),
            left_calibrated: left
                .map(|parameter| parameter.use_calibrated_value)
                .unwrap_or(true),
            operator,
            right: right_parameter.map_or_else(
                || RightOperand::Value(right_value.unwrap_or_default()),
                |parameter| RightOperand::Parameter {
                    parameter_ref: parameter.parameter_ref.clone(),
                    instance: parameter.instance,
                    calibrated: parameter.use_calibrated_value,
                },
            ),
        }
    }
}

fn and_content(children: &[BooleanNode], cx: &App) -> Vec<xtce::AnDedConditionsTypeContent> {
    children
        .iter()
        .flat_map(|child| match child {
            BooleanNode::Condition(condition) => {
                vec![xtce::AnDedConditionsTypeContent::Condition(
                    condition.to_check(cx),
                )]
            }
            BooleanNode::Group {
                operator: LogicalOperator::Or,
                children,
            } => vec![xtce::AnDedConditionsTypeContent::ORedConditions(
                xtce::ORedConditionsType {
                    content: or_content(children, cx),
                },
            )],
            BooleanNode::Group {
                operator: LogicalOperator::And,
                children,
            } => and_content(children, cx),
        })
        .collect()
}

fn or_content(children: &[BooleanNode], cx: &App) -> Vec<xtce::ORedConditionsTypeContent> {
    children
        .iter()
        .flat_map(|child| match child {
            BooleanNode::Condition(condition) => {
                vec![xtce::ORedConditionsTypeContent::Condition(
                    condition.to_check(cx),
                )]
            }
            BooleanNode::Group {
                operator: LogicalOperator::And,
                children,
            } => vec![xtce::ORedConditionsTypeContent::AnDedConditions(
                xtce::AnDedConditionsType {
                    content: and_content(children, cx),
                },
            )],
            BooleanNode::Group {
                operator: LogicalOperator::Or,
                children,
            } => or_content(children, cx),
        })
        .collect()
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn integer_value(input: &Entity<InputState>, cx: &App) -> i64 {
    value(input, cx).trim().parse().unwrap_or_default()
}

fn boolean_select(
    value: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<BooleanChoice>>> {
    select(
        BooleanChoice::VARIANTS,
        if value {
            BooleanChoice::True
        } else {
            BooleanChoice::False
        },
        window,
        cx,
    )
}

fn boolean_value(select: &Entity<SelectState<Vec<BooleanChoice>>>, cx: &App) -> bool {
    selected_value(select, BooleanChoice::True, cx) == BooleanChoice::True
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
    use super::{BooleanNodeModel, LogicalOperator, RightOperand};

    #[test]
    fn nested_boolean_expression_is_loaded_without_flattening() {
        let condition = || xtce::ComparisonCheckType {
            content: vec![
                xtce::ComparisonCheckTypeContent::ParameterInstanceRef(
                    xtce::ParameterInstanceRefType {
                        parameter_ref: "P1".to_owned(),
                        instance: 0,
                        use_calibrated_value: true,
                    },
                ),
                xtce::ComparisonCheckTypeContent::ComparisonOperator("==".to_owned()),
                xtce::ComparisonCheckTypeContent::Value("1".to_owned()),
            ],
        };
        let expression = xtce::BooleanExpressionType::AnDedConditions(xtce::AnDedConditionsType {
            content: vec![
                xtce::AnDedConditionsTypeContent::Condition(condition()),
                xtce::AnDedConditionsTypeContent::ORedConditions(xtce::ORedConditionsType {
                    content: vec![
                        xtce::ORedConditionsTypeContent::Condition(condition()),
                        xtce::ORedConditionsTypeContent::Condition(condition()),
                    ],
                }),
            ],
        });
        let BooleanNodeModel::Group { operator, children } =
            BooleanNodeModel::from_expression(&expression)
        else {
            panic!("expected group");
        };
        assert_eq!(operator, LogicalOperator::And);
        assert!(matches!(
            &children[1],
            BooleanNodeModel::Group {
                operator: LogicalOperator::Or,
                ..
            }
        ));
    }

    #[test]
    fn comparison_check_supports_a_parameter_on_the_right() {
        let check = xtce::ComparisonCheckType {
            content: vec![
                xtce::ComparisonCheckTypeContent::ParameterInstanceRef(
                    xtce::ParameterInstanceRefType {
                        parameter_ref: "LEFT".to_owned(),
                        instance: 0,
                        use_calibrated_value: true,
                    },
                ),
                xtce::ComparisonCheckTypeContent::ComparisonOperator("!=".to_owned()),
                xtce::ComparisonCheckTypeContent::ParameterInstanceRef(
                    xtce::ParameterInstanceRefType {
                        parameter_ref: "RIGHT".to_owned(),
                        instance: 2,
                        use_calibrated_value: false,
                    },
                ),
            ],
        };
        let BooleanNodeModel::Condition(values) =
            BooleanNodeModel::from_expression(&xtce::BooleanExpressionType::Condition(check))
        else {
            panic!("expected condition");
        };
        assert!(matches!(
            values.right,
            RightOperand::Parameter {
                parameter_ref,
                instance: 2,
                calibrated: false,
            } if parameter_ref == "RIGHT"
        ));
    }
}
