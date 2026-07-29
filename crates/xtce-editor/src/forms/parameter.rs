use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Task, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    IconName, IndexPath, Sizable,
    button::{Button, ButtonVariants},
    collapsible::Collapsible,
    h_flex,
    input::{CompletionProvider, InputEvent, InputState, Rope, RopeExt},
    select::{SelectEvent, SelectState},
    v_flex,
};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    Position, Range, TextEdit,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm, field, impl_select_item,
    message::MessageCriteriaForm, optional_value,
};
use crate::XtceEditor;

#[derive(Clone, Copy)]
enum ParameterElementKind {
    Parameter,
    Reference,
    Missing,
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum PresenceChoice {
    None,
    Present,
}
impl_select_item!(PresenceChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum TelemetryDataSourceChoice {
    None,
    #[strum(serialize = "telemetered")]
    Telemetered,
    #[strum(serialize = "derived")]
    Derived,
    #[strum(serialize = "constant")]
    Constant,
    #[strum(serialize = "local")]
    Local,
    #[strum(serialize = "ground")]
    Ground,
}
impl_select_item!(TelemetryDataSourceChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum TimeAssociationUnitChoice {
    #[strum(serialize = "seconds")]
    Seconds,
    #[strum(serialize = "milliseconds")]
    Milliseconds,
    #[strum(serialize = "microseconds")]
    Microseconds,
    #[strum(serialize = "nanoseconds")]
    Nanoseconds,
    #[strum(serialize = "picoseconds")]
    Picoseconds,
    #[strum(serialize = "minutes")]
    Minutes,
    #[strum(serialize = "hours")]
    Hours,
    #[strum(serialize = "days")]
    Days,
    #[strum(serialize = "months")]
    Months,
    #[strum(serialize = "years")]
    Years,
}
impl_select_item!(TimeAssociationUnitChoice);

struct ParameterPropertiesForm {
    presence: Entity<SelectState<Vec<PresenceChoice>>>,
    data_source: Entity<SelectState<Vec<TelemetryDataSourceChoice>>>,
    read_only: Entity<SelectState<Vec<BooleanChoice>>>,
    persistence: Entity<SelectState<Vec<BooleanChoice>>>,
    system_name: Entity<InputState>,
    validity_presence: Entity<SelectState<Vec<PresenceChoice>>>,
    validity_condition: Entity<MessageCriteriaForm>,
    physical_addresses: Entity<InputState>,
    time_presence: Entity<SelectState<Vec<PresenceChoice>>>,
    time_parameter_ref: Entity<InputState>,
    time_instance: Entity<InputState>,
    time_calibrated: Entity<SelectState<Vec<BooleanChoice>>>,
    time_interpolate: Entity<SelectState<Vec<BooleanChoice>>>,
    time_offset: Entity<InputState>,
    time_unit: Entity<SelectState<Vec<TimeAssociationUnitChoice>>>,
    _subscriptions: Vec<Subscription>,
}

impl ParameterPropertiesForm {
    fn new(
        properties: Option<&xtce::ParameterPropertiesType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = ParameterPropertiesValues::from_properties(properties);
        let validity_condition = MessageCriteriaForm::new(
            properties.and_then(|value| value.validity_condition.as_ref()),
            window,
            cx,
        );
        cx.new(|cx| {
            let presence = select(PresenceChoice::VARIANTS, values.presence, window, cx);
            let validity_presence = select(
                PresenceChoice::VARIANTS,
                values.validity_presence,
                window,
                cx,
            );
            let time_presence = select(PresenceChoice::VARIANTS, values.time_presence, window, cx);
            let subscriptions = vec![
                cx.subscribe(
                    &presence,
                    |_, _, _: &SelectEvent<Vec<PresenceChoice>>, cx| cx.notify(),
                ),
                cx.subscribe(
                    &validity_presence,
                    |_, _, _: &SelectEvent<Vec<PresenceChoice>>, cx| cx.notify(),
                ),
                cx.subscribe(
                    &time_presence,
                    |_, _, _: &SelectEvent<Vec<PresenceChoice>>, cx| cx.notify(),
                ),
            ];
            Self {
                presence,
                data_source: select(
                    TelemetryDataSourceChoice::VARIANTS,
                    values.data_source,
                    window,
                    cx,
                ),
                read_only: select(BooleanChoice::VARIANTS, values.read_only, window, cx),
                persistence: select(BooleanChoice::VARIANTS, values.persistence, window, cx),
                system_name: input(&values.system_name, false, window, cx),
                validity_presence,
                validity_condition,
                physical_addresses: input(&values.physical_addresses, true, window, cx),
                time_presence,
                time_parameter_ref: input(&values.time_parameter_ref, false, window, cx),
                time_instance: input(&values.time_instance, false, window, cx),
                time_calibrated: select(
                    BooleanChoice::VARIANTS,
                    values.time_calibrated,
                    window,
                    cx,
                ),
                time_interpolate: select(
                    BooleanChoice::VARIANTS,
                    values.time_interpolate,
                    window,
                    cx,
                ),
                time_offset: input(&values.time_offset, false, window, cx),
                time_unit: select(
                    TimeAssociationUnitChoice::VARIANTS,
                    values.time_unit,
                    window,
                    cx,
                ),
                _subscriptions: subscriptions,
            }
        })
    }

    fn load(
        &mut self,
        properties: Option<&xtce::ParameterPropertiesType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ParameterPropertiesValues::from_properties(properties);
        sync_select(&self.presence, values.presence, window, cx);
        sync_select(&self.data_source, values.data_source, window, cx);
        sync_select(&self.read_only, values.read_only, window, cx);
        sync_select(&self.persistence, values.persistence, window, cx);
        sync_select(
            &self.validity_presence,
            values.validity_presence,
            window,
            cx,
        );
        sync_select(&self.time_presence, values.time_presence, window, cx);
        sync_select(&self.time_calibrated, values.time_calibrated, window, cx);
        sync_select(&self.time_interpolate, values.time_interpolate, window, cx);
        sync_select(&self.time_unit, values.time_unit, window, cx);
        self.validity_condition.update(cx, |form, cx| {
            form.load(
                properties.and_then(|value| value.validity_condition.as_ref()),
                window,
                cx,
            );
        });
        for (input, value) in [
            (&self.system_name, values.system_name),
            (&self.physical_addresses, values.physical_addresses),
            (&self.time_parameter_ref, values.time_parameter_ref),
            (&self.time_instance, values.time_instance),
            (&self.time_offset, values.time_offset),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    fn apply_to(&self, properties: &mut Option<xtce::ParameterPropertiesType>, cx: &App) {
        if selected_value(&self.presence, PresenceChoice::None, cx) == PresenceChoice::None {
            *properties = None;
            return;
        }
        let validity_condition =
            (selected_value(&self.validity_presence, PresenceChoice::None, cx)
                == PresenceChoice::Present)
                .then(|| {
                    let mut criteria = default_match_criteria();
                    self.validity_condition.read(cx).apply_to(&mut criteria, cx);
                    criteria
                });
        let physical_address = decode_physical_addresses(&value(&self.physical_addresses, cx));
        let physical_address_set = (!physical_address.is_empty())
            .then_some(xtce::PhysicalAddressSetType { physical_address });
        let time_association = (selected_value(&self.time_presence, PresenceChoice::None, cx)
            == PresenceChoice::Present)
            .then(|| xtce::TimeAssociationType {
                parameter_ref: value(&self.time_parameter_ref, cx),
                instance: value(&self.time_instance, cx)
                    .trim()
                    .parse()
                    .unwrap_or_else(|_| xtce::TimeAssociationType::default_instance()),
                use_calibrated_value: selected_value(
                    &self.time_calibrated,
                    BooleanChoice::True,
                    cx,
                ) == BooleanChoice::True,
                interpolate_time: selected_value(&self.time_interpolate, BooleanChoice::True, cx)
                    == BooleanChoice::True,
                offset: value(&self.time_offset, cx).trim().parse().ok(),
                unit: time_unit_to_xtce(selected_value(
                    &self.time_unit,
                    TimeAssociationUnitChoice::Seconds,
                    cx,
                )),
            });
        *properties = Some(xtce::ParameterPropertiesType {
            data_source: data_source_to_xtce(selected_value(
                &self.data_source,
                TelemetryDataSourceChoice::None,
                cx,
            )),
            read_only: selected_value(&self.read_only, BooleanChoice::False, cx)
                == BooleanChoice::True,
            persistence: selected_value(&self.persistence, BooleanChoice::True, cx)
                == BooleanChoice::True,
            system_name: optional_value(value(&self.system_name, cx)),
            validity_condition,
            physical_address_set,
            time_association,
        });
    }
}

impl Render for ParameterPropertiesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let present =
            selected_value(&self.presence, PresenceChoice::None, cx) == PresenceChoice::Present;
        let validity_present = selected_value(&self.validity_presence, PresenceChoice::None, cx)
            == PresenceChoice::Present;
        let time_present = selected_value(&self.time_presence, PresenceChoice::None, cx)
            == PresenceChoice::Present;
        v_flex()
            .w_full()
            .gap_4()
            .child(select_field(
                "Parameter properties",
                "Optional",
                &self.presence,
                cx,
            ))
            .when(present, |form| {
                form.child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .child(div().w(px(220.)).child(select_field(
                            "Data source",
                            "Optional",
                            &self.data_source,
                            cx,
                        )))
                        .child(div().w(px(160.)).child(select_field(
                            "Read only",
                            "Required",
                            &self.read_only,
                            cx,
                        )))
                        .child(div().w(px(160.)).child(select_field(
                            "Persistence",
                            "Required",
                            &self.persistence,
                            cx,
                        ))),
                )
                .child(field(
                    "System name",
                    "Optional",
                    &self.system_name,
                    cx,
                ))
                .child(select_field(
                    "Validity condition",
                    "Optional",
                    &self.validity_presence,
                    cx,
                ))
                .when(validity_present, |form| {
                    form.child(self.validity_condition.clone())
                })
                .child(field(
                    "Physical addresses",
                    "One address per line: source name | source address; append sub-addresses with “ > ”",
                    &self.physical_addresses,
                    cx,
                ))
                .child(select_field(
                    "Time association",
                    "Optional",
                    &self.time_presence,
                    cx,
                ))
                .when(time_present, |form| {
                    form.child(field(
                        "Time parameter reference",
                        "Required",
                        &self.time_parameter_ref,
                        cx,
                    ))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(div().w(px(140.)).child(field(
                                "Instance",
                                "Required",
                                &self.time_instance,
                                cx,
                            )))
                            .child(div().w(px(180.)).child(select_field(
                                "Calibrated value",
                                "Required",
                                &self.time_calibrated,
                                cx,
                            )))
                            .child(div().w(px(180.)).child(select_field(
                                "Interpolate time",
                                "Required",
                                &self.time_interpolate,
                                cx,
                            ))),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(div().w(px(220.)).child(field(
                                "Offset",
                                "Optional",
                                &self.time_offset,
                                cx,
                            )))
                            .child(div().w(px(220.)).child(select_field(
                                "Offset unit",
                                "Required",
                                &self.time_unit,
                                cx,
                            ))),
                    )
                })
            })
    }
}

pub(super) struct ParameterForm {
    element_kind: ParameterElementKind,
    name_input: Entity<InputState>,
    parameter_type_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    parameter_properties: Entity<ParameterPropertiesForm>,
    parameter_ref_input: Entity<InputState>,
    parameter_type_names: Rc<RefCell<Vec<String>>>,
    defaults_open: bool,
    documentation_open: bool,
    metadata_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl ParameterForm {
    pub(super) fn render_name_editor(&self, cx: &App) -> Div {
        super::name_editor(&self.name_input, cx)
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
        let alias_set = AliasSetForm::new(
            parameter.and_then(|parameter| match parameter {
                xtce::ParameterSetTypeContent::Parameter(parameter) => parameter.alias_set.as_ref(),
                xtce::ParameterSetTypeContent::ParameterRef(_) => None,
            }),
            window,
            cx,
        );
        let ancillary_data_set = AncillaryDataSetForm::new(
            parameter.and_then(|parameter| match parameter {
                xtce::ParameterSetTypeContent::Parameter(parameter) => {
                    parameter.ancillary_data_set.as_ref()
                }
                xtce::ParameterSetTypeContent::ParameterRef(_) => None,
            }),
            window,
            cx,
        );
        let parameter_properties = ParameterPropertiesForm::new(
            parameter.and_then(|parameter| match parameter {
                xtce::ParameterSetTypeContent::Parameter(parameter) => {
                    parameter.parameter_properties.as_ref()
                }
                xtce::ParameterSetTypeContent::ParameterRef(_) => None,
            }),
            window,
            cx,
        );
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
                alias_set,
                ancillary_data_set,
                parameter_properties,
                parameter_ref_input: input(&values.parameter_ref, false, window, cx),
                parameter_type_names,
                defaults_open: false,
                documentation_open: false,
                metadata_open: false,
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
        self.defaults_open = false;
        self.documentation_open = false;
        self.metadata_open = false;
        self.element_kind = match parameter {
            Some(xtce::ParameterSetTypeContent::Parameter(_)) => ParameterElementKind::Parameter,
            Some(xtce::ParameterSetTypeContent::ParameterRef(_)) => ParameterElementKind::Reference,
            None => ParameterElementKind::Missing,
        };
        let values = ParameterValues::from_parameter(parameter);
        self.alias_set.load(
            parameter.and_then(|parameter| match parameter {
                xtce::ParameterSetTypeContent::Parameter(parameter) => parameter.alias_set.as_ref(),
                xtce::ParameterSetTypeContent::ParameterRef(_) => None,
            }),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            parameter.and_then(|parameter| match parameter {
                xtce::ParameterSetTypeContent::Parameter(parameter) => {
                    parameter.ancillary_data_set.as_ref()
                }
                xtce::ParameterSetTypeContent::ParameterRef(_) => None,
            }),
            window,
            cx,
        );
        self.parameter_properties.update(cx, |form, cx| {
            form.load(
                parameter.and_then(|parameter| match parameter {
                    xtce::ParameterSetTypeContent::Parameter(parameter) => {
                        parameter.parameter_properties.as_ref()
                    }
                    xtce::ParameterSetTypeContent::ParameterRef(_) => None,
                }),
                window,
                cx,
            );
        });
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
        if let xtce::ParameterSetTypeContent::Parameter(parameter) = parameter {
            self.alias_set.apply_to_option(&mut parameter.alias_set, cx);
            self.ancillary_data_set
                .apply_to_option(&mut parameter.ancillary_data_set, cx);
            self.parameter_properties
                .read(cx)
                .apply_to(&mut parameter.parameter_properties, cx);
        }
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        match self.element_kind {
            ParameterElementKind::Parameter => v_flex()
                .gap_5()
                .child(field(
                    "Parameter type reference",
                    "Required",
                    &self.parameter_type_ref_input,
                    cx,
                ))
                .child(
                    Collapsible::new()
                        .open(self.defaults_open)
                        .child(
                            Button::new("toggle-parameter-defaults")
                                .small()
                                .link()
                                .icon(if self.defaults_open {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronRight
                                })
                                .label("Defaults")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.defaults_open = !this.defaults_open;
                                    cx.notify();
                                })),
                        )
                        .content(v_flex().pt_3().child(field(
                            "Initial value",
                            "Optional",
                            &self.initial_value_input,
                            cx,
                        ))),
                )
                .child(
                    Collapsible::new()
                        .open(self.documentation_open)
                        .child(
                            Button::new("toggle-parameter-documentation")
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
                        ),
                )
                .child(
                    Collapsible::new()
                        .open(self.metadata_open)
                        .child(
                            Button::new("toggle-parameter-metadata")
                                .small()
                                .link()
                                .icon(if self.metadata_open {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronRight
                                })
                                .label("Metadata")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.metadata_open = !this.metadata_open;
                                    cx.notify();
                                })),
                        )
                        .content(
                            v_flex()
                                .pt_3()
                                .gap_4()
                                .child(self.alias_set.render(cx))
                                .child(self.ancillary_data_set.render(cx))
                                .child(self.parameter_properties.clone()),
                        ),
                ),
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

struct ParameterPropertiesValues {
    presence: PresenceChoice,
    data_source: TelemetryDataSourceChoice,
    read_only: BooleanChoice,
    persistence: BooleanChoice,
    system_name: String,
    validity_presence: PresenceChoice,
    physical_addresses: String,
    time_presence: PresenceChoice,
    time_parameter_ref: String,
    time_instance: String,
    time_calibrated: BooleanChoice,
    time_interpolate: BooleanChoice,
    time_offset: String,
    time_unit: TimeAssociationUnitChoice,
}

impl ParameterPropertiesValues {
    fn from_properties(properties: Option<&xtce::ParameterPropertiesType>) -> Self {
        let time = properties.and_then(|value| value.time_association.as_ref());
        Self {
            presence: if properties.is_some() {
                PresenceChoice::Present
            } else {
                PresenceChoice::None
            },
            data_source: properties
                .and_then(|value| value.data_source.as_ref())
                .map(data_source_from_xtce)
                .unwrap_or(TelemetryDataSourceChoice::None),
            read_only: if properties.is_some_and(|value| value.read_only) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            persistence: if properties.is_none_or(|value| value.persistence) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            system_name: properties
                .and_then(|value| value.system_name.clone())
                .unwrap_or_default(),
            validity_presence: if properties.is_some_and(|value| value.validity_condition.is_some())
            {
                PresenceChoice::Present
            } else {
                PresenceChoice::None
            },
            physical_addresses: encode_physical_addresses(
                properties.and_then(|value| value.physical_address_set.as_ref()),
            ),
            time_presence: if time.is_some() {
                PresenceChoice::Present
            } else {
                PresenceChoice::None
            },
            time_parameter_ref: time
                .map(|value| value.parameter_ref.clone())
                .unwrap_or_default(),
            time_instance: time
                .map(|value| value.instance.to_string())
                .unwrap_or_else(|| xtce::TimeAssociationType::default_instance().to_string()),
            time_calibrated: if time.is_none_or(|value| value.use_calibrated_value) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            time_interpolate: if time.is_none_or(|value| value.interpolate_time) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            time_offset: time
                .and_then(|value| value.offset)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            time_unit: time
                .map(|value| time_unit_from_xtce(&value.unit))
                .unwrap_or(TimeAssociationUnitChoice::Seconds),
        }
    }
}

fn data_source_from_xtce(value: &xtce::TelemetryDataSourceType) -> TelemetryDataSourceChoice {
    match value {
        xtce::TelemetryDataSourceType::Telemetered => TelemetryDataSourceChoice::Telemetered,
        xtce::TelemetryDataSourceType::Derived => TelemetryDataSourceChoice::Derived,
        xtce::TelemetryDataSourceType::Constant => TelemetryDataSourceChoice::Constant,
        xtce::TelemetryDataSourceType::Local => TelemetryDataSourceChoice::Local,
        xtce::TelemetryDataSourceType::Ground => TelemetryDataSourceChoice::Ground,
    }
}

fn data_source_to_xtce(value: TelemetryDataSourceChoice) -> Option<xtce::TelemetryDataSourceType> {
    match value {
        TelemetryDataSourceChoice::None => None,
        TelemetryDataSourceChoice::Telemetered => Some(xtce::TelemetryDataSourceType::Telemetered),
        TelemetryDataSourceChoice::Derived => Some(xtce::TelemetryDataSourceType::Derived),
        TelemetryDataSourceChoice::Constant => Some(xtce::TelemetryDataSourceType::Constant),
        TelemetryDataSourceChoice::Local => Some(xtce::TelemetryDataSourceType::Local),
        TelemetryDataSourceChoice::Ground => Some(xtce::TelemetryDataSourceType::Ground),
    }
}

fn time_unit_from_xtce(value: &xtce::TimeAssociationUnitType) -> TimeAssociationUnitChoice {
    match value {
        xtce::TimeAssociationUnitType::Seconds => TimeAssociationUnitChoice::Seconds,
        xtce::TimeAssociationUnitType::Milliseconds => TimeAssociationUnitChoice::Milliseconds,
        xtce::TimeAssociationUnitType::Microseconds => TimeAssociationUnitChoice::Microseconds,
        xtce::TimeAssociationUnitType::Nanoseconds => TimeAssociationUnitChoice::Nanoseconds,
        xtce::TimeAssociationUnitType::Picoseconds => TimeAssociationUnitChoice::Picoseconds,
        xtce::TimeAssociationUnitType::Minutes => TimeAssociationUnitChoice::Minutes,
        xtce::TimeAssociationUnitType::Hours => TimeAssociationUnitChoice::Hours,
        xtce::TimeAssociationUnitType::Days => TimeAssociationUnitChoice::Days,
        xtce::TimeAssociationUnitType::Months => TimeAssociationUnitChoice::Months,
        xtce::TimeAssociationUnitType::Years => TimeAssociationUnitChoice::Years,
    }
}

fn time_unit_to_xtce(value: TimeAssociationUnitChoice) -> xtce::TimeAssociationUnitType {
    match value {
        TimeAssociationUnitChoice::Seconds => xtce::TimeAssociationUnitType::Seconds,
        TimeAssociationUnitChoice::Milliseconds => xtce::TimeAssociationUnitType::Milliseconds,
        TimeAssociationUnitChoice::Microseconds => xtce::TimeAssociationUnitType::Microseconds,
        TimeAssociationUnitChoice::Nanoseconds => xtce::TimeAssociationUnitType::Nanoseconds,
        TimeAssociationUnitChoice::Picoseconds => xtce::TimeAssociationUnitType::Picoseconds,
        TimeAssociationUnitChoice::Minutes => xtce::TimeAssociationUnitType::Minutes,
        TimeAssociationUnitChoice::Hours => xtce::TimeAssociationUnitType::Hours,
        TimeAssociationUnitChoice::Days => xtce::TimeAssociationUnitType::Days,
        TimeAssociationUnitChoice::Months => xtce::TimeAssociationUnitType::Months,
        TimeAssociationUnitChoice::Years => xtce::TimeAssociationUnitType::Years,
    }
}

fn encode_physical_addresses(set: Option<&xtce::PhysicalAddressSetType>) -> String {
    set.into_iter()
        .flat_map(|set| &set.physical_address)
        .map(encode_physical_address)
        .collect::<Vec<_>>()
        .join("\n")
}

fn encode_physical_address(address: &xtce::PhysicalAddressType) -> String {
    let mut segments = Vec::new();
    let mut current = Some(address);
    while let Some(address) = current {
        segments.push(format!(
            "{} | {}",
            address.source_name.as_deref().unwrap_or_default(),
            address.source_address.as_deref().unwrap_or_default()
        ));
        current = address.sub_address.as_deref();
    }
    segments.join(" > ")
}

fn decode_physical_addresses(value: &str) -> Vec<xtce::PhysicalAddressType> {
    value
        .lines()
        .filter_map(|line| {
            line.split(" > ")
                .filter_map(|segment| {
                    let (source_name, source_address) =
                        segment.split_once(" | ").unwrap_or((segment, ""));
                    let source_name = optional_value(source_name.trim().to_owned());
                    let source_address = optional_value(source_address.trim().to_owned());
                    (source_name.is_some() || source_address.is_some())
                        .then_some((source_name, source_address))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .fold(None, |sub_address, (source_name, source_address)| {
                    Some(xtce::PhysicalAddressType {
                        source_name,
                        source_address,
                        sub_address: sub_address.map(Box::new),
                    })
                })
        })
        .collect()
}

fn default_match_criteria() -> xtce::MatchCriteriaType {
    xtce::MatchCriteriaType::Comparison(xtce::ComparisonType {
        parameter_ref: String::new(),
        instance: xtce::ComparisonType::default_instance(),
        use_calibrated_value: xtce::ComparisonType::default_use_calibrated_value(),
        comparison_operator: xtce::ComparisonType::default_comparison_operator(),
        value: String::new(),
    })
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

fn sync_select<T>(
    select: &Entity<SelectState<Vec<T>>>,
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select.update(cx, |select, cx| {
        select.set_selected_value(&selected, window, cx);
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

fn input(
    value: &str,
    multi_line: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let input = InputState::new(window, cx).default_value(value.to_owned());
        if multi_line {
            input.auto_grow(super::MULTILINE_MIN_ROWS, super::MULTILINE_MAX_ROWS)
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
    use super::{
        BooleanChoice, ParameterPropertiesValues, ParameterValues, PresenceChoice,
        TelemetryDataSourceChoice, TimeAssociationUnitChoice, decode_physical_addresses,
        matching_parameter_type_names, parameter_type_names,
    };

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

    #[test]
    fn all_parameter_properties_are_loaded() {
        let properties = xtce::ParameterPropertiesType {
            data_source: Some(xtce::TelemetryDataSourceType::Ground),
            read_only: true,
            persistence: false,
            system_name: Some("GROUND_MODE".to_owned()),
            validity_condition: Some(xtce::MatchCriteriaType::Comparison(xtce::ComparisonType {
                parameter_ref: "VALID".to_owned(),
                instance: 0,
                use_calibrated_value: true,
                comparison_operator: "==".to_owned(),
                value: "1".to_owned(),
            })),
            physical_address_set: Some(xtce::PhysicalAddressSetType {
                physical_address: vec![xtce::PhysicalAddressType {
                    source_name: Some("RAM".to_owned()),
                    source_address: Some("0x1000".to_owned()),
                    sub_address: Some(Box::new(xtce::PhysicalAddressType {
                        source_name: Some("bank".to_owned()),
                        source_address: Some("3".to_owned()),
                        sub_address: None,
                    })),
                }],
            }),
            time_association: Some(xtce::TimeAssociationType {
                parameter_ref: "MISSION_TIME".to_owned(),
                instance: -1,
                use_calibrated_value: false,
                interpolate_time: false,
                offset: Some(2.5),
                unit: xtce::TimeAssociationUnitType::Milliseconds,
            }),
        };

        let values = ParameterPropertiesValues::from_properties(Some(&properties));
        assert_eq!(values.presence, PresenceChoice::Present);
        assert_eq!(values.data_source, TelemetryDataSourceChoice::Ground);
        assert_eq!(values.read_only, BooleanChoice::True);
        assert_eq!(values.persistence, BooleanChoice::False);
        assert_eq!(values.system_name, "GROUND_MODE");
        assert_eq!(values.validity_presence, PresenceChoice::Present);
        assert_eq!(values.physical_addresses, "RAM | 0x1000 > bank | 3");
        assert_eq!(values.time_presence, PresenceChoice::Present);
        assert_eq!(values.time_parameter_ref, "MISSION_TIME");
        assert_eq!(values.time_instance, "-1");
        assert_eq!(values.time_calibrated, BooleanChoice::False);
        assert_eq!(values.time_interpolate, BooleanChoice::False);
        assert_eq!(values.time_offset, "2.5");
        assert_eq!(values.time_unit, TimeAssociationUnitChoice::Milliseconds);

        let decoded = decode_physical_addresses(&values.physical_addresses);
        assert_eq!(decoded[0].source_name.as_deref(), Some("RAM"));
        assert_eq!(
            decoded[0]
                .sub_address
                .as_deref()
                .and_then(|value| value.source_address.as_deref()),
            Some("3")
        );
    }
}
