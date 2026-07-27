use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use anyhow::Result;
use gpui::{
    AnyElement, App, AppContext, Context, Div, Entity, InteractiveElement, IntoElement,
    ListAlignment, ListState, ParentElement, Render, StatefulInteractiveElement, Styled,
    Subscription, Task, Window, div, list, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, IndexPath, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    collapsible::Collapsible,
    h_flex,
    input::{CompletionProvider, Input, InputEvent, InputState, Rope, RopeExt},
    select::{Select, SelectEvent, SelectState},
    tooltip::Tooltip,
    v_flex,
};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    Position, Range, TextEdit,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm,
    boolean_expression::BooleanExpressionForm,
    container_binary_encoding::ContainerBinaryEncodingForm, container_rate::ContainerRateForm,
    field, impl_select_item, input_algorithm::InputAlgorithmForm, optional_value,
};
use crate::{DraggedTelemetryParameter, XtceEditor};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum AbstractChoice {
    #[strum(serialize = "false")]
    Concrete,
    #[strum(serialize = "true")]
    Abstract,
}
impl_select_item!(AbstractChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ComparisonOperator {
    #[strum(serialize = "==")]
    Equal,
    #[strum(serialize = "!=")]
    NotEqual,
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = "<=")]
    LessOrEqual,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = ">=")]
    GreaterOrEqual,
}
impl_select_item!(ComparisonOperator);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CalibratedChoice {
    #[strum(serialize = "Calibrated value")]
    CalibratedValue,
    #[strum(serialize = "Raw value")]
    RawValue,
}
impl_select_item!(CalibratedChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum RestrictionCriteriaKind {
    #[strum(serialize = "Comparison list")]
    ComparisonList,
    #[strum(serialize = "Boolean expression")]
    BooleanExpression,
    #[strum(serialize = "Custom algorithm")]
    CustomAlgorithm,
    #[strum(serialize = "Next container")]
    NextContainer,
}
impl_select_item!(RestrictionCriteriaKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum EntryKind {
    #[strum(serialize = "ParameterRefEntry")]
    Parameter,
    #[strum(serialize = "ParameterSegmentRefEntry")]
    ParameterSegment,
    #[strum(serialize = "ContainerRefEntry")]
    Container,
    #[strum(serialize = "ContainerSegmentRefEntry")]
    ContainerSegment,
    #[strum(serialize = "StreamSegmentEntry")]
    StreamSegment,
}
impl_select_item!(EntryKind);

fn entry_reference_placeholder(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Parameter | EntryKind::ParameterSegment => "Select a parameter...",
        EntryKind::Container | EntryKind::ContainerSegment => "Select a container...",
        EntryKind::StreamSegment => "Select a stream...",
    }
}

pub(super) struct SequenceContainerForm {
    name_input: Entity<InputState>,
    abstract_select: Entity<SelectState<Vec<AbstractChoice>>>,
    idle_pattern_input: Entity<InputState>,
    advanced_settings_open: bool,
    documentation_open: bool,
    metadata_open: bool,
    stream_rate_open: bool,
    restriction_open: bool,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    rate: Entity<ContainerRateForm>,
    binary_encoding: Entity<ContainerBinaryEncodingForm>,
    base_container_present: Rc<Cell<bool>>,
    base_container_ref_input: Entity<InputState>,
    restriction_kind_select: Entity<SelectState<Vec<RestrictionCriteriaKind>>>,
    comparisons: Entity<ComparisonListForm>,
    restriction_boolean_expression: Entity<BooleanExpressionForm>,
    restriction_custom_algorithm: Entity<InputAlgorithmForm>,
    restriction_next_container_input: Entity<InputState>,
    reference_context: Rc<RefCell<ReferenceContext>>,
    entry_list: Entity<TelemetryEntryListView>,
    _subscriptions: Vec<Subscription>,
}

pub(super) struct ReferenceSets<'a, C> {
    pub(super) parameter_set: Option<&'a xtce::ParameterSetType>,
    pub(super) parameter_type_set: Option<&'a xtce::ParameterTypeSetType>,
    pub(super) container_set: Option<&'a C>,
    pub(super) stream_set: Option<&'a xtce::StreamSetType>,
}

impl SequenceContainerForm {
    pub(super) fn new(
        container: Option<&xtce::ContainerSetTypeContent>,
        references: ReferenceSets<'_, xtce::ContainerSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let reference_context = Rc::new(RefCell::new(ReferenceContext::new(
            references.parameter_set,
            references.parameter_type_set,
            references.container_set,
            references.stream_set,
        )));
        Self::new_with_context(sequence_container(container), reference_context, window, cx)
    }

    fn new_with_context(
        sequence: Option<&xtce::SequenceContainerType>,
        reference_context: Rc<RefCell<ReferenceContext>>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = ContainerValues::from_sequence(sequence);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        let alias_set = AliasSetForm::new(
            sequence.and_then(|value| value.alias_set.as_ref()),
            window,
            cx,
        );
        let ancillary_data_set = AncillaryDataSetForm::new(
            sequence.and_then(|value| value.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        let rate = ContainerRateForm::new(
            sequence.and_then(|value| value.default_rate_in_stream.as_ref()),
            sequence.and_then(|value| value.rate_in_stream_set.as_ref()),
            window,
            cx,
        );
        let binary_encoding = ContainerBinaryEncodingForm::new(
            sequence.and_then(|value| value.binary_encoding.as_ref()),
            window,
            cx,
        );
        let restriction_boolean_expression =
            BooleanExpressionForm::new(values.restriction_boolean_expression, window, cx);
        let restriction_custom_algorithm =
            InputAlgorithmForm::new(values.restriction_custom_algorithm, window, cx);

        cx.new(move |cx| {
            let base_container_ref_input = completion_input(
                &values.base_container_ref,
                CompletionTarget::Container,
                reference_context.clone(),
                window,
                cx,
            );
            let restriction_next_container_input = completion_input(
                &values.restriction_next_container,
                CompletionTarget::Container,
                reference_context.clone(),
                window,
                cx,
            );
            let entry_list = cx.new(|cx| {
                TelemetryEntryListView::new(
                    rows_from_entry_list(values.entry_list),
                    reference_context.clone(),
                    base_container_ref_input.clone(),
                    name_input.clone(),
                    window,
                    cx,
                )
            });
            let comparisons = ComparisonListForm::new(
                &values.restriction_criteria,
                reference_context.clone(),
                window,
                cx,
            );
            let restriction_kind_select = select(
                RestrictionCriteriaKind::VARIANTS,
                values.restriction_kind,
                window,
                cx,
            );
            let restriction_kind_subscription = cx.subscribe(
                &restriction_kind_select,
                |_, _, _: &SelectEvent<Vec<RestrictionCriteriaKind>>, cx| cx.notify(),
            );
            Self {
                name_input,
                abstract_select: select(
                    AbstractChoice::VARIANTS,
                    if values.abstract_ {
                        AbstractChoice::Abstract
                    } else {
                        AbstractChoice::Concrete
                    },
                    window,
                    cx,
                ),
                idle_pattern_input: input(&values.idle_pattern, false, window, cx),
                advanced_settings_open: false,
                documentation_open: false,
                metadata_open: false,
                stream_rate_open: false,
                restriction_open: false,
                short_description_input: input(&values.short_description, false, window, cx),
                long_description_input: input(&values.long_description, true, window, cx),
                alias_set,
                ancillary_data_set,
                rate,
                binary_encoding,
                base_container_present: Rc::new(Cell::new(values.base_container_present)),
                base_container_ref_input,
                restriction_kind_select,
                comparisons,
                restriction_boolean_expression,
                restriction_custom_algorithm,
                restriction_next_container_input,
                reference_context,
                entry_list,
                _subscriptions: vec![name_subscription, restriction_kind_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        container: Option<&xtce::ContainerSetTypeContent>,
        references: ReferenceSets<'_, xtce::ContainerSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_with_context(
            sequence_container(container),
            ReferenceContext::new(
                references.parameter_set,
                references.parameter_type_set,
                references.container_set,
                references.stream_set,
            ),
            window,
            cx,
        );
    }

    pub(super) fn load_direct(
        &mut self,
        container: Option<&xtce::SequenceContainerType>,
        references: ReferenceSets<'_, xtce::CommandContainerSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_with_context(
            container,
            ReferenceContext::new_for_command(
                references.parameter_set,
                references.parameter_type_set,
                references.container_set,
                references.stream_set,
            ),
            window,
            cx,
        );
    }

    fn load_with_context(
        &mut self,
        sequence: Option<&xtce::SequenceContainerType>,
        reference_context: ReferenceContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ContainerValues::from_sequence(sequence);
        self.advanced_settings_open = false;
        self.documentation_open = false;
        self.metadata_open = false;
        self.stream_rate_open = false;
        self.restriction_open = false;
        self.alias_set.load(
            sequence.and_then(|value| value.alias_set.as_ref()),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            sequence.and_then(|value| value.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        self.rate.update(cx, |form, cx| {
            form.load(
                sequence.and_then(|value| value.default_rate_in_stream.as_ref()),
                sequence.and_then(|value| value.rate_in_stream_set.as_ref()),
                window,
                cx,
            );
        });
        self.binary_encoding.update(cx, |form, cx| {
            form.load(
                sequence.and_then(|value| value.binary_encoding.as_ref()),
                window,
                cx,
            );
        });
        *self.reference_context.borrow_mut() = reference_context;
        self.base_container_present
            .set(values.base_container_present);
        sync_select(
            &self.restriction_kind_select,
            values.restriction_kind,
            window,
            cx,
        );
        sync_select(
            &self.abstract_select,
            if values.abstract_ {
                AbstractChoice::Abstract
            } else {
                AbstractChoice::Concrete
            },
            window,
            cx,
        );
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.idle_pattern_input, values.idle_pattern),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.base_container_ref_input, values.base_container_ref),
            (
                &self.restriction_next_container_input,
                values.restriction_next_container,
            ),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.entry_list.update(cx, |list, cx| {
            list.set_rows(rows_from_entry_list(values.entry_list), cx);
        });
        self.comparisons.update(cx, |form, cx| {
            form.load(&values.restriction_criteria, window, cx);
        });
        self.restriction_boolean_expression.update(cx, |form, cx| {
            form.load(values.restriction_boolean_expression, window, cx);
        });
        self.restriction_custom_algorithm.update(cx, |form, cx| {
            form.load(values.restriction_custom_algorithm, window, cx);
        });
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn apply_to(&self, container: &mut xtce::ContainerSetTypeContent, cx: &App) {
        let xtce::ContainerSetTypeContent::SequenceContainer(container) = container;
        self.apply_to_sequence(container, cx);
    }

    pub(super) fn apply_to_sequence(&self, container: &mut xtce::SequenceContainerType, cx: &App) {
        container.name = value(&self.name_input, cx);
        container.abstract_ = selected_value(&self.abstract_select, AbstractChoice::Concrete, cx)
            == AbstractChoice::Abstract;
        container.idle_pattern = fixed_integer_value(&value(&self.idle_pattern_input, cx));
        container.short_description = optional_value(value(&self.short_description_input, cx));
        container.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut container.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut container.ancillary_data_set, cx);
        self.rate.read(cx).apply_to(
            &mut container.default_rate_in_stream,
            &mut container.rate_in_stream_set,
            cx,
        );
        self.binary_encoding
            .read(cx)
            .apply_to(&mut container.binary_encoding, cx);

        if self.base_container_present.get() {
            let base = container
                .base_container
                .get_or_insert_with(|| xtce::BaseContainerType {
                    container_ref: String::new(),
                    restriction_criteria: None,
                });
            base.container_ref = value(&self.base_container_ref_input, cx);
            base.restriction_criteria = match selected_value(
                &self.restriction_kind_select,
                RestrictionCriteriaKind::ComparisonList,
                cx,
            ) {
                RestrictionCriteriaKind::ComparisonList => {
                    self.comparisons.read(cx).to_criteria(cx)
                }
                RestrictionCriteriaKind::BooleanExpression => Some(xtce::RestrictionCriteriaType {
                    content: Some(xtce::RestrictionCriteriaTypeContent::BooleanExpression(
                        self.restriction_boolean_expression.read(cx).expression(cx),
                    )),
                }),
                RestrictionCriteriaKind::CustomAlgorithm => Some(xtce::RestrictionCriteriaType {
                    content: Some(xtce::RestrictionCriteriaTypeContent::CustomAlgorithm(
                        self.restriction_custom_algorithm.read(cx).algorithm(cx),
                    )),
                }),
                RestrictionCriteriaKind::NextContainer => Some(xtce::RestrictionCriteriaType {
                    content: Some(xtce::RestrictionCriteriaTypeContent::NextContainer(
                        xtce::ContainerRefType {
                            container_ref: value(&self.restriction_next_container_input, cx),
                        },
                    )),
                }),
            };
        } else {
            container.base_container = None;
        }
        let rows = self.entry_list.read(cx).current_rows(cx);
        apply_entry_rows(&mut container.entry_list, rows);
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        let base_present = self.base_container_present.get();
        v_flex()
            .w_full()
            .gap_5()
            .child(h_flex().gap_4().items_start().child(select_field(
                "Abstract",
                "Required",
                &self.abstract_select,
            )))
            .child(field(
                "Short description",
                "Optional",
                &self.short_description_input,
                cx,
            ))
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(div().text_sm().font_medium().child("Base container"))
                            .child(if base_present {
                                Button::new("remove-telemetry-base-container")
                                    .small()
                                    .danger()
                                    .label("Remove base container")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.base_container_present.set(false);
                                        cx.notify();
                                    }))
                            } else {
                                Button::new("add-telemetry-base-container")
                                    .small()
                                    .icon(IconName::Plus)
                                    .label("Add base container")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.base_container_present.set(true);
                                        cx.notify();
                                    }))
                            }),
                    )
                    .when(base_present, |section| {
                        section
                            .child(field(
                                "Base container reference",
                                "Required",
                                &self.base_container_ref_input,
                                cx,
                            ))
                            .child(
                                Collapsible::new()
                                    .open(self.restriction_open)
                                    .child(
                                        Button::new(
                                            "toggle-sequence-container-restriction-criteria",
                                        )
                                        .small()
                                        .link()
                                        .icon(if self.restriction_open {
                                            IconName::ChevronDown
                                        } else {
                                            IconName::ChevronRight
                                        })
                                        .label("Restriction criteria")
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.restriction_open = !this.restriction_open;
                                                cx.notify();
                                            }),
                                        ),
                                    )
                                    .content({
                                        let kind = selected_value(
                                            &self.restriction_kind_select,
                                            RestrictionCriteriaKind::ComparisonList,
                                            cx,
                                        );
                                        v_flex()
                                            .pt_3()
                                            .gap_3()
                                            .child(select_field(
                                                "Criteria type",
                                                "Required",
                                                &self.restriction_kind_select,
                                            ))
                                            .when(
                                                kind == RestrictionCriteriaKind::ComparisonList,
                                                |form| form.child(self.comparisons.clone()),
                                            )
                                            .when(
                                                kind == RestrictionCriteriaKind::BooleanExpression,
                                                |form| {
                                                    form.child(
                                                        self.restriction_boolean_expression.clone(),
                                                    )
                                                },
                                            )
                                            .when(
                                                kind == RestrictionCriteriaKind::CustomAlgorithm,
                                                |form| {
                                                    form.child(
                                                        self.restriction_custom_algorithm.clone(),
                                                    )
                                                },
                                            )
                                            .when(
                                                kind == RestrictionCriteriaKind::NextContainer,
                                                |form| {
                                                    form.child(field(
                                                        "Next container reference",
                                                        "Required",
                                                        &self.restriction_next_container_input,
                                                        cx,
                                                    ))
                                                },
                                            )
                                            .into_any_element()
                                    }),
                            )
                    }),
            )
            .child(
                Collapsible::new()
                    .open(self.documentation_open)
                    .child(
                        Button::new("toggle-sequence-container-documentation")
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
                    .content(v_flex().pt_3().child(field(
                        "Long description",
                        "Optional",
                        &self.long_description_input,
                        cx,
                    ))),
            )
            .child(
                Collapsible::new()
                    .open(self.metadata_open)
                    .child(
                        Button::new("toggle-sequence-container-metadata")
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
                            .child(self.ancillary_data_set.render(cx)),
                    ),
            )
            .child(
                Collapsible::new()
                    .open(self.stream_rate_open)
                    .child(
                        Button::new("toggle-sequence-container-stream-rate")
                            .small()
                            .link()
                            .icon(if self.stream_rate_open {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .label("Stream rate")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.stream_rate_open = !this.stream_rate_open;
                                cx.notify();
                            })),
                    )
                    .content(v_flex().pt_3().child(self.rate.clone())),
            )
            .child(
                Collapsible::new()
                    .open(self.advanced_settings_open)
                    .child(
                        Button::new("toggle-sequence-container-advanced-settings")
                            .small()
                            .link()
                            .icon(if self.advanced_settings_open {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .label("Advanced settings")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.advanced_settings_open = !this.advanced_settings_open;
                                cx.notify();
                            })),
                    )
                    .content(
                        v_flex()
                            .pt_3()
                            .gap_4()
                            .child(field(
                                "Idle pattern",
                                "Optional; defaults to 0",
                                &self.idle_pattern_input,
                                cx,
                            ))
                            .child(self.binary_encoding.clone()),
                    ),
            )
            .child(self.entry_list.clone())
    }
}

impl Render for SequenceContainerForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

#[derive(Clone)]
struct EntryRowData {
    source_index: Rc<Cell<Option<usize>>>,
    preserve_complex_location: Rc<Cell<bool>>,
    content: EntryRowContent,
}

#[derive(Clone)]
enum EntryRowContent {
    Editable {
        kind: EntryKind,
        reference: String,
        segment_size: String,
        segment_order: String,
        offset: String,
        description: String,
    },
    Unsupported {
        label: &'static str,
    },
}

struct TelemetryEntryListView {
    rows: Vec<EntryRowData>,
    editors: HashMap<usize, Entity<TelemetryEntryRow>>,
    cache_order: VecDeque<usize>,
    context: Rc<RefCell<ReferenceContext>>,
    base_container_ref_input: Entity<InputState>,
    container_name_input: Entity<InputState>,
    optional_columns_visible: Rc<Cell<bool>>,
    packet_layout_open: bool,
    bit_positions: Vec<Option<u64>>,
    list_state: ListState,
}

struct TelemetryEntryRow {
    kind_select: Entity<SelectState<Vec<EntryKind>>>,
    reference_input: Entity<InputState>,
    reference_kind: EntryKind,
    segment_size_input: Entity<InputState>,
    segment_order_input: Entity<InputState>,
    offset_input: Entity<InputState>,
    description_input: Entity<InputState>,
    source_index: Rc<Cell<Option<usize>>>,
    preserve_complex_location: Rc<Cell<bool>>,
    optional_columns_visible: Rc<Cell<bool>>,
    _subscriptions: Vec<Subscription>,
}

struct ComparisonListForm {
    rows: Vec<Entity<ComparisonRowForm>>,
    context: Rc<RefCell<ReferenceContext>>,
}

struct ComparisonRowForm {
    parameter_ref: Entity<InputState>,
    operator: Entity<SelectState<Vec<ComparisonOperator>>>,
    comparison_value: Entity<InputState>,
    instance: Entity<InputState>,
    calibrated: Entity<SelectState<Vec<CalibratedChoice>>>,
}

impl ComparisonListForm {
    fn new(
        value: &str,
        context: Rc<RefCell<ReferenceContext>>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = comparison_models(value)
            .into_iter()
            .map(|model| comparison_row(model, context.clone(), window, cx))
            .collect();
        cx.new(move |_| Self { rows, context })
    }

    fn load(&mut self, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.rows = comparison_models(value)
            .into_iter()
            .map(|model| comparison_row(model, self.context.clone(), window, cx))
            .collect();
        cx.notify();
    }

    fn to_criteria(&self, cx: &App) -> Option<xtce::RestrictionCriteriaType> {
        let comparison = self
            .rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let parameter_ref = value(&row.parameter_ref, cx).trim().to_owned();
                let comparison_value = value(&row.comparison_value, cx).trim().to_owned();
                if parameter_ref.is_empty() || comparison_value.is_empty() {
                    return None;
                }
                Some(xtce::ComparisonType {
                    parameter_ref,
                    comparison_operator: selected_value(
                        &row.operator,
                        ComparisonOperator::Equal,
                        cx,
                    )
                    .to_string(),
                    value: comparison_value,
                    instance: value(&row.instance, cx)
                        .trim()
                        .parse()
                        .unwrap_or_else(|_| xtce::ComparisonType::default_instance()),
                    use_calibrated_value: selected_value(
                        &row.calibrated,
                        CalibratedChoice::CalibratedValue,
                        cx,
                    ) == CalibratedChoice::CalibratedValue,
                })
            })
            .collect::<Vec<_>>();
        (!comparison.is_empty()).then_some(xtce::RestrictionCriteriaType {
            content: Some(xtce::RestrictionCriteriaTypeContent::ComparisonList(
                xtce::ComparisonListType { comparison },
            )),
        })
    }
}

impl Render for ComparisonListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.rows.len();
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Comparisons"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("All comparison rows must evaluate to true"),
                            ),
                    )
                    .child(
                        Button::new("add-restriction-comparison")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add comparison")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(comparison_row(
                                    ComparisonRowModel::default(),
                                    this.context.clone(),
                                    window,
                                    cx,
                                ));
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .id("comparison-table-scroll")
                    .w_full()
                    .overflow_x_scroll()
                    .child(
                        v_flex()
                            .min_w(px(820.))
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                h_flex()
                                    .h(px(34.))
                                    .px_2()
                                    .gap_2()
                                    .bg(cx.theme().muted.opacity(0.5))
                                    .text_xs()
                                    .font_medium()
                                    .child(div().flex_1().child("Parameter reference"))
                                    .child(div().w(px(110.)).child("Operator"))
                                    .child(div().flex_1().child("Value"))
                                    .child(div().w(px(90.)).child("Instance"))
                                    .child(div().w(px(130.)).child("Compare using"))
                                    .child(div().w(px(52.)).child("Actions")),
                            )
                            .children(self.rows.iter().enumerate().map(|(index, row)| {
                                h_flex()
                                    .h(px(52.))
                                    .px_2()
                                    .gap_2()
                                    .border_b_1()
                                    .border_color(cx.theme().border)
                                    .child(row.clone())
                                    .child(
                                        div().w(px(52.)).flex_none().child(
                                            Button::new(format!(
                                                "remove-restriction-comparison-{index}"
                                            ))
                                            .small()
                                            .ghost()
                                            .icon(IconName::Minus)
                                            .tooltip("Remove comparison")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if index < this.rows.len() {
                                                    this.rows.remove(index);
                                                    cx.notify();
                                                }
                                            })),
                                        ),
                                    )
                            })),
                    ),
            )
            .when(count == 0, |form| {
                form.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("No restriction comparison is defined."),
                )
            })
    }
}

impl Render for ComparisonRowForm {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(div().flex_1().child(Input::new(&self.parameter_ref)))
            .child(
                div()
                    .w(px(110.))
                    .flex_none()
                    .child(Select::new(&self.operator).w_full()),
            )
            .child(div().flex_1().child(Input::new(&self.comparison_value)))
            .child(
                div()
                    .w(px(90.))
                    .flex_none()
                    .child(Input::new(&self.instance)),
            )
            .child(
                div()
                    .w(px(130.))
                    .flex_none()
                    .child(Select::new(&self.calibrated).w_full()),
            )
    }
}

impl TelemetryEntryListView {
    fn new(
        rows: Vec<EntryRowData>,
        context: Rc<RefCell<ReferenceContext>>,
        base_container_ref_input: Entity<InputState>,
        container_name_input: Entity<InputState>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        let optional_columns_visible = Rc::new(Cell::new(false));
        let row_count = rows.len();
        Self {
            list_state: ListState::new(rows.len(), ListAlignment::Top, px(58.))
                .with_uniform_item_height(px(54.)),
            rows,
            editors: HashMap::new(),
            cache_order: VecDeque::new(),
            context,
            base_container_ref_input,
            container_name_input,
            optional_columns_visible,
            packet_layout_open: true,
            bit_positions: vec![None; row_count],
        }
    }

    fn set_rows(&mut self, rows: Vec<EntryRowData>, cx: &mut Context<Self>) {
        self.rows = rows;
        self.editors.clear();
        self.cache_order.clear();
        self.optional_columns_visible.set(false);
        self.packet_layout_open = true;
        self.bit_positions = vec![None; self.rows.len()];
        self.list_state
            .reset_with_uniform_height(self.rows.len(), px(54.));
        cx.notify();
    }

    fn editor(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Entity<TelemetryEntryRow>> {
        if !matches!(self.rows[index].content, EntryRowContent::Editable { .. }) {
            return None;
        }
        if let Some(editor) = self.editors.get(&index).cloned() {
            touch_cache(&mut self.cache_order, index);
            return Some(editor);
        }
        let editor = new_entry_row(
            self.rows[index].clone(),
            self.context.clone(),
            self.optional_columns_visible.clone(),
            window,
            cx,
        );
        self.editors.insert(index, editor.clone());
        touch_cache(&mut self.cache_order, index);
        while self.editors.len() > 24 {
            let Some(evicted) = self.cache_order.pop_front() else {
                break;
            };
            if evicted == index {
                self.cache_order.push_back(evicted);
                continue;
            }
            if let Some(editor) = self.editors.remove(&evicted) {
                self.rows[evicted] = entry_row_data(&editor, cx);
            }
        }
        Some(editor)
    }

    fn flush_editors(&mut self, cx: &App) {
        for (index, editor) in &self.editors {
            self.rows[*index] = entry_row_data(editor, cx);
        }
    }

    fn current_rows(&self, cx: &App) -> Vec<EntryRowData> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                self.editors
                    .get(&index)
                    .map_or_else(|| row.clone(), |editor| entry_row_data(editor, cx))
            })
            .collect()
    }

    fn move_entry(&mut self, index: usize, target: usize, cx: &mut Context<Self>) {
        if index >= self.rows.len() || target >= self.rows.len() || index == target {
            return;
        }
        self.flush_editors(cx);
        self.rows.swap(index, target);
        self.editors.clear();
        self.cache_order.clear();
        cx.notify();
    }

    fn remove_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.rows.len() {
            return;
        }
        self.flush_editors(cx);
        self.rows.remove(index);
        self.editors.clear();
        self.cache_order.clear();
        self.list_state.splice(index..index + 1, 0);
        cx.notify();
    }

    fn add_parameter_reference(&mut self, reference: String, cx: &mut Context<Self>) {
        let index = self.rows.len();
        self.rows
            .push(EntryRowData::new_parameter_reference(reference));
        self.list_state.splice(index..index, 1);
        self.list_state.scroll_to_reveal_item(index);
        cx.notify();
    }

    fn render_list_item(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let controls = h_flex()
            .w(px(98.))
            .flex_none()
            .gap_1()
            .child(
                Button::new(format!("move-telemetry-entry-up-{index}"))
                    .ghost()
                    .small()
                    .icon(IconName::ArrowUp)
                    .disabled(index == 0)
                    .tooltip("Move entry up")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(target) = index.checked_sub(1) {
                            this.move_entry(index, target, cx);
                        }
                    })),
            )
            .child(
                Button::new(format!("move-telemetry-entry-down-{index}"))
                    .ghost()
                    .small()
                    .icon(IconName::ArrowDown)
                    .disabled(index + 1 >= self.rows.len())
                    .tooltip("Move entry down")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.move_entry(index, index + 1, cx);
                    })),
            )
            .child(
                Button::new(format!("remove-telemetry-entry-{index}"))
                    .ghost()
                    .small()
                    .icon(IconName::Minus)
                    .tooltip("Remove entry")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.remove_entry(index, cx);
                    })),
            );
        let content = match self.editor(index, window, cx) {
            Some(editor) => editor.into_any_element(),
            None => {
                let label = match &self.rows[index].content {
                    EntryRowContent::Unsupported { label } => *label,
                    EntryRowContent::Editable { .. } => unreachable!(),
                };
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .px_2()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{label} (preserved; editing is not yet supported)"))
                    .into_any_element()
            }
        };
        h_flex()
            .w_full()
            .h(px(54.))
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div().w(px(72.)).flex_none().text_sm().child(
                    self.bit_positions
                        .get(index)
                        .copied()
                        .flatten()
                        .map_or_else(|| "—".to_owned(), |position| position.to_string()),
                ),
            )
            .child(controls)
            .child(content)
            .into_any_element()
    }
}

impl Render for TelemetryEntryListView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let row_count = self.rows.len();
        let optional_columns_visible = self.optional_columns_visible.get();
        let reference_context = self.context.clone();
        let rows = self.current_rows(cx);
        let base_container_ref = value(&self.base_container_ref_input, cx);
        let container_name = value(&self.container_name_input, cx);
        let packet_layout = {
            let context = self.context.borrow();
            self.bit_positions = entry_bit_positions(
                &rows,
                &context.parameter_sizes,
                &base_container_ref,
                &context.container_layouts,
            );
            self.packet_layout_open.then(|| {
                packet_layout(
                    &rows,
                    &context.parameter_sizes,
                    &base_container_ref,
                    &context.container_layouts,
                    &container_name,
                )
            })
        };
        v_flex()
            .id("telemetry-entry-list-drop-target")
            .w_full()
            .gap_3()
            .can_drop(move |value, _, _| {
                value
                    .downcast_ref::<DraggedTelemetryParameter>()
                    .is_some_and(|parameter| {
                        reference_context
                            .borrow()
                            .parameter_names
                            .contains(&parameter.reference)
                    })
            })
            .drag_over::<DraggedTelemetryParameter>(|style, _, _, cx| {
                style
                    .border_2()
                    .border_color(cx.theme().drag_border)
                    .bg(cx.theme().sidebar_accent.opacity(0.25))
            })
            .on_drop(
                cx.listener(|this, parameter: &DraggedTelemetryParameter, _, cx| {
                    this.add_parameter_reference(parameter.reference.clone(), cx);
                }),
            )
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Entry list"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{row_count} entries")),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("toggle-telemetry-entry-optional-columns")
                                    .small()
                                    .label(if optional_columns_visible {
                                        "Hide optional columns"
                                    } else {
                                        "Show optional columns"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.optional_columns_visible
                                            .set(!this.optional_columns_visible.get());
                                        for editor in this.editors.values() {
                                            editor.update(cx, |_, cx| cx.notify());
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("add-telemetry-container-entry")
                                    .small()
                                    .icon(IconName::Plus)
                                    .label("Add entry")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let index = this.rows.len();
                                        this.rows
                                            .push(EntryRowData::new_editable(EntryKind::Parameter));
                                        this.list_state.splice(index..index, 1);
                                        this.list_state.scroll_to_reveal_item(index);
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .id("telemetry-entry-table-scroll-boundary")
                    .w_full()
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .id("telemetry-entry-table-horizontal-scroll")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                v_flex()
                                    .min_w(if optional_columns_visible {
                                        px(1_078.)
                                    } else {
                                        px(638.)
                                    })
                                    .rounded_md()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .child(
                                        h_flex()
                                            .h(px(34.))
                                            .px_2()
                                            .gap_2()
                                            .bg(cx.theme().muted.opacity(0.5))
                                            .text_xs()
                                            .font_medium()
                                            .child(div().w(px(72.)).child("Bit position"))
                                            .child(div().w(px(98.)).child("Actions"))
                                            .child(div().w(px(170.)).child("Type"))
                                            .child(div().flex_1().child("Reference target"))
                                            .when(optional_columns_visible, |header| {
                                                header
                                                    .child(div().w(px(100.)).child("Segment size"))
                                                    .child(div().w(px(80.)).child("Order"))
                                                    .child(div().w(px(100.)).child("Offset"))
                                                    .child(div().flex_1().child("Description"))
                                            }),
                                    )
                                    .child(
                                        list(
                                            self.list_state.clone(),
                                            cx.processor(TelemetryEntryListView::render_list_item),
                                        )
                                        .w_full()
                                        .h(px(352.)),
                                    ),
                            ),
                    ),
            )
            .child(
                Collapsible::new()
                    .open(self.packet_layout_open)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                Button::new("toggle-telemetry-packet-layout")
                                    .small()
                                    .link()
                                    .icon(if self.packet_layout_open {
                                        IconName::ChevronDown
                                    } else {
                                        IconName::ChevronRight
                                    })
                                    .label("Packet layout")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.packet_layout_open = !this.packet_layout_open;
                                        cx.notify();
                                    })),
                            )
                            .when(self.packet_layout_open, |header| {
                                header.child(
                                    Button::new("refresh-telemetry-packet-layout")
                                        .small()
                                        .ghost()
                                        .label("Refresh")
                                        .on_click(cx.listener(|_, _, _, cx| cx.notify())),
                                )
                            }),
                    )
                    .content(
                        v_flex().pt_2().child(
                            packet_layout
                                .as_ref()
                                .map_or_else(|| div(), |layout| render_packet_layout(layout, cx)),
                        ),
                    ),
            )
    }
}

const PACKET_LAYOUT_BYTES_PER_ROW: usize = 8;
const PACKET_LAYOUT_BIT_WIDTH: f32 = 11.;

#[derive(Debug, PartialEq, Eq)]
struct PacketField {
    label: String,
    start_bit: u64,
    size_bits: u64,
    inherited: bool,
    source: String,
}

struct PacketLayout {
    fields: Vec<PacketField>,
    total_bits: u64,
    unresolved: Vec<String>,
}

fn packet_layout(
    rows: &[EntryRowData],
    parameter_sizes: &HashMap<String, u64>,
    base_container_ref: &str,
    container_layouts: &HashMap<String, ContainerLayoutSource>,
    current_source: &str,
) -> PacketLayout {
    let mut layout = PacketLayout {
        fields: Vec::new(),
        total_bits: 0,
        unresolved: Vec::new(),
    };
    let mut cursor = Some(0_u64);
    if !base_container_ref.trim().is_empty() {
        append_base_container(
            base_container_ref.trim(),
            parameter_sizes,
            container_layouts,
            &mut HashSet::new(),
            &mut cursor,
            &mut layout,
        );
    }
    append_packet_rows(
        rows,
        false,
        current_source,
        parameter_sizes,
        &mut cursor,
        &mut layout,
    );
    layout.total_bits = cursor.unwrap_or_else(|| {
        layout
            .fields
            .iter()
            .map(|field| field.start_bit + field.size_bits)
            .max()
            .unwrap_or(0)
    });
    layout
}

fn entry_bit_positions(
    rows: &[EntryRowData],
    parameter_sizes: &HashMap<String, u64>,
    base_container_ref: &str,
    container_layouts: &HashMap<String, ContainerLayoutSource>,
) -> Vec<Option<u64>> {
    let mut base_layout = PacketLayout {
        fields: Vec::new(),
        total_bits: 0,
        unresolved: Vec::new(),
    };
    let mut cursor = Some(0_u64);
    if !base_container_ref.trim().is_empty() {
        append_base_container(
            base_container_ref.trim(),
            parameter_sizes,
            container_layouts,
            &mut HashSet::new(),
            &mut cursor,
            &mut base_layout,
        );
    }

    rows.iter()
        .map(|row| {
            let EntryRowContent::Editable {
                kind,
                reference,
                segment_size,
                offset,
                ..
            } = &row.content
            else {
                cursor = None;
                return None;
            };
            let offset = if offset.trim().is_empty() {
                Some(0)
            } else {
                offset.trim().parse::<u64>().ok()
            };
            let start =
                cursor.and_then(|position| offset.and_then(|offset| position.checked_add(offset)));
            let size = match kind {
                EntryKind::Parameter => parameter_sizes.get(reference).copied(),
                EntryKind::ParameterSegment => segment_size.trim().parse().ok(),
                EntryKind::Container => None,
                EntryKind::ContainerSegment => segment_size.trim().parse().ok(),
                EntryKind::StreamSegment => segment_size.trim().parse().ok(),
            };
            cursor = start.and_then(|start| size.and_then(|size| start.checked_add(size)));
            start.filter(|_| size.is_some())
        })
        .collect()
}

fn append_base_container(
    name: &str,
    parameter_sizes: &HashMap<String, u64>,
    container_layouts: &HashMap<String, ContainerLayoutSource>,
    visited: &mut HashSet<String>,
    cursor: &mut Option<u64>,
    layout: &mut PacketLayout,
) {
    if !visited.insert(name.to_owned()) {
        layout
            .unresolved
            .push(format!("Base container {name}: circular reference"));
        *cursor = None;
        return;
    }
    let Some(source) = container_layouts.get(name) else {
        layout
            .unresolved
            .push(format!("Base container {name}: not found"));
        *cursor = None;
        return;
    };
    if let Some(base) = source
        .base_container_ref
        .as_deref()
        .filter(|base| !base.is_empty())
    {
        append_base_container(
            base,
            parameter_sizes,
            container_layouts,
            visited,
            cursor,
            layout,
        );
    }
    append_packet_rows(&source.rows, true, name, parameter_sizes, cursor, layout);
    visited.remove(name);
}

fn append_packet_rows(
    rows: &[EntryRowData],
    inherited: bool,
    source: &str,
    parameter_sizes: &HashMap<String, u64>,
    cursor: &mut Option<u64>,
    layout: &mut PacketLayout,
) {
    for row in rows {
        let EntryRowContent::Editable {
            kind,
            reference,
            segment_size,
            offset,
            ..
        } = &row.content
        else {
            layout.unresolved.push("Unsupported entry".to_owned());
            *cursor = None;
            continue;
        };
        if *kind == EntryKind::Container {
            layout
                .unresolved
                .push(format!("{reference}: container size is unknown"));
            *cursor = None;
            continue;
        }
        let offset = if offset.trim().is_empty() {
            Some(0)
        } else {
            offset.trim().parse::<u64>().ok()
        };
        let Some(start) =
            cursor.and_then(|cursor| offset.and_then(|offset| cursor.checked_add(offset)))
        else {
            layout
                .unresolved
                .push(format!("{reference}: offset is unresolved"));
            *cursor = None;
            continue;
        };
        let size = match kind {
            EntryKind::Parameter => parameter_sizes.get(reference).copied(),
            EntryKind::ParameterSegment => segment_size.trim().parse().ok(),
            EntryKind::Container => None,
            EntryKind::ContainerSegment => segment_size.trim().parse().ok(),
            EntryKind::StreamSegment => segment_size.trim().parse().ok(),
        };
        let Some(size) = size else {
            layout
                .unresolved
                .push(format!("{reference}: size is unknown"));
            *cursor = None;
            continue;
        };
        let Some(end) = start.checked_add(size) else {
            layout
                .unresolved
                .push(format!("{reference}: range is too large"));
            *cursor = None;
            continue;
        };
        layout.fields.push(PacketField {
            label: reference.clone(),
            start_bit: start,
            size_bits: size,
            inherited,
            source: source.to_owned(),
        });
        *cursor = Some(end);
    }
}

fn render_packet_layout(layout: &PacketLayout, cx: &App) -> Div {
    let visible_bits = layout.total_bits;
    let bits_per_row = (PACKET_LAYOUT_BYTES_PER_ROW * 8) as u64;
    let row_count = usize::try_from(visible_bits.div_ceil(bits_per_row)).unwrap_or(0);
    let mut content = v_flex().w_full().gap_2().child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child("Fixed-size parameters · inherited fields use a lighter shade"),
    );

    if row_count == 0 {
        content = content.child(
            div()
                .p_4()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("No fixed-size packet fields can be resolved."),
        );
    } else {
        content = content.child(
            div()
                .id("telemetry-packet-layout-scroll")
                .w_full()
                .overflow_x_scroll()
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .child(
                    v_flex()
                        .min_w(px(68. + PACKET_LAYOUT_BIT_WIDTH * bits_per_row as f32))
                        .gap_1()
                        .child(
                            h_flex()
                                .ml(px(68.))
                                .children((0..PACKET_LAYOUT_BYTES_PER_ROW).map(|byte| {
                                    div()
                                        .w(px(PACKET_LAYOUT_BIT_WIDTH * 8.))
                                        .flex_none()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("+{byte}"))
                                })),
                        )
                        .children((0..row_count).map(|row| {
                            let row_start = row as u64 * bits_per_row;
                            let row_end = row_start + bits_per_row;
                            let mut cursor = row_start;
                            let mut segments = Vec::new();
                            for field in layout.fields.iter().filter(|field| {
                                field.start_bit < row_end
                                    && field.start_bit + field.size_bits > row_start
                            }) {
                                let field_start = field.start_bit.max(row_start);
                                let field_end = (field.start_bit + field.size_bits).min(row_end);
                                if field_start > cursor {
                                    segments.push(packet_segment(
                                        field_start - cursor,
                                        "Unused".to_owned(),
                                        false,
                                        false,
                                        format!("packet-layout-gap-{row}-{cursor}"),
                                        format!(
                                            "Unused\nBit offset: {cursor}\nSize: {} bits",
                                            field_start - cursor
                                        ),
                                        cx,
                                    ));
                                }
                                let continuation = field.start_bit < row_start;
                                let label = if continuation {
                                    format!("… {}", field.label)
                                } else if field.size_bits % 8 == 0 {
                                    format!("{} ({} B)", field.label, field.size_bits / 8)
                                } else {
                                    format!("{} ({} b)", field.label, field.size_bits)
                                };
                                segments.push(packet_segment(
                                    field_end - field_start,
                                    label,
                                    true,
                                    field.inherited,
                                    format!(
                                        "packet-layout-field-{row}-{}",
                                        field.start_bit
                                    ),
                                    format!(
                                        "{}\nBit offset: {} (byte 0x{:04X}, bit {})\nSize: {} bits{}\nSource: {}",
                                        field.label,
                                        field.start_bit,
                                        field.start_bit / 8,
                                        field.start_bit % 8,
                                        field.size_bits,
                                        if field.size_bits % 8 == 0 {
                                            format!(" / {} bytes", field.size_bits / 8)
                                        } else {
                                            String::new()
                                        },
                                        field.source
                                    ),
                                    cx,
                                ));
                                cursor = cursor.max(field_end);
                            }
                            if cursor < row_end {
                                segments.push(packet_segment(
                                    row_end - cursor,
                                    "Unused".to_owned(),
                                    false,
                                    false,
                                    format!("packet-layout-gap-{row}-{cursor}"),
                                    format!(
                                        "Unused\nBit offset: {cursor}\nSize: {} bits",
                                        row_end - cursor
                                    ),
                                    cx,
                                ));
                            }
                            h_flex()
                                .child(
                                    div()
                                        .w(px(68.))
                                        .flex_none()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("0x{:04X}", row_start / 8)),
                                )
                                .children(segments)
                        })),
                ),
        );
    }

    if !layout.unresolved.is_empty() {
        content = content.child(
            v_flex()
                .gap_1()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Unresolved entries")
                .children(
                    layout
                        .unresolved
                        .iter()
                        .map(|entry| div().child(format!("• {entry}"))),
                ),
        );
    }
    content
}

fn packet_segment(
    bits: u64,
    label: String,
    occupied: bool,
    inherited: bool,
    id: String,
    tooltip: String,
    cx: &App,
) -> AnyElement {
    div()
        .id(id)
        .w(px(bits as f32 * PACKET_LAYOUT_BIT_WIDTH))
        .h(px(42.))
        .flex_none()
        .px_1()
        .flex()
        .items_center()
        .border_1()
        .border_color(cx.theme().border)
        .when(occupied, |segment| {
            segment.bg(cx
                .theme()
                .sidebar_accent
                .opacity(if inherited { 0.28 } else { 0.62 }))
        })
        .text_xs()
        .truncate()
        .child(label)
        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
        .into_any_element()
}

impl Render for TelemetryEntryRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let optional_columns_visible = self.optional_columns_visible.get();
        let kind = selected_value(&self.kind_select, EntryKind::Parameter, cx);
        if self.reference_kind != kind {
            self.reference_kind = kind;
            self.reference_input.update(cx, |input, cx| {
                input.set_value("", window, cx);
                input.set_placeholder(entry_reference_placeholder(kind), window, cx);
            });
        }
        let segment = matches!(
            kind,
            EntryKind::ParameterSegment | EntryKind::ContainerSegment | EntryKind::StreamSegment
        );
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(
                div()
                    .w(px(170.))
                    .flex_none()
                    .child(Select::new(&self.kind_select).w_full()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Input::new(&self.reference_input)),
            )
            .when(optional_columns_visible, |row| {
                row.child(div().w(px(100.)).flex_none().when(segment, |cell| {
                    cell.child(Input::new(&self.segment_size_input))
                }))
                .child(div().w(px(80.)).flex_none().when(segment, |cell| {
                    cell.child(Input::new(&self.segment_order_input))
                }))
                .child(
                    div()
                        .w(px(100.))
                        .flex_none()
                        .child(Input::new(&self.offset_input)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Input::new(&self.description_input)),
                )
            })
    }
}

impl EntryRowData {
    fn new_editable(kind: EntryKind) -> Self {
        Self {
            source_index: Rc::new(Cell::new(None)),
            preserve_complex_location: Rc::new(Cell::new(false)),
            content: EntryRowContent::Editable {
                kind,
                reference: String::new(),
                segment_size: String::new(),
                segment_order: String::new(),
                offset: String::new(),
                description: String::new(),
            },
        }
    }

    fn new_parameter_reference(reference: String) -> Self {
        let mut row = Self::new_editable(EntryKind::Parameter);
        let EntryRowContent::Editable {
            reference: row_reference,
            ..
        } = &mut row.content
        else {
            unreachable!()
        };
        *row_reference = reference;
        row
    }
}

fn new_entry_row(
    row: EntryRowData,
    context: Rc<RefCell<ReferenceContext>>,
    optional_columns_visible: Rc<Cell<bool>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<TelemetryEntryRow> {
    let EntryRowContent::Editable {
        kind,
        reference,
        segment_size,
        segment_order,
        offset,
        description,
    } = row.content
    else {
        unreachable!()
    };
    cx.new(|cx| {
        let kind_select = select(EntryKind::VARIANTS, kind, window, cx);
        let kind_subscription = cx
            .subscribe(&kind_select, |_, _, _: &SelectEvent<Vec<EntryKind>>, cx| {
                cx.notify()
            });
        let reference_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .default_value(reference)
                .placeholder(entry_reference_placeholder(kind));
            input.lsp.completion_provider = Some(Rc::new(ReferenceCompletionProvider {
                context,
                target: CompletionTarget::Entry(kind_select.clone()),
            }));
            input
        });
        TelemetryEntryRow {
            kind_select,
            reference_input,
            reference_kind: kind,
            segment_size_input: input(&segment_size, false, window, cx),
            segment_order_input: input(&segment_order, false, window, cx),
            offset_input: input(&offset, false, window, cx),
            description_input: input(&description, false, window, cx),
            source_index: row.source_index,
            preserve_complex_location: row.preserve_complex_location,
            optional_columns_visible,
            _subscriptions: vec![kind_subscription],
        }
    })
}

fn entry_row_data(row: &Entity<TelemetryEntryRow>, cx: &App) -> EntryRowData {
    let row = row.read(cx);
    EntryRowData {
        source_index: row.source_index.clone(),
        preserve_complex_location: row.preserve_complex_location.clone(),
        content: EntryRowContent::Editable {
            kind: selected_value(&row.kind_select, EntryKind::Parameter, cx),
            reference: value(&row.reference_input, cx),
            segment_size: value(&row.segment_size_input, cx),
            segment_order: value(&row.segment_order_input, cx),
            offset: value(&row.offset_input, cx),
            description: value(&row.description_input, cx),
        },
    }
}

fn rows_from_entry_list(list: Option<&xtce::EntryListType>) -> Vec<EntryRowData> {
    list.into_iter()
        .flat_map(|list| &list.content)
        .enumerate()
        .map(|(index, entry)| {
            let (content, complex_location) = match entry {
                xtce::EntryListTypeContent::ParameterRefEntry(entry) => (
                    EntryRowContent::Editable {
                        kind: EntryKind::Parameter,
                        reference: entry.parameter_ref.clone(),
                        segment_size: String::new(),
                        segment_order: String::new(),
                        offset: fixed_offset(entry.location_in_container_in_bits.as_ref()),
                        description: entry.short_description.clone().unwrap_or_default(),
                    },
                    has_complex_location(entry.location_in_container_in_bits.as_ref()),
                ),
                xtce::EntryListTypeContent::ParameterSegmentRefEntry(entry) => (
                    EntryRowContent::Editable {
                        kind: EntryKind::ParameterSegment,
                        reference: entry.parameter_ref.clone(),
                        segment_size: entry.size_in_bits.to_string(),
                        segment_order: entry
                            .order
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        offset: fixed_offset(entry.location_in_container_in_bits.as_ref()),
                        description: entry.short_description.clone().unwrap_or_default(),
                    },
                    has_complex_location(entry.location_in_container_in_bits.as_ref()),
                ),
                xtce::EntryListTypeContent::ContainerRefEntry(entry) => (
                    EntryRowContent::Editable {
                        kind: EntryKind::Container,
                        reference: entry.container_ref.clone(),
                        segment_size: String::new(),
                        segment_order: String::new(),
                        offset: fixed_offset(entry.location_in_container_in_bits.as_ref()),
                        description: entry.short_description.clone().unwrap_or_default(),
                    },
                    has_complex_location(entry.location_in_container_in_bits.as_ref()),
                ),
                xtce::EntryListTypeContent::ContainerSegmentRefEntry(entry) => (
                    EntryRowContent::Editable {
                        kind: EntryKind::ContainerSegment,
                        reference: entry.container_ref.clone(),
                        segment_size: entry.size_in_bits.to_string(),
                        segment_order: entry
                            .order
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        offset: fixed_offset(entry.location_in_container_in_bits.as_ref()),
                        description: entry.short_description.clone().unwrap_or_default(),
                    },
                    has_complex_location(entry.location_in_container_in_bits.as_ref()),
                ),
                xtce::EntryListTypeContent::StreamSegmentEntry(entry) => (
                    EntryRowContent::Editable {
                        kind: EntryKind::StreamSegment,
                        reference: entry.stream_ref.clone(),
                        segment_size: entry.size_in_bits.to_string(),
                        segment_order: entry
                            .order
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        offset: fixed_offset(entry.location_in_container_in_bits.as_ref()),
                        description: entry.short_description.clone().unwrap_or_default(),
                    },
                    has_complex_location(entry.location_in_container_in_bits.as_ref()),
                ),
                entry => (
                    EntryRowContent::Unsupported {
                        label: unsupported_entry_label(entry),
                    },
                    false,
                ),
            };
            EntryRowData {
                source_index: Rc::new(Cell::new(Some(index))),
                preserve_complex_location: Rc::new(Cell::new(complex_location)),
                content,
            }
        })
        .collect()
}

fn apply_entry_rows(list: &mut xtce::EntryListType, rows: Vec<EntryRowData>) {
    let mut original = std::mem::take(&mut list.content)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    for (new_index, row) in rows.into_iter().enumerate() {
        let source = row
            .source_index
            .get()
            .and_then(|index| original.get_mut(index))
            .and_then(Option::take);
        let entry = match row.content {
            EntryRowContent::Unsupported { .. } => {
                let Some(entry) = source else {
                    continue;
                };
                entry
            }
            EntryRowContent::Editable {
                kind,
                reference,
                segment_size,
                segment_order,
                offset,
                description,
            } => match kind {
                EntryKind::Parameter => {
                    let mut entry = match source {
                        Some(xtce::EntryListTypeContent::ParameterRefEntry(entry)) => entry,
                        _ => default_parameter_ref_entry(),
                    };
                    entry.parameter_ref = reference;
                    entry.short_description = optional_value(description);
                    apply_location(
                        &mut entry.location_in_container_in_bits,
                        &offset,
                        &row.preserve_complex_location,
                    );
                    xtce::EntryListTypeContent::ParameterRefEntry(entry)
                }
                EntryKind::ParameterSegment => {
                    let mut entry = match source {
                        Some(xtce::EntryListTypeContent::ParameterSegmentRefEntry(entry)) => entry,
                        _ => default_parameter_segment_ref_entry(),
                    };
                    entry.parameter_ref = reference;
                    entry.size_in_bits = segment_size.trim().parse().unwrap_or_default();
                    entry.order = segment_order.trim().parse().ok();
                    entry.short_description = optional_value(description);
                    apply_location(
                        &mut entry.location_in_container_in_bits,
                        &offset,
                        &row.preserve_complex_location,
                    );
                    xtce::EntryListTypeContent::ParameterSegmentRefEntry(entry)
                }
                EntryKind::Container => {
                    let mut entry = match source {
                        Some(xtce::EntryListTypeContent::ContainerRefEntry(entry)) => entry,
                        _ => default_container_ref_entry(),
                    };
                    entry.container_ref = reference;
                    entry.short_description = optional_value(description);
                    apply_location(
                        &mut entry.location_in_container_in_bits,
                        &offset,
                        &row.preserve_complex_location,
                    );
                    xtce::EntryListTypeContent::ContainerRefEntry(entry)
                }
                EntryKind::ContainerSegment => {
                    let mut entry = match source {
                        Some(xtce::EntryListTypeContent::ContainerSegmentRefEntry(entry)) => entry,
                        _ => default_container_segment_ref_entry(),
                    };
                    entry.container_ref = reference;
                    entry.size_in_bits = segment_size.trim().parse().unwrap_or_default();
                    entry.order = segment_order.trim().parse().ok();
                    entry.short_description = optional_value(description);
                    apply_location(
                        &mut entry.location_in_container_in_bits,
                        &offset,
                        &row.preserve_complex_location,
                    );
                    xtce::EntryListTypeContent::ContainerSegmentRefEntry(entry)
                }
                EntryKind::StreamSegment => {
                    let mut entry = match source {
                        Some(xtce::EntryListTypeContent::StreamSegmentEntry(entry)) => entry,
                        _ => default_stream_segment_entry(),
                    };
                    entry.stream_ref = reference;
                    entry.size_in_bits = segment_size.trim().parse().unwrap_or_default();
                    entry.order = segment_order.trim().parse().ok();
                    entry.short_description = optional_value(description);
                    apply_location(
                        &mut entry.location_in_container_in_bits,
                        &offset,
                        &row.preserve_complex_location,
                    );
                    xtce::EntryListTypeContent::StreamSegmentEntry(entry)
                }
            },
        };
        row.source_index.set(Some(new_index));
        list.content.push(entry);
    }
}

fn apply_location(
    location: &mut Option<xtce::LocationInContainerInBitsType>,
    value: &str,
    preserve_complex: &Cell<bool>,
) {
    if let Ok(value) = value.trim().parse::<i64>() {
        *location = Some(xtce::LocationInContainerInBitsType {
            reference_location: xtce::ReferenceLocationType::PreviousEntry,
            content: xtce::LocationInContainerInBitsTypeContent::FixedValue(value),
        });
        preserve_complex.set(false);
    } else if !preserve_complex.get() {
        *location = None;
    }
}

fn default_parameter_ref_entry() -> xtce::ParameterRefEntryType {
    xtce::ParameterRefEntryType {
        short_description: None,
        parameter_ref: String::new(),
        location_in_container_in_bits: None,
        repeat_entry: None,
        include_condition: None,
        time_association: None,
        ancillary_data_set: None,
    }
}

fn default_parameter_segment_ref_entry() -> xtce::ParameterSegmentRefEntryType {
    xtce::ParameterSegmentRefEntryType {
        short_description: None,
        parameter_ref: String::new(),
        order: None,
        size_in_bits: 0,
        location_in_container_in_bits: None,
        repeat_entry: None,
        include_condition: None,
        time_association: None,
        ancillary_data_set: None,
    }
}

fn default_container_ref_entry() -> xtce::ContainerRefEntryType {
    xtce::ContainerRefEntryType {
        short_description: None,
        container_ref: String::new(),
        location_in_container_in_bits: None,
        repeat_entry: None,
        include_condition: None,
        time_association: None,
        ancillary_data_set: None,
    }
}

fn default_container_segment_ref_entry() -> xtce::ContainerSegmentRefEntryType {
    xtce::ContainerSegmentRefEntryType {
        short_description: None,
        container_ref: String::new(),
        order: None,
        size_in_bits: 0,
        location_in_container_in_bits: None,
        repeat_entry: None,
        include_condition: None,
        time_association: None,
        ancillary_data_set: None,
    }
}

fn default_stream_segment_entry() -> xtce::StreamSegmentEntryType {
    xtce::StreamSegmentEntryType {
        short_description: None,
        stream_ref: String::new(),
        order: None,
        size_in_bits: 0,
        location_in_container_in_bits: None,
        repeat_entry: None,
        include_condition: None,
        time_association: None,
        ancillary_data_set: None,
    }
}

fn fixed_offset(location: Option<&xtce::LocationInContainerInBitsType>) -> String {
    match location.map(|location| &location.content) {
        Some(xtce::LocationInContainerInBitsTypeContent::FixedValue(value)) => value.to_string(),
        _ => String::new(),
    }
}

fn has_complex_location(location: Option<&xtce::LocationInContainerInBitsType>) -> bool {
    matches!(
        location.map(|location| &location.content),
        Some(
            xtce::LocationInContainerInBitsTypeContent::DynamicValue(_)
                | xtce::LocationInContainerInBitsTypeContent::DiscreteLookupList(_)
        )
    )
}

fn unsupported_entry_label(entry: &xtce::EntryListTypeContent) -> &'static str {
    match entry {
        xtce::EntryListTypeContent::ParameterRefEntry(_) => "ParameterRefEntry",
        xtce::EntryListTypeContent::ParameterSegmentRefEntry(_) => "ParameterSegmentRefEntry",
        xtce::EntryListTypeContent::ContainerRefEntry(_) => "ContainerRefEntry",
        xtce::EntryListTypeContent::ContainerSegmentRefEntry(_) => "ContainerSegmentRefEntry",
        xtce::EntryListTypeContent::StreamSegmentEntry(_) => "StreamSegmentEntry",
        xtce::EntryListTypeContent::IndirectParameterRefEntry(_) => "IndirectParameterRefEntry",
        xtce::EntryListTypeContent::ArrayParameterRefEntry(_) => "ArrayParameterRefEntry",
    }
}

#[derive(Default)]
struct ReferenceContext {
    parameter_names: Vec<String>,
    parameter_sizes: HashMap<String, u64>,
    container_names: Vec<String>,
    stream_names: Vec<String>,
    container_layouts: HashMap<String, ContainerLayoutSource>,
}

struct ContainerLayoutSource {
    base_container_ref: Option<String>,
    rows: Vec<EntryRowData>,
}

impl ReferenceContext {
    fn new(
        parameter_set: Option<&xtce::ParameterSetType>,
        parameter_type_set: Option<&xtce::ParameterTypeSetType>,
        container_set: Option<&xtce::ContainerSetType>,
        stream_set: Option<&xtce::StreamSetType>,
    ) -> Self {
        Self::from_sequences(
            parameter_set,
            parameter_type_set,
            stream_set,
            container_set.into_iter().flat_map(|set| &set.content).map(
                |container| match container {
                    xtce::ContainerSetTypeContent::SequenceContainer(container) => container,
                },
            ),
        )
    }

    fn new_for_command(
        parameter_set: Option<&xtce::ParameterSetType>,
        parameter_type_set: Option<&xtce::ParameterTypeSetType>,
        container_set: Option<&xtce::CommandContainerSetType>,
        stream_set: Option<&xtce::StreamSetType>,
    ) -> Self {
        Self::from_sequences(
            parameter_set,
            parameter_type_set,
            stream_set,
            container_set
                .into_iter()
                .flat_map(|set| &set.command_container),
        )
    }

    fn from_sequences<'a>(
        parameter_set: Option<&xtce::ParameterSetType>,
        parameter_type_set: Option<&xtce::ParameterTypeSetType>,
        stream_set: Option<&xtce::StreamSetType>,
        containers: impl IntoIterator<Item = &'a xtce::SequenceContainerType>,
    ) -> Self {
        let type_sizes = parameter_type_sizes(parameter_type_set);
        let containers = containers.into_iter().collect::<Vec<_>>();
        Self {
            parameter_names: parameter_set
                .into_iter()
                .flat_map(|set| &set.content)
                .filter_map(|parameter| match parameter {
                    xtce::ParameterSetTypeContent::Parameter(parameter) => {
                        Some(parameter.name.clone())
                    }
                    xtce::ParameterSetTypeContent::ParameterRef(_) => None,
                })
                .collect(),
            parameter_sizes: parameter_set
                .into_iter()
                .flat_map(|set| &set.content)
                .filter_map(|parameter| match parameter {
                    xtce::ParameterSetTypeContent::Parameter(parameter) => type_sizes
                        .get(&parameter.parameter_type_ref)
                        .copied()
                        .map(|size| (parameter.name.clone(), size)),
                    xtce::ParameterSetTypeContent::ParameterRef(_) => None,
                })
                .collect(),
            container_names: containers
                .iter()
                .map(|container| container.name.clone())
                .collect(),
            stream_names: stream_set
                .into_iter()
                .flat_map(|set| &set.content)
                .map(stream_name)
                .map(str::to_owned)
                .collect(),
            container_layouts: containers
                .iter()
                .map(|container| {
                    (
                        container.name.clone(),
                        ContainerLayoutSource {
                            base_container_ref: container
                                .base_container
                                .as_ref()
                                .map(|base| base.container_ref.clone()),
                            rows: rows_from_entry_list(Some(&container.entry_list)),
                        },
                    )
                })
                .collect(),
        }
    }
}

fn stream_name(stream: &xtce::StreamSetTypeContent) -> &str {
    match stream {
        xtce::StreamSetTypeContent::FixedFrameStream(stream) => &stream.name,
        xtce::StreamSetTypeContent::VariableFrameStream(stream) => &stream.name,
        xtce::StreamSetTypeContent::CustomStream(stream) => &stream.name,
    }
}

fn parameter_type_sizes(set: Option<&xtce::ParameterTypeSetType>) -> HashMap<String, u64> {
    let Some(set) = set else {
        return HashMap::new();
    };
    set.content
        .iter()
        .filter_map(|parameter_type| {
            let name = parameter_type_name(parameter_type).to_owned();
            resolve_parameter_type_size(parameter_type, set, &mut HashSet::new())
                .map(|size| (name, size))
        })
        .collect()
}

fn parameter_type_name(parameter_type: &xtce::ParameterTypeSetTypeContent) -> &str {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => &value.name,
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => &value.name,
    }
}

fn resolve_parameter_type_size(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
    set: &xtce::ParameterTypeSetType,
    visiting: &mut HashSet<String>,
) -> Option<u64> {
    let name = parameter_type_name(parameter_type);
    if !visiting.insert(name.to_owned()) {
        return None;
    }
    let size = match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            encoding_content_size(&value.content)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            encoding_content_size(&value.content)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            encoding_content_size(&value.content)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            encoding_content_size(&value.content)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            encoding_content_size(&value.content)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            encoding_content_size(&value.content)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => value
            .encoding
            .as_ref()
            .and_then(|encoding| data_encoding_size(&encoding.content)),
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => value
            .encoding
            .as_ref()
            .and_then(|encoding| data_encoding_size(&encoding.content)),
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(_) => None,
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => value
            .member_list
            .member
            .iter()
            .try_fold(0_u64, |total, member| {
                let member_type = set
                    .content
                    .iter()
                    .find(|candidate| parameter_type_name(candidate) == member.type_ref)?;
                total.checked_add(resolve_parameter_type_size(member_type, set, visiting)?)
            }),
    };
    visiting.remove(name);
    size
}

trait EncodingContent {
    fn fixed_encoding_size(&self) -> Option<u64>;
}

macro_rules! impl_encoding_content {
    ($type:ty) => {
        impl EncodingContent for $type {
            fn fixed_encoding_size(&self) -> Option<u64> {
                match self {
                    Self::BinaryDataEncoding(value) => binary_encoding_size(value),
                    Self::FloatDataEncoding(value) => float_encoding_size(value),
                    Self::IntegerDataEncoding(value) => positive_size(value.size_in_bits),
                    Self::StringDataEncoding(value) => string_encoding_size(value),
                    _ => None,
                }
            }
        }
    };
}

impl_encoding_content!(xtce::StringParameterTypeContent);
impl_encoding_content!(xtce::EnumeratedParameterTypeContent);
impl_encoding_content!(xtce::IntegerParameterTypeContent);
impl_encoding_content!(xtce::BinaryParameterTypeContent);
impl_encoding_content!(xtce::FloatParameterTypeContent);
impl_encoding_content!(xtce::BooleanParameterTypeContent);

fn encoding_content_size<T: EncodingContent>(content: &[T]) -> Option<u64> {
    content
        .iter()
        .find_map(EncodingContent::fixed_encoding_size)
}

fn data_encoding_size(content: &xtce::EncodingTypeContent) -> Option<u64> {
    match content {
        xtce::EncodingTypeContent::BinaryDataEncoding(value) => binary_encoding_size(value),
        xtce::EncodingTypeContent::FloatDataEncoding(value) => float_encoding_size(value),
        xtce::EncodingTypeContent::IntegerDataEncoding(value) => positive_size(value.size_in_bits),
        xtce::EncodingTypeContent::StringDataEncoding(value) => string_encoding_size(value),
    }
}

fn binary_encoding_size(encoding: &xtce::BinaryDataEncodingType) -> Option<u64> {
    match &encoding.size_in_bits {
        xtce::IntegerValueType::FixedValue(value) => positive_size(*value),
        xtce::IntegerValueType::DynamicValue(_) | xtce::IntegerValueType::DiscreteLookupList(_) => {
            None
        }
    }
}

fn float_encoding_size(encoding: &xtce::FloatDataEncodingType) -> Option<u64> {
    Some(match encoding.size_in_bits {
        xtce::FloatEncodingSizeInBitsType::_16 => 16,
        xtce::FloatEncodingSizeInBitsType::_32 => 32,
        xtce::FloatEncodingSizeInBitsType::_40 => 40,
        xtce::FloatEncodingSizeInBitsType::_48 => 48,
        xtce::FloatEncodingSizeInBitsType::_64 => 64,
        xtce::FloatEncodingSizeInBitsType::_80 => 80,
        xtce::FloatEncodingSizeInBitsType::_128 => 128,
    })
}

fn string_encoding_size(encoding: &xtce::StringDataEncodingType) -> Option<u64> {
    encoding.content.iter().find_map(|content| match content {
        xtce::StringDataEncodingTypeContent::SizeInBits(size) => {
            positive_size(size.fixed.fixed_value)
        }
        xtce::StringDataEncodingTypeContent::ErrorDetectCorrect(_)
        | xtce::StringDataEncodingTypeContent::Variable(_) => None,
    })
}

fn positive_size(size: i64) -> Option<u64> {
    u64::try_from(size).ok().filter(|size| *size > 0)
}

enum CompletionTarget {
    Container,
    Parameter,
    Entry(Entity<SelectState<Vec<EntryKind>>>),
}

struct ReferenceCompletionProvider {
    context: Rc<RefCell<ReferenceContext>>,
    target: CompletionTarget,
}

impl CompletionProvider for ReferenceCompletionProvider {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        _: CompletionContext,
        _: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        let query = text.slice(..offset).to_string();
        let normalized = query.to_ascii_lowercase();
        let context = self.context.borrow();
        let names = match &self.target {
            CompletionTarget::Container => &context.container_names,
            CompletionTarget::Parameter => &context.parameter_names,
            CompletionTarget::Entry(kind) => match selected_value(kind, EntryKind::Parameter, cx) {
                EntryKind::Parameter | EntryKind::ParameterSegment => &context.parameter_names,
                EntryKind::Container | EntryKind::ContainerSegment => &context.container_names,
                EntryKind::StreamSegment => &context.stream_names,
            },
        };
        let end = text.offset_to_position(offset);
        let items = names
            .iter()
            .filter(|name| name.to_ascii_lowercase().contains(&normalized))
            .map(|name| CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::REFERENCE),
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

struct ContainerValues<'a> {
    name: String,
    abstract_: bool,
    idle_pattern: String,
    short_description: String,
    long_description: String,
    base_container_present: bool,
    base_container_ref: String,
    restriction_criteria: String,
    restriction_kind: RestrictionCriteriaKind,
    restriction_boolean_expression: Option<&'a xtce::BooleanExpressionType>,
    restriction_custom_algorithm: Option<&'a xtce::InputAlgorithmType>,
    restriction_next_container: String,
    entry_list: Option<&'a xtce::EntryListType>,
}

fn sequence_container(
    container: Option<&xtce::ContainerSetTypeContent>,
) -> Option<&xtce::SequenceContainerType> {
    container.map(|container| match container {
        xtce::ContainerSetTypeContent::SequenceContainer(container) => container,
    })
}

impl<'a> ContainerValues<'a> {
    fn from_sequence(container: Option<&'a xtce::SequenceContainerType>) -> Self {
        let restriction = restriction_criteria_values(
            container
                .and_then(|value| value.base_container.as_ref())
                .and_then(|base| base.restriction_criteria.as_ref()),
        );
        Self {
            name: container
                .map(|value| value.name.clone())
                .unwrap_or_default(),
            abstract_: container.is_some_and(|value| value.abstract_),
            idle_pattern: container
                .map(|value| fixed_integer_text(&value.idle_pattern))
                .unwrap_or_else(|| "0".to_owned()),
            short_description: container
                .and_then(|value| value.short_description.clone())
                .unwrap_or_default(),
            long_description: container
                .and_then(|value| value.long_description.clone())
                .unwrap_or_default(),
            base_container_present: container.is_some_and(|value| value.base_container.is_some()),
            base_container_ref: container
                .and_then(|value| value.base_container.as_ref())
                .map(|base| base.container_ref.clone())
                .unwrap_or_default(),
            restriction_criteria: restriction.comparisons,
            restriction_kind: restriction.kind,
            restriction_boolean_expression: restriction.boolean_expression,
            restriction_custom_algorithm: restriction.custom_algorithm,
            restriction_next_container: restriction.next_container,
            entry_list: container.map(|value| &value.entry_list),
        }
    }
}

struct RestrictionCriteriaValues<'a> {
    comparisons: String,
    kind: RestrictionCriteriaKind,
    boolean_expression: Option<&'a xtce::BooleanExpressionType>,
    custom_algorithm: Option<&'a xtce::InputAlgorithmType>,
    next_container: String,
}

fn restriction_criteria_values(
    criteria: Option<&xtce::RestrictionCriteriaType>,
) -> RestrictionCriteriaValues<'_> {
    match criteria.and_then(|criteria| criteria.content.as_ref()) {
        None => RestrictionCriteriaValues {
            comparisons: String::new(),
            kind: RestrictionCriteriaKind::ComparisonList,
            boolean_expression: None,
            custom_algorithm: None,
            next_container: String::new(),
        },
        Some(xtce::RestrictionCriteriaTypeContent::Comparison(comparison)) => {
            RestrictionCriteriaValues {
                comparisons: encode_comparison(comparison),
                kind: RestrictionCriteriaKind::ComparisonList,
                boolean_expression: None,
                custom_algorithm: None,
                next_container: String::new(),
            }
        }
        Some(xtce::RestrictionCriteriaTypeContent::ComparisonList(list)) => {
            RestrictionCriteriaValues {
                comparisons: list
                    .comparison
                    .iter()
                    .map(encode_comparison)
                    .collect::<Vec<_>>()
                    .join("\n"),
                kind: RestrictionCriteriaKind::ComparisonList,
                boolean_expression: None,
                custom_algorithm: None,
                next_container: String::new(),
            }
        }
        Some(xtce::RestrictionCriteriaTypeContent::BooleanExpression(expression)) => {
            RestrictionCriteriaValues {
                comparisons: String::new(),
                kind: RestrictionCriteriaKind::BooleanExpression,
                boolean_expression: Some(expression),
                custom_algorithm: None,
                next_container: String::new(),
            }
        }
        Some(xtce::RestrictionCriteriaTypeContent::CustomAlgorithm(algorithm)) => {
            RestrictionCriteriaValues {
                comparisons: String::new(),
                kind: RestrictionCriteriaKind::CustomAlgorithm,
                boolean_expression: None,
                custom_algorithm: Some(algorithm),
                next_container: String::new(),
            }
        }
        Some(xtce::RestrictionCriteriaTypeContent::NextContainer(container)) => {
            RestrictionCriteriaValues {
                comparisons: String::new(),
                kind: RestrictionCriteriaKind::NextContainer,
                boolean_expression: None,
                custom_algorithm: None,
                next_container: container.container_ref.clone(),
            }
        }
    }
}

fn encode_comparison(comparison: &xtce::ComparisonType) -> String {
    format!(
        "{} | {} | {} | {} | {}",
        comparison.parameter_ref,
        comparison.comparison_operator,
        comparison.value,
        comparison.instance,
        comparison.use_calibrated_value
    )
}

struct ComparisonRowModel {
    parameter_ref: String,
    operator: String,
    comparison_value: String,
    instance: String,
    calibrated: bool,
}

impl Default for ComparisonRowModel {
    fn default() -> Self {
        Self {
            parameter_ref: String::new(),
            operator: "==".to_owned(),
            comparison_value: String::new(),
            instance: "0".to_owned(),
            calibrated: true,
        }
    }
}

fn comparison_models(value: &str) -> Vec<ComparisonRowModel> {
    value
        .lines()
        .filter_map(|line| {
            let fields = line.split('|').map(str::trim).collect::<Vec<_>>();
            Some(ComparisonRowModel {
                parameter_ref: fields.first()?.to_string(),
                operator: fields.get(1).copied().unwrap_or("==").to_owned(),
                comparison_value: fields.get(2).copied().unwrap_or_default().to_owned(),
                instance: fields.get(3).copied().unwrap_or("0").to_owned(),
                calibrated: fields
                    .get(4)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(true),
            })
        })
        .collect()
}

fn comparison_row(
    model: ComparisonRowModel,
    context: Rc<RefCell<ReferenceContext>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ComparisonRowForm> {
    cx.new(|cx| ComparisonRowForm {
        parameter_ref: completion_input(
            &model.parameter_ref,
            CompletionTarget::Parameter,
            context,
            window,
            cx,
        ),
        operator: select(
            ComparisonOperator::VARIANTS,
            model.operator.parse().unwrap_or(ComparisonOperator::Equal),
            window,
            cx,
        ),
        comparison_value: input(&model.comparison_value, false, window, cx),
        instance: input(&model.instance, false, window, cx),
        calibrated: select(
            CalibratedChoice::VARIANTS,
            if model.calibrated {
                CalibratedChoice::CalibratedValue
            } else {
                CalibratedChoice::RawValue
            },
            window,
            cx,
        ),
    })
}

#[cfg(test)]
fn decode_restriction_criteria(value: &str) -> Option<xtce::RestrictionCriteriaType> {
    let comparison = value
        .lines()
        .filter_map(|line| {
            let fields = line.split('|').map(str::trim).collect::<Vec<_>>();
            let parameter_ref = fields.first()?.to_string();
            let comparison_operator = fields.get(1)?.to_string();
            let comparison_value = fields.get(2)?.to_string();
            if parameter_ref.is_empty()
                || comparison_operator.is_empty()
                || comparison_value.is_empty()
            {
                return None;
            }
            Some(xtce::ComparisonType {
                parameter_ref,
                comparison_operator,
                value: comparison_value,
                instance: fields
                    .get(3)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(xtce::ComparisonType::default_instance),
                use_calibrated_value: fields
                    .get(4)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(xtce::ComparisonType::default_use_calibrated_value),
            })
        })
        .collect::<Vec<_>>();
    (!comparison.is_empty()).then_some(xtce::RestrictionCriteriaType {
        content: Some(xtce::RestrictionCriteriaTypeContent::ComparisonList(
            xtce::ComparisonListType { comparison },
        )),
    })
}

fn fixed_integer_text(value: &xtce::FixedIntegerValueType) -> String {
    match value {
        xtce::FixedIntegerValueType::BigInt(value) => value.to_string(),
        xtce::FixedIntegerValueType::String(value) => value.clone(),
    }
}

fn fixed_integer_value(value: &str) -> xtce::FixedIntegerValueType {
    let value = value.trim();
    if value.is_empty() {
        return xtce::SequenceContainerType::default_idle_pattern();
    }
    value.parse().map_or_else(
        |_| xtce::FixedIntegerValueType::String(value.to_owned()),
        xtce::FixedIntegerValueType::BigInt,
    )
}

fn completion_input(
    value: &str,
    target: CompletionTarget,
    context: Rc<RefCell<ReferenceContext>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let mut input = InputState::new(window, cx).default_value(value.to_owned());
        input.lsp.completion_provider =
            Some(Rc::new(ReferenceCompletionProvider { context, target }));
        input
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
        .position(|candidate| candidate == &selected)
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

fn touch_cache(cache_order: &mut VecDeque<usize>, index: usize) {
    if let Some(position) = cache_order.iter().position(|candidate| *candidate == index) {
        cache_order.remove(position);
    }
    cache_order.push_back(index);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        ContainerLayoutSource, EntryKind, EntryRowContent, EntryRowData, PacketField,
        RestrictionCriteriaKind, apply_entry_rows, binary_encoding_size,
        decode_restriction_criteria, entry_bit_positions, fixed_integer_value, float_encoding_size,
        packet_layout, parameter_type_sizes, restriction_criteria_values, rows_from_entry_list,
        string_encoding_size,
    };

    #[test]
    fn fixed_data_encoding_sizes_are_resolved_for_binary_float_and_string() {
        let binary = xtce::BinaryDataEncodingType {
            bit_order: xtce::BitOrderType::MostSignificantBitFirst,
            byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
            error_detect_correct: None,
            size_in_bits: xtce::IntegerValueType::FixedValue(24),
            from_binary_transform_algorithm: None,
            to_binary_transform_algorithm: None,
        };
        let float = xtce::FloatDataEncodingType {
            bit_order: xtce::BitOrderType::MostSignificantBitFirst,
            byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
            encoding: xtce::FloatEncodingType::Ieee7541985,
            size_in_bits: xtce::FloatEncodingSizeInBitsType::_48,
            change_threshold: None,
            error_detect_correct: None,
            default_calibrator: None,
            context_calibrator_list: None,
        };
        let string = xtce::StringDataEncodingType {
            bit_order: xtce::BitOrderType::MostSignificantBitFirst,
            byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
            encoding: xtce::StringEncodingType::Utf8,
            content: vec![xtce::StringDataEncodingTypeContent::SizeInBits(
                xtce::SizeInBitsType {
                    fixed: xtce::SizeInBitsTypeFixedElementType { fixed_value: 80 },
                    termination_char: None,
                    leading_size: None,
                },
            )],
        };

        assert_eq!(binary_encoding_size(&binary), Some(24));
        assert_eq!(float_encoding_size(&float), Some(48));
        assert_eq!(string_encoding_size(&string), Some(80));
    }

    #[test]
    fn packet_sizes_come_from_data_encoding_not_parameter_type_size() {
        let set = xtce::ParameterTypeSetType {
            content: vec![
                xtce::ParameterTypeSetTypeContent::IntegerParameterType(
                    xtce::IntegerParameterType {
                        short_description: None,
                        name: "EncodedInteger".to_owned(),
                        base_type: None,
                        initial_value: None,
                        size_in_bits: 64,
                        signed: true,
                        content: vec![xtce::IntegerParameterTypeContent::IntegerDataEncoding(
                            xtce::IntegerDataEncodingType {
                                bit_order: xtce::BitOrderType::MostSignificantBitFirst,
                                byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
                                encoding: xtce::IntegerEncodingType::Unsigned,
                                size_in_bits: 12,
                                change_threshold: None,
                                error_detect_correct: None,
                                default_calibrator: None,
                                context_calibrator_list: None,
                            },
                        )],
                    },
                ),
                xtce::ParameterTypeSetTypeContent::IntegerParameterType(
                    xtce::IntegerParameterType {
                        short_description: None,
                        name: "NoEncoding".to_owned(),
                        base_type: None,
                        initial_value: None,
                        size_in_bits: 32,
                        signed: true,
                        content: Vec::new(),
                    },
                ),
            ],
        };

        let sizes = parameter_type_sizes(Some(&set));

        assert_eq!(sizes.get("EncodedInteger"), Some(&12));
        assert!(!sizes.contains_key("NoEncoding"));
    }

    #[test]
    fn aggregate_packet_size_is_the_sum_of_member_encodings() {
        let integer_type = |name: &str, size_in_bits| {
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: name.to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 64,
                signed: true,
                content: vec![xtce::IntegerParameterTypeContent::IntegerDataEncoding(
                    xtce::IntegerDataEncodingType {
                        bit_order: xtce::BitOrderType::MostSignificantBitFirst,
                        byte_order: xtce::ByteOrderType::MostSignificantByteFirst,
                        encoding: xtce::IntegerEncodingType::Unsigned,
                        size_in_bits,
                        change_threshold: None,
                        error_detect_correct: None,
                        default_calibrator: None,
                        context_calibrator_list: None,
                    },
                )],
            })
        };
        let member = |name: &str, type_ref: &str| xtce::MemberType {
            short_description: None,
            name: name.to_owned(),
            type_ref: type_ref.to_owned(),
            initial_value: None,
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
        };
        let set = xtce::ParameterTypeSetType {
            content: vec![
                integer_type("HeaderType", 8),
                integer_type("PayloadType", 16),
                xtce::ParameterTypeSetTypeContent::AggregateParameterType(
                    xtce::AggregateParameterType {
                        short_description: None,
                        name: "PacketType".to_owned(),
                        initial_value: None,
                        long_description: None,
                        alias_set: None,
                        ancillary_data_set: None,
                        member_list: xtce::MemberListType {
                            member: vec![
                                member("header", "HeaderType"),
                                member("payload", "PayloadType"),
                            ],
                        },
                    },
                ),
            ],
        };

        assert_eq!(
            parameter_type_sizes(Some(&set)).get("PacketType"),
            Some(&24)
        );
    }

    #[test]
    fn packet_layout_places_fixed_size_parameters_in_contiguous_fields() {
        let rows = vec![
            EntryRowData::new_parameter_reference("apid".to_owned()),
            EntryRowData::new_parameter_reference("temperature".to_owned()),
        ];
        let sizes = HashMap::from([("apid".to_owned(), 16), ("temperature".to_owned(), 8)]);

        let layout = packet_layout(&rows, &sizes, "", &HashMap::new(), "CurrentPacket");

        assert_eq!(
            layout.fields,
            vec![
                PacketField {
                    label: "apid".to_owned(),
                    start_bit: 0,
                    size_bits: 16,
                    inherited: false,
                    source: "CurrentPacket".to_owned(),
                },
                PacketField {
                    label: "temperature".to_owned(),
                    start_bit: 16,
                    size_bits: 8,
                    inherited: false,
                    source: "CurrentPacket".to_owned(),
                },
            ]
        );
        assert_eq!(layout.total_bits, 24);
        assert!(layout.unresolved.is_empty());
    }

    #[test]
    fn entry_list_positions_use_packet_bit_offsets_instead_of_row_numbers() {
        let mut payload = EntryRowData::new_parameter_reference("payload".to_owned());
        let EntryRowContent::Editable { offset, .. } = &mut payload.content else {
            unreachable!()
        };
        *offset = "4".to_owned();
        let rows = vec![
            EntryRowData::new_parameter_reference("header".to_owned()),
            payload,
        ];
        let sizes = HashMap::from([("header".to_owned(), 12), ("payload".to_owned(), 8)]);

        assert_eq!(
            entry_bit_positions(&rows, &sizes, "", &HashMap::new()),
            vec![Some(0), Some(16)]
        );
    }

    #[test]
    fn packet_layout_reports_parameters_with_unknown_sizes() {
        let rows = vec![EntryRowData::new_parameter_reference("payload".to_owned())];

        let layout = packet_layout(&rows, &HashMap::new(), "", &HashMap::new(), "Packet");

        assert!(layout.fields.is_empty());
        assert_eq!(layout.unresolved, vec!["payload: size is unknown"]);
    }

    #[test]
    fn packet_layout_prepends_base_container_fields() {
        let rows = vec![EntryRowData::new_parameter_reference("payload".to_owned())];
        let sizes = HashMap::from([("header".to_owned(), 16), ("payload".to_owned(), 8)]);
        let containers = HashMap::from([(
            "BasePacket".to_owned(),
            ContainerLayoutSource {
                base_container_ref: None,
                rows: vec![EntryRowData::new_parameter_reference("header".to_owned())],
            },
        )]);

        let layout = packet_layout(&rows, &sizes, "BasePacket", &containers, "CurrentPacket");

        assert_eq!(layout.fields[0].label, "header");
        assert_eq!(layout.fields[0].start_bit, 0);
        assert!(layout.fields[0].inherited);
        assert_eq!(layout.fields[0].source, "BasePacket");
        assert_eq!(layout.fields[1].label, "payload");
        assert_eq!(layout.fields[1].start_bit, 16);
        assert!(!layout.fields[1].inherited);
        assert_eq!(layout.fields[1].source, "CurrentPacket");
    }

    #[test]
    fn dropped_parameter_becomes_a_parameter_reference_row() {
        let row = EntryRowData::new_parameter_reference("temperature".to_owned());

        assert!(matches!(
            row.content,
            EntryRowContent::Editable {
                kind: EntryKind::Parameter,
                reference,
                ..
            } if reference == "temperature"
        ));
    }

    #[test]
    fn parameter_segment_entry_is_loaded_as_an_editable_row() {
        let list = xtce::EntryListType {
            content: vec![xtce::EntryListTypeContent::ParameterSegmentRefEntry(
                xtce::ParameterSegmentRefEntryType {
                    short_description: Some("first byte".to_owned()),
                    parameter_ref: "payload".to_owned(),
                    order: Some(2),
                    size_in_bits: 8,
                    location_in_container_in_bits: None,
                    repeat_entry: None,
                    include_condition: None,
                    time_association: None,
                    ancillary_data_set: None,
                },
            )],
        };

        let rows = rows_from_entry_list(Some(&list));

        assert!(matches!(
            &rows[0].content,
            EntryRowContent::Editable {
                kind: EntryKind::ParameterSegment,
                reference,
                segment_size,
                segment_order,
                ..
            } if reference == "payload" && segment_size == "8" && segment_order == "2"
        ));
    }

    #[test]
    fn container_segment_entry_round_trips_as_an_editable_row() {
        let mut list = xtce::EntryListType {
            content: vec![xtce::EntryListTypeContent::ContainerSegmentRefEntry(
                xtce::ContainerSegmentRefEntryType {
                    short_description: Some("second fragment".to_owned()),
                    container_ref: "Payload".to_owned(),
                    order: Some(1),
                    size_in_bits: 64,
                    location_in_container_in_bits: None,
                    repeat_entry: None,
                    include_condition: None,
                    time_association: None,
                    ancillary_data_set: None,
                },
            )],
        };

        let rows = rows_from_entry_list(Some(&list));
        assert!(matches!(
            &rows[0].content,
            EntryRowContent::Editable {
                kind: EntryKind::ContainerSegment,
                reference,
                segment_size,
                segment_order,
                ..
            } if reference == "Payload" && segment_size == "64" && segment_order == "1"
        ));

        apply_entry_rows(&mut list, rows);
        assert!(matches!(
            &list.content[0],
            xtce::EntryListTypeContent::ContainerSegmentRefEntry(entry)
                if entry.container_ref == "Payload"
                    && entry.size_in_bits == 64
                    && entry.order == Some(1)
        ));
    }

    #[test]
    fn blank_idle_pattern_uses_the_schema_default() {
        assert_eq!(super::fixed_integer_text(&fixed_integer_value("")), "0");
        assert_eq!(super::fixed_integer_text(&fixed_integer_value("   ")), "0");
        assert_eq!(super::fixed_integer_text(&fixed_integer_value("1")), "1");
    }

    #[test]
    fn comparison_restrictions_round_trip_through_the_compact_editor() {
        let criteria = decode_restriction_criteria("TLM_ID | == | 2 | 0 | true").expect("criteria");
        let values = restriction_criteria_values(Some(&criteria));

        assert_eq!(values.comparisons, "TLM_ID | == | 2 | 0 | true");
        assert_eq!(values.kind, RestrictionCriteriaKind::ComparisonList);
    }

    #[test]
    fn boolean_expression_restriction_is_editable() {
        let criteria = xtce::RestrictionCriteriaType {
            content: Some(xtce::RestrictionCriteriaTypeContent::BooleanExpression(
                xtce::BooleanExpressionType::Condition(xtce::ComparisonCheckType {
                    content: vec![
                        xtce::ComparisonCheckTypeContent::ParameterInstanceRef(
                            xtce::ParameterInstanceRefType {
                                parameter_ref: "P1".to_owned(),
                                instance: 0,
                                use_calibrated_value: true,
                            },
                        ),
                        xtce::ComparisonCheckTypeContent::ComparisonOperator("==".to_owned()),
                        xtce::ComparisonCheckTypeContent::Value("1".to_owned()),
                    ],
                }),
            )),
        };
        let values = restriction_criteria_values(Some(&criteria));

        assert_eq!(values.kind, RestrictionCriteriaKind::BooleanExpression);
        assert!(values.boolean_expression.is_some());
    }

    #[test]
    fn custom_algorithm_restriction_is_editable() {
        let criteria = xtce::RestrictionCriteriaType {
            content: Some(xtce::RestrictionCriteriaTypeContent::CustomAlgorithm(
                xtce::InputAlgorithmType {
                    short_description: None,
                    name: "containerFilter".to_owned(),
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                    algorithm_text: None,
                    external_algorithm_set: None,
                    input_set: None,
                },
            )),
        };
        let values = restriction_criteria_values(Some(&criteria));

        assert_eq!(values.kind, RestrictionCriteriaKind::CustomAlgorithm);
        assert_eq!(
            values
                .custom_algorithm
                .map(|algorithm| algorithm.name.as_str()),
            Some("containerFilter")
        );
    }

    #[test]
    fn next_container_restriction_is_editable() {
        let criteria = xtce::RestrictionCriteriaType {
            content: Some(xtce::RestrictionCriteriaTypeContent::NextContainer(
                xtce::ContainerRefType {
                    container_ref: "PacketB".to_owned(),
                },
            )),
        };
        let values = restriction_criteria_values(Some(&criteria));

        assert_eq!(values.kind, RestrictionCriteriaKind::NextContainer);
        assert_eq!(values.next_container, "PacketB");
    }

    #[test]
    fn stream_segment_entry_is_editable_and_reorderable() {
        let mut list = xtce::EntryListType {
            content: vec![
                xtce::EntryListTypeContent::ParameterRefEntry(super::default_parameter_ref_entry()),
                xtce::EntryListTypeContent::StreamSegmentEntry(xtce::StreamSegmentEntryType {
                    short_description: None,
                    stream_ref: "stream".to_owned(),
                    order: None,
                    size_in_bits: 8,
                    location_in_container_in_bits: None,
                    repeat_entry: None,
                    include_condition: None,
                    time_association: None,
                    ancillary_data_set: None,
                }),
            ],
        };
        let mut rows = rows_from_entry_list(Some(&list));
        assert!(matches!(
            &rows[1].content,
            EntryRowContent::Editable {
                kind: EntryKind::StreamSegment,
                reference,
                segment_size,
                ..
            } if reference == "stream" && segment_size == "8"
        ));
        rows.swap(0, 1);

        apply_entry_rows(&mut list, rows);

        assert!(matches!(
            list.content.first(),
            Some(xtce::EntryListTypeContent::StreamSegmentEntry(_))
        ));
        assert!(matches!(
            list.content.get(1),
            Some(xtce::EntryListTypeContent::ParameterRefEntry(_))
        ));
    }

    #[test]
    fn a_new_row_becomes_a_parameter_reference() {
        let mut list = xtce::EntryListType {
            content: Vec::new(),
        };

        apply_entry_rows(
            &mut list,
            vec![super::EntryRowData::new_editable(EntryKind::Parameter)],
        );

        assert!(matches!(
            list.content.first(),
            Some(xtce::EntryListTypeContent::ParameterRefEntry(_))
        ));
    }
}
