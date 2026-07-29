use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt, button::Button, h_flex, input::InputState, v_flex,
};

use super::field;

pub(super) struct AliasSetForm {
    form: Entity<AliasRowsForm>,
}

struct AliasRowsForm {
    rows: Vec<Entity<AliasRow>>,
}

struct AliasRow {
    namespace: Entity<InputState>,
    alias: Entity<InputState>,
}

impl AliasSetForm {
    pub(super) fn new(
        alias_set: Option<&xtce::AliasSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Self {
        Self {
            form: rows_form(alias_set.into_iter().flat_map(|set| &set.alias), window, cx),
        }
    }

    pub(super) fn new_text(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Self {
        let aliases = decode(value);
        Self {
            form: rows_form(aliases.iter(), window, cx),
        }
    }

    pub(super) fn load(
        &self,
        alias_set: Option<&xtce::AliasSetType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) {
        self.form.update(cx, |form, cx| {
            form.rows = row_entities(alias_set.into_iter().flat_map(|set| &set.alias), window, cx);
            cx.notify();
        });
    }

    pub(super) fn apply_to_option(&self, alias_set: &mut Option<xtce::AliasSetType>, cx: &App) {
        *alias_set = self.value(cx);
    }

    pub(super) fn value(&self, cx: &App) -> Option<xtce::AliasSetType> {
        let alias = self
            .form
            .read(cx)
            .rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                let name_space = input_value(&row.namespace, cx).trim().to_owned();
                let alias = input_value(&row.alias, cx).trim().to_owned();
                if name_space.is_empty() || alias.is_empty() {
                    return None;
                }
                Some(xtce::AliasType { name_space, alias })
            })
            .collect::<Vec<_>>();
        (!alias.is_empty()).then_some(xtce::AliasSetType { alias })
    }

    pub(super) fn text(&self, cx: &App) -> String {
        encode_aliases(self.value(cx).iter().flat_map(|set| &set.alias))
    }

    pub(super) fn parse(value: &str) -> Option<xtce::AliasSetType> {
        let alias = decode(value);
        (!alias.is_empty()).then_some(xtce::AliasSetType { alias })
    }

    pub(super) fn render(&self, _: &App) -> Div {
        v_flex().w_full().child(self.form.clone())
    }

    pub(super) fn encode(alias_set: Option<&xtce::AliasSetType>) -> String {
        encode_aliases(alias_set.into_iter().flat_map(|set| &set.alias))
    }
}

impl Render for AliasRowsForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Aliases"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(self.rows.len(), "alias", "aliases")),
                            ),
                    )
                    .child(
                        Button::new("add-alias")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add alias")
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
                                    .child(format!("Alias {}", index + 1)),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-alias-{}", row.entity_id()),
                                    "Remove alias",
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
                            .child(field(
                                "Namespace",
                                "Required, for example catalog",
                                &row_read.namespace,
                                cx,
                            ))
                            .child(field(
                                "Alias",
                                "Required, for example EXAMPLE-001",
                                &row_read.alias,
                                cx,
                            )),
                    )
            }))
            .when(self.rows.is_empty(), |form| {
                form.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("No aliases."),
                )
            })
    }
}

fn rows_form<'a>(
    aliases: impl Iterator<Item = &'a xtce::AliasType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<AliasRowsForm> {
    let rows = row_entities(aliases, window, cx);
    cx.new(move |_| AliasRowsForm { rows })
}

fn row_entities<'a>(
    aliases: impl Iterator<Item = &'a xtce::AliasType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<AliasRow>> {
    aliases
        .map(|alias| row_entity(Some(alias), window, cx))
        .collect()
}

fn row_entity(
    alias: Option<&xtce::AliasType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<AliasRow> {
    let namespace = alias
        .map(|alias| alias.name_space.as_str())
        .unwrap_or_default();
    let alias_value = alias.map(|alias| alias.alias.as_str()).unwrap_or_default();
    cx.new(move |cx| AliasRow {
        namespace: input(namespace, window, cx),
        alias: input(alias_value, window, cx),
    })
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn input_value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn encode_aliases<'a>(aliases: impl Iterator<Item = &'a xtce::AliasType>) -> String {
    aliases
        .map(|alias| format!("{} = {}", alias.name_space, alias.alias))
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode(value: &str) -> Vec<xtce::AliasType> {
    value
        .lines()
        .filter_map(|line| {
            let (name_space, alias) = line.split_once('=')?;
            let name_space = name_space.trim();
            let alias = alias.trim();
            if name_space.is_empty() || alias.is_empty() {
                return None;
            }
            Some(xtce::AliasType {
                name_space: name_space.to_owned(),
                alias: alias.to_owned(),
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
