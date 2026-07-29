use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, IndexPath, StyledExt, h_flex, input::InputState, select::SelectState, v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    data_encoding::{DataEncodingForm, DataEncodingKind, DataEncodingMut, DataEncodingRef},
    field, impl_select_item,
};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum TimeUnitsChoice {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Picoseconds,
    Minutes,
    Hours,
    Days,
    Months,
    Years,
}
impl_select_item!(TimeUnitsChoice);

pub(super) struct TimeEncodingForm {
    active: bool,
    units: Entity<SelectState<Vec<TimeUnitsChoice>>>,
    scale: Entity<InputState>,
    offset: Entity<InputState>,
    data_encoding: Entity<DataEncodingForm>,
}

impl TimeEncodingForm {
    pub(super) fn new(
        encoding: Option<&xtce::EncodingType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = TimeEncodingValues::from_encoding(encoding);
        let units = select(TimeUnitsChoice::VARIANTS, values.units, window, cx);
        let scale = input(&values.scale, window, cx);
        let offset = input(&values.offset, window, cx);
        let data_encoding = DataEncodingForm::new(
            encoding.map(|value| encoding_ref(&value.content)),
            window,
            cx,
        );
        cx.new(move |_| Self {
            active: encoding.is_some(),
            units,
            scale,
            offset,
            data_encoding,
        })
    }

    pub(super) fn load(
        &mut self,
        encoding: Option<&xtce::EncodingType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = TimeEncodingValues::from_encoding(encoding);
        self.active = encoding.is_some();
        set_select(&self.units, values.units, window, cx);
        for (input, value) in [(&self.scale, values.scale), (&self.offset, values.offset)] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.data_encoding.update(cx, |form, cx| {
            form.load(
                encoding.map(|value| encoding_ref(&value.content)),
                window,
                cx,
            );
        });
        cx.notify();
    }

    pub(super) fn to_value(&self, cx: &App) -> Option<xtce::EncodingType> {
        if !self.active {
            return None;
        }
        let kind = self.data_encoding.read(cx).selected_kind(cx)?;
        let mut content = default_content(kind);
        self.data_encoding
            .read(cx)
            .apply_to(encoding_mut(&mut content), cx);
        Some(xtce::EncodingType {
            units: selected_value(&self.units, TimeUnitsChoice::Seconds, cx).to_xtce(),
            scale: parse_or(&value(&self.scale, cx), xtce::EncodingType::default_scale()),
            offset: parse_or(
                &value(&self.offset, cx),
                xtce::EncodingType::default_offset(),
            ),
            content,
        })
    }
}

impl Render for TimeEncodingForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .justify_between()
            .child(
                v_flex()
                    .child(div().text_sm().font_medium().child("Time encoding"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Defines how the time value is stored"),
                    ),
            )
            .child(if self.active {
                super::section_remove_button("remove-parameter-type-time-encoding").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.active = false;
                        cx.notify();
                    }),
                )
            } else {
                super::section_add_button("add-parameter-type-time-encoding").on_click(cx.listener(
                    |this, _, window, cx| {
                        this.active = true;
                        let default = default_integer_encoding();
                        this.data_encoding = DataEncodingForm::new(
                            Some(DataEncodingRef::Integer(&default)),
                            window,
                            cx,
                        );
                        cx.notify();
                    },
                ))
            });

        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(select_field("Units", &self.units))
                        .child(field("Scale", "Defaults to 1", &self.scale, cx))
                        .child(field("Offset", "Defaults to 0", &self.offset, cx)),
                )
                .child(div().text_sm().font_medium().child("Data encoding"))
                .child(self.data_encoding.clone());
        }
        form
    }
}

struct TimeEncodingValues {
    units: TimeUnitsChoice,
    scale: String,
    offset: String,
}

impl TimeEncodingValues {
    fn from_encoding(encoding: Option<&xtce::EncodingType>) -> Self {
        let Some(encoding) = encoding else {
            return Self {
                units: TimeUnitsChoice::Seconds,
                scale: xtce::EncodingType::default_scale().to_string(),
                offset: xtce::EncodingType::default_offset().to_string(),
            };
        };
        Self {
            units: TimeUnitsChoice::from_xtce(&encoding.units),
            scale: encoding.scale.to_string(),
            offset: encoding.offset.to_string(),
        }
    }
}

impl TimeUnitsChoice {
    fn from_xtce(value: &xtce::TimeUnitsType) -> Self {
        match value {
            xtce::TimeUnitsType::Seconds => Self::Seconds,
            xtce::TimeUnitsType::Milliseconds => Self::Milliseconds,
            xtce::TimeUnitsType::Microseconds => Self::Microseconds,
            xtce::TimeUnitsType::Nanoseconds => Self::Nanoseconds,
            xtce::TimeUnitsType::Picoseconds => Self::Picoseconds,
            xtce::TimeUnitsType::Minutes => Self::Minutes,
            xtce::TimeUnitsType::Hours => Self::Hours,
            xtce::TimeUnitsType::Days => Self::Days,
            xtce::TimeUnitsType::Months => Self::Months,
            xtce::TimeUnitsType::Years => Self::Years,
        }
    }

    fn to_xtce(self) -> xtce::TimeUnitsType {
        match self {
            Self::Seconds => xtce::TimeUnitsType::Seconds,
            Self::Milliseconds => xtce::TimeUnitsType::Milliseconds,
            Self::Microseconds => xtce::TimeUnitsType::Microseconds,
            Self::Nanoseconds => xtce::TimeUnitsType::Nanoseconds,
            Self::Picoseconds => xtce::TimeUnitsType::Picoseconds,
            Self::Minutes => xtce::TimeUnitsType::Minutes,
            Self::Hours => xtce::TimeUnitsType::Hours,
            Self::Days => xtce::TimeUnitsType::Days,
            Self::Months => xtce::TimeUnitsType::Months,
            Self::Years => xtce::TimeUnitsType::Years,
        }
    }
}

fn encoding_ref(content: &xtce::EncodingTypeContent) -> DataEncodingRef<'_> {
    match content {
        xtce::EncodingTypeContent::BinaryDataEncoding(value) => DataEncodingRef::Binary(value),
        xtce::EncodingTypeContent::FloatDataEncoding(value) => DataEncodingRef::Float(value),
        xtce::EncodingTypeContent::IntegerDataEncoding(value) => DataEncodingRef::Integer(value),
        xtce::EncodingTypeContent::StringDataEncoding(value) => DataEncodingRef::String(value),
    }
}

fn encoding_mut(content: &mut xtce::EncodingTypeContent) -> DataEncodingMut<'_> {
    match content {
        xtce::EncodingTypeContent::BinaryDataEncoding(value) => DataEncodingMut::Binary(value),
        xtce::EncodingTypeContent::FloatDataEncoding(value) => DataEncodingMut::Float(value),
        xtce::EncodingTypeContent::IntegerDataEncoding(value) => DataEncodingMut::Integer(value),
        xtce::EncodingTypeContent::StringDataEncoding(value) => DataEncodingMut::String(value),
    }
}

fn default_content(kind: DataEncodingKind) -> xtce::EncodingTypeContent {
    match kind {
        DataEncodingKind::Binary => {
            xtce::EncodingTypeContent::BinaryDataEncoding(default_binary_encoding())
        }
        DataEncodingKind::Float => {
            xtce::EncodingTypeContent::FloatDataEncoding(default_float_encoding())
        }
        DataEncodingKind::Integer => {
            xtce::EncodingTypeContent::IntegerDataEncoding(default_integer_encoding())
        }
        DataEncodingKind::String => {
            xtce::EncodingTypeContent::StringDataEncoding(default_string_encoding())
        }
    }
}

fn default_binary_encoding() -> xtce::BinaryDataEncodingType {
    xtce::BinaryDataEncodingType {
        bit_order: xtce::BinaryDataEncodingType::default_bit_order(),
        byte_order: xtce::BinaryDataEncodingType::default_byte_order(),
        error_detect_correct: None,
        size_in_bits: xtce::IntegerValueType::FixedValue(8),
        from_binary_transform_algorithm: None,
        to_binary_transform_algorithm: None,
    }
}

fn default_float_encoding() -> xtce::FloatDataEncodingType {
    xtce::FloatDataEncodingType {
        bit_order: xtce::FloatDataEncodingType::default_bit_order(),
        byte_order: xtce::FloatDataEncodingType::default_byte_order(),
        encoding: xtce::FloatDataEncodingType::default_encoding(),
        size_in_bits: xtce::FloatDataEncodingType::default_size_in_bits(),
        change_threshold: None,
        error_detect_correct: None,
        default_calibrator: None,
        context_calibrator_list: None,
    }
}

fn default_integer_encoding() -> xtce::IntegerDataEncodingType {
    xtce::IntegerDataEncodingType {
        bit_order: xtce::IntegerDataEncodingType::default_bit_order(),
        byte_order: xtce::IntegerDataEncodingType::default_byte_order(),
        encoding: xtce::IntegerDataEncodingType::default_encoding(),
        size_in_bits: xtce::IntegerDataEncodingType::default_size_in_bits(),
        change_threshold: None,
        error_detect_correct: None,
        default_calibrator: None,
        context_calibrator_list: None,
    }
}

fn default_string_encoding() -> xtce::StringDataEncodingType {
    xtce::StringDataEncodingType {
        bit_order: xtce::StringDataEncodingType::default_bit_order(),
        byte_order: xtce::StringDataEncodingType::default_byte_order(),
        encoding: xtce::StringDataEncodingType::default_encoding(),
        content: vec![xtce::StringDataEncodingTypeContent::SizeInBits(
            xtce::SizeInBitsType {
                fixed: xtce::SizeInBitsTypeFixedElementType { fixed_value: 8 },
                termination_char: None,
                leading_size: None,
            },
        )],
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
    super::select_field(label, "Required", select)
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn parse_or<T: std::str::FromStr>(value: &str, fallback: T) -> T {
    value.trim().parse().unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{TimeEncodingValues, TimeUnitsChoice};

    #[test]
    fn time_encoding_attributes_round_trip() {
        let encoding = xtce::EncodingType {
            units: xtce::TimeUnitsType::Microseconds,
            scale: 0.5,
            offset: -12.0,
            content: xtce::EncodingTypeContent::IntegerDataEncoding(
                super::default_integer_encoding(),
            ),
        };

        let values = TimeEncodingValues::from_encoding(Some(&encoding));
        assert_eq!(values.units, TimeUnitsChoice::Microseconds);
        assert_eq!(values.scale, "0.5");
        assert_eq!(values.offset, "-12");
    }
}
