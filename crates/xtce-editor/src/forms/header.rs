use gpui::{App, AppContext, Context, Div, Entity, ParentElement, Styled, Window};
use gpui_component::{h_flex, input::InputState, v_flex};

use super::{field, optional_value};
use crate::XtceEditor;

pub(super) struct HeaderForm {
    version_input: Entity<InputState>,
    date_input: Entity<InputState>,
    classification_input: Entity<InputState>,
    classification_instructions_input: Entity<InputState>,
    validation_status_input: Entity<InputState>,
    authors_input: Entity<InputState>,
    notes_input: Entity<InputState>,
    history_input: Entity<InputState>,
}

impl HeaderForm {
    pub(super) fn new(
        header: Option<&xtce::HeaderType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        let values = HeaderValues::from_header(header);
        Self {
            version_input: input(&values.version, false, window, cx),
            date_input: input(&values.date, false, window, cx),
            classification_input: input(&values.classification, false, window, cx),
            classification_instructions_input: input(
                &values.classification_instructions,
                false,
                window,
                cx,
            ),
            validation_status_input: input(&values.validation_status, false, window, cx),
            authors_input: input(&values.authors, true, window, cx),
            notes_input: input(&values.notes, true, window, cx),
            history_input: input(&values.history, true, window, cx),
        }
    }

    pub(super) fn load(
        &self,
        header: Option<&xtce::HeaderType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        let values = HeaderValues::from_header(header);
        for (input, value) in [
            (&self.version_input, values.version),
            (&self.date_input, values.date),
            (&self.classification_input, values.classification),
            (
                &self.classification_instructions_input,
                values.classification_instructions,
            ),
            (&self.validation_status_input, values.validation_status),
            (&self.authors_input, values.authors),
            (&self.notes_input, values.notes),
            (&self.history_input, values.history),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
    }

    pub(super) fn apply_to(&self, header: &mut xtce::HeaderType, cx: &App) {
        header.version = optional_value(value(&self.version_input, cx));
        header.date = optional_value(value(&self.date_input, cx));
        header.classification = value(&self.classification_input, cx);
        header.classification_instructions =
            optional_value(value(&self.classification_instructions_input, cx));
        header.validation_status =
            validation_status_from_str(&value(&self.validation_status_input, cx));
        header.author_set =
            lines(&self.authors_input, cx).map(|author| xtce::AuthorSetType { author });
        header.note_set = lines(&self.notes_input, cx).map(|note| xtce::NoteSetType { note });
        header.history_set =
            lines(&self.history_input, cx).map(|history| xtce::HistorySetType { history });
    }

    pub(super) fn apply_to_option(&self, header: &mut Option<xtce::HeaderType>, cx: &App) {
        let header = header.get_or_insert_with(|| xtce::HeaderType {
            version: None,
            date: None,
            classification: xtce::HeaderType::default_classification(),
            classification_instructions: None,
            validation_status: xtce::ValidationStatusType::Unknown,
            author_set: None,
            note_set: None,
            history_set: None,
        });
        self.apply_to(header, cx);
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        v_flex()
            .gap_5()
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(field("Version", "Optional", &self.version_input, cx))
                    .child(field("Date", "Optional", &self.date_input, cx)),
            )
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(field(
                        "Classification",
                        "Required",
                        &self.classification_input,
                        cx,
                    ))
                    .child(field(
                        "Validation status",
                        "Unknown, Working, Draft, Test, Validated, Released, or Withdrawn",
                        &self.validation_status_input,
                        cx,
                    )),
            )
            .child(field(
                "Classification instructions",
                "Optional",
                &self.classification_instructions_input,
                cx,
            ))
            .child(field(
                "Authors",
                "One author per line",
                &self.authors_input,
                cx,
            ))
            .child(field("Notes", "One note per line", &self.notes_input, cx))
            .child(field(
                "History",
                "One history entry per line",
                &self.history_input,
                cx,
            ))
    }
}

struct HeaderValues {
    version: String,
    date: String,
    classification: String,
    classification_instructions: String,
    validation_status: String,
    authors: String,
    notes: String,
    history: String,
}

impl HeaderValues {
    fn from_header(header: Option<&xtce::HeaderType>) -> Self {
        Self {
            version: header
                .and_then(|value| value.version.clone())
                .unwrap_or_default(),
            date: header
                .and_then(|value| value.date.clone())
                .unwrap_or_default(),
            classification: header
                .map(|value| value.classification.clone())
                .unwrap_or_else(xtce::HeaderType::default_classification),
            classification_instructions: header
                .and_then(|value| value.classification_instructions.clone())
                .unwrap_or_default(),
            validation_status: header
                .map(|value| validation_status_label(&value.validation_status).to_owned())
                .unwrap_or_else(|| "Unknown".to_owned()),
            authors: header
                .and_then(|value| value.author_set.as_ref())
                .map(|set| set.author.join("\n"))
                .unwrap_or_default(),
            notes: header
                .and_then(|value| value.note_set.as_ref())
                .map(|set| set.note.join("\n"))
                .unwrap_or_default(),
            history: header
                .and_then(|value| value.history_set.as_ref())
                .map(|set| set.history.join("\n"))
                .unwrap_or_default(),
        }
    }
}

fn input(
    value: &str,
    multi_line: bool,
    window: &mut Window,
    cx: &mut Context<XtceEditor>,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .multi_line(multi_line)
            .default_value(value.to_owned())
    })
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn lines(input: &Entity<InputState>, cx: &App) -> Option<Vec<String>> {
    let entries = value(input, cx)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    (!entries.is_empty()).then_some(entries)
}

fn validation_status_label(status: &xtce::ValidationStatusType) -> &'static str {
    match status {
        xtce::ValidationStatusType::Unknown => "Unknown",
        xtce::ValidationStatusType::Working => "Working",
        xtce::ValidationStatusType::Draft => "Draft",
        xtce::ValidationStatusType::Test => "Test",
        xtce::ValidationStatusType::Validated => "Validated",
        xtce::ValidationStatusType::Released => "Released",
        xtce::ValidationStatusType::Withdrawn => "Withdrawn",
    }
}

fn validation_status_from_str(value: &str) -> xtce::ValidationStatusType {
    match value.trim().to_ascii_lowercase().as_str() {
        "working" => xtce::ValidationStatusType::Working,
        "draft" => xtce::ValidationStatusType::Draft,
        "test" => xtce::ValidationStatusType::Test,
        "validated" => xtce::ValidationStatusType::Validated,
        "released" => xtce::ValidationStatusType::Released,
        "withdrawn" => xtce::ValidationStatusType::Withdrawn,
        _ => xtce::ValidationStatusType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{validation_status_from_str, validation_status_label};

    #[test]
    fn validation_status_is_parsed_case_insensitively() {
        let status = validation_status_from_str("validated");

        assert_eq!(validation_status_label(&status), "Validated");
    }

    #[test]
    fn unknown_validation_status_falls_back_to_unknown() {
        let status = validation_status_from_str("not-a-status");

        assert_eq!(validation_status_label(&status), "Unknown");
    }
}
