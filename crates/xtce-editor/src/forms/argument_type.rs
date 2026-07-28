use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window,
};
use gpui_component::{
    IconName, IndexPath, Sizable,
    button::{Button, ButtonVariants},
    collapsible::Collapsible,
    h_flex,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    data_encoding::{
        DataEncodingForm, find_argument_data_encoding, find_argument_data_encoding_mut,
        set_argument_data_encoding_kind,
    },
    enumeration_list::EnumerationListForm,
    field, impl_select_item, optional_value,
};
use crate::XtceEditor;

macro_rules! long_description {
    ($content:expr, $type:ident) => {
        $content
            .iter()
            .find_map(|item| match item {
                xtce::$type::LongDescription(value) => Some(value.as_str()),
                _ => None,
            })
            .unwrap_or_default()
    };
}

macro_rules! set_long_description {
    ($content:expr, $type:ident, $description:expr) => {{
        let existing = $content.iter_mut().find_map(|item| match item {
            xtce::$type::LongDescription(value) => Some(value),
            _ => None,
        });
        if let Some(existing) = existing {
            existing.clone_from($description);
        } else if !$description.is_empty() {
            $content.insert(0, xtce::$type::LongDescription($description.to_owned()));
        }
        if $description.is_empty() {
            $content.retain(|item| !matches!(item, xtce::$type::LongDescription(_)));
        }
    }};
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ArgumentKind {
    #[strum(serialize = "StringArgumentType")]
    String,
    #[strum(serialize = "EnumeratedArgumentType")]
    Enumerated,
    #[strum(serialize = "IntegerArgumentType")]
    Integer,
    #[strum(serialize = "BinaryArgumentType")]
    Binary,
    #[strum(serialize = "FloatArgumentType")]
    Float,
    #[strum(serialize = "BooleanArgumentType")]
    Boolean,
    #[strum(serialize = "RelativeTimeArgumentType")]
    RelativeTime,
    #[strum(serialize = "AbsoluteTimeArgumentType")]
    AbsoluteTime,
    #[strum(serialize = "ArrayArgumentType")]
    Array,
    #[strum(serialize = "AggregateArgumentType")]
    Aggregate,
}
impl_select_item!(ArgumentKind);

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
enum SignedChoice {
    #[strum(serialize = "true")]
    Signed,
    #[strum(serialize = "false")]
    Unsigned,
}
impl_select_item!(SignedChoice);

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

pub(super) struct ArgumentTypeForm {
    kind_select: Entity<SelectState<Vec<ArgumentKind>>>,
    name_input: Entity<InputState>,
    base_or_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    extra_a_input: Entity<InputState>,
    extra_b_input: Entity<InputState>,
    nested_items_input: Entity<InputState>,
    character_width_select: Entity<SelectState<Vec<CharacterWidthChoice>>>,
    signed_select: Entity<SelectState<Vec<SignedChoice>>>,
    float_size_select: Entity<SelectState<Vec<FloatSizeChoice>>>,
    enumeration_list: Entity<EnumerationListForm>,
    data_encoding: Entity<DataEncodingForm>,
    base_defaults_open: bool,
    documentation_open: bool,
    type_options_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl ArgumentTypeForm {
    pub(super) fn new(
        argument_type: Option<&xtce::ArgumentTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = ArgumentValues::from_type(argument_type);
        let selected_kind = argument_type.map(kind).unwrap_or(ArgumentKind::String);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| {
            let kind_select = select(ArgumentKind::VARIANTS, selected_kind, window, cx);
            let kind_subscription = cx.subscribe(
                &kind_select,
                |_, _, _: &SelectEvent<Vec<ArgumentKind>>, cx| cx.notify(),
            );
            Self {
                kind_select,
                name_input,
                base_or_ref_input: input(&values.base_or_ref, false, window, cx),
                initial_value_input: input(&values.initial_value, false, window, cx),
                short_description_input: input(&values.short_description, false, window, cx),
                long_description_input: input(&values.long_description, true, window, cx),
                extra_a_input: input(&values.extra_a, false, window, cx),
                extra_b_input: input(&values.extra_b, false, window, cx),
                nested_items_input: input(&encode_nested_items(argument_type), true, window, cx),
                character_width_select: select(
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
                ),
                signed_select: select(
                    SignedChoice::VARIANTS,
                    parse_choice(&values.extra_b, SignedChoice::Signed),
                    window,
                    cx,
                ),
                float_size_select: select(
                    FloatSizeChoice::VARIANTS,
                    parse_choice(&values.extra_a, FloatSizeChoice::_32),
                    window,
                    cx,
                ),
                enumeration_list: EnumerationListForm::new_argument_type(
                    argument_type,
                    window,
                    cx,
                ),
                data_encoding: DataEncodingForm::new(
                    argument_type.and_then(find_argument_data_encoding),
                    window,
                    cx,
                ),
                base_defaults_open: false,
                documentation_open: false,
                type_options_open: false,
                _subscriptions: vec![name_subscription, kind_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        argument_type: Option<&xtce::ArgumentTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ArgumentValues::from_type(argument_type);
        self.base_defaults_open = false;
        self.documentation_open = false;
        self.type_options_open = false;
        let selected_kind = argument_type.map(kind).unwrap_or(ArgumentKind::String);
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&selected_kind, window, cx);
        });
        let character_width = parse_choice(
            if values.extra_b.is_empty() {
                "Default"
            } else {
                &values.extra_b
            },
            CharacterWidthChoice::Default,
        );
        let signed = parse_choice(&values.extra_b, SignedChoice::Signed);
        let float_size = parse_choice(&values.extra_a, FloatSizeChoice::_32);
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.base_or_ref_input, values.base_or_ref),
            (&self.initial_value_input, values.initial_value),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.extra_a_input, values.extra_a),
            (&self.extra_b_input, values.extra_b),
            (&self.nested_items_input, encode_nested_items(argument_type)),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        sync_select(&self.character_width_select, character_width, window, cx);
        sync_select(&self.signed_select, signed, window, cx);
        sync_select(&self.float_size_select, float_size, window, cx);
        self.enumeration_list.update(cx, |form, cx| {
            form.load_argument_type(argument_type, cx);
        });
        self.data_encoding.update(cx, |form, cx| {
            form.load(
                argument_type.and_then(find_argument_data_encoding),
                window,
                cx,
            );
        });
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(gpui::px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn apply_to(&self, argument_type: &mut xtce::ArgumentTypeSetTypeContent, cx: &App) {
        let selected_kind = self.selected_kind(cx);
        replace_kind(argument_type, selected_kind);
        let extra_a = match selected_kind {
            ArgumentKind::Float => {
                selected_value(&self.float_size_select, FloatSizeChoice::_32, cx).to_string()
            }
            _ => value(&self.extra_a_input, cx),
        };
        let extra_b = match selected_kind {
            ArgumentKind::String => {
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
            ArgumentKind::Integer => {
                selected_value(&self.signed_select, SignedChoice::Signed, cx).to_string()
            }
            _ => value(&self.extra_b_input, cx),
        };
        ArgumentValues {
            name: value(&self.name_input, cx),
            base_or_ref: value(&self.base_or_ref_input, cx),
            initial_value: value(&self.initial_value_input, cx),
            short_description: value(&self.short_description_input, cx),
            long_description: value(&self.long_description_input, cx),
            extra_a,
            extra_b,
        }
        .apply_to(argument_type);
        apply_nested_items(argument_type, &value(&self.nested_items_input, cx));
        self.enumeration_list.read(cx).apply_to_argument_type(argument_type, cx);
        set_argument_data_encoding_kind(argument_type, self.data_encoding.read(cx).selected_kind(cx));
        if let Some(encoding) = find_argument_data_encoding_mut(argument_type) {
            self.data_encoding.read(cx).apply_to(encoding, cx);
        }
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        let kind = self.selected_kind(cx);
        let mut form = v_flex().gap_5().child(select_field(
            "Argument type",
            "Required",
            &self.kind_select,
            cx,
        ));
        if kind == ArgumentKind::Array {
            form = form.child(field(
                "Array type reference",
                "Required",
                &self.base_or_ref_input,
                cx,
            ));
        }
        match kind {
            ArgumentKind::String => {
                form = form.child(self.type_options(cx));
            }
            ArgumentKind::Integer => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field("Size in bits", "Required", &self.extra_a_input, cx))
                        .child(select_field("Signed", "Required", &self.signed_select, cx)),
                );
            }
            ArgumentKind::Float => {
                form = form.child(select_field(
                    "Size in bits",
                    "Required",
                    &self.float_size_select,
                    cx,
                ));
            }
            ArgumentKind::Boolean => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field(
                            "One string value",
                            "Required",
                            &self.extra_a_input,
                            cx,
                        ))
                        .child(field(
                            "Zero string value",
                            "Required",
                            &self.extra_b_input,
                            cx,
                        )),
                );
            }
            ArgumentKind::Enumerated => {
                form = form.child(self.enumeration_list.clone());
            }
            ArgumentKind::Array => {
                form = form.child(field(
                    "Dimensions",
                    "One starting index | ending index per line",
                    &self.nested_items_input,
                    cx,
                ));
            }
            ArgumentKind::Aggregate => {
                form = form.child(field(
                    "Members",
                    "name | type ref | initial value | short description",
                    &self.nested_items_input,
                    cx,
                ));
            }
            _ => {}
        }
        if kind != ArgumentKind::Aggregate && kind != ArgumentKind::Array {
            form = form.child(self.data_encoding.clone());
        }
        form.child(self.base_defaults(cx))
            .child(self.documentation(cx))
    }

    fn type_options(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.type_options_open)
            .child(
                Button::new("toggle-argument-type-options")
                    .small()
                    .link()
                    .icon(if self.type_options_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Type options")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.type_options_open = !this.type_options_open;
                        cx.notify();
                    })),
            )
            .content(
                h_flex()
                    .pt_3()
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
            )
    }

    fn base_defaults(&self, cx: &mut Context<Self>) -> Collapsible {
        let mut content = h_flex().pt_3().gap_4().items_start();
        if self.selected_kind(cx) != ArgumentKind::Array
            && let Some((label, hint)) = self.selected_kind(cx).base_field()
        {
            content = content.child(field(label, hint, &self.base_or_ref_input, cx));
        }
        content = content.child(field(
            "Initial value",
            "Optional",
            &self.initial_value_input,
            cx,
        ));
        Collapsible::new()
            .open(self.base_defaults_open)
            .child(
                Button::new("toggle-argument-type-base-defaults")
                    .small()
                    .link()
                    .icon(if self.base_defaults_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Base and defaults")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.base_defaults_open = !this.base_defaults_open;
                        cx.notify();
                    })),
            )
            .content(content)
    }

    fn documentation(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.documentation_open)
            .child(
                Button::new("toggle-argument-type-documentation")
                    .small()
                    .link()
                    .icon(if self.documentation_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Documentation")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.documentation_open = !this.documentation_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_4()
                    .child(field(
                        "Short description",
                        "Optional",
                        &self.short_description_input,
                        cx,
                    ))
                    .child(field(
                        "Long description",
                        "Optional",
                        &self.long_description_input,
                        cx,
                    )),
            )
    }

    fn selected_kind(&self, cx: &App) -> ArgumentKind {
        self.kind_select
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(ArgumentKind::String)
    }
}

impl Render for ArgumentTypeForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

impl ArgumentKind {
    fn base_field(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Array => Some(("Array type reference", "Required")),
            Self::Aggregate => None,
            _ => Some(("Base type", "Optional; used only for type inheritance")),
        }
    }
}

struct ArgumentValues {
    name: String,
    base_or_ref: String,
    initial_value: String,
    short_description: String,
    long_description: String,
    extra_a: String,
    extra_b: String,
}

impl ArgumentValues {
    fn from_type(value: Option<&xtce::ArgumentTypeSetTypeContent>) -> Self {
        let Some(value) = value else {
            return Self::common("", None, None, None, "", "", "");
        };
        macro_rules! common_content {
            ($value:expr, $content:ident) => {
                Self::common(
                    &$value.name,
                    $value.base_type.as_deref(),
                    $value
                        .initial_value
                        .as_ref()
                        .map(ToString::to_string)
                        .as_deref(),
                    $value.short_description.as_deref(),
                    long_description!($value.content, $content),
                    "",
                    "",
                )
            };
        }
        match value {
            xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
                let mut values = common_content!(value, StringArgumentTypeContent);
                values.extra_a = value.restriction_pattern.clone().unwrap_or_default();
                values.extra_b = character_width_label(value.character_width.as_ref()).to_owned();
                values
            }
            xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
                common_content!(value, EnumeratedArgumentTypeContent)
            }
            xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
                let mut values = common_content!(value, IntegerArgumentTypeContent);
                values.extra_a = value.size_in_bits.to_string();
                values.extra_b = value.signed.to_string();
                values
            }
            xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
                common_content!(value, BinaryArgumentTypeContent)
            }
            xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
                let mut values = common_content!(value, FloatArgumentTypeContent);
                values.extra_a = float_size_label(&value.size_in_bits).to_owned();
                values
            }
            xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
                let mut values = common_content!(value, BooleanArgumentTypeContent);
                values.extra_a.clone_from(&value.one_string_value);
                values.extra_b.clone_from(&value.zero_string_value);
                values
            }
            xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => Self::common(
                &value.name,
                Some(&value.array_type_ref),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => Self::common(
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

    fn apply_to(&self, value: &mut xtce::ArgumentTypeSetTypeContent) {
        macro_rules! common {
            ($value:expr) => {{
                $value.name.clone_from(&self.name);
                $value.base_type = optional_value(self.base_or_ref.clone());
                $value.short_description = optional_value(self.short_description.clone());
            }};
        }
        macro_rules! content {
            ($value:expr, $content:ident) => {{
                common!($value);
                set_long_description!($value.content, $content, &self.long_description);
            }};
        }
        match value {
            xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
                content!(value, StringArgumentTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
                value.restriction_pattern = optional_value(self.extra_a.clone());
                value.character_width = character_width_from_str(&self.extra_b);
            }
            xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
                content!(value, EnumeratedArgumentTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
            }
            xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
                content!(value, IntegerArgumentTypeContent);
                value.initial_value = self.initial_value.parse().ok();
                value.size_in_bits = self.extra_a.parse().unwrap_or(32);
                value.signed = self.extra_b.parse().unwrap_or(true);
            }
            xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
                content!(value, BinaryArgumentTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
            }
            xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
                content!(value, FloatArgumentTypeContent);
                value.initial_value = self.initial_value.parse().ok();
                value.size_in_bits =
                    float_size_from_str(&self.extra_a).unwrap_or(xtce::FloatSizeInBitsType::_32);
            }
            xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
                content!(value, BooleanArgumentTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
                value.one_string_value.clone_from(&self.extra_a);
                value.zero_string_value.clone_from(&self.extra_b);
            }
            xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => {
                value.name.clone_from(&self.name);
                value.array_type_ref.clone_from(&self.base_or_ref);
                value.initial_value = optional_value(self.initial_value.clone());
                value.short_description = optional_value(self.short_description.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => {
                value.name.clone_from(&self.name);
                value.initial_value = optional_value(self.initial_value.clone());
                value.short_description = optional_value(self.short_description.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
        }
    }
}

fn kind(value: &xtce::ArgumentTypeSetTypeContent) -> ArgumentKind {
    match value {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(_) => ArgumentKind::String,
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(_) => ArgumentKind::Enumerated,
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(_) => ArgumentKind::Integer,
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(_) => ArgumentKind::Binary,
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(_) => ArgumentKind::Float,
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(_) => ArgumentKind::Boolean,
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(_) => ArgumentKind::RelativeTime,
        xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(_) => ArgumentKind::AbsoluteTime,
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(_) => ArgumentKind::Array,
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(_) => ArgumentKind::Aggregate,
    }
}

fn replace_kind(value: &mut xtce::ArgumentTypeSetTypeContent, selected: ArgumentKind) {
    if kind(value) == selected {
        return;
    }
    *value = default_argument_type(selected);
}

fn default_argument_type(kind: ArgumentKind) -> xtce::ArgumentTypeSetTypeContent {
    match kind {
        ArgumentKind::String => {
            xtce::ArgumentTypeSetTypeContent::StringArgumentType(xtce::StringArgumentType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                restriction_pattern: None,
                character_width: None,
                content: Vec::new(),
            })
        }
        ArgumentKind::Enumerated => {
            xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(xtce::EnumeratedArgumentType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                content: Vec::new(),
            })
        }
        ArgumentKind::Integer => {
            xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(xtce::IntegerArgumentType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            })
        }
        ArgumentKind::Binary => {
            xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(xtce::BinaryArgumentType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                content: Vec::new(),
            })
        }
        ArgumentKind::Float => {
            xtce::ArgumentTypeSetTypeContent::FloatArgumentType(xtce::FloatArgumentType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::FloatSizeInBitsType::_32,
                content: Vec::new(),
            })
        }
        ArgumentKind::Boolean => {
            xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(xtce::BooleanArgumentType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                one_string_value: "True".to_owned(),
                zero_string_value: "False".to_owned(),
                content: Vec::new(),
            })
        }
        ArgumentKind::RelativeTime => xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(
            xtce::RelativeTimeArgumentType {
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
        ),
        ArgumentKind::AbsoluteTime => xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(
            xtce::AbsoluteTimeArgumentType {
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
        ),
        ArgumentKind::Array => {
            xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(xtce::ArrayArgumentType {
                short_description: None,
                name: String::new(),
                array_type_ref: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                dimension_list: xtce::ArgumentDimensionListType {
                    dimension: vec![xtce::ArgumentDimensionType {
                        starting_index: xtce::ArgumentIntegerValueType::FixedValue(0),
                        ending_index: xtce::ArgumentIntegerValueType::FixedValue(0),
                    }],
                },
            })
        }
        ArgumentKind::Aggregate => {
            xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(xtce::AggregateArgumentType {
                short_description: None,
                name: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                member_list: xtce::MemberListType { member: Vec::new() },
            })
        }
    }
}

fn encode_nested_items(value: Option<&xtce::ArgumentTypeSetTypeContent>) -> String {
    match value {
        Some(xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value)) => value
            .dimension_list
            .dimension
            .iter()
            .map(|dimension| {
                format!(
                    "{} | {}",
                    argument_integer(&dimension.starting_index),
                    argument_integer(&dimension.ending_index)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Some(xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value)) => value
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

fn apply_nested_items(value: &mut xtce::ArgumentTypeSetTypeContent, input: &str) {
    match value {
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => {
            let rows = input
                .lines()
                .filter_map(|line| {
                    let (start, end) = line.split_once(" | ")?;
                    Some((start.trim().parse().ok()?, end.trim().parse().ok()?))
                })
                .collect::<Vec<(i64, i64)>>();
            if !rows.is_empty() {
                value.dimension_list.dimension = rows
                    .into_iter()
                    .map(|(start, end)| xtce::ArgumentDimensionType {
                        starting_index: xtce::ArgumentIntegerValueType::FixedValue(start),
                        ending_index: xtce::ArgumentIntegerValueType::FixedValue(end),
                    })
                    .collect();
            }
        }
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => {
            let rows = input
                .lines()
                .filter_map(|line| {
                    let mut fields = line.splitn(4, " | ").map(str::trim);
                    let name = fields.next()?.to_owned();
                    let type_ref = fields.next()?.to_owned();
                    (!name.is_empty() && !type_ref.is_empty()).then(|| {
                        (
                            name,
                            type_ref,
                            optional_value(fields.next().unwrap_or_default().to_owned()),
                            optional_value(fields.next().unwrap_or_default().to_owned()),
                        )
                    })
                })
                .collect::<Vec<_>>();
            if !rows.is_empty() {
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
        }
        _ => {}
    }
}

fn argument_integer(value: &xtce::ArgumentIntegerValueType) -> String {
    match value {
        xtce::ArgumentIntegerValueType::FixedValue(value) => value.to_string(),
        xtce::ArgumentIntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::ArgumentIntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
    }
}

fn character_width_label(value: Option<&xtce::CharacterWidthType>) -> &'static str {
    match value {
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

fn float_size_label(value: &xtce::FloatSizeInBitsType) -> &'static str {
    match value {
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

fn parse_choice<T>(value: &str, fallback: T) -> T
where
    T: std::str::FromStr,
{
    value.parse().unwrap_or(fallback)
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
        .position(|value| value == &selected)
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

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{ArgumentKind, ArgumentValues, default_argument_type, kind, replace_kind};

    #[test]
    fn changing_argument_kind_keeps_form_name() {
        let mut value = default_argument_type(ArgumentKind::String);
        replace_kind(&mut value, ArgumentKind::Integer);
        ArgumentValues {
            name: "CounterArgument".to_owned(),
            base_or_ref: String::new(),
            initial_value: "4".to_owned(),
            short_description: String::new(),
            long_description: String::new(),
            extra_a: "16".to_owned(),
            extra_b: "false".to_owned(),
        }
        .apply_to(&mut value);

        assert_eq!(kind(&value), ArgumentKind::Integer);
        let xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) = value else {
            panic!("expected integer argument")
        };
        assert_eq!(value.name, "CounterArgument");
        assert_eq!(value.size_in_bits, 16);
        assert!(!value.signed);
    }

    #[test]
    fn argument_data_encoding_round_trip() {
        use super::super::data_encoding::{
            DataEncodingKind, find_argument_data_encoding, find_argument_data_encoding_mut,
            set_argument_data_encoding_kind,
        };

        let mut value = default_argument_type(ArgumentKind::Integer);
        set_argument_data_encoding_kind(&mut value, Some(DataEncodingKind::Integer));

        let encoding = find_argument_data_encoding(&value).expect("data encoding present");
        assert_eq!(encoding.kind(), DataEncodingKind::Integer);

        let mut_encoding = find_argument_data_encoding_mut(&mut value).expect("mut encoding present");
        assert!(matches!(mut_encoding, super::super::data_encoding::DataEncodingMut::Integer(_)));
    }

    #[test]
    fn argument_enumeration_list_round_trip() {
        let mut value = default_argument_type(ArgumentKind::Enumerated);
        let xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(enum_type) = &mut value else {
            panic!("expected enumerated argument")
        };
        enum_type.content.push(xtce::EnumeratedArgumentTypeContent::EnumerationList(
            xtce::EnumerationListType {
                enumeration: vec![xtce::ValueEnumerationType {
                    value: 1,
                    max_value: None,
                    label: "STATUS_OK".to_string(),
                    short_description: Some("Ok".to_string()),
                }],
            },
        ));

        let rows = super::super::enumeration_list::rows_from_argument_type(Some(&value));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "STATUS_OK");
        assert_eq!(rows[0].value, "1");
    }
}
