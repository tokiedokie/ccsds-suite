use crate::{ElementKind, ElementSelection, XtceDocument};

/// Only an element's own name changes; all references keep their original values.
trait CollectionElement: Clone {
    fn name(&self) -> Option<&str>;
    fn rename(&mut self, name: String);
}

macro_rules! named_struct {
    ($($ty:ty),+ $(,)?) => {
        $(impl CollectionElement for $ty {
            fn name(&self) -> Option<&str> {
                Some(&self.name)
            }

            fn rename(&mut self, name: String) {
                self.name = name;
            }
        })+
    };
}

macro_rules! named_enum {
    ($ty:ty, [$($named:ident),+], [$($reference:ident),*]) => {
        impl CollectionElement for $ty {
            fn name(&self) -> Option<&str> {
                match self {
                    $(Self::$named(value) => Some(&value.name),)+
                    $(Self::$reference(_) => None,)*
                }
            }

            fn rename(&mut self, name: String) {
                match self {
                    $(Self::$named(value) => value.name = name,)+
                    $(Self::$reference(_) => {},)*
                }
            }
        }
    };
}

named_struct!(
    xtce::MessageType,
    xtce::SequenceContainerType,
    xtce::ServiceType
);
named_enum!(xtce::ParameterSetTypeContent, [Parameter], [ParameterRef]);
named_enum!(xtce::ContainerSetTypeContent, [SequenceContainer], []);
named_enum!(
    xtce::MetaCommandSetTypeContent,
    [MetaCommand, BlockMetaCommand],
    [MetaCommandRef]
);
named_enum!(
    xtce::StreamSetTypeContent,
    [FixedFrameStream, VariableFrameStream, CustomStream],
    []
);
named_enum!(
    xtce::AlgorithmSetTypeContent,
    [CustomAlgorithm, MathAlgorithm],
    []
);
named_enum!(
    xtce::ParameterTypeSetTypeContent,
    [
        StringParameterType,
        EnumeratedParameterType,
        IntegerParameterType,
        BinaryParameterType,
        FloatParameterType,
        BooleanParameterType,
        RelativeTimeParameterType,
        AbsoluteTimeParameterType,
        ArrayParameterType,
        AggregateParameterType
    ],
    []
);
named_enum!(
    xtce::ArgumentTypeSetTypeContent,
    [
        StringArgumentType,
        EnumeratedArgumentType,
        IntegerArgumentType,
        BinaryArgumentType,
        FloatArgumentType,
        BooleanArgumentType,
        RelativeTimeArgumentType,
        AbsoluteTimeArgumentType,
        ArrayArgumentType,
        AggregateArgumentType
    ],
    []
);

fn duplicate_item<T: CollectionElement>(items: &mut Vec<T>, index: usize) -> Option<usize> {
    let mut copy = items.get(index)?.clone();
    if let Some(name) = copy.name() {
        let base = format!("{name}_copy");
        let exists = |candidate: &str| items.iter().any(|item| item.name() == Some(candidate));
        let name = if !exists(&base) {
            base
        } else {
            (2..)
                .map(|suffix| format!("{base}{suffix}"))
                .find(|candidate| !exists(candidate))?
        };
        copy.rename(name);
    }
    // Reference-only entries have no name to change and retain their target.
    let index = items.len();
    items.push(copy);
    Some(index)
}

impl XtceDocument {
    pub(crate) fn duplicate_element(
        root: &mut xtce::SpaceSystem,
        selection: &ElementSelection,
    ) -> Option<ElementKind> {
        let mut system = root;
        for &index in &selection.system_path {
            system = system.space_system.get_mut(index)?;
        }
        match selection.kind {
            ElementKind::TelemetryParameterType(index) => {
                let items = &mut system
                    .telemetry_meta_data
                    .as_mut()?
                    .parameter_type_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::TelemetryParameterType)
            }
            ElementKind::CommandParameterType(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .parameter_type_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::CommandParameterType)
            }
            ElementKind::TelemetryParameter(index) => {
                let items = &mut system
                    .telemetry_meta_data
                    .as_mut()?
                    .parameter_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::TelemetryParameter)
            }
            ElementKind::CommandParameter(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .parameter_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::CommandParameter)
            }
            ElementKind::SequenceContainer(index) => {
                let items = &mut system
                    .telemetry_meta_data
                    .as_mut()?
                    .container_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::SequenceContainer)
            }
            ElementKind::Message(index) => {
                let items = &mut system
                    .telemetry_meta_data
                    .as_mut()?
                    .message_set
                    .as_mut()?
                    .message;
                duplicate_item(items, index).map(ElementKind::Message)
            }
            ElementKind::ArgumentType(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .argument_type_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::ArgumentType)
            }
            ElementKind::MetaCommand(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .meta_command_set
                    .as_mut()?
                    .content;
                duplicate_item(items, index).map(ElementKind::MetaCommand)
            }
            ElementKind::CommandContainer(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .command_container_set
                    .as_mut()?
                    .command_container;
                duplicate_item(items, index).map(ElementKind::CommandContainer)
            }
            ElementKind::Service(index) => {
                let items = &mut system.service_set.as_mut()?.service;
                duplicate_item(items, index).map(ElementKind::Service)
            }
            ElementKind::TelemetryFixedFrameStream(index)
            | ElementKind::TelemetryVariableFrameStream(index)
            | ElementKind::TelemetryCustomStream(index) => {
                let items = &mut system
                    .telemetry_meta_data
                    .as_mut()?
                    .stream_set
                    .as_mut()?
                    .content;
                let new_index = duplicate_item(items, index)?;
                Some(match &items[new_index] {
                    xtce::StreamSetTypeContent::FixedFrameStream(_) => {
                        ElementKind::TelemetryFixedFrameStream(new_index)
                    }
                    xtce::StreamSetTypeContent::VariableFrameStream(_) => {
                        ElementKind::TelemetryVariableFrameStream(new_index)
                    }
                    xtce::StreamSetTypeContent::CustomStream(_) => {
                        ElementKind::TelemetryCustomStream(new_index)
                    }
                })
            }
            ElementKind::TelemetryCustomAlgorithm(index)
            | ElementKind::TelemetryMathAlgorithm(index) => {
                let items = &mut system
                    .telemetry_meta_data
                    .as_mut()?
                    .algorithm_set
                    .as_mut()?
                    .content;
                let new_index = duplicate_item(items, index)?;
                Some(match &items[new_index] {
                    xtce::AlgorithmSetTypeContent::CustomAlgorithm(_) => {
                        ElementKind::TelemetryCustomAlgorithm(new_index)
                    }
                    xtce::AlgorithmSetTypeContent::MathAlgorithm(_) => {
                        ElementKind::TelemetryMathAlgorithm(new_index)
                    }
                })
            }
            ElementKind::CommandFixedFrameStream(index)
            | ElementKind::CommandVariableFrameStream(index)
            | ElementKind::CommandCustomStream(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .stream_set
                    .as_mut()?
                    .content;
                let new_index = duplicate_item(items, index)?;
                Some(match &items[new_index] {
                    xtce::StreamSetTypeContent::FixedFrameStream(_) => {
                        ElementKind::CommandFixedFrameStream(new_index)
                    }
                    xtce::StreamSetTypeContent::VariableFrameStream(_) => {
                        ElementKind::CommandVariableFrameStream(new_index)
                    }
                    xtce::StreamSetTypeContent::CustomStream(_) => {
                        ElementKind::CommandCustomStream(new_index)
                    }
                })
            }
            ElementKind::CommandCustomAlgorithm(index)
            | ElementKind::CommandMathAlgorithm(index) => {
                let items = &mut system
                    .command_meta_data
                    .as_mut()?
                    .algorithm_set
                    .as_mut()?
                    .content;
                let new_index = duplicate_item(items, index)?;
                Some(match &items[new_index] {
                    xtce::AlgorithmSetTypeContent::CustomAlgorithm(_) => {
                        ElementKind::CommandCustomAlgorithm(new_index)
                    }
                    xtce::AlgorithmSetTypeContent::MathAlgorithm(_) => {
                        ElementKind::CommandMathAlgorithm(new_index)
                    }
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlgorithmChildKind, StreamChildKind};

    fn selection(kind: ElementKind) -> ElementSelection {
        ElementSelection {
            system_path: Vec::new(),
            kind,
        }
    }

    fn collection_node<'a, 'input>(
        xml: &'a roxmltree::Document<'input>,
        selection: &ElementSelection,
    ) -> roxmltree::Node<'a, 'input> {
        let mut node = xml.root_element();
        for &index in &selection.system_path {
            node = node
                .children()
                .filter(|child| child.has_tag_name("SpaceSystem"))
                .nth(index)
                .unwrap();
        }
        for directory in selection.kind.breadcrumb_directories() {
            node = node
                .children()
                .find(|child| child.has_tag_name(*directory))
                .unwrap();
        }
        node
    }

    #[test]
    fn duplicate_all_collection_kinds_preserves_contents_and_xml_round_trip() {
        let mut document = XtceDocument::from_xml(
            include_str!("../../xtce/tests/fixtures/sample.xml"),
            "sample.xml".into(),
        )
        .unwrap();
        XtceDocument::add_metadata(&mut document.root, ElementKind::ServiceSet);
        for set in [
            ElementKind::TelemetryParameterTypeSet,
            ElementKind::CommandParameterTypeSet,
            ElementKind::TelemetryParameterSet,
            ElementKind::CommandParameterSet,
            ElementKind::ContainerSet,
            ElementKind::CommandContainerSet,
            ElementKind::ArgumentTypeSet,
            ElementKind::MetaCommandSet,
            ElementKind::MessageSet,
            ElementKind::ServiceSet,
        ] {
            XtceDocument::add_collection_item(&mut document.root, set).unwrap();
        }
        for set in [
            ElementKind::TelemetryStreamSet,
            ElementKind::CommandStreamSet,
        ] {
            for kind in [
                StreamChildKind::Fixed,
                StreamChildKind::Variable,
                StreamChildKind::Custom,
            ] {
                XtceDocument::add_stream_item(&mut document.root, set, kind).unwrap();
            }
        }
        for set in [
            ElementKind::TelemetryAlgorithmSet,
            ElementKind::CommandAlgorithmSet,
        ] {
            for kind in [AlgorithmChildKind::Custom, AlgorithmChildKind::Math] {
                XtceDocument::add_algorithm_item(&mut document.root, set, kind).unwrap();
            }
        }
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document.root, &mut Vec::new(), 0, &mut nodes);
        for node in nodes
            .into_iter()
            .filter(|node| node.selection.kind.can_delete())
        {
            let before = XtceDocument::serialize(&document.root).unwrap();
            let before_xml = roxmltree::Document::parse(&before).unwrap();
            let before_set = collection_node(&before_xml, &node.selection);
            let original = before_set
                .children()
                .find(|child| child.attribute("name") == Some(node.label.as_str()))
                .unwrap();
            let original_xml = &before[original.range()];

            let kind =
                XtceDocument::duplicate_element(&mut document.root, &node.selection).unwrap();
            assert_eq!(
                std::mem::discriminant(&kind),
                std::mem::discriminant(&node.selection.kind)
            );
            let copy_selection = ElementSelection {
                system_path: node.selection.system_path.clone(),
                kind,
            };
            assert_eq!(
                document.element_name(&copy_selection),
                Some(format!("{}_copy", node.label))
            );

            let after = XtceDocument::serialize(&document.root).unwrap();
            let after_xml = roxmltree::Document::parse(&after).unwrap();
            let after_set = collection_node(&after_xml, &copy_selection);
            let old_children: Vec<_> = before_set
                .children()
                .filter(|child| child.is_element())
                .map(|child| &before[child.range()])
                .collect();
            let new_children: Vec<_> = after_set
                .children()
                .filter(|child| child.is_element())
                .map(|child| &after[child.range()])
                .collect();
            assert_eq!(&new_children[..old_children.len()], old_children.as_slice());
            assert_eq!(new_children.len(), old_children.len() + 1);
            assert_eq!(
                *new_children.last().unwrap(),
                original_xml.replacen(
                    &format!("name=\"{}\"", node.label),
                    &format!("name=\"{}_copy\"", node.label),
                    1
                )
            );
            let reopened = XtceDocument::from_xml(&after, "copy.xml".into()).unwrap();
            assert_eq!(XtceDocument::serialize(&reopened.root).unwrap(), after);
        }
    }

    #[test]
    fn duplicate_container_in_nested_system_is_independent_and_keeps_references() {
        let xml = format!(
            r#"
            <SpaceSystem xmlns="{}" name="Root">
              <SpaceSystem name="Payload">
                <TelemetryMetaData>
                  <ContainerSet>
                    <SequenceContainer name="Packet" shortDescription="Science packet" idlePattern="0">
                      <LongDescription>Packet description</LongDescription>
                      <AliasSet><Alias nameSpace="ops" alias="SCI" /></AliasSet>
                      <AncillaryDataSet><AncillaryData name="owner">Payload</AncillaryData></AncillaryDataSet>
                      <EntryList>
                        <ParameterRefEntry parameterRef="SampleCount" shortDescription="Count" />
                        <ContainerRefEntry containerRef="../Header" />
                      </EntryList>
                      <BaseContainer containerRef="../BasePacket" />
                    </SequenceContainer>
                  </ContainerSet>
                </TelemetryMetaData>
              </SpaceSystem>
              <SpaceSystem name="Sibling" />
            </SpaceSystem>
        "#,
            xtce::XTCE_NAMESPACE
        );
        let mut root = xtce::from_str(&xml).unwrap();
        let original_system = format!("{:?}", root.space_system[0]);
        let sibling = format!("{:?}", root.space_system[1]);
        let selected = ElementSelection {
            system_path: vec![0],
            kind: ElementKind::SequenceContainer(0),
        };
        assert_eq!(
            XtceDocument::duplicate_element(&mut root, &selected),
            Some(ElementKind::SequenceContainer(1))
        );
        let items = &mut root.space_system[0]
            .telemetry_meta_data
            .as_mut()
            .unwrap()
            .container_set
            .as_mut()
            .unwrap()
            .content;
        let xtce::ContainerSetTypeContent::SequenceContainer(mut expected) = items[0].clone();
        expected.name = "Packet_copy".into();
        let xtce::ContainerSetTypeContent::SequenceContainer(copy) = &mut items[1];
        assert_eq!(format!("{copy:?}"), format!("{expected:?}"));
        copy.alias_set.as_mut().unwrap().alias[0].alias = "NEW".into();
        let xtce::EntryListTypeContent::ParameterRefEntry(entry) = &mut copy.entry_list.content[0]
        else {
            panic!("expected ParameterRefEntry");
        };
        entry.parameter_ref = "OtherParameter".into();
        copy.base_container.as_mut().unwrap().container_ref = "OtherBase".into();
        let saved = XtceDocument::serialize(&root).unwrap();
        let reopened = xtce::from_str(&saved).unwrap();
        assert_eq!(XtceDocument::serialize(&reopened).unwrap(), saved);
        root.space_system[0]
            .telemetry_meta_data
            .as_mut()
            .unwrap()
            .container_set
            .as_mut()
            .unwrap()
            .content
            .pop();
        assert_eq!(format!("{:?}", root.space_system[0]), original_system);
        assert_eq!(format!("{:?}", root.space_system[1]), sibling);
        assert!(root.telemetry_meta_data.is_none());
    }

    #[test]
    fn duplicate_names_avoid_collisions_across_variants_in_the_same_set() {
        let mut root =
            xtce::from_str(include_str!("../../xtce/tests/fixtures/sample.xml")).unwrap();
        let set = root
            .telemetry_meta_data
            .as_mut()
            .unwrap()
            .parameter_type_set
            .as_mut()
            .unwrap();
        set.content[0].rename("Value".into());
        set.content[1].rename("Value_copy".into());
        let selected = selection(ElementKind::TelemetryParameterType(0));
        for index in 2..=3 {
            assert_eq!(
                XtceDocument::duplicate_element(&mut root, &selected),
                Some(ElementKind::TelemetryParameterType(index))
            );
        }
        let set = root
            .telemetry_meta_data
            .as_ref()
            .unwrap()
            .parameter_type_set
            .as_ref()
            .unwrap();
        assert_eq!(
            set.content
                .iter()
                .map(CollectionElement::name)
                .collect::<Vec<_>>(),
            [
                Some("Value"),
                Some("Value_copy"),
                Some("Value_copy2"),
                Some("Value_copy3")
            ]
        );
        assert!(matches!(
            set.content[2],
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(_)
        ));
    }

    #[test]
    fn duplicate_reference_only_entries_keeps_the_target() {
        let mut root = xtce::from_str(&format!(r#"
            <SpaceSystem xmlns="{}" name="Root">
              <TelemetryMetaData><ParameterSet><ParameterRef parameterRef="../Value" /></ParameterSet></TelemetryMetaData>
              <CommandMetaData><MetaCommandSet><MetaCommandRef>../Command</MetaCommandRef></MetaCommandSet></CommandMetaData>
            </SpaceSystem>
        "#, xtce::XTCE_NAMESPACE)).unwrap();
        assert_eq!(
            XtceDocument::duplicate_element(
                &mut root,
                &selection(ElementKind::TelemetryParameter(0))
            ),
            Some(ElementKind::TelemetryParameter(1))
        );
        assert_eq!(
            XtceDocument::duplicate_element(&mut root, &selection(ElementKind::MetaCommand(0))),
            Some(ElementKind::MetaCommand(1))
        );
        let saved = XtceDocument::serialize(&root).unwrap();
        assert_eq!(saved.matches("parameterRef=\"../Value\"").count(), 2);
        assert_eq!(saved.matches(">../Command</").count(), 2);
        xtce::from_str(&saved).unwrap();
    }

    #[test]
    fn duplicate_invalid_selection_leaves_document_unchanged() {
        let mut root =
            xtce::from_str(include_str!("../../xtce/tests/fixtures/sample.xml")).unwrap();
        let before = XtceDocument::serialize(&root).unwrap();
        for selected in [
            selection(ElementKind::SpaceSystem),
            selection(ElementKind::TelemetryParameterSet),
            selection(ElementKind::TelemetryParameter(999)),
            selection(ElementKind::SequenceContainer(0)),
            selection(ElementKind::CommandParameter(0)),
            selection(ElementKind::Service(0)),
            ElementSelection {
                system_path: vec![999],
                kind: ElementKind::TelemetryParameter(0),
            },
        ] {
            assert_eq!(XtceDocument::duplicate_element(&mut root, &selected), None);
            assert_eq!(XtceDocument::serialize(&root).unwrap(), before);
        }
    }
}
