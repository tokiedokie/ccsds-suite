use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    IconName, IndexPath, Sizable, StyledExt, button::Button, h_flex, input::InputState,
    select::SelectState, v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum RateBasis {
    #[strum(serialize = "perSecond")]
    #[default]
    PerSecond,
    #[strum(serialize = "perContainerUpdate")]
    PerContainerUpdate,
}
impl_select_item!(RateBasis);

pub(super) struct ContainerRateForm {
    default_present: bool,
    default_basis: Entity<SelectState<Vec<RateBasis>>>,
    default_minimum: Entity<InputState>,
    default_maximum: Entity<InputState>,
    stream_rates: Entity<StreamRateList>,
}

impl ContainerRateForm {
    pub(super) fn new(
        default_rate: Option<&xtce::RateInStreamType>,
        rate_set: Option<&xtce::RateInStreamSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let default = RateValues::from_default(default_rate);
        let rows = rate_set
            .into_iter()
            .flat_map(|set| &set.rate_in_stream)
            .map(RateValues::from_stream)
            .collect::<Vec<_>>();
        let stream_rates = cx.new(|cx| StreamRateList {
            rows: rows
                .into_iter()
                .map(|row| new_stream_rate_row(row, window, cx))
                .collect(),
        });
        cx.new(|cx| Self {
            default_present: default_rate.is_some(),
            default_basis: select(default.basis, window, cx),
            default_minimum: input(&default.minimum, window, cx),
            default_maximum: input(&default.maximum, window, cx),
            stream_rates,
        })
    }

    pub(super) fn load(
        &mut self,
        default_rate: Option<&xtce::RateInStreamType>,
        rate_set: Option<&xtce::RateInStreamSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let default = RateValues::from_default(default_rate);
        self.default_present = default_rate.is_some();
        sync_select(&self.default_basis, default.basis, window, cx);
        for (input, value) in [
            (&self.default_minimum, default.minimum),
            (&self.default_maximum, default.maximum),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        let rows = rate_set
            .into_iter()
            .flat_map(|set| &set.rate_in_stream)
            .map(RateValues::from_stream)
            .collect();
        self.stream_rates
            .update(cx, |list, cx| list.set_rows(rows, window, cx));
        cx.notify();
    }

    pub(super) fn apply_to(
        &self,
        default_rate: &mut Option<xtce::RateInStreamType>,
        rate_set: &mut Option<xtce::RateInStreamSetType>,
        cx: &App,
    ) {
        *default_rate = self.default_present.then(|| xtce::RateInStreamType {
            basis: selected_basis(&self.default_basis, cx).into(),
            minimum_value: number(&self.default_minimum, cx),
            maximum_value: number(&self.default_maximum, cx),
        });
        let rows = self.stream_rates.read(cx).values(cx);
        *rate_set = (!rows.is_empty()).then(|| xtce::RateInStreamSetType {
            rate_in_stream: rows
                .into_iter()
                .map(|row| xtce::RateInStreamWithStreamNameType {
                    basis: row.basis.into(),
                    minimum_value: parse_number(&row.minimum),
                    maximum_value: parse_number(&row.maximum),
                    stream_ref: row.stream_ref,
                })
                .collect(),
        });
    }
}

impl Render for ContainerRateForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_sm().font_medium().child("Default rate"))
                    .child(if self.default_present {
                        super::section_remove_button("remove-default-container-rate").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.default_present = false;
                                cx.notify();
                            }),
                        )
                    } else {
                        super::section_add_button("add-default-container-rate").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.default_present = true;
                                cx.notify();
                            }),
                        )
                    }),
            )
            .when(self.default_present, |form| {
                form.child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(select_field("Basis", &self.default_basis))
                        .child(field(
                            "Minimum value",
                            "Optional",
                            &self.default_minimum,
                            cx,
                        ))
                        .child(field(
                            "Maximum value",
                            "Optional",
                            &self.default_maximum,
                            cx,
                        )),
                )
            })
            .child(self.stream_rates.clone())
    }
}

struct StreamRateList {
    rows: Vec<Entity<StreamRateRow>>,
}

struct StreamRateRow {
    stream_ref: Entity<InputState>,
    basis: Entity<SelectState<Vec<RateBasis>>>,
    minimum: Entity<InputState>,
    maximum: Entity<InputState>,
}

impl StreamRateList {
    fn set_rows(&mut self, rows: Vec<RateValues>, window: &mut Window, cx: &mut Context<Self>) {
        self.rows = rows
            .into_iter()
            .map(|row| new_stream_rate_row(row, window, cx))
            .collect();
        cx.notify();
    }

    fn values(&self, cx: &App) -> Vec<RateValues> {
        self.rows
            .iter()
            .map(|row| {
                let row = row.read(cx);
                RateValues {
                    stream_ref: value(&row.stream_ref, cx),
                    basis: selected_basis(&row.basis, cx),
                    minimum: value(&row.minimum, cx),
                    maximum: value(&row.maximum, cx),
                }
            })
            .filter(|row| !row.stream_ref.trim().is_empty())
            .collect()
    }
}

impl Render for StreamRateList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                super::compact_list_row(cx)
                    .items_start()
                    .child(row.clone())
                    .child(super::action_field(
                        super::row_remove_button(
                            format!("remove-stream-rate-{index}"),
                            "Remove stream rate",
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if index < this.rows.len() {
                                this.rows.remove(index);
                                cx.notify();
                            }
                        })),
                    ))
            })
            .collect::<Vec<_>>();
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_sm().font_medium().child("Per-stream rates"))
                    .child(
                        Button::new("add-stream-rate")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add stream rate")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.rows.push(new_stream_rate_row(
                                    RateValues::default(),
                                    window,
                                    cx,
                                ));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(super::empty_list_state("No per-stream rates defined.", cx))
            })
            .children(rows)
    }
}

impl Render for StreamRateRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .gap_2()
            .items_start()
            .child(field("Stream reference", "Required", &self.stream_ref, cx))
            .child(select_field("Basis", &self.basis))
            .child(field("Minimum", "Optional", &self.minimum, cx))
            .child(field("Maximum", "Optional", &self.maximum, cx))
    }
}

#[derive(Default)]
struct RateValues {
    stream_ref: String,
    basis: RateBasis,
    minimum: String,
    maximum: String,
}

impl RateValues {
    fn from_default(rate: Option<&xtce::RateInStreamType>) -> Self {
        rate.map_or_else(Self::default, |rate| Self {
            stream_ref: String::new(),
            basis: (&rate.basis).into(),
            minimum: rate
                .minimum_value
                .map(|value| value.to_string())
                .unwrap_or_default(),
            maximum: rate
                .maximum_value
                .map(|value| value.to_string())
                .unwrap_or_default(),
        })
    }

    fn from_stream(rate: &xtce::RateInStreamWithStreamNameType) -> Self {
        Self {
            stream_ref: rate.stream_ref.clone(),
            basis: (&rate.basis).into(),
            minimum: rate
                .minimum_value
                .map(|value| value.to_string())
                .unwrap_or_default(),
            maximum: rate
                .maximum_value
                .map(|value| value.to_string())
                .unwrap_or_default(),
        }
    }
}

impl From<RateBasis> for xtce::BasisType {
    fn from(value: RateBasis) -> Self {
        match value {
            RateBasis::PerSecond => Self::PerSecond,
            RateBasis::PerContainerUpdate => Self::PerContainerUpdate,
        }
    }
}

impl From<&xtce::BasisType> for RateBasis {
    fn from(value: &xtce::BasisType) -> Self {
        match value {
            xtce::BasisType::PerSecond => Self::PerSecond,
            xtce::BasisType::PerContainerUpdate => Self::PerContainerUpdate,
        }
    }
}

fn new_stream_rate_row(
    values: RateValues,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<StreamRateRow> {
    cx.new(|cx| StreamRateRow {
        stream_ref: input(&values.stream_ref, window, cx),
        basis: select(values.basis, window, cx),
        minimum: input(&values.minimum, window, cx),
        maximum: input(&values.maximum, window, cx),
    })
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn number(input: &Entity<InputState>, cx: &App) -> Option<f64> {
    parse_number(&value(input, cx))
}

fn parse_number(value: &str) -> Option<f64> {
    value.trim().parse().ok()
}

fn select(
    selected: RateBasis,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<RateBasis>>> {
    let index = RateBasis::VARIANTS
        .iter()
        .position(|candidate| candidate == &selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            RateBasis::VARIANTS.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn sync_select(
    select: &Entity<SelectState<Vec<RateBasis>>>,
    value: RateBasis,
    window: &mut Window,
    cx: &mut impl AppContext,
) {
    select.update(cx, |select, cx| {
        select.set_selected_value(&value, window, cx);
    });
}

fn selected_basis(select: &Entity<SelectState<Vec<RateBasis>>>, cx: &App) -> RateBasis {
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or_default()
}

fn select_field(label: &'static str, select: &Entity<SelectState<Vec<RateBasis>>>) -> Div {
    super::select_field(label, "Required", select)
}

#[cfg(test)]
mod tests {
    use super::{RateBasis, RateValues, parse_number};

    #[test]
    fn rate_values_keep_the_basis_and_optional_bounds() {
        let values = RateValues::from_stream(&xtce::RateInStreamWithStreamNameType {
            basis: xtce::BasisType::PerContainerUpdate,
            minimum_value: Some(1.5),
            maximum_value: None,
            stream_ref: "TelemetryStream".to_owned(),
        });

        assert_eq!(values.stream_ref, "TelemetryStream");
        assert_eq!(values.basis, RateBasis::PerContainerUpdate);
        assert_eq!(parse_number(&values.minimum), Some(1.5));
        assert_eq!(parse_number(&values.maximum), None);
    }
}
