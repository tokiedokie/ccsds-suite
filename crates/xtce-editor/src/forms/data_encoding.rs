use std::{cell::Cell, rc::Rc};

use gpui::{App, AppContext, Context, Div, Entity, ParentElement, Styled, Window, div};
use gpui_component::{
    IndexPath, StyledExt, h_flex,
    input::InputState,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};

use super::field;
use crate::XtceEditor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DataEncodingKind {
    Binary,
    Float,
    Integer,
    String,
}

impl DataEncodingKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Binary => "BinaryDataEncoding",
            Self::Float => "FloatDataEncoding",
            Self::Integer => "IntegerDataEncoding",
            Self::String => "StringDataEncoding",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "BinaryDataEncoding" => Some(Self::Binary),
            "FloatDataEncoding" => Some(Self::Float),
            "IntegerDataEncoding" => Some(Self::Integer),
            "StringDataEncoding" => Some(Self::String),
            _ => None,
        }
    }

    fn option_index(self) -> usize {
        match self {
            Self::Binary => 1,
            Self::Float => 2,
            Self::Integer => 3,
            Self::String => 4,
        }
    }
}

const DATA_ENCODING_OPTIONS: [&str; 5] = [
    "None",
    "BinaryDataEncoding",
    "FloatDataEncoding",
    "IntegerDataEncoding",
    "StringDataEncoding",
];

pub(super) struct DataEncodingForm {
    kind: Rc<Cell<Option<DataEncodingKind>>>,
    kind_select: Entity<SelectState<Vec<&'static str>>>,
    bit_order_input: Entity<InputState>,
    byte_order_input: Entity<InputState>,
    encoding_input: Entity<InputState>,
    size_in_bits_input: Entity<InputState>,
    change_threshold_input: Entity<InputState>,
}

impl DataEncodingForm {
    pub(super) fn new(
        encoding: Option<DataEncodingRef<'_>>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        let values = DataEncodingValues::from_encoding(encoding);
        let kind = encoding.map(|encoding| encoding.kind());
        let kind_state = Rc::new(Cell::new(kind));
        let kind_select = cx.new(|cx| {
            SelectState::new(
                DATA_ENCODING_OPTIONS.to_vec(),
                Some(IndexPath::default().row(kind.map_or(0, DataEncodingKind::option_index))),
                window,
                cx,
            )
        });
        let bit_order_input = input(&values.bit_order, window, cx);
        let byte_order_input = input(&values.byte_order, window, cx);
        let encoding_input = input(&values.encoding, window, cx);
        let size_in_bits_input = input(&values.size_in_bits, window, cx);
        let change_threshold_input = input(&values.change_threshold, window, cx);

        let inputs = [
            bit_order_input.clone(),
            byte_order_input.clone(),
            encoding_input.clone(),
            size_in_bits_input.clone(),
            change_threshold_input.clone(),
        ];
        let subscription_kind = kind_state.clone();
        cx.subscribe_in(
            &kind_select,
            window,
            move |_, _, event: &SelectEvent<Vec<&'static str>>, window, cx| {
                let SelectEvent::Confirm(Some(label)) = event else {
                    return;
                };
                let selected_kind = DataEncodingKind::from_label(label);
                if subscription_kind.replace(selected_kind) == selected_kind {
                    return;
                }
                let values = DataEncodingValues::defaults(selected_kind);
                for (input, value) in inputs.iter().zip(values.into_fields()) {
                    input.update(cx, |input, cx| input.set_value(value, window, cx));
                }
                cx.notify();
            },
        )
        .detach();

        Self {
            kind: kind_state,
            kind_select,
            bit_order_input,
            byte_order_input,
            encoding_input,
            size_in_bits_input,
            change_threshold_input,
        }
    }

    pub(super) fn load(
        &self,
        encoding: Option<DataEncodingRef<'_>>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        let values = DataEncodingValues::from_encoding(encoding);
        self.kind.set(encoding.map(|encoding| encoding.kind()));
        let label = encoding
            .map(|encoding| encoding.kind().label())
            .unwrap_or("None");
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&label, window, cx);
        });
        for (input, value) in [
            (&self.bit_order_input, values.bit_order),
            (&self.byte_order_input, values.byte_order),
            (&self.encoding_input, values.encoding),
            (&self.size_in_bits_input, values.size_in_bits),
            (&self.change_threshold_input, values.change_threshold),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
    }

    pub(super) fn selected_kind(&self, cx: &App) -> Option<DataEncodingKind> {
        self.kind_select
            .read(cx)
            .selected_value()
            .copied()
            .and_then(DataEncodingKind::from_label)
    }

    pub(super) fn apply_to(&self, encoding: DataEncodingMut<'_>, cx: &App) {
        DataEncodingValues {
            bit_order: value(&self.bit_order_input, cx),
            byte_order: value(&self.byte_order_input, cx),
            encoding: value(&self.encoding_input, cx),
            size_in_bits: value(&self.size_in_bits_input, cx),
            change_threshold: value(&self.change_threshold_input, cx),
        }
        .apply_to(encoding);
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        let kind = self.selected_kind(cx);
        let mut form = v_flex().gap_5().child(
            v_flex()
                .gap_2()
                .child(div().text_sm().font_medium().child("Data encoding"))
                .child(Select::new(&self.kind_select).w_full()),
        );
        let Some(kind) = kind else {
            return form;
        };
        form = form.child(
            h_flex()
                .gap_4()
                .items_start()
                .child(field(
                    "Bit order",
                    "mostSignificantBitFirst or leastSignificantBitFirst",
                    &self.bit_order_input,
                    cx,
                ))
                .child(field(
                    "Byte order",
                    "mostSignificantByteFirst, leastSignificantByteFirst, or a custom value",
                    &self.byte_order_input,
                    cx,
                )),
        );
        if kind != DataEncodingKind::Binary {
            form = form.child(field(
                "Encoding",
                encoding_hint(kind),
                &self.encoding_input,
                cx,
            ));
        }
        form = form.child(
            h_flex()
                .gap_4()
                .items_start()
                .child(field(
                    "Size in bits",
                    if kind == DataEncodingKind::Binary {
                        "Fixed size only; dynamic size expressions are preserved"
                    } else {
                        "Required"
                    },
                    &self.size_in_bits_input,
                    cx,
                ))
                .child(field(
                    "Change threshold",
                    if matches!(kind, DataEncodingKind::Float | DataEncodingKind::Integer) {
                        "Optional"
                    } else {
                        "Not used by this encoding"
                    },
                    &self.change_threshold_input,
                    cx,
                )),
        );
        form
    }
}

#[derive(Clone, Copy)]
pub(super) enum DataEncodingRef<'a> {
    Binary(&'a xtce::BinaryDataEncodingType),
    Float(&'a xtce::FloatDataEncodingType),
    Integer(&'a xtce::IntegerDataEncodingType),
    String(&'a xtce::StringDataEncodingType),
}

impl DataEncodingRef<'_> {
    fn kind(self) -> DataEncodingKind {
        match self {
            Self::Binary(_) => DataEncodingKind::Binary,
            Self::Float(_) => DataEncodingKind::Float,
            Self::Integer(_) => DataEncodingKind::Integer,
            Self::String(_) => DataEncodingKind::String,
        }
    }
}

pub(super) enum DataEncodingMut<'a> {
    Binary(&'a mut xtce::BinaryDataEncodingType),
    Float(&'a mut xtce::FloatDataEncodingType),
    Integer(&'a mut xtce::IntegerDataEncodingType),
    String(&'a mut xtce::StringDataEncodingType),
}

pub(super) fn find_data_encoding(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<DataEncodingRef<'_>> {
    macro_rules! find {
        ($content:expr, $enum_type:ident) => {
            $content.iter().find_map(|item| match item {
                xtce::$enum_type::BinaryDataEncoding(value) => Some(DataEncodingRef::Binary(value)),
                xtce::$enum_type::FloatDataEncoding(value) => Some(DataEncodingRef::Float(value)),
                xtce::$enum_type::IntegerDataEncoding(value) => {
                    Some(DataEncodingRef::Integer(value))
                }
                xtce::$enum_type::StringDataEncoding(value) => Some(DataEncodingRef::String(value)),
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            find!(value.content, StringParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            find!(value.content, EnumeratedParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            find!(value.content, IntegerParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            find!(value.content, BinaryParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            find!(value.content, FloatParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            find!(value.content, BooleanParameterTypeContent)
        }
        _ => None,
    }
}

pub(super) fn find_data_encoding_mut(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
) -> Option<DataEncodingMut<'_>> {
    macro_rules! find {
        ($content:expr, $enum_type:ident) => {
            $content.iter_mut().find_map(|item| match item {
                xtce::$enum_type::BinaryDataEncoding(value) => Some(DataEncodingMut::Binary(value)),
                xtce::$enum_type::FloatDataEncoding(value) => Some(DataEncodingMut::Float(value)),
                xtce::$enum_type::IntegerDataEncoding(value) => {
                    Some(DataEncodingMut::Integer(value))
                }
                xtce::$enum_type::StringDataEncoding(value) => Some(DataEncodingMut::String(value)),
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            find!(value.content, StringParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            find!(value.content, EnumeratedParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            find!(value.content, IntegerParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            find!(value.content, BinaryParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            find!(value.content, FloatParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            find!(value.content, BooleanParameterTypeContent)
        }
        _ => None,
    }
}

pub(super) fn set_data_encoding_kind(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    kind: Option<DataEncodingKind>,
) {
    macro_rules! set_kind {
        ($content:expr, $enum_type:ident) => {{
            let current_kind = $content.iter().find_map(|item| match item {
                xtce::$enum_type::BinaryDataEncoding(_) => Some(DataEncodingKind::Binary),
                xtce::$enum_type::FloatDataEncoding(_) => Some(DataEncodingKind::Float),
                xtce::$enum_type::IntegerDataEncoding(_) => Some(DataEncodingKind::Integer),
                xtce::$enum_type::StringDataEncoding(_) => Some(DataEncodingKind::String),
                _ => None,
            });
            if current_kind != kind {
                $content.retain(|item| {
                    !matches!(
                        item,
                        xtce::$enum_type::BinaryDataEncoding(_)
                            | xtce::$enum_type::FloatDataEncoding(_)
                            | xtce::$enum_type::IntegerDataEncoding(_)
                            | xtce::$enum_type::StringDataEncoding(_)
                    )
                });
                if let Some(kind) = kind {
                    let index = $content
                        .iter()
                        .position(|item| {
                            !matches!(
                                item,
                                xtce::$enum_type::LongDescription(_)
                                    | xtce::$enum_type::AliasSet(_)
                                    | xtce::$enum_type::AncillaryDataSet(_)
                                    | xtce::$enum_type::UnitSet(_)
                            )
                        })
                        .unwrap_or($content.len());
                    let encoding = match kind {
                        DataEncodingKind::Binary => {
                            xtce::$enum_type::BinaryDataEncoding(default_binary_encoding())
                        }
                        DataEncodingKind::Float => {
                            xtce::$enum_type::FloatDataEncoding(default_float_encoding())
                        }
                        DataEncodingKind::Integer => {
                            xtce::$enum_type::IntegerDataEncoding(default_integer_encoding())
                        }
                        DataEncodingKind::String => {
                            xtce::$enum_type::StringDataEncoding(default_string_encoding())
                        }
                    };
                    $content.insert(index, encoding);
                }
            }
        }};
    }

    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            set_kind!(value.content, StringParameterTypeContent);
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            set_kind!(value.content, EnumeratedParameterTypeContent);
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            set_kind!(value.content, IntegerParameterTypeContent);
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            set_kind!(value.content, BinaryParameterTypeContent);
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            set_kind!(value.content, FloatParameterTypeContent);
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            set_kind!(value.content, BooleanParameterTypeContent);
        }
        _ => {}
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

struct DataEncodingValues {
    bit_order: String,
    byte_order: String,
    encoding: String,
    size_in_bits: String,
    change_threshold: String,
}

impl DataEncodingValues {
    fn defaults(kind: Option<DataEncodingKind>) -> Self {
        match kind {
            Some(DataEncodingKind::Binary) => {
                let value = default_binary_encoding();
                Self::from_encoding(Some(DataEncodingRef::Binary(&value)))
            }
            Some(DataEncodingKind::Float) => {
                let value = default_float_encoding();
                Self::from_encoding(Some(DataEncodingRef::Float(&value)))
            }
            Some(DataEncodingKind::Integer) => {
                let value = default_integer_encoding();
                Self::from_encoding(Some(DataEncodingRef::Integer(&value)))
            }
            Some(DataEncodingKind::String) => {
                let value = default_string_encoding();
                Self::from_encoding(Some(DataEncodingRef::String(&value)))
            }
            None => Self::from_encoding(None),
        }
    }

    fn into_fields(self) -> [String; 5] {
        [
            self.bit_order,
            self.byte_order,
            self.encoding,
            self.size_in_bits,
            self.change_threshold,
        ]
    }

    fn from_encoding(encoding: Option<DataEncodingRef<'_>>) -> Self {
        match encoding {
            Some(DataEncodingRef::Binary(value)) => Self::new(
                &value.bit_order,
                &value.byte_order,
                "",
                &integer_value_label(&value.size_in_bits),
                "",
            ),
            Some(DataEncodingRef::Float(value)) => Self::new(
                &value.bit_order,
                &value.byte_order,
                float_encoding_label(&value.encoding),
                float_size_label(&value.size_in_bits),
                value
                    .change_threshold
                    .map(|value| value.to_string())
                    .as_deref()
                    .unwrap_or_default(),
            ),
            Some(DataEncodingRef::Integer(value)) => Self::new(
                &value.bit_order,
                &value.byte_order,
                integer_encoding_label(&value.encoding),
                &value.size_in_bits.to_string(),
                value
                    .change_threshold
                    .map(|value| value.to_string())
                    .as_deref()
                    .unwrap_or_default(),
            ),
            Some(DataEncodingRef::String(value)) => Self::new(
                &value.bit_order,
                &value.byte_order,
                string_encoding_label(&value.encoding),
                &value
                    .content
                    .iter()
                    .find_map(|item| match item {
                        xtce::StringDataEncodingTypeContent::SizeInBits(size) => {
                            Some(size.fixed.fixed_value.to_string())
                        }
                        _ => None,
                    })
                    .unwrap_or_default(),
                "",
            ),
            None => Self::new(
                &xtce::BitOrderType::MostSignificantBitFirst,
                &xtce::ByteOrderType::MostSignificantByteFirst,
                "",
                "",
                "",
            ),
        }
    }

    fn new(
        bit_order: &xtce::BitOrderType,
        byte_order: &xtce::ByteOrderType,
        encoding: &str,
        size_in_bits: &str,
        change_threshold: &str,
    ) -> Self {
        Self {
            bit_order: bit_order_label(bit_order).to_owned(),
            byte_order: byte_order_label(byte_order),
            encoding: encoding.to_owned(),
            size_in_bits: size_in_bits.to_owned(),
            change_threshold: change_threshold.to_owned(),
        }
    }

    fn apply_to(&self, encoding: DataEncodingMut<'_>) {
        match encoding {
            DataEncodingMut::Binary(value) => {
                apply_orders(&mut value.bit_order, &mut value.byte_order, self);
                if let (xtce::IntegerValueType::FixedValue(size), Ok(new_size)) =
                    (&mut value.size_in_bits, self.size_in_bits.parse())
                {
                    *size = new_size;
                }
            }
            DataEncodingMut::Float(value) => {
                apply_orders(&mut value.bit_order, &mut value.byte_order, self);
                if let Some(encoding) = float_encoding_from_str(&self.encoding) {
                    value.encoding = encoding;
                }
                if let Some(size) = float_size_from_str(&self.size_in_bits) {
                    value.size_in_bits = size;
                }
                value.change_threshold = self.change_threshold.parse().ok();
            }
            DataEncodingMut::Integer(value) => {
                apply_orders(&mut value.bit_order, &mut value.byte_order, self);
                if let Some(encoding) = integer_encoding_from_str(&self.encoding) {
                    value.encoding = encoding;
                }
                if let Ok(size) = self.size_in_bits.parse() {
                    value.size_in_bits = size;
                }
                value.change_threshold = self.change_threshold.parse().ok();
            }
            DataEncodingMut::String(value) => {
                apply_orders(&mut value.bit_order, &mut value.byte_order, self);
                if let Some(encoding) = string_encoding_from_str(&self.encoding) {
                    value.encoding = encoding;
                }
                if let Ok(size_in_bits) = self.size_in_bits.parse()
                    && let Some(size) = value.content.iter_mut().find_map(|item| match item {
                        xtce::StringDataEncodingTypeContent::SizeInBits(size) => Some(size),
                        _ => None,
                    })
                {
                    size.fixed.fixed_value = size_in_bits;
                }
            }
        }
    }
}

fn apply_orders(
    bit_order: &mut xtce::BitOrderType,
    byte_order: &mut xtce::ByteOrderType,
    values: &DataEncodingValues,
) {
    if let Some(value) = bit_order_from_str(&values.bit_order) {
        *bit_order = value;
    }
    *byte_order = byte_order_from_str(&values.byte_order);
}

fn bit_order_label(value: &xtce::BitOrderType) -> &'static str {
    match value {
        xtce::BitOrderType::LeastSignificantBitFirst => "leastSignificantBitFirst",
        xtce::BitOrderType::MostSignificantBitFirst => "mostSignificantBitFirst",
    }
}

fn bit_order_from_str(value: &str) -> Option<xtce::BitOrderType> {
    match value.trim() {
        "leastSignificantBitFirst" => Some(xtce::BitOrderType::LeastSignificantBitFirst),
        "mostSignificantBitFirst" => Some(xtce::BitOrderType::MostSignificantBitFirst),
        _ => None,
    }
}

fn byte_order_label(value: &xtce::ByteOrderType) -> String {
    match value {
        xtce::ByteOrderType::MostSignificantByteFirst => "mostSignificantByteFirst".to_owned(),
        xtce::ByteOrderType::LeastSignificantByteFirst => "leastSignificantByteFirst".to_owned(),
        xtce::ByteOrderType::String(value) => value.clone(),
    }
}

fn byte_order_from_str(value: &str) -> xtce::ByteOrderType {
    match value.trim() {
        "mostSignificantByteFirst" => xtce::ByteOrderType::MostSignificantByteFirst,
        "leastSignificantByteFirst" => xtce::ByteOrderType::LeastSignificantByteFirst,
        value => xtce::ByteOrderType::String(value.to_owned()),
    }
}

fn integer_value_label(value: &xtce::IntegerValueType) -> String {
    match value {
        xtce::IntegerValueType::FixedValue(value) => value.to_string(),
        xtce::IntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::IntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
    }
}

fn encoding_hint(kind: DataEncodingKind) -> &'static str {
    match kind {
        DataEncodingKind::Binary => "Not applicable",
        DataEncodingKind::Float => "IEEE754_1985, IEEE754, MILSTD_1750A, DEC, IBM, or TI",
        DataEncodingKind::Integer => {
            "unsigned, signMagnitude, twosComplement, onesComplement, BCD, or packedBCD"
        }
        DataEncodingKind::String => "US-ASCII, ISO-8859-1, Windows-1252, UTF-8, UTF-16, or UTF-32",
    }
}

fn integer_encoding_label(value: &xtce::IntegerEncodingType) -> &'static str {
    match value {
        xtce::IntegerEncodingType::Unsigned => "unsigned",
        xtce::IntegerEncodingType::SignMagnitude => "signMagnitude",
        xtce::IntegerEncodingType::TwosComplement => "twosComplement",
        xtce::IntegerEncodingType::OnesComplement => "onesComplement",
        xtce::IntegerEncodingType::Bcd => "BCD",
        xtce::IntegerEncodingType::PackedBcd => "packedBCD",
    }
}

fn integer_encoding_from_str(value: &str) -> Option<xtce::IntegerEncodingType> {
    match value.trim() {
        "unsigned" => Some(xtce::IntegerEncodingType::Unsigned),
        "signMagnitude" => Some(xtce::IntegerEncodingType::SignMagnitude),
        "twosComplement" => Some(xtce::IntegerEncodingType::TwosComplement),
        "onesComplement" => Some(xtce::IntegerEncodingType::OnesComplement),
        "BCD" => Some(xtce::IntegerEncodingType::Bcd),
        "packedBCD" => Some(xtce::IntegerEncodingType::PackedBcd),
        _ => None,
    }
}

fn float_encoding_label(value: &xtce::FloatEncodingType) -> &'static str {
    match value {
        xtce::FloatEncodingType::Ieee7541985 => "IEEE754_1985",
        xtce::FloatEncodingType::Ieee754 => "IEEE754",
        xtce::FloatEncodingType::Milstd1750A => "MILSTD_1750A",
        xtce::FloatEncodingType::Dec => "DEC",
        xtce::FloatEncodingType::Ibm => "IBM",
        xtce::FloatEncodingType::Ti => "TI",
    }
}

fn float_encoding_from_str(value: &str) -> Option<xtce::FloatEncodingType> {
    match value.trim() {
        "IEEE754_1985" => Some(xtce::FloatEncodingType::Ieee7541985),
        "IEEE754" => Some(xtce::FloatEncodingType::Ieee754),
        "MILSTD_1750A" => Some(xtce::FloatEncodingType::Milstd1750A),
        "DEC" => Some(xtce::FloatEncodingType::Dec),
        "IBM" => Some(xtce::FloatEncodingType::Ibm),
        "TI" => Some(xtce::FloatEncodingType::Ti),
        _ => None,
    }
}

fn float_size_label(value: &xtce::FloatEncodingSizeInBitsType) -> &'static str {
    match value {
        xtce::FloatEncodingSizeInBitsType::_16 => "16",
        xtce::FloatEncodingSizeInBitsType::_32 => "32",
        xtce::FloatEncodingSizeInBitsType::_40 => "40",
        xtce::FloatEncodingSizeInBitsType::_48 => "48",
        xtce::FloatEncodingSizeInBitsType::_64 => "64",
        xtce::FloatEncodingSizeInBitsType::_80 => "80",
        xtce::FloatEncodingSizeInBitsType::_128 => "128",
    }
}

fn float_size_from_str(value: &str) -> Option<xtce::FloatEncodingSizeInBitsType> {
    match value.trim() {
        "16" => Some(xtce::FloatEncodingSizeInBitsType::_16),
        "32" => Some(xtce::FloatEncodingSizeInBitsType::_32),
        "40" => Some(xtce::FloatEncodingSizeInBitsType::_40),
        "48" => Some(xtce::FloatEncodingSizeInBitsType::_48),
        "64" => Some(xtce::FloatEncodingSizeInBitsType::_64),
        "80" => Some(xtce::FloatEncodingSizeInBitsType::_80),
        "128" => Some(xtce::FloatEncodingSizeInBitsType::_128),
        _ => None,
    }
}

fn string_encoding_label(value: &xtce::StringEncodingType) -> &'static str {
    match value {
        xtce::StringEncodingType::UsAscii => "US-ASCII",
        xtce::StringEncodingType::Iso88591 => "ISO-8859-1",
        xtce::StringEncodingType::Windows1252 => "Windows-1252",
        xtce::StringEncodingType::Utf8 => "UTF-8",
        xtce::StringEncodingType::Utf16 => "UTF-16",
        xtce::StringEncodingType::Utf16Le => "UTF-16LE",
        xtce::StringEncodingType::Utf16Be => "UTF-16BE",
        xtce::StringEncodingType::Utf32 => "UTF-32",
        xtce::StringEncodingType::Utf32Le => "UTF-32LE",
        xtce::StringEncodingType::Utf32Be => "UTF-32BE",
    }
}

fn string_encoding_from_str(value: &str) -> Option<xtce::StringEncodingType> {
    match value.trim() {
        "US-ASCII" => Some(xtce::StringEncodingType::UsAscii),
        "ISO-8859-1" => Some(xtce::StringEncodingType::Iso88591),
        "Windows-1252" => Some(xtce::StringEncodingType::Windows1252),
        "UTF-8" => Some(xtce::StringEncodingType::Utf8),
        "UTF-16" => Some(xtce::StringEncodingType::Utf16),
        "UTF-16LE" => Some(xtce::StringEncodingType::Utf16Le),
        "UTF-16BE" => Some(xtce::StringEncodingType::Utf16Be),
        "UTF-32" => Some(xtce::StringEncodingType::Utf32),
        "UTF-32LE" => Some(xtce::StringEncodingType::Utf32Le),
        "UTF-32BE" => Some(xtce::StringEncodingType::Utf32Be),
        _ => None,
    }
}

fn input(value: &str, window: &mut Window, cx: &mut Context<XtceEditor>) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        DataEncodingKind, DataEncodingMut, DataEncodingValues, default_integer_encoding,
        set_data_encoding_kind,
    };

    #[test]
    fn applying_integer_encoding_values_preserves_nested_configuration() {
        let mut encoding = xtce::IntegerDataEncodingType {
            bit_order: xtce::BitOrderType::MostSignificantBitFirst,
            byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
            encoding: xtce::IntegerEncodingType::Unsigned,
            size_in_bits: 8,
            change_threshold: None,
            error_detect_correct: Some(xtce::ErrorDetectCorrectType {
                content: Vec::new(),
            }),
            default_calibrator: None,
            context_calibrator_list: None,
        };
        DataEncodingValues {
            bit_order: "leastSignificantBitFirst".to_owned(),
            byte_order: "leastSignificantByteFirst".to_owned(),
            encoding: "twosComplement".to_owned(),
            size_in_bits: "16".to_owned(),
            change_threshold: "2".to_owned(),
        }
        .apply_to(DataEncodingMut::Integer(&mut encoding));

        assert!(matches!(
            encoding.bit_order,
            xtce::BitOrderType::LeastSignificantBitFirst
        ));
        assert!(matches!(
            encoding.encoding,
            xtce::IntegerEncodingType::TwosComplement
        ));
        assert_eq!(encoding.size_in_bits, 16);
        assert_eq!(encoding.change_threshold, Some(2));
        assert!(encoding.error_detect_correct.is_some());
    }

    #[test]
    fn applying_binary_size_updates_only_a_fixed_size() {
        let mut encoding = xtce::BinaryDataEncodingType {
            bit_order: xtce::BitOrderType::MostSignificantBitFirst,
            byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
            error_detect_correct: None,
            size_in_bits: xtce::IntegerValueType::FixedValue(8),
            from_binary_transform_algorithm: None,
            to_binary_transform_algorithm: None,
        };
        DataEncodingValues {
            bit_order: "mostSignificantBitFirst".to_owned(),
            byte_order: "mostSignificantByteFirst".to_owned(),
            encoding: String::new(),
            size_in_bits: "32".to_owned(),
            change_threshold: String::new(),
        }
        .apply_to(DataEncodingMut::Binary(&mut encoding));

        assert!(matches!(
            encoding.size_in_bits,
            xtce::IntegerValueType::FixedValue(32)
        ));
    }

    #[test]
    fn changing_the_encoding_kind_preserves_other_parameter_type_content() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: vec![
                    xtce::IntegerParameterTypeContent::AncillaryDataSet(
                        xtce::AncillaryDataSetType {
                            ancillary_data: Vec::new(),
                        },
                    ),
                    xtce::IntegerParameterTypeContent::IntegerDataEncoding(
                        default_integer_encoding(),
                    ),
                ],
            });

        set_data_encoding_kind(&mut parameter_type, Some(DataEncodingKind::String));

        let xtce::ParameterTypeSetTypeContent::IntegerParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected an IntegerParameterType");
        };
        assert!(matches!(
            parameter_type.content[0],
            xtce::IntegerParameterTypeContent::AncillaryDataSet(_)
        ));
        let xtce::IntegerParameterTypeContent::StringDataEncoding(encoding) =
            &parameter_type.content[1]
        else {
            panic!("expected a StringDataEncoding");
        };
        assert!(matches!(
            encoding.content.first(),
            Some(xtce::StringDataEncodingTypeContent::SizeInBits(_))
        ));
    }

    #[test]
    fn selecting_none_removes_only_the_data_encoding() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: vec![
                    xtce::IntegerParameterTypeContent::AncillaryDataSet(
                        xtce::AncillaryDataSetType {
                            ancillary_data: Vec::new(),
                        },
                    ),
                    xtce::IntegerParameterTypeContent::IntegerDataEncoding(
                        default_integer_encoding(),
                    ),
                ],
            });

        set_data_encoding_kind(&mut parameter_type, None);

        let xtce::ParameterTypeSetTypeContent::IntegerParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected an IntegerParameterType");
        };
        assert_eq!(parameter_type.content.len(), 1);
        assert!(matches!(
            parameter_type.content[0],
            xtce::IntegerParameterTypeContent::AncillaryDataSet(_)
        ));
    }
}
