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
    ActiveTheme, Disableable, IconName, IndexPath, Sizable, StyledExt, WindowExt,
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
    alias_set::AliasSetForm,
    ancillary_data_set::AncillaryDataSetForm,
    container_binary_encoding::ContainerBinaryEncodingForm,
    container_rate::ContainerRateForm,
    context_significance::ContextSignificanceListForm,
    dynamic_value::DynamicValueForm,
    field, impl_select_item,
    input_algorithm::InputAlgorithmForm,
    message::{MessageCriteriaForm, MessageCriteriaRef},
    optional_value,
    rpn_operation::{RpnOperationEntry, RpnOperationForm},
};
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
    #[strum(serialize = "ParameterSegmentRefEntry")]
    ParameterSegmentRef,
    #[strum(serialize = "ContainerRefEntry")]
    ContainerRef,
    #[strum(serialize = "ContainerSegmentRefEntry")]
    ContainerSegmentRef,
    #[strum(serialize = "StreamSegmentEntry")]
    StreamSegment,
    #[strum(serialize = "ArrayParameterRefEntry")]
    ArrayParameterRef,
    #[strum(serialize = "IndirectParameterRefEntry")]
    IndirectParameterRef,
    #[strum(serialize = "ArrayArgumentRefEntry")]
    ArrayArgumentRef,
    #[strum(serialize = "FixedValueEntry")]
    FixedValue,
}
impl_select_item!(CommandContainerEntryKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CommandParameterValueChoice {
    #[strum(serialize = "Calibrated value")]
    Calibrated,
    #[strum(serialize = "Raw value")]
    Raw,
}
impl_select_item!(CommandParameterValueChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum LastArrayEntryChoice {
    No,
    Yes,
}
impl_select_item!(LastArrayEntryChoice);

fn command_entry_placeholder(kind: CommandContainerEntryKind) -> &'static str {
    match kind {
        CommandContainerEntryKind::ArgumentRef | CommandContainerEntryKind::ArrayArgumentRef => {
            "Select an argument..."
        }
        CommandContainerEntryKind::ParameterRef
        | CommandContainerEntryKind::ParameterSegmentRef => "Select a parameter...",
        CommandContainerEntryKind::ArrayParameterRef => "Select an array parameter...",
        CommandContainerEntryKind::IndirectParameterRef => {
            "Select the parameter that names the target..."
        }
        CommandContainerEntryKind::ContainerRef => "Select a container...",
        CommandContainerEntryKind::ContainerSegmentRef => "Select a container...",
        CommandContainerEntryKind::StreamSegment => "Select a stream...",
        CommandContainerEntryKind::FixedValue => "Optional entry name",
    }
}

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
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
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
    container_alias_set: AliasSetForm,
    container_ancillary_data_set: AncillaryDataSetForm,
    container_rate: Entity<ContainerRateForm>,
    container_binary_encoding: Entity<ContainerBinaryEncodingForm>,
    entry_list: Entity<EntryListView>,
    consequence_level_select: Entity<SelectState<Vec<ConsequenceLevelChoice>>>,
    reason_for_warning_input: Entity<InputState>,
    space_system_at_risk_input: Entity<InputState>,
    context_significance_list: Entity<ContextSignificanceListForm>,
    transmission_constraints: Entity<TransmissionConstraintListForm>,
    verifiers: Entity<VerifierListForm>,
    interlock_verification_select: Entity<SelectState<Vec<VerificationToWaitForChoice>>>,
    interlock_scope_input: Entity<InputState>,
    interlock_progress_input: Entity<InputState>,
    interlock_suspendable_select: Entity<SelectState<Vec<SuspendableChoice>>>,
    parameter_to_set_list: Entity<ParameterToSetListForm>,
    parameters_to_suspend_alarms: Entity<ParametersToSuspendAlarmsOnSetForm>,
    documentation_open: bool,
    metadata_open: bool,
    inheritance_open: bool,
    identification_open: bool,
    significance_open: bool,
    transmission_constraints_open: bool,
    verifiers_open: bool,
    interlock_open: bool,
    parameter_to_set_list_open: bool,
    parameters_to_suspend_alarms_open: bool,
    container_details_open: bool,
    container_metadata_open: bool,
    container_rate_open: bool,
    container_binary_encoding_open: bool,
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
        let alias_set = AliasSetForm::new(
            command.and_then(|command| match command {
                xtce::MetaCommandSetTypeContent::MetaCommand(command) => command.alias_set.as_ref(),
                xtce::MetaCommandSetTypeContent::BlockMetaCommand(command) => {
                    command.alias_set.as_ref()
                }
                xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => None,
            }),
            window,
            cx,
        );
        let ancillary_data_set = AncillaryDataSetForm::new(
            command.and_then(|command| match command {
                xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                    command.ancillary_data_set.as_ref()
                }
                xtce::MetaCommandSetTypeContent::BlockMetaCommand(command) => {
                    command.ancillary_data_set.as_ref()
                }
                xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => None,
            }),
            window,
            cx,
        );
        let command_container = command.and_then(|command| match command {
            xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                command.command_container.as_ref()
            }
            _ => None,
        });
        let container_alias_set = AliasSetForm::new(
            command_container.and_then(|container| container.alias_set.as_ref()),
            window,
            cx,
        );
        let container_ancillary_data_set = AncillaryDataSetForm::new(
            command_container.and_then(|container| container.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        let container_rate = ContainerRateForm::new(
            command_container.and_then(|container| container.default_rate_in_stream.as_ref()),
            command_container.and_then(|container| container.rate_in_stream_set.as_ref()),
            window,
            cx,
        );
        let container_binary_encoding = ContainerBinaryEncodingForm::new(
            command_container.and_then(|container| container.binary_encoding.as_ref()),
            window,
            cx,
        );
        let context_significance_list = ContextSignificanceListForm::new(
            command.and_then(|command| match command {
                xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                    command.context_significance_list.as_ref()
                }
                _ => None,
            }),
            window,
            cx,
        );
        let interlock_values = InterlockValues::from_interlock(command.and_then(|cmd| match cmd {
            xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => cmd.interlock.as_ref(),
            _ => None,
        }));
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
                rows: values.arguments.clone(),
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
                alias_set,
                ancillary_data_set,
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
                container_alias_set,
                container_ancillary_data_set,
                container_rate,
                container_binary_encoding,
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
                context_significance_list,
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
                verifiers: VerifierListForm::new(
                    command.and_then(|cmd| match cmd {
                        xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                            cmd.verifier_set.as_ref()
                        }
                        _ => None,
                    }),
                    window,
                    cx,
                ),
                interlock_verification_select: select(
                    VerificationToWaitForChoice::VARIANTS,
                    interlock_values.verification_to_wait_for,
                    window,
                    cx,
                ),
                interlock_scope_input: input(
                    &interlock_values.scope_to_space_system,
                    false,
                    window,
                    cx,
                ),
                interlock_progress_input: input(
                    &interlock_values.verification_progress_percentage,
                    false,
                    window,
                    cx,
                ),
                interlock_suspendable_select: select(
                    SuspendableChoice::VARIANTS,
                    interlock_values.suspendable,
                    window,
                    cx,
                ),
                parameter_to_set_list: ParameterToSetListForm::new(
                    command.and_then(|cmd| match cmd {
                        xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                            cmd.parameter_to_set_list.as_ref()
                        }
                        _ => None,
                    }),
                    window,
                    cx,
                ),
                parameters_to_suspend_alarms: ParametersToSuspendAlarmsOnSetForm::new(
                    command.and_then(|cmd| match cmd {
                        xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                            cmd.parameters_to_suspend_alarms_on_set.as_ref()
                        }
                        _ => None,
                    }),
                    window,
                    cx,
                ),
                documentation_open: false,
                metadata_open: false,
                inheritance_open: false,
                identification_open: false,
                significance_open: false,
                transmission_constraints_open: false,
                verifiers_open: false,
                interlock_open: false,
                parameter_to_set_list_open: false,
                parameters_to_suspend_alarms_open: false,
                container_details_open: false,
                container_metadata_open: false,
                container_rate_open: false,
                container_binary_encoding_open: false,
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
        self.alias_set.load(
            command.and_then(|command| match command {
                xtce::MetaCommandSetTypeContent::MetaCommand(command) => command.alias_set.as_ref(),
                xtce::MetaCommandSetTypeContent::BlockMetaCommand(command) => {
                    command.alias_set.as_ref()
                }
                xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => None,
            }),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            command.and_then(|command| match command {
                xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                    command.ancillary_data_set.as_ref()
                }
                xtce::MetaCommandSetTypeContent::BlockMetaCommand(command) => {
                    command.ancillary_data_set.as_ref()
                }
                xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => None,
            }),
            window,
            cx,
        );
        let interlock_values = InterlockValues::from_interlock(command.and_then(|cmd| match cmd {
            xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => cmd.interlock.as_ref(),
            _ => None,
        }));
        self.documentation_open = false;
        self.metadata_open = false;
        self.inheritance_open = false;
        self.identification_open = false;
        self.significance_open = false;
        self.transmission_constraints_open = false;
        self.verifiers_open = false;
        self.interlock_open = false;
        self.parameter_to_set_list_open = false;
        self.parameters_to_suspend_alarms_open = false;
        self.container_details_open = false;
        self.container_metadata_open = false;
        self.container_rate_open = false;
        self.container_binary_encoding_open = false;
        let command_container = command.and_then(|command| match command {
            xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                command.command_container.as_ref()
            }
            _ => None,
        });
        self.container_alias_set.load(
            command_container.and_then(|container| container.alias_set.as_ref()),
            window,
            cx,
        );
        self.container_ancillary_data_set.load(
            command_container.and_then(|container| container.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        self.container_rate.update(cx, |form, cx| {
            form.load(
                command_container.and_then(|container| container.default_rate_in_stream.as_ref()),
                command_container.and_then(|container| container.rate_in_stream_set.as_ref()),
                window,
                cx,
            );
        });
        self.container_binary_encoding.update(cx, |form, cx| {
            form.load(
                command_container.and_then(|container| container.binary_encoding.as_ref()),
                window,
                cx,
            );
        });
        self.context_significance_list.update(cx, |form, cx| {
            form.load(
                command.and_then(|command| match command {
                    xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                        command.context_significance_list.as_ref()
                    }
                    _ => None,
                }),
                window,
                cx,
            );
        });
        self.parameter_to_set_list.update(cx, |form, cx| {
            form.load(
                command.and_then(|cmd| match cmd {
                    xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                        cmd.parameter_to_set_list.as_ref()
                    }
                    _ => None,
                }),
                window,
                cx,
            );
        });
        self.parameters_to_suspend_alarms.update(cx, |form, cx| {
            form.load(
                command.and_then(|cmd| match cmd {
                    xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => {
                        cmd.parameters_to_suspend_alarms_on_set.as_ref()
                    }
                    _ => None,
                }),
                window,
                cx,
            );
        });
        self.interlock_scope_input.update(cx, |input, cx| {
            input.set_value(interlock_values.scope_to_space_system, window, cx);
        });
        self.interlock_progress_input.update(cx, |input, cx| {
            input.set_value(
                interlock_values.verification_progress_percentage,
                window,
                cx,
            );
        });
        sync_select(
            &self.interlock_verification_select,
            interlock_values.verification_to_wait_for,
            window,
            cx,
        );
        sync_select(
            &self.interlock_suspendable_select,
            interlock_values.suspendable,
            window,
            cx,
        );
        let verifier_set = command.and_then(|cmd| match cmd {
            xtce::MetaCommandSetTypeContent::MetaCommand(cmd) => cmd.verifier_set.as_ref(),
            _ => None,
        });
        self.verifiers.update(cx, |form, cx| {
            form.load(verifier_set, window, cx);
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
            list.rows = values.arguments.clone();
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
        match command {
            xtce::MetaCommandSetTypeContent::MetaCommand(command) => {
                self.alias_set.apply_to_option(&mut command.alias_set, cx);
                self.ancillary_data_set
                    .apply_to_option(&mut command.ancillary_data_set, cx);
            }
            xtce::MetaCommandSetTypeContent::BlockMetaCommand(command) => {
                self.alias_set.apply_to_option(&mut command.alias_set, cx);
                self.ancillary_data_set
                    .apply_to_option(&mut command.ancillary_data_set, cx);
            }
            xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => {}
        }
        if let xtce::MetaCommandSetTypeContent::MetaCommand(cmd) = command {
            self.context_significance_list
                .read(cx)
                .apply_to(&mut cmd.context_significance_list, cx);
            if let Some(container) = cmd.command_container.as_mut() {
                self.container_alias_set
                    .apply_to_option(&mut container.alias_set, cx);
                self.container_ancillary_data_set
                    .apply_to_option(&mut container.ancillary_data_set, cx);
                self.container_rate.read(cx).apply_to(
                    &mut container.default_rate_in_stream,
                    &mut container.rate_in_stream_set,
                    cx,
                );
                self.container_binary_encoding
                    .read(cx)
                    .apply_to(&mut container.binary_encoding, cx);
            }
            cmd.transmission_constraint_list = self.transmission_constraints.read(cx).to_list(cx);
            self.verifiers
                .read(cx)
                .apply_to_verifier_set(&mut cmd.verifier_set, cx);

            let interlock_values = InterlockValues {
                scope_to_space_system: value(&self.interlock_scope_input, cx),
                verification_to_wait_for: selected_value(
                    &self.interlock_verification_select,
                    VerificationToWaitForChoice::None,
                    cx,
                ),
                verification_progress_percentage: value(&self.interlock_progress_input, cx),
                suspendable: selected_value(
                    &self.interlock_suspendable_select,
                    SuspendableChoice::False,
                    cx,
                ),
            };
            interlock_values.apply_to(&mut cmd.interlock);

            cmd.parameter_to_set_list = self.parameter_to_set_list.read(cx).to_list(cx);
            cmd.parameters_to_suspend_alarms_on_set =
                self.parameters_to_suspend_alarms.read(cx).to_list(cx);
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
                .child(self.render_metadata(cx))
                .child(self.render_inheritance(cx))
                .child(self.render_identification(cx))
                .child(self.render_significance(cx))
                .child(self.render_transmission_constraints(cx))
                .child(self.render_verifiers(cx))
                .child(self.render_interlock(cx))
                .child(self.render_parameter_to_set_list(cx))
                .child(self.render_parameters_to_suspend_alarms(cx))
                .child(self.render_command_container(cx)),
            MetaCommandKind::BlockMetaCommand => form
                .child(field(
                    "Meta command steps",
                    "meta command ref | argument=value, argument=value",
                    &self.block_steps_input,
                    cx,
                ))
                .child(self.render_documentation(cx))
                .child(self.render_metadata(cx)),
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

    fn render_metadata(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.metadata_open)
            .child(
                Button::new("toggle-meta-command-metadata")
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
                    ))
                    .child(self.context_significance_list.clone()),
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
            .content(v_flex().pt_3().child(self.transmission_constraints.clone()))
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
            .content(v_flex().pt_3().child(self.verifiers.clone()))
    }

    fn render_interlock(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.interlock_open)
            .child(
                Button::new("toggle-meta-command-interlock")
                    .small()
                    .link()
                    .icon(if self.interlock_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Interlock")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.interlock_open = !this.interlock_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_3()
                            .items_end()
                            .child(div().flex_1().child(select_field(
                                "Verification to wait for",
                                "",
                                &self.interlock_verification_select,
                                cx,
                            )))
                            .child(div().flex_1().child(field(
                                "Scope to space system",
                                "",
                                &self.interlock_scope_input,
                                cx,
                            ))),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .items_end()
                            .child(div().flex_1().child(field(
                                "Progress percentage",
                                "",
                                &self.interlock_progress_input,
                                cx,
                            )))
                            .child(div().w(px(140.)).child(select_field(
                                "Suspendable",
                                "",
                                &self.interlock_suspendable_select,
                                cx,
                            ))),
                    ),
            )
    }

    fn render_parameter_to_set_list(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.parameter_to_set_list_open)
            .child(
                Button::new("toggle-meta-command-parameter-to-set-list")
                    .small()
                    .link()
                    .icon(if self.parameter_to_set_list_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Parameter to set list")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.parameter_to_set_list_open = !this.parameter_to_set_list_open;
                        cx.notify();
                    })),
            )
            .content(v_flex().pt_3().child(self.parameter_to_set_list.clone()))
    }

    fn render_parameters_to_suspend_alarms(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.parameters_to_suspend_alarms_open)
            .child(
                Button::new("toggle-meta-command-parameters-to-suspend-alarms")
                    .small()
                    .link()
                    .icon(if self.parameters_to_suspend_alarms_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Parameters to suspend alarms on set")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.parameters_to_suspend_alarms_open =
                            !this.parameters_to_suspend_alarms_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .child(self.parameters_to_suspend_alarms.clone()),
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
                    super::section_add_button("add-command-container").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.command_container_present.set(true);
                            cx.notify();
                        },
                    )),
                );
        }

        v_flex()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_lg().font_semibold().child("Command container"))
                    .child(
                        super::section_remove_button("remove-command-container").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.command_container_present.set(false);
                                cx.notify();
                            }),
                        ),
                    ),
            )
            .child(
                Collapsible::new()
                    .open(self.container_metadata_open)
                    .child(
                        Button::new("toggle-command-container-metadata")
                            .small()
                            .link()
                            .icon(if self.container_metadata_open {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .label("Metadata")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.container_metadata_open = !this.container_metadata_open;
                                cx.notify();
                            })),
                    )
                    .content(
                        v_flex()
                            .pt_3()
                            .gap_4()
                            .child(self.container_alias_set.render(cx))
                            .child(self.container_ancillary_data_set.render(cx)),
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
            .child(
                Collapsible::new()
                    .open(self.container_rate_open)
                    .child(
                        Button::new("toggle-command-container-rates")
                            .small()
                            .link()
                            .icon(if self.container_rate_open {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .label("Stream rates")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.container_rate_open = !this.container_rate_open;
                                cx.notify();
                            })),
                    )
                    .content(div().pt_3().child(self.container_rate.clone())),
            )
            .child(
                Collapsible::new()
                    .open(self.container_binary_encoding_open)
                    .child(
                        Button::new("toggle-command-container-binary-encoding")
                            .small()
                            .link()
                            .icon(if self.container_binary_encoding_open {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .label("Binary encoding")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.container_binary_encoding_open =
                                    !this.container_binary_encoding_open;
                                cx.notify();
                            })),
                    )
                    .content(div().pt_3().child(self.container_binary_encoding.clone())),
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
    long_description: String,
    aliases: String,
    ancillary_data: String,
}

struct CommandArgumentRow {
    name_input: Entity<InputState>,
    type_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
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
    primary_kind: CommandContainerEntryKind,
    secondary_input: Entity<InputState>,
    tertiary_input: Entity<InputState>,
    order_input: Entity<InputState>,
    instance_input: Entity<InputState>,
    calibrated_select: Entity<SelectState<Vec<CommandParameterValueChoice>>>,
    alias_namespace_input: Entity<InputState>,
    dimensions_input: Entity<InputState>,
    last_array_entry_select: Entity<SelectState<Vec<LastArrayEntryChoice>>>,
    description_input: Entity<InputState>,
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
        long_description: value(&row.long_description_input, cx),
        aliases: row.alias_set.text(cx),
        ancillary_data: row.ancillary_data_set.text(cx),
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
    let order = value(&row.order_input, cx);
    let instance = value(&row.instance_input, cx);
    let use_calibrated_value = selected_value(
        &row.calibrated_select,
        CommandParameterValueChoice::Calibrated,
        cx,
    ) == CommandParameterValueChoice::Calibrated;
    let alias_namespace = value(&row.alias_namespace_input, cx);
    let dimensions = value(&row.dimensions_input, cx);
    let last_array_entry =
        selected_value(&row.last_array_entry_select, LastArrayEntryChoice::No, cx)
            == LastArrayEntryChoice::Yes;
    let description = value(&row.description_input, cx);
    match kind {
        CommandContainerEntryKind::ArgumentRef => EditableContainerEntry::ArgumentRef {
            reference: primary,
            offset: secondary.trim().parse().ok(),
            description: optional_value(description),
        },
        CommandContainerEntryKind::ParameterRef => EditableContainerEntry::ParameterRef {
            reference: primary,
            offset: secondary.trim().parse().ok(),
            description: optional_value(description),
        },
        CommandContainerEntryKind::ParameterSegmentRef => {
            EditableContainerEntry::ParameterSegmentRef {
                reference: primary,
                size_in_bits: tertiary.trim().parse().unwrap_or(i64::MIN),
                order: order.trim().parse().ok(),
                offset: secondary.trim().parse().ok(),
                description: optional_value(description),
            }
        }
        CommandContainerEntryKind::ContainerRef => EditableContainerEntry::ContainerRef {
            reference: primary,
            offset: secondary.trim().parse().ok(),
            description: optional_value(description),
        },
        CommandContainerEntryKind::ContainerSegmentRef => {
            EditableContainerEntry::ContainerSegmentRef {
                reference: primary,
                size_in_bits: tertiary.trim().parse().unwrap_or(i64::MIN),
                order: order.trim().parse().ok(),
                offset: secondary.trim().parse().ok(),
                description: optional_value(description),
            }
        }
        CommandContainerEntryKind::StreamSegment => EditableContainerEntry::StreamSegment {
            reference: primary,
            size_in_bits: tertiary.trim().parse().unwrap_or(i64::MIN),
            order: order.trim().parse().ok(),
            offset: secondary.trim().parse().ok(),
            description: optional_value(description),
        },
        CommandContainerEntryKind::ArrayParameterRef => EditableContainerEntry::ArrayParameterRef {
            reference: primary,
            dimensions,
            last_array_entry,
            offset: secondary.trim().parse().ok(),
            description: optional_value(description),
        },
        CommandContainerEntryKind::IndirectParameterRef => {
            EditableContainerEntry::IndirectParameterRef {
                reference: primary,
                instance: instance
                    .trim()
                    .parse()
                    .unwrap_or_else(|_| xtce::ParameterInstanceRefType::default_instance()),
                use_calibrated_value,
                alias_namespace: optional_value(alias_namespace),
                offset: secondary.trim().parse().ok(),
                description: optional_value(description),
            }
        }
        CommandContainerEntryKind::ArrayArgumentRef => EditableContainerEntry::ArrayArgumentRef {
            reference: primary,
            dimensions,
            last_array_entry,
            offset: secondary.trim().parse().ok(),
            description: optional_value(description),
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
                    super::row_remove_button(
                        format!("remove-base-assignment-{index}"),
                        "Remove assignment",
                    )
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
            .when(row_count == 0, |form| {
                form.child(super::empty_list_state(
                    "No base argument assignments defined.",
                    cx,
                ))
            })
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
            .child(row.clone())
            .child(
                h_flex()
                    .w(px(88.))
                    .flex_none()
                    .gap_1()
                    .child(
                        Button::new(format!("command-argument-options-{index}"))
                            .ghost()
                            .small()
                            .icon(IconName::Ellipsis)
                            .tooltip("Argument options")
                            .on_click({
                                let row = row.clone();
                                move |_, window, cx| {
                                    open_command_argument_options(row.clone(), window, cx);
                                }
                            }),
                    )
                    .child(
                        super::row_remove_button(
                            format!("remove-command-argument-{index}"),
                            "Remove argument",
                        )
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
                                            .child(div().w(px(88.)).flex_none().child("Actions")),
                                    )
                                    .when(row_count > 0, |table| table.children(rows)),
                            ),
                    ),
            )
            .when(row_count == 0, |form| {
                form.child(super::empty_list_state("No arguments defined.", cx))
            })
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

fn open_command_argument_options(
    editor: Entity<CommandArgumentRow>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title("Argument options")
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                let row = editor.read(cx);
                let long_description = row.long_description_input.clone();
                content.child(
                    super::form_dialog_content()
                        .child(field("Long description", "Optional", &long_description, cx))
                        .child(row.alias_set.render(cx))
                        .child(row.ancillary_data_set.render(cx)),
                )
            })
    });
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
        let details_editor = editor.clone();
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
                    .w(px(160.))
                    .flex_none()
                    .gap_1()
                    .child(
                        Button::new(format!("edit-command-container-entry-details-{index}"))
                            .small()
                            .label("Details")
                            .tooltip("Edit entry details")
                            .on_click(move |_, window, cx| {
                                open_command_entry_details(details_editor.clone(), window, cx);
                            }),
                    )
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
                        super::row_remove_button(
                            format!("remove-command-container-entry-{index}"),
                            "Remove entry",
                        )
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
                            .min_w(px(700.))
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
                                    .child(div().w(px(160.)).child("Actions"))
                                    .child(div().w(px(130.)).child("Type"))
                                    .child(div().flex_1().child("Entry")),
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
                                        .h(px(if row_count == 0 { 0. } else { 352. })),
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
                            .child(super::count_label(row_count, "entry", "entries")),
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
            .when(row_count == 0, |form| {
                form.child(super::empty_list_state("No entries defined.", cx))
            })
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let kind = selected_value(
            &self.kind_select,
            CommandContainerEntryKind::ArgumentRef,
            cx,
        );
        if self.primary_kind != kind {
            self.primary_kind = kind;
            self.primary_input.update(cx, |input, cx| {
                input.set_value("", window, cx);
                input.set_placeholder(command_entry_placeholder(kind), window, cx);
            });
            for input in [
                &self.secondary_input,
                &self.tertiary_input,
                &self.order_input,
                &self.instance_input,
                &self.alias_namespace_input,
                &self.dimensions_input,
                &self.description_input,
            ] {
                input.update(cx, |input, cx| input.set_value("", window, cx));
            }
            self.calibrated_select.update(cx, |select, cx| {
                select.set_selected_value(&CommandParameterValueChoice::Calibrated, window, cx);
            });
            self.last_array_entry_select.update(cx, |select, cx| {
                select.set_selected_value(&LastArrayEntryChoice::No, window, cx);
            });
        }
        h_flex()
            .flex_1()
            .min_w_0()
            .gap_2()
            .child(
                div()
                    .w(px(190.))
                    .flex_none()
                    .child(Select::new(&self.kind_select).w_full()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Input::new(&self.primary_input)),
            )
    }
}

fn open_command_entry_details(
    editor: Entity<CommandContainerEntryRow>,
    window: &mut Window,
    cx: &mut App,
) {
    let title = {
        let row = editor.read(cx);
        format!(
            "{} details",
            selected_value(&row.kind_select, CommandContainerEntryKind::ArgumentRef, cx)
        )
    };
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title(title.clone())
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                let row = editor.read(cx);
                let kind =
                    selected_value(&row.kind_select, CommandContainerEntryKind::ArgumentRef, cx);
                let secondary = row.secondary_input.clone();
                let tertiary = row.tertiary_input.clone();
                let order = row.order_input.clone();
                let instance = row.instance_input.clone();
                let calibrated = row.calibrated_select.clone();
                let alias_namespace = row.alias_namespace_input.clone();
                let dimensions = row.dimensions_input.clone();
                let last_array_entry = row.last_array_entry_select.clone();
                let description = row.description_input.clone();
                let fixed = kind == CommandContainerEntryKind::FixedValue;
                let indirect = kind == CommandContainerEntryKind::IndirectParameterRef;
                let array = matches!(
                    kind,
                    CommandContainerEntryKind::ArrayParameterRef
                        | CommandContainerEntryKind::ArrayArgumentRef
                );
                let segment = matches!(
                    kind,
                    CommandContainerEntryKind::ParameterSegmentRef
                        | CommandContainerEntryKind::ContainerSegmentRef
                        | CommandContainerEntryKind::StreamSegment
                );
                content.child(
                    super::form_dialog_content()
                        .when(fixed, |form| {
                            form.child(field("Binary value", "Required", &secondary, cx))
                                .child(field("Size in bits", "Required", &tertiary, cx))
                        })
                        .when(!fixed, |form| {
                            form.child(field("Offset", "Optional", &secondary, cx))
                                .when(segment, |form| {
                                    form.child(field("Size in bits", "Required", &tertiary, cx))
                                        .child(field("Order", "Optional", &order, cx))
                                })
                                .when(indirect, |form| {
                                    form.child(field(
                                        "Instance",
                                        "Optional; defaults to 0",
                                        &instance,
                                        cx,
                                    ))
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_1()
                                            .child(div().text_sm().child("Parameter value"))
                                            .child(Select::new(&calibrated).w_full()),
                                    )
                                    .child(field(
                                        "Alias namespace",
                                        "Optional",
                                        &alias_namespace,
                                        cx,
                                    ))
                                })
                                .when(array, |form| {
                                    form.child(field(
                                        "Dimensions",
                                        "Optional; comma-separated ranges such as 0..3, 1..2",
                                        &dimensions,
                                        cx,
                                    ))
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .child("Last entry for this array instance"),
                                            )
                                            .child(Select::new(&last_array_entry).w_full()),
                                    )
                                })
                                .child(field("Description", "Optional", &description, cx))
                        }),
                )
            })
    });
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
        if !matches!(
            selected_value(
                &self.kind_select,
                CommandContainerEntryKind::ArgumentRef,
                cx,
            ),
            CommandContainerEntryKind::ArgumentRef | CommandContainerEntryKind::ArrayArgumentRef
        ) {
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

fn command_argument_models(list: Option<&xtce::ArgumentListType>) -> Vec<CommandArgumentData> {
    list.into_iter()
        .flat_map(|list| &list.argument)
        .map(|argument| CommandArgumentData {
            name: argument.name.clone(),
            type_ref: argument.argument_type_ref.clone(),
            initial_value: argument.initial_value.clone().unwrap_or_default(),
            short_description: argument.short_description.clone().unwrap_or_default(),
            long_description: argument.long_description.clone().unwrap_or_default(),
            aliases: AliasSetForm::encode(argument.alias_set.as_ref()),
            ancillary_data: AncillaryDataSetForm::encode(argument.ancillary_data_set.as_ref()),
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
            long_description_input: input(&data.long_description, true, window, cx),
            alias_set: AliasSetForm::new_text(&data.aliases, window, cx),
            ancillary_data_set: AncillaryDataSetForm::new_text(&data.ancillary_data, window, cx),
            optional_fields_open,
        }
    })
}

fn command_argument_rows_value(
    rows: &Entity<ArgumentListView>,
    cx: &App,
) -> Vec<CommandArgumentData> {
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
                CommandArgumentData {
                    name: data.name.trim().to_owned(),
                    type_ref: data.type_ref.trim().to_owned(),
                    initial_value: initial_value.trim().to_owned(),
                    short_description: data.short_description.trim().to_owned(),
                    long_description: data.long_description,
                    aliases: data.aliases,
                    ancillary_data: data.ancillary_data,
                }
            })
        })
        .collect()
}

fn new_command_container_entry_row(
    entry: EditableContainerEntry,
    arguments: Entity<ArgumentListView>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<CommandContainerEntryRow> {
    let (instance, use_calibrated_value, alias_namespace) = match &entry {
        EditableContainerEntry::IndirectParameterRef {
            instance,
            use_calibrated_value,
            alias_namespace,
            ..
        } => (
            instance.to_string(),
            *use_calibrated_value,
            alias_namespace.clone().unwrap_or_default(),
        ),
        _ => (String::new(), true, String::new()),
    };
    let (dimensions, last_array_entry) = match &entry {
        EditableContainerEntry::ArrayParameterRef {
            dimensions,
            last_array_entry,
            ..
        }
        | EditableContainerEntry::ArrayArgumentRef {
            dimensions,
            last_array_entry,
            ..
        } => (dimensions.clone(), *last_array_entry),
        _ => (String::new(), false),
    };
    let (kind, primary, secondary, tertiary, order, description) = match entry {
        EditableContainerEntry::ArgumentRef {
            reference,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ArgumentRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            String::new(),
            String::new(),
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
            String::new(),
            String::new(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::ParameterSegmentRef {
            reference,
            size_in_bits,
            order,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ParameterSegmentRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            size_in_bits.to_string(),
            order.map(|value| value.to_string()).unwrap_or_default(),
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
            String::new(),
            String::new(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::ContainerSegmentRef {
            reference,
            size_in_bits,
            order,
            offset,
            description,
        } => (
            CommandContainerEntryKind::ContainerSegmentRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            size_in_bits.to_string(),
            order.map(|value| value.to_string()).unwrap_or_default(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::StreamSegment {
            reference,
            size_in_bits,
            order,
            offset,
            description,
        } => (
            CommandContainerEntryKind::StreamSegment,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            size_in_bits.to_string(),
            order.map(|value| value.to_string()).unwrap_or_default(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::ArrayParameterRef {
            reference,
            offset,
            description,
            ..
        } => (
            CommandContainerEntryKind::ArrayParameterRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            String::new(),
            String::new(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::IndirectParameterRef {
            reference,
            offset,
            description,
            ..
        } => (
            CommandContainerEntryKind::IndirectParameterRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            String::new(),
            String::new(),
            description.unwrap_or_default(),
        ),
        EditableContainerEntry::ArrayArgumentRef {
            reference,
            offset,
            description,
            ..
        } => (
            CommandContainerEntryKind::ArrayArgumentRef,
            reference,
            offset.map(|value| value.to_string()).unwrap_or_default(),
            String::new(),
            String::new(),
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
            String::new(),
            String::new(),
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
        let calibrated_select = cx.new(|cx| {
            SelectState::new(
                CommandParameterValueChoice::VARIANTS.to_vec(),
                Some(IndexPath::default().row(usize::from(!use_calibrated_value))),
                window,
                cx,
            )
        });
        let last_array_entry_select = cx.new(|cx| {
            SelectState::new(
                LastArrayEntryChoice::VARIANTS.to_vec(),
                Some(IndexPath::default().row(usize::from(last_array_entry))),
                window,
                cx,
            )
        });
        let primary_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .default_value(primary.clone())
                .placeholder(command_entry_placeholder(kind));
            input.lsp.completion_provider = Some(Rc::new(EntryArgumentCompletionProvider {
                arguments: arguments.clone(),
                kind_select: kind_select.clone(),
            }));
            input
        });

        CommandContainerEntryRow {
            kind_select,
            primary_input,
            primary_kind: kind,
            secondary_input: input(&secondary, false, window, cx),
            tertiary_input: input(&tertiary, false, window, cx),
            order_input: input(&order, false, window, cx),
            instance_input: input(&instance, false, window, cx),
            calibrated_select,
            alias_namespace_input: input(&alias_namespace, false, window, cx),
            dimensions_input: input(&dimensions, false, window, cx),
            last_array_entry_select,
            description_input: input(&description, false, window, cx),
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
    if let EditableContainerEntry::ArrayArgumentRef {
        reference,
        dimensions,
        last_array_entry,
        offset,
        description,
    } = entry
    {
        return (!reference.trim().is_empty()).then(|| {
            format!(
                "ArrayArgumentRefEntry | {} | {} | {} | {} | {}",
                reference.trim(),
                dimensions.trim(),
                last_array_entry,
                offset.map(|value| value.to_string()).unwrap_or_default(),
                description.as_deref().unwrap_or_default()
            )
        });
    }
    if let EditableContainerEntry::ArrayParameterRef {
        reference,
        dimensions,
        last_array_entry,
        offset,
        description,
    } = entry
    {
        return (!reference.trim().is_empty()).then(|| {
            format!(
                "ArrayParameterRefEntry | {} | {} | {} | {} | {}",
                reference.trim(),
                dimensions.trim(),
                last_array_entry,
                offset.map(|value| value.to_string()).unwrap_or_default(),
                description.as_deref().unwrap_or_default()
            )
        });
    }
    if let EditableContainerEntry::IndirectParameterRef {
        reference,
        instance,
        use_calibrated_value,
        alias_namespace,
        offset,
        description,
    } = entry
    {
        return (!reference.trim().is_empty()).then(|| {
            format!(
                "IndirectParameterRefEntry | {} | {} | {} | {} | {} | {}",
                reference.trim(),
                instance,
                use_calibrated_value,
                alias_namespace.as_deref().unwrap_or_default(),
                offset.map(|value| value.to_string()).unwrap_or_default(),
                description.as_deref().unwrap_or_default()
            )
        });
    }
    if let EditableContainerEntry::StreamSegment {
        reference,
        size_in_bits,
        order,
        offset,
        description,
    } = entry
    {
        return (!reference.trim().is_empty() && *size_in_bits != i64::MIN).then(|| {
            format!(
                "StreamSegmentEntry | {} | {} | {} | {} | {}",
                reference.trim(),
                size_in_bits,
                order.map(|value| value.to_string()).unwrap_or_default(),
                offset.map(|value| value.to_string()).unwrap_or_default(),
                description.as_deref().unwrap_or_default()
            )
        });
    }
    if let EditableContainerEntry::ContainerSegmentRef {
        reference,
        size_in_bits,
        order,
        offset,
        description,
    } = entry
    {
        return (!reference.trim().is_empty() && *size_in_bits != i64::MIN).then(|| {
            format!(
                "ContainerSegmentRefEntry | {} | {} | {} | {} | {}",
                reference.trim(),
                size_in_bits,
                order.map(|value| value.to_string()).unwrap_or_default(),
                offset.map(|value| value.to_string()).unwrap_or_default(),
                description.as_deref().unwrap_or_default()
            )
        });
    }
    if let EditableContainerEntry::ParameterSegmentRef {
        reference,
        size_in_bits,
        order,
        offset,
        description,
    } = entry
    {
        return (!reference.trim().is_empty() && *size_in_bits != i64::MIN).then(|| {
            format!(
                "ParameterSegmentRefEntry | {} | {} | {} | {} | {}",
                reference.trim(),
                size_in_bits,
                order.map(|value| value.to_string()).unwrap_or_default(),
                offset.map(|value| value.to_string()).unwrap_or_default(),
                description.as_deref().unwrap_or_default()
            )
        });
    }
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
        EditableContainerEntry::ParameterSegmentRef { .. } => unreachable!(),
        EditableContainerEntry::ContainerSegmentRef { .. } => unreachable!(),
        EditableContainerEntry::StreamSegment { .. } => unreachable!(),
        EditableContainerEntry::ArrayParameterRef { .. } => unreachable!(),
        EditableContainerEntry::IndirectParameterRef { .. } => unreachable!(),
        EditableContainerEntry::ArrayArgumentRef { .. } => unreachable!(),
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
    arguments: Vec<CommandArgumentData>,
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
            arguments: Vec::new(),
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
                values.arguments = command_argument_models(command.argument_list.as_ref());
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
    ParameterSegmentRef {
        reference: String,
        size_in_bits: i64,
        order: Option<i64>,
        offset: Option<i64>,
        description: Option<String>,
    },
    ContainerRef {
        reference: String,
        offset: Option<i64>,
        description: Option<String>,
    },
    ContainerSegmentRef {
        reference: String,
        size_in_bits: i64,
        order: Option<i64>,
        offset: Option<i64>,
        description: Option<String>,
    },
    StreamSegment {
        reference: String,
        size_in_bits: i64,
        order: Option<i64>,
        offset: Option<i64>,
        description: Option<String>,
    },
    ArrayParameterRef {
        reference: String,
        dimensions: String,
        last_array_entry: bool,
        offset: Option<i64>,
        description: Option<String>,
    },
    IndirectParameterRef {
        reference: String,
        instance: i64,
        use_calibrated_value: bool,
        alias_namespace: Option<String>,
        offset: Option<i64>,
        description: Option<String>,
    },
    ArrayArgumentRef {
        reference: String,
        dimensions: String,
        last_array_entry: bool,
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
                | EditableContainerEntry::ContainerRef { .. }
                | EditableContainerEntry::ArrayParameterRef { .. }
                | EditableContainerEntry::ArrayArgumentRef { .. }
                | EditableContainerEntry::IndirectParameterRef { .. } => (None, None),
                EditableContainerEntry::ParameterSegmentRef {
                    offset,
                    size_in_bits,
                    ..
                }
                | EditableContainerEntry::ContainerSegmentRef {
                    offset,
                    size_in_bits,
                    ..
                }
                | EditableContainerEntry::StreamSegment {
                    offset,
                    size_in_bits,
                    ..
                } => (
                    offset
                        .unwrap_or_default()
                        .try_into()
                        .ok()
                        .and_then(|offset: u64| {
                            cursor.and_then(|cursor| cursor.checked_add(offset))
                        }),
                    u64::try_from(*size_in_bits).ok().filter(|size| *size > 0),
                ),
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
            EditableContainerEntry::IndirectParameterRef { reference, .. } => {
                layout
                    .unresolved
                    .push(format!("{reference}: indirect parameter size is unknown"));
                *cursor = None;
                continue;
            }
            EditableContainerEntry::ArrayParameterRef { reference, .. } => {
                layout
                    .unresolved
                    .push(format!("{reference}: array parameter size is unknown"));
                *cursor = None;
                continue;
            }
            EditableContainerEntry::ArrayArgumentRef { reference, .. } => {
                layout
                    .unresolved
                    .push(format!("{reference}: array argument size is unknown"));
                *cursor = None;
                continue;
            }
            EditableContainerEntry::ParameterSegmentRef {
                reference,
                offset,
                size_in_bits,
                ..
            } => (
                reference.clone(),
                cursor.and_then(|cursor| {
                    u64::try_from(offset.unwrap_or_default())
                        .ok()
                        .and_then(|offset| cursor.checked_add(offset))
                }),
                u64::try_from(*size_in_bits).ok().filter(|size| *size > 0),
            ),
            EditableContainerEntry::ContainerSegmentRef {
                reference,
                offset,
                size_in_bits,
                ..
            } => (
                reference.clone(),
                cursor.and_then(|cursor| {
                    u64::try_from(offset.unwrap_or_default())
                        .ok()
                        .and_then(|offset| cursor.checked_add(offset))
                }),
                u64::try_from(*size_in_bits).ok().filter(|size| *size > 0),
            ),
            EditableContainerEntry::StreamSegment {
                reference,
                offset,
                size_in_bits,
                ..
            } => (
                reference.clone(),
                cursor.and_then(|cursor| {
                    u64::try_from(offset.unwrap_or_default())
                        .ok()
                        .and_then(|offset| cursor.checked_add(offset))
                }),
                u64::try_from(*size_in_bits).ok().filter(|size| *size > 0),
            ),
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
            xtce::CommandContainerEntryListTypeContent::ParameterSegmentRefEntry(entry) => {
                Some(format!(
                    "ParameterSegmentRefEntry | {} | {} | {} | {} | {}",
                    entry.parameter_ref,
                    entry.size_in_bits,
                    entry
                        .order
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                    entry.short_description.as_deref().unwrap_or_default()
                ))
            }
            xtce::CommandContainerEntryListTypeContent::ContainerRefEntry(entry) => Some(format!(
                "ContainerRefEntry | {} | {} | {}",
                entry.container_ref,
                fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                entry.short_description.as_deref().unwrap_or_default()
            )),
            xtce::CommandContainerEntryListTypeContent::ContainerSegmentRefEntry(entry) => {
                Some(format!(
                    "ContainerSegmentRefEntry | {} | {} | {} | {} | {}",
                    entry.container_ref,
                    entry.size_in_bits,
                    entry
                        .order
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                    entry.short_description.as_deref().unwrap_or_default()
                ))
            }
            xtce::CommandContainerEntryListTypeContent::StreamSegmentEntry(entry) => Some(format!(
                "StreamSegmentEntry | {} | {} | {} | {} | {}",
                entry.stream_ref,
                entry.size_in_bits,
                entry
                    .order
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                entry.short_description.as_deref().unwrap_or_default()
            )),
            xtce::CommandContainerEntryListTypeContent::ArrayParameterRefEntry(entry) => {
                Some(format!(
                    "ArrayParameterRefEntry | {} | {} | {} | {} | {}",
                    entry.parameter_ref,
                    entry
                        .content
                        .as_ref()
                        .map(|content| { encode_command_dimensions(&content.dimension_list) })
                        .unwrap_or_default(),
                    entry.last_entry_for_this_array_instance,
                    fixed_entry_offset(
                        entry
                            .content
                            .as_ref()
                            .and_then(|content| { content.location_in_container_in_bits.as_ref() })
                    ),
                    entry.short_description.as_deref().unwrap_or_default()
                ))
            }
            xtce::CommandContainerEntryListTypeContent::IndirectParameterRefEntry(entry) => {
                Some(format!(
                    "IndirectParameterRefEntry | {} | {} | {} | {} | {} | {}",
                    entry.parameter_instance.parameter_ref,
                    entry.parameter_instance.instance,
                    entry.parameter_instance.use_calibrated_value,
                    entry.alias_name_space.as_deref().unwrap_or_default(),
                    fixed_entry_offset(entry.location_in_container_in_bits.as_ref()),
                    entry.short_description.as_deref().unwrap_or_default()
                ))
            }
            xtce::CommandContainerEntryListTypeContent::ArrayArgumentRefEntry(entry) => {
                Some(format!(
                    "ArrayArgumentRefEntry | {} | {} | {} | {} | {}",
                    entry.argument_ref,
                    entry
                        .content
                        .as_ref()
                        .map(|content| {
                            encode_command_argument_dimensions(&content.dimension_list)
                        })
                        .unwrap_or_default(),
                    entry.last_entry_for_this_array_instance,
                    fixed_entry_offset(
                        entry
                            .content
                            .as_ref()
                            .and_then(|content| { content.location_in_container_in_bits.as_ref() })
                    ),
                    entry.short_description.as_deref().unwrap_or_default()
                ))
            }
            xtce::CommandContainerEntryListTypeContent::FixedValueEntry(entry) => Some(format!(
                "FixedValueEntry | {} | {} | {}",
                entry.name.as_deref().unwrap_or_default(),
                entry.binary_value,
                entry.size_in_bits
            )),
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
                "ParameterSegmentRefEntry" => Some(EditableContainerEntry::ParameterSegmentRef {
                    reference: nonempty_field(&fields, 1)?,
                    size_in_bits: fields.get(2)?.parse().ok()?,
                    order: numeric_field(&fields, 3),
                    offset: numeric_field(&fields, 4),
                    description: optional_field(&fields, 5),
                }),
                "ContainerRefEntry" => Some(EditableContainerEntry::ContainerRef {
                    reference: nonempty_field(&fields, 1)?,
                    offset: numeric_field(&fields, 2),
                    description: optional_field(&fields, 3),
                }),
                "ContainerSegmentRefEntry" => Some(EditableContainerEntry::ContainerSegmentRef {
                    reference: nonempty_field(&fields, 1)?,
                    size_in_bits: fields.get(2)?.parse().ok()?,
                    order: numeric_field(&fields, 3),
                    offset: numeric_field(&fields, 4),
                    description: optional_field(&fields, 5),
                }),
                "StreamSegmentEntry" => Some(EditableContainerEntry::StreamSegment {
                    reference: nonempty_field(&fields, 1)?,
                    size_in_bits: fields.get(2)?.parse().ok()?,
                    order: numeric_field(&fields, 3),
                    offset: numeric_field(&fields, 4),
                    description: optional_field(&fields, 5),
                }),
                "ArrayParameterRefEntry" => Some(EditableContainerEntry::ArrayParameterRef {
                    reference: nonempty_field(&fields, 1)?,
                    dimensions: fields.get(2)?.trim().to_owned(),
                    last_array_entry: fields
                        .get(3)
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(
                            xtce::ArgumentArrayParameterRefEntryType::default_last_entry_for_this_array_instance,
                        ),
                    offset: numeric_field(&fields, 4),
                    description: optional_field(&fields, 5),
                }),
                "IndirectParameterRefEntry" => Some(EditableContainerEntry::IndirectParameterRef {
                    reference: nonempty_field(&fields, 1)?,
                    instance: fields
                        .get(2)
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(xtce::ParameterInstanceRefType::default_instance),
                    use_calibrated_value: fields
                        .get(3)
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(
                            xtce::ParameterInstanceRefType::default_use_calibrated_value,
                        ),
                    alias_namespace: optional_field(&fields, 4),
                    offset: numeric_field(&fields, 5),
                    description: optional_field(&fields, 6),
                }),
                "ArrayArgumentRefEntry" => Some(EditableContainerEntry::ArrayArgumentRef {
                    reference: nonempty_field(&fields, 1)?,
                    dimensions: fields.get(2)?.trim().to_owned(),
                    last_array_entry: fields
                        .get(3)
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(
                            xtce::ArgumentArrayArgumentRefEntryType::default_last_entry_for_this_array_instance,
                        ),
                    offset: numeric_field(&fields, 4),
                    description: optional_field(&fields, 5),
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

fn encode_command_dimensions(list: &xtce::DimensionListType) -> String {
    list.dimension
        .iter()
        .map(|dimension| {
            format!(
                "{}..{}",
                command_dimension_value_text(&dimension.starting_index),
                command_dimension_value_text(&dimension.ending_index)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn command_dimension_value_text(value: &xtce::IntegerValueType) -> String {
    match value {
        xtce::IntegerValueType::FixedValue(value) => value.to_string(),
        xtce::IntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::IntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
    }
}

fn encode_command_argument_dimensions(list: &xtce::ArgumentDimensionListType) -> String {
    list.dimension
        .iter()
        .map(|dimension| {
            format!(
                "{}..{}",
                command_argument_dimension_value_text(&dimension.starting_index),
                command_argument_dimension_value_text(&dimension.ending_index)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn command_argument_dimension_value_text(value: &xtce::ArgumentIntegerValueType) -> String {
    match value {
        xtce::ArgumentIntegerValueType::FixedValue(value) => value.to_string(),
        xtce::ArgumentIntegerValueType::DynamicValue(_) => "<dynamic>".to_owned(),
        xtce::ArgumentIntegerValueType::DiscreteLookupList(_) => "<lookup>".to_owned(),
    }
}

fn decode_command_dimensions(value: &str) -> Option<Vec<(i64, i64)>> {
    let value = value.trim();
    if value.is_empty() {
        return Some(Vec::new());
    }
    value
        .split(',')
        .map(|dimension| {
            let (start, end) = dimension.trim().split_once("..")?;
            Some((start.trim().parse().ok()?, end.trim().parse().ok()?))
        })
        .collect()
}

fn apply_command_array_dimensions(
    entry: &mut xtce::ArgumentArrayParameterRefEntryType,
    value: &str,
) {
    let Some(dimensions) = decode_command_dimensions(value) else {
        return;
    };
    if dimensions.is_empty() {
        entry.content = None;
        return;
    }
    let content =
        entry
            .content
            .get_or_insert_with(|| xtce::ArgumentArrayParameterRefEntryTypeContent {
                location_in_container_in_bits: None,
                repeat_entry: None,
                include_condition: None,
                ancillary_data_set: None,
                dimension_list: xtce::DimensionListType {
                    dimension: Vec::new(),
                },
            });
    let mut existing = std::mem::take(&mut content.dimension_list.dimension).into_iter();
    content.dimension_list.dimension = dimensions
        .into_iter()
        .map(|(start, end)| {
            let mut dimension = existing.next().unwrap_or(xtce::DimensionType {
                starting_index: xtce::IntegerValueType::FixedValue(0),
                ending_index: xtce::IntegerValueType::FixedValue(0),
            });
            dimension.starting_index = xtce::IntegerValueType::FixedValue(start);
            dimension.ending_index = xtce::IntegerValueType::FixedValue(end);
            dimension
        })
        .collect();
}

fn apply_command_array_argument_dimensions(
    entry: &mut xtce::ArgumentArrayArgumentRefEntryType,
    value: &str,
) {
    let Some(dimensions) = decode_command_dimensions(value) else {
        return;
    };
    if dimensions.is_empty() {
        entry.content = None;
        return;
    }
    let content =
        entry
            .content
            .get_or_insert_with(|| xtce::ArgumentArrayArgumentRefEntryTypeContent {
                location_in_container_in_bits: None,
                repeat_entry: None,
                include_condition: None,
                ancillary_data_set: None,
                dimension_list: xtce::ArgumentDimensionListType {
                    dimension: Vec::new(),
                },
            });
    let mut existing = std::mem::take(&mut content.dimension_list.dimension).into_iter();
    content.dimension_list.dimension = dimensions
        .into_iter()
        .map(|(start, end)| {
            let mut dimension = existing.next().unwrap_or(xtce::ArgumentDimensionType {
                starting_index: xtce::ArgumentIntegerValueType::FixedValue(0),
                ending_index: xtce::ArgumentIntegerValueType::FixedValue(0),
            });
            dimension.starting_index = xtce::ArgumentIntegerValueType::FixedValue(start);
            dimension.ending_index = xtce::ArgumentIntegerValueType::FixedValue(end);
            dimension
        })
        .collect();
}

fn apply_container_entries(list: &mut xtce::CommandContainerEntryListType, value: &str) {
    let mut argument_entries = VecDeque::new();
    let mut parameter_entries = VecDeque::new();
    let mut parameter_segment_entries = VecDeque::new();
    let mut container_entries = VecDeque::new();
    let mut container_segment_entries = VecDeque::new();
    let mut stream_segment_entries = VecDeque::new();
    let mut array_parameter_entries = VecDeque::new();
    let mut indirect_parameter_entries = VecDeque::new();
    let mut array_argument_entries = VecDeque::new();
    let mut fixed_entries = VecDeque::new();

    for entry in std::mem::take(&mut list.content) {
        match entry {
            xtce::CommandContainerEntryListTypeContent::ArgumentRefEntry(entry) => {
                argument_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ParameterRefEntry(entry) => {
                parameter_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ParameterSegmentRefEntry(entry) => {
                parameter_segment_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ContainerRefEntry(entry) => {
                container_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ContainerSegmentRefEntry(entry) => {
                container_segment_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::StreamSegmentEntry(entry) => {
                stream_segment_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ArrayParameterRefEntry(entry) => {
                array_parameter_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::IndirectParameterRefEntry(entry) => {
                indirect_parameter_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::ArrayArgumentRefEntry(entry) => {
                array_argument_entries.push_back(entry);
            }
            xtce::CommandContainerEntryListTypeContent::FixedValueEntry(entry) => {
                fixed_entries.push_back(entry);
            }
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
                EditableContainerEntry::ParameterSegmentRef {
                    reference,
                    size_in_bits,
                    order,
                    offset,
                    description,
                } => {
                    let mut entry = parameter_segment_entries.pop_front().unwrap_or(
                        xtce::ArgumentParameterSegmentRefEntryType {
                            short_description: None,
                            parameter_ref: String::new(),
                            order: None,
                            size_in_bits: 0,
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                        },
                    );
                    entry.parameter_ref = reference;
                    entry.size_in_bits = size_in_bits;
                    entry.order = order;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::ParameterSegmentRefEntry(entry)
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
                EditableContainerEntry::ContainerSegmentRef {
                    reference,
                    size_in_bits,
                    order,
                    offset,
                    description,
                } => {
                    let mut entry = container_segment_entries.pop_front().unwrap_or(
                        xtce::ArgumentContainerSegmentRefEntryType {
                            short_description: None,
                            container_ref: String::new(),
                            order: None,
                            size_in_bits: 0,
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                        },
                    );
                    entry.container_ref = reference;
                    entry.size_in_bits = size_in_bits;
                    entry.order = order;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::ContainerSegmentRefEntry(entry)
                }
                EditableContainerEntry::StreamSegment {
                    reference,
                    size_in_bits,
                    order,
                    offset,
                    description,
                } => {
                    let mut entry = stream_segment_entries.pop_front().unwrap_or(
                        xtce::ArgumentStreamSegmentEntryType {
                            short_description: None,
                            stream_ref: String::new(),
                            order: None,
                            size_in_bits: 0,
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                        },
                    );
                    entry.stream_ref = reference;
                    entry.size_in_bits = size_in_bits;
                    entry.order = order;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::StreamSegmentEntry(entry)
                }
                EditableContainerEntry::ArrayParameterRef {
                    reference,
                    dimensions,
                    last_array_entry,
                    offset,
                    description,
                } => {
                    let mut entry = array_parameter_entries.pop_front().unwrap_or(
                        xtce::ArgumentArrayParameterRefEntryType {
                            short_description: None,
                            parameter_ref: String::new(),
                            last_entry_for_this_array_instance:
                                xtce::ArgumentArrayParameterRefEntryType::default_last_entry_for_this_array_instance(),
                            content: None,
                        },
                    );
                    entry.parameter_ref = reference;
                    entry.last_entry_for_this_array_instance = last_array_entry;
                    entry.short_description = description;
                    apply_command_array_dimensions(&mut entry, &dimensions);
                    if let Some(content) = &mut entry.content {
                        apply_fixed_entry_offset(
                            &mut content.location_in_container_in_bits,
                            offset,
                        );
                    }
                    xtce::CommandContainerEntryListTypeContent::ArrayParameterRefEntry(entry)
                }
                EditableContainerEntry::IndirectParameterRef {
                    reference,
                    instance,
                    use_calibrated_value,
                    alias_namespace,
                    offset,
                    description,
                } => {
                    let mut entry = indirect_parameter_entries.pop_front().unwrap_or(
                        xtce::ArgumentIndirectParameterRefEntryType {
                            short_description: None,
                            alias_name_space: None,
                            location_in_container_in_bits: None,
                            repeat_entry: None,
                            include_condition: None,
                            ancillary_data_set: None,
                            parameter_instance: xtce::ParameterInstanceRefType {
                                parameter_ref: String::new(),
                                instance: xtce::ParameterInstanceRefType::default_instance(),
                                use_calibrated_value:
                                    xtce::ParameterInstanceRefType::default_use_calibrated_value(),
                            },
                        },
                    );
                    entry.parameter_instance.parameter_ref = reference;
                    entry.parameter_instance.instance = instance;
                    entry.parameter_instance.use_calibrated_value = use_calibrated_value;
                    entry.alias_name_space = alias_namespace;
                    entry.short_description = description;
                    apply_fixed_entry_offset(&mut entry.location_in_container_in_bits, offset);
                    xtce::CommandContainerEntryListTypeContent::IndirectParameterRefEntry(entry)
                }
                EditableContainerEntry::ArrayArgumentRef {
                    reference,
                    dimensions,
                    last_array_entry,
                    offset,
                    description,
                } => {
                    let mut entry = array_argument_entries.pop_front().unwrap_or(
                        xtce::ArgumentArrayArgumentRefEntryType {
                            short_description: None,
                            argument_ref: String::new(),
                            last_entry_for_this_array_instance:
                                xtce::ArgumentArrayArgumentRefEntryType::default_last_entry_for_this_array_instance(),
                            content: None,
                        },
                    );
                    entry.argument_ref = reference;
                    entry.last_entry_for_this_array_instance = last_array_entry;
                    entry.short_description = description;
                    apply_command_array_argument_dimensions(&mut entry, &dimensions);
                    if let Some(content) = &mut entry.content {
                        apply_fixed_entry_offset(
                            &mut content.location_in_container_in_bits,
                            offset,
                        );
                    }
                    xtce::CommandContainerEntryListTypeContent::ArrayArgumentRefEntry(entry)
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

#[cfg(test)]
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
#[cfg(test)]
impl_select_item!(ComparisonOperatorChoice);

#[cfg(test)]
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

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum SuspendableChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(SuspendableChoice);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerificationToWaitForChoice {
    None,
    Release,
    TransferredToRange,
    SentFromRange,
    Received,
    Accepted,
    Queued,
    Executing,
    Complete,
    Failed,
}

impl VerificationToWaitForChoice {
    const VARIANTS: &'static [Self] = &[
        Self::None,
        Self::Release,
        Self::TransferredToRange,
        Self::SentFromRange,
        Self::Received,
        Self::Accepted,
        Self::Queued,
        Self::Executing,
        Self::Complete,
        Self::Failed,
    ];
}

impl std::fmt::Display for VerificationToWaitForChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::None => "None (Disabled)",
            Self::Release => "Release",
            Self::TransferredToRange => "Transferred to range",
            Self::SentFromRange => "Sent from range",
            Self::Received => "Received",
            Self::Accepted => "Accepted",
            Self::Queued => "Queued",
            Self::Executing => "Executing",
            Self::Complete => "Complete",
            Self::Failed => "Failed",
        };
        write!(f, "{s}")
    }
}

impl_select_item!(VerificationToWaitForChoice);

struct InterlockValues {
    scope_to_space_system: String,
    verification_to_wait_for: VerificationToWaitForChoice,
    verification_progress_percentage: String,
    suspendable: SuspendableChoice,
}

impl InterlockValues {
    fn from_interlock(interlock: Option<&xtce::InterlockType>) -> Self {
        match interlock {
            Some(interlock) => Self {
                scope_to_space_system: interlock.scope_to_space_system.clone().unwrap_or_default(),
                verification_to_wait_for: match interlock.verification_to_wait_for {
                    xtce::VerifierEnumerationType::Release => VerificationToWaitForChoice::Release,
                    xtce::VerifierEnumerationType::TransferredToRange => {
                        VerificationToWaitForChoice::TransferredToRange
                    }
                    xtce::VerifierEnumerationType::SentFromRange => {
                        VerificationToWaitForChoice::SentFromRange
                    }
                    xtce::VerifierEnumerationType::Received => {
                        VerificationToWaitForChoice::Received
                    }
                    xtce::VerifierEnumerationType::Accepted => {
                        VerificationToWaitForChoice::Accepted
                    }
                    xtce::VerifierEnumerationType::Queued => VerificationToWaitForChoice::Queued,
                    xtce::VerifierEnumerationType::Executing => {
                        VerificationToWaitForChoice::Executing
                    }
                    xtce::VerifierEnumerationType::Complete => {
                        VerificationToWaitForChoice::Complete
                    }
                    xtce::VerifierEnumerationType::Failed => VerificationToWaitForChoice::Failed,
                },
                verification_progress_percentage: interlock
                    .verification_progress_percentage
                    .map(|p| p.to_string())
                    .unwrap_or_default(),
                suspendable: if interlock.suspendable {
                    SuspendableChoice::True
                } else {
                    SuspendableChoice::False
                },
            },
            None => Self {
                scope_to_space_system: String::new(),
                verification_to_wait_for: VerificationToWaitForChoice::None,
                verification_progress_percentage: String::new(),
                suspendable: SuspendableChoice::False,
            },
        }
    }

    fn apply_to(&self, interlock: &mut Option<xtce::InterlockType>) {
        if self.verification_to_wait_for == VerificationToWaitForChoice::None {
            *interlock = None;
            return;
        }
        let verification_to_wait_for = match self.verification_to_wait_for {
            VerificationToWaitForChoice::Release => xtce::VerifierEnumerationType::Release,
            VerificationToWaitForChoice::TransferredToRange => {
                xtce::VerifierEnumerationType::TransferredToRange
            }
            VerificationToWaitForChoice::SentFromRange => {
                xtce::VerifierEnumerationType::SentFromRange
            }
            VerificationToWaitForChoice::Received => xtce::VerifierEnumerationType::Received,
            VerificationToWaitForChoice::Accepted => xtce::VerifierEnumerationType::Accepted,
            VerificationToWaitForChoice::Queued => xtce::VerifierEnumerationType::Queued,
            VerificationToWaitForChoice::Executing => xtce::VerifierEnumerationType::Executing,
            VerificationToWaitForChoice::Complete => xtce::VerifierEnumerationType::Complete,
            VerificationToWaitForChoice::Failed => xtce::VerifierEnumerationType::Failed,
            VerificationToWaitForChoice::None => unreachable!(),
        };
        let progress = self.verification_progress_percentage.parse::<f64>().ok();
        *interlock = Some(xtce::InterlockType {
            scope_to_space_system: optional_value(self.scope_to_space_system.clone()),
            verification_to_wait_for,
            verification_progress_percentage: progress,
            suspendable: self.suspendable == SuspendableChoice::True,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerifierStageChoice {
    Release,
    Received,
    Accepted,
    Queued,
    Execution,
    Complete,
    Failed,
    TransferredToRange,
    SentFromRange,
}

impl VerifierStageChoice {
    const VARIANTS: &'static [Self] = &[
        Self::Release,
        Self::Execution,
        Self::Complete,
        Self::Received,
        Self::Accepted,
        Self::Queued,
        Self::Failed,
        Self::TransferredToRange,
        Self::SentFromRange,
    ];
}

impl std::fmt::Display for VerifierStageChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Release => "Release",
            Self::Received => "Received",
            Self::Accepted => "Accepted",
            Self::Queued => "Queued",
            Self::Execution => "Execution",
            Self::Complete => "Complete",
            Self::Failed => "Failed",
            Self::TransferredToRange => "Transferred to range",
            Self::SentFromRange => "Sent from range",
        };
        write!(f, "{s}")
    }
}

impl_select_item!(VerifierStageChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum PercentCompleteChoice {
    None,
    #[strum(serialize = "Fixed value")]
    Fixed,
    #[strum(serialize = "Dynamic value")]
    Dynamic,
}
impl_select_item!(PercentCompleteChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum VerifierConditionChoice {
    #[strum(serialize = "Match criteria")]
    MatchCriteria,
    #[strum(serialize = "Container reference")]
    ContainerRef,
    #[strum(serialize = "Parameter value change")]
    ParameterValueChange,
}
impl_select_item!(VerifierConditionChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum VerifierWindowChoice {
    #[strum(serialize = "Fixed window")]
    Fixed,
    #[strum(serialize = "Algorithmic window")]
    Algorithms,
}
impl_select_item!(VerifierWindowChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum WindowRelativeToChoice {
    #[strum(serialize = "Command release")]
    CommandRelease,
    #[strum(serialize = "Last verifier passed")]
    LastVerifierPassed,
}
impl_select_item!(WindowRelativeToChoice);

pub(super) struct VerifierListForm {
    rows: Vec<Entity<VerifierRowForm>>,
}

struct VerifierRowForm {
    stage: Entity<SelectState<Vec<VerifierStageChoice>>>,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    long_description: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    condition_kind: Entity<SelectState<Vec<VerifierConditionChoice>>>,
    criteria: Entity<MessageCriteriaForm>,
    container_ref: Entity<InputState>,
    change_parameter_ref: Entity<InputState>,
    change_value: Entity<InputState>,
    window_kind: Entity<SelectState<Vec<VerifierWindowChoice>>>,
    time_to_start: Entity<InputState>,
    time_to_stop: Entity<InputState>,
    window_relative_to: Entity<SelectState<Vec<WindowRelativeToChoice>>>,
    start_check_algorithm: Entity<InputAlgorithmForm>,
    stop_time_algorithm: Entity<InputAlgorithmForm>,
    argument_restrictions: Entity<InputState>,
    return_parameter: Entity<InputState>,
    percent_complete_kind: Entity<SelectState<Vec<PercentCompleteChoice>>>,
    percent_complete_fixed: Entity<InputState>,
    percent_complete_dynamic: Entity<DynamicValueForm>,
}

#[cfg(test)]
pub(super) struct VerifierModel {
    stage: VerifierStageChoice,
    name: String,
    short_description: String,
    parameter: String,
    operator: ComparisonOperatorChoice,
    value: String,
    time_to_stop: String,
    return_parameter: String,
    percent_complete_kind: PercentCompleteChoice,
    percent_complete_fixed: String,
    percent_complete_dynamic: Option<xtce::DynamicValueType>,
}

enum VerifierConditionRef<'a> {
    Comparison(&'a xtce::ComparisonType),
    ComparisonList(&'a xtce::ComparisonListType),
    ContainerRef(&'a xtce::ContainerRefType),
    ParameterValueChange(&'a xtce::ParameterValueChangeType),
    CustomAlgorithm(&'a xtce::InputAlgorithmType),
    BooleanExpression(&'a xtce::BooleanExpressionType),
}

struct VerifierSource<'a> {
    stage: VerifierStageChoice,
    name: Option<&'a str>,
    short_description: Option<&'a str>,
    long_description: Option<&'a str>,
    alias_set: Option<&'a xtce::AliasSetType>,
    ancillary_data_set: Option<&'a xtce::AncillaryDataSetType>,
    condition: Option<VerifierConditionRef<'a>>,
    check_window: Option<&'a xtce::CheckWindowType>,
    check_window_algorithms: Option<&'a xtce::CheckWindowAlgorithmsType>,
    argument_restrictions: Option<&'a xtce::ArgumentAssignmentListType>,
    percent_complete: Option<&'a xtce::PercentCompleteType>,
    return_parameter: Option<&'a xtce::ParameterRefType>,
}

enum VerifierCondition {
    MatchCriteria(xtce::ContextMatchType),
    ContainerRef(xtce::ContainerRefType),
    ParameterValueChange(xtce::ParameterValueChangeType),
}

enum VerifierWindow {
    Fixed(xtce::CheckWindowType),
    Algorithms(xtce::CheckWindowAlgorithmsType),
}

struct VerifierCommon {
    name: Option<String>,
    short_description: Option<String>,
    long_description: Option<String>,
    alias_set: Option<xtce::AliasSetType>,
    ancillary_data_set: Option<xtce::AncillaryDataSetType>,
    condition: VerifierCondition,
    window: VerifierWindow,
    argument_restrictions: Option<xtce::ArgumentAssignmentListType>,
}

macro_rules! verifier_source {
    ($verifier:expr, $stage:expr, $content:ident, $percent:expr, $return_parameter:expr) => {{
        let verifier = $verifier;
        let mut source = VerifierSource {
            stage: $stage,
            name: verifier.name.as_deref(),
            short_description: verifier.short_description.as_deref(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            condition: None,
            check_window: None,
            check_window_algorithms: None,
            argument_restrictions: None,
            percent_complete: $percent,
            return_parameter: $return_parameter,
        };
        for item in &verifier.content {
            #[allow(unreachable_patterns)]
            match item {
                xtce::$content::LongDescription(value) => {
                    source.long_description = Some(value.as_str())
                }
                xtce::$content::AliasSet(value) => source.alias_set = Some(value),
                xtce::$content::AncillaryDataSet(value) => source.ancillary_data_set = Some(value),
                xtce::$content::Comparison(value) => {
                    source.condition = Some(VerifierConditionRef::Comparison(value))
                }
                xtce::$content::ComparisonList(value) => {
                    source.condition = Some(VerifierConditionRef::ComparisonList(value))
                }
                xtce::$content::ContainerRef(value) => {
                    source.condition = Some(VerifierConditionRef::ContainerRef(value))
                }
                xtce::$content::ParameterValueChange(value) => {
                    source.condition = Some(VerifierConditionRef::ParameterValueChange(value))
                }
                xtce::$content::CustomAlgorithm(value) => {
                    source.condition = Some(VerifierConditionRef::CustomAlgorithm(value))
                }
                xtce::$content::BooleanExpression(value) => {
                    source.condition = Some(VerifierConditionRef::BooleanExpression(value))
                }
                xtce::$content::CheckWindow(value) => source.check_window = Some(value),
                xtce::$content::CheckWindowAlgorithms(value) => {
                    source.check_window_algorithms = Some(value)
                }
                xtce::$content::ArgumentRestrictionList(value) => {
                    source.argument_restrictions = Some(value)
                }
                _ => {}
            }
        }
        source
    }};
}

macro_rules! build_verifier {
    ($function:ident, $verifier:ident, $content:ident) => {
        fn $function(common: VerifierCommon) -> xtce::$verifier {
            let mut content = Vec::new();
            if let Some(value) = common.long_description {
                content.push(xtce::$content::LongDescription(value));
            }
            if let Some(value) = common.alias_set {
                content.push(xtce::$content::AliasSet(value));
            }
            if let Some(value) = common.ancillary_data_set {
                content.push(xtce::$content::AncillaryDataSet(value));
            }
            match common.condition {
                VerifierCondition::MatchCriteria(xtce::ContextMatchType::Comparison(value)) => {
                    content.push(xtce::$content::Comparison(value))
                }
                VerifierCondition::MatchCriteria(xtce::ContextMatchType::ComparisonList(value)) => {
                    content.push(xtce::$content::ComparisonList(value))
                }
                VerifierCondition::MatchCriteria(xtce::ContextMatchType::BooleanExpression(
                    value,
                )) => content.push(xtce::$content::BooleanExpression(value)),
                VerifierCondition::MatchCriteria(xtce::ContextMatchType::CustomAlgorithm(
                    value,
                )) => content.push(xtce::$content::CustomAlgorithm(value)),
                VerifierCondition::ContainerRef(value) => {
                    content.push(xtce::$content::ContainerRef(value))
                }
                VerifierCondition::ParameterValueChange(value) => {
                    content.push(xtce::$content::ParameterValueChange(value))
                }
            }
            match common.window {
                VerifierWindow::Fixed(value) => content.push(xtce::$content::CheckWindow(value)),
                VerifierWindow::Algorithms(value) => {
                    content.push(xtce::$content::CheckWindowAlgorithms(value))
                }
            }
            if let Some(value) = common.argument_restrictions {
                content.push(xtce::$content::ArgumentRestrictionList(value));
            }
            xtce::$verifier {
                short_description: common.short_description,
                name: common.name,
                content,
            }
        }
    };
}

build_verifier!(
    build_received_verifier_full,
    ReceivedVerifierType,
    ReceivedVerifierTypeContent
);
build_verifier!(
    build_accepted_verifier_full,
    AcceptedVerifierType,
    AcceptedVerifierTypeContent
);
build_verifier!(
    build_queued_verifier_full,
    QueuedVerifierType,
    QueuedVerifierTypeContent
);
build_verifier!(
    build_execution_verifier_full,
    ExecutionVerifierType,
    ExecutionVerifierTypeContent
);
build_verifier!(
    build_complete_verifier_full,
    CompleteVerifierType,
    CompleteVerifierTypeContent
);
build_verifier!(
    build_failed_verifier_full,
    FailedVerifierType,
    FailedVerifierTypeContent
);
build_verifier!(
    build_transferred_verifier_full,
    TransferredToRangeVerifierType,
    TransferredToRangeVerifierTypeContent
);
build_verifier!(
    build_sent_verifier_full,
    SentFromRangeVerifierType,
    SentFromRangeVerifierTypeContent
);

impl VerifierListForm {
    pub(super) fn new(
        verifier_set: Option<&xtce::VerifierSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = verifier_entities_from_set(verifier_set, window, cx);
        cx.new(|_| Self { rows })
    }

    pub(super) fn load(
        &mut self,
        verifier_set: Option<&xtce::VerifierSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = verifier_entities_from_set(verifier_set, window, cx);
        cx.notify();
    }

    pub(super) fn apply_to_verifier_set(&self, set: &mut Option<xtce::VerifierSetType>, cx: &App) {
        let mut received = None;
        let mut accepted = None;
        let mut queued = None;
        let mut execution = Vec::new();
        let mut complete = Vec::new();
        let mut failed = None;
        let mut transferred = None;
        let mut sent = None;

        for row in &self.rows {
            let row_read = row.read(cx);
            let stage = selected_value(&row_read.stage, VerifierStageChoice::Execution, cx);
            if stage == VerifierStageChoice::Release {
                continue;
            }
            let mut alias_set = None;
            row_read.alias_set.apply_to_option(&mut alias_set, cx);
            let mut ancillary_data_set = None;
            row_read
                .ancillary_data_set
                .apply_to_option(&mut ancillary_data_set, cx);
            let condition = match selected_value(
                &row_read.condition_kind,
                VerifierConditionChoice::MatchCriteria,
                cx,
            ) {
                VerifierConditionChoice::MatchCriteria => {
                    VerifierCondition::MatchCriteria(row_read.criteria.read(cx).context_match(cx))
                }
                VerifierConditionChoice::ContainerRef => {
                    VerifierCondition::ContainerRef(xtce::ContainerRefType {
                        container_ref: value(&row_read.container_ref, cx).trim().to_owned(),
                    })
                }
                VerifierConditionChoice::ParameterValueChange => {
                    VerifierCondition::ParameterValueChange(xtce::ParameterValueChangeType {
                        parameter_ref: xtce::ParameterRefType {
                            parameter_ref: value(&row_read.change_parameter_ref, cx)
                                .trim()
                                .to_owned(),
                        },
                        change: xtce::ChangeValueType {
                            value: value(&row_read.change_value, cx)
                                .trim()
                                .parse()
                                .unwrap_or_default(),
                        },
                    })
                }
            };
            let window =
                match selected_value(&row_read.window_kind, VerifierWindowChoice::Fixed, cx) {
                    VerifierWindowChoice::Fixed => VerifierWindow::Fixed(xtce::CheckWindowType {
                        time_to_start_checking: optional_value(
                            value(&row_read.time_to_start, cx).trim().to_owned(),
                        ),
                        time_to_stop_checking: value(&row_read.time_to_stop, cx).trim().to_owned(),
                        time_window_is_relative_to: match selected_value(
                            &row_read.window_relative_to,
                            WindowRelativeToChoice::LastVerifierPassed,
                            cx,
                        ) {
                            WindowRelativeToChoice::CommandRelease => {
                                xtce::TimeWindowIsRelativeToType::CommandRelease
                            }
                            WindowRelativeToChoice::LastVerifierPassed => {
                                xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed
                            }
                        },
                    }),
                    VerifierWindowChoice::Algorithms => {
                        VerifierWindow::Algorithms(xtce::CheckWindowAlgorithmsType {
                            start_check: row_read.start_check_algorithm.read(cx).algorithm(cx),
                            stop_time: row_read.stop_time_algorithm.read(cx).algorithm(cx),
                        })
                    }
                };
            let argument_restrictions =
                decode_assignments(&value(&row_read.argument_restrictions, cx));
            let common = VerifierCommon {
                name: optional_value(value(&row_read.name, cx).trim().to_owned()),
                short_description: optional_value(
                    value(&row_read.short_description, cx).trim().to_owned(),
                ),
                long_description: optional_value(value(&row_read.long_description, cx)),
                alias_set,
                ancillary_data_set,
                condition,
                window,
                argument_restrictions: (!argument_restrictions.is_empty()).then_some(
                    xtce::ArgumentAssignmentListType {
                        argument_assignment: argument_restrictions,
                    },
                ),
            };
            let return_parameter = value(&row_read.return_parameter, cx).trim().to_owned();
            let percent_complete = match selected_value(
                &row_read.percent_complete_kind,
                PercentCompleteChoice::None,
                cx,
            ) {
                PercentCompleteChoice::None => None,
                PercentCompleteChoice::Fixed => value(&row_read.percent_complete_fixed, cx)
                    .trim()
                    .parse()
                    .ok()
                    .map(xtce::PercentCompleteType::FixedValue),
                PercentCompleteChoice::Dynamic => Some(xtce::PercentCompleteType::DynamicValue(
                    row_read.percent_complete_dynamic.read(cx).value(cx),
                )),
            };
            match stage {
                VerifierStageChoice::Received => {
                    received = Some(build_received_verifier_full(common));
                }
                VerifierStageChoice::Accepted => {
                    accepted = Some(build_accepted_verifier_full(common));
                }
                VerifierStageChoice::Queued => {
                    queued = Some(build_queued_verifier_full(common));
                }
                VerifierStageChoice::Execution => {
                    let mut verifier = build_execution_verifier_full(common);
                    if let Some(value) = percent_complete {
                        verifier
                            .content
                            .push(xtce::ExecutionVerifierTypeContent::PercentComplete(value));
                    }
                    execution.push(verifier);
                }
                VerifierStageChoice::Complete => {
                    let mut verifier = build_complete_verifier_full(common);
                    if !return_parameter.is_empty() {
                        verifier
                            .content
                            .push(xtce::CompleteVerifierTypeContent::ReturnParmRef(
                                xtce::ParameterRefType {
                                    parameter_ref: return_parameter,
                                },
                            ));
                    }
                    complete.push(verifier);
                }
                VerifierStageChoice::Failed => {
                    let mut verifier = build_failed_verifier_full(common);
                    if !return_parameter.is_empty() {
                        verifier
                            .content
                            .push(xtce::FailedVerifierTypeContent::ReturnParmRef(
                                xtce::ParameterRefType {
                                    parameter_ref: return_parameter,
                                },
                            ));
                    }
                    failed = Some(verifier);
                }
                VerifierStageChoice::TransferredToRange => {
                    transferred = Some(build_transferred_verifier_full(common));
                }
                VerifierStageChoice::SentFromRange => {
                    sent = Some(build_sent_verifier_full(common));
                }
                VerifierStageChoice::Release => unreachable!(),
            }
        }

        let has_any = received.is_some()
            || accepted.is_some()
            || queued.is_some()
            || !execution.is_empty()
            || !complete.is_empty()
            || failed.is_some()
            || transferred.is_some()
            || sent.is_some();

        if has_any {
            *set = Some(xtce::VerifierSetType {
                transferred_to_range_verifier: transferred,
                sent_from_range_verifier: sent,
                received_verifier: received,
                accepted_verifier: accepted,
                queued_verifier: queued,
                execution_verifier: execution,
                complete_verifier: complete,
                failed_verifier: failed,
            });
        } else {
            *set = None;
        }
    }
}

#[cfg(test)]
fn verifier_models(set: Option<&xtce::VerifierSetType>) -> Vec<VerifierModel> {
    let mut models = Vec::new();
    let Some(set) = set else { return models };

    if let Some(v) = &set.received_verifier {
        if let Some(m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::Received,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::ReceivedVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::ReceivedVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            models.push(m);
        }
    }
    if let Some(v) = &set.accepted_verifier {
        if let Some(m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::Accepted,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::AcceptedVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::AcceptedVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            models.push(m);
        }
    }
    if let Some(v) = &set.queued_verifier {
        if let Some(m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::Queued,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::QueuedVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::QueuedVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            models.push(m);
        }
    }
    for v in &set.execution_verifier {
        if let Some(mut m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::Execution,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::ExecutionVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::ExecutionVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            if let Some(percent_complete) = v.content.iter().find_map(|item| match item {
                xtce::ExecutionVerifierTypeContent::PercentComplete(value) => Some(value),
                _ => None,
            }) {
                match percent_complete {
                    xtce::PercentCompleteType::FixedValue(value) => {
                        m.percent_complete_kind = PercentCompleteChoice::Fixed;
                        m.percent_complete_fixed = value.to_string();
                    }
                    xtce::PercentCompleteType::DynamicValue(value) => {
                        m.percent_complete_kind = PercentCompleteChoice::Dynamic;
                        m.percent_complete_dynamic = Some(copy_dynamic_value(value));
                    }
                }
            }
            models.push(m);
        }
    }
    for v in &set.complete_verifier {
        if let Some(mut m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::Complete,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::CompleteVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::CompleteVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            m.return_parameter = v
                .content
                .iter()
                .find_map(|item| match item {
                    xtce::CompleteVerifierTypeContent::ReturnParmRef(reference) => {
                        Some(reference.parameter_ref.clone())
                    }
                    _ => None,
                })
                .unwrap_or_default();
            models.push(m);
        }
    }
    if let Some(v) = &set.failed_verifier {
        if let Some(mut m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::Failed,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::FailedVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::FailedVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            m.return_parameter = v
                .content
                .iter()
                .find_map(|item| match item {
                    xtce::FailedVerifierTypeContent::ReturnParmRef(reference) => {
                        Some(reference.parameter_ref.clone())
                    }
                    _ => None,
                })
                .unwrap_or_default();
            models.push(m);
        }
    }
    if let Some(v) = &set.transferred_to_range_verifier {
        if let Some(m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::TransferredToRange,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::TransferredToRangeVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::TransferredToRangeVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            models.push(m);
        }
    }
    if let Some(v) = &set.sent_from_range_verifier {
        if let Some(m) = parse_verifier_items(
            &v.content,
            VerifierStageChoice::SentFromRange,
            v.name.as_deref(),
            v.short_description.as_deref(),
            |item| match item {
                xtce::SentFromRangeVerifierTypeContent::Comparison(c) => Some(c),
                _ => None,
            },
            |item| match item {
                xtce::SentFromRangeVerifierTypeContent::CheckWindow(w) => Some(w),
                _ => None,
            },
        ) {
            models.push(m);
        }
    }

    models
}

#[cfg(test)]
fn parse_verifier_items<T>(
    items: &[T],
    stage: VerifierStageChoice,
    name: Option<&str>,
    short_description: Option<&str>,
    get_comp: impl Fn(&T) -> Option<&xtce::ComparisonType>,
    get_win: impl Fn(&T) -> Option<&xtce::CheckWindowType>,
) -> Option<VerifierModel> {
    let mut param = String::new();
    let mut op = ComparisonOperatorChoice::Equal;
    let mut val = String::new();
    let mut stop = String::new();

    for item in items {
        if let Some(c) = get_comp(item) {
            param = c.parameter_ref.clone();
            op = operator_choice_from_str(&c.comparison_operator);
            val = c.value.clone();
        }
        if let Some(w) = get_win(item) {
            stop = w.time_to_stop_checking.clone();
        }
    }

    (!param.is_empty()).then_some(VerifierModel {
        stage,
        name: name.unwrap_or_default().to_owned(),
        short_description: short_description.unwrap_or_default().to_owned(),
        parameter: param,
        operator: op,
        value: val,
        time_to_stop: stop,
        return_parameter: String::new(),
        percent_complete_kind: PercentCompleteChoice::None,
        percent_complete_fixed: String::new(),
        percent_complete_dynamic: None,
    })
}

#[cfg(test)]
fn copy_dynamic_value(value: &xtce::DynamicValueType) -> xtce::DynamicValueType {
    xtce::DynamicValueType {
        parameter_instance_ref: xtce::ParameterInstanceRefType {
            parameter_ref: value.parameter_instance_ref.parameter_ref.clone(),
            instance: value.parameter_instance_ref.instance,
            use_calibrated_value: value.parameter_instance_ref.use_calibrated_value,
        },
        linear_adjustment: value.linear_adjustment.as_ref().map(|adjustment| {
            xtce::LinearAdjustmentType {
                slope: adjustment.slope,
                intercept: adjustment.intercept,
            }
        }),
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_received_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
) -> xtce::ReceivedVerifierType {
    let mut content = vec![xtce::ReceivedVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
            value: val,
        },
    )];
    if !time_to_stop.is_empty() {
        content.push(xtce::ReceivedVerifierTypeContent::CheckWindow(
            xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: time_to_stop,
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            },
        ));
    }
    xtce::ReceivedVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_accepted_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
) -> xtce::AcceptedVerifierType {
    let mut content = vec![xtce::AcceptedVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
            value: val,
        },
    )];
    if !time_to_stop.is_empty() {
        content.push(xtce::AcceptedVerifierTypeContent::CheckWindow(
            xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: time_to_stop,
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            },
        ));
    }
    xtce::AcceptedVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_queued_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
) -> xtce::QueuedVerifierType {
    let mut content = vec![xtce::QueuedVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
            value: val,
        },
    )];
    if !time_to_stop.is_empty() {
        content.push(xtce::QueuedVerifierTypeContent::CheckWindow(
            xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: time_to_stop,
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            },
        ));
    }
    xtce::QueuedVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
fn build_execution_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
    percent_complete: Option<xtce::PercentCompleteType>,
) -> xtce::ExecutionVerifierType {
    let mut content = vec![xtce::ExecutionVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
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
    if let Some(percent_complete) = percent_complete {
        content.push(xtce::ExecutionVerifierTypeContent::PercentComplete(
            percent_complete,
        ));
    }
    xtce::ExecutionVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_complete_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
    return_parameter: String,
) -> xtce::CompleteVerifierType {
    let mut content = vec![xtce::CompleteVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
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
    if !return_parameter.is_empty() {
        content.push(xtce::CompleteVerifierTypeContent::ReturnParmRef(
            xtce::ParameterRefType {
                parameter_ref: return_parameter,
            },
        ));
    }
    xtce::CompleteVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_failed_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
    return_parameter: String,
) -> xtce::FailedVerifierType {
    let mut content = vec![xtce::FailedVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
            value: val,
        },
    )];
    if !time_to_stop.is_empty() {
        content.push(xtce::FailedVerifierTypeContent::CheckWindow(
            xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: time_to_stop,
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            },
        ));
    }
    if !return_parameter.is_empty() {
        content.push(xtce::FailedVerifierTypeContent::ReturnParmRef(
            xtce::ParameterRefType {
                parameter_ref: return_parameter,
            },
        ));
    }
    xtce::FailedVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_transferred_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
) -> xtce::TransferredToRangeVerifierType {
    let mut content = vec![xtce::TransferredToRangeVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
            value: val,
        },
    )];
    if !time_to_stop.is_empty() {
        content.push(xtce::TransferredToRangeVerifierTypeContent::CheckWindow(
            xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: time_to_stop,
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            },
        ));
    }
    xtce::TransferredToRangeVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn build_sent_verifier(
    param: String,
    op: &str,
    val: String,
    time_to_stop: String,
) -> xtce::SentFromRangeVerifierType {
    let mut content = vec![xtce::SentFromRangeVerifierTypeContent::Comparison(
        xtce::ComparisonType {
            parameter_ref: param,
            instance: 0,
            use_calibrated_value: true,
            comparison_operator: op.to_owned(),
            value: val,
        },
    )];
    if !time_to_stop.is_empty() {
        content.push(xtce::SentFromRangeVerifierTypeContent::CheckWindow(
            xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: time_to_stop,
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            },
        ));
    }
    xtce::SentFromRangeVerifierType {
        short_description: None,
        name: None,
        content,
    }
}

fn verifier_entities_from_set(
    set: Option<&xtce::VerifierSetType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<VerifierRowForm>> {
    let mut rows = Vec::new();
    let Some(set) = set else { return rows };
    if let Some(value) = &set.transferred_to_range_verifier {
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::TransferredToRange,
                TransferredToRangeVerifierTypeContent,
                None,
                None
            ),
            window,
            cx,
        ));
    }
    if let Some(value) = &set.sent_from_range_verifier {
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::SentFromRange,
                SentFromRangeVerifierTypeContent,
                None,
                None
            ),
            window,
            cx,
        ));
    }
    if let Some(value) = &set.received_verifier {
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::Received,
                ReceivedVerifierTypeContent,
                None,
                None
            ),
            window,
            cx,
        ));
    }
    if let Some(value) = &set.accepted_verifier {
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::Accepted,
                AcceptedVerifierTypeContent,
                None,
                None
            ),
            window,
            cx,
        ));
    }
    if let Some(value) = &set.queued_verifier {
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::Queued,
                QueuedVerifierTypeContent,
                None,
                None
            ),
            window,
            cx,
        ));
    }
    for value in &set.execution_verifier {
        let percent_complete = value.content.iter().find_map(|item| match item {
            xtce::ExecutionVerifierTypeContent::PercentComplete(value) => Some(value),
            _ => None,
        });
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::Execution,
                ExecutionVerifierTypeContent,
                percent_complete,
                None
            ),
            window,
            cx,
        ));
    }
    for value in &set.complete_verifier {
        let return_parameter = value.content.iter().find_map(|item| match item {
            xtce::CompleteVerifierTypeContent::ReturnParmRef(value) => Some(value),
            _ => None,
        });
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::Complete,
                CompleteVerifierTypeContent,
                None,
                return_parameter
            ),
            window,
            cx,
        ));
    }
    if let Some(value) = &set.failed_verifier {
        let return_parameter = value.content.iter().find_map(|item| match item {
            xtce::FailedVerifierTypeContent::ReturnParmRef(value) => Some(value),
            _ => None,
        });
        rows.push(verifier_entity_from_source(
            verifier_source!(
                value,
                VerifierStageChoice::Failed,
                FailedVerifierTypeContent,
                None,
                return_parameter
            ),
            window,
            cx,
        ));
    }
    rows
}

fn verifier_entity_from_source(
    source: VerifierSource<'_>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<VerifierRowForm> {
    let stage = select(VerifierStageChoice::VARIANTS, source.stage, window, cx);
    let name = input(source.name.unwrap_or_default(), false, window, cx);
    let short_description = input(
        source.short_description.unwrap_or_default(),
        false,
        window,
        cx,
    );
    let long_description = input(
        source.long_description.unwrap_or_default(),
        true,
        window,
        cx,
    );
    let alias_set = AliasSetForm::new(source.alias_set, window, cx);
    let ancillary_data_set = AncillaryDataSetForm::new(source.ancillary_data_set, window, cx);
    let (condition_kind_value, criteria_ref, container_ref_value, change_parameter, change_value) =
        match source.condition {
            Some(VerifierConditionRef::Comparison(value)) => (
                VerifierConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::Comparison(value)),
                "",
                "",
                String::new(),
            ),
            Some(VerifierConditionRef::ComparisonList(value)) => (
                VerifierConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::ComparisonList(value)),
                "",
                "",
                String::new(),
            ),
            Some(VerifierConditionRef::BooleanExpression(value)) => (
                VerifierConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::BooleanExpression(value)),
                "",
                "",
                String::new(),
            ),
            Some(VerifierConditionRef::CustomAlgorithm(value)) => (
                VerifierConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::CustomAlgorithm(value)),
                "",
                "",
                String::new(),
            ),
            Some(VerifierConditionRef::ContainerRef(value)) => (
                VerifierConditionChoice::ContainerRef,
                None,
                value.container_ref.as_str(),
                "",
                String::new(),
            ),
            Some(VerifierConditionRef::ParameterValueChange(value)) => (
                VerifierConditionChoice::ParameterValueChange,
                None,
                "",
                value.parameter_ref.parameter_ref.as_str(),
                value.change.value.to_string(),
            ),
            None => (
                VerifierConditionChoice::MatchCriteria,
                None,
                "",
                "",
                String::new(),
            ),
        };
    let condition_kind = select(
        VerifierConditionChoice::VARIANTS,
        condition_kind_value,
        window,
        cx,
    );
    let criteria = MessageCriteriaForm::new_ref(criteria_ref, window, cx);
    let container_ref = input(container_ref_value, false, window, cx);
    let change_parameter_ref = input(change_parameter, false, window, cx);
    let change_value = input(&change_value, false, window, cx);
    let window_kind_value = if source.check_window_algorithms.is_some() {
        VerifierWindowChoice::Algorithms
    } else {
        VerifierWindowChoice::Fixed
    };
    let window_kind = select(
        VerifierWindowChoice::VARIANTS,
        window_kind_value,
        window,
        cx,
    );
    let time_to_start = input(
        source
            .check_window
            .and_then(|value| value.time_to_start_checking.as_deref())
            .unwrap_or_default(),
        false,
        window,
        cx,
    );
    let time_to_stop = input(
        source
            .check_window
            .map(|value| value.time_to_stop_checking.as_str())
            .unwrap_or_default(),
        false,
        window,
        cx,
    );
    let relative_to = match source
        .check_window
        .map(|value| &value.time_window_is_relative_to)
    {
        Some(xtce::TimeWindowIsRelativeToType::CommandRelease) => {
            WindowRelativeToChoice::CommandRelease
        }
        _ => WindowRelativeToChoice::LastVerifierPassed,
    };
    let window_relative_to = select(WindowRelativeToChoice::VARIANTS, relative_to, window, cx);
    let start_check_algorithm = InputAlgorithmForm::new(
        source
            .check_window_algorithms
            .map(|value| &value.start_check),
        window,
        cx,
    );
    let stop_time_algorithm = InputAlgorithmForm::new(
        source.check_window_algorithms.map(|value| &value.stop_time),
        window,
        cx,
    );
    let argument_restrictions = input(
        &encode_assignments(source.argument_restrictions),
        true,
        window,
        cx,
    );
    let return_parameter = input(
        source
            .return_parameter
            .map(|value| value.parameter_ref.as_str())
            .unwrap_or_default(),
        false,
        window,
        cx,
    );
    let (percent_complete_kind_value, percent_complete_fixed_value, percent_dynamic) =
        match source.percent_complete {
            Some(xtce::PercentCompleteType::FixedValue(value)) => {
                (PercentCompleteChoice::Fixed, value.to_string(), None)
            }
            Some(xtce::PercentCompleteType::DynamicValue(value)) => {
                (PercentCompleteChoice::Dynamic, String::new(), Some(value))
            }
            None => (PercentCompleteChoice::None, String::new(), None),
        };
    let percent_complete_kind = select(
        PercentCompleteChoice::VARIANTS,
        percent_complete_kind_value,
        window,
        cx,
    );
    let percent_complete_fixed = input(&percent_complete_fixed_value, false, window, cx);
    let percent_complete_dynamic = DynamicValueForm::new(percent_dynamic, window, cx);

    cx.new(|_| VerifierRowForm {
        stage,
        name,
        short_description,
        long_description,
        alias_set,
        ancillary_data_set,
        condition_kind,
        criteria,
        container_ref,
        change_parameter_ref,
        change_value,
        window_kind,
        time_to_start,
        time_to_stop,
        window_relative_to,
        start_check_algorithm,
        stop_time_algorithm,
        argument_restrictions,
        return_parameter,
        percent_complete_kind,
        percent_complete_fixed,
        percent_complete_dynamic,
    })
}

fn default_verifier_entity(
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<VerifierRowForm> {
    verifier_entity_from_source(
        VerifierSource {
            stage: VerifierStageChoice::Execution,
            name: None,
            short_description: None,
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            condition: None,
            check_window: None,
            check_window_algorithms: None,
            argument_restrictions: None,
            percent_complete: None,
            return_parameter: None,
        },
        window,
        cx,
    )
}

pub(super) struct ParameterToSetListForm {
    rows: Vec<Entity<ParameterToSetRowForm>>,
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ParameterToSetContentChoice {
    #[strum(serialize = "New value")]
    NewValue,
    Derivation,
}
impl_select_item!(ParameterToSetContentChoice);

struct ParameterToSetRowForm {
    parameter: Entity<InputState>,
    value: Entity<InputState>,
    content_kind: Entity<SelectState<Vec<ParameterToSetContentChoice>>>,
    derivation: Entity<RpnOperationForm>,
    trigger: Entity<SelectState<Vec<VerifierStageChoice>>>,
}

struct ParameterToSetModel {
    parameter: String,
    value: String,
    content_kind: ParameterToSetContentChoice,
    derivation: Vec<RpnOperationEntry>,
    trigger: VerifierStageChoice,
}

impl ParameterToSetListForm {
    pub(super) fn new(
        list: Option<&xtce::ParameterToSetListType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let models = parameter_to_set_models(list);
        cx.new(move |cx| Self {
            rows: parameter_to_set_entities(models, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        list: Option<&xtce::ParameterToSetListType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = parameter_to_set_entities(parameter_to_set_models(list), window, cx);
        cx.notify();
    }

    pub(super) fn to_list(&self, cx: &App) -> Option<xtce::ParameterToSetListType> {
        let items = self
            .rows
            .iter()
            .filter_map(|row| {
                let row_read = row.read(cx);
                let param = value(&row_read.parameter, cx).trim().to_owned();
                if param.is_empty() {
                    return None;
                }
                let val = value(&row_read.value, cx);
                let trigger = selected_value(&row_read.trigger, VerifierStageChoice::Complete, cx);
                let content = match selected_value(
                    &row_read.content_kind,
                    ParameterToSetContentChoice::NewValue,
                    cx,
                ) {
                    ParameterToSetContentChoice::NewValue => {
                        xtce::ParameterToSetTypeContent::NewValue(val)
                    }
                    ParameterToSetContentChoice::Derivation => {
                        xtce::ParameterToSetTypeContent::Derivation(argument_math_operation(
                            row_read.derivation.read(cx).entries(cx),
                        ))
                    }
                };
                Some(xtce::ParameterToSetType {
                    parameter_ref: param,
                    set_on_verification: stage_choice_to_verifier(trigger),
                    content,
                })
            })
            .collect::<Vec<_>>();

        (!items.is_empty()).then_some(xtce::ParameterToSetListType {
            parameter_to_set: items,
        })
    }
}

fn parameter_to_set_models(
    list: Option<&xtce::ParameterToSetListType>,
) -> Vec<ParameterToSetModel> {
    list.map(|l| {
        l.parameter_to_set
            .iter()
            .map(|item| {
                let (content_kind, new_value, derivation) = match &item.content {
                    xtce::ParameterToSetTypeContent::NewValue(val) => (
                        ParameterToSetContentChoice::NewValue,
                        val.clone(),
                        Vec::new(),
                    ),
                    xtce::ParameterToSetTypeContent::Derivation(operation) => (
                        ParameterToSetContentChoice::Derivation,
                        String::new(),
                        rpn_entries_from_argument_math(operation),
                    ),
                };
                ParameterToSetModel {
                    parameter: item.parameter_ref.clone(),
                    value: new_value,
                    content_kind,
                    derivation,
                    trigger: stage_choice_from_verifier(&item.set_on_verification),
                }
            })
            .collect()
    })
    .unwrap_or_default()
}

fn parameter_to_set_entities(
    models: Vec<ParameterToSetModel>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<ParameterToSetRowForm>> {
    models
        .into_iter()
        .map(|model| {
            let parameter = input(&model.parameter, false, window, cx);
            let value_input = input(&model.value, false, window, cx);
            let content_kind = select(
                ParameterToSetContentChoice::VARIANTS,
                model.content_kind,
                window,
                cx,
            );
            let derivation = RpnOperationForm::new_argument(model.derivation, window, cx);
            let trigger = select(VerifierStageChoice::VARIANTS, model.trigger, window, cx);

            cx.new(|_| ParameterToSetRowForm {
                parameter,
                value: value_input,
                content_kind,
                derivation,
                trigger,
            })
        })
        .collect()
}

impl Render for ParameterToSetListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Parameter to set list"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(
                                        self.rows.len(),
                                        "parameter",
                                        "parameters",
                                    )),
                            ),
                    )
                    .child(
                        Button::new("add-parameter-to-set")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add parameter to set")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let parameter = input("", false, window, cx);
                                let value_input = input("", false, window, cx);
                                let content_kind = select(
                                    ParameterToSetContentChoice::VARIANTS,
                                    ParameterToSetContentChoice::NewValue,
                                    window,
                                    cx,
                                );
                                let derivation =
                                    RpnOperationForm::new_argument(Vec::new(), window, cx);
                                let trigger = select(
                                    VerifierStageChoice::VARIANTS,
                                    VerifierStageChoice::Complete,
                                    window,
                                    cx,
                                );
                                this.rows.push(cx.new(|_| ParameterToSetRowForm {
                                    parameter,
                                    value: value_input,
                                    content_kind,
                                    derivation,
                                    trigger,
                                }));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(super::empty_list_state("No parameters to set defined.", cx))
            })
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                super::compact_list_row(cx)
                    .items_start()
                    .child(
                        div()
                            .flex_1()
                            .child(field("Parameter", "", &row_read.parameter, cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(field("New value", "", &row_read.value, cx)),
                    )
                    .child(div().w(px(140.)).child(select_field(
                        "Value source",
                        "",
                        &row_read.content_kind,
                        cx,
                    )))
                    .child(div().w(px(140.)).child(select_field(
                        "Verification trigger",
                        "",
                        &row_read.trigger,
                        cx,
                    )))
                    .child(super::action_field(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new(format!("parameter-to-set-options-{index}"))
                                    .small()
                                    .ghost()
                                    .icon(IconName::Ellipsis)
                                    .tooltip("Value source options")
                                    .on_click({
                                        let row = row.clone();
                                        move |_, window, cx| {
                                            open_parameter_to_set_options(row.clone(), window, cx);
                                        }
                                    }),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-parameter-to-set-{index}"),
                                    "Remove parameter assignment",
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.rows.remove(index);
                                        cx.notify();
                                    },
                                )),
                            ),
                    ))
            }))
    }
}

fn open_parameter_to_set_options(
    editor: Entity<ParameterToSetRowForm>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title("Parameter value source")
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                let row = editor.read(cx);
                let kind =
                    selected_value(&row.content_kind, ParameterToSetContentChoice::NewValue, cx);
                let derivation = row.derivation.clone();
                content.child(
                    super::form_dialog_content()
                        .when(kind == ParameterToSetContentChoice::Derivation, |form| {
                            form.child(derivation)
                        })
                        .when(kind == ParameterToSetContentChoice::NewValue, |form| {
                            form.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("The new value is edited directly in the list."),
                            )
                        }),
                )
            })
    });
}

fn rpn_entries_from_argument_math(
    operation: &xtce::ArgumentMathOperationType,
) -> Vec<RpnOperationEntry> {
    operation
        .content
        .iter()
        .map(|entry| match entry {
            xtce::ArgumentMathOperationTypeContent::ValueOperand(value) => {
                RpnOperationEntry::Value(value.clone())
            }
            xtce::ArgumentMathOperationTypeContent::ThisParameterOperand(value) => {
                RpnOperationEntry::ThisParameter(value.clone())
            }
            xtce::ArgumentMathOperationTypeContent::Operator(value) => {
                RpnOperationEntry::Operator(value.clone())
            }
            xtce::ArgumentMathOperationTypeContent::ParameterInstanceRefOperand(value) => {
                RpnOperationEntry::ParameterInstance {
                    parameter_ref: value.parameter_ref.clone(),
                    instance: value.instance,
                    use_calibrated_value: value.use_calibrated_value,
                }
            }
            xtce::ArgumentMathOperationTypeContent::ArgumentInstanceRefOperand(value) => {
                RpnOperationEntry::ArgumentInstance {
                    argument_ref: value.argument_ref.clone(),
                    use_calibrated_value: value.use_calibrated_value,
                }
            }
        })
        .collect()
}

fn argument_math_operation(entries: Vec<RpnOperationEntry>) -> xtce::ArgumentMathOperationType {
    xtce::ArgumentMathOperationType {
        content: entries
            .into_iter()
            .map(|entry| match entry {
                RpnOperationEntry::Value(value) => {
                    xtce::ArgumentMathOperationTypeContent::ValueOperand(value)
                }
                RpnOperationEntry::ThisParameter(value) => {
                    xtce::ArgumentMathOperationTypeContent::ThisParameterOperand(value)
                }
                RpnOperationEntry::Operator(value) => {
                    xtce::ArgumentMathOperationTypeContent::Operator(value)
                }
                RpnOperationEntry::ParameterInstance {
                    parameter_ref,
                    instance,
                    use_calibrated_value,
                } => xtce::ArgumentMathOperationTypeContent::ParameterInstanceRefOperand(
                    xtce::ParameterInstanceRefType {
                        parameter_ref,
                        instance,
                        use_calibrated_value,
                    },
                ),
                RpnOperationEntry::ArgumentInstance {
                    argument_ref,
                    use_calibrated_value,
                } => xtce::ArgumentMathOperationTypeContent::ArgumentInstanceRefOperand(
                    xtce::ArgumentInstanceRefType {
                        argument_ref,
                        use_calibrated_value,
                    },
                ),
            })
            .collect(),
    }
}

pub(super) struct ParametersToSuspendAlarmsOnSetForm {
    rows: Vec<Entity<ParameterToSuspendAlarmsOnRowForm>>,
}

struct ParameterToSuspendAlarmsOnRowForm {
    parameter: Entity<InputState>,
    suspense_time: Entity<InputState>,
    trigger: Entity<SelectState<Vec<VerifierStageChoice>>>,
}

struct ParameterToSuspendAlarmsOnModel {
    parameter: String,
    suspense_time: String,
    trigger: VerifierStageChoice,
}

impl ParametersToSuspendAlarmsOnSetForm {
    pub(super) fn new(
        list: Option<&xtce::ParametersToSuspendAlarmsOnSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let models = parameter_to_suspend_alarms_on_models(list);
        cx.new(move |cx| Self {
            rows: parameter_to_suspend_alarms_on_entities(models, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        list: Option<&xtce::ParametersToSuspendAlarmsOnSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = parameter_to_suspend_alarms_on_entities(
            parameter_to_suspend_alarms_on_models(list),
            window,
            cx,
        );
        cx.notify();
    }

    pub(super) fn to_list(&self, cx: &App) -> Option<xtce::ParametersToSuspendAlarmsOnSetType> {
        let items = self
            .rows
            .iter()
            .filter_map(|row| {
                let row_read = row.read(cx);
                let param = value(&row_read.parameter, cx).trim().to_owned();
                if param.is_empty() {
                    return None;
                }
                let suspense_time = value(&row_read.suspense_time, cx).trim().to_owned();
                let trigger = selected_value(&row_read.trigger, VerifierStageChoice::Release, cx);
                Some(xtce::ParameterToSuspendAlarmsOnType {
                    parameter_ref: param,
                    suspense_time,
                    verifier_to_trigger_on: stage_choice_to_verifier(trigger),
                })
            })
            .collect::<Vec<_>>();

        (!items.is_empty()).then_some(xtce::ParametersToSuspendAlarmsOnSetType {
            parameter_to_suspend_alarms_on: items,
        })
    }
}

fn parameter_to_suspend_alarms_on_models(
    list: Option<&xtce::ParametersToSuspendAlarmsOnSetType>,
) -> Vec<ParameterToSuspendAlarmsOnModel> {
    list.map(|l| {
        l.parameter_to_suspend_alarms_on
            .iter()
            .map(|item| ParameterToSuspendAlarmsOnModel {
                parameter: item.parameter_ref.clone(),
                suspense_time: item.suspense_time.clone(),
                trigger: stage_choice_from_verifier(&item.verifier_to_trigger_on),
            })
            .collect()
    })
    .unwrap_or_default()
}

fn parameter_to_suspend_alarms_on_entities(
    models: Vec<ParameterToSuspendAlarmsOnModel>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<ParameterToSuspendAlarmsOnRowForm>> {
    models
        .into_iter()
        .map(|model| {
            let parameter = input(&model.parameter, false, window, cx);
            let suspense_time = input(&model.suspense_time, false, window, cx);
            let trigger = select(VerifierStageChoice::VARIANTS, model.trigger, window, cx);

            cx.new(|_| ParameterToSuspendAlarmsOnRowForm {
                parameter,
                suspense_time,
                trigger,
            })
        })
        .collect()
}

impl Render for ParametersToSuspendAlarmsOnSetForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
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
                                    .child("Parameters to suspend alarms on set"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(
                                        self.rows.len(),
                                        "parameter",
                                        "parameters",
                                    )),
                            ),
                    )
                    .child(
                        Button::new("add-parameter-to-suspend-alarm")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add alarm suspension")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let parameter = input("", false, window, cx);
                                let suspense_time = input("", false, window, cx);
                                let trigger = select(
                                    VerifierStageChoice::VARIANTS,
                                    VerifierStageChoice::Release,
                                    window,
                                    cx,
                                );
                                this.rows
                                    .push(cx.new(|_| ParameterToSuspendAlarmsOnRowForm {
                                        parameter,
                                        suspense_time,
                                        trigger,
                                    }));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(super::empty_list_state("No alarm suspensions defined.", cx))
            })
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                super::compact_list_row(cx)
                    .items_start()
                    .child(
                        div()
                            .flex_1()
                            .child(field("Parameter", "", &row_read.parameter, cx)),
                    )
                    .child(div().w(px(140.)).child(field(
                        "Suspense time (e.g. PT30S)",
                        "",
                        &row_read.suspense_time,
                        cx,
                    )))
                    .child(div().w(px(140.)).child(select_field(
                        "Verification trigger",
                        "",
                        &row_read.trigger,
                        cx,
                    )))
                    .child(super::action_field(
                        super::row_remove_button(
                            format!("remove-suspend-alarm-{index}"),
                            "Remove alarm suspension",
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.rows.remove(index);
                            cx.notify();
                        })),
                    ))
            }))
    }
}

fn stage_choice_from_verifier(v: &xtce::VerifierEnumerationType) -> VerifierStageChoice {
    match v {
        xtce::VerifierEnumerationType::Release => VerifierStageChoice::Release,
        xtce::VerifierEnumerationType::TransferredToRange => {
            VerifierStageChoice::TransferredToRange
        }
        xtce::VerifierEnumerationType::SentFromRange => VerifierStageChoice::SentFromRange,
        xtce::VerifierEnumerationType::Received => VerifierStageChoice::Received,
        xtce::VerifierEnumerationType::Accepted => VerifierStageChoice::Accepted,
        xtce::VerifierEnumerationType::Queued => VerifierStageChoice::Queued,
        xtce::VerifierEnumerationType::Executing => VerifierStageChoice::Execution,
        xtce::VerifierEnumerationType::Complete => VerifierStageChoice::Complete,
        xtce::VerifierEnumerationType::Failed => VerifierStageChoice::Failed,
    }
}

fn stage_choice_to_verifier(v: VerifierStageChoice) -> xtce::VerifierEnumerationType {
    match v {
        VerifierStageChoice::Release => xtce::VerifierEnumerationType::Release,
        VerifierStageChoice::TransferredToRange => {
            xtce::VerifierEnumerationType::TransferredToRange
        }
        VerifierStageChoice::SentFromRange => xtce::VerifierEnumerationType::SentFromRange,
        VerifierStageChoice::Received => xtce::VerifierEnumerationType::Received,
        VerifierStageChoice::Accepted => xtce::VerifierEnumerationType::Accepted,
        VerifierStageChoice::Queued => xtce::VerifierEnumerationType::Queued,
        VerifierStageChoice::Execution => xtce::VerifierEnumerationType::Executing,
        VerifierStageChoice::Complete => xtce::VerifierEnumerationType::Complete,
        VerifierStageChoice::Failed => xtce::VerifierEnumerationType::Failed,
    }
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
                            .child(div().text_sm().font_medium().child("Command verifiers"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(
                                        self.rows.len(),
                                        "verifier",
                                        "verifiers",
                                    )),
                            ),
                    )
                    .child(
                        Button::new("add-verifier")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add verifier")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(default_verifier_entity(window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(super::empty_list_state("No command verifiers defined.", cx))
            })
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                let condition_kind = selected_value(
                    &row_read.condition_kind,
                    VerifierConditionChoice::MatchCriteria,
                    cx,
                );
                let window_kind =
                    selected_value(&row_read.window_kind, VerifierWindowChoice::Fixed, cx);
                super::detail_list_card(cx)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Verifier {}", index + 1)),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new(format!("verifier-options-{index}"))
                                            .small()
                                            .ghost()
                                            .icon(IconName::Ellipsis)
                                            .tooltip("Optional and stage-specific settings")
                                            .on_click({
                                                let row = row.clone();
                                                move |_, window, cx| {
                                                    open_verifier_options(row.clone(), window, cx);
                                                }
                                            }),
                                    )
                                    .child(
                                        super::row_remove_button(
                                            format!("remove-verifier-{index}"),
                                            "Remove verifier",
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                this.rows.remove(index);
                                                cx.notify();
                                            }),
                                        ),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .items_start()
                            .child(div().w(px(180.)).child(select_field(
                                "Stage",
                                "Required",
                                &row_read.stage,
                                cx,
                            )))
                            .child(div().flex_1().child(field(
                                "Name (optional)",
                                "",
                                &row_read.name,
                                cx,
                            ))),
                    )
                    .child(field(
                        "Short description",
                        "Optional",
                        &row_read.short_description,
                        cx,
                    ))
                    .child(
                        v_flex()
                            .w_full()
                            .gap_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child("Verification condition"),
                            )
                            .child(select_field(
                                "Condition type",
                                "Required",
                                &row_read.condition_kind,
                                cx,
                            ))
                            .when(
                                condition_kind == VerifierConditionChoice::MatchCriteria,
                                |form| form.child(row_read.criteria.clone()),
                            )
                            .when(
                                condition_kind == VerifierConditionChoice::ContainerRef,
                                |form| {
                                    form.child(field(
                                        "Container reference",
                                        "Required",
                                        &row_read.container_ref,
                                        cx,
                                    ))
                                },
                            )
                            .when(
                                condition_kind == VerifierConditionChoice::ParameterValueChange,
                                |form| {
                                    form.child(field(
                                        "Parameter reference",
                                        "Required",
                                        &row_read.change_parameter_ref,
                                        cx,
                                    ))
                                    .child(field(
                                        "Change value",
                                        "Required; floating-point delta",
                                        &row_read.change_value,
                                        cx,
                                    ))
                                },
                            ),
                    )
                    .child(
                        v_flex()
                            .w_full()
                            .gap_3()
                            .child(div().text_sm().font_medium().child("Check window"))
                            .child(select_field(
                                "Window type",
                                "Required",
                                &row_read.window_kind,
                                cx,
                            ))
                            .when(window_kind == VerifierWindowChoice::Fixed, |form| {
                                form.child(
                                    h_flex()
                                        .w_full()
                                        .gap_3()
                                        .items_end()
                                        .child(div().flex_1().child(field(
                                            "Start checking after (optional)",
                                            "",
                                            &row_read.time_to_start,
                                            cx,
                                        )))
                                        .child(div().flex_1().child(field(
                                            "Stop checking after",
                                            "Required",
                                            &row_read.time_to_stop,
                                            cx,
                                        )))
                                        .child(div().w(px(200.)).child(select_field(
                                            "Window relative to",
                                            "Required",
                                            &row_read.window_relative_to,
                                            cx,
                                        ))),
                                )
                            })
                            .when(window_kind == VerifierWindowChoice::Algorithms, |form| {
                                form.child(
                                    v_flex()
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_medium()
                                                .child("Start-check algorithm"),
                                        )
                                        .child(row_read.start_check_algorithm.clone())
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_medium()
                                                .child("Stop-time algorithm"),
                                        )
                                        .child(row_read.stop_time_algorithm.clone()),
                                )
                            }),
                    )
            }))
    }
}

fn open_verifier_options(editor: Entity<VerifierRowForm>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, _| {
        let editor = editor.clone();
        dialog
            .title("Verifier options")
            .w(px(super::FORM_DIALOG_WIDTH))
            .content(move |content, _, cx| {
                let row = editor.read(cx);
                let stage = selected_value(&row.stage, VerifierStageChoice::Execution, cx);
                let long_description = row.long_description.clone();
                let argument_restrictions = row.argument_restrictions.clone();
                let return_parameter = row.return_parameter.clone();
                let percent_complete_kind = row.percent_complete_kind.clone();
                let percent_complete_fixed = row.percent_complete_fixed.clone();
                let percent_complete_dynamic = row.percent_complete_dynamic.clone();
                let percent_complete_choice =
                    selected_value(&percent_complete_kind, PercentCompleteChoice::None, cx);
                content.child(
                    super::form_dialog_content()
                        .child(div().text_sm().font_medium().child("Optional metadata"))
                        .child(field("Long description", "Optional", &long_description, cx))
                        .child(row.alias_set.render(cx))
                        .child(row.ancillary_data_set.render(cx))
                        .child(field(
                            "Argument restrictions",
                            "Optional; one “argument = value” entry per line",
                            &argument_restrictions,
                            cx,
                        ))
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .child("Stage-specific options"),
                        )
                        .when(stage == VerifierStageChoice::Execution, |form| {
                            form.child(select_field(
                                "Percent complete source",
                                "Optional",
                                &percent_complete_kind,
                                cx,
                            ))
                            .when(
                                percent_complete_choice == PercentCompleteChoice::Fixed,
                                |form| {
                                    form.child(field(
                                        "Percent complete",
                                        "Required; numeric percentage",
                                        &percent_complete_fixed,
                                        cx,
                                    ))
                                },
                            )
                            .when(
                                percent_complete_choice == PercentCompleteChoice::Dynamic,
                                |form| form.child(percent_complete_dynamic),
                            )
                        })
                        .when(
                            matches!(
                                stage,
                                VerifierStageChoice::Complete | VerifierStageChoice::Failed
                            ),
                            |form| {
                                form.child(field(
                                    "Return parameter reference",
                                    "Optional",
                                    &return_parameter,
                                    cx,
                                ))
                            },
                        )
                        .when(
                            !matches!(
                                stage,
                                VerifierStageChoice::Execution
                                    | VerifierStageChoice::Complete
                                    | VerifierStageChoice::Failed
                            ),
                            |form| {
                                form.child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(
                                            "This verifier stage has no stage-specific options.",
                                        ),
                                )
                            },
                        ),
                )
            })
    });
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum TransmissionConstraintConditionChoice {
    None,
    #[strum(serialize = "Match criteria")]
    MatchCriteria,
    #[strum(serialize = "Argument restrictions")]
    ArgumentRestrictions,
}
impl_select_item!(TransmissionConstraintConditionChoice);

struct TransmissionConstraintRowForm {
    condition_kind: Entity<SelectState<Vec<TransmissionConstraintConditionChoice>>>,
    criteria: Entity<MessageCriteriaForm>,
    argument_restrictions: Entity<InputState>,
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
        let rows = constraint_entities(list, window, cx);
        cx.new(move |_| Self { rows })
    }

    pub(super) fn load(
        &mut self,
        list: Option<&xtce::TransmissionConstraintListType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = constraint_entities(list, window, cx);
        cx.notify();
    }

    pub(super) fn to_list(&self, cx: &App) -> Option<xtce::TransmissionConstraintListType> {
        let constraints = self
            .rows
            .iter()
            .map(|row| {
                let row = row.read(cx);
                let content = match selected_value(
                    &row.condition_kind,
                    TransmissionConstraintConditionChoice::MatchCriteria,
                    cx,
                ) {
                    TransmissionConstraintConditionChoice::None => None,
                    TransmissionConstraintConditionChoice::MatchCriteria => Some(
                        constraint_content_from_match(row.criteria.read(cx).context_match(cx)),
                    ),
                    TransmissionConstraintConditionChoice::ArgumentRestrictions => Some(
                        xtce::TransmissionConstraintTypeContent::ArgumentRestrictionList(
                            xtce::ArgumentAssignmentListType {
                                argument_assignment: decode_assignments(&value(
                                    &row.argument_restrictions,
                                    cx,
                                )),
                            },
                        ),
                    ),
                };
                let time_out = optional_value(value(&row.time_out, cx));
                let suspendable = row
                    .suspendable
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or(SuspendableChoice::False)
                    == SuspendableChoice::True;

                xtce::TransmissionConstraintType {
                    time_out,
                    suspendable,
                    content,
                }
            })
            .collect::<Vec<_>>();

        (!constraints.is_empty()).then_some(xtce::TransmissionConstraintListType {
            transmission_constraint: constraints,
        })
    }
}

struct TransmissionConstraintValues<'a> {
    condition_kind: TransmissionConstraintConditionChoice,
    criteria: Option<MessageCriteriaRef<'a>>,
    argument_restrictions: Option<&'a xtce::ArgumentAssignmentListType>,
    time_out: &'a str,
    suspendable: SuspendableChoice,
}

impl<'a> TransmissionConstraintValues<'a> {
    fn from_constraint(constraint: &'a xtce::TransmissionConstraintType) -> Self {
        let (condition_kind, criteria, argument_restrictions) = match &constraint.content {
            Some(xtce::TransmissionConstraintTypeContent::Comparison(value)) => (
                TransmissionConstraintConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::Comparison(value)),
                None,
            ),
            Some(xtce::TransmissionConstraintTypeContent::ComparisonList(value)) => (
                TransmissionConstraintConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::ComparisonList(value)),
                None,
            ),
            Some(xtce::TransmissionConstraintTypeContent::BooleanExpression(value)) => (
                TransmissionConstraintConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::BooleanExpression(value)),
                None,
            ),
            Some(xtce::TransmissionConstraintTypeContent::CustomAlgorithm(value)) => (
                TransmissionConstraintConditionChoice::MatchCriteria,
                Some(MessageCriteriaRef::CustomAlgorithm(value)),
                None,
            ),
            Some(xtce::TransmissionConstraintTypeContent::ArgumentRestrictionList(value)) => (
                TransmissionConstraintConditionChoice::ArgumentRestrictions,
                None,
                Some(value),
            ),
            None => (TransmissionConstraintConditionChoice::None, None, None),
        };
        Self {
            condition_kind,
            criteria,
            argument_restrictions,
            time_out: constraint.time_out.as_deref().unwrap_or_default(),
            suspendable: if constraint.suspendable {
                SuspendableChoice::True
            } else {
                SuspendableChoice::False
            },
        }
    }
}

fn constraint_content_from_match(
    criteria: xtce::ContextMatchType,
) -> xtce::TransmissionConstraintTypeContent {
    match criteria {
        xtce::ContextMatchType::Comparison(value) => {
            xtce::TransmissionConstraintTypeContent::Comparison(value)
        }
        xtce::ContextMatchType::ComparisonList(value) => {
            xtce::TransmissionConstraintTypeContent::ComparisonList(value)
        }
        xtce::ContextMatchType::BooleanExpression(value) => {
            xtce::TransmissionConstraintTypeContent::BooleanExpression(value)
        }
        xtce::ContextMatchType::CustomAlgorithm(value) => {
            xtce::TransmissionConstraintTypeContent::CustomAlgorithm(value)
        }
    }
}

fn constraint_entities(
    list: Option<&xtce::TransmissionConstraintListType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<TransmissionConstraintRowForm>> {
    list.into_iter()
        .flat_map(|list| &list.transmission_constraint)
        .map(|constraint| constraint_entity(Some(constraint), window, cx))
        .collect()
}

fn constraint_entity(
    constraint: Option<&xtce::TransmissionConstraintType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<TransmissionConstraintRowForm> {
    let values = constraint.map(TransmissionConstraintValues::from_constraint);
    let condition_kind_value = values
        .as_ref()
        .map(|values| values.condition_kind)
        .unwrap_or(TransmissionConstraintConditionChoice::MatchCriteria);
    let condition_kind = select(
        TransmissionConstraintConditionChoice::VARIANTS,
        condition_kind_value,
        window,
        cx,
    );
    let criteria = MessageCriteriaForm::new_ref(
        values.as_ref().and_then(|values| values.criteria),
        window,
        cx,
    );
    let argument_restrictions = input(
        &encode_assignments(
            values
                .as_ref()
                .and_then(|values| values.argument_restrictions),
        ),
        true,
        window,
        cx,
    );
    let time_out = input(
        values
            .as_ref()
            .map(|values| values.time_out)
            .unwrap_or_default(),
        false,
        window,
        cx,
    );
    let suspendable = select(
        SuspendableChoice::VARIANTS,
        values
            .as_ref()
            .map(|values| values.suspendable)
            .unwrap_or(SuspendableChoice::False),
        window,
        cx,
    );

    cx.new(|_| TransmissionConstraintRowForm {
        condition_kind,
        criteria,
        argument_restrictions,
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
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child("Transmission constraints"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(
                                        self.rows.len(),
                                        "constraint",
                                        "constraints",
                                    )),
                            ),
                    )
                    .child(
                        Button::new("add-transmission-constraint")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add constraint")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(constraint_entity(None, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(super::empty_list_state(
                    "No transmission constraints defined.",
                    cx,
                ))
            })
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                let condition_kind = selected_value(
                    &row_read.condition_kind,
                    TransmissionConstraintConditionChoice::MatchCriteria,
                    cx,
                );
                super::detail_list_card(cx)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Constraint {}", index + 1)),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-transmission-constraint-{index}"),
                                    "Remove transmission constraint",
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.rows.remove(index);
                                        cx.notify();
                                    },
                                )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .items_start()
                            .child(div().flex_1().child(select_field(
                                "Condition type",
                                "Required",
                                &row_read.condition_kind,
                                cx,
                            )))
                            .child(div().flex_1().child(field(
                                "Timeout (optional, e.g. PT5S)",
                                "",
                                &row_read.time_out,
                                cx,
                            )))
                            .child(div().w(px(160.)).child(select_field(
                                "Suspendable",
                                "Required",
                                &row_read.suspendable,
                                cx,
                            ))),
                    )
                    .when(
                        condition_kind == TransmissionConstraintConditionChoice::MatchCriteria,
                        |form| form.child(row_read.criteria.clone()),
                    )
                    .when(
                        condition_kind
                            == TransmissionConstraintConditionChoice::ArgumentRestrictions,
                        |form| {
                            form.child(field(
                                "Argument restrictions",
                                "One “argument | value” entry per line",
                                &row_read.argument_restrictions,
                                cx,
                            ))
                        },
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

fn apply_arguments(list: &mut Option<xtce::ArgumentListType>, rows: &[CommandArgumentData]) {
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
        .iter()
        .map(|row| {
            let mut argument = existing.next().unwrap_or(xtce::ArgumentType {
                short_description: None,
                name: String::new(),
                argument_type_ref: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
            });
            argument.name.clone_from(&row.name);
            argument.argument_type_ref.clone_from(&row.type_ref);
            argument.initial_value = optional_value(row.initial_value.clone());
            argument.short_description = optional_value(row.short_description.clone());
            argument.long_description = optional_value(row.long_description.clone());
            argument.alias_set = AliasSetForm::parse(&row.aliases);
            argument.ancillary_data_set = AncillaryDataSetForm::parse(&row.ancillary_data);
            argument
        })
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
        ArgumentSpec, AssignmentContext, CommandArgumentData, CommandArguments,
        CommandContainerValues, EditableContainerEntry, MetaCommandValues, SignificanceValues,
        ValueRule, apply_arguments, apply_container_entries, apply_steps, command_argument_models,
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
    fn all_argument_metadata_is_editable() {
        let mut list = Some(xtce::ArgumentListType {
            argument: vec![xtce::ArgumentType {
                short_description: None,
                name: "old".to_owned(),
                argument_type_ref: "OldType".to_owned(),
                initial_value: None,
                long_description: Some("preserved".to_owned()),
                alias_set: Some(xtce::AliasSetType {
                    alias: vec![xtce::AliasType {
                        name_space: "legacy".to_owned(),
                        alias: "OLD".to_owned(),
                    }],
                }),
                ancillary_data_set: Some(xtce::AncillaryDataSetType {
                    ancillary_data: vec![xtce::AncillaryDataType {
                        name: "legacy-guide".to_owned(),
                        mime_type: "text/plain".to_owned(),
                        href: None,
                        content: "Legacy guide".to_owned(),
                    }],
                }),
            }],
        });

        let loaded = command_argument_models(list.as_ref());
        assert_eq!(loaded[0].long_description, "preserved");
        assert_eq!(loaded[0].aliases, "legacy = OLD");
        assert_eq!(
            loaded[0].ancillary_data,
            "legacy-guide | text/plain |  | Legacy guide"
        );

        apply_arguments(
            &mut list,
            &[CommandArgumentData {
                name: "mode".to_owned(),
                type_ref: "ModeType".to_owned(),
                initial_value: "SAFE".to_owned(),
                short_description: "Operating mode".to_owned(),
                long_description: "First line\nSecond line".to_owned(),
                aliases: "ops = MODE".to_owned(),
                ancillary_data: "guide | text/plain | https://example.invalid/mode | Mode guide"
                    .to_owned(),
            }],
        );

        let argument = &list.expect("argument list").argument[0];
        assert_eq!(argument.name, "mode");
        assert_eq!(
            argument.long_description.as_deref(),
            Some("First line\nSecond line")
        );
        let aliases = argument.alias_set.as_ref().expect("argument aliases");
        assert_eq!(aliases.alias[0].name_space, "ops");
        assert_eq!(aliases.alias[0].alias, "MODE");
        let ancillary = argument
            .ancillary_data_set
            .as_ref()
            .expect("argument ancillary data");
        assert_eq!(ancillary.ancillary_data[0].name, "guide");
        assert_eq!(
            ancillary.ancillary_data[0].href.as_deref(),
            Some("https://example.invalid/mode")
        );
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
            arguments: Vec::new(),
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
        use super::{
            MessageCriteriaRef, SuspendableChoice, TransmissionConstraintConditionChoice,
            TransmissionConstraintValues,
        };

        let tc = xtce::TransmissionConstraintType {
            time_out: Some("PT5S".to_owned()),
            suspendable: true,
            content: Some(xtce::TransmissionConstraintTypeContent::ComparisonList(
                xtce::ComparisonListType {
                    comparison: vec![xtce::ComparisonType {
                        parameter_ref: "BUS_VOLTAGE".to_owned(),
                        instance: 0,
                        use_calibrated_value: true,
                        comparison_operator: ">=".to_owned(),
                        value: "28.0".to_owned(),
                    }],
                },
            )),
        };

        let values = TransmissionConstraintValues::from_constraint(&tc);
        assert_eq!(
            values.condition_kind,
            TransmissionConstraintConditionChoice::MatchCriteria
        );
        assert!(matches!(
            values.criteria,
            Some(MessageCriteriaRef::ComparisonList(_))
        ));
        assert_eq!(values.time_out, "PT5S");
        assert_eq!(values.suspendable, SuspendableChoice::True);

        let restrictions = xtce::TransmissionConstraintType {
            time_out: None,
            suspendable: false,
            content: Some(
                xtce::TransmissionConstraintTypeContent::ArgumentRestrictionList(
                    xtce::ArgumentAssignmentListType {
                        argument_assignment: vec![xtce::ArgumentAssignmentType {
                            argument_name: "mode".to_owned(),
                            argument_value: "SAFE".to_owned(),
                        }],
                    },
                ),
            ),
        };
        let values = TransmissionConstraintValues::from_constraint(&restrictions);
        assert_eq!(
            values.condition_kind,
            TransmissionConstraintConditionChoice::ArgumentRestrictions
        );
        assert_eq!(
            values
                .argument_restrictions
                .expect("argument restrictions")
                .argument_assignment[0]
                .argument_name,
            "mode"
        );
    }

    #[test]
    fn meta_command_verifiers_roundtrip() {
        use super::{
            ComparisonOperatorChoice, PercentCompleteChoice, VerifierStageChoice,
            build_execution_verifier, verifier_models,
        };

        let exec = xtce::ExecutionVerifierType {
            short_description: Some("Reports execution progress".to_owned()),
            name: Some("execution-progress".to_owned()),
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
                xtce::ExecutionVerifierTypeContent::PercentComplete(
                    xtce::PercentCompleteType::FixedValue(42.5),
                ),
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
                xtce::CompleteVerifierTypeContent::ReturnParmRef(xtce::ParameterRefType {
                    parameter_ref: "COMMAND_RESULT".to_owned(),
                }),
            ],
        };

        let set = xtce::VerifierSetType {
            transferred_to_range_verifier: None,
            sent_from_range_verifier: None,
            received_verifier: None,
            accepted_verifier: None,
            queued_verifier: None,
            execution_verifier: vec![exec],
            complete_verifier: vec![comp],
            failed_verifier: None,
        };

        let models = verifier_models(Some(&set));
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].stage, VerifierStageChoice::Execution);
        assert_eq!(models[0].name, "execution-progress");
        assert_eq!(models[0].short_description, "Reports execution progress");
        assert_eq!(models[0].parameter, "EXEC_STATUS");
        assert_eq!(models[0].operator, ComparisonOperatorChoice::Equal);
        assert_eq!(models[0].value, "RUNNING");
        assert_eq!(models[0].time_to_stop, "PT5S");
        assert_eq!(
            models[0].percent_complete_kind,
            PercentCompleteChoice::Fixed
        );
        assert_eq!(models[0].percent_complete_fixed, "42.5");

        assert_eq!(models[1].stage, VerifierStageChoice::Complete);
        assert_eq!(models[1].parameter, "EXEC_STATUS");
        assert_eq!(models[1].operator, ComparisonOperatorChoice::Equal);
        assert_eq!(models[1].value, "COMPLETED");
        assert_eq!(models[1].time_to_stop, "PT30S");
        assert_eq!(models[1].return_parameter, "COMMAND_RESULT");

        let rebuilt = build_execution_verifier(
            "EXEC_STATUS".to_owned(),
            "==",
            "RUNNING".to_owned(),
            "PT5S".to_owned(),
            Some(xtce::PercentCompleteType::FixedValue(42.5)),
        );
        assert!(rebuilt.content.iter().any(|item| matches!(
            item,
            xtce::ExecutionVerifierTypeContent::PercentComplete(
                xtce::PercentCompleteType::FixedValue(value)
            ) if *value == 42.5
        )));
    }

    #[test]
    fn command_verifier_full_content_is_preserved_in_schema_order() {
        use super::{
            VerifierCommon, VerifierCondition, VerifierWindow, build_received_verifier_full,
        };

        let verifier = build_received_verifier_full(VerifierCommon {
            name: Some("received-check".to_owned()),
            short_description: Some("Checks receipt".to_owned()),
            long_description: Some("Detailed verifier documentation".to_owned()),
            alias_set: Some(xtce::AliasSetType {
                alias: vec![xtce::AliasType {
                    name_space: "ops".to_owned(),
                    alias: "RX".to_owned(),
                }],
            }),
            ancillary_data_set: Some(xtce::AncillaryDataSetType {
                ancillary_data: Vec::new(),
            }),
            condition: VerifierCondition::MatchCriteria(xtce::ContextMatchType::ComparisonList(
                xtce::ComparisonListType {
                    comparison: vec![xtce::ComparisonType {
                        parameter_ref: "STATUS".to_owned(),
                        instance: 1,
                        use_calibrated_value: false,
                        comparison_operator: "==".to_owned(),
                        value: "RECEIVED".to_owned(),
                    }],
                },
            )),
            window: VerifierWindow::Fixed(xtce::CheckWindowType {
                time_to_start_checking: Some("PT1S".to_owned()),
                time_to_stop_checking: "PT10S".to_owned(),
                time_window_is_relative_to: xtce::TimeWindowIsRelativeToType::CommandRelease,
            }),
            argument_restrictions: Some(xtce::ArgumentAssignmentListType {
                argument_assignment: vec![xtce::ArgumentAssignmentType {
                    argument_name: "mode".to_owned(),
                    argument_value: "SAFE".to_owned(),
                }],
            }),
        });

        assert_eq!(verifier.name.as_deref(), Some("received-check"));
        assert!(matches!(
            verifier.content.as_slice(),
            [
                xtce::ReceivedVerifierTypeContent::LongDescription(_),
                xtce::ReceivedVerifierTypeContent::AliasSet(_),
                xtce::ReceivedVerifierTypeContent::AncillaryDataSet(_),
                xtce::ReceivedVerifierTypeContent::ComparisonList(_),
                xtce::ReceivedVerifierTypeContent::CheckWindow(_),
                xtce::ReceivedVerifierTypeContent::ArgumentRestrictionList(_),
            ]
        ));
    }

    #[test]
    fn command_verifier_supports_non_comparison_conditions_and_algorithmic_windows() {
        use super::{
            VerifierCommon, VerifierCondition, VerifierWindow, build_accepted_verifier_full,
            build_queued_verifier_full,
        };

        let algorithm = || xtce::InputAlgorithmType {
            short_description: None,
            name: "window".to_owned(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            algorithm_text: None,
            external_algorithm_set: None,
            input_set: None,
        };
        let common = |condition, window| VerifierCommon {
            name: None,
            short_description: None,
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            condition,
            window,
            argument_restrictions: None,
        };

        let accepted = build_accepted_verifier_full(common(
            VerifierCondition::ContainerRef(xtce::ContainerRefType {
                container_ref: "ACK_PACKET".to_owned(),
            }),
            VerifierWindow::Algorithms(xtce::CheckWindowAlgorithmsType {
                start_check: algorithm(),
                stop_time: algorithm(),
            }),
        ));
        assert!(
            accepted
                .content
                .iter()
                .any(|item| matches!(item, xtce::AcceptedVerifierTypeContent::ContainerRef(_)))
        );
        assert!(accepted.content.iter().any(|item| matches!(
            item,
            xtce::AcceptedVerifierTypeContent::CheckWindowAlgorithms(_)
        )));

        let queued = build_queued_verifier_full(common(
            VerifierCondition::ParameterValueChange(xtce::ParameterValueChangeType {
                parameter_ref: xtce::ParameterRefType {
                    parameter_ref: "COUNTER".to_owned(),
                },
                change: xtce::ChangeValueType { value: 1.0 },
            }),
            VerifierWindow::Fixed(xtce::CheckWindowType {
                time_to_start_checking: None,
                time_to_stop_checking: "PT5S".to_owned(),
                time_window_is_relative_to:
                    xtce::TimeWindowIsRelativeToType::TimeLastVerifierPassed,
            }),
        ));
        assert!(queued.content.iter().any(|item| matches!(
            item,
            xtce::QueuedVerifierTypeContent::ParameterValueChange(_)
        )));
    }

    #[test]
    fn meta_command_interlock_roundtrip() {
        use super::{InterlockValues, SuspendableChoice, VerificationToWaitForChoice};

        let interlock = xtce::InterlockType {
            scope_to_space_system: Some("/Vehicle".to_owned()),
            verification_to_wait_for: xtce::VerifierEnumerationType::Complete,
            verification_progress_percentage: Some(100.0),
            suspendable: true,
        };

        let values = InterlockValues::from_interlock(Some(&interlock));
        assert_eq!(values.scope_to_space_system, "/Vehicle");
        assert_eq!(
            values.verification_to_wait_for,
            VerificationToWaitForChoice::Complete
        );
        assert_eq!(values.verification_progress_percentage, "100");
        assert_eq!(values.suspendable, SuspendableChoice::True);

        let mut applied = None;
        values.apply_to(&mut applied);
        let applied = applied.expect("interlock present");
        assert_eq!(applied.scope_to_space_system.as_deref(), Some("/Vehicle"));
        assert!(matches!(
            applied.verification_to_wait_for,
            xtce::VerifierEnumerationType::Complete
        ));
        assert_eq!(applied.verification_progress_percentage, Some(100.0));
        assert!(applied.suspendable);

        let none_values = InterlockValues::from_interlock(None);
        assert_eq!(
            none_values.verification_to_wait_for,
            VerificationToWaitForChoice::None
        );
        let mut cleared = Some(interlock);
        none_values.apply_to(&mut cleared);
        assert!(cleared.is_none());
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
    fn stream_segment_entry_round_trips_through_the_form() {
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

        let encoded = super::encode_container_entries(&container.entry_list);
        apply_container_entries(&mut container.entry_list, &encoded);

        assert_eq!(container.entry_list.content.len(), 1);
        assert!(matches!(
            &container.entry_list.content[0],
            xtce::CommandContainerEntryListTypeContent::StreamSegmentEntry(entry)
                if entry.stream_ref == "CommandStream"
        ));
    }

    #[test]
    fn indirect_parameter_entry_round_trips_through_the_form() {
        let mut container = default_command_container();
        container.entry_list.content.push(
            xtce::CommandContainerEntryListTypeContent::IndirectParameterRefEntry(
                xtce::ArgumentIndirectParameterRefEntryType {
                    short_description: Some("resolve target".to_owned()),
                    alias_name_space: Some("OPS".to_owned()),
                    location_in_container_in_bits: None,
                    repeat_entry: None,
                    include_condition: None,
                    ancillary_data_set: None,
                    parameter_instance: xtce::ParameterInstanceRefType {
                        parameter_ref: "target_name".to_owned(),
                        instance: -1,
                        use_calibrated_value: false,
                    },
                },
            ),
        );

        let encoded = super::encode_container_entries(&container.entry_list);
        apply_container_entries(&mut container.entry_list, &encoded);

        assert!(matches!(
            &container.entry_list.content[0],
            xtce::CommandContainerEntryListTypeContent::IndirectParameterRefEntry(entry)
                if entry.parameter_instance.parameter_ref == "target_name"
                    && entry.parameter_instance.instance == -1
                    && !entry.parameter_instance.use_calibrated_value
                    && entry.alias_name_space.as_deref() == Some("OPS")
                    && entry.short_description.as_deref() == Some("resolve target")
        ));
    }

    #[test]
    fn array_parameter_entry_round_trips_through_the_form() {
        let mut container = default_command_container();
        container.entry_list.content.push(
            xtce::CommandContainerEntryListTypeContent::ArrayParameterRefEntry(
                xtce::ArgumentArrayParameterRefEntryType {
                    short_description: Some("array slice".to_owned()),
                    parameter_ref: "samples".to_owned(),
                    last_entry_for_this_array_instance: true,
                    content: Some(xtce::ArgumentArrayParameterRefEntryTypeContent {
                        location_in_container_in_bits: None,
                        repeat_entry: None,
                        include_condition: None,
                        ancillary_data_set: None,
                        dimension_list: xtce::DimensionListType {
                            dimension: vec![
                                xtce::DimensionType {
                                    starting_index: xtce::IntegerValueType::FixedValue(0),
                                    ending_index: xtce::IntegerValueType::FixedValue(3),
                                },
                                xtce::DimensionType {
                                    starting_index: xtce::IntegerValueType::FixedValue(1),
                                    ending_index: xtce::IntegerValueType::FixedValue(2),
                                },
                            ],
                        },
                    }),
                },
            ),
        );

        let encoded = super::encode_container_entries(&container.entry_list);
        apply_container_entries(&mut container.entry_list, &encoded);

        let xtce::CommandContainerEntryListTypeContent::ArrayParameterRefEntry(entry) =
            &container.entry_list.content[0]
        else {
            panic!("expected ArrayParameterRefEntry")
        };
        assert_eq!(entry.parameter_ref, "samples");
        assert!(entry.last_entry_for_this_array_instance);
        assert_eq!(
            super::encode_command_dimensions(
                &entry.content.as_ref().expect("dimensions").dimension_list
            ),
            "0..3, 1..2"
        );
        assert_eq!(entry.short_description.as_deref(), Some("array slice"));
    }

    #[test]
    fn array_argument_entry_round_trips_through_the_form() {
        let mut container = default_command_container();
        container.entry_list.content.push(
            xtce::CommandContainerEntryListTypeContent::ArrayArgumentRefEntry(
                xtce::ArgumentArrayArgumentRefEntryType {
                    short_description: Some("argument slice".to_owned()),
                    argument_ref: "samples".to_owned(),
                    last_entry_for_this_array_instance: true,
                    content: Some(xtce::ArgumentArrayArgumentRefEntryTypeContent {
                        location_in_container_in_bits: None,
                        repeat_entry: None,
                        include_condition: None,
                        ancillary_data_set: None,
                        dimension_list: xtce::ArgumentDimensionListType {
                            dimension: vec![xtce::ArgumentDimensionType {
                                starting_index: xtce::ArgumentIntegerValueType::FixedValue(2),
                                ending_index: xtce::ArgumentIntegerValueType::FixedValue(5),
                            }],
                        },
                    }),
                },
            ),
        );

        let encoded = super::encode_container_entries(&container.entry_list);
        apply_container_entries(&mut container.entry_list, &encoded);

        let xtce::CommandContainerEntryListTypeContent::ArrayArgumentRefEntry(entry) =
            &container.entry_list.content[0]
        else {
            panic!("expected ArrayArgumentRefEntry")
        };
        assert_eq!(entry.argument_ref, "samples");
        assert!(entry.last_entry_for_this_array_instance);
        assert_eq!(
            super::encode_command_argument_dimensions(
                &entry.content.as_ref().expect("dimensions").dimension_list
            ),
            "2..5"
        );
        assert_eq!(entry.short_description.as_deref(), Some("argument slice"));
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

    #[test]
    fn command_container_base_restriction_criteria_roundtrip() {
        let mut container = default_command_container();
        container.base_container = Some(xtce::BaseContainerType {
            container_ref: "BaseCmdContainer".to_owned(),
            restriction_criteria: Some(xtce::RestrictionCriteriaType {
                content: Some(xtce::RestrictionCriteriaTypeContent::Comparison(
                    xtce::ComparisonType {
                        parameter_ref: "OpCode".to_owned(),
                        comparison_operator: "==".to_owned(),
                        value: "0x12".to_owned(),
                        instance: 0,
                        use_calibrated_value: true,
                    },
                )),
            }),
        });
        let mut value = Some(container);

        let container_values = CommandContainerValues::from_container(value.as_ref());
        assert_eq!(container_values.base_ref, "BaseCmdContainer");

        container_values.apply_to(&mut value);

        let result = value.expect("command container");
        let base = result.base_container.expect("base container");
        assert_eq!(base.container_ref, "BaseCmdContainer");
        let criteria = base.restriction_criteria.expect("restriction criteria");
        let Some(xtce::RestrictionCriteriaTypeContent::Comparison(cmp)) = criteria.content else {
            panic!("expected Comparison")
        };
        assert_eq!(cmp.parameter_ref, "OpCode");
        assert_eq!(cmp.value, "0x12");
    }

    #[test]
    fn meta_command_parameter_to_set_roundtrip() {
        use super::{
            ParameterToSetContentChoice, VerifierStageChoice, argument_math_operation,
            parameter_to_set_models, rpn_entries_from_argument_math, stage_choice_to_verifier,
        };
        use crate::forms::rpn_operation::RpnOperationEntry;

        let list = xtce::ParameterToSetListType {
            parameter_to_set: vec![
                xtce::ParameterToSetType {
                    parameter_ref: "MODE_PARAM".to_owned(),
                    set_on_verification: xtce::VerifierEnumerationType::Complete,
                    content: xtce::ParameterToSetTypeContent::NewValue("ACTIVE".to_owned()),
                },
                xtce::ParameterToSetType {
                    parameter_ref: "TARGET_PARAM".to_owned(),
                    set_on_verification: xtce::VerifierEnumerationType::Executing,
                    content: xtce::ParameterToSetTypeContent::Derivation(
                        xtce::ArgumentMathOperationType {
                            content: vec![
                                xtce::ArgumentMathOperationTypeContent::ArgumentInstanceRefOperand(
                                    xtce::ArgumentInstanceRefType {
                                        argument_ref: "target".to_owned(),
                                        use_calibrated_value: true,
                                    },
                                ),
                                xtce::ArgumentMathOperationTypeContent::ValueOperand(
                                    "2".to_owned(),
                                ),
                                xtce::ArgumentMathOperationTypeContent::Operator("*".to_owned()),
                            ],
                        },
                    ),
                },
            ],
        };

        let models = parameter_to_set_models(Some(&list));
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].parameter, "MODE_PARAM");
        assert_eq!(models[0].value, "ACTIVE");
        assert_eq!(models[0].trigger, VerifierStageChoice::Complete);
        assert_eq!(
            models[1].content_kind,
            ParameterToSetContentChoice::Derivation
        );
        assert_eq!(
            models[1].derivation,
            vec![
                RpnOperationEntry::ArgumentInstance {
                    argument_ref: "target".to_owned(),
                    use_calibrated_value: true,
                },
                RpnOperationEntry::Value("2".to_owned()),
                RpnOperationEntry::Operator("*".to_owned()),
            ]
        );

        assert!(matches!(
            stage_choice_to_verifier(models[0].trigger),
            xtce::VerifierEnumerationType::Complete
        ));

        let rebuilt = argument_math_operation(models[1].derivation.clone());
        assert_eq!(
            rpn_entries_from_argument_math(&rebuilt),
            models[1].derivation
        );
    }

    #[test]
    fn meta_command_parameters_to_suspend_alarms_roundtrip() {
        use super::{
            VerifierStageChoice, parameter_to_suspend_alarms_on_models, stage_choice_to_verifier,
        };

        let list = xtce::ParametersToSuspendAlarmsOnSetType {
            parameter_to_suspend_alarms_on: vec![xtce::ParameterToSuspendAlarmsOnType {
                parameter_ref: "TEMP_PARAM".to_owned(),
                suspense_time: "PT30S".to_owned(),
                verifier_to_trigger_on: xtce::VerifierEnumerationType::Release,
            }],
        };

        let models = parameter_to_suspend_alarms_on_models(Some(&list));
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].parameter, "TEMP_PARAM");
        assert_eq!(models[0].suspense_time, "PT30S");
        assert_eq!(models[0].trigger, VerifierStageChoice::Release);

        assert!(matches!(
            stage_choice_to_verifier(models[0].trigger),
            xtce::VerifierEnumerationType::Release
        ));
    }
}
