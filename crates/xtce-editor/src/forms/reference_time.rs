use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, prelude::FluentBuilder, px,
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

use super::{field, impl_select_item};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ReferenceTimeKind {
    OffsetFrom,
    Epoch,
}
impl_select_item!(ReferenceTimeKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ValueFormChoice {
    #[strum(serialize = "Calibrated value")]
    Calibrated,
    #[strum(serialize = "Raw value")]
    Raw,
}
impl_select_item!(ValueFormChoice);

pub(super) struct ReferenceTimeForm {
    active: bool,
    kind: ReferenceTimeKind,
    kind_select: Entity<SelectState<Vec<ReferenceTimeKind>>>,
    target: Entity<InputState>,
    instance: Entity<InputState>,
    value_form: Entity<SelectState<Vec<ValueFormChoice>>>,
    _subscriptions: Vec<Subscription>,
}

impl ReferenceTimeForm {
    pub(super) fn new(
        reference_time: Option<&xtce::ReferenceTimeType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = ReferenceTimeValues::from_value(reference_time);
        let kind_select = select(ReferenceTimeKind::VARIANTS, values.kind, window, cx);
        let target = input(&values.target, window, cx);
        let instance = input(&values.instance, window, cx);
        let value_form = select(ValueFormChoice::VARIANTS, values.value_form, window, cx);
        cx.new(move |cx| {
            let target_for_kind = target.clone();
            let instance_for_kind = instance.clone();
            let value_form_for_kind = value_form.clone();
            let subscription = cx.subscribe_in(
                &kind_select,
                window,
                move |this: &mut Self,
                      _,
                      event: &SelectEvent<Vec<ReferenceTimeKind>>,
                      window,
                      cx| {
                    let SelectEvent::Confirm(Some(kind)) = event else {
                        return;
                    };
                    if this.kind == *kind {
                        return;
                    }
                    this.kind = *kind;
                    target_for_kind.update(cx, |input, cx| {
                        input.set_value(
                            if *kind == ReferenceTimeKind::Epoch {
                                "UNIX".to_owned()
                            } else {
                                String::new()
                            },
                            window,
                            cx,
                        );
                    });
                    instance_for_kind.update(cx, |input, cx| {
                        input.set_value("0".to_owned(), window, cx);
                    });
                    set_select(
                        &value_form_for_kind,
                        ValueFormChoice::Calibrated,
                        window,
                        cx,
                    );
                    cx.notify();
                },
            );
            Self {
                active: reference_time.is_some(),
                kind: values.kind,
                kind_select,
                target,
                instance,
                value_form,
                _subscriptions: vec![subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        reference_time: Option<&xtce::ReferenceTimeType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ReferenceTimeValues::from_value(reference_time);
        self.active = reference_time.is_some();
        self.kind = values.kind;
        set_select(&self.kind_select, values.kind, window, cx);
        set_select(&self.value_form, values.value_form, window, cx);
        for (input, value) in [
            (&self.target, values.target),
            (&self.instance, values.instance),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    pub(super) fn to_value(&self, cx: &App) -> Option<xtce::ReferenceTimeType> {
        if !self.active {
            return None;
        }
        let target = value(&self.target, cx);
        Some(match self.kind {
            ReferenceTimeKind::OffsetFrom => {
                xtce::ReferenceTimeType::OffsetFrom(xtce::ParameterInstanceRefType {
                    parameter_ref: target,
                    instance: value(&self.instance, cx)
                        .trim()
                        .parse()
                        .unwrap_or_else(|_| xtce::ParameterInstanceRefType::default_instance()),
                    use_calibrated_value: selected_value(
                        &self.value_form,
                        ValueFormChoice::Calibrated,
                        cx,
                    ) == ValueFormChoice::Calibrated,
                })
            }
            ReferenceTimeKind::Epoch => xtce::ReferenceTimeType::Epoch(epoch(&target)),
        })
    }
}

impl Render for ReferenceTimeForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .justify_between()
            .child(
                v_flex()
                    .child(div().text_sm().font_medium().child("Reference time"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Defines the origin used to interpret the time value"),
                    ),
            )
            .child(if self.active {
                super::section_remove_button("remove-parameter-type-reference-time").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.active = false;
                        cx.notify();
                    }),
                )
            } else {
                super::section_add_button("add-parameter-type-reference-time").on_click(
                    cx.listener(|this, _, window, cx| {
                        this.active = true;
                        this.kind = ReferenceTimeKind::Epoch;
                        set_select(&this.kind_select, ReferenceTimeKind::Epoch, window, cx);
                        this.target.update(cx, |input, cx| {
                            input.set_value("UNIX".to_owned(), window, cx);
                        });
                        cx.notify();
                    }),
                )
            });

        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form.child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_start()
                    .child(select_field("Type", &self.kind_select))
                    .child(field(
                        if self.kind == ReferenceTimeKind::OffsetFrom {
                            "Parameter reference"
                        } else {
                            "Epoch"
                        },
                        if self.kind == ReferenceTimeKind::OffsetFrom {
                            "Required"
                        } else {
                            "TAI, J2000, UNIX, GPS, or a custom epoch"
                        },
                        &self.target,
                        cx,
                    ))
                    .when(self.kind == ReferenceTimeKind::OffsetFrom, |row| {
                        row.child(
                            Button::new("parameter-type-reference-time-options")
                                .mt(px(26.))
                                .small()
                                .ghost()
                                .icon(IconName::Ellipsis)
                                .tooltip("OffsetFrom options")
                                .on_click({
                                    let instance = self.instance.clone();
                                    let value_form = self.value_form.clone();
                                    move |_, window, cx| {
                                        open_offset_options(
                                            instance.clone(),
                                            value_form.clone(),
                                            window,
                                            cx,
                                        );
                                    }
                                }),
                        )
                    }),
            );
        }
        form
    }
}

fn open_offset_options(
    instance: Entity<InputState>,
    value_form: Entity<SelectState<Vec<ValueFormChoice>>>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        let instance = instance.clone();
        let value_form = value_form.clone();
        dialog
            .title("Offset options")
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                content.child(
                    super::form_dialog_content()
                        .child(field("Instance", "Defaults to 0", &instance, cx))
                        .child(select_field("Value form", &value_form)),
                )
            })
    });
}

struct ReferenceTimeValues {
    kind: ReferenceTimeKind,
    target: String,
    instance: String,
    value_form: ValueFormChoice,
}

impl ReferenceTimeValues {
    fn from_value(reference_time: Option<&xtce::ReferenceTimeType>) -> Self {
        match reference_time {
            Some(xtce::ReferenceTimeType::OffsetFrom(value)) => Self {
                kind: ReferenceTimeKind::OffsetFrom,
                target: value.parameter_ref.clone(),
                instance: value.instance.to_string(),
                value_form: if value.use_calibrated_value {
                    ValueFormChoice::Calibrated
                } else {
                    ValueFormChoice::Raw
                },
            },
            Some(xtce::ReferenceTimeType::Epoch(value)) => Self {
                kind: ReferenceTimeKind::Epoch,
                target: epoch_text(value),
                instance: "0".to_owned(),
                value_form: ValueFormChoice::Calibrated,
            },
            None => Self {
                kind: ReferenceTimeKind::Epoch,
                target: "UNIX".to_owned(),
                instance: "0".to_owned(),
                value_form: ValueFormChoice::Calibrated,
            },
        }
    }
}

fn epoch(value: &str) -> xtce::EpochType {
    match value.trim().to_ascii_uppercase().as_str() {
        "TAI" => xtce::EpochType::Tai,
        "J2000" => xtce::EpochType::J2000,
        "UNIX" => xtce::EpochType::Unix,
        "GPS" => xtce::EpochType::Gps,
        _ => xtce::EpochType::String(value.trim().to_owned()),
    }
}

fn epoch_text(value: &xtce::EpochType) -> String {
    match value {
        xtce::EpochType::String(value) => value.clone(),
        xtce::EpochType::Tai => "TAI".to_owned(),
        xtce::EpochType::J2000 => "J2000".to_owned(),
        xtce::EpochType::Unix => "UNIX".to_owned(),
        xtce::EpochType::Gps => "GPS".to_owned(),
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

#[cfg(test)]
mod tests {
    use super::{ReferenceTimeValues, ValueFormChoice, epoch, epoch_text};

    #[test]
    fn reference_time_variants_round_trip() {
        let offset = xtce::ReferenceTimeType::OffsetFrom(xtce::ParameterInstanceRefType {
            parameter_ref: "Clock".to_owned(),
            instance: -2,
            use_calibrated_value: false,
        });
        let values = ReferenceTimeValues::from_value(Some(&offset));
        assert_eq!(values.target, "Clock");
        assert_eq!(values.instance, "-2");
        assert_eq!(values.value_form, ValueFormChoice::Raw);

        for text in ["TAI", "J2000", "UNIX", "GPS", "2025-01-01T00:00:00Z"] {
            assert_eq!(epoch_text(&epoch(text)), text);
        }
    }
}
