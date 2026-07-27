use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div,
};
use gpui_component::{
    IndexPath, StyledExt, h_flex,
    input::InputState,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    default_calibrator::DefaultCalibratorForm, discrete_lookup::DiscreteLookupListForm,
    dynamic_value::DynamicValueForm, error_detect_correct::ErrorDetectCorrectForm, field,
    impl_select_item,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DataEncodingKind {
    Binary,
    Float,
    Integer,
    String,
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum DataEncodingChoice {
    None,
    #[strum(serialize = "BinaryDataEncoding")]
    Binary,
    #[strum(serialize = "FloatDataEncoding")]
    Float,
    #[strum(serialize = "IntegerDataEncoding")]
    Integer,
    #[strum(serialize = "StringDataEncoding")]
    String,
}
impl_select_item!(DataEncodingChoice);

impl DataEncodingChoice {
    fn from_kind(kind: Option<DataEncodingKind>) -> Self {
        match kind {
            None => Self::None,
            Some(DataEncodingKind::Binary) => Self::Binary,
            Some(DataEncodingKind::Float) => Self::Float,
            Some(DataEncodingKind::Integer) => Self::Integer,
            Some(DataEncodingKind::String) => Self::String,
        }
    }

    fn kind(self) -> Option<DataEncodingKind> {
        match self {
            Self::None => None,
            Self::Binary => Some(DataEncodingKind::Binary),
            Self::Float => Some(DataEncodingKind::Float),
            Self::Integer => Some(DataEncodingKind::Integer),
            Self::String => Some(DataEncodingKind::String),
        }
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BitOrderChoice {
    #[strum(serialize = "mostSignificantBitFirst")]
    MostSignificantFirst,
    #[strum(serialize = "leastSignificantBitFirst")]
    LeastSignificantFirst,
}
impl_select_item!(BitOrderChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ByteOrderChoice {
    #[strum(serialize = "mostSignificantByteFirst")]
    MostSignificantFirst,
    #[strum(serialize = "leastSignificantByteFirst")]
    LeastSignificantFirst,
}
impl_select_item!(ByteOrderChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum FloatEncodingChoice {
    #[strum(serialize = "IEEE754_1985")]
    Ieee7541985,
    #[strum(serialize = "IEEE754")]
    Ieee754,
    #[strum(serialize = "MILSTD_1750A")]
    Milstd1750A,
    #[strum(serialize = "DEC")]
    Dec,
    #[strum(serialize = "IBM")]
    Ibm,
    #[strum(serialize = "TI")]
    Ti,
}
impl_select_item!(FloatEncodingChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum IntegerEncodingChoice {
    #[strum(serialize = "unsigned")]
    Unsigned,
    #[strum(serialize = "signMagnitude")]
    SignMagnitude,
    #[strum(serialize = "twosComplement")]
    TwosComplement,
    #[strum(serialize = "onesComplement")]
    OnesComplement,
    #[strum(serialize = "BCD")]
    Bcd,
    #[strum(serialize = "packedBCD")]
    PackedBcd,
}
impl_select_item!(IntegerEncodingChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum StringEncodingChoice {
    #[strum(serialize = "US-ASCII")]
    UsAscii,
    #[strum(serialize = "ISO-8859-1")]
    Iso88591,
    #[strum(serialize = "Windows-1252")]
    Windows1252,
    #[strum(serialize = "UTF-8")]
    Utf8,
    #[strum(serialize = "UTF-16")]
    Utf16,
    #[strum(serialize = "UTF-16LE")]
    Utf16Le,
    #[strum(serialize = "UTF-16BE")]
    Utf16Be,
    #[strum(serialize = "UTF-32")]
    Utf32,
    #[strum(serialize = "UTF-32LE")]
    Utf32Le,
    #[strum(serialize = "UTF-32BE")]
    Utf32Be,
}
impl_select_item!(StringEncodingChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum FloatSizeChoice {
    #[strum(serialize = "16")]
    _16,
    #[strum(serialize = "32")]
    _32,
    #[strum(serialize = "40")]
    _40,
    #[strum(serialize = "48")]
    _48,
    #[strum(serialize = "64")]
    _64,
    #[strum(serialize = "80")]
    _80,
    #[strum(serialize = "128")]
    _128,
}
impl_select_item!(FloatSizeChoice);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BinarySizeKind {
    #[default]
    Fixed,
    Dynamic,
    DiscreteLookup,
}
impl_select_item!(BinarySizeKind);

pub(super) struct DataEncodingForm {
    kind_select: Entity<SelectState<Vec<DataEncodingChoice>>>,
    bit_order_select: Entity<SelectState<Vec<BitOrderChoice>>>,
    byte_order_select: Entity<SelectState<Vec<ByteOrderChoice>>>,
    float_encoding_select: Entity<SelectState<Vec<FloatEncodingChoice>>>,
    integer_encoding_select: Entity<SelectState<Vec<IntegerEncodingChoice>>>,
    string_encoding_select: Entity<SelectState<Vec<StringEncodingChoice>>>,
    float_size_select: Entity<SelectState<Vec<FloatSizeChoice>>>,
    binary_size_kind_select: Entity<SelectState<Vec<BinarySizeKind>>>,
    size_in_bits_input: Entity<InputState>,
    binary_dynamic_size: Entity<DynamicValueForm>,
    binary_discrete_size: Entity<DiscreteLookupListForm>,
    error_detect_correct: Entity<ErrorDetectCorrectForm>,
    change_threshold_input: Entity<InputState>,
    default_calibrator: Entity<DefaultCalibratorForm>,
    _subscriptions: Vec<Subscription>,
}

impl DataEncodingForm {
    pub(super) fn new(
        encoding: Option<DataEncodingRef<'_>>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = DataEncodingValues::from_encoding(encoding);
        let kind = encoding.map(|encoding| encoding.kind());
        cx.new(move |cx| {
            let kind_choice = DataEncodingChoice::from_kind(kind);
            let kind_select = select(DataEncodingChoice::VARIANTS, kind_choice, window, cx);
            let bit_order_select = select(
                BitOrderChoice::VARIANTS,
                parse_choice(&values.bit_order, BitOrderChoice::MostSignificantFirst),
                window,
                cx,
            );
            let byte_order_select = select(
                ByteOrderChoice::VARIANTS,
                parse_choice(&values.byte_order, ByteOrderChoice::MostSignificantFirst),
                window,
                cx,
            );
            let float_encoding_select = select(
                FloatEncodingChoice::VARIANTS,
                parse_choice(&values.encoding, FloatEncodingChoice::Ieee7541985),
                window,
                cx,
            );
            let integer_encoding_select = select(
                IntegerEncodingChoice::VARIANTS,
                parse_choice(&values.encoding, IntegerEncodingChoice::Unsigned),
                window,
                cx,
            );
            let string_encoding_select = select(
                StringEncodingChoice::VARIANTS,
                parse_choice(&values.encoding, StringEncodingChoice::Utf8),
                window,
                cx,
            );
            let float_size_select = select(
                FloatSizeChoice::VARIANTS,
                parse_choice(&values.size_in_bits, FloatSizeChoice::_32),
                window,
                cx,
            );
            let binary_size_kind_select = select(
                BinarySizeKind::VARIANTS,
                binary_size_kind(encoding),
                window,
                cx,
            );
            let size_in_bits_input = input(&values.size_in_bits, window, cx);
            let binary_dynamic_size =
                DynamicValueForm::new(binary_dynamic_value(encoding), window, cx);
            let binary_discrete_size =
                DiscreteLookupListForm::new(binary_discrete_value(encoding), window, cx);
            let error_detect_correct =
                ErrorDetectCorrectForm::new(data_error_detect_correct(encoding), window, cx);
            let change_threshold_input = input(&values.change_threshold, window, cx);
            let default_calibrator = DefaultCalibratorForm::new(
                encoding.and_then(DataEncodingRef::default_calibrator),
                window,
                cx,
            );
            let kind_subscription = cx.subscribe_in(
                &kind_select,
                window,
                |this: &mut DataEncodingForm,
                 _,
                 event: &SelectEvent<Vec<DataEncodingChoice>>,
                 window,
                 cx| {
                    let SelectEvent::Confirm(selected_kind) = event;
                    let selected_kind = (*selected_kind).unwrap_or(DataEncodingChoice::None).kind();
                    let values = DataEncodingValues::defaults(selected_kind);
                    sync_select(
                        &this.bit_order_select,
                        parse_choice(&values.bit_order, BitOrderChoice::MostSignificantFirst),
                        window,
                        cx,
                    );
                    sync_select(
                        &this.byte_order_select,
                        parse_choice(&values.byte_order, ByteOrderChoice::MostSignificantFirst),
                        window,
                        cx,
                    );
                    sync_select(
                        &this.float_encoding_select,
                        parse_choice(&values.encoding, FloatEncodingChoice::Ieee7541985),
                        window,
                        cx,
                    );
                    sync_select(
                        &this.integer_encoding_select,
                        parse_choice(&values.encoding, IntegerEncodingChoice::Unsigned),
                        window,
                        cx,
                    );
                    sync_select(
                        &this.string_encoding_select,
                        parse_choice(&values.encoding, StringEncodingChoice::Utf8),
                        window,
                        cx,
                    );
                    sync_select(
                        &this.float_size_select,
                        parse_choice(&values.size_in_bits, FloatSizeChoice::_32),
                        window,
                        cx,
                    );
                    sync_select(
                        &this.binary_size_kind_select,
                        BinarySizeKind::Fixed,
                        window,
                        cx,
                    );
                    this.size_in_bits_input.update(cx, |input, cx| {
                        input.set_value(values.size_in_bits, window, cx)
                    });
                    this.change_threshold_input.update(cx, |input, cx| {
                        input.set_value(values.change_threshold, window, cx)
                    });
                    this.binary_dynamic_size
                        .update(cx, |form, cx| form.load(None, window, cx));
                    this.binary_discrete_size
                        .update(cx, |form, cx| form.load(None, window, cx));
                    this.error_detect_correct
                        .update(cx, |form, cx| form.load(None, window, cx));
                    cx.notify();
                },
            );
            let binary_size_subscription = cx.subscribe_in(
                &binary_size_kind_select,
                window,
                |_: &mut DataEncodingForm, _, _: &SelectEvent<Vec<BinarySizeKind>>, _, cx| {
                    cx.notify();
                },
            );

            Self {
                kind_select,
                bit_order_select,
                byte_order_select,
                float_encoding_select,
                integer_encoding_select,
                string_encoding_select,
                float_size_select,
                binary_size_kind_select,
                size_in_bits_input,
                binary_dynamic_size,
                binary_discrete_size,
                error_detect_correct,
                change_threshold_input,
                default_calibrator,
                _subscriptions: vec![kind_subscription, binary_size_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        encoding: Option<DataEncodingRef<'_>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = DataEncodingValues::from_encoding(encoding);
        let kind = encoding.map(|encoding| encoding.kind());
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&DataEncodingChoice::from_kind(kind), window, cx);
        });
        for (input, value) in [
            (&self.size_in_bits_input, values.size_in_bits.clone()),
            (&self.change_threshold_input, values.change_threshold),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        sync_select(
            &self.bit_order_select,
            parse_choice(&values.bit_order, BitOrderChoice::MostSignificantFirst),
            window,
            cx,
        );
        sync_select(
            &self.byte_order_select,
            parse_choice(&values.byte_order, ByteOrderChoice::MostSignificantFirst),
            window,
            cx,
        );
        sync_select(
            &self.float_encoding_select,
            parse_choice(&values.encoding, FloatEncodingChoice::Ieee7541985),
            window,
            cx,
        );
        sync_select(
            &self.integer_encoding_select,
            parse_choice(&values.encoding, IntegerEncodingChoice::Unsigned),
            window,
            cx,
        );
        sync_select(
            &self.string_encoding_select,
            parse_choice(&values.encoding, StringEncodingChoice::Utf8),
            window,
            cx,
        );
        sync_select(
            &self.float_size_select,
            parse_choice(&values.size_in_bits, FloatSizeChoice::_32),
            window,
            cx,
        );
        sync_select(
            &self.binary_size_kind_select,
            binary_size_kind(encoding),
            window,
            cx,
        );
        self.binary_dynamic_size.update(cx, |form, cx| {
            form.load(binary_dynamic_value(encoding), window, cx)
        });
        self.binary_discrete_size.update(cx, |form, cx| {
            form.load(binary_discrete_value(encoding), window, cx)
        });
        self.error_detect_correct.update(cx, |form, cx| {
            form.load(data_error_detect_correct(encoding), window, cx)
        });
        self.default_calibrator.update(cx, |form, cx| {
            form.load(
                encoding.and_then(DataEncodingRef::default_calibrator),
                window,
                cx,
            );
        });
        cx.notify();
    }

    pub(super) fn selected_kind(&self, cx: &App) -> Option<DataEncodingKind> {
        self.kind_select
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(DataEncodingChoice::None)
            .kind()
    }

    pub(super) fn apply_to(&self, mut encoding: DataEncodingMut<'_>, cx: &App) {
        if let Some(calibrator) = encoding.default_calibrator_mut() {
            self.default_calibrator.read(cx).apply_to(calibrator, cx);
        }
        apply_data_error_detect_correct(&mut encoding, self.error_detect_correct.read(cx), cx);
        if let DataEncodingMut::Binary(binary) = &mut encoding {
            match selected_value(&self.binary_size_kind_select, BinarySizeKind::Fixed, cx) {
                BinarySizeKind::Fixed => {
                    if let Ok(size) = value(&self.size_in_bits_input, cx).trim().parse() {
                        binary.size_in_bits = xtce::IntegerValueType::FixedValue(size);
                    }
                }
                BinarySizeKind::Dynamic => {
                    binary.size_in_bits = xtce::IntegerValueType::DynamicValue(
                        self.binary_dynamic_size.read(cx).value(cx),
                    );
                }
                BinarySizeKind::DiscreteLookup => {
                    binary.size_in_bits = xtce::IntegerValueType::DiscreteLookupList(
                        self.binary_discrete_size.read(cx).value(cx),
                    );
                }
            }
        }
        DataEncodingValues {
            bit_order: selected_value(
                &self.bit_order_select,
                BitOrderChoice::MostSignificantFirst,
                cx,
            )
            .to_string(),
            byte_order: selected_value(
                &self.byte_order_select,
                ByteOrderChoice::MostSignificantFirst,
                cx,
            )
            .to_string(),
            encoding: match self.selected_kind(cx) {
                Some(DataEncodingKind::Float) => selected_value(
                    &self.float_encoding_select,
                    FloatEncodingChoice::Ieee7541985,
                    cx,
                )
                .to_string(),
                Some(DataEncodingKind::Integer) => selected_value(
                    &self.integer_encoding_select,
                    IntegerEncodingChoice::Unsigned,
                    cx,
                )
                .to_string(),
                Some(DataEncodingKind::String) => {
                    selected_value(&self.string_encoding_select, StringEncodingChoice::Utf8, cx)
                        .to_string()
                }
                _ => String::new(),
            },
            size_in_bits: if self.selected_kind(cx) == Some(DataEncodingKind::Float) {
                selected_value(&self.float_size_select, FloatSizeChoice::_32, cx).to_string()
            } else {
                value(&self.size_in_bits_input, cx)
            },
            change_threshold: value(&self.change_threshold_input, cx),
        }
        .apply_to(encoding);
    }

    fn render_form(&self, cx: &App) -> Div {
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
                .child(select_field(
                    "Bit order",
                    "Required",
                    &self.bit_order_select,
                    cx,
                ))
                .child(select_field(
                    "Byte order",
                    "Required",
                    &self.byte_order_select,
                    cx,
                )),
        );
        if kind != DataEncodingKind::Binary {
            let encoding_field = match kind {
                DataEncodingKind::Float => {
                    select_field("Encoding", "Required", &self.float_encoding_select, cx)
                }
                DataEncodingKind::Integer => {
                    select_field("Encoding", "Required", &self.integer_encoding_select, cx)
                }
                DataEncodingKind::String => {
                    select_field("Encoding", "Required", &self.string_encoding_select, cx)
                }
                DataEncodingKind::Binary => unreachable!(),
            };
            form = form.child(encoding_field);
        }
        form = form.child(
            h_flex()
                .gap_4()
                .items_start()
                .child(if kind == DataEncodingKind::Float {
                    select_field("Size in bits", "Required", &self.float_size_select, cx)
                } else if kind == DataEncodingKind::Binary {
                    select_field(
                        "Size in bits",
                        "Required",
                        &self.binary_size_kind_select,
                        cx,
                    )
                } else {
                    field("Size in bits", "Required", &self.size_in_bits_input, cx)
                })
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
        if kind == DataEncodingKind::Binary {
            form = match selected_value(&self.binary_size_kind_select, BinarySizeKind::Fixed, cx) {
                BinarySizeKind::Fixed => form.child(field(
                    "Fixed size in bits",
                    "Required",
                    &self.size_in_bits_input,
                    cx,
                )),
                BinarySizeKind::Dynamic => form.child(self.binary_dynamic_size.clone()),
                BinarySizeKind::DiscreteLookup => form.child(self.binary_discrete_size.clone()),
            };
        }
        form = form.child(self.error_detect_correct.clone());
        if matches!(kind, DataEncodingKind::Float | DataEncodingKind::Integer) {
            form = form.child(self.default_calibrator.clone());
        }
        form
    }
}

impl Render for DataEncodingForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

#[derive(Clone, Copy)]
pub(super) enum DataEncodingRef<'a> {
    Binary(&'a xtce::BinaryDataEncodingType),
    Float(&'a xtce::FloatDataEncodingType),
    Integer(&'a xtce::IntegerDataEncodingType),
    String(&'a xtce::StringDataEncodingType),
}

impl<'a> DataEncodingRef<'a> {
    fn kind(self) -> DataEncodingKind {
        match self {
            Self::Binary(_) => DataEncodingKind::Binary,
            Self::Float(_) => DataEncodingKind::Float,
            Self::Integer(_) => DataEncodingKind::Integer,
            Self::String(_) => DataEncodingKind::String,
        }
    }

    fn default_calibrator(self) -> Option<&'a xtce::CalibratorType> {
        match self {
            Self::Float(value) => value.default_calibrator.as_ref(),
            Self::Integer(value) => value.default_calibrator.as_ref(),
            Self::Binary(_) | Self::String(_) => None,
        }
    }
}

fn binary_size(encoding: Option<DataEncodingRef<'_>>) -> Option<&xtce::IntegerValueType> {
    match encoding {
        Some(DataEncodingRef::Binary(value)) => Some(&value.size_in_bits),
        _ => None,
    }
}

fn binary_size_kind(encoding: Option<DataEncodingRef<'_>>) -> BinarySizeKind {
    match binary_size(encoding) {
        Some(xtce::IntegerValueType::DynamicValue(_)) => BinarySizeKind::Dynamic,
        Some(xtce::IntegerValueType::DiscreteLookupList(_)) => BinarySizeKind::DiscreteLookup,
        _ => BinarySizeKind::Fixed,
    }
}

fn binary_dynamic_value(encoding: Option<DataEncodingRef<'_>>) -> Option<&xtce::DynamicValueType> {
    match binary_size(encoding) {
        Some(xtce::IntegerValueType::DynamicValue(value)) => Some(value),
        _ => None,
    }
}

fn binary_discrete_value(
    encoding: Option<DataEncodingRef<'_>>,
) -> Option<&xtce::DiscreteLookupListType> {
    match binary_size(encoding) {
        Some(xtce::IntegerValueType::DiscreteLookupList(value)) => Some(value),
        _ => None,
    }
}

fn data_error_detect_correct(
    encoding: Option<DataEncodingRef<'_>>,
) -> Option<&xtce::ErrorDetectCorrectType> {
    match encoding {
        Some(DataEncodingRef::Binary(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::Float(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::Integer(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::String(value)) => value.content.iter().find_map(|item| match item {
            xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(value) => Some(value),
            _ => None,
        }),
        None => None,
    }
}

fn apply_data_error_detect_correct(
    encoding: &mut DataEncodingMut<'_>,
    form: &ErrorDetectCorrectForm,
    cx: &App,
) {
    let mut error_detect_correct = None;
    form.apply_to(&mut error_detect_correct, cx);
    match encoding {
        DataEncodingMut::Binary(value) => value.error_detect_correct = error_detect_correct,
        DataEncodingMut::Float(value) => value.error_detect_correct = error_detect_correct,
        DataEncodingMut::Integer(value) => value.error_detect_correct = error_detect_correct,
        DataEncodingMut::String(string) => {
            let current = string.content.iter().position(|item| {
                matches!(
                    item,
                    xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(_)
                )
            });
            match (current, error_detect_correct) {
                (Some(index), Some(error)) => {
                    string.content[index] =
                        xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(error);
                }
                (None, Some(error)) => string.content.insert(
                    0,
                    xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(error),
                ),
                (Some(index), None) => {
                    string.content.remove(index);
                }
                (None, None) => {}
            }
        }
    }
}

pub(super) enum DataEncodingMut<'a> {
    Binary(&'a mut xtce::BinaryDataEncodingType),
    Float(&'a mut xtce::FloatDataEncodingType),
    Integer(&'a mut xtce::IntegerDataEncodingType),
    String(&'a mut xtce::StringDataEncodingType),
}

impl DataEncodingMut<'_> {
    fn default_calibrator_mut(&mut self) -> Option<&mut Option<xtce::CalibratorType>> {
        match self {
            Self::Float(value) => Some(&mut value.default_calibrator),
            Self::Integer(value) => Some(&mut value.default_calibrator),
            Self::Binary(_) | Self::String(_) => None,
        }
    }
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
            byte_order: byte_order_label(byte_order).to_owned(),
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

fn byte_order_label(value: &xtce::ByteOrderType) -> &'static str {
    match value {
        xtce::ByteOrderType::MostSignificantByteFirst => "mostSignificantByteFirst",
        xtce::ByteOrderType::LeastSignificantByteFirst => "leastSignificantByteFirst",
        xtce::ByteOrderType::String(_) => "mostSignificantByteFirst",
    }
}

fn byte_order_from_str(value: &str) -> xtce::ByteOrderType {
    match value.trim() {
        "mostSignificantByteFirst" => xtce::ByteOrderType::MostSignificantByteFirst,
        "leastSignificantByteFirst" => xtce::ByteOrderType::LeastSignificantByteFirst,
        _ => xtce::ByteOrderType::MostSignificantByteFirst,
    }
}

fn integer_value_label(value: &xtce::IntegerValueType) -> String {
    match value {
        xtce::IntegerValueType::FixedValue(value) => value.to_string(),
        xtce::IntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::IntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
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

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn parse_choice<T>(label: &str, fallback: T) -> T
where
    T: std::str::FromStr,
{
    label.parse().unwrap_or(fallback)
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
    let selected_index = options
        .iter()
        .position(|option| option == &selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            options.to_vec(),
            Some(IndexPath::default().row(selected_index)),
            window,
            cx,
        )
    })
}

fn sync_select<T>(
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

fn select_field<T>(
    label: &'static str,
    hint: &'static str,
    select: &Entity<SelectState<Vec<T>>>,
    _cx: &App,
) -> Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    let required = hint == "Required";
    let field = gpui_component::form::field()
        .label(label)
        .required(required)
        .child(Select::new(select).w_full());
    v_flex().w_full().child(if required {
        field
    } else {
        field.description(hint)
    })
}

fn value(input: &Entity<InputState>, cx: &impl AppContext) -> String {
    cx.read_entity(input, |input, _| input.value().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        BinarySizeKind, DataEncodingKind, DataEncodingMut, DataEncodingRef, DataEncodingValues,
        binary_size_kind, byte_order_from_str, byte_order_label, data_error_detect_correct,
        default_binary_encoding, default_integer_encoding, default_string_encoding,
        set_data_encoding_kind,
    };

    #[test]
    fn arbitrary_byte_order_falls_back_to_the_common_default() {
        let arbitrary = xtce::ByteOrderType::String("1,0,3,2".to_owned());

        assert_eq!(byte_order_label(&arbitrary), "mostSignificantByteFirst");
        assert!(matches!(
            byte_order_from_str("1,0,3,2"),
            xtce::ByteOrderType::MostSignificantByteFirst
        ));
    }

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
    fn binary_size_kind_exposes_dynamic_and_lookup_values() {
        let mut encoding = default_binary_encoding();
        encoding.size_in_bits = xtce::IntegerValueType::DynamicValue(xtce::DynamicValueType {
            parameter_instance_ref: xtce::ParameterInstanceRefType {
                parameter_ref: "length".to_owned(),
                instance: 0,
                use_calibrated_value: true,
            },
            linear_adjustment: None,
        });
        assert_eq!(
            binary_size_kind(Some(DataEncodingRef::Binary(&encoding))),
            BinarySizeKind::Dynamic
        );

        encoding.size_in_bits =
            xtce::IntegerValueType::DiscreteLookupList(xtce::DiscreteLookupListType {
                default_value: 8,
                discrete_lookup: Vec::new(),
            });
        assert_eq!(
            binary_size_kind(Some(DataEncodingRef::Binary(&encoding))),
            BinarySizeKind::DiscreteLookup
        );
    }

    #[test]
    fn string_error_detection_is_available_to_the_common_form() {
        let mut encoding = default_string_encoding();
        encoding.content.insert(
            0,
            xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(xtce::ErrorDetectCorrectType {
                content: Vec::new(),
            }),
        );

        assert!(data_error_detect_correct(Some(DataEncodingRef::String(&encoding))).is_some());
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
