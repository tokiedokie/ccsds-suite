use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Task, Window,
};
use gpui_component::{
    h_flex,
    input::{CompletionProvider, Input, InputEvent, InputState, Rope, RopeExt},
    v_flex,
};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    Position, Range, TextEdit,
};

use super::{field, optional_value};
use crate::XtceEditor;

#[derive(Clone, Copy)]
enum ParameterElementKind {
    Parameter,
    Reference,
    Missing,
}

pub(super) struct ParameterForm {
    element_kind: ParameterElementKind,
    name_input: Entity<InputState>,
    parameter_type_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    parameter_ref_input: Entity<InputState>,
    parameter_type_names: Rc<RefCell<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl ParameterForm {
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
        parameter: Option<&xtce::ParameterSetTypeContent>,
        parameter_type_set: Option<&xtce::ParameterTypeSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = ParameterValues::from_parameter(parameter);
        let parameter_type_names = Rc::new(RefCell::new(parameter_type_names(parameter_type_set)));
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| {
            let parameter_type_ref_input = cx.new(|cx| {
                let mut input = InputState::new(window, cx)
                    .default_value(values.parameter_type_ref.clone())
                    .placeholder("Start typing a ParameterType name");
                input.lsp.completion_provider = Some(Rc::new(ParameterTypeCompletionProvider {
                    names: parameter_type_names.clone(),
                }));
                input
            });
            Self {
                element_kind: match parameter {
                    Some(xtce::ParameterSetTypeContent::Parameter(_)) => {
                        ParameterElementKind::Parameter
                    }
                    Some(xtce::ParameterSetTypeContent::ParameterRef(_)) => {
                        ParameterElementKind::Reference
                    }
                    None => ParameterElementKind::Missing,
                },
                name_input,
                parameter_type_ref_input,
                initial_value_input: input(&values.initial_value, false, window, cx),
                short_description_input: input(&values.short_description, false, window, cx),
                long_description_input: input(&values.long_description, true, window, cx),
                parameter_ref_input: input(&values.parameter_ref, false, window, cx),
                parameter_type_names,
                _subscriptions: vec![name_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        parameter: Option<&xtce::ParameterSetTypeContent>,
        parameter_type_set: Option<&xtce::ParameterTypeSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        *self.parameter_type_names.borrow_mut() = parameter_type_names(parameter_type_set);
        self.element_kind = match parameter {
            Some(xtce::ParameterSetTypeContent::Parameter(_)) => ParameterElementKind::Parameter,
            Some(xtce::ParameterSetTypeContent::ParameterRef(_)) => ParameterElementKind::Reference,
            None => ParameterElementKind::Missing,
        };
        let values = ParameterValues::from_parameter(parameter);
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.parameter_type_ref_input, values.parameter_type_ref),
            (&self.initial_value_input, values.initial_value),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.parameter_ref_input, values.parameter_ref),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    pub(super) fn apply_to(&self, parameter: &mut xtce::ParameterSetTypeContent, cx: &App) {
        ParameterValues {
            name: value(&self.name_input, cx),
            parameter_type_ref: value(&self.parameter_type_ref_input, cx),
            initial_value: value(&self.initial_value_input, cx),
            short_description: value(&self.short_description_input, cx),
            long_description: value(&self.long_description_input, cx),
            parameter_ref: value(&self.parameter_ref_input, cx),
        }
        .apply_to(parameter);
    }

    fn render_form(&self, cx: &App) -> Div {
        match self.element_kind {
            ParameterElementKind::Parameter => v_flex()
                .gap_5()
                .child(h_flex().gap_4().items_start().child(field(
                    "Parameter type reference",
                    "Required",
                    &self.parameter_type_ref_input,
                    cx,
                )))
                .child(
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
                )
                .child(field(
                    "Long description",
                    "Optional",
                    &self.long_description_input,
                    cx,
                )),
            ParameterElementKind::Reference => field(
                "Parameter reference",
                "Required",
                &self.parameter_ref_input,
                cx,
            ),
            ParameterElementKind::Missing => v_flex(),
        }
    }
}

impl Render for ParameterForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

struct ParameterTypeCompletionProvider {
    names: Rc<RefCell<Vec<String>>>,
}

impl CompletionProvider for ParameterTypeCompletionProvider {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        _: CompletionContext,
        _: &mut Window,
        _: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        let query = text.slice(..offset).to_string();
        let normalized_query = query.to_ascii_lowercase();
        let end = text.offset_to_position(offset);
        let names = self.names.borrow();
        let items = matching_parameter_type_names(&names, &normalized_query)
            .into_iter()
            .map(|name| CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::REFERENCE),
                filter_text: Some(query.clone()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end,
                    },
                    new_text: name.clone(),
                })),
                ..Default::default()
            })
            .collect();
        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(&self, _: usize, _: &str, _: &mut Context<InputState>) -> bool {
        true
    }
}

fn parameter_type_names(parameter_type_set: Option<&xtce::ParameterTypeSetType>) -> Vec<String> {
    parameter_type_set
        .into_iter()
        .flat_map(|set| &set.content)
        .map(parameter_type_name)
        .filter(|name| !name.is_empty())
        .collect()
}

fn matching_parameter_type_names<'a>(
    names: &'a [String],
    normalized_query: &str,
) -> Vec<&'a String> {
    names
        .iter()
        .filter(|name| name.to_ascii_lowercase().contains(normalized_query))
        .collect()
}

fn parameter_type_name(parameter_type: &xtce::ParameterTypeSetTypeContent) -> String {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => value.name.clone(),
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => value.name.clone(),
    }
}

struct ParameterValues {
    name: String,
    parameter_type_ref: String,
    initial_value: String,
    short_description: String,
    long_description: String,
    parameter_ref: String,
}

impl ParameterValues {
    fn from_parameter(parameter: Option<&xtce::ParameterSetTypeContent>) -> Self {
        match parameter {
            Some(xtce::ParameterSetTypeContent::Parameter(parameter)) => Self {
                name: parameter.name.clone(),
                parameter_type_ref: parameter.parameter_type_ref.clone(),
                initial_value: parameter.initial_value.clone().unwrap_or_default(),
                short_description: parameter.short_description.clone().unwrap_or_default(),
                long_description: parameter.long_description.clone().unwrap_or_default(),
                parameter_ref: String::new(),
            },
            Some(xtce::ParameterSetTypeContent::ParameterRef(parameter)) => Self {
                name: String::new(),
                parameter_type_ref: String::new(),
                initial_value: String::new(),
                short_description: String::new(),
                long_description: String::new(),
                parameter_ref: parameter.parameter_ref.clone(),
            },
            None => Self {
                name: String::new(),
                parameter_type_ref: String::new(),
                initial_value: String::new(),
                short_description: String::new(),
                long_description: String::new(),
                parameter_ref: String::new(),
            },
        }
    }

    fn apply_to(&self, parameter: &mut xtce::ParameterSetTypeContent) {
        match parameter {
            xtce::ParameterSetTypeContent::Parameter(parameter) => {
                parameter.name.clone_from(&self.name);
                parameter
                    .parameter_type_ref
                    .clone_from(&self.parameter_type_ref);
                parameter.initial_value = optional_value(self.initial_value.clone());
                parameter.short_description = optional_value(self.short_description.clone());
                parameter.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterSetTypeContent::ParameterRef(parameter) => {
                parameter.parameter_ref.clone_from(&self.parameter_ref);
            }
        }
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

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{ParameterValues, matching_parameter_type_names, parameter_type_names};

    #[test]
    fn applying_form_values_preserves_nested_parameter_metadata() {
        let mut parameter = xtce::ParameterSetTypeContent::Parameter(xtce::ParameterType {
            short_description: None,
            name: "OldName".to_owned(),
            parameter_type_ref: "OldType".to_owned(),
            initial_value: None,
            long_description: None,
            alias_set: Some(xtce::AliasSetType {
                alias: vec![xtce::AliasType {
                    name_space: "operations".to_owned(),
                    alias: "MODE".to_owned(),
                }],
            }),
            ancillary_data_set: None,
            parameter_properties: None,
        });
        ParameterValues {
            name: "Mode".to_owned(),
            parameter_type_ref: "ModeType".to_owned(),
            initial_value: "SAFE".to_owned(),
            short_description: "Current mode".to_owned(),
            long_description: "Current operational mode".to_owned(),
            parameter_ref: String::new(),
        }
        .apply_to(&mut parameter);

        match parameter {
            xtce::ParameterSetTypeContent::Parameter(parameter) => {
                assert_eq!(parameter.name, "Mode");
                assert_eq!(parameter.initial_value.as_deref(), Some("SAFE"));
                assert_eq!(
                    parameter.alias_set.expect("alias set").alias[0].alias,
                    "MODE"
                );
            }
            xtce::ParameterSetTypeContent::ParameterRef(_) => panic!("expected a Parameter"),
        }
    }

    #[test]
    fn parameter_type_suggestions_use_names_from_the_parameter_type_set() {
        let parameter_type_set = xtce::ParameterTypeSetType {
            content: vec![
                xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                    short_description: None,
                    name: "ModeType".to_owned(),
                    base_type: None,
                    initial_value: None,
                    restriction_pattern: None,
                    character_width: None,
                    content: Vec::new(),
                }),
                xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                    short_description: None,
                    name: "CounterType".to_owned(),
                    base_type: None,
                    initial_value: None,
                    restriction_pattern: None,
                    character_width: None,
                    content: Vec::new(),
                }),
            ],
        };

        let names = parameter_type_names(Some(&parameter_type_set));
        let matching = matching_parameter_type_names(&names, "mode");

        assert_eq!(names, ["ModeType", "CounterType"]);
        assert_eq!(matching, [&"ModeType".to_owned()]);
    }
}
