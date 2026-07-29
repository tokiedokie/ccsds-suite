use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, prelude::FluentBuilder,
};
use gpui_component::{
    IndexPath, h_flex,
    input::InputState,
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    context_calibrator::ContextCalibratorListForm, default_calibrator::DefaultCalibratorForm,
    discrete_lookup::DiscreteLookupListForm, dynamic_value::DynamicValueForm,
    error_detect_correct::ErrorDetectCorrectForm, field, impl_select_item,
    input_algorithm::InputAlgorithmForm, variable_string::VariableStringForm,
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

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum StringSizeKind {
    #[default]
    Fixed,
    Variable,
}
impl_select_item!(StringSizeKind);

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
    from_transform_present: bool,
    from_transform: Entity<InputAlgorithmForm>,
    to_transform_present: bool,
    to_transform: Entity<InputAlgorithmForm>,
    string_size_kind_select: Entity<SelectState<Vec<StringSizeKind>>>,
    variable_string: Entity<VariableStringForm>,
    fixed_termination_present: bool,
    fixed_termination: Entity<InputState>,
    fixed_leading_size_present: bool,
    fixed_leading_size: Entity<InputState>,
    change_threshold_input: Entity<InputState>,
    default_calibrator: Entity<DefaultCalibratorForm>,
    context_calibrators: Entity<ContextCalibratorListForm>,
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
            let from_transform =
                InputAlgorithmForm::new(binary_from_transform(encoding), window, cx);
            let to_transform = InputAlgorithmForm::new(binary_to_transform(encoding), window, cx);
            let string_size_kind_select = select(
                StringSizeKind::VARIANTS,
                string_size_kind(encoding),
                window,
                cx,
            );
            let variable_string = VariableStringForm::new(string_variable(encoding), window, cx);
            let fixed_termination_value = string_fixed_size(encoding)
                .and_then(|size| size.termination_char.clone())
                .unwrap_or_default();
            let fixed_leading_size_value = string_fixed_size(encoding)
                .and_then(|size| size.leading_size.as_ref())
                .map(|size| size.size_in_bits_of_size_tag.to_string())
                .unwrap_or_else(|| {
                    xtce::LeadingSizeType::default_size_in_bits_of_size_tag().to_string()
                });
            let change_threshold_input = input(&values.change_threshold, window, cx);
            let default_calibrator = DefaultCalibratorForm::new(
                encoding.and_then(DataEncodingRef::default_calibrator),
                window,
                cx,
            );
            let context_calibrators = ContextCalibratorListForm::new(
                encoding.and_then(DataEncodingRef::context_calibrator_list),
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
                    this.from_transform_present = false;
                    this.from_transform
                        .update(cx, |form, cx| form.load(None, window, cx));
                    this.to_transform_present = false;
                    this.to_transform
                        .update(cx, |form, cx| form.load(None, window, cx));
                    sync_select(
                        &this.string_size_kind_select,
                        StringSizeKind::Fixed,
                        window,
                        cx,
                    );
                    this.variable_string
                        .update(cx, |form, cx| form.load(None, window, cx));
                    this.fixed_termination_present = false;
                    this.fixed_termination
                        .update(cx, |input, cx| input.set_value("", window, cx));
                    this.fixed_leading_size_present = false;
                    this.fixed_leading_size.update(cx, |input, cx| {
                        input.set_value(
                            xtce::LeadingSizeType::default_size_in_bits_of_size_tag().to_string(),
                            window,
                            cx,
                        )
                    });
                    this.context_calibrators
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
            let string_size_subscription = cx.subscribe_in(
                &string_size_kind_select,
                window,
                |_: &mut DataEncodingForm, _, _: &SelectEvent<Vec<StringSizeKind>>, _, cx| {
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
                from_transform_present: binary_from_transform(encoding).is_some(),
                from_transform,
                to_transform_present: binary_to_transform(encoding).is_some(),
                to_transform,
                string_size_kind_select,
                variable_string,
                fixed_termination_present: string_fixed_size(encoding)
                    .is_some_and(|size| size.termination_char.is_some()),
                fixed_termination: input(&fixed_termination_value, window, cx),
                fixed_leading_size_present: string_fixed_size(encoding)
                    .is_some_and(|size| size.leading_size.is_some()),
                fixed_leading_size: input(&fixed_leading_size_value, window, cx),
                change_threshold_input,
                default_calibrator,
                context_calibrators,
                _subscriptions: vec![
                    kind_subscription,
                    binary_size_subscription,
                    string_size_subscription,
                ],
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
        self.from_transform_present = binary_from_transform(encoding).is_some();
        self.from_transform.update(cx, |form, cx| {
            form.load(binary_from_transform(encoding), window, cx)
        });
        self.to_transform_present = binary_to_transform(encoding).is_some();
        self.to_transform.update(cx, |form, cx| {
            form.load(binary_to_transform(encoding), window, cx)
        });
        sync_select(
            &self.string_size_kind_select,
            string_size_kind(encoding),
            window,
            cx,
        );
        self.variable_string.update(cx, |form, cx| {
            form.load(string_variable(encoding), window, cx)
        });
        self.fixed_termination_present =
            string_fixed_size(encoding).is_some_and(|size| size.termination_char.is_some());
        self.fixed_termination.update(cx, |input, cx| {
            input.set_value(
                string_fixed_size(encoding)
                    .and_then(|size| size.termination_char.clone())
                    .unwrap_or_default(),
                window,
                cx,
            )
        });
        self.fixed_leading_size_present =
            string_fixed_size(encoding).is_some_and(|size| size.leading_size.is_some());
        self.fixed_leading_size.update(cx, |input, cx| {
            input.set_value(
                string_fixed_size(encoding)
                    .and_then(|size| size.leading_size.as_ref())
                    .map(|size| size.size_in_bits_of_size_tag.to_string())
                    .unwrap_or_else(|| {
                        xtce::LeadingSizeType::default_size_in_bits_of_size_tag().to_string()
                    }),
                window,
                cx,
            )
        });
        self.default_calibrator.update(cx, |form, cx| {
            form.load(
                encoding.and_then(DataEncodingRef::default_calibrator),
                window,
                cx,
            );
        });
        self.context_calibrators.update(cx, |form, cx| {
            form.load(
                encoding.and_then(DataEncodingRef::context_calibrator_list),
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
        match &mut encoding {
            DataEncodingMut::Float(value) => self
                .context_calibrators
                .read(cx)
                .apply_to(&mut value.context_calibrator_list, cx),
            DataEncodingMut::Integer(value) => self
                .context_calibrators
                .read(cx)
                .apply_to(&mut value.context_calibrator_list, cx),
            DataEncodingMut::Binary(_)
            | DataEncodingMut::ArgumentBinary(_)
            | DataEncodingMut::String(_)
            | DataEncodingMut::ArgumentString(_) => {}
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
            binary.from_binary_transform_algorithm = self
                .from_transform_present
                .then(|| self.from_transform.read(cx).algorithm(cx));
            binary.to_binary_transform_algorithm = self
                .to_transform_present
                .then(|| self.to_transform.read(cx).algorithm(cx));
        }
        if let DataEncodingMut::String(string) = &mut encoding {
            apply_string_size(
                string,
                selected_value(&self.string_size_kind_select, StringSizeKind::Fixed, cx),
                &self.size_in_bits_input,
                &self.variable_string,
                cx,
            );
            if let Some(size) = string.content.iter_mut().find_map(|item| match item {
                xtce::StringDataEncodingTypeContent::SizeInBits(size) => Some(size),
                _ => None,
            }) {
                size.termination_char = self
                    .fixed_termination_present
                    .then(|| value(&self.fixed_termination, cx));
                size.leading_size =
                    self.fixed_leading_size_present
                        .then(|| xtce::LeadingSizeType {
                            size_in_bits_of_size_tag: value(&self.fixed_leading_size, cx)
                                .trim()
                                .parse()
                                .unwrap_or_else(|_| {
                                    xtce::LeadingSizeType::default_size_in_bits_of_size_tag()
                                }),
                        });
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

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        let kind = self.selected_kind(cx);
        let mut form = v_flex().gap_5().child(select_field(
            "Data encoding",
            "Required",
            &self.kind_select,
            cx,
        ));
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
                } else if kind == DataEncodingKind::String {
                    select_field(
                        "Size in bits",
                        "Required",
                        &self.string_size_kind_select,
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
            form = form.child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(super::section_title("From-binary transform"))
                            .child(if self.from_transform_present {
                                super::section_remove_button("remove-data-from-binary-transform")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.from_transform_present = false;
                                        cx.notify();
                                    }))
                            } else {
                                super::section_add_button("add-data-from-binary-transform")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.from_transform_present = true;
                                        cx.notify();
                                    }))
                            }),
                    )
                    .when(self.from_transform_present, |section| {
                        section.child(self.from_transform.clone())
                    }),
            );
            form = form.child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(super::section_title("To-binary transform"))
                            .child(if self.to_transform_present {
                                super::section_remove_button("remove-data-to-binary-transform")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.to_transform_present = false;
                                        cx.notify();
                                    }))
                            } else {
                                super::section_add_button("add-data-to-binary-transform").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.to_transform_present = true;
                                        cx.notify();
                                    }),
                                )
                            }),
                    )
                    .when(self.to_transform_present, |section| {
                        section.child(self.to_transform.clone())
                    }),
            );
        }
        if kind == DataEncodingKind::String {
            form = match selected_value(&self.string_size_kind_select, StringSizeKind::Fixed, cx) {
                StringSizeKind::Fixed => form
                    .child(field(
                        "Fixed size in bits",
                        "Required",
                        &self.size_in_bits_input,
                        cx,
                    ))
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(super::section_title("Termination character"))
                                    .child(if self.fixed_termination_present {
                                        super::section_remove_button(
                                            "remove-fixed-string-termination",
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.fixed_termination_present = false;
                                                cx.notify();
                                            }),
                                        )
                                    } else {
                                        super::section_add_button("add-fixed-string-termination")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.fixed_termination_present = true;
                                                cx.notify();
                                            }))
                                    }),
                            )
                            .when(self.fixed_termination_present, |section| {
                                section.child(field(
                                    "Termination character",
                                    "Required",
                                    &self.fixed_termination,
                                    cx,
                                ))
                            }),
                    )
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(super::section_title("Leading size"))
                                    .child(if self.fixed_leading_size_present {
                                        super::section_remove_button(
                                            "remove-fixed-string-leading-size",
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.fixed_leading_size_present = false;
                                                cx.notify();
                                            }),
                                        )
                                    } else {
                                        super::section_add_button("add-fixed-string-leading-size")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.fixed_leading_size_present = true;
                                                cx.notify();
                                            }))
                                    }),
                            )
                            .when(self.fixed_leading_size_present, |section| {
                                section.child(field(
                                    "Size tag width in bits",
                                    "Optional; defaults to 16",
                                    &self.fixed_leading_size,
                                    cx,
                                ))
                            }),
                    ),
                StringSizeKind::Variable => form.child(self.variable_string.clone()),
            };
        }
        form = form.child(self.error_detect_correct.clone());
        if matches!(kind, DataEncodingKind::Float | DataEncodingKind::Integer) {
            form = form
                .child(self.default_calibrator.clone())
                .child(self.context_calibrators.clone());
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
    ArgumentBinary(&'a xtce::ArgumentBinaryDataEncodingType),
    Float(&'a xtce::FloatDataEncodingType),
    Integer(&'a xtce::IntegerDataEncodingType),
    String(&'a xtce::StringDataEncodingType),
    ArgumentString(&'a xtce::ArgumentStringDataEncodingType),
}

impl<'a> DataEncodingRef<'a> {
    pub(super) fn kind(self) -> DataEncodingKind {
        match self {
            Self::Binary(_) | Self::ArgumentBinary(_) => DataEncodingKind::Binary,
            Self::Float(_) => DataEncodingKind::Float,
            Self::Integer(_) => DataEncodingKind::Integer,
            Self::String(_) | Self::ArgumentString(_) => DataEncodingKind::String,
        }
    }

    fn default_calibrator(self) -> Option<&'a xtce::CalibratorType> {
        match self {
            Self::Float(value) => value.default_calibrator.as_ref(),
            Self::Integer(value) => value.default_calibrator.as_ref(),
            Self::Binary(_)
            | Self::ArgumentBinary(_)
            | Self::String(_)
            | Self::ArgumentString(_) => None,
        }
    }

    fn context_calibrator_list(self) -> Option<&'a xtce::ContextCalibratorListType> {
        match self {
            Self::Float(value) => value.context_calibrator_list.as_ref(),
            Self::Integer(value) => value.context_calibrator_list.as_ref(),
            Self::Binary(_)
            | Self::ArgumentBinary(_)
            | Self::String(_)
            | Self::ArgumentString(_) => None,
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
    match encoding {
        Some(DataEncodingRef::Binary(value)) => match &value.size_in_bits {
            xtce::IntegerValueType::DynamicValue(_) => BinarySizeKind::Dynamic,
            xtce::IntegerValueType::DiscreteLookupList(_) => BinarySizeKind::DiscreteLookup,
            _ => BinarySizeKind::Fixed,
        },
        Some(DataEncodingRef::ArgumentBinary(value)) => match &value.size_in_bits {
            xtce::ArgumentIntegerValueType::DynamicValue(_) => BinarySizeKind::Dynamic,
            xtce::ArgumentIntegerValueType::DiscreteLookupList(_) => BinarySizeKind::DiscreteLookup,
            _ => BinarySizeKind::Fixed,
        },
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

fn binary_from_transform(
    encoding: Option<DataEncodingRef<'_>>,
) -> Option<&xtce::InputAlgorithmType> {
    match encoding {
        Some(DataEncodingRef::Binary(value)) => value.from_binary_transform_algorithm.as_ref(),
        _ => None,
    }
}

fn binary_to_transform(encoding: Option<DataEncodingRef<'_>>) -> Option<&xtce::InputAlgorithmType> {
    match encoding {
        Some(DataEncodingRef::Binary(value)) => value.to_binary_transform_algorithm.as_ref(),
        _ => None,
    }
}

fn string_fixed_size(encoding: Option<DataEncodingRef<'_>>) -> Option<&xtce::SizeInBitsType> {
    match encoding {
        Some(DataEncodingRef::String(value)) => value.content.iter().find_map(|item| match item {
            xtce::StringDataEncodingTypeContent::SizeInBits(value) => Some(value),
            _ => None,
        }),
        Some(DataEncodingRef::ArgumentString(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::ArgumentStringDataEncodingTypeContent::SizeInBits(value) => Some(value),
                _ => None,
            })
        }
        _ => None,
    }
}

fn string_variable(encoding: Option<DataEncodingRef<'_>>) -> Option<&xtce::VariableStringType> {
    match encoding {
        Some(DataEncodingRef::String(value)) => value.content.iter().find_map(|item| match item {
            xtce::StringDataEncodingTypeContent::Variable(value) => Some(value),
            _ => None,
        }),
        _ => None,
    }
}

fn string_size_kind(encoding: Option<DataEncodingRef<'_>>) -> StringSizeKind {
    if string_variable(encoding).is_some() {
        StringSizeKind::Variable
    } else {
        StringSizeKind::Fixed
    }
}

fn apply_string_size(
    string: &mut xtce::StringDataEncodingType,
    kind: StringSizeKind,
    fixed_size: &Entity<InputState>,
    variable: &Entity<VariableStringForm>,
    cx: &App,
) {
    match kind {
        StringSizeKind::Fixed => {
            string
                .content
                .retain(|item| !matches!(item, xtce::StringDataEncodingTypeContent::Variable(_)));
            if !string
                .content
                .iter()
                .any(|item| matches!(item, xtce::StringDataEncodingTypeContent::SizeInBits(_)))
            {
                let fixed_value = value(fixed_size, cx).trim().parse().unwrap_or_default();
                let index = string
                    .content
                    .iter()
                    .take_while(|item| {
                        matches!(
                            item,
                            xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(_)
                        )
                    })
                    .count();
                string.content.insert(
                    index,
                    xtce::StringDataEncodingTypeContent::SizeInBits(xtce::SizeInBitsType {
                        fixed: xtce::SizeInBitsTypeFixedElementType { fixed_value },
                        termination_char: None,
                        leading_size: None,
                    }),
                );
            }
        }
        StringSizeKind::Variable => {
            string.content.retain(|item| {
                !matches!(
                    item,
                    xtce::StringDataEncodingTypeContent::SizeInBits(_)
                        | xtce::StringDataEncodingTypeContent::Variable(_)
                )
            });
            let index = string
                .content
                .iter()
                .take_while(|item| {
                    matches!(
                        item,
                        xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(_)
                    )
                })
                .count();
            string.content.insert(
                index,
                xtce::StringDataEncodingTypeContent::Variable(variable.read(cx).value(cx)),
            );
        }
    }
}

fn data_error_detect_correct(
    encoding: Option<DataEncodingRef<'_>>,
) -> Option<&xtce::ErrorDetectCorrectType> {
    match encoding {
        Some(DataEncodingRef::Binary(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::ArgumentBinary(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::Float(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::Integer(value)) => value.error_detect_correct.as_ref(),
        Some(DataEncodingRef::String(value)) => value.content.iter().find_map(|item| match item {
            xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(value) => Some(value),
            _ => None,
        }),
        Some(DataEncodingRef::ArgumentString(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::ArgumentStringDataEncodingTypeContent::ErrorDetectCorrect(value) => {
                    Some(value)
                }
                _ => None,
            })
        }
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
        DataEncodingMut::ArgumentBinary(value) => value.error_detect_correct = error_detect_correct,
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
        DataEncodingMut::ArgumentString(string) => {
            let current = string.content.iter().position(|item| {
                matches!(
                    item,
                    xtce::ArgumentStringDataEncodingTypeContent::ErrorDetectCorrect(_)
                )
            });
            match (current, error_detect_correct) {
                (Some(index), Some(error)) => {
                    string.content[index] =
                        xtce::ArgumentStringDataEncodingTypeContent::ErrorDetectCorrect(error);
                }
                (None, Some(error)) => string.content.insert(
                    0,
                    xtce::ArgumentStringDataEncodingTypeContent::ErrorDetectCorrect(error),
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
    ArgumentBinary(&'a mut xtce::ArgumentBinaryDataEncodingType),
    Float(&'a mut xtce::FloatDataEncodingType),
    Integer(&'a mut xtce::IntegerDataEncodingType),
    String(&'a mut xtce::StringDataEncodingType),
    ArgumentString(&'a mut xtce::ArgumentStringDataEncodingType),
}

impl DataEncodingMut<'_> {
    fn default_calibrator_mut(&mut self) -> Option<&mut Option<xtce::CalibratorType>> {
        match self {
            Self::Float(value) => Some(&mut value.default_calibrator),
            Self::Integer(value) => Some(&mut value.default_calibrator),
            Self::Binary(_)
            | Self::ArgumentBinary(_)
            | Self::String(_)
            | Self::ArgumentString(_) => None,
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

pub(super) fn find_argument_data_encoding(
    argument_type: &xtce::ArgumentTypeSetTypeContent,
) -> Option<DataEncodingRef<'_>> {
    macro_rules! find {
        ($content:expr, $enum_type:ident) => {
            $content.iter().find_map(|item| match item {
                xtce::$enum_type::BinaryDataEncoding(value) => {
                    Some(DataEncodingRef::ArgumentBinary(value))
                }
                xtce::$enum_type::FloatDataEncoding(value) => Some(DataEncodingRef::Float(value)),
                xtce::$enum_type::IntegerDataEncoding(value) => {
                    Some(DataEncodingRef::Integer(value))
                }
                xtce::$enum_type::StringDataEncoding(value) => {
                    Some(DataEncodingRef::ArgumentString(value))
                }
                _ => None,
            })
        };
    }
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
            find!(value.content, StringArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
            find!(value.content, EnumeratedArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
            find!(value.content, IntegerArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
            find!(value.content, BinaryArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
            find!(value.content, FloatArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
            find!(value.content, BooleanArgumentTypeContent)
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

pub(super) fn find_argument_data_encoding_mut(
    argument_type: &mut xtce::ArgumentTypeSetTypeContent,
) -> Option<DataEncodingMut<'_>> {
    macro_rules! find {
        ($content:expr, $enum_type:ident) => {
            $content.iter_mut().find_map(|item| match item {
                xtce::$enum_type::BinaryDataEncoding(value) => {
                    Some(DataEncodingMut::ArgumentBinary(value))
                }
                xtce::$enum_type::FloatDataEncoding(value) => Some(DataEncodingMut::Float(value)),
                xtce::$enum_type::IntegerDataEncoding(value) => {
                    Some(DataEncodingMut::Integer(value))
                }
                xtce::$enum_type::StringDataEncoding(value) => {
                    Some(DataEncodingMut::ArgumentString(value))
                }
                _ => None,
            })
        };
    }
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
            find!(value.content, StringArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
            find!(value.content, EnumeratedArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
            find!(value.content, IntegerArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
            find!(value.content, BinaryArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
            find!(value.content, FloatArgumentTypeContent)
        }
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
            find!(value.content, BooleanArgumentTypeContent)
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

pub(super) fn set_argument_data_encoding_kind(
    argument_type: &mut xtce::ArgumentTypeSetTypeContent,
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
                            xtce::$enum_type::BinaryDataEncoding(default_argument_binary_encoding())
                        }
                        DataEncodingKind::Float => {
                            xtce::$enum_type::FloatDataEncoding(default_float_encoding())
                        }
                        DataEncodingKind::Integer => {
                            xtce::$enum_type::IntegerDataEncoding(default_integer_encoding())
                        }
                        DataEncodingKind::String => {
                            xtce::$enum_type::StringDataEncoding(default_argument_string_encoding())
                        }
                    };
                    $content.insert(index, encoding);
                }
            }
        }};
    }

    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
            set_kind!(value.content, StringArgumentTypeContent);
        }
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
            set_kind!(value.content, EnumeratedArgumentTypeContent);
        }
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
            set_kind!(value.content, IntegerArgumentTypeContent);
        }
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
            set_kind!(value.content, BinaryArgumentTypeContent);
        }
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
            set_kind!(value.content, FloatArgumentTypeContent);
        }
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
            set_kind!(value.content, BooleanArgumentTypeContent);
        }
        _ => {}
    }
}

fn default_argument_binary_encoding() -> xtce::ArgumentBinaryDataEncodingType {
    xtce::ArgumentBinaryDataEncodingType {
        bit_order: xtce::BitOrderType::MostSignificantBitFirst,
        byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
        error_detect_correct: None,
        size_in_bits: xtce::ArgumentIntegerValueType::FixedValue(8),
        from_binary_transform_algorithm: None,
        to_binary_transform_algorithm: None,
    }
}

fn default_argument_string_encoding() -> xtce::ArgumentStringDataEncodingType {
    xtce::ArgumentStringDataEncodingType {
        bit_order: xtce::BitOrderType::MostSignificantBitFirst,
        byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
        encoding: xtce::StringEncodingType::Utf8,
        content: Vec::new(),
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
            Some(DataEncodingRef::ArgumentBinary(value)) => Self::new(
                &value.bit_order,
                &value.byte_order,
                "",
                &argument_integer_value_label(&value.size_in_bits),
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
            Some(DataEncodingRef::ArgumentString(value)) => Self::new(
                &value.bit_order,
                &value.byte_order,
                string_encoding_label(&value.encoding),
                &value
                    .content
                    .iter()
                    .find_map(|item| match item {
                        xtce::ArgumentStringDataEncodingTypeContent::SizeInBits(size) => {
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
            DataEncodingMut::ArgumentBinary(value) => {
                apply_orders(&mut value.bit_order, &mut value.byte_order, self);
                if let (xtce::ArgumentIntegerValueType::FixedValue(size), Ok(new_size)) =
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
            DataEncodingMut::ArgumentString(value) => {
                apply_orders(&mut value.bit_order, &mut value.byte_order, self);
                if let Some(encoding) = string_encoding_from_str(&self.encoding) {
                    value.encoding = encoding;
                }
                if let Ok(size_in_bits) = self.size_in_bits.parse()
                    && let Some(size) = value.content.iter_mut().find_map(|item| match item {
                        xtce::ArgumentStringDataEncodingTypeContent::SizeInBits(size) => Some(size),
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

fn argument_integer_value_label(value: &xtce::ArgumentIntegerValueType) -> String {
    match value {
        xtce::ArgumentIntegerValueType::FixedValue(value) => value.to_string(),
        xtce::ArgumentIntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::ArgumentIntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
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
    super::select_field(label, hint, select)
}

fn value(input: &Entity<InputState>, cx: &impl AppContext) -> String {
    cx.read_entity(input, |input, _| input.value().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        BinarySizeKind, DataEncodingKind, DataEncodingMut, DataEncodingRef, DataEncodingValues,
        binary_from_transform, binary_size_kind, binary_to_transform, byte_order_from_str,
        byte_order_label, data_error_detect_correct, default_binary_encoding,
        default_integer_encoding, default_string_encoding, set_data_encoding_kind,
        string_fixed_size,
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
    fn fixed_string_termination_is_available_to_the_form() {
        let mut encoding = default_string_encoding();
        let size = encoding.content.iter_mut().find_map(|item| match item {
            xtce::StringDataEncodingTypeContent::SizeInBits(size) => Some(size),
            _ => None,
        });
        size.unwrap().termination_char = Some("00".to_owned());

        assert_eq!(
            string_fixed_size(Some(DataEncodingRef::String(&encoding)))
                .and_then(|size| size.termination_char.as_deref()),
            Some("00")
        );
    }

    #[test]
    fn fixed_string_leading_size_is_available_to_the_form() {
        let mut encoding = default_string_encoding();
        let size = encoding.content.iter_mut().find_map(|item| match item {
            xtce::StringDataEncodingTypeContent::SizeInBits(size) => Some(size),
            _ => None,
        });
        size.unwrap().leading_size = Some(xtce::LeadingSizeType {
            size_in_bits_of_size_tag: 12,
        });

        assert_eq!(
            string_fixed_size(Some(DataEncodingRef::String(&encoding)))
                .and_then(|size| size.leading_size.as_ref())
                .map(|size| size.size_in_bits_of_size_tag),
            Some(12)
        );
    }

    #[test]
    fn binary_from_transform_is_available_to_the_form() {
        let mut encoding = default_binary_encoding();
        encoding.from_binary_transform_algorithm = Some(xtce::InputAlgorithmType {
            short_description: None,
            name: "decodeValue".to_owned(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            algorithm_text: None,
            external_algorithm_set: None,
            input_set: None,
        });

        assert_eq!(
            binary_from_transform(Some(DataEncodingRef::Binary(&encoding)))
                .map(|algorithm| algorithm.name.as_str()),
            Some("decodeValue")
        );
    }

    #[test]
    fn binary_to_transform_is_available_to_the_form() {
        let mut encoding = default_binary_encoding();
        encoding.to_binary_transform_algorithm = Some(xtce::InputAlgorithmType {
            short_description: None,
            name: "encodeValue".to_owned(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            algorithm_text: None,
            external_algorithm_set: None,
            input_set: None,
        });

        assert_eq!(
            binary_to_transform(Some(DataEncodingRef::Binary(&encoding)))
                .map(|algorithm| algorithm.name.as_str()),
            Some("encodeValue")
        );
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
