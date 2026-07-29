use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt,
    button::Button,
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm, field, impl_select_item,
    optional_value,
};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ReferenceKind {
    #[strum(serialize = "Container references")]
    Containers,
    #[strum(serialize = "Message references")]
    Messages,
}
impl_select_item!(ReferenceKind);

pub(super) struct ServiceForm {
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    long_description: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    kind: Entity<SelectState<Vec<ReferenceKind>>>,
    references: Vec<Entity<InputState>>,
}

impl ServiceForm {
    pub(super) fn new(
        service: Option<&xtce::ServiceType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let model = ServiceModel::from_service(service);
        cx.new(move |cx| Self {
            name: input(&model.name, window, cx),
            short_description: input(&model.short_description, window, cx),
            long_description: input(&model.long_description, window, cx),
            alias_set: AliasSetForm::new_text(&model.aliases, window, cx),
            ancillary_data_set: AncillaryDataSetForm::new(
                service.and_then(service_ancillary_data_set),
                window,
                cx,
            ),
            kind: select(model.kind, window, cx),
            references: model
                .references
                .iter()
                .map(|reference| input(reference, window, cx))
                .collect(),
        })
    }

    pub(super) fn load(
        &mut self,
        service: Option<&xtce::ServiceType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let model = ServiceModel::from_service(service);
        for (input, value) in [
            (&self.name, model.name),
            (&self.short_description, model.short_description),
            (&self.long_description, model.long_description),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.alias_set
            .load(service.and_then(service_alias_set), window, cx);
        self.ancillary_data_set
            .load(service.and_then(service_ancillary_data_set), window, cx);
        self.kind.update(cx, |select, cx| {
            select.set_selected_value(&model.kind, window, cx);
        });
        self.references = model
            .references
            .iter()
            .map(|reference| input(reference, window, cx))
            .collect();
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name, cx)
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(px(520.))
            .child(Input::new(&self.name))
    }

    pub(super) fn apply_to(&self, service: &mut xtce::ServiceType, cx: &App) {
        let name = value(&self.name, cx).trim().to_owned();
        if !name.is_empty() {
            service.name = name;
        }
        service.short_description = optional_value(value(&self.short_description, cx));
        set_long_description(&mut service.content, value(&self.long_description, cx));
        set_alias_set(&mut service.content, self.alias_set.value(cx));
        set_ancillary_data_set(&mut service.content, self.ancillary_data_set.value(cx));
        let references = self
            .references
            .iter()
            .map(|reference| value(reference, cx).trim().to_owned())
            .filter(|reference| !reference.is_empty())
            .collect::<Vec<_>>();
        let kind = self
            .kind
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(ReferenceKind::Containers);
        set_references(&mut service.content, kind, references);
    }
}

impl Render for ServiceForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let reference_count = self.references.len();
        v_flex()
            .w_full()
            .gap_4()
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(field("Name", "Required", &self.name, cx))
                    .child(field(
                        "Short description",
                        "Optional",
                        &self.short_description,
                        cx,
                    )),
            )
            .child(field(
                "Long description",
                "Optional",
                &self.long_description,
                cx,
            ))
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
            .child(
                v_flex()
                    .gap_2()
                    .child(div().text_sm().font_medium().child("Reference type"))
                    .child(Select::new(&self.kind).w_full()),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(div().text_sm().font_medium().child("References"))
                            .child(
                                Button::new("add-service-reference")
                                    .small()
                                    .icon(IconName::Plus)
                                    .label("Add reference")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.references.push(input("", window, cx));
                                        cx.notify();
                                    })),
                            ),
                    )
                    .children(
                        self.references
                            .iter()
                            .enumerate()
                            .map(|(index, reference)| {
                                super::compact_list_row(cx)
                                    .child(Input::new(reference))
                                    .child(
                                        super::row_remove_button(
                                            format!("remove-service-reference-{index}"),
                                            "Remove reference",
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                if index < this.references.len() {
                                                    this.references.remove(index);
                                                    cx.notify();
                                                }
                                            }),
                                        ),
                                    )
                            }),
                    )
                    .when(reference_count == 0, |list| {
                        list.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("No references."),
                        )
                    }),
            )
    }
}

pub(crate) fn default_service(name: String) -> xtce::ServiceType {
    xtce::ServiceType {
        short_description: None,
        name,
        content: vec![xtce::ServiceTypeContent::ContainerRefSet(
            xtce::ContainerRefSetType {
                container_ref: Vec::new(),
            },
        )],
    }
}

struct ServiceModel {
    name: String,
    short_description: String,
    long_description: String,
    aliases: String,
    kind: ReferenceKind,
    references: Vec<String>,
}

impl ServiceModel {
    fn from_service(service: Option<&xtce::ServiceType>) -> Self {
        let Some(service) = service else {
            return Self {
                name: "Service1".to_owned(),
                short_description: String::new(),
                long_description: String::new(),
                aliases: String::new(),
                kind: ReferenceKind::Containers,
                references: Vec::new(),
            };
        };
        let (kind, references) = service
            .content
            .iter()
            .find_map(|content| match content {
                xtce::ServiceTypeContent::ContainerRefSet(set) => Some((
                    ReferenceKind::Containers,
                    set.container_ref
                        .iter()
                        .map(|value| value.container_ref.clone())
                        .collect(),
                )),
                xtce::ServiceTypeContent::MessageRefSet(set) => Some((
                    ReferenceKind::Messages,
                    set.message_ref
                        .iter()
                        .map(|value| value.message_ref.clone())
                        .collect(),
                )),
                _ => None,
            })
            .unwrap_or((ReferenceKind::Containers, Vec::new()));
        Self {
            name: service.name.clone(),
            short_description: service.short_description.clone().unwrap_or_default(),
            long_description: service
                .content
                .iter()
                .find_map(|content| match content {
                    xtce::ServiceTypeContent::LongDescription(value) => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_default(),
            aliases: AliasSetForm::encode(service_alias_set(service)),
            kind,
            references,
        }
    }
}

fn service_alias_set(service: &xtce::ServiceType) -> Option<&xtce::AliasSetType> {
    service.content.iter().find_map(|content| match content {
        xtce::ServiceTypeContent::AliasSet(value) => Some(value),
        _ => None,
    })
}

fn service_ancillary_data_set(service: &xtce::ServiceType) -> Option<&xtce::AncillaryDataSetType> {
    service.content.iter().find_map(|content| match content {
        xtce::ServiceTypeContent::AncillaryDataSet(value) => Some(value),
        _ => None,
    })
}

fn set_long_description(content: &mut Vec<xtce::ServiceTypeContent>, value: String) {
    content.retain(|content| !matches!(content, xtce::ServiceTypeContent::LongDescription(_)));
    if !value.trim().is_empty() {
        content.insert(0, xtce::ServiceTypeContent::LongDescription(value));
    }
}

fn set_alias_set(
    content: &mut Vec<xtce::ServiceTypeContent>,
    alias_set: Option<xtce::AliasSetType>,
) {
    content.retain(|content| !matches!(content, xtce::ServiceTypeContent::AliasSet(_)));
    if let Some(alias_set) = alias_set {
        let index = content
            .iter()
            .position(|content| {
                matches!(
                    content,
                    xtce::ServiceTypeContent::AncillaryDataSet(_)
                        | xtce::ServiceTypeContent::MessageRefSet(_)
                        | xtce::ServiceTypeContent::ContainerRefSet(_)
                )
            })
            .unwrap_or(content.len());
        content.insert(index, xtce::ServiceTypeContent::AliasSet(alias_set));
    }
}

fn set_ancillary_data_set(
    content: &mut Vec<xtce::ServiceTypeContent>,
    ancillary_data_set: Option<xtce::AncillaryDataSetType>,
) {
    content.retain(|content| !matches!(content, xtce::ServiceTypeContent::AncillaryDataSet(_)));
    if let Some(ancillary_data_set) = ancillary_data_set {
        let index = content
            .iter()
            .position(|content| {
                matches!(
                    content,
                    xtce::ServiceTypeContent::MessageRefSet(_)
                        | xtce::ServiceTypeContent::ContainerRefSet(_)
                )
            })
            .unwrap_or(content.len());
        content.insert(
            index,
            xtce::ServiceTypeContent::AncillaryDataSet(ancillary_data_set),
        );
    }
}

fn set_references(
    content: &mut Vec<xtce::ServiceTypeContent>,
    kind: ReferenceKind,
    references: Vec<String>,
) {
    content.retain(|content| {
        !matches!(
            content,
            xtce::ServiceTypeContent::MessageRefSet(_)
                | xtce::ServiceTypeContent::ContainerRefSet(_)
        )
    });
    content.push(match kind {
        ReferenceKind::Containers => {
            xtce::ServiceTypeContent::ContainerRefSet(xtce::ContainerRefSetType {
                container_ref: references
                    .into_iter()
                    .map(|container_ref| xtce::ContainerRefType { container_ref })
                    .collect(),
            })
        }
        ReferenceKind::Messages => {
            xtce::ServiceTypeContent::MessageRefSet(xtce::MessageRefSetType {
                message_ref: references
                    .into_iter()
                    .map(|message_ref| xtce::MessageRefType { message_ref })
                    .collect(),
            })
        }
    });
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn select(
    kind: ReferenceKind,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<ReferenceKind>>> {
    let index = ReferenceKind::VARIANTS
        .iter()
        .position(|value| *value == kind)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            ReferenceKind::VARIANTS.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{ReferenceKind, ServiceModel, set_ancillary_data_set, set_references};

    #[test]
    fn service_loads_container_references() {
        let service = xtce::ServiceType {
            short_description: Some("Telemetry grouping".to_owned()),
            name: "Housekeeping".to_owned(),
            content: vec![xtce::ServiceTypeContent::ContainerRefSet(
                xtce::ContainerRefSetType {
                    container_ref: vec![xtce::ContainerRefType {
                        container_ref: "HousekeepingPacket".to_owned(),
                    }],
                },
            )],
        };

        let model = ServiceModel::from_service(Some(&service));

        assert_eq!(model.name, "Housekeeping");
        assert_eq!(model.kind, ReferenceKind::Containers);
        assert_eq!(model.references, ["HousekeepingPacket"]);
    }

    #[test]
    fn changing_reference_type_preserves_unrelated_service_content() {
        let mut content = vec![
            xtce::ServiceTypeContent::LongDescription("Keep this".to_owned()),
            xtce::ServiceTypeContent::ContainerRefSet(xtce::ContainerRefSetType {
                container_ref: vec![xtce::ContainerRefType {
                    container_ref: "OldPacket".to_owned(),
                }],
            }),
        ];

        set_references(
            &mut content,
            ReferenceKind::Messages,
            vec!["ModeChanged".to_owned()],
        );

        assert!(matches!(
            content.first(),
            Some(xtce::ServiceTypeContent::LongDescription(value)) if value == "Keep this"
        ));
        assert!(matches!(
            content.last(),
            Some(xtce::ServiceTypeContent::MessageRefSet(set))
                if set.message_ref[0].message_ref == "ModeChanged"
        ));
    }

    #[test]
    fn ancillary_data_is_inserted_before_service_references() {
        let mut content = vec![xtce::ServiceTypeContent::ContainerRefSet(
            xtce::ContainerRefSetType {
                container_ref: Vec::new(),
            },
        )];

        set_ancillary_data_set(
            &mut content,
            Some(xtce::AncillaryDataSetType {
                ancillary_data: vec![xtce::AncillaryDataType {
                    name: "owner".to_owned(),
                    mime_type: "text/plain".to_owned(),
                    href: None,
                    content: "flight".to_owned(),
                }],
            }),
        );

        assert!(matches!(
            content.as_slice(),
            [
                xtce::ServiceTypeContent::AncillaryDataSet(_),
                xtce::ServiceTypeContent::ContainerRefSet(_)
            ]
        ));
    }
}
