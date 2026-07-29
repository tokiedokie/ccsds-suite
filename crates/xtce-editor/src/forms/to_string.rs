use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder, px,
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

use super::{field, impl_select_item};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum RadixChoice {
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
}
impl_select_item!(RadixChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum NotationChoice {
    Normal,
    Scientific,
    Engineering,
}
impl_select_item!(NotationChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum GroupingChoice {
    #[strum(serialize = "No")]
    Disabled,
    #[strum(serialize = "Yes")]
    Enabled,
}
impl_select_item!(GroupingChoice);

pub(super) struct ToStringForm {
    active: bool,
    number_base: Entity<SelectState<Vec<RadixChoice>>>,
    notation: Entity<SelectState<Vec<NotationChoice>>>,
    minimum_integer_digits: Entity<InputState>,
    maximum_integer_digits: Entity<InputState>,
    minimum_fraction_digits: Entity<InputState>,
    maximum_fraction_digits: Entity<InputState>,
    negative_prefix: Entity<InputState>,
    negative_suffix: Entity<InputState>,
    positive_prefix: Entity<InputState>,
    positive_suffix: Entity<InputState>,
    show_thousands_grouping: Entity<SelectState<Vec<GroupingChoice>>>,
}

impl ToStringForm {
    pub(super) fn new(
        value: Option<&xtce::ToStringType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = ToStringValues::from_value(value);
        let number_base = select(RadixChoice::VARIANTS, values.number_base, window, cx);
        let notation = select(NotationChoice::VARIANTS, values.notation, window, cx);
        let minimum_integer_digits = input(&values.minimum_integer_digits, window, cx);
        let maximum_integer_digits = input(&values.maximum_integer_digits, window, cx);
        let minimum_fraction_digits = input(&values.minimum_fraction_digits, window, cx);
        let maximum_fraction_digits = input(&values.maximum_fraction_digits, window, cx);
        let negative_prefix = input(&values.negative_prefix, window, cx);
        let negative_suffix = input(&values.negative_suffix, window, cx);
        let positive_prefix = input(&values.positive_prefix, window, cx);
        let positive_suffix = input(&values.positive_suffix, window, cx);
        let show_thousands_grouping = select(
            GroupingChoice::VARIANTS,
            values.show_thousands_grouping,
            window,
            cx,
        );
        cx.new(move |_| Self {
            active: value.is_some(),
            number_base,
            notation,
            minimum_integer_digits,
            maximum_integer_digits,
            minimum_fraction_digits,
            maximum_fraction_digits,
            negative_prefix,
            negative_suffix,
            positive_prefix,
            positive_suffix,
            show_thousands_grouping,
        })
    }

    pub(super) fn load(
        &mut self,
        value: Option<&xtce::ToStringType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ToStringValues::from_value(value);
        self.active = value.is_some();
        set_select(&self.number_base, values.number_base, window, cx);
        set_select(&self.notation, values.notation, window, cx);
        set_select(
            &self.show_thousands_grouping,
            values.show_thousands_grouping,
            window,
            cx,
        );
        for (input, value) in [
            (&self.minimum_integer_digits, values.minimum_integer_digits),
            (&self.maximum_integer_digits, values.maximum_integer_digits),
            (
                &self.minimum_fraction_digits,
                values.minimum_fraction_digits,
            ),
            (
                &self.maximum_fraction_digits,
                values.maximum_fraction_digits,
            ),
            (&self.negative_prefix, values.negative_prefix),
            (&self.negative_suffix, values.negative_suffix),
            (&self.positive_prefix, values.positive_prefix),
            (&self.positive_suffix, values.positive_suffix),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    pub(super) fn to_value(&self, cx: &App) -> Option<xtce::ToStringType> {
        self.active.then(|| {
            ToStringValues {
                number_base: selected_value(&self.number_base, RadixChoice::Decimal, cx),
                notation: selected_value(&self.notation, NotationChoice::Normal, cx),
                minimum_integer_digits: value(&self.minimum_integer_digits, cx),
                maximum_integer_digits: value(&self.maximum_integer_digits, cx),
                minimum_fraction_digits: value(&self.minimum_fraction_digits, cx),
                maximum_fraction_digits: value(&self.maximum_fraction_digits, cx),
                negative_prefix: value(&self.negative_prefix, cx),
                negative_suffix: value(&self.negative_suffix, cx),
                positive_prefix: value(&self.positive_prefix, cx),
                positive_suffix: value(&self.positive_suffix, cx),
                show_thousands_grouping: selected_value(
                    &self.show_thousands_grouping,
                    GroupingChoice::Disabled,
                    cx,
                ),
            }
            .to_value()
        })
    }
}

impl Render for ToStringForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .justify_between()
            .child(
                v_flex()
                    .child(div().text_sm().font_medium().child("Number formatting"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Controls how numeric values are displayed as text"),
                    ),
            )
            .child(if self.active {
                super::section_remove_button("remove-parameter-type-to-string").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.active = false;
                        cx.notify();
                    }),
                )
            } else {
                super::section_add_button("add-parameter-type-to-string").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.active = true;
                        cx.notify();
                    },
                ))
            });

        v_flex()
            .w_full()
            .gap_3()
            .child(header)
            .when(self.active, |form| {
                form.child(
                    v_flex()
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
                                .items_end()
                                .child(
                                    div()
                                        .flex_1()
                                        .child(select_field("Number base", &self.number_base)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .child(select_field("Notation", &self.notation)),
                                ),
                        )
                        .child(
                            h_flex()
                                .w_full()
                                .gap_3()
                                .items_start()
                                .child(div().flex_1().child(field(
                                    "Minimum integer digits",
                                    "Defaults to 1",
                                    &self.minimum_integer_digits,
                                    cx,
                                )))
                                .child(div().flex_1().child(field(
                                    "Maximum integer digits",
                                    "Optional",
                                    &self.maximum_integer_digits,
                                    cx,
                                )))
                                .child(super::action_field(
                                    Button::new("parameter-type-to-string-options")
                                        .small()
                                        .ghost()
                                        .icon(IconName::Ellipsis)
                                        .tooltip("Additional number formatting options")
                                        .on_click({
                                            let minimum_fraction_digits =
                                                self.minimum_fraction_digits.clone();
                                            let maximum_fraction_digits =
                                                self.maximum_fraction_digits.clone();
                                            let negative_prefix = self.negative_prefix.clone();
                                            let negative_suffix = self.negative_suffix.clone();
                                            let positive_prefix = self.positive_prefix.clone();
                                            let positive_suffix = self.positive_suffix.clone();
                                            let show_thousands_grouping =
                                                self.show_thousands_grouping.clone();
                                            move |_, window, cx| {
                                                open_options(
                                                    minimum_fraction_digits.clone(),
                                                    maximum_fraction_digits.clone(),
                                                    negative_prefix.clone(),
                                                    negative_suffix.clone(),
                                                    positive_prefix.clone(),
                                                    positive_suffix.clone(),
                                                    show_thousands_grouping.clone(),
                                                    window,
                                                    cx,
                                                );
                                            }
                                        }),
                                )),
                        ),
                )
            })
    }
}

#[allow(clippy::too_many_arguments)]
fn open_options(
    minimum_fraction_digits: Entity<InputState>,
    maximum_fraction_digits: Entity<InputState>,
    negative_prefix: Entity<InputState>,
    negative_suffix: Entity<InputState>,
    positive_prefix: Entity<InputState>,
    positive_suffix: Entity<InputState>,
    show_thousands_grouping: Entity<SelectState<Vec<GroupingChoice>>>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        let minimum_fraction_digits = minimum_fraction_digits.clone();
        let maximum_fraction_digits = maximum_fraction_digits.clone();
        let negative_prefix = negative_prefix.clone();
        let negative_suffix = negative_suffix.clone();
        let positive_prefix = positive_prefix.clone();
        let positive_suffix = positive_suffix.clone();
        let show_thousands_grouping = show_thousands_grouping.clone();
        dialog
            .title("Number formatting options")
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                content.child(
                    super::form_dialog_content()
                        .child(
                            h_flex()
                                .gap_4()
                                .items_start()
                                .child(field(
                                    "Minimum fraction digits",
                                    "Defaults to 0",
                                    &minimum_fraction_digits,
                                    cx,
                                ))
                                .child(field(
                                    "Maximum fraction digits",
                                    "Optional",
                                    &maximum_fraction_digits,
                                    cx,
                                )),
                        )
                        .child(
                            h_flex()
                                .gap_4()
                                .items_start()
                                .child(field(
                                    "Negative prefix",
                                    "Defaults to -",
                                    &negative_prefix,
                                    cx,
                                ))
                                .child(field("Negative suffix", "Optional", &negative_suffix, cx)),
                        )
                        .child(
                            h_flex()
                                .gap_4()
                                .items_start()
                                .child(field("Positive prefix", "Optional", &positive_prefix, cx))
                                .child(field("Positive suffix", "Optional", &positive_suffix, cx)),
                        )
                        .child(select_field("Thousands grouping", &show_thousands_grouping)),
                )
            })
    });
}

struct ToStringValues {
    number_base: RadixChoice,
    notation: NotationChoice,
    minimum_integer_digits: String,
    maximum_integer_digits: String,
    minimum_fraction_digits: String,
    maximum_fraction_digits: String,
    negative_prefix: String,
    negative_suffix: String,
    positive_prefix: String,
    positive_suffix: String,
    show_thousands_grouping: GroupingChoice,
}

impl ToStringValues {
    fn from_value(value: Option<&xtce::ToStringType>) -> Self {
        let Some(value) = value else {
            return Self::defaults();
        };
        let format = &value.number_format;
        Self {
            number_base: RadixChoice::from_xtce(&format.number_base),
            notation: NotationChoice::from_xtce(&format.notation),
            minimum_integer_digits: format.minimum_integer_digits.to_string(),
            maximum_integer_digits: optional_number(format.maximum_integer_digits),
            minimum_fraction_digits: format.minimum_fraction_digits.to_string(),
            maximum_fraction_digits: optional_number(format.maximum_fraction_digits),
            negative_prefix: format.negative_prefix.clone(),
            negative_suffix: format.negative_suffix.clone(),
            positive_prefix: format.positive_prefix.clone(),
            positive_suffix: format.positive_suffix.clone(),
            show_thousands_grouping: if format.show_thousands_grouping {
                GroupingChoice::Enabled
            } else {
                GroupingChoice::Disabled
            },
        }
    }

    fn defaults() -> Self {
        Self {
            number_base: RadixChoice::Decimal,
            notation: NotationChoice::Normal,
            minimum_integer_digits: "1".to_owned(),
            maximum_integer_digits: String::new(),
            minimum_fraction_digits: "0".to_owned(),
            maximum_fraction_digits: String::new(),
            negative_prefix: "-".to_owned(),
            negative_suffix: String::new(),
            positive_prefix: String::new(),
            positive_suffix: String::new(),
            show_thousands_grouping: GroupingChoice::Disabled,
        }
    }

    fn to_value(&self) -> xtce::ToStringType {
        xtce::ToStringType {
            number_format: xtce::NumberFormatType {
                number_base: self.number_base.to_xtce(),
                minimum_fraction_digits: parse_or(
                    &self.minimum_fraction_digits,
                    xtce::NumberFormatType::default_minimum_fraction_digits(),
                ),
                maximum_fraction_digits: parse_optional(&self.maximum_fraction_digits),
                minimum_integer_digits: parse_or(
                    &self.minimum_integer_digits,
                    xtce::NumberFormatType::default_minimum_integer_digits(),
                ),
                maximum_integer_digits: parse_optional(&self.maximum_integer_digits),
                negative_suffix: self.negative_suffix.clone(),
                positive_suffix: self.positive_suffix.clone(),
                negative_prefix: self.negative_prefix.clone(),
                positive_prefix: self.positive_prefix.clone(),
                show_thousands_grouping: self.show_thousands_grouping == GroupingChoice::Enabled,
                notation: self.notation.to_xtce(),
            },
        }
    }
}

impl RadixChoice {
    fn from_xtce(value: &xtce::RadixType) -> Self {
        match value {
            xtce::RadixType::Decimal => Self::Decimal,
            xtce::RadixType::Hexadecimal => Self::Hexadecimal,
            xtce::RadixType::Octal => Self::Octal,
            xtce::RadixType::Binary => Self::Binary,
        }
    }

    fn to_xtce(self) -> xtce::RadixType {
        match self {
            Self::Decimal => xtce::RadixType::Decimal,
            Self::Hexadecimal => xtce::RadixType::Hexadecimal,
            Self::Octal => xtce::RadixType::Octal,
            Self::Binary => xtce::RadixType::Binary,
        }
    }
}

impl NotationChoice {
    fn from_xtce(value: &xtce::FloatingPointNotationType) -> Self {
        match value {
            xtce::FloatingPointNotationType::Normal => Self::Normal,
            xtce::FloatingPointNotationType::Scientific => Self::Scientific,
            xtce::FloatingPointNotationType::Engineering => Self::Engineering,
        }
    }

    fn to_xtce(self) -> xtce::FloatingPointNotationType {
        match self {
            Self::Normal => xtce::FloatingPointNotationType::Normal,
            Self::Scientific => xtce::FloatingPointNotationType::Scientific,
            Self::Engineering => xtce::FloatingPointNotationType::Engineering,
        }
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

fn parse_or<T: std::str::FromStr>(value: &str, fallback: T) -> T {
    value.trim().parse().unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{GroupingChoice, NotationChoice, RadixChoice, ToStringValues};

    #[test]
    fn number_format_round_trips_all_fields() {
        let value = ToStringValues {
            number_base: RadixChoice::Hexadecimal,
            notation: NotationChoice::Engineering,
            minimum_integer_digits: "3".to_owned(),
            maximum_integer_digits: "8".to_owned(),
            minimum_fraction_digits: "2".to_owned(),
            maximum_fraction_digits: "5".to_owned(),
            negative_prefix: "(".to_owned(),
            negative_suffix: ")".to_owned(),
            positive_prefix: "+".to_owned(),
            positive_suffix: " V".to_owned(),
            show_thousands_grouping: GroupingChoice::Enabled,
        }
        .to_value();

        let values = ToStringValues::from_value(Some(&value));
        assert_eq!(values.number_base, RadixChoice::Hexadecimal);
        assert_eq!(values.notation, NotationChoice::Engineering);
        assert_eq!(values.minimum_integer_digits, "3");
        assert_eq!(values.maximum_integer_digits, "8");
        assert_eq!(values.minimum_fraction_digits, "2");
        assert_eq!(values.maximum_fraction_digits, "5");
        assert_eq!(values.negative_prefix, "(");
        assert_eq!(values.negative_suffix, ")");
        assert_eq!(values.positive_prefix, "+");
        assert_eq!(values.positive_suffix, " V");
        assert_eq!(values.show_thousands_grouping, GroupingChoice::Enabled);
    }
}
