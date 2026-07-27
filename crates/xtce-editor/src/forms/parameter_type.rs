use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div,
};
use gpui_component::{
    IndexPath, StyledExt, h_flex,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    data_encoding::{
        DataEncodingForm, find_data_encoding, find_data_encoding_mut, set_data_encoding_kind,
    },
    field, impl_select_item, optional_value,
};
use crate::XtceEditor;

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CharacterWidthChoice {
    Default,
    #[strum(serialize = "8")]
    _8,
    #[strum(serialize = "16")]
    _16,
    #[strum(serialize = "32")]
    _32,
}
impl_select_item!(CharacterWidthChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum FloatSizeChoice {
    #[strum(serialize = "32")]
    _32,
    #[strum(serialize = "64")]
    _64,
    #[strum(serialize = "128")]
    _128,
}
impl_select_item!(FloatSizeChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum SignedChoice {
    #[strum(serialize = "true")]
    Signed,
    #[strum(serialize = "false")]
    Unsigned,
}
impl_select_item!(SignedChoice);

pub(super) struct ParameterTypeForm {
    present: bool,
    kind: ParameterTypeKind,
    kind_select: Entity<SelectState<Vec<ParameterTypeKind>>>,
    name_input: Entity<InputState>,
    base_or_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    extra_a_input: Entity<InputState>,
    extra_b_input: Entity<InputState>,
    character_width_select: Entity<SelectState<Vec<CharacterWidthChoice>>>,
    signed_select: Entity<SelectState<Vec<SignedChoice>>>,
    float_size_select: Entity<SelectState<Vec<FloatSizeChoice>>>,
    nested_items_input: Entity<InputState>,
    data_encoding: Entity<DataEncodingForm>,
    _subscriptions: Vec<Subscription>,
}

impl ParameterTypeForm {
    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(gpui::px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn new(
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = ParameterTypeValues::from_type(parameter_type);
        let kind = parameter_type
            .map(kind)
            .unwrap_or(ParameterTypeKind::String);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| {
            let kind_select = cx.new(|cx| {
                SelectState::new(
                    ParameterTypeKind::VARIANTS.to_vec(),
                    Some(IndexPath::default().row(kind.option_index())),
                    window,
                    cx,
                )
            });
            let base_or_ref_input = input(&values.base_or_ref, false, window, cx);
            let initial_value_input = input(&values.initial_value, false, window, cx);
            let short_description_input = input(&values.short_description, false, window, cx);
            let long_description_input = input(&values.long_description, true, window, cx);
            let extra_a_input = input(&values.extra_a, false, window, cx);
            let extra_b_input = input(&values.extra_b, false, window, cx);
            let character_width_select = select(
                CharacterWidthChoice::VARIANTS,
                parse_choice(
                    if values.extra_b.is_empty() {
                        "Default"
                    } else {
                        &values.extra_b
                    },
                    CharacterWidthChoice::Default,
                ),
                window,
                cx,
            );
            let signed_select = select(
                SignedChoice::VARIANTS,
                parse_choice(&values.extra_b, SignedChoice::Signed),
                window,
                cx,
            );
            let float_size_select = select(
                FloatSizeChoice::VARIANTS,
                parse_choice(&values.extra_a, FloatSizeChoice::_32),
                window,
                cx,
            );
            let mut subscriptions = vec![name_subscription];
            let nested_items_input = input(&encode_nested_items(parameter_type), true, window, cx);

            let kind_extra_a = extra_a_input.clone();
            let kind_extra_b = extra_b_input.clone();
            let kind_character_width = character_width_select.clone();
            let kind_signed = signed_select.clone();
            let kind_float_size = float_size_select.clone();
            let kind_nested_items = nested_items_input.clone();
            subscriptions.push(cx.subscribe_in(
                &kind_select,
                window,
                move |this: &mut ParameterTypeForm,
                      _,
                      event: &SelectEvent<Vec<ParameterTypeKind>>,
                      window,
                      cx| {
                    let SelectEvent::Confirm(Some(selected_kind)) = event else {
                        return;
                    };
                    let selected_kind = *selected_kind;
                    if this.kind == selected_kind {
                        return;
                    }
                    this.kind = selected_kind;
                    let (extra_a, extra_b) = selected_kind.default_extra_values();
                    for (input, value) in [
                        (&kind_extra_a, extra_a.to_owned()),
                        (&kind_extra_b, extra_b.to_owned()),
                    ] {
                        input.update(cx, |input, cx| input.set_value(value, window, cx));
                    }
                    sync_select(
                        &kind_character_width,
                        parse_choice(
                            if selected_kind == ParameterTypeKind::String {
                                if extra_b.is_empty() {
                                    "Default"
                                } else {
                                    extra_b
                                }
                            } else {
                                "Default"
                            },
                            CharacterWidthChoice::Default,
                        ),
                        window,
                        cx,
                    );
                    sync_select(
                        &kind_signed,
                        parse_choice(
                            if selected_kind == ParameterTypeKind::Integer {
                                extra_b
                            } else {
                                "true"
                            },
                            SignedChoice::Signed,
                        ),
                        window,
                        cx,
                    );
                    sync_select(
                        &kind_float_size,
                        parse_choice(
                            if selected_kind == ParameterTypeKind::Float {
                                extra_a
                            } else {
                                "32"
                            },
                            FloatSizeChoice::_32,
                        ),
                        window,
                        cx,
                    );
                    kind_nested_items.update(cx, |input, cx| {
                        input.set_value(default_nested_items(selected_kind).to_owned(), window, cx);
                    });
                    cx.notify();
                },
            ));

            Self {
                present: parameter_type.is_some(),
                kind,
                kind_select,
                name_input,
                base_or_ref_input,
                initial_value_input,
                short_description_input,
                long_description_input,
                extra_a_input,
                extra_b_input,
                character_width_select,
                signed_select,
                float_size_select,
                nested_items_input,
                data_encoding: DataEncodingForm::new(
                    parameter_type.and_then(find_data_encoding),
                    window,
                    cx,
                ),
                _subscriptions: subscriptions,
            }
        })
    }

    pub(super) fn load(
        &mut self,
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ParameterTypeValues::from_type(parameter_type);
        self.present = parameter_type.is_some();
        let selected_kind = parameter_type
            .map(kind)
            .unwrap_or(ParameterTypeKind::String);
        self.kind = selected_kind;
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&selected_kind, window, cx);
        });
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.base_or_ref_input, values.base_or_ref),
            (&self.initial_value_input, values.initial_value),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.extra_a_input, values.extra_a),
            (&self.extra_b_input, values.extra_b),
            (
                &self.nested_items_input,
                encode_nested_items(parameter_type),
            ),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        let extra_a = value(&self.extra_a_input, cx);
        let extra_b = value(&self.extra_b_input, cx);
        let character_width = if extra_b.is_empty() {
            "Default"
        } else {
            &extra_b
        };
        sync_select(
            &self.character_width_select,
            parse_choice(character_width, CharacterWidthChoice::Default),
            window,
            cx,
        );
        sync_select(
            &self.signed_select,
            parse_choice(&extra_b, SignedChoice::Signed),
            window,
            cx,
        );
        sync_select(
            &self.float_size_select,
            parse_choice(&extra_a, FloatSizeChoice::_32),
            window,
            cx,
        );
        self.data_encoding.update(cx, |form, cx| {
            form.load(parameter_type.and_then(find_data_encoding), window, cx);
        });
        cx.notify();
    }

    pub(super) fn apply_to(
        &self,
        parameter_type: &mut xtce::ParameterTypeSetTypeContent,
        cx: &App,
    ) {
        replace_parameter_type_kind(parameter_type, self.kind);
        let extra_a = if self.kind == ParameterTypeKind::Float {
            selected_value(&self.float_size_select, FloatSizeChoice::_32, cx).to_string()
        } else {
            value(&self.extra_a_input, cx)
        };
        let extra_b = match self.kind {
            ParameterTypeKind::String => {
                let width = selected_value(
                    &self.character_width_select,
                    CharacterWidthChoice::Default,
                    cx,
                );
                if width == CharacterWidthChoice::Default {
                    String::new()
                } else {
                    width.to_string()
                }
            }
            ParameterTypeKind::Integer => {
                selected_value(&self.signed_select, SignedChoice::Signed, cx).to_string()
            }
            _ => value(&self.extra_b_input, cx),
        };
        ParameterTypeValues {
            name: value(&self.name_input, cx),
            base_or_ref: value(&self.base_or_ref_input, cx),
            initial_value: value(&self.initial_value_input, cx),
            short_description: value(&self.short_description_input, cx),
            long_description: value(&self.long_description_input, cx),
            extra_a,
            extra_b,
        }
        .apply_to(parameter_type);
        apply_nested_items(parameter_type, &self.nested_items_input.read(cx).value());
        set_data_encoding_kind(
            parameter_type,
            self.data_encoding.read(cx).selected_kind(cx),
        );
        if let Some(encoding) = find_data_encoding_mut(parameter_type) {
            self.data_encoding.read(cx).apply_to(encoding, cx);
        }
    }

    fn render_form(&self, cx: &App) -> Div {
        if !self.present {
            return v_flex();
        }
        let kind = self.kind;
        let mut identity_fields = h_flex().gap_4().items_start();
        if let Some((label, hint)) = kind.base_field() {
            identity_fields =
                identity_fields.child(field(label, hint, &self.base_or_ref_input, cx));
        }

        let mut form = v_flex().gap_5().child(
            v_flex()
                .gap_2()
                .child(div().text_sm().font_medium().child("Parameter type"))
                .child(Select::new(&self.kind_select).w_full()),
        );
        if kind.base_field().is_some() {
            form = form.child(identity_fields);
        }
        form = form.child(
            h_flex()
                .gap_4()
                .items_start()
                .child(field(
                    "Initial value",
                    "Optional",
                    &self.initial_value_input,
                    cx,
                ))
                .child(field(
                    "Short description",
                    "Optional",
                    &self.short_description_input,
                    cx,
                )),
        );
        match kind {
            ParameterTypeKind::String => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field(
                            "Restriction pattern",
                            "Optional",
                            &self.extra_a_input,
                            cx,
                        ))
                        .child(select_field(
                            "Character width",
                            "Optional",
                            &self.character_width_select,
                            cx,
                        )),
                );
            }
            ParameterTypeKind::Integer => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field("Size in bits", "Integer", &self.extra_a_input, cx))
                        .child(select_field("Signed", "Required", &self.signed_select, cx)),
                );
            }
            ParameterTypeKind::Float => {
                form = form.child(select_field(
                    "Size in bits",
                    "Required",
                    &self.float_size_select,
                    cx,
                ));
            }
            ParameterTypeKind::Boolean => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field(
                            "One string value",
                            "Defaults to True",
                            &self.extra_a_input,
                            cx,
                        ))
                        .child(field(
                            "Zero string value",
                            "Defaults to False",
                            &self.extra_b_input,
                            cx,
                        )),
                );
            }
            _ => {}
        }
        if kind.has_direct_long_description() {
            form = form.child(field(
                "Long description",
                "Optional",
                &self.long_description_input,
                cx,
            ));
        }
        match kind {
            ParameterTypeKind::Array => {
                form = form.child(field(
                    "Dimensions",
                    "One dimension per line: starting index | ending index",
                    &self.nested_items_input,
                    cx,
                ));
            }
            ParameterTypeKind::Aggregate => {
                form = form.child(field(
                    "Members",
                    "One member per line: name | type ref | initial value | short description",
                    &self.nested_items_input,
                    cx,
                ));
            }
            _ => {}
        }
        if kind.supports_data_encoding() {
            form = form
                .child(div().text_lg().font_semibold().child("Data encoding"))
                .child(self.data_encoding.clone());
        }
        form
    }
}

impl Render for ParameterTypeForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ParameterTypeKind {
    #[strum(serialize = "StringParameterType")]
    String,
    #[strum(serialize = "EnumeratedParameterType")]
    Enumerated,
    #[strum(serialize = "IntegerParameterType")]
    Integer,
    #[strum(serialize = "BinaryParameterType")]
    Binary,
    #[strum(serialize = "FloatParameterType")]
    Float,
    #[strum(serialize = "BooleanParameterType")]
    Boolean,
    #[strum(serialize = "RelativeTimeParameterType")]
    RelativeTime,
    #[strum(serialize = "AbsoluteTimeParameterType")]
    AbsoluteTime,
    #[strum(serialize = "ArrayParameterType")]
    Array,
    #[strum(serialize = "AggregateParameterType")]
    Aggregate,
}
impl_select_item!(ParameterTypeKind);

impl ParameterTypeKind {
    fn option_index(self) -> usize {
        match self {
            Self::String => 0,
            Self::Enumerated => 1,
            Self::Integer => 2,
            Self::Binary => 3,
            Self::Float => 4,
            Self::Boolean => 5,
            Self::RelativeTime => 6,
            Self::AbsoluteTime => 7,
            Self::Array => 8,
            Self::Aggregate => 9,
        }
    }

    fn default_extra_values(self) -> (&'static str, &'static str) {
        match self {
            Self::Integer => ("32", "true"),
            Self::Float => ("32", "Float"),
            Self::Boolean => ("True", "False"),
            _ => ("", ""),
        }
    }

    fn supports_data_encoding(self) -> bool {
        matches!(
            self,
            Self::String
                | Self::Enumerated
                | Self::Integer
                | Self::Binary
                | Self::Float
                | Self::Boolean
        )
    }

    fn base_field(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Array => Some(("Array type reference", "Required")),
            Self::Aggregate => None,
            _ => Some(("Base type", "Optional; used only for type inheritance")),
        }
    }

    fn has_direct_long_description(self) -> bool {
        matches!(
            self,
            Self::RelativeTime | Self::AbsoluteTime | Self::Array | Self::Aggregate
        )
    }
}

struct ParameterTypeValues {
    name: String,
    base_or_ref: String,
    initial_value: String,
    short_description: String,
    long_description: String,
    extra_a: String,
    extra_b: String,
}

impl ParameterTypeValues {
    fn from_type(parameter_type: Option<&xtce::ParameterTypeSetTypeContent>) -> Self {
        let Some(parameter_type) = parameter_type else {
            return Self::empty();
        };
        match parameter_type {
            xtce::ParameterTypeSetTypeContent::StringParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                "",
                value.restriction_pattern.as_deref().unwrap_or_default(),
                character_width_label(value.character_width.as_ref()),
            ),
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                "",
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value
                    .initial_value
                    .map(|value| value.to_string())
                    .as_deref(),
                value.short_description.as_deref(),
                "",
                &value.size_in_bits.to_string(),
                &value.signed.to_string(),
            ),
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                "",
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value
                    .initial_value
                    .map(|value| value.to_string())
                    .as_deref(),
                value.short_description.as_deref(),
                "",
                float_size_label(&value.size_in_bits),
                "Float",
            ),
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                "",
                &value.one_string_value,
                &value.zero_string_value,
            ),
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => Self::common(
                &value.name,
                Some(&value.array_type_ref),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => Self::common(
                &value.name,
                None,
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
        }
    }

    fn common(
        name: &str,
        base_or_ref: Option<&str>,
        initial_value: Option<&str>,
        short_description: Option<&str>,
        long_description: &str,
        extra_a: &str,
        extra_b: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            base_or_ref: base_or_ref.unwrap_or_default().to_owned(),
            initial_value: initial_value.unwrap_or_default().to_owned(),
            short_description: short_description.unwrap_or_default().to_owned(),
            long_description: long_description.to_owned(),
            extra_a: extra_a.to_owned(),
            extra_b: extra_b.to_owned(),
        }
    }

    fn empty() -> Self {
        Self::common("", None, None, None, "", "", "")
    }

    fn apply_to(&self, parameter_type: &mut xtce::ParameterTypeSetTypeContent) {
        macro_rules! common {
            ($value:expr) => {{
                $value.name.clone_from(&self.name);
                $value.base_type = optional_value(self.base_or_ref.clone());
                $value.short_description = optional_value(self.short_description.clone());
            }};
        }
        match parameter_type {
            xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.restriction_pattern = optional_value(self.extra_a.clone());
                value.character_width = character_width_from_str(&self.extra_b);
            }
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
            }
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
                common!(value);
                value.initial_value = self.initial_value.parse().ok();
                if let Ok(size) = self.extra_a.parse() {
                    value.size_in_bits = size;
                }
                if let Some(signed) = bool_from_str(&self.extra_b) {
                    value.signed = signed;
                }
            }
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
            }
            xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
                common!(value);
                value.initial_value = self.initial_value.parse().ok();
                if let Some(size) = float_size_from_str(&self.extra_a) {
                    value.size_in_bits = size;
                }
            }
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.one_string_value.clone_from(&self.extra_a);
                value.zero_string_value.clone_from(&self.extra_b);
            }
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => {
                value.name.clone_from(&self.name);
                value.array_type_ref.clone_from(&self.base_or_ref);
                value.initial_value = optional_value(self.initial_value.clone());
                value.short_description = optional_value(self.short_description.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
                value.name.clone_from(&self.name);
                value.initial_value = optional_value(self.initial_value.clone());
                value.short_description = optional_value(self.short_description.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
        }
    }
}

fn default_nested_items(kind: ParameterTypeKind) -> &'static str {
    match kind {
        ParameterTypeKind::Array => "0 | 0",
        ParameterTypeKind::Aggregate => "member | MemberType |  | ",
        _ => "",
    }
}

fn encode_nested_items(parameter_type: Option<&xtce::ParameterTypeSetTypeContent>) -> String {
    match parameter_type {
        Some(xtce::ParameterTypeSetTypeContent::ArrayParameterType(value)) => value
            .dimension_list
            .dimension
            .iter()
            .map(|dimension| {
                format!(
                    "{} | {}",
                    integer_value_text(&dimension.starting_index),
                    integer_value_text(&dimension.ending_index)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Some(xtce::ParameterTypeSetTypeContent::AggregateParameterType(value)) => value
            .member_list
            .member
            .iter()
            .map(|member| {
                format!(
                    "{} | {} | {} | {}",
                    member.name,
                    member.type_ref,
                    member.initial_value.as_deref().unwrap_or_default(),
                    member.short_description.as_deref().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn integer_value_text(value: &xtce::IntegerValueType) -> String {
    match value {
        xtce::IntegerValueType::FixedValue(value) => value.to_string(),
        xtce::IntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::IntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
    }
}

fn apply_nested_items(parameter_type: &mut xtce::ParameterTypeSetTypeContent, input: &str) {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => {
            let rows = input
                .lines()
                .filter_map(|line| {
                    let (starting_index, ending_index) = line.split_once(" | ")?;
                    Some((
                        starting_index.trim().parse::<i64>().ok()?,
                        ending_index.trim().parse::<i64>().ok()?,
                    ))
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                return;
            }
            let mut existing = std::mem::take(&mut value.dimension_list.dimension).into_iter();
            value.dimension_list.dimension = rows
                .into_iter()
                .map(|(starting_index, ending_index)| {
                    let mut dimension = existing.next().unwrap_or(xtce::DimensionType {
                        starting_index: xtce::IntegerValueType::FixedValue(0),
                        ending_index: xtce::IntegerValueType::FixedValue(0),
                    });
                    dimension.starting_index = xtce::IntegerValueType::FixedValue(starting_index);
                    dimension.ending_index = xtce::IntegerValueType::FixedValue(ending_index);
                    dimension
                })
                .collect();
        }
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
            let rows = input
                .lines()
                .filter_map(|line| {
                    let mut columns = line.splitn(4, " | ").map(str::trim);
                    let name = columns.next()?.to_owned();
                    let type_ref = columns.next()?.to_owned();
                    if name.is_empty() || type_ref.is_empty() {
                        return None;
                    }
                    Some((
                        name,
                        type_ref,
                        optional_value(columns.next().unwrap_or_default().to_owned()),
                        optional_value(columns.next().unwrap_or_default().to_owned()),
                    ))
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                return;
            }
            let mut existing = std::mem::take(&mut value.member_list.member).into_iter();
            value.member_list.member = rows
                .into_iter()
                .map(|(name, type_ref, initial_value, short_description)| {
                    let mut member = existing.next().unwrap_or(xtce::MemberType {
                        short_description: None,
                        name: String::new(),
                        type_ref: String::new(),
                        initial_value: None,
                        long_description: None,
                        alias_set: None,
                        ancillary_data_set: None,
                    });
                    member.name = name;
                    member.type_ref = type_ref;
                    member.initial_value = initial_value;
                    member.short_description = short_description;
                    member
                })
                .collect();
        }
        _ => {}
    }
}

fn replace_parameter_type_kind(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    selected_kind: ParameterTypeKind,
) {
    if kind(parameter_type) == selected_kind {
        return;
    }

    *parameter_type = match selected_kind {
        ParameterTypeKind::String => {
            xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                restriction_pattern: None,
                character_width: None,
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Enumerated => {
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(
                xtce::EnumeratedParameterType {
                    short_description: None,
                    name: String::new(),
                    base_type: None,
                    initial_value: None,
                    content: Vec::new(),
                },
            )
        }
        ParameterTypeKind::Integer => {
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::IntegerParameterType::default_size_in_bits(),
                signed: xtce::IntegerParameterType::default_signed(),
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Binary => {
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(xtce::BinaryParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Float => {
            xtce::ParameterTypeSetTypeContent::FloatParameterType(xtce::FloatParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::FloatParameterType::default_size_in_bits(),
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Boolean => {
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(xtce::BooleanParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                one_string_value: xtce::BooleanParameterType::default_one_string_value(),
                zero_string_value: xtce::BooleanParameterType::default_zero_string_value(),
                content: Vec::new(),
            })
        }
        ParameterTypeKind::RelativeTime => {
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(
                xtce::RelativeTimeParameterType {
                    short_description: None,
                    name: String::new(),
                    base_type: None,
                    initial_value: None,
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                    encoding: None,
                    reference_time: None,
                    default_alarm: None,
                    context_alarm_list: None,
                },
            )
        }
        ParameterTypeKind::AbsoluteTime => {
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(
                xtce::AbsoluteTimeParameterType {
                    short_description: None,
                    name: String::new(),
                    base_type: None,
                    initial_value: None,
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                    encoding: None,
                    reference_time: None,
                },
            )
        }
        ParameterTypeKind::Array => {
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(xtce::ArrayParameterType {
                short_description: None,
                name: String::new(),
                array_type_ref: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                dimension_list: xtce::DimensionListType {
                    dimension: vec![xtce::DimensionType {
                        starting_index: xtce::IntegerValueType::FixedValue(0),
                        ending_index: xtce::IntegerValueType::FixedValue(0),
                    }],
                },
            })
        }
        ParameterTypeKind::Aggregate => xtce::ParameterTypeSetTypeContent::AggregateParameterType(
            xtce::AggregateParameterType {
                short_description: None,
                name: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                member_list: xtce::MemberListType {
                    member: vec![xtce::MemberType {
                        short_description: None,
                        name: "member".to_owned(),
                        type_ref: "MemberType".to_owned(),
                        initial_value: None,
                        long_description: None,
                        alias_set: None,
                        ancillary_data_set: None,
                    }],
                },
            },
        ),
    };
}

fn kind(parameter_type: &xtce::ParameterTypeSetTypeContent) -> ParameterTypeKind {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(_) => ParameterTypeKind::String,
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(_) => {
            ParameterTypeKind::Enumerated
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(_) => ParameterTypeKind::Integer,
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(_) => ParameterTypeKind::Binary,
        xtce::ParameterTypeSetTypeContent::FloatParameterType(_) => ParameterTypeKind::Float,
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(_) => ParameterTypeKind::Boolean,
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(_) => {
            ParameterTypeKind::RelativeTime
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(_) => {
            ParameterTypeKind::AbsoluteTime
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(_) => ParameterTypeKind::Array,
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(_) => {
            ParameterTypeKind::Aggregate
        }
    }
}

fn character_width_label(width: Option<&xtce::CharacterWidthType>) -> &'static str {
    match width {
        Some(xtce::CharacterWidthType::_8) => "8",
        Some(xtce::CharacterWidthType::_16) => "16",
        Some(xtce::CharacterWidthType::_32) => "32",
        None => "",
    }
}

fn character_width_from_str(value: &str) -> Option<xtce::CharacterWidthType> {
    match value.trim() {
        "8" => Some(xtce::CharacterWidthType::_8),
        "16" => Some(xtce::CharacterWidthType::_16),
        "32" => Some(xtce::CharacterWidthType::_32),
        _ => None,
    }
}

fn float_size_label(size: &xtce::FloatSizeInBitsType) -> &'static str {
    match size {
        xtce::FloatSizeInBitsType::_32 => "32",
        xtce::FloatSizeInBitsType::_64 => "64",
        xtce::FloatSizeInBitsType::_128 => "128",
    }
}

fn float_size_from_str(value: &str) -> Option<xtce::FloatSizeInBitsType> {
    match value.trim() {
        "32" => Some(xtce::FloatSizeInBitsType::_32),
        "64" => Some(xtce::FloatSizeInBitsType::_64),
        "128" => Some(xtce::FloatSizeInBitsType::_128),
        _ => None,
    }
}

fn bool_from_str(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

fn input(
    value: &str,
    multi_line: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let input = InputState::new(window, cx).default_value(value.to_owned());
        if multi_line {
            input.auto_grow(3, 12)
        } else {
            input
        }
    })
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

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        ParameterTypeKind, ParameterTypeValues, apply_nested_items, encode_nested_items,
        replace_parameter_type_kind,
    };

    #[test]
    fn applying_boolean_type_values_preserves_child_elements() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(xtce::BooleanParameterType {
                short_description: None,
                name: "OldFlagType".to_owned(),
                base_type: None,
                initial_value: None,
                one_string_value: "True".to_owned(),
                zero_string_value: "False".to_owned(),
                content: vec![xtce::BooleanParameterTypeContent::AncillaryDataSet(
                    xtce::AncillaryDataSetType {
                        ancillary_data: Vec::new(),
                    },
                )],
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        values.name = "OperationalFlagType".to_owned();
        values.initial_value = "Enabled".to_owned();
        values.extra_a = "Enabled".to_owned();
        values.extra_b = "Disabled".to_owned();
        values.apply_to(&mut parameter_type);

        match parameter_type {
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
                assert_eq!(value.name, "OperationalFlagType");
                assert_eq!(value.initial_value.as_deref(), Some("Enabled"));
                assert_eq!(value.one_string_value, "Enabled");
                assert_eq!(value.content.len(), 1);
            }
            _ => panic!("expected a BooleanParameterType"),
        }
    }

    #[test]
    fn applying_integer_type_values_updates_size_and_signedness() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        values.extra_a = "16".to_owned();
        values.extra_b = "false".to_owned();
        values.apply_to(&mut parameter_type);

        match parameter_type {
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
                assert_eq!(value.size_in_bits, 16);
                assert!(!value.signed);
            }
            _ => panic!("expected an IntegerParameterType"),
        }
    }

    #[test]
    fn changing_the_parameter_type_kind_keeps_editable_common_values() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: Some("Counter".to_owned()),
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: Some(3),
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        values.extra_a = "Enabled".to_owned();
        values.extra_b = "Disabled".to_owned();

        replace_parameter_type_kind(&mut parameter_type, ParameterTypeKind::Boolean);
        values.apply_to(&mut parameter_type);

        let xtce::ParameterTypeSetTypeContent::BooleanParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected a BooleanParameterType");
        };
        assert_eq!(parameter_type.name, "CounterType");
        assert_eq!(parameter_type.short_description.as_deref(), Some("Counter"));
        assert_eq!(parameter_type.initial_value.as_deref(), Some("3"));
        assert_eq!(parameter_type.one_string_value, "Enabled");
        assert_eq!(parameter_type.zero_string_value, "Disabled");
    }

    #[test]
    fn selecting_aggregate_creates_its_required_member_list() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "ValueType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });

        replace_parameter_type_kind(&mut parameter_type, ParameterTypeKind::Aggregate);

        let xtce::ParameterTypeSetTypeContent::AggregateParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected an AggregateParameterType");
        };
        assert_eq!(parameter_type.member_list.member.len(), 1);
        assert_eq!(parameter_type.member_list.member[0].name, "member");
    }

    #[test]
    fn base_field_matches_the_selected_parameter_type() {
        assert_eq!(
            ParameterTypeKind::Integer.base_field(),
            Some(("Base type", "Optional; used only for type inheritance"))
        );
        assert_eq!(
            ParameterTypeKind::Array.base_field(),
            Some(("Array type reference", "Required"))
        );
        assert_eq!(ParameterTypeKind::Aggregate.base_field(), None);
    }

    #[test]
    fn aggregate_member_fields_can_be_edited_without_losing_hidden_metadata() {
        let mut parameter_type = xtce::ParameterTypeSetTypeContent::AggregateParameterType(
            xtce::AggregateParameterType {
                short_description: None,
                name: "StatusType".to_owned(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                member_list: xtce::MemberListType {
                    member: vec![xtce::MemberType {
                        short_description: None,
                        name: "old_name".to_owned(),
                        type_ref: "OldType".to_owned(),
                        initial_value: None,
                        long_description: Some("Keep this".to_owned()),
                        alias_set: None,
                        ancillary_data_set: None,
                    }],
                },
            },
        );

        apply_nested_items(&mut parameter_type, "mode | ModeType | SAFE | Current mode");

        let xtce::ParameterTypeSetTypeContent::AggregateParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected an AggregateParameterType");
        };
        let member = &parameter_type.member_list.member[0];
        assert_eq!(member.name, "mode");
        assert_eq!(member.type_ref, "ModeType");
        assert_eq!(member.initial_value.as_deref(), Some("SAFE"));
        assert_eq!(member.long_description.as_deref(), Some("Keep this"));
    }

    #[test]
    fn array_dimensions_can_be_edited_from_form_rows() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(xtce::ArrayParameterType {
                short_description: None,
                name: "SamplesType".to_owned(),
                array_type_ref: "SampleType".to_owned(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                dimension_list: xtce::DimensionListType {
                    dimension: vec![xtce::DimensionType {
                        starting_index: xtce::IntegerValueType::FixedValue(0),
                        ending_index: xtce::IntegerValueType::FixedValue(0),
                    }],
                },
            });

        apply_nested_items(&mut parameter_type, "1 | 8\n0 | 3");

        assert_eq!(encode_nested_items(Some(&parameter_type)), "1 | 8\n0 | 3");
    }
}
