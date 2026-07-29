use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, IndexPath, StyledExt, h_flex, input::InputState, select::SelectState, v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValidRangeKind {
    Integer,
    Float,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BoundaryChoice {
    Inclusive,
    Exclusive,
}
impl_select_item!(BoundaryChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ValueFormChoice {
    #[strum(serialize = "Calibrated values")]
    Calibrated,
    #[strum(serialize = "Raw values")]
    Raw,
}
impl_select_item!(ValueFormChoice);

pub(super) enum ValidRangeValue {
    Integer(xtce::IntegerDataTypeValidRangeElementType),
    Float(xtce::FloatDataTypeValidRangeElementType),
}

pub(super) struct ValidRangeForm {
    kind: ValidRangeKind,
    active: bool,
    minimum: Entity<InputState>,
    maximum: Entity<InputState>,
    minimum_boundary: Entity<SelectState<Vec<BoundaryChoice>>>,
    maximum_boundary: Entity<SelectState<Vec<BoundaryChoice>>>,
    value_form: Entity<SelectState<Vec<ValueFormChoice>>>,
}

impl ValidRangeForm {
    pub(super) fn new(
        kind: ValidRangeKind,
        integer: Option<&xtce::IntegerDataTypeValidRangeElementType>,
        float: Option<&xtce::FloatDataTypeValidRangeElementType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = ValidRangeValues::from_range(kind, integer, float);
        let minimum = input(&values.minimum, window, cx);
        let maximum = input(&values.maximum, window, cx);
        let minimum_boundary = select(
            BoundaryChoice::VARIANTS,
            values.minimum_boundary,
            window,
            cx,
        );
        let maximum_boundary = select(
            BoundaryChoice::VARIANTS,
            values.maximum_boundary,
            window,
            cx,
        );
        let value_form = select(ValueFormChoice::VARIANTS, values.value_form, window, cx);
        cx.new(move |_| Self {
            kind,
            active: integer.is_some() || float.is_some(),
            minimum,
            maximum,
            minimum_boundary,
            maximum_boundary,
            value_form,
        })
    }

    pub(super) fn load(
        &mut self,
        kind: ValidRangeKind,
        integer: Option<&xtce::IntegerDataTypeValidRangeElementType>,
        float: Option<&xtce::FloatDataTypeValidRangeElementType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ValidRangeValues::from_range(kind, integer, float);
        self.kind = kind;
        self.active = integer.is_some() || float.is_some();
        for (input, value) in [
            (&self.minimum, values.minimum),
            (&self.maximum, values.maximum),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        set_select(&self.minimum_boundary, values.minimum_boundary, window, cx);
        set_select(&self.maximum_boundary, values.maximum_boundary, window, cx);
        set_select(&self.value_form, values.value_form, window, cx);
        cx.notify();
    }

    pub(super) fn reset(
        &mut self,
        kind: ValidRangeKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load(kind, None, None, window, cx);
    }

    pub(super) fn to_value(&self, cx: &App) -> Option<ValidRangeValue> {
        if !self.active {
            return None;
        }
        let values = ValidRangeValues {
            minimum: value(&self.minimum, cx),
            maximum: value(&self.maximum, cx),
            minimum_boundary: selected_value(&self.minimum_boundary, BoundaryChoice::Inclusive, cx),
            maximum_boundary: selected_value(&self.maximum_boundary, BoundaryChoice::Inclusive, cx),
            value_form: selected_value(&self.value_form, ValueFormChoice::Calibrated, cx),
        };
        match self.kind {
            ValidRangeKind::Integer => Some(ValidRangeValue::Integer(values.to_integer())),
            ValidRangeKind::Float => Some(ValidRangeValue::Float(values.to_float())),
            ValidRangeKind::Unsupported => None,
        }
    }
}

impl Render for ValidRangeForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.kind == ValidRangeKind::Unsupported {
            return v_flex();
        }
        let header = h_flex()
            .justify_between()
            .child(
                v_flex()
                    .child(div().text_sm().font_medium().child("Valid range"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Limits values accepted as valid"),
                    ),
            )
            .child(if self.active {
                super::section_remove_button("remove-parameter-type-valid-range").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.active = false;
                        cx.notify();
                    }),
                )
            } else {
                super::section_add_button("add-parameter-type-valid-range").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.active = true;
                        cx.notify();
                    },
                ))
            });

        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            let mut card = v_flex()
                .w_full()
                .p_3()
                .gap_3()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(field(
                            "Minimum",
                            if self.kind == ValidRangeKind::Integer {
                                "Optional; inclusive"
                            } else {
                                "Optional"
                            },
                            &self.minimum,
                            cx,
                        ))
                        .child(field(
                            "Maximum",
                            if self.kind == ValidRangeKind::Integer {
                                "Optional; inclusive"
                            } else {
                                "Optional"
                            },
                            &self.maximum,
                            cx,
                        ))
                        .child(select_field("Range applies to", &self.value_form)),
                );
            if self.kind == ValidRangeKind::Float {
                card = card.child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(select_field("Minimum boundary", &self.minimum_boundary))
                        .child(select_field("Maximum boundary", &self.maximum_boundary)),
                );
            }
            form = form.child(card);
        }
        form
    }
}

struct ValidRangeValues {
    minimum: String,
    maximum: String,
    minimum_boundary: BoundaryChoice,
    maximum_boundary: BoundaryChoice,
    value_form: ValueFormChoice,
}

impl ValidRangeValues {
    fn from_range(
        kind: ValidRangeKind,
        integer: Option<&xtce::IntegerDataTypeValidRangeElementType>,
        float: Option<&xtce::FloatDataTypeValidRangeElementType>,
    ) -> Self {
        match (kind, integer, float) {
            (ValidRangeKind::Integer, Some(value), _) => Self {
                minimum: optional_number(value.min_inclusive),
                maximum: optional_number(value.max_inclusive),
                minimum_boundary: BoundaryChoice::Inclusive,
                maximum_boundary: BoundaryChoice::Inclusive,
                value_form: value_form(value.valid_range_applies_to_calibrated),
            },
            (ValidRangeKind::Float, _, Some(value)) => Self {
                minimum: value
                    .min_inclusive
                    .or(value.min_exclusive)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                maximum: value
                    .max_inclusive
                    .or(value.max_exclusive)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                minimum_boundary: if value.min_exclusive.is_some() && value.min_inclusive.is_none()
                {
                    BoundaryChoice::Exclusive
                } else {
                    BoundaryChoice::Inclusive
                },
                maximum_boundary: if value.max_exclusive.is_some() && value.max_inclusive.is_none()
                {
                    BoundaryChoice::Exclusive
                } else {
                    BoundaryChoice::Inclusive
                },
                value_form: value_form(value.valid_range_applies_to_calibrated),
            },
            _ => Self::defaults(),
        }
    }

    fn defaults() -> Self {
        Self {
            minimum: String::new(),
            maximum: String::new(),
            minimum_boundary: BoundaryChoice::Inclusive,
            maximum_boundary: BoundaryChoice::Inclusive,
            value_form: ValueFormChoice::Calibrated,
        }
    }

    fn to_integer(&self) -> xtce::IntegerDataTypeValidRangeElementType {
        xtce::IntegerDataTypeValidRangeElementType {
            min_inclusive: parse_optional(&self.minimum),
            max_inclusive: parse_optional(&self.maximum),
            valid_range_applies_to_calibrated: self.value_form == ValueFormChoice::Calibrated,
        }
    }

    fn to_float(&self) -> xtce::FloatDataTypeValidRangeElementType {
        let minimum = parse_optional(&self.minimum);
        let maximum = parse_optional(&self.maximum);
        xtce::FloatDataTypeValidRangeElementType {
            min_inclusive: (self.minimum_boundary == BoundaryChoice::Inclusive)
                .then_some(minimum)
                .flatten(),
            min_exclusive: (self.minimum_boundary == BoundaryChoice::Exclusive)
                .then_some(minimum)
                .flatten(),
            max_inclusive: (self.maximum_boundary == BoundaryChoice::Inclusive)
                .then_some(maximum)
                .flatten(),
            max_exclusive: (self.maximum_boundary == BoundaryChoice::Exclusive)
                .then_some(maximum)
                .flatten(),
            valid_range_applies_to_calibrated: self.value_form == ValueFormChoice::Calibrated,
        }
    }
}

fn value_form(calibrated: bool) -> ValueFormChoice {
    if calibrated {
        ValueFormChoice::Calibrated
    } else {
        ValueFormChoice::Raw
    }
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
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

fn set_select<T>(
    select: &Entity<SelectState<Vec<T>>>,
    value: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select.update(cx, |select, cx| {
        select.set_selected_value(&value, window, cx);
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
    super::select_field(label, "", select)
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn optional_number<T: ToString>(value: Option<T>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn parse_optional<T: std::str::FromStr>(value: &str) -> Option<T> {
    value.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{BoundaryChoice, ValidRangeKind, ValidRangeValues, ValueFormChoice};

    #[test]
    fn float_range_round_trips_exclusive_boundaries() {
        let value = ValidRangeValues {
            minimum: "-1.5".to_owned(),
            maximum: "12.5".to_owned(),
            minimum_boundary: BoundaryChoice::Exclusive,
            maximum_boundary: BoundaryChoice::Exclusive,
            value_form: ValueFormChoice::Raw,
        }
        .to_float();

        let values = ValidRangeValues::from_range(ValidRangeKind::Float, None, Some(&value));
        assert_eq!(values.minimum, "-1.5");
        assert_eq!(values.maximum, "12.5");
        assert_eq!(values.minimum_boundary, BoundaryChoice::Exclusive);
        assert_eq!(values.maximum_boundary, BoundaryChoice::Exclusive);
        assert_eq!(values.value_form, ValueFormChoice::Raw);
    }
}
