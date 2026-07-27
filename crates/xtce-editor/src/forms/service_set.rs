use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item, optional_value};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ReferenceKind {
    #[strum(serialize = "Container references")]
    Containers,
    #[strum(serialize = "Message references")]
    Messages,
}
impl_select_item!(ReferenceKind);

pub(super) struct ServiceSetForm {
    rows: Vec<Entity<ServiceForm>>,
}

struct ServiceForm {
    source_index: Option<usize>,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    long_description: Entity<InputState>,
    kind: Entity<SelectState<Vec<ReferenceKind>>>,
    references: Vec<Entity<InputState>>,
}

impl ServiceSetForm {
    pub(super) fn new(
        set: Option<&xtce::ServiceSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let models = service_models(set);
        cx.new(move |cx| Self {
            rows: service_entities(models, window, cx),
        })
    }

    pub(super) fn load(
        &mut self,
        set: Option<&xtce::ServiceSetType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = service_entities(service_models(set), window, cx);
        cx.notify();
    }

    pub(super) fn apply_to(&self, set: &mut Option<xtce::ServiceSetType>, cx: &App) {
        let mut existing = set
            .take()
            .map(|set| set.service.into_iter().map(Some).collect::<Vec<_>>())
            .unwrap_or_default();
        let services = self
            .rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let name = value(&row.name, cx).trim().to_owned();
                if name.is_empty() {
                    return None;
                }
                let mut service = row
                    .source_index
                    .and_then(|index| existing.get_mut(index))
                    .and_then(Option::take)
                    .unwrap_or(xtce::ServiceType {
                        short_description: None,
                        name: String::new(),
                        content: Vec::new(),
                    });
                service.name = name;
                service.short_description = optional_value(value(&row.short_description, cx));
                set_long_description(&mut service.content, value(&row.long_description, cx));
                let references = row
                    .references
                    .iter()
                    .map(|reference| value(reference, cx).trim().to_owned())
                    .filter(|reference| !reference.is_empty())
                    .collect::<Vec<_>>();
                let kind = row
                    .kind
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or(ReferenceKind::Containers);
                set_references(&mut service.content, kind, references);
                Some(service)
            })
            .collect::<Vec<_>>();
        *set = Some(xtce::ServiceSetType { service: services });
    }
}

impl Render for ServiceSetForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_lg().font_semibold().child("Services"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} services", self.rows.len())),
                            ),
                    )
                    .child(
                        Button::new("add-service")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add service")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(service_entity(
                                    ServiceModel::new(this.rows.len()),
                                    window,
                                    cx,
                                ));
                                cx.notify();
                            })),
                    ),
            )
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                v_flex()
                    .w_full()
                    .p_4()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(div().font_medium().child(format!("Service {}", index + 1)))
                            .child(
                                Button::new(format!("remove-service-{index}"))
                                    .small()
                                    .danger()
                                    .icon(IconName::Minus)
                                    .label("Remove")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index < this.rows.len() {
                                            this.rows.remove(index);
                                            cx.notify();
                                        }
                                    })),
                            ),
                    )
                    .child(row.clone())
            }))
            .when(self.rows.is_empty(), |form| {
                form.child(
                    div()
                        .p_4()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No services are defined."),
                )
            })
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
                                h_flex().gap_2().child(Input::new(reference)).child(
                                    Button::new(format!("remove-service-reference-{index}"))
                                        .small()
                                        .ghost()
                                        .icon(IconName::Minus)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            if index < this.references.len() {
                                                this.references.remove(index);
                                                cx.notify();
                                            }
                                        })),
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

struct ServiceModel {
    source_index: Option<usize>,
    name: String,
    short_description: String,
    long_description: String,
    kind: ReferenceKind,
    references: Vec<String>,
}

impl ServiceModel {
    fn new(index: usize) -> Self {
        Self {
            source_index: None,
            name: format!("Service{}", index + 1),
            short_description: String::new(),
            long_description: String::new(),
            kind: ReferenceKind::Containers,
            references: Vec::new(),
        }
    }
}

fn service_models(set: Option<&xtce::ServiceSetType>) -> Vec<ServiceModel> {
    set.into_iter()
        .flat_map(|set| &set.service)
        .enumerate()
        .map(|(index, service)| {
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
            ServiceModel {
                source_index: Some(index),
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
                kind,
                references,
            }
        })
        .collect()
}

fn service_entities(
    models: Vec<ServiceModel>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<ServiceForm>> {
    models
        .into_iter()
        .map(|model| service_entity(model, window, cx))
        .collect()
}

fn service_entity(
    model: ServiceModel,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ServiceForm> {
    cx.new(|cx| ServiceForm {
        source_index: model.source_index,
        name: input(&model.name, window, cx),
        short_description: input(&model.short_description, window, cx),
        long_description: input(&model.long_description, window, cx),
        kind: select(model.kind, window, cx),
        references: model
            .references
            .iter()
            .map(|value| input(value, window, cx))
            .collect(),
    })
}

fn set_long_description(content: &mut Vec<xtce::ServiceTypeContent>, value: String) {
    content.retain(|content| !matches!(content, xtce::ServiceTypeContent::LongDescription(_)));
    if !value.trim().is_empty() {
        content.insert(0, xtce::ServiceTypeContent::LongDescription(value));
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
    use super::{ReferenceKind, service_models, set_references};

    #[test]
    fn services_load_container_references_into_form_rows() {
        let set = xtce::ServiceSetType {
            service: vec![xtce::ServiceType {
                short_description: Some("Telemetry grouping".to_owned()),
                name: "Housekeeping".to_owned(),
                content: vec![xtce::ServiceTypeContent::ContainerRefSet(
                    xtce::ContainerRefSetType {
                        container_ref: vec![xtce::ContainerRefType {
                            container_ref: "HousekeepingPacket".to_owned(),
                        }],
                    },
                )],
            }],
        };

        let rows = service_models(Some(&set));

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Housekeeping");
        assert_eq!(rows[0].kind, ReferenceKind::Containers);
        assert_eq!(rows[0].references, ["HousekeepingPacket"]);
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
}
