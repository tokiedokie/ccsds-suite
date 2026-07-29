use std::cell::Cell;

use gpui::{App, AppContext, Div, Entity, ParentElement, Styled, Window};
use gpui_component::{input::InputState, v_flex};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm, field, optional_value,
};

pub(super) struct MessageSetForm {
    present: Cell<bool>,
    name_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
}

impl MessageSetForm {
    pub(super) fn new(
        set: Option<&xtce::MessageSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Self {
        let values = MessageSetValues::from_set(set);
        Self {
            present: Cell::new(set.is_some()),
            name_input: input(&values.name, false, window, cx),
            short_description_input: input(&values.short_description, false, window, cx),
            long_description_input: input(&values.long_description, true, window, cx),
            alias_set: AliasSetForm::new(set.and_then(|set| set.alias_set.as_ref()), window, cx),
            ancillary_data_set: AncillaryDataSetForm::new(
                set.and_then(|set| set.ancillary_data_set.as_ref()),
                window,
                cx,
            ),
        }
    }

    pub(super) fn load(
        &self,
        set: Option<&xtce::MessageSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) {
        self.present.set(set.is_some());
        let values = MessageSetValues::from_set(set);
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.alias_set
            .load(set.and_then(|set| set.alias_set.as_ref()), window, cx);
        self.ancillary_data_set.load(
            set.and_then(|set| set.ancillary_data_set.as_ref()),
            window,
            cx,
        );
    }

    pub(super) fn apply_to(&self, set: &mut Option<xtce::MessageSetType>, cx: &App) {
        if !self.present.get() && set.is_none() {
            return;
        }
        let set = set.get_or_insert_with(default_message_set);
        set.name = optional_value(value(&self.name_input, cx));
        set.short_description = optional_value(value(&self.short_description_input, cx));
        set.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut set.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut set.ancillary_data_set, cx);
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        v_flex()
            .w_full()
            .gap_5()
            .child(field("Name", "Optional", &self.name_input, cx))
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
            ))
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
    }
}

pub(crate) fn default_message_set() -> xtce::MessageSetType {
    xtce::MessageSetType {
        short_description: None,
        name: None,
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        message: Vec::new(),
    }
}

struct MessageSetValues {
    name: String,
    short_description: String,
    long_description: String,
}

impl MessageSetValues {
    fn from_set(set: Option<&xtce::MessageSetType>) -> Self {
        Self {
            name: set.and_then(|set| set.name.clone()).unwrap_or_default(),
            short_description: set
                .and_then(|set| set.short_description.clone())
                .unwrap_or_default(),
            long_description: set
                .and_then(|set| set.long_description.clone())
                .unwrap_or_default(),
        }
    }
}

fn input(
    value: &str,
    multiline: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let input = InputState::new(window, cx).default_value(value.to_owned());
        if multiline {
            input.auto_grow(super::MULTILINE_MIN_ROWS, super::MULTILINE_MAX_ROWS)
        } else {
            input
        }
    })
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}
