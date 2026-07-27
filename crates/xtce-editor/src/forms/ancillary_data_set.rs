use gpui::{App, AppContext, Context, Div, Entity, Window};
use gpui_component::input::InputState;

use super::{field, optional_value};
use crate::XtceEditor;

pub(super) struct AncillaryDataSetForm {
    entries_input: Entity<InputState>,
}

impl AncillaryDataSetForm {
    pub(super) fn new(
        data_set: Option<&xtce::AncillaryDataSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        Self {
            entries_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .auto_grow(4, 20)
                    .default_value(Self::encode(data_set))
            }),
        }
    }

    pub(super) fn load(
        &self,
        data_set: Option<&xtce::AncillaryDataSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        self.entries_input.update(cx, |input, cx| {
            input.set_value(Self::encode(data_set), window, cx);
        });
    }

    pub(super) fn apply_to_option(
        &self,
        data_set: &mut Option<xtce::AncillaryDataSetType>,
        cx: &App,
    ) {
        let ancillary_data = decode(&self.entries_input.read(cx).value());
        *data_set =
            (!ancillary_data.is_empty()).then_some(xtce::AncillaryDataSetType { ancillary_data });
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        field(
            "Ancillary data",
            "name | MIME type | href | content",
            &self.entries_input,
            cx,
        )
    }

    fn encode(data_set: Option<&xtce::AncillaryDataSetType>) -> String {
        data_set
            .into_iter()
            .flat_map(|set| &set.ancillary_data)
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
    use super::decode;

    #[test]
    fn decodes_ancillary_data_columns() {
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
    }
}
