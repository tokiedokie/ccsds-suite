use gpui::{App, AppContext, Context, Div, Entity, Window};
use gpui_component::input::InputState;

use super::field;
use crate::XtceEditor;

pub(super) struct AliasSetForm {
    aliases_input: Entity<InputState>,
}

impl AliasSetForm {
    pub(super) fn new(
        alias_set: Option<&xtce::AliasSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        Self {
            aliases_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .auto_grow(4, 20)
                    .default_value(Self::encode(alias_set))
            }),
        }
    }

    pub(super) fn load(
        &self,
        alias_set: Option<&xtce::AliasSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        self.aliases_input.update(cx, |input, cx| {
            input.set_value(Self::encode(alias_set), window, cx);
        });
    }

    pub(super) fn apply_to_option(&self, alias_set: &mut Option<xtce::AliasSetType>, cx: &App) {
        let alias = decode(&self.aliases_input.read(cx).value());
        *alias_set = (!alias.is_empty()).then_some(xtce::AliasSetType { alias });
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        field(
            "Aliases",
            "One “namespace = alias” entry per line",
            &self.aliases_input,
            cx,
        )
    }

    fn encode(alias_set: Option<&xtce::AliasSetType>) -> String {
        alias_set
            .into_iter()
            .flat_map(|set| &set.alias)
            .map(|alias| format!("{} = {}", alias.name_space, alias.alias))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn decode(value: &str) -> Vec<xtce::AliasType> {
    value
        .lines()
        .filter_map(|line| {
            let (name_space, alias) = line.split_once('=')?;
            Some(xtce::AliasType {
                name_space: name_space.trim().to_owned(),
                alias: alias.trim().to_owned(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decodes_one_alias_per_line() {
        let aliases = decode("catalog = EXAMPLE-001\noperations = EXM");

        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name_space, "catalog");
        assert_eq!(aliases[1].alias, "EXM");
    }
}
