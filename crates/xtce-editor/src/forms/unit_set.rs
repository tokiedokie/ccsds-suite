use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    px,
};
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    select::SelectState,
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item, optional_value};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum UnitFormChoice {
    #[strum(serialize = "calibrated")]
    Calibrated,
    #[strum(serialize = "uncalibrated")]
    Uncalibrated,
    #[strum(serialize = "raw")]
    Raw,
}
impl_select_item!(UnitFormChoice);

struct UnitRowForm {
    text: Entity<InputState>,
    form: Entity<SelectState<Vec<UnitFormChoice>>>,
    power: Entity<InputState>,
    factor: Entity<InputState>,
    description: Entity<InputState>,
}

pub(super) struct UnitSetForm {
    rows: Vec<Entity<UnitRowForm>>,
}

impl UnitSetForm {
    pub(super) fn new(
        set: Option<&xtce::UnitSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = unit_entities(set, window, cx);
        cx.new(move |_| Self { rows })
    }

    pub(super) fn load(
        &mut self,
        set: Option<&xtce::UnitSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = unit_entities(set, window, cx);
        cx.notify();
    }

    pub(super) fn to_set(&self, cx: &App) -> Option<xtce::UnitSetType> {
        let unit = self
            .rows
            .iter()
            .map(|row| {
                let row = row.read(cx);
                UnitValues {
                    text: value(&row.text, cx),
                    form: selected_value(&row.form, UnitFormChoice::Calibrated, cx),
                    power: value(&row.power, cx),
                    factor: value(&row.factor, cx),
                    description: value(&row.description, cx),
                }
                .to_unit()
            })
            .collect::<Vec<_>>();
        (!unit.is_empty()).then_some(xtce::UnitSetType { unit })
    }
}

impl Render for UnitSetForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Units"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} unit(s)", self.rows.len())),
                            ),
                    )
                    .child(
                        Button::new("add-parameter-type-unit")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add unit")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(unit_entity(None, window, cx));
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
                    .items_start()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .flex_1()
                            .child(field("Unit text", "Optional", &row_read.text, cx)),
                    )
                    .child(
                        div()
                            .w(px(180.))
                            .child(select_field("Form", &row_read.form)),
                    )
                    .child(
                        h_flex()
                            .mt(px(26.))
                            .gap_1()
                            .child(
                                Button::new(format!("parameter-type-unit-options-{index}"))
                                    .small()
                                    .ghost()
                                    .icon(IconName::Ellipsis)
                                    .tooltip("Unit options")
                                    .on_click({
                                        let row = row.clone();
                                        move |_, window, cx| {
                                            open_unit_options(row.clone(), window, cx);
                                        }
                                    }),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-parameter-type-unit-{index}"),
                                    "Remove unit",
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.rows.remove(index);
                                        cx.notify();
                                    },
                                )),
                            ),
                    )
            }))
    }
}

fn open_unit_options(editor: Entity<UnitRowForm>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title("Unit options")
            .w(px(680.))
            .content(move |content, _, cx| {
                let row = editor.read(cx);
                content.child(
                    v_flex()
                        .p_4()
                        .gap_4()
                        .child(field("Power", "Optional; defaults to 1", &row.power, cx))
                        .child(field("Factor", "Optional; defaults to 1", &row.factor, cx))
                        .child(field("Description", "Optional", &row.description, cx)),
                )
            })
    });
}

struct UnitValues {
    text: String,
    form: UnitFormChoice,
    power: String,
    factor: String,
    description: String,
}

impl UnitValues {
    fn from_unit(unit: &xtce::UnitType) -> Self {
        Self {
            text: unit
                .text
                .as_ref()
                .map(|text| text.0.clone())
                .unwrap_or_default(),
            form: match unit.form {
                xtce::UnitFormType::Calibrated => UnitFormChoice::Calibrated,
                xtce::UnitFormType::Uncalibrated => UnitFormChoice::Uncalibrated,
                xtce::UnitFormType::Raw => UnitFormChoice::Raw,
            },
            power: unit.power.to_string(),
            factor: unit.factor.clone(),
            description: unit.description.clone().unwrap_or_default(),
        }
    }

    fn to_unit(&self) -> xtce::UnitType {
        xtce::UnitType {
            power: self
                .power
                .trim()
                .parse()
                .unwrap_or_else(|_| xtce::UnitType::default_power()),
            factor: if self.factor.trim().is_empty() {
                xtce::UnitType::default_factor()
            } else {
                self.factor.trim().to_owned()
            },
            description: optional_value(self.description.trim().to_owned()),
            form: match self.form {
                UnitFormChoice::Calibrated => xtce::UnitFormType::Calibrated,
                UnitFormChoice::Uncalibrated => xtce::UnitFormType::Uncalibrated,
                UnitFormChoice::Raw => xtce::UnitFormType::Raw,
            },
            text: optional_value(self.text.clone()).map(xtce::XmlText),
        }
    }
}

fn unit_entities(
    set: Option<&xtce::UnitSetType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<UnitRowForm>> {
    set.into_iter()
        .flat_map(|set| &set.unit)
        .map(|unit| unit_entity(Some(unit), window, cx))
        .collect()
}

fn unit_entity(
    unit: Option<&xtce::UnitType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<UnitRowForm> {
    let values = unit.map(UnitValues::from_unit).unwrap_or(UnitValues {
        text: String::new(),
        form: UnitFormChoice::Calibrated,
        power: xtce::UnitType::default_power().to_string(),
        factor: xtce::UnitType::default_factor(),
        description: String::new(),
    });
    let text = input(&values.text, window, cx);
    let form = select(UnitFormChoice::VARIANTS, values.form, window, cx);
    let power = input(&values.power, window, cx);
    let factor = input(&values.factor, window, cx);
    let description = input(&values.description, window, cx);
    cx.new(|_| UnitRowForm {
        text,
        form,
        power,
        factor,
        description,
    })
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
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
    use super::{UnitFormChoice, UnitValues};

    #[test]
    fn unit_values_round_trip_all_fields() {
        let values = UnitValues {
            text: "m/s".to_owned(),
            form: UnitFormChoice::Raw,
            power: "2".to_owned(),
            factor: "0.5".to_owned(),
            description: "velocity".to_owned(),
        };
        let unit = values.to_unit();
        let loaded = UnitValues::from_unit(&unit);

        assert_eq!(loaded.text, "m/s");
        assert_eq!(loaded.form, UnitFormChoice::Raw);
        assert_eq!(loaded.power, "2");
        assert_eq!(loaded.factor, "0.5");
        assert_eq!(loaded.description, "velocity");
    }
}
