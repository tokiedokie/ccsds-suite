use gpui::{
    App, AppContext, Context, Div, Entity, InteractiveElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, IndexPath, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm,
    boolean_expression::BooleanExpressionForm, field, impl_select_item,
    input_algorithm::InputAlgorithmForm, optional_value,
};
use crate::XtceEditor;

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CriteriaKind {
    Comparison,
    ComparisonList,
    BooleanExpression,
    CustomAlgorithm,
}
impl_select_item!(CriteriaKind);

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
enum CalibratedChoice {
    #[strum(serialize = "Calibrated")]
    Calibrated,
    #[strum(serialize = "Raw")]
    Raw,
}
impl_select_item!(CalibratedChoice);

pub(super) struct MessageForm {
    present: bool,
    name_input: Entity<InputState>,
    container_ref_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    criteria: Entity<MessageCriteriaForm>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    _subscriptions: Vec<Subscription>,
}

impl MessageForm {
    pub(super) fn new(
        message: Option<&xtce::MessageType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = MessageValues::from_message(message);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        let criteria =
            MessageCriteriaForm::new(message.map(|message| &message.match_criteria), window, cx);
        cx.new(move |cx| Self {
            present: message.is_some(),
            name_input,
            container_ref_input: input(&values.container_ref, false, window, cx),
            short_description_input: input(&values.short_description, false, window, cx),
            long_description_input: input(&values.long_description, true, window, cx),
            criteria,
            alias_set: AliasSetForm::new(
                message.and_then(|message| message.alias_set.as_ref()),
                window,
                cx,
            ),
            ancillary_data_set: AncillaryDataSetForm::new(
                message.and_then(|message| message.ancillary_data_set.as_ref()),
                window,
                cx,
            ),
            _subscriptions: vec![name_subscription],
        })
    }

    pub(super) fn load(
        &mut self,
        message: Option<&xtce::MessageType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.present = message.is_some();
        let values = MessageValues::from_message(message);
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.container_ref_input, values.container_ref),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.criteria.update(cx, |criteria, cx| {
            criteria.load(message.map(|message| &message.match_criteria), window, cx);
        });
        self.alias_set.load(
            message.and_then(|message| message.alias_set.as_ref()),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            message.and_then(|message| message.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn apply_to(&self, message: &mut xtce::MessageType, cx: &App) {
        if !self.present {
            return;
        }
        message.name = value(&self.name_input, cx);
        message.container_ref.container_ref = value(&self.container_ref_input, cx);
        message.short_description = optional_value(value(&self.short_description_input, cx));
        message.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut message.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut message.ancillary_data_set, cx);
        self.criteria
            .read(cx)
            .apply_to(&mut message.match_criteria, cx);
    }

    fn render_form(&self, cx: &App) -> Div {
        if !self.present {
            return v_flex().child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("The selected Message is not present."),
            );
        }
        v_flex()
            .w_full()
            .gap_5()
            .child(field(
                "Container reference",
                "Required",
                &self.container_ref_input,
                cx,
            ))
            .child(field(
                "Short description",
                "Optional",
                &self.short_description_input,
                cx,
            ))
            .child(self.criteria.clone())
            .child(field(
                "Long description",
                "Optional",
                &self.long_description_input,
                cx,
            ))
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
    }
}

impl Render for MessageForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_form(cx)
    }
}

pub(super) struct MessageCriteriaForm {
    kind_select: Entity<SelectState<Vec<CriteriaKind>>>,
    rows: Vec<Entity<ComparisonRow>>,
    boolean_expression: Entity<BooleanExpressionForm>,
    custom_algorithm: Entity<InputAlgorithmForm>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
pub(super) enum MessageCriteriaRef<'a> {
    Comparison(&'a xtce::ComparisonType),
    ComparisonList(&'a xtce::ComparisonListType),
    BooleanExpression(&'a xtce::BooleanExpressionType),
    CustomAlgorithm(&'a xtce::InputAlgorithmType),
}

impl MessageCriteriaForm {
    pub(super) fn new(
        criteria: Option<&xtce::MatchCriteriaType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = CriteriaValues::from_criteria(criteria);
        Self::new_values(values, window, cx)
    }

    pub(super) fn new_context(
        criteria: Option<&xtce::ContextMatchType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        Self::new_values(CriteriaValues::from_context(criteria), window, cx)
    }

    pub(super) fn new_ref(
        criteria: Option<MessageCriteriaRef<'_>>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        Self::new_values(CriteriaValues::from_ref(criteria), window, cx)
    }

    fn new_values(
        values: CriteriaValues<'_>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let boolean_expression = BooleanExpressionForm::new(values.boolean_expression, window, cx);
        let custom_algorithm = InputAlgorithmForm::new(values.custom_algorithm, window, cx);
        cx.new(move |cx| {
            let kind_select = select(CriteriaKind::VARIANTS, values.kind, window, cx);
            let kind_subscription = cx.subscribe(
                &kind_select,
                |_, _, _: &SelectEvent<Vec<CriteriaKind>>, cx| cx.notify(),
            );
            Self {
                kind_select,
                rows: comparison_rows(&values.comparisons, window, cx),
                boolean_expression,
                custom_algorithm,
                _subscriptions: vec![kind_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        criteria: Option<&xtce::MatchCriteriaType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = CriteriaValues::from_criteria(criteria);
        sync_select(&self.kind_select, values.kind, window, cx);
        self.rows = comparison_rows(&values.comparisons, window, cx);
        self.boolean_expression.update(cx, |form, cx| {
            form.load(values.boolean_expression, window, cx);
        });
        self.custom_algorithm.update(cx, |form, cx| {
            form.load(values.custom_algorithm, window, cx);
        });
        cx.notify();
    }

    pub(super) fn apply_to(&self, criteria: &mut xtce::MatchCriteriaType, cx: &App) {
        *criteria = match selected_value(&self.kind_select, CriteriaKind::Comparison, cx) {
            CriteriaKind::BooleanExpression => xtce::MatchCriteriaType::BooleanExpression(
                self.boolean_expression.read(cx).expression(cx),
            ),
            CriteriaKind::Comparison => {
                xtce::MatchCriteriaType::Comparison(self.comparisons(cx).remove(0))
            }
            CriteriaKind::ComparisonList => {
                xtce::MatchCriteriaType::ComparisonList(xtce::ComparisonListType {
                    comparison: self.comparisons(cx),
                })
            }
            CriteriaKind::CustomAlgorithm => xtce::MatchCriteriaType::CustomAlgorithm(
                self.custom_algorithm.read(cx).algorithm(cx),
            ),
        };
    }

    pub(super) fn context_match(&self, cx: &App) -> xtce::ContextMatchType {
        match selected_value(&self.kind_select, CriteriaKind::Comparison, cx) {
            CriteriaKind::BooleanExpression => xtce::ContextMatchType::BooleanExpression(
                self.boolean_expression.read(cx).expression(cx),
            ),
            CriteriaKind::Comparison => {
                xtce::ContextMatchType::Comparison(self.comparisons(cx).remove(0))
            }
            CriteriaKind::ComparisonList => {
                xtce::ContextMatchType::ComparisonList(xtce::ComparisonListType {
                    comparison: self.comparisons(cx),
                })
            }
            CriteriaKind::CustomAlgorithm => xtce::ContextMatchType::CustomAlgorithm(
                self.custom_algorithm.read(cx).algorithm(cx),
            ),
        }
    }

    fn comparisons(&self, cx: &App) -> Vec<xtce::ComparisonType> {
        let mut comparisons = self
            .rows
            .iter()
            .map(|row| row.read(cx).to_comparison(cx))
            .collect::<Vec<_>>();
        if comparisons.is_empty() {
            comparisons.push(default_comparison());
        }
        comparisons
    }
}

impl Render for MessageCriteriaForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let kind = selected_value(&self.kind_select, CriteriaKind::Comparison, cx);
        let is_list = kind == CriteriaKind::ComparisonList;
        let count = self.rows.len();
        v_flex()
            .w_full()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_medium()
                    .child("Match criteria"),
            )
            .child(select_field("Criteria type", &self.kind_select))
            .when(kind == CriteriaKind::BooleanExpression, |form| {
                form.child(self.boolean_expression.clone())
            })
            .when(kind == CriteriaKind::CustomAlgorithm, |form| {
                form.child(self.custom_algorithm.clone())
            })
            .when(
                matches!(
                    kind,
                    CriteriaKind::Comparison | CriteriaKind::ComparisonList
                ),
                |form| {
                    form.child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if is_list {
                                        "Every comparison in the list must match."
                                    } else {
                                        "A single comparison must match."
                                    }),
                            )
                            .when(is_list, |header| {
                                header.child(
                                    Button::new("add-message-comparison")
                                        .small()
                                        .icon(IconName::Plus)
                                        .label("Add comparison")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.rows.push(comparison_row(
                                                &default_comparison(),
                                                window,
                                                cx,
                                            ));
                                            cx.notify();
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("message-comparison-table-scroll")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                v_flex()
                                    .min_w(px(820.))
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
                                            .child(div().flex_1().child("Parameter reference"))
                                            .child(div().w(px(100.)).child("Operator"))
                                            .child(div().flex_1().child("Value"))
                                            .child(div().w(px(85.)).child("Instance"))
                                            .child(div().w(px(120.)).child("Value type"))
                                            .when(is_list, |header| {
                                                header.child(div().w(px(52.)).child("Actions"))
                                            }),
                                    )
                                    .children(
                                        self.rows
                                            .iter()
                                            .take(if is_list { count } else { 1 })
                                            .enumerate()
                                            .map(|(index, row)| {
                                                h_flex()
                                                    .h(px(52.))
                                                    .px_2()
                                                    .gap_2()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border)
                                                    .child(row.clone())
                                                    .when(is_list, |line| {
                                                        line.child(
                                                            div()
                                                                .w(px(52.))
                                                                .flex_none()
                                                                .child(
                                                                    Button::new(format!(
                                                                        "remove-message-comparison-{index}"
                                                                    ))
                                                                    .small()
                                                                    .ghost()
                                                                    .icon(IconName::Minus)
                                                                    .tooltip("Remove comparison")
                                                                    .disabled(count == 1)
                                                                    .on_click(cx.listener(
                                                                        move |this, _, _, cx| {
                                                                            if this.rows.len() > 1
                                                                                && index
                                                                                    < this.rows.len()
                                                                            {
                                                                                this.rows
                                                                                    .remove(index);
                                                                                cx.notify();
                                                                            }
                                                                        },
                                                                    )),
                                                                ),
                                                        )
                                                    })
                                            }),
                                    ),
                            ),
                    )
                },
            )
    }
}

struct ComparisonRow {
    parameter_ref: Entity<InputState>,
    operator: Entity<SelectState<Vec<ComparisonOperator>>>,
    comparison_value: Entity<InputState>,
    instance: Entity<InputState>,
    calibrated: Entity<SelectState<Vec<CalibratedChoice>>>,
}

impl ComparisonRow {
    fn to_comparison(&self, cx: &App) -> xtce::ComparisonType {
        xtce::ComparisonType {
            parameter_ref: value(&self.parameter_ref, cx),
            instance: value(&self.instance, cx)
                .trim()
                .parse()
                .unwrap_or_else(|_| xtce::ComparisonType::default_instance()),
            use_calibrated_value: selected_value(
                &self.calibrated,
                CalibratedChoice::Calibrated,
                cx,
            ) == CalibratedChoice::Calibrated,
            comparison_operator: selected_value(&self.operator, ComparisonOperator::Equal, cx)
                .to_string(),
            value: value(&self.comparison_value, cx),
        }
    }
}

impl Render for ComparisonRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(div().flex_1().child(Input::new(&self.parameter_ref)))
            .child(
                div()
                    .w(px(100.))
                    .flex_none()
                    .child(Select::new(&self.operator).w_full()),
            )
            .child(div().flex_1().child(Input::new(&self.comparison_value)))
            .child(
                div()
                    .w(px(85.))
                    .flex_none()
                    .child(Input::new(&self.instance)),
            )
            .child(
                div()
                    .w(px(120.))
                    .flex_none()
                    .child(Select::new(&self.calibrated).w_full()),
            )
    }
}

struct MessageValues {
    name: String,
    container_ref: String,
    short_description: String,
    long_description: String,
}

impl MessageValues {
    fn from_message(message: Option<&xtce::MessageType>) -> Self {
        Self {
            name: message
                .map(|message| message.name.clone())
                .unwrap_or_default(),
            container_ref: message
                .map(|message| message.container_ref.container_ref.clone())
                .unwrap_or_default(),
            short_description: message
                .and_then(|message| message.short_description.clone())
                .unwrap_or_default(),
            long_description: message
                .and_then(|message| message.long_description.clone())
                .unwrap_or_default(),
        }
    }
}

struct CriteriaValues<'a> {
    kind: CriteriaKind,
    comparisons: Vec<&'a xtce::ComparisonType>,
    boolean_expression: Option<&'a xtce::BooleanExpressionType>,
    custom_algorithm: Option<&'a xtce::InputAlgorithmType>,
}

impl<'a> CriteriaValues<'a> {
    fn from_ref(criteria: Option<MessageCriteriaRef<'a>>) -> Self {
        match criteria {
            Some(MessageCriteriaRef::Comparison(comparison)) => Self {
                kind: CriteriaKind::Comparison,
                comparisons: vec![comparison],
                boolean_expression: None,
                custom_algorithm: None,
            },
            Some(MessageCriteriaRef::ComparisonList(list)) => Self {
                kind: CriteriaKind::ComparisonList,
                comparisons: list.comparison.iter().collect(),
                boolean_expression: None,
                custom_algorithm: None,
            },
            Some(MessageCriteriaRef::BooleanExpression(expression)) => Self {
                kind: CriteriaKind::BooleanExpression,
                comparisons: Vec::new(),
                boolean_expression: Some(expression),
                custom_algorithm: None,
            },
            Some(MessageCriteriaRef::CustomAlgorithm(algorithm)) => Self {
                kind: CriteriaKind::CustomAlgorithm,
                comparisons: Vec::new(),
                boolean_expression: None,
                custom_algorithm: Some(algorithm),
            },
            None => Self {
                kind: CriteriaKind::Comparison,
                comparisons: Vec::new(),
                boolean_expression: None,
                custom_algorithm: None,
            },
        }
    }

    fn from_criteria(criteria: Option<&'a xtce::MatchCriteriaType>) -> Self {
        match criteria {
            Some(xtce::MatchCriteriaType::Comparison(comparison)) => Self {
                kind: CriteriaKind::Comparison,
                comparisons: vec![comparison],
                boolean_expression: None,
                custom_algorithm: None,
            },
            Some(xtce::MatchCriteriaType::ComparisonList(list)) => Self {
                kind: CriteriaKind::ComparisonList,
                comparisons: list.comparison.iter().collect(),
                boolean_expression: None,
                custom_algorithm: None,
            },
            Some(xtce::MatchCriteriaType::BooleanExpression(expression)) => Self {
                kind: CriteriaKind::BooleanExpression,
                comparisons: Vec::new(),
                boolean_expression: Some(expression),
                custom_algorithm: None,
            },
            Some(xtce::MatchCriteriaType::CustomAlgorithm(algorithm)) => Self {
                kind: CriteriaKind::CustomAlgorithm,
                comparisons: Vec::new(),
                boolean_expression: None,
                custom_algorithm: Some(algorithm),
            },
            None => Self {
                kind: CriteriaKind::Comparison,
                comparisons: Vec::new(),
                boolean_expression: None,
                custom_algorithm: None,
            },
        }
    }

    fn from_context(criteria: Option<&'a xtce::ContextMatchType>) -> Self {
        match criteria {
            Some(xtce::ContextMatchType::Comparison(comparison)) => Self {
                kind: CriteriaKind::Comparison,
                comparisons: vec![comparison],
                boolean_expression: None,
                custom_algorithm: None,
            },
            Some(xtce::ContextMatchType::ComparisonList(list)) => Self {
                kind: CriteriaKind::ComparisonList,
                comparisons: list.comparison.iter().collect(),
                boolean_expression: None,
                custom_algorithm: None,
            },
            Some(xtce::ContextMatchType::BooleanExpression(expression)) => Self {
                kind: CriteriaKind::BooleanExpression,
                comparisons: Vec::new(),
                boolean_expression: Some(expression),
                custom_algorithm: None,
            },
            Some(xtce::ContextMatchType::CustomAlgorithm(algorithm)) => Self {
                kind: CriteriaKind::CustomAlgorithm,
                comparisons: Vec::new(),
                boolean_expression: None,
                custom_algorithm: Some(algorithm),
            },
            None => Self {
                kind: CriteriaKind::Comparison,
                comparisons: Vec::new(),
                boolean_expression: None,
                custom_algorithm: None,
            },
        }
    }
}

fn comparison_rows(
    comparisons: &[&xtce::ComparisonType],
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<ComparisonRow>> {
    if comparisons.is_empty() {
        vec![comparison_row(&default_comparison(), window, cx)]
    } else {
        comparisons
            .iter()
            .map(|comparison| comparison_row(comparison, window, cx))
            .collect()
    }
}

fn comparison_row(
    comparison: &xtce::ComparisonType,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ComparisonRow> {
    let operator = parse_operator(&comparison.comparison_operator);
    let calibrated = if comparison.use_calibrated_value {
        CalibratedChoice::Calibrated
    } else {
        CalibratedChoice::Raw
    };
    cx.new(|cx| ComparisonRow {
        parameter_ref: input(&comparison.parameter_ref, false, window, cx),
        operator: select(ComparisonOperator::VARIANTS, operator, window, cx),
        comparison_value: input(&comparison.value, false, window, cx),
        instance: input(&comparison.instance.to_string(), false, window, cx),
        calibrated: select(CalibratedChoice::VARIANTS, calibrated, window, cx),
    })
}

fn default_comparison() -> xtce::ComparisonType {
    xtce::ComparisonType {
        parameter_ref: String::new(),
        instance: xtce::ComparisonType::default_instance(),
        use_calibrated_value: xtce::ComparisonType::default_use_calibrated_value(),
        comparison_operator: xtce::ComparisonType::default_comparison_operator(),
        value: String::new(),
    }
}

fn parse_operator(value: &str) -> ComparisonOperator {
    match value {
        "!=" => ComparisonOperator::NotEqual,
        "<" => ComparisonOperator::Less,
        "<=" => ComparisonOperator::LessOrEqual,
        ">" => ComparisonOperator::Greater,
        ">=" => ComparisonOperator::GreaterOrEqual,
        _ => ComparisonOperator::Equal,
    }
}

fn input(
    value: &str,
    multiline: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let input = InputState::new(window, cx).default_value(value.to_owned());
        if multiline {
            input.auto_grow(4, 20)
        } else {
            input
        }
    })
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

fn sync_select<T>(
    select: &Entity<SelectState<Vec<T>>>,
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select.update(cx, |select, cx| {
        select.set_selected_value(&selected, window, cx);
    });
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

fn select_field<T>(label: &'static str, select: &Entity<SelectState<Vec<T>>>) -> Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    super::select_field(label, "Required", select)
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{CriteriaKind, CriteriaValues, default_comparison, parse_operator};

    #[test]
    fn new_comparison_has_an_empty_parameter_reference() {
        assert!(default_comparison().parameter_ref.is_empty());
    }

    #[test]
    fn comparison_criteria_loads_as_a_single_structured_row() {
        let criteria = xtce::MatchCriteriaType::Comparison(default_comparison());
        let values = CriteriaValues::from_criteria(Some(&criteria));

        assert_eq!(values.kind, CriteriaKind::Comparison);
        assert_eq!(values.comparisons.len(), 1);
    }

    #[test]
    fn comparison_operator_choices_cover_the_xtce_operators() {
        assert_eq!(parse_operator("!=").to_string(), "!=");
        assert_eq!(parse_operator("<=").to_string(), "<=");
        assert_eq!(parse_operator(">=").to_string(), ">=");
    }

    #[test]
    fn boolean_expression_criteria_loads_as_an_editable_form() {
        let criteria = xtce::MatchCriteriaType::BooleanExpression(
            xtce::BooleanExpressionType::Condition(xtce::ComparisonCheckType {
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
            }),
        );
        let values = CriteriaValues::from_criteria(Some(&criteria));

        assert_eq!(values.kind, CriteriaKind::BooleanExpression);
        assert!(values.boolean_expression.is_some());
    }

    #[test]
    fn custom_algorithm_criteria_loads_as_an_editable_form() {
        let criteria = xtce::MatchCriteriaType::CustomAlgorithm(xtce::InputAlgorithmType {
            short_description: None,
            name: "packetFilter".to_owned(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            algorithm_text: None,
            external_algorithm_set: None,
            input_set: None,
        });
        let values = CriteriaValues::from_criteria(Some(&criteria));

        assert_eq!(values.kind, CriteriaKind::CustomAlgorithm);
        assert_eq!(
            values
                .custom_algorithm
                .map(|algorithm| algorithm.name.as_str()),
            Some("packetFilter")
        );
    }
}
