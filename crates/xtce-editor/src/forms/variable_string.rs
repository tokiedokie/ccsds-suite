use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    prelude::FluentBuilder,
};
use gpui_component::{
    IndexPath, h_flex,
    input::InputState,
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    discrete_lookup::DiscreteLookupListForm, dynamic_value::DynamicValueForm, field,
    impl_select_item,
};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum VariableKind {
    #[default]
    DynamicValue,
    DiscreteLookup,
    LeadingSize,
    TerminationChar,
}
impl_select_item!(VariableKind);

pub(super) struct VariableStringForm {
    max_size_in_bits: Entity<InputState>,
    kind: VariableKind,
    kind_select: Entity<SelectState<Vec<VariableKind>>>,
    dynamic_value: Entity<DynamicValueForm>,
    discrete_lookup: Entity<DiscreteLookupListForm>,
    leading_size: Entity<InputState>,
    termination_char: Entity<InputState>,
}

impl VariableStringForm {
    pub(super) fn new(
        value: Option<&xtce::VariableStringType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = VariableValues::from_value(value);
        let dynamic_value = DynamicValueForm::new(values.dynamic_value, window, cx);
        let discrete_lookup = DiscreteLookupListForm::new(values.discrete_lookup, window, cx);
        cx.new(move |cx| {
            let kind_select = select(values.kind, window, cx);
            let subscription = cx.subscribe_in(
                &kind_select,
                window,
                |this: &mut Self, _, event: &SelectEvent<Vec<VariableKind>>, _, cx| {
                    if let SelectEvent::Confirm(Some(kind)) = event {
                        this.kind = *kind;
                        cx.notify();
                    }
                },
            );
            subscription.detach();
            Self {
                max_size_in_bits: input(&values.max_size_in_bits, window, cx),
                kind: values.kind,
                kind_select,
                dynamic_value,
                discrete_lookup,
                leading_size: input(&values.leading_size, window, cx),
                termination_char: input(&values.termination_char, window, cx),
            }
        })
    }

    pub(super) fn load(
        &mut self,
        value: Option<&xtce::VariableStringType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = VariableValues::from_value(value);
        self.kind = values.kind;
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&values.kind, window, cx);
        });
        for (input, value) in [
            (&self.max_size_in_bits, values.max_size_in_bits),
            (&self.leading_size, values.leading_size),
            (&self.termination_char, values.termination_char),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.dynamic_value
            .update(cx, |form, cx| form.load(values.dynamic_value, window, cx));
        self.discrete_lookup
            .update(cx, |form, cx| form.load(values.discrete_lookup, window, cx));
        cx.notify();
    }

    pub(super) fn value(&self, cx: &App) -> xtce::VariableStringType {
        let content = match self.kind {
            VariableKind::DynamicValue => {
                xtce::VariableStringTypeContent::DynamicValue(self.dynamic_value.read(cx).value(cx))
            }
            VariableKind::DiscreteLookup => xtce::VariableStringTypeContent::DiscreteLookupList(
                self.discrete_lookup.read(cx).value(cx),
            ),
            VariableKind::LeadingSize => {
                xtce::VariableStringTypeContent::LeadingSize(xtce::LeadingSizeType {
                    size_in_bits_of_size_tag: integer(&self.leading_size, cx),
                })
            }
            VariableKind::TerminationChar => xtce::VariableStringTypeContent::TerminationChar(
                input_value(&self.termination_char, cx),
            ),
        };
        xtce::VariableStringType {
            max_size_in_bits: integer(&self.max_size_in_bits, cx),
            content: vec![content],
        }
    }
}

impl Render for VariableStringForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(field(
                        "Maximum size in bits",
                        "Required",
                        &self.max_size_in_bits,
                        cx,
                    ))
                    .child(select_field("Size source", &self.kind_select)),
            )
            .when(self.kind == VariableKind::DynamicValue, |form| {
                form.child(self.dynamic_value.clone())
            })
            .when(self.kind == VariableKind::DiscreteLookup, |form| {
                form.child(self.discrete_lookup.clone())
            })
            .when(self.kind == VariableKind::LeadingSize, |form| {
                form.child(field(
                    "Size tag width in bits",
                    "Defaults to 16",
                    &self.leading_size,
                    cx,
                ))
            })
            .when(self.kind == VariableKind::TerminationChar, |form| {
                form.child(field(
                    "Termination character",
                    "Required",
                    &self.termination_char,
                    cx,
                ))
            })
    }
}

struct VariableValues<'a> {
    max_size_in_bits: String,
    kind: VariableKind,
    dynamic_value: Option<&'a xtce::DynamicValueType>,
    discrete_lookup: Option<&'a xtce::DiscreteLookupListType>,
    leading_size: String,
    termination_char: String,
}

impl<'a> VariableValues<'a> {
    fn from_value(value: Option<&'a xtce::VariableStringType>) -> Self {
        let first = value.and_then(|value| value.content.first());
        Self {
            max_size_in_bits: value
                .map(|value| value.max_size_in_bits.to_string())
                .unwrap_or_default(),
            kind: match first {
                Some(xtce::VariableStringTypeContent::DiscreteLookupList(_)) => {
                    VariableKind::DiscreteLookup
                }
                Some(xtce::VariableStringTypeContent::LeadingSize(_)) => VariableKind::LeadingSize,
                Some(xtce::VariableStringTypeContent::TerminationChar(_)) => {
                    VariableKind::TerminationChar
                }
                _ => VariableKind::DynamicValue,
            },
            dynamic_value: first.and_then(|content| match content {
                xtce::VariableStringTypeContent::DynamicValue(value) => Some(value),
                _ => None,
            }),
            discrete_lookup: first.and_then(|content| match content {
                xtce::VariableStringTypeContent::DiscreteLookupList(value) => Some(value),
                _ => None,
            }),
            leading_size: first
                .and_then(|content| match content {
                    xtce::VariableStringTypeContent::LeadingSize(value) => {
                        Some(value.size_in_bits_of_size_tag.to_string())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| {
                    xtce::LeadingSizeType::default_size_in_bits_of_size_tag().to_string()
                }),
            termination_char: first
                .and_then(|content| match content {
                    xtce::VariableStringTypeContent::TerminationChar(value) => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_default(),
        }
    }
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn input_value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn integer(input: &Entity<InputState>, cx: &App) -> i64 {
    input_value(input, cx).trim().parse().unwrap_or_default()
}

fn select(
    selected: VariableKind,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<VariableKind>>> {
    let index = VariableKind::VARIANTS
        .iter()
        .position(|value| *value == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            VariableKind::VARIANTS.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn select_field(label: &'static str, select: &Entity<SelectState<Vec<VariableKind>>>) -> gpui::Div {
    super::select_field(label, "Required", select)
}

#[cfg(test)]
mod tests {
    use super::{VariableKind, VariableValues};

    #[test]
    fn leading_size_fields_are_loaded() {
        let value = xtce::VariableStringType {
            max_size_in_bits: 1024,
            content: vec![xtce::VariableStringTypeContent::LeadingSize(
                xtce::LeadingSizeType {
                    size_in_bits_of_size_tag: 12,
                },
            )],
        };

        let values = VariableValues::from_value(Some(&value));

        assert_eq!(values.kind, VariableKind::LeadingSize);
        assert_eq!(values.max_size_in_bits, "1024");
        assert_eq!(values.leading_size, "12");
    }
}
