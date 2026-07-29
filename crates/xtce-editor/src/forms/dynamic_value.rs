use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    prelude::FluentBuilder,
};
use gpui_component::{
    IndexPath, StyledExt, h_flex, input::InputState, select::SelectState, v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[default]
    #[strum(serialize = "true")]
    True,
    #[strum(serialize = "false")]
    False,
}
impl_select_item!(BooleanChoice);

pub(super) struct DynamicValueForm {
    parameter_ref: Entity<InputState>,
    instance: Entity<InputState>,
    use_calibrated_value: Entity<SelectState<Vec<BooleanChoice>>>,
    linear_adjustment_present: bool,
    slope: Entity<InputState>,
    intercept: Entity<InputState>,
}

impl DynamicValueForm {
    pub(super) fn new(
        value: Option<&xtce::DynamicValueType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = DynamicValues::from_value(value);
        cx.new(|cx| Self {
            parameter_ref: input(&values.parameter_ref, window, cx),
            instance: input(&values.instance, window, cx),
            use_calibrated_value: boolean_select(values.use_calibrated_value, window, cx),
            linear_adjustment_present: values.linear_adjustment_present,
            slope: input(&values.slope, window, cx),
            intercept: input(&values.intercept, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        value: Option<&xtce::DynamicValueType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = DynamicValues::from_value(value);
        for (input, value) in [
            (&self.parameter_ref, values.parameter_ref),
            (&self.instance, values.instance),
            (&self.slope, values.slope),
            (&self.intercept, values.intercept),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.use_calibrated_value.update(cx, |select, cx| {
            select.set_selected_value(
                &BooleanChoice::from(values.use_calibrated_value),
                window,
                cx,
            );
        });
        self.linear_adjustment_present = values.linear_adjustment_present;
        cx.notify();
    }

    pub(super) fn value(&self, cx: &App) -> xtce::DynamicValueType {
        xtce::DynamicValueType {
            parameter_instance_ref: xtce::ParameterInstanceRefType {
                parameter_ref: input_value(&self.parameter_ref, cx),
                instance: integer(&self.instance, cx),
                use_calibrated_value: selected_boolean(&self.use_calibrated_value, cx),
            },
            linear_adjustment: self
                .linear_adjustment_present
                .then(|| xtce::LinearAdjustmentType {
                    slope: number(&self.slope, 1.0, cx),
                    intercept: number(&self.intercept, 0.0, cx),
                }),
        }
    }
}

impl Render for DynamicValueForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(field(
                        "Parameter reference",
                        "Required",
                        &self.parameter_ref,
                        cx,
                    ))
                    .child(field("Instance", "Defaults to 0", &self.instance, cx))
                    .child(select_field(
                        "Use calibrated value",
                        &self.use_calibrated_value,
                    )),
            )
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        gpui::div()
                            .text_sm()
                            .font_medium()
                            .child("Linear adjustment"),
                    )
                    .child(if self.linear_adjustment_present {
                        super::section_remove_button("remove-dynamic-linear-adjustment").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.linear_adjustment_present = false;
                                cx.notify();
                            }),
                        )
                    } else {
                        super::section_add_button("add-dynamic-linear-adjustment").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.linear_adjustment_present = true;
                                cx.notify();
                            }),
                        )
                    }),
            )
            .when(self.linear_adjustment_present, |form| {
                form.child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(field("Slope", "Defaults to 1", &self.slope, cx))
                        .child(field("Intercept", "Defaults to 0", &self.intercept, cx)),
                )
            })
    }
}

struct DynamicValues {
    parameter_ref: String,
    instance: String,
    use_calibrated_value: bool,
    linear_adjustment_present: bool,
    slope: String,
    intercept: String,
}

impl DynamicValues {
    fn from_value(value: Option<&xtce::DynamicValueType>) -> Self {
        Self {
            parameter_ref: value
                .map(|value| value.parameter_instance_ref.parameter_ref.clone())
                .unwrap_or_default(),
            instance: value
                .map(|value| value.parameter_instance_ref.instance.to_string())
                .unwrap_or_else(|| "0".to_owned()),
            use_calibrated_value: value
                .map(|value| value.parameter_instance_ref.use_calibrated_value)
                .unwrap_or_else(xtce::ParameterInstanceRefType::default_use_calibrated_value),
            linear_adjustment_present: value.is_some_and(|value| value.linear_adjustment.is_some()),
            slope: value
                .and_then(|value| value.linear_adjustment.as_ref())
                .map(|value| value.slope.to_string())
                .unwrap_or_else(|| "1".to_owned()),
            intercept: value
                .and_then(|value| value.linear_adjustment.as_ref())
                .map(|value| value.intercept.to_string())
                .unwrap_or_else(|| "0".to_owned()),
        }
    }
}

impl From<bool> for BooleanChoice {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
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

fn number(input: &Entity<InputState>, fallback: f64, cx: &App) -> f64 {
    input_value(input, cx).trim().parse().unwrap_or(fallback)
}

fn boolean_select(
    value: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<BooleanChoice>>> {
    let selected = BooleanChoice::from(value);
    let index = BooleanChoice::VARIANTS
        .iter()
        .position(|value| *value == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            BooleanChoice::VARIANTS.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn selected_boolean(select: &Entity<SelectState<Vec<BooleanChoice>>>, cx: &App) -> bool {
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or_default()
        == BooleanChoice::True
}

fn select_field(
    label: &'static str,
    select: &Entity<SelectState<Vec<BooleanChoice>>>,
) -> gpui::Div {
    super::select_field(label, "", select)
}

#[cfg(test)]
mod tests {
    use super::DynamicValues;

    #[test]
    fn dynamic_value_fields_are_loaded() {
        let value = xtce::DynamicValueType {
            parameter_instance_ref: xtce::ParameterInstanceRefType {
                parameter_ref: "packetLength".to_owned(),
                instance: -1,
                use_calibrated_value: false,
            },
            linear_adjustment: Some(xtce::LinearAdjustmentType {
                slope: 8.0,
                intercept: 16.0,
            }),
        };

        let values = DynamicValues::from_value(Some(&value));

        assert_eq!(values.parameter_ref, "packetLength");
        assert_eq!(values.instance, "-1");
        assert!(!values.use_calibrated_value);
        assert_eq!(values.slope, "8");
        assert_eq!(values.intercept, "16");
    }
}
