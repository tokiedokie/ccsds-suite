use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, div, prelude::FluentBuilder, px,
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
    field, impl_select_item, optional_value,
    rpn_operation::{RpnOperationEntry, RpnOperationForm},
};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CalibratorKind {
    Polynomial,
    Spline,
    MathOperation,
}
impl_select_item!(CalibratorKind);

#[derive(Clone)]
struct CalibratorRowData {
    first: String,
    second: String,
    third: String,
}

struct CalibratorRow {
    first: Entity<InputState>,
    second: Entity<InputState>,
    third: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

pub(super) struct DefaultCalibratorForm {
    present: bool,
    required: bool,
    kind: CalibratorKind,
    kind_select: Entity<SelectState<Vec<CalibratorKind>>>,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    order: Entity<InputState>,
    extrapolate: Entity<SelectState<Vec<BooleanChoice>>>,
    math_operation: Entity<RpnOperationForm>,
    rows: Vec<Entity<CalibratorRow>>,
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "true")]
    True,
    #[strum(serialize = "false")]
    False,
}
impl_select_item!(BooleanChoice);

impl DefaultCalibratorForm {
    pub(super) fn new(
        calibrator: Option<&xtce::CalibratorType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        Self::new_with_requirement(calibrator, false, window, cx)
    }

    pub(super) fn new_required(
        calibrator: Option<&xtce::CalibratorType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        Self::new_with_requirement(calibrator, true, window, cx)
    }

    fn new_with_requirement(
        calibrator: Option<&xtce::CalibratorType>,
        required: bool,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = CalibratorValues::from_calibrator(calibrator);
        cx.new(move |cx| {
            let kind_select = select(values.kind, window, cx);
            let rows = row_entities(&values.rows, window, cx);
            let math_operation = RpnOperationForm::new(values.math_operation, window, cx);
            let kind_subscription = cx.subscribe_in(
                &kind_select,
                window,
                |this: &mut Self, _, event: &SelectEvent<Vec<CalibratorKind>>, window, cx| {
                    let SelectEvent::Confirm(Some(kind)) = event else {
                        return;
                    };
                    if this.kind == *kind {
                        return;
                    }
                    this.kind = *kind;
                    this.rows = row_entities(&default_rows(*kind), window, cx);
                    cx.notify();
                },
            );
            kind_subscription.detach();
            Self {
                present: required || calibrator.is_some(),
                required,
                kind: values.kind,
                kind_select,
                name: input(&values.name, window, cx),
                short_description: input(&values.short_description, window, cx),
                order: input(&values.order, window, cx),
                extrapolate: select_boolean(values.extrapolate, window, cx),
                math_operation,
                rows,
            }
        })
    }

    pub(super) fn load(
        &mut self,
        calibrator: Option<&xtce::CalibratorType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = CalibratorValues::from_calibrator(calibrator);
        self.present = self.required || calibrator.is_some();
        self.kind = values.kind;
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&values.kind, window, cx);
        });
        for (input, value) in [
            (&self.name, values.name),
            (&self.short_description, values.short_description),
            (&self.order, values.order),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.extrapolate.update(cx, |select, cx| {
            select.set_selected_value(
                &if values.extrapolate {
                    BooleanChoice::True
                } else {
                    BooleanChoice::False
                },
                window,
                cx,
            );
        });
        self.math_operation.update(cx, |form, cx| {
            form.load(values.math_operation, window, cx);
        });
        self.rows = row_entities(&values.rows, window, cx);
        cx.notify();
    }

    pub(super) fn apply_to(&self, calibrator: &mut Option<xtce::CalibratorType>, cx: &App) {
        if !self.required && !self.present {
            *calibrator = None;
            return;
        }
        let calibrator = calibrator.get_or_insert_with(|| xtce::CalibratorType {
            name: None,
            short_description: None,
            content: Vec::new(),
        });
        calibrator.name = optional_value(value(&self.name, cx));
        calibrator.short_description = optional_value(value(&self.short_description, cx));
        let rows = self
            .rows
            .iter()
            .map(|row| {
                let row = row.read(cx);
                CalibratorRowData {
                    first: value(&row.first, cx),
                    second: value(&row.second, cx),
                    third: value(&row.third, cx),
                }
            })
            .collect::<Vec<_>>();
        match self.kind {
            CalibratorKind::Polynomial => {
                let terms = rows
                    .into_iter()
                    .filter_map(|row| {
                        Some(xtce::TermType {
                            coefficient: row.first.trim().parse().ok()?,
                            exponent: row.second.trim().parse().ok()?,
                        })
                    })
                    .collect::<Vec<_>>();
                let terms = if terms.is_empty() {
                    vec![xtce::TermType {
                        coefficient: 1.,
                        exponent: 1,
                    }]
                } else {
                    terms
                };
                if let Some(existing) =
                    calibrator
                        .content
                        .iter_mut()
                        .find_map(|content| match content {
                            xtce::CalibratorTypeContent::PolynomialCalibrator(value) => Some(value),
                            _ => None,
                        })
                {
                    existing.term = terms;
                } else {
                    replace_algorithm(
                        calibrator,
                        xtce::CalibratorTypeContent::PolynomialCalibrator(
                            xtce::PolynomialCalibratorType {
                                name: None,
                                short_description: None,
                                ancillary_data_set: None,
                                term: terms,
                            },
                        ),
                    );
                }
            }
            CalibratorKind::Spline => {
                let mut points = rows
                    .into_iter()
                    .filter_map(|row| {
                        Some(xtce::SplinePointType {
                            raw: row.first.trim().parse().ok()?,
                            calibrated: row.second.trim().parse().ok()?,
                            order: row.third.trim().parse().ok()?,
                        })
                    })
                    .collect::<Vec<_>>();
                while points.len() < 2 {
                    points.push(xtce::SplinePointType {
                        raw: points.len() as f64,
                        calibrated: points.len() as f64,
                        order: 1,
                    });
                }
                let order = value(&self.order, cx).trim().parse().unwrap_or(1);
                let extrapolate = self
                    .extrapolate
                    .read(cx)
                    .selected_value()
                    .is_some_and(|value| *value == BooleanChoice::True);
                if let Some(existing) =
                    calibrator
                        .content
                        .iter_mut()
                        .find_map(|content| match content {
                            xtce::CalibratorTypeContent::SplineCalibrator(value) => Some(value),
                            _ => None,
                        })
                {
                    existing.order = order;
                    existing.extrapolate = extrapolate;
                    existing.spline_point = points;
                } else {
                    replace_algorithm(
                        calibrator,
                        xtce::CalibratorTypeContent::SplineCalibrator(xtce::SplineCalibratorType {
                            name: None,
                            short_description: None,
                            order,
                            extrapolate,
                            ancillary_data_set: None,
                            spline_point: points,
                        }),
                    );
                }
            }
            CalibratorKind::MathOperation => {
                let entries = self
                    .math_operation
                    .read(cx)
                    .entries(cx)
                    .into_iter()
                    .map(math_calibrator_content)
                    .collect::<Vec<_>>();
                if let Some(existing) =
                    calibrator
                        .content
                        .iter_mut()
                        .find_map(|content| match content {
                            xtce::CalibratorTypeContent::MathOperationCalibrator(value) => {
                                Some(value)
                            }
                            _ => None,
                        })
                {
                    let ancillary =
                        std::mem::take(&mut existing.content)
                            .into_iter()
                            .find(|content| {
                                matches!(
                                    content,
                                    xtce::MathOperationCalibratorTypeContent::AncillaryDataSet(_)
                                )
                            });
                    existing.content = ancillary.into_iter().chain(entries).collect();
                } else {
                    replace_algorithm(
                        calibrator,
                        xtce::CalibratorTypeContent::MathOperationCalibrator(
                            xtce::MathOperationCalibratorType {
                                name: None,
                                short_description: None,
                                content: entries,
                            },
                        ),
                    );
                }
            }
        }
    }

    pub(super) fn calibrator(&self, cx: &App) -> xtce::CalibratorType {
        let mut calibrator = None;
        self.apply_to(&mut calibrator, cx);
        calibrator.unwrap_or_else(|| xtce::CalibratorType {
            name: None,
            short_description: None,
            content: Vec::new(),
        })
    }

    fn formula_preview(&self, cx: &App) -> String {
        let rows = self.rows.iter().map(|row| {
            let row = row.read(cx);
            (
                value(&row.first, cx),
                value(&row.second, cx),
                value(&row.third, cx),
            )
        });
        match self.kind {
            CalibratorKind::Polynomial => {
                let terms = rows
                    .filter(|(coefficient, exponent, _)| {
                        !coefficient.trim().is_empty() && !exponent.trim().is_empty()
                    })
                    .map(|(coefficient, exponent, _)| match exponent.trim() {
                        "0" => format!("({})", coefficient.trim()),
                        "1" => format!("({})·x", coefficient.trim()),
                        exponent => format!("({})·x^{}", coefficient.trim(), exponent),
                    })
                    .collect::<Vec<_>>();
                format!("y = {}", terms.join(" + "))
            }
            CalibratorKind::Spline => {
                let points = rows
                    .filter(|(raw, calibrated, _)| {
                        !raw.trim().is_empty() && !calibrated.trim().is_empty()
                    })
                    .map(|(raw, calibrated, order)| {
                        format!(
                            "({}, {}) [order {}]",
                            raw.trim(),
                            calibrated.trim(),
                            order.trim()
                        )
                    })
                    .collect::<Vec<_>>();
                format!("f(x): {}", points.join(" → "))
            }
            CalibratorKind::MathOperation => {
                let operation = self
                    .math_operation
                    .read(cx)
                    .entries(cx)
                    .iter()
                    .map(rpn_entry_preview)
                    .collect::<Vec<_>>()
                    .join("  ");
                format!("RPN: {operation}")
            }
        }
    }
}

impl Render for DefaultCalibratorForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let present = self.present;
        let kind = self.kind;
        let formula = self.formula_preview(cx);
        v_flex()
            .w_full()
            .gap_3()
            .when(!self.required, |form| {
                form.child(
                    h_flex()
                        .justify_between()
                        .child(div().text_sm().font_medium().child("Default calibrator"))
                        .child(if present {
                            Button::new("remove-default-calibrator")
                                .small()
                                .danger()
                                .label("Remove default calibrator")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.present = false;
                                    cx.notify();
                                }))
                        } else {
                            Button::new("add-default-calibrator")
                                .small()
                                .icon(IconName::Plus)
                                .label("Add default calibrator")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.present = true;
                                    cx.notify();
                                }))
                        }),
                )
            })
            .when(present, |form| {
                form.child(
                    v_flex()
                        .gap_4()
                        .child(Select::new(&self.kind_select).w_full())
                        .child(
                            h_flex()
                                .gap_4()
                                .items_start()
                                .child(field("Name", "Optional", &self.name, cx))
                                .child(field(
                                    "Short description",
                                    "Optional",
                                    &self.short_description,
                                    cx,
                                )),
                        )
                        .when(kind == CalibratorKind::Spline, |body| {
                            body.child(
                                h_flex()
                                    .gap_4()
                                    .items_start()
                                    .child(field(
                                        "Default interpolation order",
                                        "0 or greater",
                                        &self.order,
                                        cx,
                                    ))
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .child(div().text_sm().child("Extrapolate"))
                                            .child(Select::new(&self.extrapolate).w_full()),
                                    ),
                            )
                        })
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_xs().font_medium().child("Formula preview"))
                                .child(
                                    div()
                                        .w_full()
                                        .p_3()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .bg(cx.theme().muted.opacity(0.35))
                                        .text_sm()
                                        .child(formula),
                                ),
                        )
                        .when(kind == CalibratorKind::MathOperation, |body| {
                            body.child(self.math_operation.clone())
                        })
                        .when(kind != CalibratorKind::MathOperation, |body| {
                            body.child(render_rows(kind, &self.rows, cx))
                        }),
                )
            })
    }
}

impl Render for CalibratorRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .gap_2()
            .child(div().flex_1().child(Input::new(&self.first)))
            .child(div().flex_1().child(Input::new(&self.second)))
            .child(div().flex_1().child(Input::new(&self.third)))
    }
}

fn render_rows(
    kind: CalibratorKind,
    rows: &[Entity<CalibratorRow>],
    cx: &mut Context<DefaultCalibratorForm>,
) -> impl IntoElement {
    v_flex()
        .w_full()
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
                .child(div().flex_1().child(if kind == CalibratorKind::Polynomial {
                    "Coefficient"
                } else {
                    "Raw value"
                }))
                .child(div().flex_1().child(if kind == CalibratorKind::Polynomial {
                    "Exponent"
                } else {
                    "Calibrated value"
                }))
                .child(div().flex_1().child(if kind == CalibratorKind::Polynomial {
                    ""
                } else {
                    "Order"
                }))
                .child(div().w(px(52.)).child("Actions")),
        )
        .children(rows.iter().enumerate().map(|(index, row)| {
            h_flex()
                .h(px(50.))
                .px_2()
                .gap_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(row.clone())
                .child(
                    div().w(px(52.)).flex_none().child(
                        Button::new(format!("remove-calibrator-row-{index}"))
                            .small()
                            .ghost()
                            .icon(IconName::Minus)
                            .disabled(kind == CalibratorKind::Spline && rows.len() <= 2)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if index < this.rows.len()
                                    && !(this.kind == CalibratorKind::Spline
                                        && this.rows.len() <= 2)
                                {
                                    this.rows.remove(index);
                                    cx.notify();
                                }
                            })),
                    ),
                )
        }))
        .child(
            Button::new("add-calibrator-row")
                .small()
                .icon(IconName::Plus)
                .label(if kind == CalibratorKind::Polynomial {
                    "Add term"
                } else {
                    "Add point"
                })
                .on_click(cx.listener(|this, _, window, cx| {
                    let data = if this.kind == CalibratorKind::Polynomial {
                        CalibratorRowData {
                            first: "0".to_owned(),
                            second: this.rows.len().to_string(),
                            third: String::new(),
                        }
                    } else {
                        CalibratorRowData {
                            first: this.rows.len().to_string(),
                            second: this.rows.len().to_string(),
                            third: "1".to_owned(),
                        }
                    };
                    this.rows.push(row_entity(&data, window, cx));
                    cx.notify();
                })),
        )
}

struct CalibratorValues {
    kind: CalibratorKind,
    name: String,
    short_description: String,
    order: String,
    extrapolate: bool,
    math_operation: Vec<RpnOperationEntry>,
    rows: Vec<CalibratorRowData>,
}

impl CalibratorValues {
    fn from_calibrator(value: Option<&xtce::CalibratorType>) -> Self {
        let mut result = Self {
            kind: CalibratorKind::Polynomial,
            name: value
                .and_then(|value| value.name.clone())
                .unwrap_or_default(),
            short_description: value
                .and_then(|value| value.short_description.clone())
                .unwrap_or_default(),
            order: "1".to_owned(),
            extrapolate: false,
            math_operation: Vec::new(),
            rows: default_rows(CalibratorKind::Polynomial),
        };
        let Some(value) = value else {
            return result;
        };
        if let Some(algorithm) = value
            .content
            .iter()
            .find(|content| !matches!(content, xtce::CalibratorTypeContent::AncillaryDataSet(_)))
        {
            match algorithm {
                xtce::CalibratorTypeContent::PolynomialCalibrator(value) => {
                    result.rows = value
                        .term
                        .iter()
                        .map(|term| CalibratorRowData {
                            first: term.coefficient.to_string(),
                            second: term.exponent.to_string(),
                            third: String::new(),
                        })
                        .collect();
                }
                xtce::CalibratorTypeContent::SplineCalibrator(value) => {
                    result.kind = CalibratorKind::Spline;
                    result.order = value.order.to_string();
                    result.extrapolate = value.extrapolate;
                    result.rows = value
                        .spline_point
                        .iter()
                        .map(|point| CalibratorRowData {
                            first: point.raw.to_string(),
                            second: point.calibrated.to_string(),
                            third: point.order.to_string(),
                        })
                        .collect();
                }
                xtce::CalibratorTypeContent::MathOperationCalibrator(value) => {
                    result.kind = CalibratorKind::MathOperation;
                    result.math_operation = rpn_entries_from_calibrator(&value.content);
                    result.rows = Vec::new();
                }
                xtce::CalibratorTypeContent::AncillaryDataSet(_) => {}
            }
        }
        result
    }
}

fn replace_algorithm(calibrator: &mut xtce::CalibratorType, value: xtce::CalibratorTypeContent) {
    calibrator
        .content
        .retain(|content| matches!(content, xtce::CalibratorTypeContent::AncillaryDataSet(_)));
    calibrator.content.push(value);
}

fn default_rows(kind: CalibratorKind) -> Vec<CalibratorRowData> {
    match kind {
        CalibratorKind::Polynomial => vec![CalibratorRowData {
            first: "1".to_owned(),
            second: "1".to_owned(),
            third: String::new(),
        }],
        CalibratorKind::Spline => vec![
            CalibratorRowData {
                first: "0".to_owned(),
                second: "0".to_owned(),
                third: "1".to_owned(),
            },
            CalibratorRowData {
                first: "1".to_owned(),
                second: "1".to_owned(),
                third: "1".to_owned(),
            },
        ],
        CalibratorKind::MathOperation => Vec::new(),
    }
}

fn rpn_entries_from_calibrator(
    content: &[xtce::MathOperationCalibratorTypeContent],
) -> Vec<RpnOperationEntry> {
    content
        .iter()
        .filter_map(|entry| match entry {
            xtce::MathOperationCalibratorTypeContent::ValueOperand(value) => {
                Some(RpnOperationEntry::Value(value.clone()))
            }
            xtce::MathOperationCalibratorTypeContent::ThisParameterOperand(value) => {
                Some(RpnOperationEntry::ThisParameter(value.clone()))
            }
            xtce::MathOperationCalibratorTypeContent::Operator(value) => {
                Some(RpnOperationEntry::Operator(value.clone()))
            }
            xtce::MathOperationCalibratorTypeContent::ParameterInstanceRefOperand(value) => {
                Some(RpnOperationEntry::ParameterInstance {
                    parameter_ref: value.parameter_ref.clone(),
                    instance: value.instance,
                    use_calibrated_value: value.use_calibrated_value,
                })
            }
            xtce::MathOperationCalibratorTypeContent::AncillaryDataSet(_) => None,
        })
        .collect()
}

fn math_calibrator_content(entry: RpnOperationEntry) -> xtce::MathOperationCalibratorTypeContent {
    match entry {
        RpnOperationEntry::Value(value) => {
            xtce::MathOperationCalibratorTypeContent::ValueOperand(value)
        }
        RpnOperationEntry::ThisParameter(value) => {
            xtce::MathOperationCalibratorTypeContent::ThisParameterOperand(value)
        }
        RpnOperationEntry::Operator(value) => {
            xtce::MathOperationCalibratorTypeContent::Operator(value)
        }
        RpnOperationEntry::ParameterInstance {
            parameter_ref,
            instance,
            use_calibrated_value,
        } => xtce::MathOperationCalibratorTypeContent::ParameterInstanceRefOperand(
            xtce::ParameterInstanceRefType {
                parameter_ref,
                instance,
                use_calibrated_value,
            },
        ),
        RpnOperationEntry::ArgumentInstance { .. } => {
            unreachable!("argument operands are not enabled for calibrators")
        }
    }
}

fn rpn_entry_preview(entry: &RpnOperationEntry) -> String {
    match entry {
        RpnOperationEntry::Value(value) => value.clone(),
        RpnOperationEntry::ThisParameter(value) => format!("this({value})"),
        RpnOperationEntry::Operator(value) => value.clone(),
        RpnOperationEntry::ParameterInstance {
            parameter_ref,
            instance,
            ..
        } => format!("{parameter_ref}[{instance}]"),
        RpnOperationEntry::ArgumentInstance { .. } => {
            unreachable!("argument operands are not enabled for calibrators")
        }
    }
}

fn row_entities(
    rows: &[CalibratorRowData],
    window: &mut Window,
    cx: &mut Context<DefaultCalibratorForm>,
) -> Vec<Entity<CalibratorRow>> {
    rows.iter().map(|row| row_entity(row, window, cx)).collect()
}

fn row_entity(
    row: &CalibratorRowData,
    window: &mut Window,
    cx: &mut Context<DefaultCalibratorForm>,
) -> Entity<CalibratorRow> {
    let first = input(&row.first, window, cx);
    let second = input(&row.second, window, cx);
    let third = input(&row.third, window, cx);
    let subscriptions = [&first, &second, &third]
        .into_iter()
        .map(|input| cx.subscribe(input, |_, _, _: &InputEvent, cx| cx.notify()))
        .collect();
    cx.new(|_| CalibratorRow {
        first,
        second,
        third,
        _subscriptions: subscriptions,
    })
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn select(
    kind: CalibratorKind,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<CalibratorKind>>> {
    let index = CalibratorKind::VARIANTS
        .iter()
        .position(|value| *value == kind)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            CalibratorKind::VARIANTS.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn select_boolean(
    value: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<BooleanChoice>>> {
    let selected = if value {
        BooleanChoice::True
    } else {
        BooleanChoice::False
    };
    cx.new(|cx| {
        SelectState::new(
            BooleanChoice::VARIANTS.to_vec(),
            Some(
                IndexPath::default().row(
                    BooleanChoice::VARIANTS
                        .iter()
                        .position(|value| *value == selected)
                        .unwrap_or_default(),
                ),
            ),
            window,
            cx,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CalibratorKind, CalibratorValues, math_calibrator_content, rpn_entries_from_calibrator,
    };
    use crate::forms::rpn_operation::RpnOperationEntry;

    #[test]
    fn polynomial_calibrator_is_loaded_as_editable_term_rows() {
        let calibrator = xtce::CalibratorType {
            name: Some("temperature".to_owned()),
            short_description: Some("Raw counts to degrees".to_owned()),
            content: vec![xtce::CalibratorTypeContent::PolynomialCalibrator(
                xtce::PolynomialCalibratorType {
                    name: None,
                    short_description: None,
                    ancillary_data_set: None,
                    term: vec![
                        xtce::TermType {
                            coefficient: 1.5,
                            exponent: 0,
                        },
                        xtce::TermType {
                            coefficient: 0.25,
                            exponent: 1,
                        },
                    ],
                },
            )],
        };

        let values = CalibratorValues::from_calibrator(Some(&calibrator));

        assert_eq!(values.kind, CalibratorKind::Polynomial);
        assert_eq!(values.name, "temperature");
        assert_eq!(values.rows.len(), 2);
        assert_eq!(values.rows[1].first, "0.25");
        assert_eq!(values.rows[1].second, "1");
    }

    #[test]
    fn math_operation_calibrator_is_loaded_as_editable_rpn() {
        let entries = vec![
            RpnOperationEntry::ThisParameter("raw".to_owned()),
            RpnOperationEntry::Value("2".to_owned()),
            RpnOperationEntry::Operator("*".to_owned()),
            RpnOperationEntry::ParameterInstance {
                parameter_ref: "P1".to_owned(),
                instance: 1,
                use_calibrated_value: false,
            },
        ];
        let content = entries
            .clone()
            .into_iter()
            .map(math_calibrator_content)
            .collect::<Vec<_>>();
        assert_eq!(rpn_entries_from_calibrator(&content), entries);

        let calibrator = xtce::CalibratorType {
            name: Some("scale".to_owned()),
            short_description: None,
            content: vec![xtce::CalibratorTypeContent::MathOperationCalibrator(
                xtce::MathOperationCalibratorType {
                    name: None,
                    short_description: None,
                    content,
                },
            )],
        };
        let values = CalibratorValues::from_calibrator(Some(&calibrator));
        assert_eq!(values.kind, CalibratorKind::MathOperation);
        assert!(matches!(
            &values.math_operation[2],
            RpnOperationEntry::Operator(value) if value == "*"
        ));
    }
}
