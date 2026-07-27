use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
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
    v_flex,
};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    Position, Range, TextEdit,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm,
    container_binary_encoding::ContainerBinaryEncodingForm, container_rate::ContainerRateForm,
    field, impl_select_item, optional_value,
};
use crate::XtceEditor;

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum AbstractChoice {
    #[strum(serialize = "false")]
    Concrete,
    #[strum(serialize = "true")]
    Abstract,
}
impl_select_item!(AbstractChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum EntryKind {
    #[strum(serialize = "ParameterRefEntry")]
    ParameterReference,
    #[strum(serialize = "ContainerRefEntry")]
    ContainerReference,
}
impl_select_item!(EntryKind);

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
    restriction_criteria_input: Entity<InputState>,
    restriction_criteria_editable: bool,
    reference_context: Rc<RefCell<ReferenceContext>>,
    entry_list: Entity<TelemetryEntryListView>,
    _subscriptions: Vec<Subscription>,
}

impl SequenceContainerForm {
    pub(super) fn new(
        container: Option<&xtce::ContainerSetTypeContent>,
        parameter_set: Option<&xtce::ParameterSetType>,
        container_set: Option<&xtce::ContainerSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = ContainerValues::from_container(container);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        let reference_context = Rc::new(RefCell::new(ReferenceContext::new(
            parameter_set,
            container_set,
        )));
        let sequence = sequence_container(container);
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

        cx.new(move |cx| {
            let base_container_ref_input = completion_input(
                &values.base_container_ref,
                CompletionTarget::Container,
                reference_context.clone(),
                window,
                cx,
            );
            let entry_list = cx.new(|cx| {
                TelemetryEntryListView::new(
                    rows_from_entry_list(values.entry_list),
                    reference_context.clone(),
                    window,
                    cx,
                )
            });
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
                restriction_criteria_input: input(&values.restriction_criteria, true, window, cx),
                restriction_criteria_editable: values.restriction_criteria_editable,
                reference_context,
                entry_list,
                _subscriptions: vec![name_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        container: Option<&xtce::ContainerSetTypeContent>,
        parameter_set: Option<&xtce::ParameterSetType>,
        container_set: Option<&xtce::ContainerSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ContainerValues::from_container(container);
        self.advanced_settings_open = false;
        self.documentation_open = false;
        self.metadata_open = false;
        self.stream_rate_open = false;
        self.restriction_open = false;
        let sequence = sequence_container(container);
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
        *self.reference_context.borrow_mut() = ReferenceContext::new(parameter_set, container_set);
        self.base_container_present
            .set(values.base_container_present);
        self.restriction_criteria_editable = values.restriction_criteria_editable;
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
                &self.restriction_criteria_input,
                values.restriction_criteria,
            ),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.entry_list.update(cx, |list, cx| {
            list.set_rows(rows_from_entry_list(values.entry_list), cx);
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
            if self.restriction_criteria_editable {
                base.restriction_criteria =
                    decode_restriction_criteria(&value(&self.restriction_criteria_input, cx));
            }
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
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.restriction_open = !this.restriction_open;
                                            cx.notify();
                                        })),
                                    )
                                    .content(if self.restriction_criteria_editable {
                                        v_flex()
                                            .pt_3()
                                            .child(field(
                                                "Comparisons",
                                                "parameter reference | operator | value | instance | calibrated",
                                                &self.restriction_criteria_input,
                                                cx,
                                            ))
                                            .into_any_element()
                                    } else {
                                        div()
                                            .pt_3()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(
                                                "This restriction type is preserved without modification",
                                            )
                                            .into_any_element()
                                    }),
                            )
                    }),
            )
            .child(self.entry_list.clone())
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
    optional_columns_visible: Rc<Cell<bool>>,
    list_state: ListState,
}

struct TelemetryEntryRow {
    kind_select: Entity<SelectState<Vec<EntryKind>>>,
    reference_input: Entity<InputState>,
    offset_input: Entity<InputState>,
    description_input: Entity<InputState>,
    source_index: Rc<Cell<Option<usize>>>,
    preserve_complex_location: Rc<Cell<bool>>,
    optional_columns_visible: Rc<Cell<bool>>,
    _subscriptions: Vec<Subscription>,
}

impl TelemetryEntryListView {
    fn new(
        rows: Vec<EntryRowData>,
        context: Rc<RefCell<ReferenceContext>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        let optional_columns_visible = Rc::new(Cell::new(false));
        Self {
            list_state: ListState::new(rows.len(), ListAlignment::Top, px(58.))
                .with_uniform_item_height(px(54.)),
            rows,
            editors: HashMap::new(),
            cache_order: VecDeque::new(),
            context,
            optional_columns_visible,
        }
    }

    fn set_rows(&mut self, rows: Vec<EntryRowData>, cx: &mut Context<Self>) {
        self.rows = rows;
        self.editors.clear();
        self.cache_order.clear();
        self.optional_columns_visible.set(false);
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
                div()
                    .w(px(44.))
                    .flex_none()
                    .text_sm()
                    .child((index + 1).to_string()),
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
        v_flex()
            .w_full()
            .gap_3()
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
                                        this.rows.push(EntryRowData::new_editable(
                                            EntryKind::ParameterReference,
                                        ));
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
                                        px(870.)
                                    } else {
                                        px(610.)
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
                                            .child(div().w(px(44.)).child("#"))
                                            .child(div().w(px(98.)).child("Actions"))
                                            .child(div().w(px(170.)).child("Type"))
                                            .child(div().flex_1().child("Reference"))
                                            .when(optional_columns_visible, |header| {
                                                header
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
    }
}

impl Render for TelemetryEntryRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let optional_columns_visible = self.optional_columns_visible.get();
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
                row.child(
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
                offset: String::new(),
                description: String::new(),
            },
        }
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
            let mut input = InputState::new(window, cx).default_value(reference);
            input.lsp.completion_provider = Some(Rc::new(ReferenceCompletionProvider {
                context,
                target: CompletionTarget::Entry(kind_select.clone()),
            }));
            input
        });
        TelemetryEntryRow {
            kind_select,
            reference_input,
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
            kind: selected_value(&row.kind_select, EntryKind::ParameterReference, cx),
            reference: value(&row.reference_input, cx),
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
                        kind: EntryKind::ParameterReference,
                        reference: entry.parameter_ref.clone(),
                        offset: fixed_offset(entry.location_in_container_in_bits.as_ref()),
                        description: entry.short_description.clone().unwrap_or_default(),
                    },
                    has_complex_location(entry.location_in_container_in_bits.as_ref()),
                ),
                xtce::EntryListTypeContent::ContainerRefEntry(entry) => (
                    EntryRowContent::Editable {
                        kind: EntryKind::ContainerReference,
                        reference: entry.container_ref.clone(),
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
                offset,
                description,
            } => match kind {
                EntryKind::ParameterReference => {
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
                EntryKind::ContainerReference => {
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
    container_names: Vec<String>,
}

impl ReferenceContext {
    fn new(
        parameter_set: Option<&xtce::ParameterSetType>,
        container_set: Option<&xtce::ContainerSetType>,
    ) -> Self {
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
            container_names: container_set
                .into_iter()
                .flat_map(|set| &set.content)
                .map(|container| match container {
                    xtce::ContainerSetTypeContent::SequenceContainer(container) => {
                        container.name.clone()
                    }
                })
                .collect(),
        }
    }
}

enum CompletionTarget {
    Container,
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
            CompletionTarget::Entry(kind) => {
                if selected_value(kind, EntryKind::ParameterReference, cx)
                    == EntryKind::ParameterReference
                {
                    &context.parameter_names
                } else {
                    &context.container_names
                }
            }
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
    restriction_criteria_editable: bool,
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
    fn from_container(container: Option<&'a xtce::ContainerSetTypeContent>) -> Self {
        let container = container.map(|container| match container {
            xtce::ContainerSetTypeContent::SequenceContainer(container) => container,
        });
        let (restriction_criteria, restriction_criteria_editable) = restriction_criteria_values(
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
            restriction_criteria,
            restriction_criteria_editable,
            entry_list: container.map(|value| &value.entry_list),
        }
    }
}

fn restriction_criteria_values(criteria: Option<&xtce::RestrictionCriteriaType>) -> (String, bool) {
    match criteria.and_then(|criteria| criteria.content.as_ref()) {
        None => (String::new(), true),
        Some(xtce::RestrictionCriteriaTypeContent::Comparison(comparison)) => {
            (encode_comparison(comparison), true)
        }
        Some(xtce::RestrictionCriteriaTypeContent::ComparisonList(list)) => (
            list.comparison
                .iter()
                .map(encode_comparison)
                .collect::<Vec<_>>()
                .join("\n"),
            true,
        ),
        Some(_) => (String::new(), false),
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
    use super::{
        EntryKind, apply_entry_rows, decode_restriction_criteria, fixed_integer_value,
        restriction_criteria_values, rows_from_entry_list,
    };

    #[test]
    fn blank_idle_pattern_uses_the_schema_default() {
        assert_eq!(super::fixed_integer_text(&fixed_integer_value("")), "0");
        assert_eq!(super::fixed_integer_text(&fixed_integer_value("   ")), "0");
        assert_eq!(super::fixed_integer_text(&fixed_integer_value("1")), "1");
    }

    #[test]
    fn comparison_restrictions_round_trip_through_the_compact_editor() {
        let criteria = decode_restriction_criteria("TLM_ID | == | 2 | 0 | true").expect("criteria");
        let (encoded, editable) = restriction_criteria_values(Some(&criteria));

        assert!(editable);
        assert_eq!(encoded, "TLM_ID | == | 2 | 0 | true");
    }

    #[test]
    fn reordering_keeps_unsupported_entries_in_place() {
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
            vec![super::EntryRowData::new_editable(
                EntryKind::ParameterReference,
            )],
        );

        assert!(matches!(
            list.content.first(),
            Some(xtce::EntryListTypeContent::ParameterRefEntry(_))
        ));
    }
}
