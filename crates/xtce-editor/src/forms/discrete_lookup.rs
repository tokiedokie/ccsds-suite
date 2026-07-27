use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, IndexPath, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    boolean_expression::BooleanExpressionForm, field, impl_select_item,
    input_algorithm::InputAlgorithmForm,
};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CriteriaKind {
    #[default]
    Comparison,
    ComparisonList,
    BooleanExpression,
    CustomAlgorithm,
}
impl_select_item!(CriteriaKind);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ComparisonOperator {
    #[default]
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

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CalibratedChoice {
    #[default]
    Calibrated,
    Raw,
}
impl_select_item!(CalibratedChoice);

pub(super) struct DiscreteLookupListForm {
    default_value: Entity<InputState>,
    rows: Vec<Entity<LookupRow>>,
}

struct LookupRow {
    result_value: Entity<InputState>,
    kind: CriteriaKind,
    kind_select: Entity<SelectState<Vec<CriteriaKind>>>,
    comparisons: Entity<ComparisonList>,
    boolean_expression: Entity<BooleanExpressionForm>,
    custom_algorithm: Entity<InputAlgorithmForm>,
}

struct ComparisonList {
    rows: Vec<Entity<ComparisonRow>>,
}

struct ComparisonRow {
    parameter_ref: Entity<InputState>,
    instance: Entity<InputState>,
    calibrated: Entity<SelectState<Vec<CalibratedChoice>>>,
    operator: Entity<SelectState<Vec<ComparisonOperator>>>,
    value: Entity<InputState>,
}

impl DiscreteLookupListForm {
    pub(super) fn new(
        value: Option<&xtce::DiscreteLookupListType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let default_value = value.map_or(0, |value| value.default_value);
        let rows = value
            .into_iter()
            .flat_map(|value| &value.discrete_lookup)
            .map(|value| new_lookup_row(value, window, cx))
            .collect();
        cx.new(|cx| Self {
            default_value: input(&default_value.to_string(), window, cx),
            rows,
        })
    }

    pub(super) fn load(
        &mut self,
        value: Option<&xtce::DiscreteLookupListType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let default_value = value.map_or(0, |value| value.default_value);
        self.default_value.update(cx, |input, cx| {
            input.set_value(default_value.to_string(), window, cx)
        });
        self.rows = value
            .into_iter()
            .flat_map(|value| &value.discrete_lookup)
            .map(|value| new_lookup_row(value, window, cx))
            .collect();
        cx.notify();
    }

    pub(super) fn value(&self, cx: &App) -> xtce::DiscreteLookupListType {
        xtce::DiscreteLookupListType {
            default_value: integer(&self.default_value, cx),
            discrete_lookup: self.rows.iter().map(|row| row.read(cx).value(cx)).collect(),
        }
    }
}

impl Render for DiscreteLookupListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
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
                                    .child(format!("Lookup {}", index + 1)),
                            )
                            .child(
                                Button::new(format!("remove-discrete-lookup-{index}"))
                                    .ghost()
                                    .small()
                                    .icon(IconName::Minus)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index < this.rows.len() {
                                            this.rows.remove(index);
                                            cx.notify();
                                        }
                                    })),
                            ),
                    )
                    .child(row.clone())
            })
            .collect::<Vec<_>>();
        v_flex()
            .w_full()
            .gap_3()
            .child(field(
                "Default size",
                "Used when no condition matches",
                &self.default_value,
                cx,
            ))
            .children(rows)
            .child(
                Button::new("add-discrete-lookup")
                    .small()
                    .icon(IconName::Plus)
                    .label("Add lookup")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.rows.push(new_lookup_row_default(window, cx));
                        cx.notify();
                    })),
            )
    }
}

impl LookupRow {
    fn value(&self, cx: &App) -> xtce::DiscreteLookupType {
        let comparisons = self.comparisons.read(cx).values(cx);
        let content = match self.kind {
            CriteriaKind::Comparison => xtce::DiscreteLookupTypeContent::Comparison(
                comparisons
                    .into_iter()
                    .next()
                    .unwrap_or_else(default_comparison),
            ),
            CriteriaKind::ComparisonList => {
                xtce::DiscreteLookupTypeContent::ComparisonList(xtce::ComparisonListType {
                    comparison: comparisons,
                })
            }
            CriteriaKind::BooleanExpression => xtce::DiscreteLookupTypeContent::BooleanExpression(
                self.boolean_expression.read(cx).expression(cx),
            ),
            CriteriaKind::CustomAlgorithm => xtce::DiscreteLookupTypeContent::CustomAlgorithm(
                self.custom_algorithm.read(cx).algorithm(cx),
            ),
        };
        xtce::DiscreteLookupType {
            value: integer(&self.result_value, cx),
            content,
        }
    }
}

impl Render for LookupRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(field("Result size", "Required", &self.result_value, cx))
                    .child(select_field("Criteria", &self.kind_select)),
            )
            .when(
                matches!(
                    self.kind,
                    CriteriaKind::Comparison | CriteriaKind::ComparisonList
                ),
                |form| form.child(self.comparisons.clone()),
            )
            .when(self.kind == CriteriaKind::BooleanExpression, |form| {
                form.child(self.boolean_expression.clone())
            })
            .when(self.kind == CriteriaKind::CustomAlgorithm, |form| {
                form.child(self.custom_algorithm.clone())
            })
    }
}

impl ComparisonList {
    fn values(&self, cx: &App) -> Vec<xtce::ComparisonType> {
        self.rows.iter().map(|row| row.read(cx).value(cx)).collect()
    }
}

impl Render for ComparisonList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                h_flex()
                    .w_full()
                    .gap_2()
                    .items_end()
                    .child(row.clone())
                    .child(
                        Button::new(format!("remove-lookup-comparison-{index}"))
                            .ghost()
                            .small()
                            .icon(IconName::Minus)
                            .disabled(self.rows.len() <= 1)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if this.rows.len() > 1 && index < this.rows.len() {
                                    this.rows.remove(index);
                                    cx.notify();
                                }
                            })),
                    )
            })
            .collect::<Vec<_>>();
        v_flex().w_full().gap_2().children(rows).child(
            Button::new("add-lookup-comparison")
                .small()
                .icon(IconName::Plus)
                .label("Add comparison")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.rows
                        .push(new_comparison_row(&default_comparison(), window, cx));
                    cx.notify();
                })),
        )
    }
}

impl ComparisonRow {
    fn value(&self, cx: &App) -> xtce::ComparisonType {
        xtce::ComparisonType {
            parameter_ref: input_value(&self.parameter_ref, cx),
            instance: integer(&self.instance, cx),
            use_calibrated_value: selected(&self.calibrated, CalibratedChoice::Calibrated, cx)
                == CalibratedChoice::Calibrated,
            comparison_operator: selected(&self.operator, ComparisonOperator::Equal, cx)
                .to_string(),
            value: input_value(&self.value, cx),
        }
    }
}

impl Render for ComparisonRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .gap_2()
            .items_start()
            .child(field(
                "Parameter reference",
                "Required",
                &self.parameter_ref,
                cx,
            ))
            .child(field("Instance", "Defaults to 0", &self.instance, cx))
            .child(select_field("Value form", &self.calibrated))
            .child(select_field("Operator", &self.operator))
            .child(field("Value", "Required", &self.value, cx))
    }
}

fn new_lookup_row(
    value: &xtce::DiscreteLookupType,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<LookupRow> {
    let (kind, comparisons, expression, algorithm) = match &value.content {
        xtce::DiscreteLookupTypeContent::Comparison(comparison) => {
            (CriteriaKind::Comparison, vec![comparison], None, None)
        }
        xtce::DiscreteLookupTypeContent::ComparisonList(list) => (
            CriteriaKind::ComparisonList,
            list.comparison.iter().collect(),
            None,
            None,
        ),
        xtce::DiscreteLookupTypeContent::BooleanExpression(expression) => (
            CriteriaKind::BooleanExpression,
            Vec::new(),
            Some(expression),
            None,
        ),
        xtce::DiscreteLookupTypeContent::CustomAlgorithm(algorithm) => (
            CriteriaKind::CustomAlgorithm,
            Vec::new(),
            None,
            Some(algorithm),
        ),
    };
    new_lookup_row_parts(
        value.value,
        kind,
        comparisons,
        expression,
        algorithm,
        window,
        cx,
    )
}

fn new_lookup_row_default(window: &mut Window, cx: &mut impl AppContext) -> Entity<LookupRow> {
    let comparison = default_comparison();
    new_lookup_row_parts(
        0,
        CriteriaKind::Comparison,
        vec![&comparison],
        None,
        None,
        window,
        cx,
    )
}

fn new_lookup_row_parts(
    result_value: i64,
    kind: CriteriaKind,
    comparisons: Vec<&xtce::ComparisonType>,
    expression: Option<&xtce::BooleanExpressionType>,
    algorithm: Option<&xtce::InputAlgorithmType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<LookupRow> {
    let kind_select = select(CriteriaKind::VARIANTS, kind, window, cx);
    let comparisons = cx.new(|cx| ComparisonList {
        rows: if comparisons.is_empty() {
            vec![new_comparison_row(&default_comparison(), window, cx)]
        } else {
            comparisons
                .into_iter()
                .map(|value| new_comparison_row(value, window, cx))
                .collect()
        },
    });
    let boolean_expression = BooleanExpressionForm::new(expression, window, cx);
    let custom_algorithm = InputAlgorithmForm::new(algorithm, window, cx);
    cx.new(move |cx| {
        let subscription = cx.subscribe_in(
            &kind_select,
            window,
            |this: &mut LookupRow, _, event: &SelectEvent<Vec<CriteriaKind>>, _, cx| {
                if let SelectEvent::Confirm(Some(kind)) = event {
                    this.kind = *kind;
                    cx.notify();
                }
            },
        );
        subscription.detach();
        LookupRow {
            result_value: input(&result_value.to_string(), window, cx),
            kind,
            kind_select,
            comparisons,
            boolean_expression,
            custom_algorithm,
        }
    })
}

fn new_comparison_row(
    value: &xtce::ComparisonType,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ComparisonRow> {
    cx.new(|cx| ComparisonRow {
        parameter_ref: input(&value.parameter_ref, window, cx),
        instance: input(&value.instance.to_string(), window, cx),
        calibrated: select(
            CalibratedChoice::VARIANTS,
            if value.use_calibrated_value {
                CalibratedChoice::Calibrated
            } else {
                CalibratedChoice::Raw
            },
            window,
            cx,
        ),
        operator: select(
            ComparisonOperator::VARIANTS,
            ComparisonOperator::from_text(&value.comparison_operator),
            window,
            cx,
        ),
        value: input(&value.value, window, cx),
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

impl ComparisonOperator {
    fn from_text(value: &str) -> Self {
        match value {
            "!=" => Self::NotEqual,
            "<" => Self::Less,
            "<=" => Self::LessOrEqual,
            ">" => Self::Greater,
            ">=" => Self::GreaterOrEqual,
            _ => Self::Equal,
        }
    }
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn input_value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn integer(input: &Entity<InputState>, cx: &App) -> i64 {
    input_value(input, cx).trim().parse().unwrap_or_default()
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
    let index = choices
        .iter()
        .position(|choice| *choice == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            choices.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn selected<T>(select: &Entity<SelectState<Vec<T>>>, fallback: T, cx: &App) -> T
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(fallback)
}

fn select_field<T>(label: &'static str, select: &Entity<SelectState<Vec<T>>>) -> gpui::Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem + 'static,
{
    v_flex().w_full().child(
        gpui_component::form::field()
            .label(label)
            .child(Select::new(select).w_full()),
    )
}

#[cfg(test)]
mod tests {
    use super::ComparisonOperator;

    #[test]
    fn operator_text_covers_all_xtce_comparison_operators() {
        assert_eq!(
            ComparisonOperator::from_text("!="),
            ComparisonOperator::NotEqual
        );
        assert_eq!(
            ComparisonOperator::from_text("<="),
            ComparisonOperator::LessOrEqual
        );
        assert_eq!(
            ComparisonOperator::from_text(">="),
            ComparisonOperator::GreaterOrEqual
        );
    }
}
