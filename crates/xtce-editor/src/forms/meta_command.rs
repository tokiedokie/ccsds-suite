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

use super::{field, impl_select_item, optional_value};
use crate::XtceEditor;

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum MetaCommandKind {
    MetaCommand,
    MetaCommandRef,
    BlockMetaCommand,
}
impl_select_item!(MetaCommandKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum AbstractChoice {
    #[strum(serialize = "false")]
    Concrete,
    #[strum(serialize = "true")]
    Abstract,
}
impl_select_item!(AbstractChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CommandContainerEntryKind {
    #[strum(serialize = "ArgumentRefEntry")]
    ArgumentRef,
    #[strum(serialize = "ParameterRefEntry")]
    ParameterRef,
    #[strum(serialize = "ContainerRefEntry")]
    ContainerRef,
    #[strum(serialize = "FixedValueEntry")]
    FixedValue,
}
impl_select_item!(CommandContainerEntryKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ConsequenceLevelChoice {
    #[strum(serialize = "None")]
    None,
    #[strum(serialize = "normal")]
    Normal,
    #[strum(serialize = "vital")]
    Vital,
    #[strum(serialize = "critical")]
    Critical,
    #[strum(serialize = "forbidden")]
    Forbidden,
    #[strum(serialize = "user1")]
    User1,
    #[strum(serialize = "user2")]
    User2,
}
impl_select_item!(ConsequenceLevelChoice);

pub(super) struct MetaCommandForm {
    kind_select: Entity<SelectState<Vec<MetaCommandKind>>>,
    abstract_select: Entity<SelectState<Vec<AbstractChoice>>>,
    name_or_ref_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    system_name_input: Entity<InputState>,
    base_meta_command_ref_input: Entity<InputState>,
    base_assignment_list: Entity<BaseAssignmentListView>,
    assignment_context: Rc<RefCell<AssignmentContext>>,
    argument_list: Entity<ArgumentListView>,
    block_steps_input: Entity<InputState>,
    command_container_present: Rc<Cell<bool>>,
    container_name_input: Entity<InputState>,
    container_short_description_input: Entity<InputState>,
    container_long_description_input: Entity<InputState>,
    container_base_ref_input: Entity<InputState>,
    entry_list: Entity<EntryListView>,
    consequence_level_select: Entity<SelectState<Vec<ConsequenceLevelChoice>>>,
    reason_for_warning_input: Entity<InputState>,
    space_system_at_risk_input: Entity<InputState>,
    transmission_constraints: Entity<TransmissionConstraintListForm>,
    execution_verifiers: Entity<VerifierListForm>,
    complete_verifiers: Entity<VerifierListForm>,
    documentation_open: bool,
    inheritance_open: bool,
    identification_open: bool,
    significance_open: bool,
    transmission_constraints_open: bool,
    verifiers_open: bool,
    container_details_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl MetaCommandForm {
    pub(super) fn new(
        command: Option<&xtce::MetaCommandSetTypeContent>,
        meta_command_set: Option<&xtce::MetaCommandSetType>,
        argument_type_set: Option<&xtce::ArgumentTypeSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = MetaCommandValues::from_command(command);
        let name_or_ref_input = input(&values.name_or_ref, false, window, cx);
        // Name changes intentionally continue to invalidate XtceEditor directly so
        // the title and tree labels are updated by the existing mechanism.
        let name_subscription =
            cx.subscribe(&name_or_ref_input, |editor, _, _: &InputEvent, cx| {
                editor.refresh_tree(cx);
                cx.notify();
            });

        cx.new(move |cx| {
            let kind_select = select(
                MetaCommandKind::VARIANTS,
                command.map(kind).unwrap_or(MetaCommandKind::MetaCommand),
                window,
                cx,
            );
            let kind_subscription = cx.subscribe(
                &kind_select,
                |_, _, _: &SelectEvent<Vec<MetaCommandKind>>, cx| cx.notify(),
            );
            let base_meta_command_ref_input =
                input(&values.base_meta_command_ref, false, window, cx);
            let assignment_context = Rc::new(RefCell::new(AssignmentContext::new(
                &values.base_meta_command_ref,
                meta_command_set,
                argument_type_set,
            )));
            let base_assignment_list = cx.new(|_| BaseAssignmentListView {
                rows: base_assignment_models(&values.base_assignments),
                editors: HashMap::new(),
                cache_order: VecDeque::new(),
                context: assignment_context.clone(),
                list_state: ListState::new(
                    decode_assignments(&values.base_assignments).len(),
                    ListAlignment::Top,
                    px(54.),
                )
                .with_uniform_item_height(px(50.)),
            });
            let argument_list = cx.new(|_| ArgumentListView {
                rows: command_argument_models(&values.arguments),
                editors: HashMap::new(),
                context: assignment_context.clone(),
                optional_fields_open: false,
            });
            let command_container_present = Rc::new(Cell::new(values.command_container.present));
            let subscription_context = assignment_context.clone();
            let base_ref_subscription = cx.subscribe(
                &base_meta_command_ref_input,
                move |_, input, _: &InputEvent, cx| {
                    subscription_context.borrow_mut().current_base =
                        input.read(cx).value().to_string();
                    cx.notify();
                },
            );
            let entry_list = cx.new(|cx| {
                EntryListView::new(
                    decode_container_entries(&values.command_container.entries),
                    argument_list.clone(),
                    name_or_ref_input.clone(),
                    window,
                    cx,
                )
            });

            Self {
                kind_select,
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
                name_or_ref_input,
                short_description_input: input(&values.short_description, false, window, cx),
                long_description_input: input(&values.long_description, true, window, cx),
                system_name_input: input(&values.system_name, false, window, cx),
                base_meta_command_ref_input,
                base_assignment_list,
                assignment_context,
                argument_list,
                block_steps_input: input(&values.block_steps, true, window, cx),
                command_container_present,
                container_name_input: input(&values.command_container.name, false, window, cx),
                container_short_description_input: input(
                    &values.command_container.short_description,
                    false,
                    window,
                    cx,
                ),
                container_long_description_input: input(
                    &values.command_container.long_description,
                    true,
                    window,
                    cx,
                ),
                container_base_ref_input: input(
                    &values.command_container.base_ref,
                    false,
                    window,
                    cx,
                ),
                entry_list,
                consequence_level_select: select(
                    ConsequenceLevelChoice::VARIANTS,
                    values.default_significance.consequence_level,
                    window,
                    cx,
                ),
                reason_for_warning_input: input(
                    &values.default_significance.reason_for_warning,
                    false,
                    window,
                    cx,
                ),
                space_system_at_risk_input: input(
                    &values.default_significance.space_system_at_risk,
                    false,
                    window,
                    cx,
                ),
                transmission_constraints: TransmissionConstraintListForm::new(
                    command.and_then(|cmd| match cmd {
                        xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                            cmd.transmission_constraint_list.as_ref()
                        }
                        _ => None,
                    }),
                    window,
                    cx,
                ),
                execution_verifiers: VerifierListForm::new(
                    "Execution verifiers",
                    "Add execution verifier",
                    command
                        .and_then(|cmd| match cmd {
                            xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                                cmd.verifier_set.as_ref()
                            }
                            _ => None,
                        })
                        .map(|v| execution_verifier_models(&v.execution_verifier))
                        .unwrap_or_default(),
                    window,
                    cx,
                ),
                complete_verifiers: VerifierListForm::new(
                    "Complete verifiers",
                    "Add complete verifier",
                    command
                        .and_then(|cmd| match cmd {
                            xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                                cmd.verifier_set.as_ref()
                            }
                            _ => None,
                        })
                        .map(|v| complete_verifier_models(&v.complete_verifier))
                        .unwrap_or_default(),
                    window,
                    cx,
                ),
                documentation_open: false,
                inheritance_open: false,
                identification_open: false,
                significance_open: false,
                transmission_constraints_open: false,
                verifiers_open: false,
                container_details_open: false,
                _subscriptions: vec![name_subscription, kind_subscription, base_ref_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        command: Option<&xtce::MetaCommandSetTypeContent>,
        meta_command_set: Option<&xtce::MetaCommandSetType>,
        argument_type_set: Option<&xtce::ArgumentTypeSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = MetaCommandValues::from_command(command);
        self.documentation_open = false;
        self.inheritance_open = false;
        self.identification_open = false;
        self.significance_open = false;
        self.transmission_constraints_open = false;
        self.verifiers_open = false;
        self.container_details_open = false;
        let verifier_set = command.and_then(|cmd| match cmd {
            xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => cmd.verifier_set.as_ref(),
            _ => None,
        });
        self.execution_verifiers.update(cx, |form, cx| {
            form.load(
                verifier_set
                    .map(|v| execution_verifier_models(&v.execution_verifier))
                    .unwrap_or_default(),
                window,
                cx,
            );
        });
        self.complete_verifiers.update(cx, |form, cx| {
            form.load(
                verifier_set
                    .map(|v| complete_verifier_models(&v.complete_verifier))
                    .unwrap_or_default(),
                window,
                cx,
            );
        });
        self.transmission_constraints.update(cx, |form, cx| {
            form.load(
                command.and_then(|cmd| match cmd {
                    xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                        cmd.transmission_constraint_list.as_ref()
                    }
                    _ => None,
                }),
                window,
                cx,
            );
        });
        sync_select(
            &self.consequence_level_select,
            values.default_significance.consequence_level,
            window,
            cx,
        );
        *self.assignment_context.borrow_mut() = AssignmentContext::new(
            &values.base_meta_command_ref,
            meta_command_set,
            argument_type_set,
        );
        sync_select(
            &self.kind_select,
            command.map(kind).unwrap_or(MetaCommandKind::MetaCommand),
            window,
            cx,
        );
        self.command_container_present
            .set(values.command_container.present);
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
        self.base_assignment_list.update(cx, |list, cx| {
            list.rows = base_assignment_models(&values.base_assignments);
            list.editors.clear();
            list.cache_order.clear();
            list.context = self.assignment_context.clone();
            list.list_state
                .reset_with_uniform_height(list.rows.len(), px(50.));
            cx.notify();
        });
        self.argument_list.update(cx, |list, cx| {
            list.rows = command_argument_models(&values.arguments);
            list.editors.clear();
            list.context = self.assignment_context.clone();
            list.optional_fields_open = false;
            cx.notify();
        });
        self.entry_list.update(cx, |list, cx| {
            list.set_rows(
                decode_container_entries(&values.command_container.entries),
                window,
                cx,
            );
        });
        for (input, value) in [
            (&self.name_or_ref_input, values.name_or_ref),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.system_name_input, values.system_name),
            (
                &self.base_meta_command_ref_input,
                values.base_meta_command_ref,
            ),
            (&self.block_steps_input, values.block_steps),
            (&self.container_name_input, values.command_container.name),
            (
                &self.container_short_description_input,
                values.command_container.short_description,
            ),
            (
                &self.container_long_description_input,
                values.command_container.long_description,
            ),
            (
                &self.container_base_ref_input,
                values.command_container.base_ref,
            ),
            (
                &self.reason_for_warning_input,
                values.default_significance.reason_for_warning,
            ),
            (
                &self.space_system_at_risk_input,
                values.default_significance.space_system_at_risk,
            ),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_or_ref_input, cx)
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(gpui::px(520.))
            .child(Input::new(&self.name_or_ref_input))
    }

    pub(super) fn apply_to(&self, command: &mut xtce::MetaCommandSetTypeContent, cx: &App) {
        let selected_kind = selected_value(&self.kind_select, MetaCommandKind::MetaCommand, cx);
        replace_kind(command, selected_kind);
        let command_container = CommandContainerValues {
            present: self.command_container_present.get(),
            name: value(&self.container_name_input, cx),
            short_description: value(&self.container_short_description_input, cx),
            long_description: value(&self.container_long_description_input, cx),
            base_ref: value(&self.container_base_ref_input, cx),
            entries: command_container_entry_rows_value(&self.entry_list, cx),
        };
        let default_significance = SignificanceValues {
            consequence_level: selected_value(
                &self.consequence_level_select,
                ConsequenceLevelChoice::None,
                cx,
            ),
            reason_for_warning: value(&self.reason_for_warning_input, cx),
            space_system_at_risk: value(&self.space_system_at_risk_input, cx),
        };
        MetaCommandValues {
            name_or_ref: value(&self.name_or_ref_input, cx),
            short_description: value(&self.short_description_input, cx),
            long_description: value(&self.long_description_input, cx),
            abstract_: selected_value(&self.abstract_select, AbstractChoice::Concrete, cx)
                == AbstractChoice::Abstract,
            system_name: value(&self.system_name_input, cx),
            base_meta_command_ref: value(&self.base_meta_command_ref_input, cx),
            base_assignments: assignment_rows_value(
                &self.base_assignment_list,
                &self.assignment_context,
                cx,
            ),
            arguments: command_argument_rows_value(&self.argument_list, cx),
            block_steps: value(&self.block_steps_input, cx),
            command_container,
            default_significance,
        }
        .apply_to(command);
        if let xtce::MetaCommandSetTypeContent::MetaCommand(cmd) = command {
            cmd.transmission_constraint_list = self.transmission_constraints.read(cx).to_list(cx);
            let execs = self.execution_verifiers.read(cx).to_execution_verifiers(cx);
            let comps = self.complete_verifiers.read(cx).to_complete_verifiers(cx);

            if execs.is_empty() && comps.is_empty() {
                if let Some(set) = &mut cmd.verifier_set {
                    set.execution_verifier.clear();
                    set.complete_verifier.clear();
                    if set.transferred_to_range_verifier.is_none()
                        && set.sent_from_range_verifier.is_none()
                        && set.received_verifier.is_none()
                        && set.accepted_verifier.is_none()
                        && set.queued_verifier.is_none()
                        && set.failed_verifier.is_none()
                    {
                        cmd.verifier_set = None;
                    }
                }
            } else {
                let set = cmd.verifier_set.get_or_insert_with(|| xtce::VerifierSetType {
                    transferred_to_range_verifier: None,
                    sent_from_range_verifier: None,
                    received_verifier: None,
                    accepted_verifier: None,
                    queued_verifier: None,
                    execution_verifier: Vec::new(),
                    complete_verifier: Vec::new(),
                    failed_verifier: None,
                });
                set.execution_verifier = execs;
                set.complete_verifier = comps;
            }
        }
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        let selected_kind = selected_value(&self.kind_select, MetaCommandKind::MetaCommand, cx);
        let form = v_flex().gap_5().child(select_field(
            "Meta command element",
            "Required",
            &self.kind_select,
            cx,
        ));
        if selected_kind == MetaCommandKind::MetaCommandRef {
            return form.child(field(
                "Meta command reference",
                "Required",
                &self.name_or_ref_input,
                cx,
            ));
        }
        match selected_kind {
            MetaCommandKind::MetaCommand => form
                .child(select_field(
                    "Abstract",
                    "Required",
                    &self.abstract_select,
                    cx,
                ))
                .child(self.render_arguments(cx))
                .child(self.render_documentation(cx))
                .child(self.render_inheritance(cx))
                .child(self.render_identification(cx))
                .child(self.render_significance(cx))
                .child(self.render_transmission_constraints(cx))
                .child(self.render_verifiers(cx))
                .child(self.render_command_container(cx)),
            MetaCommandKind::BlockMetaCommand => form
                .child(field(
                    "Meta command steps",
                    "meta command ref | argument=value, argument=value",
                    &self.block_steps_input,
                    cx,
                ))
                .child(self.render_documentation(cx)),
            MetaCommandKind::MetaCommandRef => unreachable!(),
        }
    }

    fn render_documentation(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.documentation_open)
            .child(
                Button::new("toggle-meta-command-documentation")
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

    fn render_inheritance(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.inheritance_open)
            .child(
                Button::new("toggle-meta-command-inheritance")
                    .small()
                    .link()
                    .icon(if self.inheritance_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Inheritance")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.inheritance_open = !this.inheritance_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_4()
                    .child(field(
                        "Base meta command reference",
                        "Optional",
                        &self.base_meta_command_ref_input,
                        cx,
                    ))
                    .child(self.render_base_assignments(cx)),
            )
    }

    fn render_identification(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.identification_open)
            .child(
                Button::new("toggle-meta-command-identification")
                    .small()
                    .link()
                    .icon(if self.identification_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Identification")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.identification_open = !this.identification_open;
                        cx.notify();
                    })),
            )
            .content(v_flex().pt_3().child(field(
                "System name",
                "Optional",
                &self.system_name_input,
                cx,
            )))
    }

    fn render_significance(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.significance_open)
            .child(
                Button::new("toggle-meta-command-significance")
                    .small()
                    .link()
                    .icon(if self.significance_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Significance")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.significance_open = !this.significance_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_4()
                    .child(select_field(
                        "Consequence level",
                        "Required if significance specified",
                        &self.consequence_level_select,
                        cx,
                    ))
                    .child(field(
                        "Reason for warning",
                        "Optional",
                        &self.reason_for_warning_input,
                        cx,
                    ))
                    .child(field(
                        "Space system at risk",
                        "Optional",
                        &self.space_system_at_risk_input,
                        cx,
                    )),
            )
    }

    fn render_transmission_constraints(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.transmission_constraints_open)
            .child(
                Button::new("toggle-meta-command-transmission-constraints")
                    .small()
                    .link()
                    .icon(if self.transmission_constraints_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Transmission constraints")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.transmission_constraints_open = !this.transmission_constraints_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .child(self.transmission_constraints.clone()),
            )
    }

    fn render_verifiers(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.verifiers_open)
            .child(
                Button::new("toggle-meta-command-verifiers")
                    .small()
                    .link()
                    .icon(if self.verifiers_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Verifiers")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.verifiers_open = !this.verifiers_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_4()
                    .child(self.execution_verifiers.clone())
                    .child(self.complete_verifiers.clone()),
            )
    }

    fn render_base_assignments(&self, _: &App) -> Div {
        v_flex().w_full().child(self.base_assignment_list.clone())
    }

    fn render_arguments(&self, _: &App) -> Div {
        v_flex().w_full().child(self.argument_list.clone())
    }

    fn render_command_container(&self, cx: &mut Context<Self>) -> Div {
        let present = self.command_container_present.get();
        if !present {
            let description = if value(&self.base_meta_command_ref_input, cx)
                .trim()
                .is_empty()
            {
                "No command container is defined. This command does not define binary packaging."
            } else {
                "No local command container is defined. The base MetaCommand's packaging will be used."
            };
            return v_flex()
                .gap_3()
                .child(div().text_lg().font_semibold().child("Command container"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(description),
                )
                .child(
                    Button::new("add-command-container")
                        .primary()
                        .icon(IconName::Plus)
                        .label("Add command container")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.command_container_present.set(true);
                            cx.notify();
                        })),
                );
        }

        v_flex()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_lg().font_semibold().child("Command container"))
                    .child(
                        Button::new("remove-command-container")
                            .ghost()
                            .icon(IconName::Minus)
                            .label("Remove command container")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.command_container_present.set(false);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Collapsible::new()
                    .open(self.container_details_open)
                    .child(
                        Button::new("toggle-command-container-details")
                            .small()
                            .link()
                            .icon(if self.container_details_open {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .label("Container details")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.container_details_open = !this.container_details_open;
                                cx.notify();
                            })),
                    )
                    .content(
                        v_flex()
                            .pt_3()
                            .gap_4()
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_start()
                                    .child(field(
                                        "Container name",
                                        "Required",
                                        &self.container_name_input,
                                        cx,
                                    ))
                                    .child(field(
                                        "Base container reference",
                                        "Optional",
                                        &self.container_base_ref_input,
                                        cx,
                                    )),
                            )
                            .child(field(
                                "Short description",
                                "Optional",
                                &self.container_short_description_input,
                                cx,
                            ))
                            .child(field(
                                "Long description",
                                "Optional",
                                &self.container_long_description_input,
                                cx,
                            )),
                    ),
            )
            .child(self.entry_list.clone())
    }
}

impl Render for MetaCommandForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

struct BaseAssignmentListView {
    rows: Vec<BaseAssignmentData>,
    editors: HashMap<usize, Entity<BaseAssignmentRow>>,
    cache_order: VecDeque<usize>,
    context: Rc<RefCell<AssignmentContext>>,
    list_state: ListState,
}

#[derive(Clone, Default)]
struct BaseAssignmentData {
    name: String,
    value: String,
}

struct BaseAssignmentRow {
    name_input: Entity<InputState>,
    value_input: Entity<InputState>,
}

struct ArgumentListView {
    rows: Vec<CommandArgumentData>,
    editors: HashMap<usize, Entity<CommandArgumentRow>>,
    context: Rc<RefCell<AssignmentContext>>,
    optional_fields_open: bool,
}

#[derive(Clone, Default)]
struct CommandArgumentData {
    name: String,
    type_ref: String,
    initial_value: String,
    short_description: String,
}

struct CommandArgumentRow {
    name_input: Entity<InputState>,
    type_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    optional_fields_open: bool,
}

struct EntryListView {
    rows: Vec<EditableContainerEntry>,
    editors: HashMap<usize, Entity<CommandContainerEntryRow>>,
    cache_order: VecDeque<usize>,
    arguments: Entity<ArgumentListView>,
    command_name_input: Entity<InputState>,
    selected_index: Option<usize>,
    visible_indices: Vec<usize>,
    bit_positions: Vec<Option<u64>>,
    packet_layout_open: bool,
    list_state: ListState,
}

struct CommandContainerEntryRow {
    kind_select: Entity<SelectState<Vec<CommandContainerEntryKind>>>,
    primary_input: Entity<InputState>,
    secondary_input: Entity<InputState>,
    tertiary_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

fn touch_cache(cache_order: &mut VecDeque<usize>, index: usize) {
    if let Some(position) = cache_order.iter().position(|candidate| *candidate == index) {
        cache_order.remove(position);
    }
    cache_order.push_back(index);
}

fn base_assignment_data(row: &Entity<BaseAssignmentRow>, cx: &App) -> BaseAssignmentData {
    let row = row.read(cx);
    BaseAssignmentData {
        name: value(&row.name_input, cx),
        value: value(&row.value_input, cx),
    }
}

fn command_argument_data(row: &Entity<CommandArgumentRow>, cx: &App) -> CommandArgumentData {
    let row = row.read(cx);
    CommandArgumentData {
        name: value(&row.name_input, cx),
        type_ref: value(&row.type_ref_input, cx),
        initial_value: value(&row.initial_value_input, cx),
        short_description: value(&row.short_description_input, cx),
    }
}

fn container_entry_data(
    row: &Entity<CommandContainerEntryRow>,
    cx: &App,
) -> EditableContainerEntry {
    let row = row.read(cx);
    let kind = selected_value(&row.kind_select, CommandContainerEntryKind::ArgumentRef, cx);
    let primary = value(&row.primary_input, cx);
    let secondary = value(&row.secondary_input, cx);
    let tertiary = value(&row.tertiary_input, cx);
    match kind {
        CommandContainerEntryKind::ArgumentRef => EditableContainerEntry::ArgumentRef {
            reference: primary,
            offset: secondary.trim().parse().ok(),
            description: optional_value(tertiary),
        },
        CommandContainerEntryKind::ParameterRef => EditableContainerEntry::ParameterRef {
            reference: primary,
            offset: secondary.trim().parse().ok(),
            description: optional_value(tertiary),
        },
        CommandContainerEntryKind::ContainerRef => EditableContainerEntry::ContainerRef {
            reference: primary,
            offset: secondary.trim().parse().ok(),
            description: optional_value(tertiary),
        },
        CommandContainerEntryKind::FixedValue => EditableContainerEntry::FixedValue {
            name: optional_value(primary),
            binary_value: secondary,
            size_in_bits: tertiary.trim().parse().unwrap_or(i64::MIN),
        },
    }
}

impl BaseAssignmentListView {
    fn editor(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<BaseAssignmentRow> {
        if let Some(editor) = self.editors.get(&index).cloned() {
            touch_cache(&mut self.cache_order, index);
            return editor;
        }
        let data = self.rows[index].clone();
        let editor = new_assignment_row(&data.name, &data.value, self.context.clone(), window, cx);
        self.editors.insert(index, editor.clone());
        touch_cache(&mut self.cache_order, index);
        while self.editors.len() > 8 {
            let Some(evicted) = self.cache_order.pop_front() else {
                break;
            };
            if evicted == index {
                self.cache_order.push_back(evicted);
                continue;
            }
            if let Some(editor) = self.editors.remove(&evicted) {
                self.rows[evicted] = base_assignment_data(&editor, cx);
            }
        }
        editor
    }

    fn flush_editors(&mut self, cx: &App) {
        for (index, editor) in &self.editors {
            self.rows[*index] = base_assignment_data(editor, cx);
        }
    }

    fn render_item(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let row = self.editor(index, window, cx);
        h_flex()
            .w_full()
            .h(px(50.))
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(row)
            .child(
                div().w(px(52.)).flex_none().child(
                    Button::new(format!("remove-base-assignment-{index}"))
                        .ghost()
                        .small()
                        .icon(IconName::Minus)
                        .tooltip("Remove assignment")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if index < this.rows.len() {
                                this.flush_editors(cx);
                                this.rows.remove(index);
                                this.editors.clear();
                                this.cache_order.clear();
                                this.list_state.splice(index..index + 1, 0);
                                cx.notify();
                            }
                        })),
                ),
            )
            .into_any_element()
    }
}

impl Render for BaseAssignmentListView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let row_count = self.rows.len();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child("Base argument assignments"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        "Assign values to arguments inherited from the base command",
                                    ),
                            ),
                    )
                    .child(
                        Button::new("add-base-argument-assignment")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add assignment")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let index = this.rows.len();
                                this.rows.push(BaseAssignmentData::default());
                                this.list_state.splice(index..index, 1);
                                this.list_state.scroll_to_reveal_item(index);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                v_flex()
                    .w_full()
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
                            .child(div().flex_1().child("Argument name"))
                            .child(div().flex_1().child("Value"))
                            .child(div().w(px(52.)).flex_none().child("Actions")),
                    )
                    .when(row_count > 0, |table| {
                        table.child(
                            list(
                                self.list_state.clone(),
                                cx.processor(BaseAssignmentListView::render_item),
                            )
                            .w_full()
                            .h(virtual_list_height(row_count, 50., 6)),
                        )
                    }),
            )
    }
}

impl Render for BaseAssignmentRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(div().flex_1().min_w_0().child(Input::new(&self.name_input)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Input::new(&self.value_input)),
            )
    }
}

impl ArgumentListView {
    fn editor(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CommandArgumentRow> {
        if let Some(editor) = self.editors.get(&index).cloned() {
            return editor;
        }
        let data = self.rows[index].clone();
        let editor = new_command_argument_row(
            &data,
            self.optional_fields_open,
            self.context.clone(),
            window,
            cx,
        );
        self.editors.insert(index, editor.clone());
        editor
    }

    fn flush_editors(&mut self, cx: &App) {
        for (index, editor) in &self.editors {
            self.rows[*index] = command_argument_data(editor, cx);
        }
    }

    fn render_item(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let row = self.editor(index, window, cx);
        h_flex()
            .w_full()
            .h(px(50.))
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(row)
            .child(
                div().w(px(52.)).flex_none().child(
                    Button::new(format!("remove-command-argument-{index}"))
                        .ghost()
                        .small()
                        .icon(IconName::Minus)
                        .tooltip("Remove argument")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if index < this.rows.len() {
                                this.flush_editors(cx);
                                this.rows.remove(index);
                                this.editors.clear();
                                cx.notify();
                            }
                        })),
                ),
            )
            .into_any_element()
    }
}

impl Render for ArgumentListView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let row_count = self.rows.len();
        let rows = (0..row_count)
            .map(|index| self.render_item(index, window, cx))
            .collect::<Vec<_>>();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Arguments"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Arguments declared by this MetaCommand"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("toggle-command-argument-optional-fields")
                                    .small()
                                    .link()
                                    .icon(if self.optional_fields_open {
                                        IconName::ChevronDown
                                    } else {
                                        IconName::ChevronRight
                                    })
                                    .label("Optional fields")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.flush_editors(cx);
                                        this.editors.clear();
                                        this.optional_fields_open = !this.optional_fields_open;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("add-command-argument")
                                    .small()
                                    .icon(IconName::Plus)
                                    .label("Add argument")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.rows.push(CommandArgumentData::default());
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .id("command-arguments-table-scroll-boundary")
                    .w_full()
                    .on_scroll_wheel(|event, _, cx| {
                        let delta = event.delta.pixel_delta(px(20.));
                        if delta.x.abs() > delta.y.abs() {
                            cx.stop_propagation();
                        }
                    })
                    .child(
                        div()
                            .id("command-arguments-table-horizontal-scroll")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                v_flex()
                                    .min_w(if self.optional_fields_open {
                                        px(900.)
                                    } else {
                                        px(620.)
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
                                            .child(
                                                div()
                                                    .w(px(180.))
                                                    .flex_none()
                                                    .child("Argument name"),
                                            )
                                            .child(div().flex_1().child("Argument type reference"))
                                            .when(self.optional_fields_open, |header| {
                                                header
                                                    .child(
                                                        div()
                                                            .w(px(180.))
                                                            .flex_none()
                                                            .child("Initial value"),
                                                    )
                                                    .child(
                                                        div().flex_1().child("Short description"),
                                                    )
                                            })
                                            .child(div().w(px(52.)).flex_none().child("Actions")),
                                    )
                                    .when(row_count > 0, |table| table.children(rows)),
                            ),
                    ),
            )
    }
}

impl Render for CommandArgumentRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(
                div()
                    .w(px(180.))
                    .flex_none()
                    .child(Input::new(&self.name_input)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Input::new(&self.type_ref_input)),
            )
            .when(self.optional_fields_open, |row| {
                row.child(
                    div()
                        .w(px(180.))
                        .flex_none()
                        .child(Input::new(&self.initial_value_input)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Input::new(&self.short_description_input)),
                )
            })
    }
}

impl EntryListView {
    fn new(
        rows: Vec<EditableContainerEntry>,
        arguments: Entity<ArgumentListView>,
        command_name_input: Entity<InputState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let selected_index = (!rows.is_empty()).then_some(0);
        let row_count = rows.len();
        let mut this = Self {
            visible_indices: (0..rows.len()).collect(),
            list_state: ListState::new(rows.len(), ListAlignment::Top, px(58.))
                .with_uniform_item_height(px(54.)),
            rows,
            editors: HashMap::new(),
            cache_order: VecDeque::new(),
            arguments,
            command_name_input,
            selected_index,
            bit_positions: vec![None; row_count],
            packet_layout_open: true,
        };
        this.rebuild_visible(cx);
        this
    }

    fn set_rows(
        &mut self,
        rows: Vec<EditableContainerEntry>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = rows;
        self.editors.clear();
        self.cache_order.clear();
        self.selected_index = (!self.rows.is_empty()).then_some(0);
        self.bit_positions = vec![None; self.rows.len()];
        self.packet_layout_open = true;
        self.rebuild_visible(cx);
    }

    fn rebuild_visible(&mut self, cx: &mut Context<Self>) {
        self.flush_editors(cx);
        self.visible_indices = (0..self.rows.len()).collect();
        self.list_state
            .reset_with_uniform_height(self.visible_indices.len(), px(54.));
        if self
            .selected_index
            .is_none_or(|selected| selected >= self.rows.len())
        {
            self.selected_index = self.visible_indices.first().copied();
        }
        cx.notify();
    }

    fn editor(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CommandContainerEntryRow> {
        if let Some(editor) = self.editors.get(&index).cloned() {
            touch_cache(&mut self.cache_order, index);
            return editor;
        }
        let editor = new_command_container_entry_row(
            self.rows[index].clone(),
            self.arguments.clone(),
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
                self.rows[evicted] = container_entry_data(&editor, cx);
            }
        }
        editor
    }

    fn flush_editors(&mut self, cx: &App) {
        for (index, editor) in &self.editors {
            self.rows[*index] = container_entry_data(editor, cx);
        }
    }

    fn current_rows(&self, cx: &App) -> Vec<EditableContainerEntry> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                self.editors
                    .get(&index)
                    .map_or_else(|| row.clone(), |editor| container_entry_data(editor, cx))
            })
            .collect()
    }

    fn move_entry(&mut self, index: usize, target: usize, cx: &mut Context<Self>) {
        self.flush_editors(cx);
        if swap_rows(&mut self.rows, index, target) {
            self.editors.clear();
            self.cache_order.clear();
            self.selected_index = Some(target);
            self.rebuild_visible(cx);
            cx.notify();
        }
    }

    fn remove_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.rows.len() {
            return;
        }
        self.flush_editors(cx);
        self.rows.remove(index);
        self.editors.clear();
        self.cache_order.clear();
        self.selected_index = if self.rows.is_empty() {
            None
        } else {
            Some(index.min(self.rows.len() - 1))
        };
        self.rebuild_visible(cx);
    }

    fn render_list_item(
        &mut self,
        visible_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(&index) = self.visible_indices.get(visible_index) else {
            return div().into_any_element();
        };
        let editor = self.editor(index, window, cx);
        let selected = self.selected_index == Some(index);
        h_flex()
            .id(format!("command-container-entry-list-row-{index}"))
            .w_full()
            .h(px(54.))
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .when(selected, |row| row.bg(cx.theme().sidebar_accent))
            .child(
                div().w(px(52.)).flex_none().child(
                    Button::new(format!("select-command-container-entry-{index}"))
                        .ghost()
                        .small()
                        .label(
                            self.bit_positions
                                .get(index)
                                .copied()
                                .flatten()
                                .map_or_else(|| "—".to_owned(), |position| position.to_string()),
                        )
                        .tooltip("Select entry")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_index = Some(index);
                            cx.notify();
                        })),
                ),
            )
            .child(
                h_flex()
                    .w(px(100.))
                    .flex_none()
                    .gap_1()
                    .child(
                        Button::new(format!("move-command-container-entry-up-{index}"))
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
                        Button::new(format!("move-command-container-entry-down-{index}"))
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
                        Button::new(format!("remove-command-container-entry-{index}"))
                            .ghost()
                            .small()
                            .icon(IconName::Minus)
                            .tooltip("Remove entry")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_entry(index, cx);
                            })),
                    ),
            )
            .child(editor)
            .into_any_element()
    }
}

impl Render for EntryListView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let row_count = self.rows.len();
        let rows = self.current_rows(cx);
        self.bit_positions = command_entry_bit_positions(&rows, &self.arguments, cx);
        let command_name = value(&self.command_name_input, cx);
        let packet_layout = self
            .packet_layout_open
            .then(|| command_packet_layout(&rows, &self.arguments, &command_name, cx));
        let entry_table = div()
            .id("entry-table-scroll-boundary")
            .w_full()
            .on_scroll_wheel(|_, _, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .id("entry-table-horizontal-scroll")
                    .w_full()
                    .overflow_x_scroll()
                    .child(
                        v_flex()
                            .w_full()
                            .min_w(px(930.))
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
                                    .child(div().w(px(52.)).child("Bit"))
                                    .child(div().w(px(100.)).child("Actions"))
                                    .child(div().w(px(130.)).child("Type"))
                                    .child(div().flex_1().child("Reference / name"))
                                    .child(div().w(px(86.)).child("Offset"))
                                    .child(div().w(px(120.)).child("Binary value"))
                                    .child(div().w(px(86.)).child("Size in bits"))
                                    .child(div().flex_1().child("Description")),
                            )
                            .child(
                                div()
                                    .id("entry-list-scroll-boundary")
                                    .on_scroll_wheel(|event, _, cx| {
                                        let delta = event.delta.pixel_delta(px(20.));
                                        if delta.y.abs() >= delta.x.abs() {
                                            cx.stop_propagation();
                                        }
                                    })
                                    .child(
                                        list(
                                            self.list_state.clone(),
                                            cx.processor(EntryListView::render_list_item),
                                        )
                                        .w_full()
                                        .h(px(352.)),
                                    ),
                            ),
                    ),
            );
        v_flex()
            .w_full()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .child(format!("{row_count} entries")),
                    )
                    .child(
                        Button::new("add-command-container-entry")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add entry")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.rows.push(EditableContainerEntry::ArgumentRef {
                                    reference: String::new(),
                                    offset: None,
                                    description: None,
                                });
                                this.selected_index = Some(this.rows.len() - 1);
                                this.rebuild_visible(cx);
                            })),
                    ),
            )
            .child(entry_table)
            .child(
                Collapsible::new()
                    .open(self.packet_layout_open)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                Button::new("toggle-command-packet-layout")
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
                                    Button::new("refresh-command-packet-layout")
                                        .small()
                                        .ghost()
                                        .label("Refresh")
                                        .on_click(cx.listener(|_, _, _, cx| cx.notify())),
                                )
                            }),
                    )
                    .content(v_flex().pt_2().child(
                        packet_layout.as_ref().map_or_else(
                            || div(),
                            |layout| render_command_packet_layout(layout, cx),
                        ),
                    )),
            )
    }
}

fn swap_rows<T>(rows: &mut [T], index: usize, target: usize) -> bool {
    if index >= rows.len() || target >= rows.len() || index == target {
        return false;
    }
    rows.swap(index, target);
    true
}

fn virtual_list_height(
    row_count: usize,
    estimated_row_height: f32,
    visible_rows: usize,
) -> gpui::Pixels {
    px(estimated_row_height * row_count.min(visible_rows) as f32)
}

impl Render for CommandContainerEntryRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fixed_value = selected_value(
            &self.kind_select,
            CommandContainerEntryKind::ArgumentRef,
            cx,
        ) == CommandContainerEntryKind::FixedValue;
        let empty_cell = || {
            div()
                .h(px(32.))
                .rounded_md()
                .bg(cx.theme().muted.opacity(0.45))
        };
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(
                div()
                    .w(px(130.))
                    .flex_none()
                    .child(Select::new(&self.kind_select).w_full()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Input::new(&self.primary_input)),
            )
            .child(div().w(px(86.)).flex_none().child(if fixed_value {
                empty_cell().into_any_element()
            } else {
                Input::new(&self.secondary_input).into_any_element()
            }))
            .child(div().w(px(120.)).flex_none().child(if fixed_value {
                Input::new(&self.secondary_input).into_any_element()
            } else {
                empty_cell().into_any_element()
            }))
            .child(div().w(px(86.)).flex_none().child(if fixed_value {
                Input::new(&self.tertiary_input).into_any_element()
            } else {
                empty_cell().into_any_element()
            }))
            .child(div().flex_1().min_w_0().child(if fixed_value {
                empty_cell().into_any_element()
            } else {
                Input::new(&self.tertiary_input).into_any_element()
            }))
    }
}

#[derive(Clone)]
struct ArgumentSpec {
    name: String,
    rule: ValueRule,
}

#[derive(Clone)]
enum ValueRule {
    String,
    Integer,
    Float,
    Boolean(Vec<String>),
    Enumerated(Vec<String>),
    Binary,
    Other,
}

struct CommandArguments {
    name: String,
    base_ref: Option<String>,
    arguments: Vec<(String, String)>,
    container_entries: Vec<EditableContainerEntry>,
}

struct AssignmentContext {
    current_base: String,
    commands: HashMap<String, CommandArguments>,
    type_rules: HashMap<String, ValueRule>,
    type_sizes: HashMap<String, u64>,
    type_names: Vec<String>,
}

impl AssignmentContext {
    fn new(
        current_base: &str,
        meta_command_set: Option<&xtce::MetaCommandSetType>,
        argument_type_set: Option<&xtce::ArgumentTypeSetType>,
    ) -> Self {
        let commands = meta_command_set
            .into_iter()
            .flat_map(|set| &set.content)
            .filter_map(|command| {
                let xtce::MetaCommandSetTypeContent::MetaCommand(command) = command else {
                    return None;
                };
                Some((
                    command.name.clone(),
                    CommandArguments {
                        name: command.name.clone(),
                        base_ref: command
                            .base_meta_command
                            .as_ref()
                            .map(|base| base.meta_command_ref.clone()),
                        arguments: command
                            .argument_list
                            .iter()
                            .flat_map(|list| &list.argument)
                            .map(|argument| {
                                (argument.name.clone(), argument.argument_type_ref.clone())
                            })
                            .collect(),
                        container_entries: command
                            .command_container
                            .as_ref()
                            .map(|container| {
                                decode_container_entries(&encode_container_entries(
                                    &container.entry_list,
                                ))
                            })
                            .unwrap_or_default(),
                    },
                ))
            })
            .collect();
        let type_entries = argument_type_set
            .into_iter()
            .flat_map(|set| &set.content)
            .map(argument_type_rule)
            .collect::<Vec<_>>();
        let type_names = type_entries.iter().map(|(name, _)| name.clone()).collect();
        let type_rules = type_entries.into_iter().collect();
        let type_sizes = argument_type_sizes(argument_type_set);
        Self {
            current_base: current_base.to_owned(),
            commands,
            type_rules,
            type_sizes,
            type_names,
        }
    }

    fn active_arguments(&self) -> Vec<ArgumentSpec> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        self.collect_arguments(&self.current_base, &mut visited, &mut result);
        result
    }

    fn active_argument_sizes(&self) -> HashMap<String, u64> {
        let mut arguments = Vec::new();
        self.collect_argument_types(&self.current_base, &mut HashSet::new(), &mut arguments);
        arguments
            .into_iter()
            .filter_map(|(name, type_ref)| {
                self.type_sizes
                    .get(&type_ref)
                    .or_else(|| {
                        self.type_sizes
                            .get(type_ref.rsplit('/').next().unwrap_or(&type_ref))
                    })
                    .copied()
                    .map(|size| (name, size))
            })
            .collect()
    }

    fn collect_argument_types(
        &self,
        command_ref: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<(String, String)>,
    ) {
        let Some(command) = self.command(command_ref, visited) else {
            return;
        };
        if let Some(base_ref) = &command.base_ref {
            self.collect_argument_types(base_ref, visited, result);
        }
        result.extend(command.arguments.iter().cloned());
    }

    fn active_container_entries(&self) -> Vec<EditableContainerEntry> {
        let mut entries = Vec::new();
        self.collect_container_entries(&self.current_base, &mut HashSet::new(), &mut entries);
        entries
    }

    fn active_container_entries_with_sources(&self) -> Vec<(String, Vec<EditableContainerEntry>)> {
        let mut entries = Vec::new();
        self.collect_container_entries_with_sources(
            &self.current_base,
            &mut HashSet::new(),
            &mut entries,
        );
        entries
    }

    fn collect_container_entries_with_sources(
        &self,
        command_ref: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<(String, Vec<EditableContainerEntry>)>,
    ) {
        let Some(command) = self.command(command_ref, visited) else {
            return;
        };
        if let Some(base_ref) = &command.base_ref {
            self.collect_container_entries_with_sources(base_ref, visited, result);
        }
        if !command.container_entries.is_empty() {
            result.push((command.name.clone(), command.container_entries.clone()));
        }
    }

    fn collect_container_entries(
        &self,
        command_ref: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<EditableContainerEntry>,
    ) {
        let Some(command) = self.command(command_ref, visited) else {
            return;
        };
        if let Some(base_ref) = &command.base_ref {
            self.collect_container_entries(base_ref, visited, result);
        }
        result.extend(command.container_entries.iter().cloned());
    }

    fn command<'a>(
        &'a self,
        command_ref: &str,
        visited: &mut HashSet<String>,
    ) -> Option<&'a CommandArguments> {
        if command_ref.is_empty() || !visited.insert(command_ref.to_owned()) {
            return None;
        }
        let command_name = command_ref.rsplit('/').next().unwrap_or(command_ref);
        self.commands
            .get(command_ref)
            .or_else(|| self.commands.get(command_name))
    }

    fn collect_arguments(
        &self,
        command_ref: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<ArgumentSpec>,
    ) {
        if command_ref.is_empty() || !visited.insert(command_ref.to_owned()) {
            return;
        }
        let command_name = command_ref.rsplit('/').next().unwrap_or(command_ref);
        let Some(command) = self
            .commands
            .get(command_ref)
            .or_else(|| self.commands.get(command_name))
        else {
            return;
        };
        if let Some(base_ref) = &command.base_ref {
            self.collect_arguments(base_ref, visited, result);
        }
        result.extend(command.arguments.iter().map(|(name, type_ref)| {
            ArgumentSpec {
                name: name.clone(),
                rule: self
                    .type_rules
                    .get(type_ref)
                    .or_else(|| {
                        self.type_rules
                            .get(type_ref.rsplit('/').next().unwrap_or(type_ref))
                    })
                    .cloned()
                    .unwrap_or(ValueRule::Other),
            }
        }));
    }

    fn rule_for(&self, argument_name: &str) -> Option<ValueRule> {
        self.active_arguments()
            .into_iter()
            .find(|argument| argument.name == argument_name)
            .map(|argument| argument.rule)
    }

    fn rule_for_type(&self, type_ref: &str) -> Option<ValueRule> {
        self.type_rules
            .get(type_ref)
            .or_else(|| {
                self.type_rules
                    .get(type_ref.rsplit('/').next().unwrap_or(type_ref))
            })
            .cloned()
    }

    fn accepts_input(&self, argument_name: &str, value: &str) -> bool {
        self.rule_for(argument_name)
            .is_none_or(|rule| rule.accepts_input(value))
    }

    fn value_suggestions(&self, argument_name: &str) -> Vec<String> {
        self.rule_for(argument_name)
            .map_or_else(Vec::new, |rule| rule.suggestions())
    }

    fn is_complete_value(&self, argument_name: &str, value: &str) -> bool {
        self.rule_for(argument_name)
            .is_none_or(|rule| rule.is_complete(value))
    }

    fn accepts_type_value(&self, type_ref: &str, value: &str) -> bool {
        self.rule_for_type(type_ref)
            .is_none_or(|rule| rule.accepts_input(value))
    }

    fn type_value_suggestions(&self, type_ref: &str) -> Vec<String> {
        self.rule_for_type(type_ref)
            .map_or_else(Vec::new, |rule| rule.suggestions())
    }

    fn is_complete_type_value(&self, type_ref: &str, value: &str) -> bool {
        value.is_empty()
            || self
                .rule_for_type(type_ref)
                .is_none_or(|rule| rule.is_complete(value))
    }
}

impl ValueRule {
    fn accepts_input(&self, value: &str) -> bool {
        if value.is_empty() {
            return true;
        }
        match self {
            Self::String | Self::Other => true,
            Self::Integer => value == "-" || value.parse::<i64>().is_ok(),
            Self::Float => {
                value.parse::<f64>().is_ok()
                    || matches!(value, "-" | "+" | "." | "-." | "+.")
                    || value.ends_with(['e', 'E', '+', '-'])
                        && value[..value.len() - 1].parse::<f64>().is_ok()
            }
            Self::Boolean(values) | Self::Enumerated(values) => {
                values.iter().any(|candidate| candidate.starts_with(value))
            }
            Self::Binary => {
                let digits = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                    .unwrap_or(value);
                digits.is_empty() || digits.bytes().all(|byte| byte.is_ascii_hexdigit())
            }
        }
    }

    fn suggestions(&self) -> Vec<String> {
        match self {
            Self::Boolean(values) | Self::Enumerated(values) => values.clone(),
            _ => Vec::new(),
        }
    }

    fn is_complete(&self, value: &str) -> bool {
        match self {
            Self::String | Self::Other => true,
            Self::Integer => value.parse::<i64>().is_ok(),
            Self::Float => value.parse::<f64>().is_ok(),
            Self::Boolean(values) | Self::Enumerated(values) => {
                values.iter().any(|candidate| candidate == value)
            }
            Self::Binary => {
                let digits = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                    .unwrap_or(value);
                !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
            }
        }
    }
}

fn argument_type_rule(argument_type: &xtce::ArgumentTypeSetTypeContent) -> (String, ValueRule) {
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
            (value.name.clone(), ValueRule::String)
        }
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
            let labels = value
                .content
                .iter()
                .find_map(|content| match content {
                    xtce::EnumeratedArgumentTypeContent::EnumerationList(list) => Some(
                        list.enumeration
                            .iter()
                            .map(|entry| entry.label.clone())
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();
            (value.name.clone(), ValueRule::Enumerated(labels))
        }
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
            (value.name.clone(), ValueRule::Integer)
        }
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
            (value.name.clone(), ValueRule::Binary)
        }
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
            (value.name.clone(), ValueRule::Float)
        }
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => (
            value.name.clone(),
            ValueRule::Boolean(vec![
                value.one_string_value.clone(),
                value.zero_string_value.clone(),
            ]),
        ),
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => {
            (value.name.clone(), ValueRule::Other)
        }
        xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => {
            (value.name.clone(), ValueRule::Other)
        }
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => {
            (value.name.clone(), ValueRule::Other)
        }
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => {
            (value.name.clone(), ValueRule::Other)
        }
    }
}

fn argument_type_sizes(set: Option<&xtce::ArgumentTypeSetType>) -> HashMap<String, u64> {
    let Some(set) = set else {
        return HashMap::new();
    };
    set.content
        .iter()
        .filter_map(|argument_type| {
            resolve_argument_type_size(argument_type, set, &mut HashSet::new())
                .map(|size| (argument_type_name(argument_type).to_owned(), size))
        })
        .collect()
}

fn argument_type_name(argument_type: &xtce::ArgumentTypeSetTypeContent) -> &str {
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => &value.name,
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => &value.name,
    }
}

fn resolve_argument_type_size(
    argument_type: &xtce::ArgumentTypeSetTypeContent,
    set: &xtce::ArgumentTypeSetType,
    visiting: &mut HashSet<String>,
) -> Option<u64> {
    let name = argument_type_name(argument_type);
    if !visiting.insert(name.to_owned()) {
        return None;
    }
    let size = match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
            argument_encoding_size(&value.content)
        }
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
            argument_encoding_size(&value.content)
        }
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
            argument_encoding_size(&value.content)
        }
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
            argument_encoding_size(&value.content)
        }
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
            argument_encoding_size(&value.content)
        }
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
            argument_encoding_size(&value.content)
        }
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => value
            .member_list
            .member
            .iter()
            .try_fold(0_u64, |total, member| {
                let member_type = set
                    .content
                    .iter()
                    .find(|candidate| argument_type_name(candidate) == member.type_ref)?;
                total.checked_add(resolve_argument_type_size(member_type, set, visiting)?)
            }),
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(_)
        | xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(_)
        | xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(_) => None,
    };
    visiting.remove(name);
    size
}

trait ArgumentEncodingContent {
    fn fixed_size(&self) -> Option<u64>;
}

macro_rules! impl_argument_encoding_content {
    ($type:ty) => {
        impl ArgumentEncodingContent for $type {
            fn fixed_size(&self) -> Option<u64> {
                match self {
                    Self::BinaryDataEncoding(value) => match &value.size_in_bits {
                        xtce::ArgumentIntegerValueType::FixedValue(size) => {
                            u64::try_from(*size).ok().filter(|size| *size > 0)
                        }
                        _ => None,
                    },
                    Self::FloatDataEncoding(value) => Some(match value.size_in_bits {
                        xtce::FloatEncodingSizeInBitsType::_16 => 16,
                        xtce::FloatEncodingSizeInBitsType::_32 => 32,
                        xtce::FloatEncodingSizeInBitsType::_40 => 40,
                        xtce::FloatEncodingSizeInBitsType::_48 => 48,
                        xtce::FloatEncodingSizeInBitsType::_64 => 64,
                        xtce::FloatEncodingSizeInBitsType::_80 => 80,
                        xtce::FloatEncodingSizeInBitsType::_128 => 128,
                    }),
                    Self::IntegerDataEncoding(value) => u64::try_from(value.size_in_bits)
                        .ok()
                        .filter(|size| *size > 0),
                    Self::StringDataEncoding(value) => {
                        value.content.iter().find_map(|content| match content {
                            xtce::ArgumentStringDataEncodingTypeContent::SizeInBits(value) => {
                                u64::try_from(value.fixed.fixed_value)
                                    .ok()
                                    .filter(|size| *size > 0)
                            }
                            _ => None,
                        })
                    }
                    _ => None,
                }
            }
        }
    };
}

impl_argument_encoding_content!(xtce::StringArgumentTypeContent);
impl_argument_encoding_content!(xtce::EnumeratedArgumentTypeContent);
impl_argument_encoding_content!(xtce::IntegerArgumentTypeContent);
impl_argument_encoding_content!(xtce::BinaryArgumentTypeContent);
impl_argument_encoding_content!(xtce::FloatArgumentTypeContent);
impl_argument_encoding_content!(xtce::BooleanArgumentTypeContent);

fn argument_encoding_size<T: ArgumentEncodingContent>(content: &[T]) -> Option<u64> {
    content.iter().find_map(ArgumentEncodingContent::fixed_size)
}

enum AssignmentCompletionKind {
    ArgumentName,
    ArgumentTypeReference,
    Value(Entity<InputState>),
    InitialValue(Entity<InputState>),
}

struct AssignmentCompletionProvider {
    context: Rc<RefCell<AssignmentContext>>,
    kind: AssignmentCompletionKind,
}

struct EntryArgumentCompletionProvider {
    arguments: Entity<ArgumentListView>,
    kind_select: Entity<SelectState<Vec<CommandContainerEntryKind>>>,
}

fn matching_argument_names(names: impl IntoIterator<Item = String>, query: &str) -> Vec<String> {
    let query = query.to_ascii_lowercase();
    let mut candidates = names
        .into_iter()
        .filter_map(|name| {
            let name = name.trim();
            (!name.is_empty() && name.to_ascii_lowercase().contains(&query))
                .then(|| name.to_owned())
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    candidates
}

impl CompletionProvider for AssignmentCompletionProvider {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        _: CompletionContext,
        _: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        let query = text.slice(..offset).to_string();
        let normalized_query = query.to_ascii_lowercase();
        let candidates = match &self.kind {
            AssignmentCompletionKind::ArgumentName => self
                .context
                .borrow()
                .active_arguments()
                .into_iter()
                .map(|argument| argument.name)
                .collect(),
            AssignmentCompletionKind::ArgumentTypeReference => {
                self.context.borrow().type_names.clone()
            }
            AssignmentCompletionKind::Value(name_input) => {
                let argument_name = name_input.read(cx).value().to_string();
                self.context.borrow().value_suggestions(&argument_name)
            }
            AssignmentCompletionKind::InitialValue(type_ref_input) => {
                let type_ref = type_ref_input.read(cx).value().to_string();
                self.context.borrow().type_value_suggestions(&type_ref)
            }
        };
        let end = text.offset_to_position(offset);
        let items = candidates
            .into_iter()
            .filter(|candidate: &String| candidate.to_ascii_lowercase().contains(&normalized_query))
            .map(|candidate| CompletionItem {
                label: candidate.clone(),
                kind: Some(CompletionItemKind::VALUE),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end,
                    },
                    new_text: candidate,
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

impl CompletionProvider for EntryArgumentCompletionProvider {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        _: CompletionContext,
        _: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        if selected_value(
            &self.kind_select,
            CommandContainerEntryKind::ArgumentRef,
            cx,
        ) != CommandContainerEntryKind::ArgumentRef
        {
            return Task::ready(Ok(CompletionResponse::Array(Vec::new())));
        }

        let query = text.slice(..offset).to_string();
        let arguments = self.arguments.read(cx);
        let candidates = matching_argument_names(
            arguments.rows.iter().enumerate().map(|(index, model)| {
                arguments.editors.get(&index).map_or_else(
                    || model.name.clone(),
                    |editor| value(&editor.read(cx).name_input, cx),
                )
            }),
            &query,
        );

        let end = text.offset_to_position(offset);
        let items = candidates
            .into_iter()
            .map(|candidate| CompletionItem {
                label: candidate.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end,
                    },
                    new_text: candidate,
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

fn new_assignment_row(
    name: &str,
    value: &str,
    context: Rc<RefCell<AssignmentContext>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<BaseAssignmentRow> {
    cx.new(|cx| {
        let name_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .default_value(name.to_owned())
                .placeholder("Start typing an inherited argument name");
            input.lsp.completion_provider = Some(Rc::new(AssignmentCompletionProvider {
                context: context.clone(),
                kind: AssignmentCompletionKind::ArgumentName,
            }));
            input
        });
        let validation_name = name_input.clone();
        let validation_context = context.clone();
        let value_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .default_value(value.to_owned())
                .validate(move |value, cx| {
                    let name = validation_name.read(cx).value().to_string();
                    validation_context.borrow().accepts_input(&name, value)
                });
            input.lsp.completion_provider = Some(Rc::new(AssignmentCompletionProvider {
                context,
                kind: AssignmentCompletionKind::Value(name_input.clone()),
            }));
            input
        });
        BaseAssignmentRow {
            name_input,
            value_input,
        }
    })
}

fn command_argument_models(value: &str) -> Vec<CommandArgumentData> {
    value
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(4, " | ").map(str::trim);
            Some(CommandArgumentData {
                name: fields.next()?.to_owned(),
                type_ref: fields.next()?.to_owned(),
                initial_value: fields.next().unwrap_or_default().to_owned(),
                short_description: fields.next().unwrap_or_default().to_owned(),
            })
        })
        .collect()
}

fn new_command_argument_row(
    data: &CommandArgumentData,
    optional_fields_open: bool,
    context: Rc<RefCell<AssignmentContext>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<CommandArgumentRow> {
    cx.new(|cx| {
        let name_input = input_with_context(&data.name, window, cx);
        let type_ref_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .default_value(data.type_ref.clone())
                .placeholder("Start typing an ArgumentType name");
            input.lsp.completion_provider = Some(Rc::new(AssignmentCompletionProvider {
                context: context.clone(),
                kind: AssignmentCompletionKind::ArgumentTypeReference,
            }));
            input
        });
        let validation_type_ref = type_ref_input.clone();
        let validation_context = context.clone();
        let initial_value_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .default_value(data.initial_value.clone())
                .validate(move |value, cx| {
                    let type_ref = validation_type_ref.read(cx).value().to_string();
                    validation_context
                        .borrow()
                        .accepts_type_value(&type_ref, value)
                });
            input.lsp.completion_provider = Some(Rc::new(AssignmentCompletionProvider {
                context: context.clone(),
                kind: AssignmentCompletionKind::InitialValue(type_ref_input.clone()),
            }));
            input
        });
        CommandArgumentRow {
            name_input,
            type_ref_input,
            initial_value_input,
            short_description_input: input_with_context(&data.short_description, window, cx),
            optional_fields_open,
        }
    })
}

fn command_argument_rows_value(rows: &Entity<ArgumentListView>, cx: &App) -> String {
    let rows = rows.read(cx);
    rows.rows
        .iter()
        .enumerate()
        .filter_map(|(index, model)| {
            let data = rows
                .editors
                .get(&index)
                .map(|editor| command_argument_data(editor, cx))
                .unwrap_or_else(|| model.clone());
            let initial_value = if rows
                .context
                .borrow()
                .is_complete_type_value(data.type_ref.trim(), data.initial_value.trim())
            {
                data.initial_value
            } else {
                String::new()
            };
            (!data.name.trim().is_empty() && !data.type_ref.is_empty()).then(|| {
                format!(
                    "{} | {} | {} | {}",
                    data.name.trim(),
                    data.type_ref,
                    initial_value.trim(),
                    data.short_description.trim()
                )
            })
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn new_command_container_entry_row(
    entry: EditableContainerEntry,
    arguments: Entity<ArgumentListView>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<CommandContainerEntryRow> {
    let (kind, primary, secondary, tertiary) = match entry {
        EditableContainerEntry::ArgumentRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ArgumentRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::ParameterRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ParameterRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::ContainerRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ContainerRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::FixedValue {
            name,
            binary_value,
            size_in_bits,
        } => (
            CommandContainerEntryKind::FixedValue,
            name.unwrap_or_default(),
            binary_value,
            size_in_bits.to_string(),
        ),
    };
    let selected_index = CommandContainerEntryKind::VARIANTS
        .iter()
        .position(|candidate| candidate == &kind)
        .unwrap_or_default();
    cx.new(|cx| {
        let kind_select = cx.new(|cx| {
            SelectState::new(
                CommandContainerEntryKind::VARIANTS.to_vec(),
                Some(IndexPath::default().row(selected_index)),
                window,
                cx,
            )
        });
        let kind_subscription = cx.subscribe(
            &kind_select,
            |_, _, _: &SelectEvent<Vec<CommandContainerEntryKind>>, cx| cx.notify(),
        );
        let primary_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx).default_value(primary.clone());
            input.lsp.completion_provider = Some(Rc::new(EntryArgumentCompletionProvider {
                arguments: arguments.clone(),
                kind_select: kind_select.clone(),
            }));
            input
        });

        CommandContainerEntryRow {
            kind_select,
            primary_input,
            secondary_input: input(&secondary, false, window, cx),
            tertiary_input: input(&tertiary, false, window, cx),
            _subscriptions: vec![kind_subscription],
        }
    })
}

fn command_container_entry_rows_value(rows: &Entity<EntryListView>, cx: &App) -> String {
    let rows = rows.read(cx);
    rows.rows
        .iter()
        .enumerate()
        .filter_map(|(index, model)| {
            let entry = rows
                .editors
                .get(&index)
                .map(|editor| container_entry_data(editor, cx))
                .unwrap_or_else(|| model.clone());
            encode_editable_entry(&entry)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn encode_editable_entry(entry: &EditableContainerEntry) -> Option<String> {
    let (kind, primary, secondary, tertiary) = match entry {
        EditableContainerEntry::ArgumentRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ArgumentRef,
            reference.as_str(),
            offset.map(|value| value.to_string()).unwrap_or_default(),
            description.clone().unwrap_or_default(),
        ),
        EditableContainerEntry::ParameterRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ParameterRef,
            reference.as_str(),
            offset.map(|value| value.to_string()).unwrap_or_default(),
            description.clone().unwrap_or_default(),
        ),
        EditableContainerEntry::ContainerRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ContainerRef,
            reference.as_str(),
            offset.map(|value| value.to_string()).unwrap_or_default(),
            description.clone().unwrap_or_default(),
        ),
        EditableContainerEntry::FixedValue {
            name,
            binary_value,
            size_in_bits,
        } => {
            if *size_in_bits == i64::MIN {
                return None;
            }
            (
                CommandContainerEntryKind::FixedValue,
                name.as_deref().unwrap_or_default(),
                binary_value.clone(),
                size_in_bits.to_string(),
            )
        }
    };
    if primary.trim().is_empty() && kind != CommandContainerEntryKind::FixedValue {
        return None;
    }
    if kind == CommandContainerEntryKind::FixedValue && secondary.trim().is_empty() {
        return None;
    }
    Some(format!(
        "{kind} | {} | {} | {}",
        primary.trim(),
        secondary.trim(),
        tertiary.trim()
    ))
}

struct MetaCommandValues {
    name_or_ref: String,
    short_description: String,
    long_description: String,
    abstract_: bool,
    system_name: String,
    base_meta_command_ref: String,
    base_assignments: String,
    arguments: String,
    block_steps: String,
    command_container: CommandContainerValues,
    default_significance: SignificanceValues,
}

struct CommandContainerValues {
    present: bool,
    name: String,
    short_description: String,
    long_description: String,
    base_ref: String,
    entries: String,
}

impl CommandContainerValues {
    fn from_container(container: Option<&xtce::CommandContainerType>) -> Self {
        Self {
            present: container.is_some(),
            name: container
                .map(|value| value.name.clone())
                .unwrap_or_default(),
            short_description: container
                .and_then(|value| value.short_description.clone())
                .unwrap_or_default(),
            long_description: container
                .and_then(|value| value.long_description.clone())
                .unwrap_or_default(),
            base_ref: container
                .and_then(|value| value.base_container.as_ref())
                .map(|base| base.container_ref.clone())
                .unwrap_or_default(),
            entries: container
                .map(|value| encode_container_entries(&value.entry_list))
                .unwrap_or_default(),
        }
    }

    fn apply_to(&self, container: &mut Option<xtce::CommandContainerType>) {
        if !self.present {
            *container = None;
            return;
        }
        let container = container.get_or_insert_with(default_command_container);
        container.name = if self.name.is_empty() {
            "CommandContainer".to_owned()
        } else {
            self.name.clone()
        };
        container.short_description = optional_value(self.short_description.clone());
        container.long_description = optional_value(self.long_description.clone());
        if self.base_ref.is_empty() {
            container.base_container = None;
        } else {
            let base = container
                .base_container
                .get_or_insert_with(|| xtce::BaseContainerType {
                    container_ref: String::new(),
                    restriction_criteria: None,
                });
            base.container_ref.clone_from(&self.base_ref);
        }
        apply_container_entries(&mut container.entry_list, &self.entries);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SignificanceValues {
    consequence_level: ConsequenceLevelChoice,
    reason_for_warning: String,
    space_system_at_risk: String,
}

impl SignificanceValues {
    fn from_significance(significance: Option<&xtce::SignificanceType>) -> Self {
        match significance {
            Some(sig) => Self {
                consequence_level: match sig.consequence_level {
                    xtce::ConsequenceLevelType::Normal => ConsequenceLevelChoice::Normal,
                    xtce::ConsequenceLevelType::Vital => ConsequenceLevelChoice::Vital,
                    xtce::ConsequenceLevelType::Critical => ConsequenceLevelChoice::Critical,
                    xtce::ConsequenceLevelType::Forbidden => ConsequenceLevelChoice::Forbidden,
                    xtce::ConsequenceLevelType::User1 => ConsequenceLevelChoice::User1,
                    xtce::ConsequenceLevelType::User2 => ConsequenceLevelChoice::User2,
                },
                reason_for_warning: sig.reason_for_warning.clone().unwrap_or_default(),
                space_system_at_risk: sig.space_system_at_risk.clone().unwrap_or_default(),
            },
            None => Self {
                consequence_level: ConsequenceLevelChoice::None,
                reason_for_warning: String::new(),
                space_system_at_risk: String::new(),
            },
        }
    }

    fn apply_to(&self, significance: &mut Option<xtce::SignificanceType>) {
        if self.consequence_level == ConsequenceLevelChoice::None {
            *significance = None;
            return;
        }
        let consequence_level = match self.consequence_level {
            ConsequenceLevelChoice::Normal => xtce::ConsequenceLevelType::Normal,
            ConsequenceLevelChoice::Vital => xtce::ConsequenceLevelType::Vital,
            ConsequenceLevelChoice::Critical => xtce::ConsequenceLevelType::Critical,
            ConsequenceLevelChoice::Forbidden => xtce::ConsequenceLevelType::Forbidden,
            ConsequenceLevelChoice::User1 => xtce::ConsequenceLevelType::User1,
            ConsequenceLevelChoice::User2 => xtce::ConsequenceLevelType::User2,
            ConsequenceLevelChoice::None => unreachable!(),
        };
        *significance = Some(xtce::SignificanceType {
            space_system_at_risk: optional_value(self.space_system_at_risk.clone()),
            reason_for_warning: optional_value(self.reason_for_warning.clone()),
            consequence_level,
        });
    }
}

impl MetaCommandValues {
    fn from_command(command: Option<&xtce::MetaCommandSetTypeContent>) -> Self {
        let mut values = Self {
            name_or_ref: String::new(),
            short_description: String::new(),
            long_description: String::new(),
            abstract_: false,
            system_name: String::new(),
            base_meta_command_ref: String::new(),
            base_assignments: String::new(),
            arguments: String::new(),
            block_steps: String::new(),
            command_container: CommandContainerValues::from_container(None),
            default_significance: SignificanceValues::from_significance(None),
        };
        match command {
            Some(xtce::MetaCommandSetTypeContent::MetaCommand(command)) => {
                values.name_or_ref.clone_from(&command.name);
                values.short_description = command.short_description.clone().unwrap_or_default();
                values.long_description = command.long_description.clone().unwrap_or_default();
                values.abstract_ = command.abstract_;
                values.system_name = command.system_name.clone().unwrap_or_default();
                if let Some(base) = &command.base_meta_command {
                    values
                        .base_meta_command_ref
                        .clone_from(&base.meta_command_ref);
                    values.base_assignments =
                        encode_assignments(base.argument_assignment_list.as_ref());
                }
                values.arguments = encode_arguments(command.argument_list.as_ref());
                values.command_container =
                    CommandContainerValues::from_container(command.command_container.as_ref());
                values.default_significance =
                    SignificanceValues::from_significance(command.default_significance.as_ref());
            }
            Some(xtce::MetaCommandSetTypeContent::MetaCommandRef(reference)) => {
                values.name_or_ref.clone_from(reference);
            }
            Some(xtce::MetaCommandSetTypeContent::BlockMetaCommand(command)) => {
                values.name_or_ref.clone_from(&command.name);
                values.short_description = command.short_description.clone().unwrap_or_default();
                values.long_description = command.long_description.clone().unwrap_or_default();
                values.block_steps = encode_steps(&command.meta_command_step_list);
            }
            None => {}
        }
        values
    }

    fn apply_to(&self, command: &mut xtce::MetaCommandSetTypeContent) {
        match command {
            xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                command.name.clone_from(&self.name_or_ref);
                command.short_description = optional_value(self.short_description.clone());
                command.long_description = optional_value(self.long_description.clone());
                command.abstract_ = self.abstract_;
                command.system_name = optional_value(self.system_name.clone());
                apply_base(
                    &mut command.base_meta_command,
                    &self.base_meta_command_ref,
                    &self.base_assignments,
                );
                apply_arguments(&mut command.argument_list, &self.arguments);
                self.command_container
                    .apply_to(&mut command.command_container);
                self.default_significance
                    .apply_to(&mut command.default_significance);
            }
            xtce::MetaCommandSetTypeContent::MetaCommandRef(reference) => {
                reference.clone_from(&self.name_or_ref);
            }
            xtce::MetaCommandSetTypeContent::BlockMetaCommand(command) => {
                command.name.clone_from(&self.name_or_ref);
                command.short_description = optional_value(self.short_description.clone());
                command.long_description = optional_value(self.long_description.clone());
                apply_steps(&mut command.meta_command_step_list, &self.block_steps);
            }
        }
    }
}

fn kind(command: &xtce::MetaCommandSetTypeContent) -> MetaCommandKind {
    match command {
        xtce::MetaCommandSetTypeContent::MetaCommand(_) => MetaCommandKind::MetaCommand,
        xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => MetaCommandKind::MetaCommandRef,
        xtce::MetaCommandSetTypeContent::BlockMetaCommand(_) => MetaCommandKind::BlockMetaCommand,
    }
}

fn replace_kind(command: &mut xtce::MetaCommandSetTypeContent, selected: MetaCommandKind) {
    if kind(command) == selected {
        return;
    }
    *command = match selected {
        MetaCommandKind::MetaCommand => {
            xtce::MetaCommandSetTypeContent::MetaCommand(default_meta_command())
        }
        MetaCommandKind::MetaCommandRef => {
            xtce::MetaCommandSetTypeContent::MetaCommandRef(String::new())
        }
        MetaCommandKind::BlockMetaCommand => {
            xtce::MetaCommandSetTypeContent::BlockMetaCommand(xtce::BlockMetaCommandType {
                short_description: None,
                name: String::new(),
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                meta_command_step_list: xtce::MetaCommandStepListType {
                    meta_command_step: Vec::new(),
                },
            })
        }
    };
}

fn default_meta_command() -> xtce::MetaCommandType {
    xtce::MetaCommandType {
        short_description: None,
        name: String::new(),
        abstract_: false,
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        base_meta_command: None,
        system_name: None,
        argument_list: None,
        command_container: None,
        transmission_constraint_list: None,
        default_significance: None,
        context_significance_list: None,
        interlock: None,
        verifier_set: None,
        parameter_to_set_list: None,
        parameters_to_suspend_alarms_on_set: None,
    }
}

fn default_command_container() -> xtce::CommandContainerType {
    xtce::CommandContainerType {
        short_description: None,
        name: "CommandContainer".to_owned(),
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        default_rate_in_stream: None,
        rate_in_stream_set: None,
        binary_encoding: None,
        entry_list: xtce::CommandContainerEntryListType {
            content: Vec::new(),
        },
        base_container: None,
    }
}

#[derive(Clone)]
enum EditableContainerEntry {
    ArgumentRef {
        reference: String,
        offset: Option<i64>,
        description: Option<String>,
    },
    ParameterRef {
        reference: String,
        offset: Option<i64>,
        description: Option<String>,
    },
    ContainerRef {
        reference: String,
        offset: Option<i64>,
        description: Option<String>,
    },
    FixedValue {
        name: Option<String>,
        binary_value: String,
        size_in_bits: i64,
    },
}

fn command_entry_bit_positions(
    entries: &[EditableContainerEntry],
    arguments: &Entity<ArgumentListView>,
    cx: &App,
) -> Vec<Option<u64>> {
    let context = command_entry_context(arguments, cx);
    let inherited_count = context.inherited_entries.len();
    let mut all_entries = context.inherited_entries;
    all_entries.extend_from_slice(entries);
    command_entry_bit_positions_from_sizes(&all_entries, &context.argument_sizes)
        .into_iter()
        .skip(inherited_count)
        .collect()
}

struct CommandEntryContext {
    argument_sizes: HashMap<String, u64>,
    inherited_entries: Vec<EditableContainerEntry>,
    inherited_sources: Vec<(String, Vec<EditableContainerEntry>)>,
}

fn command_entry_context(arguments: &Entity<ArgumentListView>, cx: &App) -> CommandEntryContext {
    let arguments = arguments.read(cx);
    let context = arguments.context.borrow();
    let mut argument_sizes = context.active_argument_sizes();
    argument_sizes.extend(
        arguments
            .rows
            .iter()
            .enumerate()
            .filter_map(|(index, model)| {
                let argument = arguments
                    .editors
                    .get(&index)
                    .map_or_else(|| model.clone(), |editor| command_argument_data(editor, cx));
                let size = context
                    .type_sizes
                    .get(&argument.type_ref)
                    .or_else(|| {
                        context.type_sizes.get(
                            argument
                                .type_ref
                                .rsplit('/')
                                .next()
                                .unwrap_or(&argument.type_ref),
                        )
                    })
                    .copied();
                size.map(|size| (argument.name, size))
            })
            .collect::<HashMap<_, _>>(),
    );
    CommandEntryContext {
        argument_sizes,
        inherited_entries: context.active_container_entries(),
        inherited_sources: context.active_container_entries_with_sources(),
    }
}

fn command_entry_bit_positions_from_sizes(
    entries: &[EditableContainerEntry],
    argument_sizes: &HashMap<String, u64>,
) -> Vec<Option<u64>> {
    let mut cursor = Some(0_u64);
    entries
        .iter()
        .map(|entry| {
            let (offset, size) = match entry {
                EditableContainerEntry::ArgumentRef {
                    reference, offset, ..
                } => (
                    offset
                        .unwrap_or_default()
                        .try_into()
                        .ok()
                        .and_then(|offset: u64| {
                            cursor.and_then(|cursor| cursor.checked_add(offset))
                        }),
                    argument_sizes.get(reference).copied(),
                ),
                EditableContainerEntry::FixedValue { size_in_bits, .. } => (
                    cursor,
                    u64::try_from(*size_in_bits).ok().filter(|size| *size > 0),
                ),
                EditableContainerEntry::ParameterRef { .. }
                | EditableContainerEntry::ContainerRef { .. } => (None, None),
            };
            cursor = offset.and_then(|offset| size.and_then(|size| offset.checked_add(size)));
            offset.filter(|_| size.is_some())
        })
        .collect()
}

const COMMAND_LAYOUT_BYTES_PER_ROW: usize = 8;
const COMMAND_LAYOUT_BIT_WIDTH: f32 = 11.;

struct CommandPacketField {
    label: String,
    start_bit: u64,
    size_bits: u64,
    inherited: bool,
    source: String,
}

struct CommandPacketLayout {
    fields: Vec<CommandPacketField>,
    unresolved: Vec<String>,
    total_bits: u64,
}

fn command_packet_layout(
    entries: &[EditableContainerEntry],
    arguments: &Entity<ArgumentListView>,
    current_source: &str,
    cx: &App,
) -> CommandPacketLayout {
    let context = command_entry_context(arguments, cx);
    command_packet_layout_from_sizes(
        &context.inherited_sources,
        entries,
        &context.argument_sizes,
        current_source,
    )
}

fn command_packet_layout_from_sizes(
    inherited_entries: &[(String, Vec<EditableContainerEntry>)],
    entries: &[EditableContainerEntry],
    argument_sizes: &HashMap<String, u64>,
    current_source: &str,
) -> CommandPacketLayout {
    let mut layout = CommandPacketLayout {
        fields: Vec::new(),
        unresolved: Vec::new(),
        total_bits: 0,
    };
    let mut cursor = Some(0_u64);
    for (source, entries) in inherited_entries {
        append_command_packet_entries(
            entries,
            true,
            source,
            argument_sizes,
            &mut cursor,
            &mut layout,
        );
    }
    append_command_packet_entries(
        entries,
        false,
        current_source,
        argument_sizes,
        &mut cursor,
        &mut layout,
    );
    layout.total_bits = cursor.unwrap_or_else(|| {
        layout
            .fields
            .iter()
            .map(|field| field.start_bit + field.size_bits)
            .max()
            .unwrap_or_default()
    });
    layout
}

fn append_command_packet_entries(
    entries: &[EditableContainerEntry],
    inherited: bool,
    source: &str,
    argument_sizes: &HashMap<String, u64>,
    cursor: &mut Option<u64>,
    layout: &mut CommandPacketLayout,
) {
    for entry in entries {
        let (label, start, size) = match entry {
            EditableContainerEntry::ArgumentRef {
                reference, offset, ..
            } => {
                let offset = u64::try_from(offset.unwrap_or_default()).ok();
                (
                    reference.clone(),
                    cursor.and_then(|cursor| offset.and_then(|offset| cursor.checked_add(offset))),
                    argument_sizes.get(reference).copied(),
                )
            }
            EditableContainerEntry::FixedValue {
                name, size_in_bits, ..
            } => (
                name.clone().unwrap_or_else(|| "Fixed value".to_owned()),
                *cursor,
                u64::try_from(*size_in_bits).ok().filter(|size| *size > 0),
            ),
            EditableContainerEntry::ParameterRef { reference, .. } => {
                layout
                    .unresolved
                    .push(format!("{reference}: parameter size is unknown"));
                *cursor = None;
                continue;
            }
            EditableContainerEntry::ContainerRef { reference, .. } => {
                layout
                    .unresolved
                    .push(format!("{reference}: container size is unknown"));
                *cursor = None;
                continue;
            }
        };
        let Some((start, size)) = start.zip(size) else {
            layout
                .unresolved
                .push(format!("{label}: size or offset is unknown"));
            *cursor = None;
            continue;
        };
        let Some(end) = start.checked_add(size) else {
            layout
                .unresolved
                .push(format!("{label}: range is too large"));
            *cursor = None;
            continue;
        };
        layout.fields.push(CommandPacketField {
            label,
            start_bit: start,
            size_bits: size,
            inherited,
            source: source.to_owned(),
        });
        *cursor = Some(end);
    }
}

fn render_command_packet_layout(layout: &CommandPacketLayout, cx: &App) -> Div {
    let bits_per_row = (COMMAND_LAYOUT_BYTES_PER_ROW * 8) as u64;
    let row_count = usize::try_from(layout.total_bits.div_ceil(bits_per_row)).unwrap_or_default();
    let mut content = v_flex().w_full().gap_2().child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child("Fixed-size command entries · inherited fields use a lighter shade"),
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
                .child("No fixed-size command fields can be resolved."),
        );
    } else {
        content = content.child(
            div()
                .id("command-packet-layout-scroll")
                .w_full()
                .overflow_x_scroll()
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .child(
                    v_flex()
                        .min_w(px(68. + COMMAND_LAYOUT_BIT_WIDTH * bits_per_row as f32))
                        .gap_1()
                        .child(
                            h_flex()
                                .ml(px(68.))
                                .children((0..COMMAND_LAYOUT_BYTES_PER_ROW).map(|byte| {
                                    div()
                                        .w(px(COMMAND_LAYOUT_BIT_WIDTH * 8.))
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
                                let start = field.start_bit.max(row_start);
                                let end = (field.start_bit + field.size_bits).min(row_end);
                                if start > cursor {
                                    segments.push(command_packet_segment(
                                        start - cursor,
                                        "Unused".to_owned(),
                                        false,
                                        false,
                                        format!("command-layout-gap-{row}-{cursor}"),
                                        format!("Unused\nBit offset: {cursor}\nSize: {} bits", start - cursor),
                                        cx,
                                    ));
                                }
                                let label = if field.start_bit < row_start {
                                    format!("… {}", field.label)
                                } else if field.size_bits % 8 == 0 {
                                    format!("{} ({} B)", field.label, field.size_bits / 8)
                                } else {
                                    format!("{} ({} b)", field.label, field.size_bits)
                                };
                                segments.push(command_packet_segment(
                                    end - start,
                                    label,
                                    true,
                                    field.inherited,
                                    format!("command-layout-field-{row}-{}", field.start_bit),
                                    format!(
                                        "{}\nBit offset: {} (byte 0x{:04X}, bit {})\nSize: {} bits\nSource: {}",
                                        field.label,
                                        field.start_bit,
                                        field.start_bit / 8,
                                        field.start_bit % 8,
                                        field.size_bits,
                                        field.source
                                    ),
                                    cx,
                                ));
                                cursor = cursor.max(end);
                            }
                            if cursor < row_end {
                                segments.push(command_packet_segment(
                                    row_end - cursor,
                                    "Unused".to_owned(),
                                    false,
                                    false,
                                    format!("command-layout-gap-{row}-{cursor}"),
                                    format!("Unused\nBit offset: {cursor}\nSize: {} bits", row_end - cursor),
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

fn command_packet_segment(
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
        .w(px(bits as f32 * COMMAND_LAYOUT_BIT_WIDTH))
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

fn encode_container_entries(list: &xtce::CommandContainerEntryListType) -> String {
    list.content
        .iter()
        .filter_map(|entry| match entry {
            xtce::CommandContainerEntryListTypeContent::ArgumentRefEntry(entry) => Some(format!(
                "ArgumentRefEntry | {} | {} | {}",
                entry.argument_ref,
                fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                entry.short_description.as_deref().unwrap_or_default()
            )),
            xtce::CommandContainerEntryListTypeContent::ParameterRefEntry(entry) => Some(format!(
                "ParameterRefEntry | {} | {} | {}",
                entry.parameter_ref,
                fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                entry.short_description.as_deref().unwrap_or_default()
            )),
            xtce::CommandContainerEntryListTypeContent::ContainerRefEntry(entry) => Some(format!(
                "ContainerRefEntry | {} | {} | {}",
                entry.container_ref,
                fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                entry.short_description.as_deref().unwrap_or_default()
            )),
            xtce::CommandContainerEntryListTypeContent::FixedValueEntry(entry) => Some(format!(
                "FixedValueEntry | {} | {} | {}",
                entry.name.as_deref().unwrap_or_default(),
                entry.binary_value,
                entry.size_in_bits
            )),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn fixed_entry_offset(location: Option<&xtce::ArgumentLocationInContainerInBitsType>) -> String {
    match location.map(|location| &location.content) {
        Some(xtce::ArgumentLocationInContainerInBitsTypeContent::FixedValue(value)) => {
            value.to_string()
        }
        _ => String::new(),
    }
}

fn decode_container_entries(value: &str) -> Vec<EditableContainerEntry> {
    value
        .lines()
        .filter_map(|line| {
            let fields = line.split('|').map(str::trim).collect::<Vec<_>>();
            match fields.first().copied()? {
                "ArgumentRefEntry" => Some(EditableContainerEntry::ArgumentRef {
                    reference: nonempty_field(&fields, 1)?,
                    offset: numeric_field(&fields, 2),
                    description: optional_field(&fields, 3),
                }),
                "ParameterRefEntry" => Some(EditableContainerEntry::ParameterRef {
                    reference: nonempty_field(&fields, 1)?,
                    offset: numeric_field(&fields, 2),
                    description: optional_field(&fields, 3),
                }),
                "ContainerRefEntry" => Some(EditableContainerEntry::ContainerRef {
                    reference: nonempty_field(&fields, 1)?,
                    offset: numeric_field(&fields, 2),
                    description: optional_field(&fields, 3),
                }),
                "FixedValueEntry" => Some(EditableContainerEntry::FixedValue {
                    name: optional_field(&fields, 1),
                    binary_value: nonempty_field(&fields, 2)?,
                    size_in_bits: fields.get(3)?.parse().ok()?,
                }),
                _ => None,
            }
        })
        .collect()
}

fn nonempty_field(fields: &[&str], index: usize) -> Option<String> {
    let value = fields.get(index)?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn optional_field(fields: &[&str], index: usize) -> Option<String> {
    fields
        .get(index)
        .and_then(|value| optional_value(value.trim().to_owned()))
}

fn numeric_field(fields: &[&str], index: usize) -> Option<i64> {
    fields.get(index).and_then(|value| value.parse().ok())
}

fn apply_container_entries(list: &mut xtce::CommandContainerEntryListType, value: &str) {
    let mut argument_entries = VecDeque::new();
    let mut parameter_entries = VecDeque::new();
    let mut container_entries = VecDeque::new();
    let mut fixed_entries = VecDeque::new();
    let mut unsupported_entries = Vec::new();

    for entry in std::mem::take(&mut list.content) {
        match entry {
            xtce::CommandContainerEntryListTypeContent::ArgumentRefEntry(entry) => {
                argument_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ParameterRefEntry(entry) => {
                parameter_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ContainerRefEntry(entry) => {
                container_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::FixedValueEntry(entry) => {
                fixed_entries.push_back(entry);
            }
            entry => unsupported_entries.push(entry),
        }
    }

    list.content =
        decode_container_entries(value)
            .into_iter()
            .map(|entry| match entry {
                EditableContainerEntry::ArgumentRef {
                    reference,
                    offset,
                    description,
                } => {
                    let mut entry = argument_entries.pop_front().unwrap_or(
                        xtce::ArgumentArgumentRefEntryType {
                            short_description: None,
                            argument_ref: String::new(),
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                        },
                    );
                    entry.argument_ref = reference;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::ArgumentRefEntry(entry)
                }
                EditableContainerEntry::ParameterRef {
                    reference,
                    offset,
                    description,
                } => {
                    let mut entry = parameter_entries.pop_front().unwrap_or(
                        xtce::ArgumentParameterRefEntryType {
                            short_description: None,
                            parameter_ref: String::new(),
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                        },
                    );
                    entry.parameter_ref = reference;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::ParameterRefEntry(entry)
                }
                EditableContainerEntry::ContainerRef {
                    reference,
                    offset,
                    description,
                } => {
                    let mut entry = container_entries.pop_front().unwrap_or(
                        xtce::ArgumentContainerRefEntryType {
                            short_description: None,
                            container_ref: String::new(),
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                        },
                    );
                    entry.container_ref = reference;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::ContainerRefEntry(entry)
                }
                EditableContainerEntry::FixedValue {
                    name,
                    binary_value,
                    size_in_bits,
                } => {
                    let mut entry =
                        fixed_entries
                            .pop_front()
                            .unwrap_or(xtce::ArgumentFixedValueEntryType {
                                short_description: None,
                                name: None,
                                binary_value: String::new(),
                                size_in_bits: 0,
                                location_in_container_in_bits: None,
                                repeat_entry: None,
                                include_condition: None,
                                ancillary_data_set: None,
                            });
                    entry.name = name;
                    entry.binary_value = binary_value;
                    entry.size_in_bits = size_in_bits;
                    xtce::CommandContainerEntryListTypeContent::FixedValueEntry(entry)
                }
            })
            .chain(unsupported_entries)
            .collect();
}

fn apply_fixed_entry_offset(
    location: &mut Option<xtce::ArgumentLocationInContainerInBitsType>,
    offset: Option<i64>,
) {
    *location = offset.map(|offset| xtce::ArgumentLocationInContainerInBitsType {
        reference_location: xtce::ArgumentLocationInContainerInBitsType::default_reference_location(
        ),
        content: xtce::ArgumentLocationInContainerInBitsTypeContent::FixedValue(offset),
    });
}

fn encode_assignments(list: Option<&xtce::ArgumentAssignmentListType>) -> String {
    list.into_iter()
        .flat_map(|list| &list.argument_assignment)
        .map(|assignment| {
            format!(
                "{} | {}",
                assignment.argument_name, assignment.argument_value
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ComparisonOperatorChoice {
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
impl_select_item!(ComparisonOperatorChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum SuspendableChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(SuspendableChoice);

pub(super) struct VerifierListForm {
    title: &'static str,
    add_label: &'static str,
    rows: Vec<Entity<VerifierRowForm>>,
}

struct VerifierRowForm {
    parameter: Entity<InputState>,
    operator: Entity<SelectState<Vec<ComparisonOperatorChoice>>>,
    value: Entity<InputState>,
    time_to_stop: Entity<InputState>,
}

impl VerifierListForm {
    pub(super) fn new(
        title: &'static str,
        add_label: &'static str,
        models: Vec<VerifierModel>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        cx.new(move |cx| Self {
            title,
            add_label,
            rows: verifier_entities(models, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        models: Vec<VerifierModel>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = verifier_entities(models, window, cx);
        cx.notify();
    }

    pub(super) fn to_execution_verifiers(&self, cx: &App) -> Vec<xtce::ExecutionVerifierType> {
        self.rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let param = value(&row.parameter, cx).trim().to_owned();
                if param.is_empty() {
                    return None;
                }
                let op = row
                    .operator
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or(ComparisonOperatorChoice::Equal);
                let op_str = match op {
                    ComparisonOperatorChoice::Equal => "==",
                    ComparisonOperatorChoice::NotEqual => "!=",
                    ComparisonOperatorChoice::Less => "<",
                    ComparisonOperatorChoice::LessOrEqual => "<=",
                    ComparisonOperatorChoice::Greater => ">",
                    ComparisonOperatorChoice::GreaterOrEqual => ">=",
                };
                let val = value(&row.value, cx);
                let time_to_stop = value(&row.time_to_stop, cx).trim().to_owned();

                let mut content = vec![xtce::ExecutionVerifierTypeContent::Comparison(
                    xtce::ComparisonType {
                        parameter_ref: param,
                        instance: 0,
                        use_calibrated_value: true,
                        comparison_operator: op_str.to_owned(),
                        value: val,
                    },
                )];

                if !time_to_stop.is_empty() {
                    content.push(xtce::ExecutionVerifierTypeContent::CheckWindow(
                        xtce::CheckWindowType {
                            time_to_start_checking: None,
                            time_to_stop_checking: time_to_stop,
                            time_window_is_relative_to:
                                xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
                        },
                    ));
                }

                Some(xtce::ExecutionVerifierType {
                    short_description: None,
                    name: None,
                    content,
                })
            })
            .collect()
    }

    pub(super) fn to_complete_verifiers(&self, cx: &App) -> Vec<xtce::CompleteVerifierType> {
        self.rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let param = value(&row.parameter, cx).trim().to_owned();
                if param.is_empty() {
                    return None;
                }
                let op = row
                    .operator
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or(ComparisonOperatorChoice::Equal);
                let op_str = match op {
                    ComparisonOperatorChoice::Equal => "==",
                    ComparisonOperatorChoice::NotEqual => "!=",
                    ComparisonOperatorChoice::Less => "<",
                    ComparisonOperatorChoice::LessOrEqual => "<=",
                    ComparisonOperatorChoice::Greater => ">",
                    ComparisonOperatorChoice::GreaterOrEqual => ">=",
                };
                let val = value(&row.value, cx);
                let time_to_stop = value(&row.time_to_stop, cx).trim().to_owned();

                let mut content = vec![xtce::CompleteVerifierTypeContent::Comparison(
                    xtce::ComparisonType {
                        parameter_ref: param,
                        instance: 0,
                        use_calibrated_value: true,
                        comparison_operator: op_str.to_owned(),
                        value: val,
                    },
                )];

                if !time_to_stop.is_empty() {
                    content.push(xtce::CompleteVerifierTypeContent::CheckWindow(
                        xtce::CheckWindowType {
                            time_to_start_checking: None,
                            time_to_stop_checking: time_to_stop,
                            time_window_is_relative_to:
                                xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
                        },
                    ));
                }

                Some(xtce::CompleteVerifierType {
                    short_description: None,
                    name: None,
                    content,
                })
            })
            .collect()
    }
}

pub(super) struct VerifierModel {
    parameter: String,
    operator: ComparisonOperatorChoice,
    value: String,
    time_to_stop: String,
}

fn execution_verifier_models(verifiers: &[xtce::ExecutionVerifierType]) -> Vec<VerifierModel> {
    verifiers
        .iter()
        .map(|v| {
            let mut param = String::new();
            let mut op = ComparisonOperatorChoice::Equal;
            let mut val = String::new();
            let mut stop = String::new();

            for item in &v.content {
                match item {
                    xtce::ExecutionVerifierTypeContent::Comparison(c) => {
                        param = c.parameter_ref.clone();
                        op = operator_choice_from_str(&c.comparison_operator);
                        val = c.value.clone();
                    }
                    xtce::ExecutionVerifierTypeContent::CheckWindow(w) => {
                        stop = w.time_to_stop_checking.clone();
                    }
                    _ => {}
                }
            }

            VerifierModel {
                parameter: param,
                operator: op,
                value: val,
                time_to_stop: stop,
            }
        })
        .collect()
}

fn complete_verifier_models(verifiers: &[xtce::CompleteVerifierType]) -> Vec<VerifierModel> {
    verifiers
        .iter()
        .map(|v| {
            let mut param = String::new();
            let mut op = ComparisonOperatorChoice::Equal;
            let mut val = String::new();
            let mut stop = String::new();

            for item in &v.content {
                match item {
                    xtce::CompleteVerifierTypeContent::Comparison(c) => {
                        param = c.parameter_ref.clone();
                        op = operator_choice_from_str(&c.comparison_operator);
                        val = c.value.clone();
                    }
                    xtce::CompleteVerifierTypeContent::CheckWindow(w) => {
                        stop = w.time_to_stop_checking.clone();
                    }
                    _ => {}
                }
            }

            VerifierModel {
                parameter: param,
                operator: op,
                value: val,
                time_to_stop: stop,
            }
        })
        .collect()
}

fn verifier_entities(
    models: Vec<VerifierModel>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<VerifierRowForm>> {
    models
        .into_iter()
        .map(|model| verifier_entity(model, window, cx))
        .collect()
}

fn verifier_entity(
    model: VerifierModel,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<VerifierRowForm> {
    let parameter = input(&model.parameter, false, window, cx);
    let value_input = input(&model.value, false, window, cx);
    let time_to_stop = input(&model.time_to_stop, false, window, cx);
    let operator = select(ComparisonOperatorChoice::VARIANTS, model.operator, window, cx);

    cx.new(|_| VerifierRowForm {
        parameter,
        operator,
        value: value_input,
        time_to_stop,
    })
}

impl Render for VerifierListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child(self.title))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} verifier(s)", self.rows.len())),
                            ),
                    )
                    .child(
                        Button::new(format!("add-verifier-{}", self.title))
                            .small()
                            .icon(IconName::Plus)
                            .label(self.add_label)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(verifier_entity(
                                    VerifierModel {
                                        parameter: String::new(),
                                        operator: ComparisonOperatorChoice::Equal,
                                        value: String::new(),
                                        time_to_stop: String::new(),
                                    },
                                    window,
                                    cx,
                                ));
                                cx.notify();
                            })),
                    ),
            )
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                h_flex()
                    .w_full()
                    .p_3()
                    .gap_3()
                    .items_end()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(div().flex_1().child(field(
                        "Parameter",
                        "",
                        &row_read.parameter,
                        cx,
                    )))
                    .child(div().w(px(100.)).child(select_field(
                        "Op",
                        "",
                        &row_read.operator,
                        cx,
                    )))
                    .child(div().flex_1().child(field(
                        "Value",
                        "",
                        &row_read.value,
                        cx,
                    )))
                    .child(div().w(px(140.)).child(field(
                        "Timeout (e.g. PT10S)",
                        "",
                        &row_read.time_to_stop,
                        cx,
                    )))
                    .child(
                        Button::new(format!("remove-verifier-{}-{index}", self.title))
                            .small()
                            .icon(IconName::Minus)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.rows.remove(index);
                                cx.notify();
                            })),
                    )
            }))
    }
}

struct TransmissionConstraintRowForm {
    parameter: Entity<InputState>,
    operator: Entity<SelectState<Vec<ComparisonOperatorChoice>>>,
    value: Entity<InputState>,
    time_out: Entity<InputState>,
    suspendable: Entity<SelectState<Vec<SuspendableChoice>>>,
}

pub(super) struct TransmissionConstraintListForm {
    rows: Vec<Entity<TransmissionConstraintRowForm>>,
}

impl TransmissionConstraintListForm {
    pub(super) fn new(
        list: Option<&xtce::TransmissionConstraintListType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let models = constraint_models(list);
        cx.new(move |cx| Self {
            rows: constraint_entities(models, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        list: Option<&xtce::TransmissionConstraintListType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = constraint_entities(constraint_models(list), window, cx);
        cx.notify();
    }

    pub(super) fn to_list(&self, cx: &App) -> Option<xtce::TransmissionConstraintListType> {
        let constraints = self
            .rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let parameter = value(&row.parameter, cx).trim().to_owned();
                if parameter.is_empty() {
                    return None;
                }
                let op = row
                    .operator
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or(ComparisonOperatorChoice::Equal);
                let op_str = match op {
                    ComparisonOperatorChoice::Equal => "==",
                    ComparisonOperatorChoice::NotEqual => "!=",
                    ComparisonOperatorChoice::Less => "<",
                    ComparisonOperatorChoice::LessOrEqual => "<=",
                    ComparisonOperatorChoice::Greater => ">",
                    ComparisonOperatorChoice::GreaterOrEqual => ">=",
                };
                let val = value(&row.value, cx);
                let time_out = optional_value(value(&row.time_out, cx));
                let suspendable = row
                    .suspendable
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or(SuspendableChoice::False)
                    == SuspendableChoice::True;

                Some(xtce::TransmissionConstraintType {
                    time_out,
                    suspendable,
                    content: Some(xtce::TransmissionConstraintTypeContent::Comparison(
                        xtce::ComparisonType {
                            parameter_ref: parameter,
                            instance: 0,
                            use_calibrated_value: true,
                            comparison_operator: op_str.to_owned(),
                            value: val,
                        },
                    )),
                })
            })
            .collect::<Vec<_>>();

        (!constraints.is_empty()).then_some(xtce::TransmissionConstraintListType {
            transmission_constraint: constraints,
        })
    }
}

struct ConstraintModel {
    parameter: String,
    operator: ComparisonOperatorChoice,
    value: String,
    time_out: String,
    suspendable: SuspendableChoice,
}

fn constraint_models(list: Option<&xtce::TransmissionConstraintListType>) -> Vec<ConstraintModel> {
    let Some(list) = list else {
        return Vec::new();
    };
    list.transmission_constraint
        .iter()
        .map(|tc| {
            let (param, op, val) = match &tc.content {
                Some(xtce::TransmissionConstraintTypeContent::Comparison(c)) => (
                    c.parameter_ref.clone(),
                    operator_choice_from_str(&c.comparison_operator),
                    c.value.clone(),
                ),
                _ => (String::new(), ComparisonOperatorChoice::Equal, String::new()),
            };
            ConstraintModel {
                parameter: param,
                operator: op,
                value: val,
                time_out: tc.time_out.clone().unwrap_or_default(),
                suspendable: if tc.suspendable {
                    SuspendableChoice::True
                } else {
                    SuspendableChoice::False
                },
            }
        })
        .collect()
}

fn operator_choice_from_str(op: &str) -> ComparisonOperatorChoice {
    match op.trim() {
        "!=" => ComparisonOperatorChoice::NotEqual,
        "<" => ComparisonOperatorChoice::Less,
        "<=" => ComparisonOperatorChoice::LessOrEqual,
        ">" => ComparisonOperatorChoice::Greater,
        ">=" => ComparisonOperatorChoice::GreaterOrEqual,
        _ => ComparisonOperatorChoice::Equal,
    }
}

fn constraint_entities(
    models: Vec<ConstraintModel>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<TransmissionConstraintRowForm>> {
    models
        .into_iter()
        .map(|model| constraint_entity(model, window, cx))
        .collect()
}

fn constraint_entity(
    model: ConstraintModel,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<TransmissionConstraintRowForm> {
    let parameter = input(&model.parameter, false, window, cx);
    let value_input = input(&model.value, false, window, cx);
    let time_out = input(&model.time_out, false, window, cx);
    let operator = select(ComparisonOperatorChoice::VARIANTS, model.operator, window, cx);
    let suspendable = select(SuspendableChoice::VARIANTS, model.suspendable, window, cx);

    cx.new(|_| TransmissionConstraintRowForm {
        parameter,
        operator,
        value: value_input,
        time_out,
        suspendable,
    })
}

impl Render for TransmissionConstraintListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Transmission constraints"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} constraint(s)", self.rows.len())),
                            ),
                    )
                    .child(
                        Button::new("add-transmission-constraint")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add constraint")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(constraint_entity(
                                    ConstraintModel {
                                        parameter: String::new(),
                                        operator: ComparisonOperatorChoice::Equal,
                                        value: String::new(),
                                        time_out: String::new(),
                                        suspendable: SuspendableChoice::False,
                                    },
                                    window,
                                    cx,
                                ));
                                cx.notify();
                            })),
                    ),
            )
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                v_flex()
                    .w_full()
                    .p_3()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .gap_3()
                            .items_end()
                            .child(div().flex_1().child(field(
                                "Parameter",
                                "",
                                &row_read.parameter,
                                cx,
                            )))
                            .child(div().w(px(120.)).child(select_field(
                                "Op",
                                "",
                                &row_read.operator,
                                cx,
                            )))
                            .child(div().flex_1().child(field(
                                "Value",
                                "",
                                &row_read.value,
                                cx,
                            )))
                            .child(
                                Button::new(format!("remove-transmission-constraint-{index}"))
                                    .small()
                                    .icon(IconName::Minus)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.rows.remove(index);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .items_end()
                            .child(div().flex_1().child(field(
                                "Timeout (e.g. PT5S)",
                                "",
                                &row_read.time_out,
                                cx,
                            )))
                            .child(div().w(px(140.)).child(select_field(
                                "Suspendable",
                                "",
                                &row_read.suspendable,
                                cx,
                            ))),
                    )
            }))
    }
}

fn decode_assignments(value: &str) -> Vec<xtce::ArgumentAssignmentType> {
    value
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(" | ")?;
            let name = name.trim();
            (!name.is_empty()).then(|| xtce::ArgumentAssignmentType {
                argument_name: name.to_owned(),
                argument_value: value.trim().to_owned(),
            })
        })
        .collect()
}

fn base_assignment_models(value: &str) -> Vec<BaseAssignmentData> {
    decode_assignments(value)
        .into_iter()
        .map(|assignment| BaseAssignmentData {
            name: assignment.argument_name,
            value: assignment.argument_value,
        })
        .collect()
}

fn assignment_rows_value(
    rows: &Entity<BaseAssignmentListView>,
    context: &Rc<RefCell<AssignmentContext>>,
    cx: &App,
) -> String {
    let rows = rows.read(cx);
    rows.rows
        .iter()
        .enumerate()
        .filter_map(|(index, model)| {
            let data = rows
                .editors
                .get(&index)
                .map(|editor| base_assignment_data(editor, cx))
                .unwrap_or_else(|| model.clone());
            let name = data.name;
            let value = data.value;
            (!name.trim().is_empty()
                && context
                    .borrow()
                    .is_complete_value(name.trim(), value.trim()))
            .then(|| format!("{} | {}", name.trim(), value.trim()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn apply_base(base: &mut Option<xtce::BaseMetaCommandType>, reference: &str, assignments: &str) {
    if reference.is_empty() {
        *base = None;
        return;
    }
    let base = base.get_or_insert_with(|| xtce::BaseMetaCommandType {
        meta_command_ref: String::new(),
        argument_assignment_list: None,
    });
    base.meta_command_ref = reference.to_owned();
    let assignments = decode_assignments(assignments);
    base.argument_assignment_list =
        (!assignments.is_empty()).then_some(xtce::ArgumentAssignmentListType {
            argument_assignment: assignments,
        });
}

fn encode_arguments(list: Option<&xtce::ArgumentListType>) -> String {
    list.into_iter()
        .flat_map(|list| &list.argument)
        .map(|argument| {
            format!(
                "{} | {} | {} | {}",
                argument.name,
                argument.argument_type_ref,
                argument.initial_value.as_deref().unwrap_or_default(),
                argument.short_description.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn apply_arguments(list: &mut Option<xtce::ArgumentListType>, value: &str) {
    let rows = value
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
    if rows.is_empty() {
        *list = None;
        return;
    }
    let mut existing = list
        .take()
        .into_iter()
        .flat_map(|list| list.argument)
        .collect::<Vec<_>>()
        .into_iter();
    let arguments = rows
        .into_iter()
        .map(
            |(name, argument_type_ref, initial_value, short_description)| {
                let mut argument = existing.next().unwrap_or(xtce::ArgumentType {
                    short_description: None,
                    name: String::new(),
                    argument_type_ref: String::new(),
                    initial_value: None,
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                });
                argument.name = name;
                argument.argument_type_ref = argument_type_ref;
                argument.initial_value = initial_value;
                argument.short_description = short_description;
                argument
            },
        )
        .collect();
    *list = Some(xtce::ArgumentListType {
        argument: arguments,
    });
}

fn encode_steps(list: &xtce::MetaCommandStepListType) -> String {
    list.meta_command_step
        .iter()
        .map(|step| {
            let assignments = step
                .argument_assignment_list
                .as_ref()
                .map(|list| {
                    list.argument_assignment
                        .iter()
                        .map(|value| format!("{}={}", value.argument_name, value.argument_value))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            format!("{} | {}", step.meta_command_ref, assignments)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn apply_steps(list: &mut xtce::MetaCommandStepListType, value: &str) {
    let steps = value
        .lines()
        .filter_map(|line| {
            let (reference, assignments) = line.split_once(" | ").unwrap_or((line, ""));
            let reference = reference.trim();
            if reference.is_empty() {
                return None;
            }
            let assignments = assignments
                .split(',')
                .filter_map(|assignment| {
                    let (name, value) = assignment.trim().split_once('=')?;
                    Some(xtce::ArgumentAssignmentType {
                        argument_name: name.trim().to_owned(),
                        argument_value: value.trim().to_owned(),
                    })
                })
                .collect::<Vec<_>>();
            Some(xtce::MetaCommandStepType {
                meta_command_ref: reference.to_owned(),
                argument_assignment_list: (!assignments.is_empty()).then_some(
                    xtce::ArgumentAssignmentListType {
                        argument_assignment: assignments,
                    },
                ),
            })
        })
        .collect();
    list.meta_command_step = steps;
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
    v_flex().w_full().child(
        gpui_component::form::v_form().child(
            gpui_component::form::field()
                .label(label)
                .required(required)
                .when(!required && !hint.is_empty(), |field| {
                    field.description(hint)
                })
                .child(Select::new(select).w_full()),
        ),
    )
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

fn input_with_context(
    value: &str,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    input(value, false, window, cx)
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        ArgumentSpec, AssignmentContext, CommandArguments, CommandContainerValues,
        ConsequenceLevelChoice, EditableContainerEntry, MetaCommandValues, SignificanceValues,
        ValueRule, apply_arguments, apply_container_entries, apply_steps,
        command_entry_bit_positions_from_sizes, command_packet_layout_from_sizes,
        decode_assignments, default_command_container, default_meta_command,
        matching_argument_names, swap_rows,
    };
    use std::collections::HashMap;

    #[test]
    fn argument_ref_suggestions_use_non_empty_matching_argument_names() {
        let names = vec![
            "mode".to_owned(),
            " target ".to_owned(),
            String::new(),
            "mode".to_owned(),
        ];

        assert_eq!(matching_argument_names(names, "MO"), vec!["mode"]);
    }

    #[test]
    fn moving_an_entry_changes_its_stored_order() {
        let mut rows = vec![
            EditableContainerEntry::ArgumentRef {
                reference: "first".to_owned(),
                offset: None,
                description: None,
            },
            EditableContainerEntry::ArgumentRef {
                reference: "second".to_owned(),
                offset: None,
                description: None,
            },
        ];

        assert!(swap_rows(&mut rows, 0, 1));
        assert!(matches!(
            &rows[0],
            EditableContainerEntry::ArgumentRef { reference, .. }
                if reference == "second"
        ));
        assert!(matches!(
            &rows[1],
            EditableContainerEntry::ArgumentRef { reference, .. }
                if reference == "first"
        ));
        assert!(!swap_rows(&mut rows, 1, 2));
    }

    #[test]
    fn command_entry_list_displays_bit_positions_instead_of_row_numbers() {
        let entries = vec![
            EditableContainerEntry::ArgumentRef {
                reference: "mode".to_owned(),
                offset: None,
                description: None,
            },
            EditableContainerEntry::FixedValue {
                name: Some("marker".to_owned()),
                binary_value: "1010".to_owned(),
                size_in_bits: 4,
            },
            EditableContainerEntry::ArgumentRef {
                reference: "value".to_owned(),
                offset: Some(4),
                description: None,
            },
        ];
        let sizes = HashMap::from([("mode".to_owned(), 8), ("value".to_owned(), 16)]);

        assert_eq!(
            command_entry_bit_positions_from_sizes(&entries, &sizes),
            vec![Some(0), Some(8), Some(16)]
        );
    }

    #[test]
    fn command_entry_positions_include_base_meta_command_containers() {
        let context = AssignmentContext {
            current_base: "Derived".to_owned(),
            commands: HashMap::from([
                (
                    "Base".to_owned(),
                    CommandArguments {
                        name: "Base".to_owned(),
                        base_ref: None,
                        arguments: Vec::new(),
                        container_entries: vec![EditableContainerEntry::FixedValue {
                            name: Some("base-header".to_owned()),
                            binary_value: "0".to_owned(),
                            size_in_bits: 8,
                        }],
                    },
                ),
                (
                    "Derived".to_owned(),
                    CommandArguments {
                        name: "Derived".to_owned(),
                        base_ref: Some("Base".to_owned()),
                        arguments: Vec::new(),
                        container_entries: vec![EditableContainerEntry::FixedValue {
                            name: Some("derived-header".to_owned()),
                            binary_value: "0".to_owned(),
                            size_in_bits: 4,
                        }],
                    },
                ),
            ]),
            type_rules: HashMap::new(),
            type_sizes: HashMap::new(),
            type_names: Vec::new(),
        };
        let mut entries = context.active_container_entries();
        entries.push(EditableContainerEntry::FixedValue {
            name: Some("current".to_owned()),
            binary_value: "0".to_owned(),
            size_in_bits: 16,
        });

        assert_eq!(
            command_entry_bit_positions_from_sizes(&entries, &HashMap::new()),
            vec![Some(0), Some(8), Some(12)]
        );
    }

    #[test]
    fn command_packet_layout_marks_base_meta_command_fields_as_inherited() {
        let inherited = vec![(
            "BaseCommand".to_owned(),
            vec![EditableContainerEntry::FixedValue {
                name: Some("header".to_owned()),
                binary_value: "0".to_owned(),
                size_in_bits: 8,
            }],
        )];
        let entries = vec![EditableContainerEntry::ArgumentRef {
            reference: "mode".to_owned(),
            offset: None,
            description: None,
        }];
        let layout = command_packet_layout_from_sizes(
            &inherited,
            &entries,
            &HashMap::from([("mode".to_owned(), 16)]),
            "CurrentCommand",
        );

        assert_eq!(layout.total_bits, 24);
        assert!(layout.fields[0].inherited);
        assert_eq!(layout.fields[0].source, "BaseCommand");
        assert_eq!(layout.fields[1].start_bit, 8);
        assert!(!layout.fields[1].inherited);
        assert_eq!(layout.fields[1].source, "CurrentCommand");
    }

    #[test]
    fn editing_arguments_preserves_unshown_argument_metadata() {
        let mut list = Some(xtce::ArgumentListType {
            argument: vec![xtce::ArgumentType {
                short_description: None,
                name: "old".to_owned(),
                argument_type_ref: "OldType".to_owned(),
                initial_value: None,
                long_description: Some("preserved".to_owned()),
                alias_set: None,
                ancillary_data_set: None,
            }],
        });

        apply_arguments(&mut list, "mode | ModeType | SAFE | Operating mode");

        let argument = &list.expect("argument list").argument[0];
        assert_eq!(argument.name, "mode");
        assert_eq!(argument.long_description.as_deref(), Some("preserved"));
    }

    #[test]
    fn block_steps_support_argument_assignments() {
        let mut list = xtce::MetaCommandStepListType {
            meta_command_step: Vec::new(),
        };

        apply_steps(&mut list, "Prepare | mode=SAFE, count=2");

        assert_eq!(list.meta_command_step[0].meta_command_ref, "Prepare");
        assert_eq!(
            list.meta_command_step[0]
                .argument_assignment_list
                .as_ref()
                .expect("assignments")
                .argument_assignment
                .len(),
            2
        );
    }

    #[test]
    fn base_argument_assignments_support_multiple_rows() {
        let assignments = decode_assignments("mode | SAFE\ncount | 2");

        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments[0].argument_name, "mode");
        assert_eq!(assignments[1].argument_value, "2");
    }

    #[test]
    fn inherited_argument_names_are_available_as_suggestions() {
        let context = AssignmentContext {
            current_base: "/Vehicle/Derived".to_owned(),
            commands: HashMap::from([
                (
                    "Base".to_owned(),
                    CommandArguments {
                        name: "Base".to_owned(),
                        base_ref: None,
                        arguments: vec![("mode".to_owned(), "ModeType".to_owned())],
                        container_entries: Vec::new(),
                    },
                ),
                (
                    "Derived".to_owned(),
                    CommandArguments {
                        name: "Derived".to_owned(),
                        base_ref: Some("Base".to_owned()),
                        arguments: vec![("count".to_owned(), "/Vehicle/CountType".to_owned())],
                        container_entries: Vec::new(),
                    },
                ),
            ]),
            type_rules: HashMap::from([
                (
                    "ModeType".to_owned(),
                    ValueRule::Enumerated(vec!["SAFE".to_owned(), "ACTIVE".to_owned()]),
                ),
                ("CountType".to_owned(), ValueRule::Integer),
            ]),
            type_sizes: HashMap::new(),
            type_names: vec!["ModeType".to_owned(), "CountType".to_owned()],
        };

        let names = context
            .active_arguments()
            .into_iter()
            .map(|ArgumentSpec { name, .. }| name)
            .collect::<Vec<_>>();

        assert_eq!(names, ["mode", "count"]);
        assert!(context.is_complete_value("mode", "SAFE"));
        assert!(!context.is_complete_value("mode", "UNKNOWN"));
        assert!(context.is_complete_value("count", "42"));
        assert!(!context.is_complete_value("count", "4.2"));
    }

    #[test]
    fn argument_initial_values_follow_the_referenced_type() {
        let context = AssignmentContext {
            current_base: String::new(),
            commands: HashMap::new(),
            type_rules: HashMap::from([
                ("CountType".to_owned(), ValueRule::Integer),
                (
                    "ModeType".to_owned(),
                    ValueRule::Enumerated(vec!["SAFE".to_owned(), "ACTIVE".to_owned()]),
                ),
                (
                    "EnabledType".to_owned(),
                    ValueRule::Boolean(vec!["true".to_owned(), "false".to_owned()]),
                ),
            ]),
            type_sizes: HashMap::new(),
            type_names: vec![
                "CountType".to_owned(),
                "ModeType".to_owned(),
                "EnabledType".to_owned(),
            ],
        };

        assert!(context.accepts_type_value("CountType", "42"));
        assert!(!context.accepts_type_value("CountType", "4.2"));
        assert!(context.accepts_type_value("/Commands/ModeType", "SAFE"));
        assert!(!context.is_complete_type_value("ModeType", "S"));
        assert_eq!(
            context.type_value_suggestions("EnabledType"),
            ["true", "false"]
        );
        assert!(context.is_complete_type_value("CountType", ""));
    }

    #[test]
    fn applying_meta_command_values_preserves_unshown_fields() {
        let mut command = xtce::MetaCommandSetTypeContent::MetaCommand(default_meta_command());
        let xtce::MetaCommandSetTypeContent::MetaCommand(value) = &mut command else {
            unreachable!()
        };
        value.alias_set = Some(xtce::AliasSetType { alias: Vec::new() });

        MetaCommandValues {
            name_or_ref: "Reset".to_owned(),
            short_description: String::new(),
            long_description: String::new(),
            abstract_: false,
            system_name: String::new(),
            base_meta_command_ref: String::new(),
            base_assignments: String::new(),
            arguments: String::new(),
            block_steps: String::new(),
            command_container: CommandContainerValues::from_container(None),
            default_significance: SignificanceValues::from_significance(None),
        }
        .apply_to(&mut command);

        let xtce::MetaCommandSetTypeContent::MetaCommand(value) = command else {
            panic!("expected MetaCommand")
        };
        assert_eq!(value.name, "Reset");
        assert!(value.alias_set.is_some());
    }

    #[test]
    fn meta_command_transmission_constraints_roundtrip() {
        use super::{ComparisonOperatorChoice, SuspendableChoice, constraint_models};

        let tc = xtce::TransmissionConstraintType {
            time_out: Some("PT5S".to_owned()),
            suspendable: true,
            content: Some(xtce::TransmissionConstraintTypeContent::Comparison(
                xtce::ComparisonType {
                    parameter_ref: "BUS_VOLTAGE".to_owned(),
                    instance: 0,
                    use_calibrated_value: true,
                    comparison_operator: ">=".to_owned(),
                    value: "28.0".to_owned(),
                },
            )),
        };
        let list = xtce::TransmissionConstraintListType {
            transmission_constraint: vec![tc],
        };

        let models = constraint_models(Some(&list));
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].parameter, "BUS_VOLTAGE");
        assert_eq!(models[0].operator, ComparisonOperatorChoice::GreaterOrEqual);
        assert_eq!(models[0].value, "28.0");
        assert_eq!(models[0].time_out, "PT5S");
        assert_eq!(models[0].suspendable, SuspendableChoice::True);
    }

    #[test]
    fn meta_command_verifiers_roundtrip() {
        use super::{
            ComparisonOperatorChoice, complete_verifier_models, execution_verifier_models,
        };

        let exec = xtce::ExecutionVerifierType {
            short_description: None,
            name: None,
            content: vec![
                xtce::ExecutionVerifierTypeContent::Comparison(xtce::ComparisonType {
                    parameter_ref: "EXEC_STATUS".to_owned(),
                    instance: 0,
                    use_calibrated_value: true,
                    comparison_operator: "==".to_owned(),
                    value: "RUNNING".to_owned(),
                }),
                xtce::ExecutionVerifierTypeContent::CheckWindow(xtce::CheckWindowType {
                    time_to_start_checking: None,
                    time_to_stop_checking: "PT5S".to_owned(),
                    time_window_is_relative_to:
                        xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
                }),
            ],
        };

        let comp = xtce::CompleteVerifierType {
            short_description: None,
            name: None,
            content: vec![
                xtce::CompleteVerifierTypeContent::Comparison(xtce::ComparisonType {
                    parameter_ref: "EXEC_STATUS".to_owned(),
                    instance: 0,
                    use_calibrated_value: true,
                    comparison_operator: "==".to_owned(),
                    value: "COMPLETED".to_owned(),
                }),
                xtce::CompleteVerifierTypeContent::CheckWindow(xtce::CheckWindowType {
                    time_to_start_checking: None,
                    time_to_stop_checking: "PT30S".to_owned(),
                    time_window_is_relative_to:
                        xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
                }),
            ],
        };

        let exec_models = execution_verifier_models(&[exec]);
        assert_eq!(exec_models.len(), 1);
        assert_eq!(exec_models[0].parameter, "EXEC_STATUS");
        assert_eq!(exec_models[0].operator, ComparisonOperatorChoice::Equal);
        assert_eq!(exec_models[0].value, "RUNNING");
        assert_eq!(exec_models[0].time_to_stop, "PT5S");

        let comp_models = complete_verifier_models(&[comp]);
        assert_eq!(comp_models.len(), 1);
        assert_eq!(comp_models[0].parameter, "EXEC_STATUS");
        assert_eq!(comp_models[0].operator, ComparisonOperatorChoice::Equal);
        assert_eq!(comp_models[0].value, "COMPLETED");
        assert_eq!(comp_models[0].time_to_stop, "PT30S");
    }

    #[test]
    fn command_container_supports_editable_entry_types() {
        let mut container = default_command_container();

        apply_container_entries(
            &mut container.entry_list,
            "ArgumentRefEntry | mode | 0 | Mode argument\n\
             ParameterRefEntry | status | 8 | Status parameter\n\
             ContainerRefEntry | Header | 16 | Header fields\n\
             FixedValueEntry | sync | 0x1ACF | 16",
        );

        assert_eq!(container.entry_list.content.len(), 4);
        let xtce::CommandContainerEntryListTypeContent::ArgumentRefEntry(argument) =
            &container.entry_list.content[0]
        else {
            panic!("expected ArgumentRefEntry")
        };
        assert_eq!(argument.argument_ref, "mode");
        assert_eq!(
            super::fixed_entry_offset(argument.location_in_container_in_bits.as_ref()),
            "0"
        );
        let xtce::CommandContainerEntryListTypeContent::FixedValueEntry(fixed) =
            &container.entry_list.content[3]
        else {
            panic!("expected FixedValueEntry")
        };
        assert_eq!(fixed.binary_value, "0x1ACF");
        assert_eq!(fixed.size_in_bits, 16);
    }

    #[test]
    fn blank_entry_offset_removes_location_in_container() {
        let mut container = default_command_container();

        apply_container_entries(
            &mut container.entry_list,
            "ArgumentRefEntry | mode | 8 | Mode argument",
        );
        apply_container_entries(
            &mut container.entry_list,
            "ArgumentRefEntry | mode | | Mode argument",
        );

        let xtce::CommandContainerEntryListTypeContent::ArgumentRefEntry(argument) =
            &container.entry_list.content[0]
        else {
            panic!("expected ArgumentRefEntry")
        };
        assert!(argument.location_in_container_in_bits.is_none());
    }

    #[test]
    fn editing_entries_preserves_entry_types_not_exposed_by_the_form() {
        let mut container = default_command_container();
        container.entry_list.content.push(
            xtce::CommandContainerEntryListTypeContent::StreamSegmentEntry(
                xtce::ArgumentStreamSegmentEntryType {
                    short_description: None,
                    stream_ref: "CommandStream".to_owned(),
                    order: None,
                    size_in_bits: 32,
                    location_in_container_in_bits: None,
                    repeat_entry: None,
                    include_condition: None,
                    ancillary_data_set: None,
                },
            ),
        );

        apply_container_entries(
            &mut container.entry_list,
            "ArgumentRefEntry | mode | 0 | Mode argument",
        );

        assert_eq!(container.entry_list.content.len(), 2);
        assert!(matches!(
            &container.entry_list.content[1],
            xtce::CommandContainerEntryListTypeContent::StreamSegmentEntry(entry)
                if entry.stream_ref == "CommandStream"
        ));
    }

    #[test]
    fn editing_command_container_preserves_unshown_metadata() {
        let mut container = default_command_container();
        container.alias_set = Some(xtce::AliasSetType { alias: Vec::new() });
        container.base_container = Some(xtce::BaseContainerType {
            container_ref: "OldBase".to_owned(),
            restriction_criteria: None,
        });
        let mut value = Some(container);

        CommandContainerValues {
            present: true,
            name: "Packet".to_owned(),
            short_description: "Command packet".to_owned(),
            long_description: String::new(),
            base_ref: "NewBase".to_owned(),
            entries: String::new(),
        }
        .apply_to(&mut value);

        let container = value.expect("command container");
        assert_eq!(container.name, "Packet");
        assert_eq!(
            container.base_container.expect("base").container_ref,
            "NewBase"
        );
        assert!(container.alias_set.is_some());
    }
}
