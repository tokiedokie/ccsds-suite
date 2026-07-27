use gpui::{App, Context, Div, Entity, ParentElement, Styled, WeakEntity, Window};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm,
    argument_type::ArgumentTypeForm, command_metadata::CommandMetaDataForm, header::HeaderForm,
    meta_command::MetaCommandForm, parameter::ParameterForm, parameter_type::ParameterTypeForm,
    sequence_container::SequenceContainerForm, service_set::ServiceSetForm,
    space_system::SpaceSystemForm, telemetry_metadata::TelemetryMetaDataForm,
};
use crate::{ElementKind, XtceDocument, XtceEditor};

pub(crate) struct ElementForms {
    editor: WeakEntity<XtceEditor>,
    space_system: SpaceSystemForm,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    header: HeaderForm,
    telemetry_metadata: TelemetryMetaDataForm,
    command_metadata: CommandMetaDataForm,
    parameter: Entity<ParameterForm>,
    parameter_type: Entity<ParameterTypeForm>,
    sequence_container: Entity<SequenceContainerForm>,
    argument_type: Entity<ArgumentTypeForm>,
    meta_command: Entity<MetaCommandForm>,
    service_set: ServiceSetForm,
}

impl ElementForms {
    pub(crate) fn element_title(
        &self,
        kind: ElementKind,
        system: &xtce::SpaceSystem,
        cx: &App,
    ) -> String {
        if let Some(name) = self.draft_name(kind, system, cx) {
            return name;
        }
        match kind {
            ElementKind::TelemetryParameter(index) => telemetry_parameter_set(system)
                .and_then(|set| set.content.get(index))
                .map(parameter_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::CommandParameter(index) => command_parameter_set(system)
                .and_then(|set| set.content.get(index))
                .map(parameter_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::ArgumentType(index) => system
                .command_meta_data
                .as_ref()
                .and_then(|metadata| metadata.argument_type_set.as_ref())
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::argument_type_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::SequenceContainer(index) => telemetry_container_set(system)
                .and_then(|set| set.content.get(index))
                .map(sequence_container_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::MetaCommand(index) => system
                .command_meta_data
                .as_ref()
                .and_then(|metadata| metadata.meta_command_set.as_ref())
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::meta_command_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            _ => kind.label().to_owned(),
        }
    }

    pub(crate) fn draft_name(
        &self,
        kind: ElementKind,
        system: &xtce::SpaceSystem,
        cx: &App,
    ) -> Option<String> {
        match kind {
            ElementKind::SpaceSystem => Some(self.space_system.name(cx)),
            ElementKind::TelemetryParameter(index)
                if matches!(
                    telemetry_parameter_set(system).and_then(|set| set.content.get(index)),
                    Some(xtce::ParameterSetTypeContent::Parameter(_))
                ) =>
            {
                Some(self.parameter.read(cx).name(cx))
            }
            ElementKind::CommandParameter(index)
                if matches!(
                    command_parameter_set(system).and_then(|set| set.content.get(index)),
                    Some(xtce::ParameterSetTypeContent::Parameter(_))
                ) =>
            {
                Some(self.parameter.read(cx).name(cx))
            }
            ElementKind::TelemetryParameterType(_) | ElementKind::CommandParameterType(_) => {
                Some(self.parameter_type.read(cx).name(cx))
            }
            ElementKind::ArgumentType(_) => Some(self.argument_type.read(cx).name(cx)),
            ElementKind::SequenceContainer(_) => Some(self.sequence_container.read(cx).name(cx)),
            ElementKind::MetaCommand(_) => Some(self.meta_command.read(cx).name(cx)),
            _ => None,
        }
    }

    pub(crate) fn render_name_editor(
        &self,
        kind: ElementKind,
        system: &xtce::SpaceSystem,
        cx: &App,
    ) -> Option<Div> {
        match kind {
            ElementKind::SpaceSystem => Some(self.space_system.render_name_editor()),
            ElementKind::TelemetryParameter(index)
                if matches!(
                    telemetry_parameter_set(system).and_then(|set| set.content.get(index)),
                    Some(xtce::ParameterSetTypeContent::Parameter(_))
                ) =>
            {
                Some(self.parameter.read(cx).render_name_editor())
            }
            ElementKind::CommandParameter(index)
                if matches!(
                    command_parameter_set(system).and_then(|set| set.content.get(index)),
                    Some(xtce::ParameterSetTypeContent::Parameter(_))
                ) =>
            {
                Some(self.parameter.read(cx).render_name_editor())
            }
            ElementKind::TelemetryParameterType(_) | ElementKind::CommandParameterType(_) => {
                Some(self.parameter_type.read(cx).render_name_editor())
            }
            ElementKind::ArgumentType(_) => Some(self.argument_type.read(cx).render_name_editor()),
            ElementKind::SequenceContainer(_) => {
                Some(self.sequence_container.read(cx).render_name_editor())
            }
            ElementKind::MetaCommand(_) => Some(self.meta_command.read(cx).render_name_editor()),
            _ => None,
        }
    }

    pub(crate) fn new(
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        Self {
            editor: cx.entity().downgrade(),
            space_system: SpaceSystemForm::new(system, window, cx),
            alias_set: AliasSetForm::new(system.alias_set.as_ref(), window, cx),
            ancillary_data_set: AncillaryDataSetForm::new(
                system.ancillary_data_set.as_ref(),
                window,
                cx,
            ),
            header: HeaderForm::new(system.header.as_ref(), window, cx),
            telemetry_metadata: TelemetryMetaDataForm,
            command_metadata: CommandMetaDataForm,
            parameter: ParameterForm::new(
                telemetry_parameter_set(system).and_then(|set| set.content.first()),
                telemetry_parameter_type_set(system),
                window,
                cx,
            ),
            parameter_type: ParameterTypeForm::new(
                telemetry_parameter_type_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
            sequence_container: SequenceContainerForm::new(
                telemetry_container_set(system).and_then(|set| set.content.first()),
                telemetry_parameter_set(system),
                telemetry_parameter_type_set(system),
                telemetry_container_set(system),
                window,
                cx,
            ),
            argument_type: ArgumentTypeForm::new(
                argument_type_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
            meta_command: MetaCommandForm::new(
                meta_command_set(system).and_then(|set| set.content.first()),
                meta_command_set(system),
                argument_type_set(system),
                window,
                cx,
            ),
            service_set: ServiceSetForm,
        }
    }

    pub(crate) fn load(
        &self,
        kind: ElementKind,
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        match kind {
            ElementKind::SpaceSystem => {
                self.space_system.load(system, window, cx);
                self.alias_set.load(system.alias_set.as_ref(), window, cx);
                self.ancillary_data_set
                    .load(system.ancillary_data_set.as_ref(), window, cx);
                self.header.load(system.header.as_ref(), window, cx);
            }
            ElementKind::TelemetryParameter(index) => {
                self.parameter.update(cx, |form, cx| {
                    form.load(
                        telemetry_parameter_set(system).and_then(|set| set.content.get(index)),
                        telemetry_parameter_type_set(system),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandParameter(index) => {
                self.parameter.update(cx, |form, cx| {
                    form.load(
                        command_parameter_set(system).and_then(|set| set.content.get(index)),
                        command_parameter_type_set(system),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::TelemetryParameterType(index) => {
                self.parameter_type.update(cx, |form, cx| {
                    form.load(
                        telemetry_parameter_type_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::SequenceContainer(index) => {
                self.sequence_container.update(cx, |form, cx| {
                    form.load(
                        telemetry_container_set(system).and_then(|set| set.content.get(index)),
                        telemetry_parameter_set(system),
                        telemetry_parameter_type_set(system),
                        telemetry_container_set(system),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandParameterType(index) => {
                self.parameter_type.update(cx, |form, cx| {
                    form.load(
                        command_parameter_type_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::ArgumentType(index) => {
                self.argument_type.update(cx, |form, cx| {
                    form.load(
                        argument_type_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::MetaCommand(index) => {
                self.meta_command.update(cx, |form, cx| {
                    form.load(
                        meta_command_set(system).and_then(|set| set.content.get(index)),
                        meta_command_set(system),
                        argument_type_set(system),
                        window,
                        cx,
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn apply_to(&self, kind: ElementKind, system: &mut xtce::SpaceSystem, cx: &App) {
        match kind {
            ElementKind::SpaceSystem => {
                self.space_system.apply_to(system, cx);
                self.alias_set.apply_to_option(&mut system.alias_set, cx);
                self.ancillary_data_set
                    .apply_to_option(&mut system.ancillary_data_set, cx);
                self.header.apply_to_option(&mut system.header, cx);
            }
            ElementKind::TelemetryParameter(index) => {
                if let Some(parameter) =
                    telemetry_parameter_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.parameter.read(cx).apply_to(parameter, cx);
                }
            }
            ElementKind::CommandParameter(index) => {
                if let Some(parameter) =
                    command_parameter_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.parameter.read(cx).apply_to(parameter, cx);
                }
            }
            ElementKind::TelemetryParameterType(index) => {
                if let Some(parameter_type) = telemetry_parameter_type_set_mut(system)
                    .and_then(|set| set.content.get_mut(index))
                {
                    self.parameter_type.read(cx).apply_to(parameter_type, cx);
                }
            }
            ElementKind::SequenceContainer(index) => {
                if let Some(container) =
                    telemetry_container_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.sequence_container.read(cx).apply_to(container, cx);
                }
            }
            ElementKind::CommandParameterType(index) => {
                if let Some(parameter_type) = command_parameter_type_set_mut(system)
                    .and_then(|set| set.content.get_mut(index))
                {
                    self.parameter_type.read(cx).apply_to(parameter_type, cx);
                }
            }
            ElementKind::ArgumentType(index) => {
                if let Some(argument_type) =
                    argument_type_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.argument_type.read(cx).apply_to(argument_type, cx);
                }
            }
            ElementKind::MetaCommand(index) => {
                if let Some(command) =
                    meta_command_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.meta_command.read(cx).apply_to(command, cx);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn is_editable(kind: ElementKind) -> bool {
        matches!(
            kind,
            ElementKind::SpaceSystem
                | ElementKind::TelemetryParameter(_)
                | ElementKind::CommandParameter(_)
                | ElementKind::TelemetryParameterType(_)
                | ElementKind::SequenceContainer(_)
                | ElementKind::CommandParameterType(_)
                | ElementKind::ArgumentType(_)
                | ElementKind::MetaCommand(_)
        )
    }

    pub(crate) fn render(&self, kind: ElementKind, system: &xtce::SpaceSystem, cx: &App) -> Div {
        match kind {
            ElementKind::TelemetryParameterSet => self.telemetry_metadata.render(
                kind,
                system.telemetry_meta_data.as_ref(),
                self.editor.clone(),
                cx,
            ),
            ElementKind::CommandParameterSet => self.command_metadata.render(
                kind,
                system.command_meta_data.as_ref(),
                self.editor.clone(),
                cx,
            ),
            ElementKind::TelemetryParameter(_) | ElementKind::CommandParameter(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.parameter.clone())
            }
            ElementKind::TelemetryParameterType(_) | ElementKind::CommandParameterType(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.parameter_type.clone())
            }
            ElementKind::ArgumentType(_) => gpui_component::v_flex()
                .w_full()
                .child(self.argument_type.clone()),
            ElementKind::SequenceContainer(_) => gpui_component::v_flex()
                .w_full()
                .child(self.sequence_container.clone()),
            ElementKind::MetaCommand(_) => gpui_component::v_flex()
                .w_full()
                .child(self.meta_command.clone()),
            ElementKind::TelemetryMetaData
            | ElementKind::TelemetryParameterTypeSet
            | ElementKind::ContainerSet
            | ElementKind::MessageSet
            | ElementKind::TelemetryStreamSet
            | ElementKind::TelemetryAlgorithmSet => self.telemetry_metadata.render(
                kind,
                system.telemetry_meta_data.as_ref(),
                self.editor.clone(),
                cx,
            ),
            ElementKind::CommandMetaData
            | ElementKind::CommandParameterTypeSet
            | ElementKind::ArgumentTypeSet
            | ElementKind::MetaCommandSet
            | ElementKind::CommandContainerSet
            | ElementKind::CommandStreamSet
            | ElementKind::CommandAlgorithmSet => self.command_metadata.render(
                kind,
                system.command_meta_data.as_ref(),
                self.editor.clone(),
                cx,
            ),
            ElementKind::ServiceSet => {
                self.service_set
                    .render(system.service_set.as_ref(), self.editor.clone(), cx)
            }
            ElementKind::SpaceSystem => self.space_system.render_identity(cx),
        }
    }

    pub(crate) fn render_space_system_description(&self, cx: &App) -> Div {
        self.space_system.render_description(cx)
    }

    pub(crate) fn render_space_system_aliases(&self, cx: &App) -> Div {
        self.alias_set.render(cx)
    }

    pub(crate) fn render_space_system_ancillary_data(&self, cx: &App) -> Div {
        self.ancillary_data_set.render(cx)
    }

    pub(crate) fn render_space_system_header(&self, cx: &App) -> Div {
        self.header.render(cx)
    }
}

fn telemetry_parameter_set(system: &xtce::SpaceSystem) -> Option<&xtce::ParameterSetType> {
    system
        .telemetry_meta_data
        .as_ref()
        .and_then(|metadata| metadata.parameter_set.as_ref())
}

fn telemetry_parameter_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::ParameterSetType> {
    system
        .telemetry_meta_data
        .as_mut()
        .and_then(|metadata| metadata.parameter_set.as_mut())
}

fn command_parameter_set(system: &xtce::SpaceSystem) -> Option<&xtce::ParameterSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.parameter_set.as_ref())
}

fn command_parameter_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::ParameterSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.parameter_set.as_mut())
}

fn telemetry_parameter_type_set(system: &xtce::SpaceSystem) -> Option<&xtce::ParameterTypeSetType> {
    system
        .telemetry_meta_data
        .as_ref()
        .and_then(|metadata| metadata.parameter_type_set.as_ref())
}

fn telemetry_container_set(system: &xtce::SpaceSystem) -> Option<&xtce::ContainerSetType> {
    system
        .telemetry_meta_data
        .as_ref()
        .and_then(|metadata| metadata.container_set.as_ref())
}

fn telemetry_container_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::ContainerSetType> {
    system
        .telemetry_meta_data
        .as_mut()
        .and_then(|metadata| metadata.container_set.as_mut())
}

fn telemetry_parameter_type_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::ParameterTypeSetType> {
    system
        .telemetry_meta_data
        .as_mut()
        .and_then(|metadata| metadata.parameter_type_set.as_mut())
}

fn command_parameter_type_set(system: &xtce::SpaceSystem) -> Option<&xtce::ParameterTypeSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.parameter_type_set.as_ref())
}

fn command_parameter_type_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::ParameterTypeSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.parameter_type_set.as_mut())
}

fn argument_type_set(system: &xtce::SpaceSystem) -> Option<&xtce::ArgumentTypeSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.argument_type_set.as_ref())
}

fn argument_type_set_mut(system: &mut xtce::SpaceSystem) -> Option<&mut xtce::ArgumentTypeSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.argument_type_set.as_mut())
}

fn meta_command_set(system: &xtce::SpaceSystem) -> Option<&xtce::MetaCommandSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.meta_command_set.as_ref())
}

fn meta_command_set_mut(system: &mut xtce::SpaceSystem) -> Option<&mut xtce::MetaCommandSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.meta_command_set.as_mut())
}

fn parameter_title(parameter: &xtce::ParameterSetTypeContent) -> String {
    match parameter {
        xtce::ParameterSetTypeContent::Parameter(parameter) => parameter.name.clone(),
        xtce::ParameterSetTypeContent::ParameterRef(parameter) => {
            format!("→ {}", parameter.parameter_ref)
        }
    }
}

fn sequence_container_title(container: &xtce::ContainerSetTypeContent) -> String {
    match container {
        xtce::ContainerSetTypeContent::SequenceContainer(container) => container.name.clone(),
    }
}
