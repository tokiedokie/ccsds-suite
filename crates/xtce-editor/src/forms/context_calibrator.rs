use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{IconName, Sizable, StyledExt, button::Button, h_flex, v_flex};

use super::{default_calibrator::DefaultCalibratorForm, message::MessageCriteriaForm};

pub(super) struct ContextCalibratorListForm {
    rows: Vec<Entity<ContextCalibratorRow>>,
}

struct ContextCalibratorRow {
    criteria: Entity<MessageCriteriaForm>,
    calibrator: Entity<DefaultCalibratorForm>,
}

impl ContextCalibratorListForm {
    pub(super) fn new(
        list: Option<&xtce::ContextCalibratorListType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = list
            .into_iter()
            .flat_map(|list| &list.context_calibrator)
            .map(|value| new_row(Some(value), window, cx))
            .collect();
        cx.new(|_| Self { rows })
    }

    pub(super) fn load(
        &mut self,
        list: Option<&xtce::ContextCalibratorListType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = list
            .into_iter()
            .flat_map(|list| &list.context_calibrator)
            .map(|value| new_row(Some(value), window, cx))
            .collect();
        cx.notify();
    }

    pub(super) fn apply_to(&self, list: &mut Option<xtce::ContextCalibratorListType>, cx: &App) {
        let rows = self
            .rows
            .iter()
            .map(|row| row.read(cx).value(cx))
            .collect::<Vec<_>>();
        *list = (!rows.is_empty()).then_some(xtce::ContextCalibratorListType {
            context_calibrator: rows,
        });
    }
}

impl Render for ContextCalibratorListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                super::detail_list_card(cx)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Context calibrator {}", index + 1)),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-context-calibrator-{index}"),
                                    "Remove context calibrator",
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
                    .child(row.clone())
            })
            .collect::<Vec<_>>();

        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_sm().font_medium().child("Context calibrators"))
                    .child(
                        Button::new("add-context-calibrator")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add context calibrator")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows.push(new_row(None, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .children(rows)
    }
}

impl ContextCalibratorRow {
    fn value(&self, cx: &App) -> xtce::ContextCalibratorType {
        xtce::ContextCalibratorType {
            context_match: self.criteria.read(cx).context_match(cx),
            calibrator: self.calibrator.read(cx).calibrator(cx),
        }
    }
}

impl Render for ContextCalibratorRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .child(self.criteria.clone())
            .child(div().text_sm().font_medium().child("Calibrator"))
            .child(self.calibrator.clone())
    }
}

fn new_row(
    value: Option<&xtce::ContextCalibratorType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ContextCalibratorRow> {
    let criteria =
        MessageCriteriaForm::new_context(value.map(|value| &value.context_match), window, cx);
    let calibrator =
        DefaultCalibratorForm::new_required(value.map(|value| &value.calibrator), window, cx);
    cx.new(|_| ContextCalibratorRow {
        criteria,
        calibrator,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn context_calibrator_list_keeps_multiple_rows() {
        let list = xtce::ContextCalibratorListType {
            context_calibrator: vec![
                context_calibrator("mode", "1"),
                context_calibrator("mode", "2"),
            ],
        };

        assert_eq!(list.context_calibrator.len(), 2);
    }

    fn context_calibrator(parameter: &str, value: &str) -> xtce::ContextCalibratorType {
        xtce::ContextCalibratorType {
            context_match: xtce::ContextMatchType::Comparison(xtce::ComparisonType {
                parameter_ref: parameter.to_owned(),
                instance: 0,
                use_calibrated_value: true,
                comparison_operator: "==".to_owned(),
                value: value.to_owned(),
            }),
            calibrator: xtce::CalibratorType {
                name: None,
                short_description: None,
                content: Vec::new(),
            },
        }
    }
}
