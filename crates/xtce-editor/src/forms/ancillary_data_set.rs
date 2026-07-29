use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt, button::Button, h_flex, input::InputState, v_flex,
};

use super::{field, optional_value};

pub(super) struct AncillaryDataSetForm {
    form: Entity<AncillaryDataRowsForm>,
}

struct AncillaryDataRowsForm {
    rows: Vec<Entity<AncillaryDataRow>>,
}

struct AncillaryDataRow {
    name: Entity<InputState>,
    mime_type: Entity<InputState>,
    href: Entity<InputState>,
    content: Entity<InputState>,
}

impl AncillaryDataSetForm {
    pub(super) fn new(
        data_set: Option<&xtce::AncillaryDataSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Self {
        Self {
            form: rows_form(
                data_set.into_iter().flat_map(|set| &set.ancillary_data),
                window,
                cx,
            ),
        }
    }

    pub(super) fn new_text(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Self {
        let entries = decode(value);
        Self {
            form: rows_form(entries.iter(), window, cx),
        }
    }

    pub(super) fn load(
        &self,
        data_set: Option<&xtce::AncillaryDataSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) {
        self.form.update(cx, |form, cx| {
            form.rows = row_entities(
                data_set.into_iter().flat_map(|set| &set.ancillary_data),
                window,
                cx,
            );
            cx.notify();
        });
    }

    pub(super) fn apply_to_option(
        &self,
        data_set: &mut Option<xtce::AncillaryDataSetType>,
        cx: &App,
    ) {
        *data_set = self.value(cx);
    }

    pub(super) fn value(&self, cx: &App) -> Option<xtce::AncillaryDataSetType> {
        let ancillary_data = self
            .form
            .read(cx)
            .rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let name = input_value(&row.name, cx).trim().to_owned();
                if name.is_empty() {
                    return None;
                }
                Some(xtce::AncillaryDataType {
                    name,
                    mime_type: non_empty_or(
                        input_value(&row.mime_type, cx),
                        xtce::AncillaryDataType::default_mime_type(),
                    ),
                    href: optional_value(input_value(&row.href, cx)),
                    content: input_value(&row.content, cx),
                })
            })
            .collect::<Vec<_>>();
        (!ancillary_data.is_empty()).then_some(xtce::AncillaryDataSetType { ancillary_data })
    }

    pub(super) fn text(&self, cx: &App) -> String {
        encode_entries(self.value(cx).iter().flat_map(|set| &set.ancillary_data))
    }

    pub(super) fn parse(value: &str) -> Option<xtce::AncillaryDataSetType> {
        let ancillary_data = decode(value);
        (!ancillary_data.is_empty()).then_some(xtce::AncillaryDataSetType { ancillary_data })
    }

    pub(super) fn render(&self, _: &App) -> Div {
        v_flex().w_full().child(self.form.clone())
    }

    pub(super) fn encode(data_set: Option<&xtce::AncillaryDataSetType>) -> String {
        encode_entries(data_set.into_iter().flat_map(|set| &set.ancillary_data))
    }
}

impl Render for AncillaryDataRowsForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Ancillary data"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(self.rows.len(), "entry", "entries")),
                            ),
                    )
                    .child(
                        Button::new("add-ancillary-data")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add entry")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(row_entity(None, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                let row_read = row.read(cx);
                super::detail_list_card(cx)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Entry {}", index + 1)),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-ancillary-data-{}", row.entity_id()),
                                    "Remove ancillary data",
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        if index < this.rows.len() {
                                            this.rows.remove(index);
                                            cx.notify();
                                        }
                                    },
                                )),
                            ),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .items_start()
                            .child(field("Name", "Required", &row_read.name, cx))
                            .child(field(
                                "MIME type",
                                "Defaults to text/plain",
                                &row_read.mime_type,
                                cx,
                            )),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .items_start()
                            .child(field("Href", "Optional", &row_read.href, cx))
                            .child(field("Content", "Optional", &row_read.content, cx)),
                    )
            }))
            .when(self.rows.is_empty(), |form| {
                form.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("No ancillary data."),
                )
            })
    }
}

fn rows_form<'a>(
    entries: impl Iterator<Item = &'a xtce::AncillaryDataType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<AncillaryDataRowsForm> {
    let rows = row_entities(entries, window, cx);
    cx.new(move |_| AncillaryDataRowsForm { rows })
}

fn row_entities<'a>(
    entries: impl Iterator<Item = &'a xtce::AncillaryDataType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<AncillaryDataRow>> {
    entries
        .map(|entry| row_entity(Some(entry), window, cx))
        .collect()
}

fn row_entity(
    entry: Option<&xtce::AncillaryDataType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<AncillaryDataRow> {
    let name = entry.map(|entry| entry.name.as_str()).unwrap_or_default();
    let mime_type = entry
        .map(|entry| entry.mime_type.as_str())
        .unwrap_or("text/plain");
    let href = entry
        .and_then(|entry| entry.href.as_deref())
        .unwrap_or_default();
    let content = entry
        .map(|entry| entry.content.as_str())
        .unwrap_or_default();
    cx.new(move |cx| AncillaryDataRow {
        name: input(name, window, cx),
        mime_type: input(mime_type, window, cx),
        href: input(href, window, cx),
        content: input(content, window, cx),
    })
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn input_value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn non_empty_or(value: String, fallback: String) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback
    } else {
        value.to_owned()
    }
}

fn encode_entries<'a>(entries: impl Iterator<Item = &'a xtce::AncillaryDataType>) -> String {
    entries
        .map(|data| {
            format!(
                "{} | {} | {} | {}",
                data.name,
                data.mime_type,
                data.href.as_deref().unwrap_or_default(),
                data.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode(value: &str) -> Vec<xtce::AncillaryDataType> {
    value
        .lines()
        .filter_map(|line| {
            let mut columns = line.splitn(4, " | ").map(str::trim);
            let name = columns.next()?.to_owned();
            let mime_type = columns.next()?.to_owned();
            let href = optional_value(columns.next()?.to_owned());
            let content = columns.next()?.to_owned();
            Some(xtce::AncillaryDataType {
                name,
                mime_type,
                href,
                content,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{decode, encode_entries};

    #[test]
    fn ancillary_data_text_compatibility_round_trips() {
        let entries = decode(
            "documentation | text/html | https://example.invalid/docs | Mission documentation",
        );

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].mime_type, "text/html");
        assert_eq!(
            entries[0].href.as_deref(),
            Some("https://example.invalid/docs")
        );
        assert_eq!(entries[0].content, "Mission documentation");
        assert_eq!(
            encode_entries(entries.iter()),
            "documentation | text/html | https://example.invalid/docs | Mission documentation"
        );
    }
}
