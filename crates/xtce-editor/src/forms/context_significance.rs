use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    select::SelectState,
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item, message::MessageCriteriaForm, optional_value};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ConsequenceLevel {
    #[strum(serialize = "normal")]
    Normal,
    #[strum(serialize = "vital")]
    Vital,
    #[strum(serialize = "critical")]
    Critical,
    #[strum(serialize = "forbidden")]
    Forbidden,
    #[strum(serialize = "user1")]
    User1,
    #[strum(serialize = "user2")]
    User2,
}
impl_select_item!(ConsequenceLevel);

pub(super) struct ContextSignificanceListForm {
    rows: Vec<Entity<ContextSignificanceRow>>,
}

struct ContextSignificanceRow {
    criteria: Entity<MessageCriteriaForm>,
    consequence_level: Entity<SelectState<Vec<ConsequenceLevel>>>,
    reason_for_warning: Entity<InputState>,
    space_system_at_risk: Entity<InputState>,
}

impl ContextSignificanceListForm {
    pub(super) fn new(
        list: Option<&xtce::ContextSignificanceListType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = list
            .into_iter()
            .flat_map(|list| &list.context_significance)
            .map(|value| new_row(Some(value), window, cx))
            .collect();
        cx.new(|_| Self { rows })
    }

    pub(super) fn load(
        &mut self,
        list: Option<&xtce::ContextSignificanceListType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = list
            .into_iter()
            .flat_map(|list| &list.context_significance)
            .map(|value| new_row(Some(value), window, cx))
            .collect();
        cx.notify();
    }

    pub(super) fn apply_to(&self, list: &mut Option<xtce::ContextSignificanceListType>, cx: &App) {
        let rows = self
            .rows
            .iter()
            .map(|row| row.read(cx).value(cx))
            .collect::<Vec<_>>();
        *list = (!rows.is_empty()).then_some(xtce::ContextSignificanceListType {
            context_significance: rows,
        });
    }
}

impl Render for ContextSignificanceListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                v_flex()
                    .w_full()
                    .p_3()
                    .gap_4()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Context significance {}", index + 1)),
                            )
                            .child(
                                Button::new(format!("remove-context-significance-{index}"))
                                    .ghost()
                                    .small()
                                    .icon(IconName::Minus)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index < this.rows.len() {
                                            this.rows.remove(index);
                                            cx.notify();
                                        }
                                    })),
                            ),
                    )
                    .child(row.clone())
            })
            .collect::<Vec<_>>();

        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_sm().font_medium().child("Context significance"))
                    .child(
                        Button::new("add-context-significance")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add context significance")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(new_row(None, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No context-specific significance is defined."),
                )
            })
            .children(rows)
    }
}

impl ContextSignificanceRow {
    fn value(&self, cx: &App) -> xtce::ContextSignificanceType {
        xtce::ContextSignificanceType {
            context_match: self.criteria.read(cx).context_match(cx),
            significance: xtce::SignificanceType {
                space_system_at_risk: optional_value(value(&self.space_system_at_risk, cx)),
                reason_for_warning: optional_value(value(&self.reason_for_warning, cx)),
                consequence_level: level_to_xtce(selected_value(
                    &self.consequence_level,
                    ConsequenceLevel::Normal,
                    cx,
                )),
            },
        }
    }
}

impl Render for ContextSignificanceRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .child(self.criteria.clone())
            .child(
                div()
                    .text_sm()
                    .font_medium()
                    .child("Significance when matched"),
            )
            .child(select_field("Consequence level", &self.consequence_level))
            .child(field(
                "Reason for warning",
                "Optional",
                &self.reason_for_warning,
                cx,
            ))
            .child(field(
                "Space system at risk",
                "Optional",
                &self.space_system_at_risk,
                cx,
            ))
    }
}

fn new_row(
    value: Option<&xtce::ContextSignificanceType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ContextSignificanceRow> {
    let criteria =
        MessageCriteriaForm::new_context(value.map(|value| &value.context_match), window, cx);
    let level = value
        .map(|value| level_from_xtce(&value.significance.consequence_level))
        .unwrap_or(ConsequenceLevel::Normal);
    let reason = value
        .and_then(|value| value.significance.reason_for_warning.as_deref())
        .unwrap_or_default();
    let risk = value
        .and_then(|value| value.significance.space_system_at_risk.as_deref())
        .unwrap_or_default();
    let consequence_level = select(ConsequenceLevel::VARIANTS, level, window, cx);
    let reason_for_warning = input(reason, window, cx);
    let space_system_at_risk = input(risk, window, cx);
    cx.new(|_| ContextSignificanceRow {
        criteria,
        consequence_level,
        reason_for_warning,
        space_system_at_risk,
    })
}

fn level_from_xtce(level: &xtce::ConsequenceLevelType) -> ConsequenceLevel {
    match level {
        xtce::ConsequenceLevelType::Normal => ConsequenceLevel::Normal,
        xtce::ConsequenceLevelType::Vital => ConsequenceLevel::Vital,
        xtce::ConsequenceLevelType::Critical => ConsequenceLevel::Critical,
        xtce::ConsequenceLevelType::Forbidden => ConsequenceLevel::Forbidden,
        xtce::ConsequenceLevelType::User1 => ConsequenceLevel::User1,
        xtce::ConsequenceLevelType::User2 => ConsequenceLevel::User2,
    }
}

fn level_to_xtce(level: ConsequenceLevel) -> xtce::ConsequenceLevelType {
    match level {
        ConsequenceLevel::Normal => xtce::ConsequenceLevelType::Normal,
        ConsequenceLevel::Vital => xtce::ConsequenceLevelType::Vital,
        ConsequenceLevel::Critical => xtce::ConsequenceLevelType::Critical,
        ConsequenceLevel::Forbidden => xtce::ConsequenceLevelType::Forbidden,
        ConsequenceLevel::User1 => xtce::ConsequenceLevelType::User1,
        ConsequenceLevel::User2 => xtce::ConsequenceLevelType::User2,
    }
}

fn select(
    choices: &[ConsequenceLevel],
    selected: ConsequenceLevel,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<ConsequenceLevel>>> {
    let selected_index = choices
        .iter()
        .position(|choice| *choice == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            choices.to_vec(),
            Some(IndexPath::default().row(selected_index)),
            window,
            cx,
        )
    })
}

fn selected_value(
    select: &Entity<SelectState<Vec<ConsequenceLevel>>>,
    fallback: ConsequenceLevel,
    cx: &App,
) -> ConsequenceLevel {
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(fallback)
}

fn select_field(
    label: &'static str,
    select: &Entity<SelectState<Vec<ConsequenceLevel>>>,
) -> gpui::Div {
    super::select_field(label, "Required", select)
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{ConsequenceLevel, level_from_xtce, level_to_xtce};

    #[test]
    fn consequence_levels_round_trip() {
        for level in [
            ConsequenceLevel::Normal,
            ConsequenceLevel::Vital,
            ConsequenceLevel::Critical,
            ConsequenceLevel::Forbidden,
            ConsequenceLevel::User1,
            ConsequenceLevel::User2,
        ] {
            assert_eq!(level_from_xtce(&level_to_xtce(level)), level);
        }
    }
}
