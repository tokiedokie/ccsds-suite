use gpui::{App, Context, Div, Entity, ParentElement, Styled, WeakEntity, Window};

use super::{
    alias_set::AliasSetForm,
    ancillary_data_set::AncillaryDataSetForm,
    argument_type::ArgumentTypeForm,
    command_metadata::CommandMetaDataForm,
    custom_algorithm::CustomAlgorithmForm,
    custom_stream::CustomStreamForm,
    fixed_frame_stream::FixedFrameStreamForm,
    header::HeaderForm,
    math_algorithm::MathAlgorithmForm,
    message::MessageForm,
    message_set::MessageSetForm,
    meta_command::MetaCommandForm,
    parameter::ParameterForm,
    parameter_type::ParameterTypeForm,
    sequence_container::{ReferenceSets, SequenceContainerForm},
    service_set::ServiceForm,
    space_system::SpaceSystemForm,
    telemetry_metadata::TelemetryMetaDataForm,
    variable_frame_stream::VariableFrameStreamForm,
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
    message_set: MessageSetForm,
    message: Entity<MessageForm>,
    fixed_frame_stream: Entity<FixedFrameStreamForm>,
    variable_frame_stream: Entity<VariableFrameStreamForm>,
    custom_stream: Entity<CustomStreamForm>,
    custom_algorithm: Entity<CustomAlgorithmForm>,
    math_algorithm: Entity<MathAlgorithmForm>,
    parameter: Entity<ParameterForm>,
    parameter_type: Entity<ParameterTypeForm>,
    sequence_container: Entity<SequenceContainerForm>,
    argument_type: Entity<ArgumentTypeForm>,
    meta_command: Entity<MetaCommandForm>,
    service: Entity<ServiceForm>,
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
            ElementKind::CommandContainer(index) => command_container_set(system)
                .and_then(|set| set.command_container.get(index))
                .map(|container| container.name.clone())
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::MetaCommand(index) => system
                .command_meta_data
                .as_ref()
                .and_then(|metadata| metadata.meta_command_set.as_ref())
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::meta_command_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::Message(index) => message_set(system)
                .and_then(|set| set.message.get(index))
                .map(|message| message.name.clone())
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::TelemetryFixedFrameStream(index) => telemetry_stream_set(system)
                .and_then(|set| set.content.get(index))
                .map(stream_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::CommandFixedFrameStream(index) => command_stream_set(system)
                .and_then(|set| set.content.get(index))
                .map(stream_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::TelemetryVariableFrameStream(index) => telemetry_stream_set(system)
                .and_then(|set| set.content.get(index))
                .map(stream_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::CommandVariableFrameStream(index) => command_stream_set(system)
                .and_then(|set| set.content.get(index))
                .map(stream_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::TelemetryCustomStream(index) => telemetry_stream_set(system)
                .and_then(|set| set.content.get(index))
                .map(stream_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::CommandCustomStream(index) => command_stream_set(system)
                .and_then(|set| set.content.get(index))
                .map(stream_title)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::TelemetryCustomAlgorithm(index) => telemetry_algorithm_set(system)
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::algorithm_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::CommandCustomAlgorithm(index) => command_algorithm_set(system)
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::algorithm_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::TelemetryMathAlgorithm(index) => telemetry_algorithm_set(system)
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::algorithm_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::CommandMathAlgorithm(index) => command_algorithm_set(system)
                .and_then(|set| set.content.get(index))
                .map(XtceDocument::algorithm_label)
                .unwrap_or_else(|| kind.label().to_owned()),
            ElementKind::Service(index) => service_set(system)
                .and_then(|set| set.service.get(index))
                .map(|service| service.name.clone())
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
            ElementKind::CommandContainer(_) => Some(self.sequence_container.read(cx).name(cx)),
            ElementKind::MetaCommand(_) => Some(self.meta_command.read(cx).name(cx)),
            ElementKind::Message(_) => Some(self.message.read(cx).name(cx)),
            ElementKind::TelemetryFixedFrameStream(_) | ElementKind::CommandFixedFrameStream(_) => {
                Some(self.fixed_frame_stream.read(cx).name(cx))
            }
            ElementKind::TelemetryVariableFrameStream(_)
            | ElementKind::CommandVariableFrameStream(_) => {
                Some(self.variable_frame_stream.read(cx).name(cx))
            }
            ElementKind::TelemetryCustomStream(_) | ElementKind::CommandCustomStream(_) => {
                Some(self.custom_stream.read(cx).name(cx))
            }
            ElementKind::TelemetryCustomAlgorithm(_) | ElementKind::CommandCustomAlgorithm(_) => {
                Some(self.custom_algorithm.read(cx).name(cx))
            }
            ElementKind::TelemetryMathAlgorithm(_) | ElementKind::CommandMathAlgorithm(_) => {
                Some(self.math_algorithm.read(cx).name(cx))
            }
            ElementKind::Service(_) => Some(self.service.read(cx).name(cx)),
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
            ElementKind::CommandContainer(_) => {
                Some(self.sequence_container.read(cx).render_name_editor())
            }
            ElementKind::MetaCommand(_) => Some(self.meta_command.read(cx).render_name_editor()),
            ElementKind::Message(_) => Some(self.message.read(cx).render_name_editor()),
            ElementKind::TelemetryFixedFrameStream(_) | ElementKind::CommandFixedFrameStream(_) => {
                Some(self.fixed_frame_stream.read(cx).render_name_editor())
            }
            ElementKind::TelemetryVariableFrameStream(_)
            | ElementKind::CommandVariableFrameStream(_) => {
                Some(self.variable_frame_stream.read(cx).render_name_editor())
            }
            ElementKind::TelemetryCustomStream(_) | ElementKind::CommandCustomStream(_) => {
                Some(self.custom_stream.read(cx).render_name_editor())
            }
            ElementKind::TelemetryCustomAlgorithm(_) | ElementKind::CommandCustomAlgorithm(_) => {
                Some(self.custom_algorithm.read(cx).render_name_editor())
            }
            ElementKind::TelemetryMathAlgorithm(_) | ElementKind::CommandMathAlgorithm(_) => {
                Some(self.math_algorithm.read(cx).render_name_editor())
            }
            ElementKind::Service(_) => Some(self.service.read(cx).render_name_editor()),
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
            message_set: MessageSetForm::new(
                system
                    .telemetry_meta_data
                    .as_ref()
                    .and_then(|metadata| metadata.message_set.as_ref()),
                window,
                cx,
            ),
            message: MessageForm::new(
                message_set(system).and_then(|set| set.message.first()),
                window,
                cx,
            ),
            fixed_frame_stream: FixedFrameStreamForm::new(
                telemetry_stream_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
            variable_frame_stream: VariableFrameStreamForm::new(
                telemetry_stream_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
            custom_stream: CustomStreamForm::new(
                telemetry_stream_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
            custom_algorithm: CustomAlgorithmForm::new(
                telemetry_algorithm_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
            math_algorithm: MathAlgorithmForm::new(
                telemetry_algorithm_set(system).and_then(|set| set.content.first()),
                window,
                cx,
            ),
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
                ReferenceSets {
                    parameter_set: telemetry_parameter_set(system),
                    parameter_type_set: telemetry_parameter_type_set(system),
                    container_set: telemetry_container_set(system),
                    stream_set: telemetry_stream_set(system),
                },
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
            service: ServiceForm::new(
                service_set(system).and_then(|set| set.service.first()),
                window,
                cx,
            ),
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
                        ReferenceSets {
                            parameter_set: telemetry_parameter_set(system),
                            parameter_type_set: telemetry_parameter_type_set(system),
                            container_set: telemetry_container_set(system),
                            stream_set: telemetry_stream_set(system),
                        },
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandContainer(index) => {
                self.sequence_container.update(cx, |form, cx| {
                    form.load_direct(
                        command_container_set(system)
                            .and_then(|set| set.command_container.get(index)),
                        ReferenceSets {
                            parameter_set: command_parameter_set(system),
                            parameter_type_set: command_parameter_type_set(system),
                            container_set: command_container_set(system),
                            stream_set: command_stream_set(system),
                        },
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
            ElementKind::MessageSet => {
                self.message_set.load(
                    system
                        .telemetry_meta_data
                        .as_ref()
                        .and_then(|metadata| metadata.message_set.as_ref()),
                    window,
                    cx,
                );
            }
            ElementKind::Message(index) => {
                self.message.update(cx, |form, cx| {
                    form.load(
                        message_set(system).and_then(|set| set.message.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::TelemetryFixedFrameStream(index) => {
                self.fixed_frame_stream.update(cx, |form, cx| {
                    form.load(
                        telemetry_stream_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandFixedFrameStream(index) => {
                self.fixed_frame_stream.update(cx, |form, cx| {
                    form.load(
                        command_stream_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::TelemetryVariableFrameStream(index) => {
                self.variable_frame_stream.update(cx, |form, cx| {
                    form.load(
                        telemetry_stream_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandVariableFrameStream(index) => {
                self.variable_frame_stream.update(cx, |form, cx| {
                    form.load(
                        command_stream_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::TelemetryCustomStream(index) => {
                self.custom_stream.update(cx, |form, cx| {
                    form.load(
                        telemetry_stream_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandCustomStream(index) => {
                self.custom_stream.update(cx, |form, cx| {
                    form.load(
                        command_stream_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::TelemetryCustomAlgorithm(index) => {
                self.custom_algorithm.update(cx, |form, cx| {
                    form.load(
                        telemetry_algorithm_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandCustomAlgorithm(index) => {
                self.custom_algorithm.update(cx, |form, cx| {
                    form.load(
                        command_algorithm_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::TelemetryMathAlgorithm(index) => {
                self.math_algorithm.update(cx, |form, cx| {
                    form.load(
                        telemetry_algorithm_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::CommandMathAlgorithm(index) => {
                self.math_algorithm.update(cx, |form, cx| {
                    form.load(
                        command_algorithm_set(system).and_then(|set| set.content.get(index)),
                        window,
                        cx,
                    );
                });
            }
            ElementKind::Service(index) => {
                self.service.update(cx, |form, cx| {
                    form.load(
                        service_set(system).and_then(|set| set.service.get(index)),
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
            ElementKind::CommandContainer(index) => {
                if let Some(container) = command_container_set_mut(system)
                    .and_then(|set| set.command_container.get_mut(index))
                {
                    self.sequence_container
                        .read(cx)
                        .apply_to_sequence(container, cx);
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
            ElementKind::MessageSet => {
                if let Some(metadata) = system.telemetry_meta_data.as_mut() {
                    self.message_set.apply_to(&mut metadata.message_set, cx);
                }
            }
            ElementKind::Message(index) => {
                if let Some(message) =
                    message_set_mut(system).and_then(|set| set.message.get_mut(index))
                {
                    self.message.read(cx).apply_to(message, cx);
                }
            }
            ElementKind::TelemetryFixedFrameStream(index) => {
                if let Some(stream) =
                    telemetry_stream_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.fixed_frame_stream.read(cx).apply_to(stream, cx);
                }
            }
            ElementKind::CommandFixedFrameStream(index) => {
                if let Some(stream) =
                    command_stream_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.fixed_frame_stream.read(cx).apply_to(stream, cx);
                }
            }
            ElementKind::TelemetryVariableFrameStream(index) => {
                if let Some(stream) =
                    telemetry_stream_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.variable_frame_stream.read(cx).apply_to(stream, cx);
                }
            }
            ElementKind::CommandVariableFrameStream(index) => {
                if let Some(stream) =
                    command_stream_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.variable_frame_stream.read(cx).apply_to(stream, cx);
                }
            }
            ElementKind::TelemetryCustomStream(index) => {
                if let Some(stream) =
                    telemetry_stream_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.custom_stream.read(cx).apply_to(stream, cx);
                }
            }
            ElementKind::CommandCustomStream(index) => {
                if let Some(stream) =
                    command_stream_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.custom_stream.read(cx).apply_to(stream, cx);
                }
            }
            ElementKind::TelemetryCustomAlgorithm(index) => {
                if let Some(algorithm) =
                    telemetry_algorithm_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.custom_algorithm.read(cx).apply_to(algorithm, cx);
                }
            }
            ElementKind::CommandCustomAlgorithm(index) => {
                if let Some(algorithm) =
                    command_algorithm_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.custom_algorithm.read(cx).apply_to(algorithm, cx);
                }
            }
            ElementKind::TelemetryMathAlgorithm(index) => {
                if let Some(algorithm) =
                    telemetry_algorithm_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.math_algorithm.read(cx).apply_to(algorithm, cx);
                }
            }
            ElementKind::CommandMathAlgorithm(index) => {
                if let Some(algorithm) =
                    command_algorithm_set_mut(system).and_then(|set| set.content.get_mut(index))
                {
                    self.math_algorithm.read(cx).apply_to(algorithm, cx);
                }
            }
            ElementKind::Service(index) => {
                if let Some(service) =
                    service_set_mut(system).and_then(|set| set.service.get_mut(index))
                {
                    self.service.read(cx).apply_to(service, cx);
                }
            }
            _ => {}
        }
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
            ElementKind::SequenceContainer(_) | ElementKind::CommandContainer(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.sequence_container.clone())
            }
            ElementKind::MetaCommand(_) => gpui_component::v_flex()
                .w_full()
                .child(self.meta_command.clone()),
            ElementKind::MessageSet => self.message_set.render(cx),
            ElementKind::Message(_) => gpui_component::v_flex()
                .w_full()
                .child(self.message.clone()),
            ElementKind::TelemetryFixedFrameStream(_) | ElementKind::CommandFixedFrameStream(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.fixed_frame_stream.clone())
            }
            ElementKind::TelemetryVariableFrameStream(_)
            | ElementKind::CommandVariableFrameStream(_) => gpui_component::v_flex()
                .w_full()
                .child(self.variable_frame_stream.clone()),
            ElementKind::TelemetryCustomStream(_) | ElementKind::CommandCustomStream(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.custom_stream.clone())
            }
            ElementKind::TelemetryCustomAlgorithm(_) | ElementKind::CommandCustomAlgorithm(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.custom_algorithm.clone())
            }
            ElementKind::TelemetryMathAlgorithm(_) | ElementKind::CommandMathAlgorithm(_) => {
                gpui_component::v_flex()
                    .w_full()
                    .child(self.math_algorithm.clone())
            }
            ElementKind::TelemetryMetaData
            | ElementKind::TelemetryParameterTypeSet
            | ElementKind::ContainerSet
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
            ElementKind::Service(_) => gpui_component::v_flex()
                .w_full()
                .child(self.service.clone()),
            ElementKind::ServiceSet => gpui_component::v_flex().w_full(),
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

fn service_set(system: &xtce::SpaceSystem) -> Option<&xtce::ServiceSetType> {
    system.service_set.as_ref()
}

fn service_set_mut(system: &mut xtce::SpaceSystem) -> Option<&mut xtce::ServiceSetType> {
    system.service_set.as_mut()
}

fn message_set(system: &xtce::SpaceSystem) -> Option<&xtce::MessageSetType> {
    system
        .telemetry_meta_data
        .as_ref()
        .and_then(|metadata| metadata.message_set.as_ref())
}

fn message_set_mut(system: &mut xtce::SpaceSystem) -> Option<&mut xtce::MessageSetType> {
    system
        .telemetry_meta_data
        .as_mut()
        .and_then(|metadata| metadata.message_set.as_mut())
}

fn telemetry_stream_set(system: &xtce::SpaceSystem) -> Option<&xtce::StreamSetType> {
    system
        .telemetry_meta_data
        .as_ref()
        .and_then(|metadata| metadata.stream_set.as_ref())
}

fn telemetry_stream_set_mut(system: &mut xtce::SpaceSystem) -> Option<&mut xtce::StreamSetType> {
    system
        .telemetry_meta_data
        .as_mut()
        .and_then(|metadata| metadata.stream_set.as_mut())
}

fn command_stream_set(system: &xtce::SpaceSystem) -> Option<&xtce::StreamSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.stream_set.as_ref())
}

fn command_stream_set_mut(system: &mut xtce::SpaceSystem) -> Option<&mut xtce::StreamSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.stream_set.as_mut())
}

fn telemetry_algorithm_set(system: &xtce::SpaceSystem) -> Option<&xtce::AlgorithmSetType> {
    system
        .telemetry_meta_data
        .as_ref()
        .and_then(|metadata| metadata.algorithm_set.as_ref())
}

fn telemetry_algorithm_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::AlgorithmSetType> {
    system
        .telemetry_meta_data
        .as_mut()
        .and_then(|metadata| metadata.algorithm_set.as_mut())
}

fn command_algorithm_set(system: &xtce::SpaceSystem) -> Option<&xtce::AlgorithmSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.algorithm_set.as_ref())
}

fn command_algorithm_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::AlgorithmSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.algorithm_set.as_mut())
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

fn command_container_set(system: &xtce::SpaceSystem) -> Option<&xtce::CommandContainerSetType> {
    system
        .command_meta_data
        .as_ref()
        .and_then(|metadata| metadata.command_container_set.as_ref())
}

fn command_container_set_mut(
    system: &mut xtce::SpaceSystem,
) -> Option<&mut xtce::CommandContainerSetType> {
    system
        .command_meta_data
        .as_mut()
        .and_then(|metadata| metadata.command_container_set.as_mut())
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

fn stream_title(stream: &xtce::StreamSetTypeContent) -> String {
    match stream {
        xtce::StreamSetTypeContent::FixedFrameStream(stream) => stream.name.clone(),
        xtce::StreamSetTypeContent::VariableFrameStream(stream) => stream.name.clone(),
        xtce::StreamSetTypeContent::CustomStream(stream) => stream.name.clone(),
    }
}
