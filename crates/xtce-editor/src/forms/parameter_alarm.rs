use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt, button::Button, h_flex,
    input::InputState, select::SelectState, v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    ancillary_data_set::AncillaryDataSetForm, field, impl_select_item,
    input_algorithm::InputAlgorithmForm, message::MessageCriteriaForm,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AlarmKind {
    String,
    Enumerated,
    Numeric,
    Binary,
    Boolean,
    Time,
    Unsupported,
}

#[derive(Clone, Copy)]
pub(super) enum AlarmRef<'a> {
    String(&'a xtce::StringAlarmType),
    Enumerated(&'a xtce::EnumerationAlarmType),
    Numeric(&'a xtce::NumericAlarmType),
    Binary(&'a xtce::BinaryAlarmType),
    Boolean(&'a xtce::BooleanAlarmType),
    Time(&'a xtce::TimeAlarmType),
}

#[derive(Clone, Copy)]
pub(super) enum ContextAlarmListRef<'a> {
    String(&'a xtce::StringContextAlarmListType),
    Enumerated(&'a xtce::EnumerationContextAlarmListType),
    Numeric(&'a xtce::NumericContextAlarmListType),
    Binary(&'a xtce::BinaryContextAlarmListType),
    Boolean(&'a xtce::BooleanContextAlarmListType),
    Time(&'a xtce::TimeContextAlarmListType),
}

#[derive(Clone, Copy)]
enum ContextAlarmRef<'a> {
    String(&'a xtce::StringContextAlarmType),
    Enumerated(&'a xtce::EnumerationContextAlarmType),
    Numeric(&'a xtce::NumericContextAlarmType),
    Binary(&'a xtce::BinaryContextAlarmType),
    Boolean(&'a xtce::BooleanContextAlarmType),
    Time(&'a xtce::TimeContextAlarmType),
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ConcernLevelChoice {
    Normal,
    Watch,
    Warning,
    Distress,
    Critical,
    Severe,
}
impl_select_item!(ConcernLevelChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum RangeFormChoice {
    Outside,
    Inside,
}
impl_select_item!(RangeFormChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BoundaryChoice {
    Inclusive,
    Exclusive,
}
impl_select_item!(BoundaryChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum TimeUnitsChoice {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Picoseconds,
    Minutes,
    Hours,
    Days,
    Months,
    Years,
}
impl_select_item!(TimeUnitsChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ChangeSpanChoice {
    #[strum(serialize = "Change per second")]
    PerSecond,
    #[strum(serialize = "Change per sample")]
    PerSample,
}
impl_select_item!(ChangeSpanChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ChangeBasisChoice {
    #[strum(serialize = "Absolute change")]
    Absolute,
    #[strum(serialize = "Percentage change")]
    Percentage,
}
impl_select_item!(ChangeBasisChoice);

pub(super) struct AlarmEdit {
    pub(super) name: String,
    pub(super) short_description: String,
    pub(super) min_violations: i64,
    pub(super) min_conformance: i64,
    pub(super) disabled: bool,
    pub(super) default_alarm_level: xtce::ConcernLevelsType,
    pub(super) range_form: xtce::RangeFormType,
    pub(super) time_units: xtce::TimeUnitsType,
    pub(super) details: String,
    pub(super) static_ranges_active: bool,
    pub(super) ancillary_data_set: Option<xtce::AncillaryDataSetType>,
    pub(super) alarm_conditions: Option<xtce::AlarmConditionsType>,
    pub(super) custom_alarm: Option<xtce::CustomAlarmType>,
    pub(super) static_range_name: String,
    pub(super) static_range_short_description: String,
    pub(super) static_range_ancillary_data_set: Option<xtce::AncillaryDataSetType>,
    pub(super) change_alarm_ranges: Option<xtce::ChangeAlarmRangesType>,
    pub(super) alarm_multi_ranges: Option<xtce::AlarmMultiRangesType>,
    pub(super) change_per_second_alarm_ranges: Option<xtce::TimeAlarmRangesType>,
}

pub(super) struct ContextAlarmEdit {
    pub(super) context_match: xtce::ContextMatchType,
    pub(super) alarm: AlarmEdit,
}

struct AlarmEditor {
    kind: AlarmKind,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    min_violations: Entity<InputState>,
    min_conformance: Entity<InputState>,
    disabled: Entity<SelectState<Vec<BooleanChoice>>>,
    default_alarm_level: Entity<SelectState<Vec<ConcernLevelChoice>>>,
    details: Entity<InputState>,
    ancillary_data_set: AncillaryDataSetForm,
    alarm_conditions: Entity<AlarmConditionsForm>,
    custom_alarm: Entity<CustomAlarmForm>,
    static_alarm_ranges: Entity<StaticAlarmRangesForm>,
    change_alarm_ranges: Entity<ChangeAlarmRangesForm>,
    alarm_multi_ranges: Entity<AlarmMultiRangesForm>,
    change_per_second_alarm_ranges: Entity<TimeChangeAlarmRangesForm>,
}

pub(super) struct DefaultAlarmForm {
    kind: AlarmKind,
    active: bool,
    editor: Entity<AlarmEditor>,
}

pub(super) struct ContextAlarmListForm {
    kind: AlarmKind,
    rows: Vec<Entity<ContextAlarmRow>>,
}

struct ContextAlarmRow {
    criteria: Entity<MessageCriteriaForm>,
    editor: Entity<AlarmEditor>,
}

struct AlarmConditionsForm {
    rows: Vec<Entity<AlarmConditionRow>>,
}

struct AlarmConditionRow {
    level: &'static str,
    active: bool,
    criteria: Entity<MessageCriteriaForm>,
}

struct CustomAlarmForm {
    active: bool,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    ancillary_data_set: AncillaryDataSetForm,
    algorithm: Entity<InputAlgorithmForm>,
}

struct StaticAlarmRangesEdit {
    active: bool,
    name: String,
    short_description: String,
    range_form: xtce::RangeFormType,
    time_units: xtce::TimeUnitsType,
    details: String,
    ancillary_data_set: Option<xtce::AncillaryDataSetType>,
}

struct StaticAlarmRangesForm {
    kind: AlarmKind,
    active: bool,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    range_form: Entity<SelectState<Vec<RangeFormChoice>>>,
    time_units: Entity<SelectState<Vec<TimeUnitsChoice>>>,
    ancillary_data_set: AncillaryDataSetForm,
    rows: Vec<Entity<StaticAlarmRangeRow>>,
}

struct StaticAlarmRangeRow {
    level: &'static str,
    active: bool,
    minimum: Entity<InputState>,
    minimum_boundary: Entity<SelectState<Vec<BoundaryChoice>>>,
    maximum: Entity<InputState>,
    maximum_boundary: Entity<SelectState<Vec<BoundaryChoice>>>,
}

struct ChangeAlarmRangesForm {
    active: bool,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    range_form: Entity<SelectState<Vec<RangeFormChoice>>>,
    change_type: Entity<SelectState<Vec<ChangeSpanChoice>>>,
    change_basis: Entity<SelectState<Vec<ChangeBasisChoice>>>,
    span_samples: Entity<InputState>,
    span_seconds: Entity<InputState>,
    ancillary_data_set: AncillaryDataSetForm,
    details: Entity<InputState>,
}

struct AlarmMultiRangesForm {
    active: bool,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    ancillary_data_set: AncillaryDataSetForm,
    details: Entity<InputState>,
}

struct TimeChangeAlarmRangesForm {
    active: bool,
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    range_form: Entity<SelectState<Vec<RangeFormChoice>>>,
    time_units: Entity<SelectState<Vec<TimeUnitsChoice>>>,
    ancillary_data_set: AncillaryDataSetForm,
    details: Entity<InputState>,
}

impl DefaultAlarmForm {
    pub(super) fn new(
        kind: AlarmKind,
        alarm: Option<AlarmRef<'_>>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let editor = alarm_editor(kind, alarm.map(AlarmSource::Default), window, cx);
        cx.new(move |_| Self {
            kind,
            active: alarm.is_some(),
            editor,
        })
    }

    pub(super) fn load(
        &mut self,
        kind: AlarmKind,
        alarm: Option<AlarmRef<'_>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.kind = kind;
        self.active = alarm.is_some();
        self.editor = alarm_editor(kind, alarm.map(AlarmSource::Default), window, cx);
        cx.notify();
    }

    pub(super) fn edit(&self, cx: &App) -> Option<AlarmEdit> {
        self.active.then(|| self.editor.read(cx).edit(cx))
    }
}

impl Render for DefaultAlarmForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.kind == AlarmKind::Unsupported {
            return v_flex();
        }
        let header = h_flex()
            .justify_between()
            .child(
                v_flex()
                    .child(div().text_sm().font_medium().child("Default alarm"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Alarm behavior used when no context alarm matches"),
                    ),
            )
            .child(if self.active {
                super::section_remove_button("remove-parameter-type-default-alarm").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.active = false;
                        cx.notify();
                    }),
                )
            } else {
                super::section_add_button("add-parameter-type-default-alarm").on_click(cx.listener(
                    |this, _, window, cx| {
                        this.active = true;
                        this.editor = alarm_editor(this.kind, None, window, cx);
                        cx.notify();
                    },
                ))
            });
        v_flex()
            .w_full()
            .gap_3()
            .child(header)
            .children(self.active.then(|| self.editor.clone()))
    }
}

impl ContextAlarmListForm {
    pub(super) fn new(
        kind: AlarmKind,
        list: Option<ContextAlarmListRef<'_>>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let rows = context_alarm_refs(list)
            .into_iter()
            .map(|alarm| context_alarm_row(kind, Some(alarm), window, cx))
            .collect();
        cx.new(move |_| Self { kind, rows })
    }

    pub(super) fn load(
        &mut self,
        kind: AlarmKind,
        list: Option<ContextAlarmListRef<'_>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.kind = kind;
        self.rows = context_alarm_refs(list)
            .into_iter()
            .map(|alarm| context_alarm_row(kind, Some(alarm), window, cx))
            .collect();
        cx.notify();
    }

    pub(super) fn edits(&self, cx: &App) -> Vec<ContextAlarmEdit> {
        self.rows
            .iter()
            .map(|row| {
                let row = row.read(cx);
                ContextAlarmEdit {
                    context_match: row.criteria.read(cx).context_match(cx),
                    alarm: row.editor.read(cx).edit(cx),
                }
            })
            .collect()
    }
}

impl Render for ContextAlarmListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.kind == AlarmKind::Unsupported {
            return v_flex();
        }
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Context alarms"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(super::count_label(
                                        self.rows.len(),
                                        "context alarm",
                                        "context alarms",
                                    )),
                            ),
                    )
                    .child(
                        Button::new("add-parameter-type-context-alarm")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add context alarm")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.rows
                                    .push(context_alarm_row(this.kind, None, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .when(self.rows.is_empty(), |form| {
                form.child(super::empty_list_state("No context alarms defined.", cx))
            })
            .children(self.rows.iter().enumerate().map(|(index, row)| {
                super::detail_list_card(cx)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Context alarm {}", index + 1)),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-parameter-context-alarm-{index}"),
                                    "Remove context alarm",
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.rows.remove(index);
                                        cx.notify();
                                    },
                                )),
                            ),
                    )
                    .child(row.clone())
            }))
    }
}

impl Render for ContextAlarmRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .child(self.criteria.clone())
            .child(self.editor.clone())
    }
}

impl AlarmConditionsForm {
    fn new(
        conditions: Option<&xtce::AlarmConditionsType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = [
            (
                "Watch",
                conditions.and_then(|value| value.watch_alarm.as_ref()),
            ),
            (
                "Warning",
                conditions.and_then(|value| value.warning_alarm.as_ref()),
            ),
            (
                "Distress",
                conditions.and_then(|value| value.distress_alarm.as_ref()),
            ),
            (
                "Critical",
                conditions.and_then(|value| value.critical_alarm.as_ref()),
            ),
            (
                "Severe",
                conditions.and_then(|value| value.severe_alarm.as_ref()),
            ),
        ];
        let rows = values
            .into_iter()
            .map(|(level, criteria)| {
                let criteria_form = MessageCriteriaForm::new(criteria, window, cx);
                cx.new(move |_| AlarmConditionRow {
                    level,
                    active: criteria.is_some(),
                    criteria: criteria_form,
                })
            })
            .collect();
        cx.new(move |_| Self { rows })
    }

    fn value(&self, cx: &App) -> Option<xtce::AlarmConditionsType> {
        let mut criteria = self.rows.iter().map(|row| {
            let row = row.read(cx);
            row.active.then(|| {
                let mut value = default_match_criteria();
                row.criteria.read(cx).apply_to(&mut value, cx);
                value
            })
        });
        let value = xtce::AlarmConditionsType {
            watch_alarm: criteria.next().flatten(),
            warning_alarm: criteria.next().flatten(),
            distress_alarm: criteria.next().flatten(),
            critical_alarm: criteria.next().flatten(),
            severe_alarm: criteria.next().flatten(),
        };
        (value.watch_alarm.is_some()
            || value.warning_alarm.is_some()
            || value.distress_alarm.is_some()
            || value.critical_alarm.is_some()
            || value.severe_alarm.is_some())
        .then_some(value)
    }
}

impl Render for AlarmConditionsForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(div().text_sm().font_medium().child("Alarm conditions"))
            .children(self.rows.iter().map(|row| {
                let row_read = row.read(cx);
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(div().text_sm().child(row_read.level))
                            .child(if row_read.active {
                                super::section_remove_button(format!(
                                    "remove-{}-alarm-condition",
                                    row_read.level
                                ))
                                .on_click({
                                    let row = row.clone();
                                    move |_, _, cx| {
                                        row.update(cx, |row, cx| {
                                            row.active = false;
                                            cx.notify();
                                        });
                                    }
                                })
                            } else {
                                super::section_add_button(format!(
                                    "add-{}-alarm-condition",
                                    row_read.level
                                ))
                                .on_click({
                                    let row = row.clone();
                                    move |_, _, cx| {
                                        row.update(cx, |row, cx| {
                                            row.active = true;
                                            cx.notify();
                                        });
                                    }
                                })
                            }),
                    )
                    .children(row_read.active.then(|| row_read.criteria.clone()))
            }))
    }
}

impl CustomAlarmForm {
    fn new(
        custom: Option<&xtce::CustomAlarmType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let name = custom
            .and_then(|value| value.name.as_deref())
            .unwrap_or_default();
        let short_description = custom
            .and_then(|value| value.short_description.as_deref())
            .unwrap_or_default();
        let ancillary_data_set = AncillaryDataSetForm::new(
            custom.and_then(|value| value.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        let algorithm =
            InputAlgorithmForm::new(custom.map(|value| &value.input_algorithm), window, cx);
        cx.new(move |cx| Self {
            active: custom.is_some(),
            name: input(name, false, window, cx),
            short_description: input(short_description, false, window, cx),
            ancillary_data_set,
            algorithm,
        })
    }

    fn value(&self, cx: &App) -> Option<xtce::CustomAlarmType> {
        self.active.then(|| xtce::CustomAlarmType {
            name: optional_text(value(&self.name, cx)),
            short_description: optional_text(value(&self.short_description, cx)),
            ancillary_data_set: self.ancillary_data_set.value(cx),
            input_algorithm: self.algorithm.read(cx).algorithm(cx),
        })
    }
}

impl Render for CustomAlarmForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .justify_between()
            .child(div().text_sm().font_medium().child("Custom alarm"))
            .child(if self.active {
                super::section_remove_button("remove-parameter-custom-alarm").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.active = false;
                        cx.notify();
                    },
                ))
            } else {
                super::section_add_button("add-parameter-custom-alarm").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.active = true;
                        cx.notify();
                    },
                ))
            });
        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form
                .child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(field("Name", "Optional", &self.name, cx))
                        .child(field(
                            "Short description",
                            "Optional",
                            &self.short_description,
                            cx,
                        )),
                )
                .child(self.ancillary_data_set.render(cx))
                .child(self.algorithm.clone());
        }
        form
    }
}

impl StaticAlarmRangesForm {
    fn new(
        kind: AlarmKind,
        ranges: Option<AlarmRangeRef<'_>>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let name = ranges.and_then(AlarmRangeRef::name).unwrap_or_default();
        let short_description = ranges
            .and_then(AlarmRangeRef::short_description)
            .unwrap_or_default();
        let range_form = ranges
            .map(AlarmRangeRef::range_form)
            .unwrap_or(RangeFormChoice::Outside);
        let time_units = ranges
            .map(AlarmRangeRef::time_units)
            .unwrap_or(TimeUnitsChoice::Seconds);
        let ancillary_data_set = AncillaryDataSetForm::new(
            ranges.and_then(AlarmRangeRef::ancillary_data_set),
            window,
            cx,
        );
        let rows = static_alarm_range_rows(ranges, window, cx);
        cx.new(move |cx| Self {
            kind,
            active: ranges.is_some(),
            name: input(name, false, window, cx),
            short_description: input(short_description, false, window, cx),
            range_form: select(RangeFormChoice::VARIANTS, range_form, window, cx),
            time_units: select(TimeUnitsChoice::VARIANTS, time_units, window, cx),
            ancillary_data_set,
            rows,
        })
    }

    fn edit(&self, cx: &App) -> StaticAlarmRangesEdit {
        let details = self
            .rows
            .iter()
            .filter_map(|row| {
                let row = row.read(cx);
                row.active.then(|| {
                    format!(
                        "{} | {} | {} | {} | {}",
                        row.level.to_ascii_lowercase(),
                        value(&row.minimum, cx),
                        selected_value(&row.minimum_boundary, BoundaryChoice::Inclusive, cx)
                            .to_string()
                            .to_ascii_lowercase(),
                        value(&row.maximum, cx),
                        selected_value(&row.maximum_boundary, BoundaryChoice::Inclusive, cx)
                            .to_string()
                            .to_ascii_lowercase(),
                    )
                })
            })
            .collect::<Vec<_>>()
            .join("\n");
        StaticAlarmRangesEdit {
            active: self.active,
            name: value(&self.name, cx),
            short_description: value(&self.short_description, cx),
            range_form: selected_value(&self.range_form, RangeFormChoice::Outside, cx).to_xtce(),
            time_units: selected_value(&self.time_units, TimeUnitsChoice::Seconds, cx).to_xtce(),
            details,
            ancillary_data_set: self.ancillary_data_set.value(cx),
        }
    }
}

impl Render for StaticAlarmRangesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = optional_section_header(
            "Static alarm ranges",
            "static-alarm-ranges",
            self.active,
            cx,
            |this: &mut Self| &mut this.active,
        );
        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(field("Name", "Optional", &self.name, cx))
                        .child(field(
                            "Short description",
                            "Optional",
                            &self.short_description,
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(select_field("Range form", &self.range_form))
                        .children(
                            (self.kind == AlarmKind::Time)
                                .then(|| select_field("Time units", &self.time_units)),
                        ),
                )
                .children(self.rows.iter().map(|row| {
                    let row_read = row.read(cx);
                    let header = h_flex()
                        .justify_between()
                        .child(div().text_sm().font_medium().child(row_read.level))
                        .child(if row_read.active {
                            super::section_remove_button(format!(
                                "remove-static-range-{}",
                                row.entity_id()
                            ))
                            .on_click({
                                let row = row.clone();
                                move |_, _, cx| {
                                    row.update(cx, |row, cx| {
                                        row.active = false;
                                        cx.notify();
                                    });
                                }
                            })
                        } else {
                            super::section_add_button(format!(
                                "add-static-range-{}",
                                row.entity_id()
                            ))
                            .on_click({
                                let row = row.clone();
                                move |_, _, cx| {
                                    row.update(cx, |row, cx| {
                                        row.active = true;
                                        cx.notify();
                                    });
                                }
                            })
                        });
                    let mut range = super::detail_list_card(cx).child(header);
                    if row_read.active {
                        range = range
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_3()
                                    .items_start()
                                    .child(field("Minimum", "Optional", &row_read.minimum, cx))
                                    .child(select_field(
                                        "Minimum boundary",
                                        &row_read.minimum_boundary,
                                    )),
                            )
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_3()
                                    .items_start()
                                    .child(field("Maximum", "Optional", &row_read.maximum, cx))
                                    .child(select_field(
                                        "Maximum boundary",
                                        &row_read.maximum_boundary,
                                    )),
                            );
                    }
                    range
                }))
                .child(self.ancillary_data_set.render(cx));
        }
        form
    }
}

impl ChangeAlarmRangesForm {
    fn new(
        ranges: Option<&xtce::ChangeAlarmRangesType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let name = ranges
            .and_then(|value| value.name.as_deref())
            .unwrap_or_default();
        let short_description = ranges
            .and_then(|value| value.short_description.as_deref())
            .unwrap_or_default();
        let range_form = ranges
            .map(|value| RangeFormChoice::from_xtce(&value.range_form))
            .unwrap_or(RangeFormChoice::Outside);
        let change_type = ranges
            .map(|value| ChangeSpanChoice::from_xtce(&value.change_type))
            .unwrap_or(ChangeSpanChoice::PerSecond);
        let change_basis = ranges
            .map(|value| ChangeBasisChoice::from_xtce(&value.change_basis))
            .unwrap_or(ChangeBasisChoice::Absolute);
        let span_samples = ranges
            .map(|value| value.span_of_interest_in_samples.to_string())
            .unwrap_or_else(|| "1".to_owned());
        let span_seconds = ranges
            .map(|value| value.span_of_interest_in_seconds.to_string())
            .unwrap_or_else(|| "0".to_owned());
        let details = ranges
            .map(|value| range_details(Some(change_range_values(value))))
            .unwrap_or_else(default_range_details);
        let ancillary_data_set = AncillaryDataSetForm::new(
            ranges.and_then(|value| value.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        cx.new(move |cx| Self {
            active: ranges.is_some(),
            name: input(name, false, window, cx),
            short_description: input(short_description, false, window, cx),
            range_form: select(RangeFormChoice::VARIANTS, range_form, window, cx),
            change_type: select(ChangeSpanChoice::VARIANTS, change_type, window, cx),
            change_basis: select(ChangeBasisChoice::VARIANTS, change_basis, window, cx),
            span_samples: input(&span_samples, false, window, cx),
            span_seconds: input(&span_seconds, false, window, cx),
            ancillary_data_set,
            details: input(&details, true, window, cx),
        })
    }

    fn value(&self, cx: &App) -> Option<xtce::ChangeAlarmRangesType> {
        self.active.then(|| {
            let (watch_range, warning_range, distress_range, critical_range, severe_range) =
                split_ranges(parse_alarm_ranges(&value(&self.details, cx)));
            xtce::ChangeAlarmRangesType {
                name: optional_text(value(&self.name, cx)),
                short_description: optional_text(value(&self.short_description, cx)),
                range_form: selected_value(&self.range_form, RangeFormChoice::Outside, cx)
                    .to_xtce(),
                change_type: selected_value(&self.change_type, ChangeSpanChoice::PerSecond, cx)
                    .to_xtce(),
                change_basis: selected_value(&self.change_basis, ChangeBasisChoice::Absolute, cx)
                    .to_xtce(),
                span_of_interest_in_samples: parse_or(&value(&self.span_samples, cx), 1),
                span_of_interest_in_seconds: parse_or(&value(&self.span_seconds, cx), 0.0),
                ancillary_data_set: self.ancillary_data_set.value(cx),
                watch_range,
                warning_range,
                distress_range,
                critical_range,
                severe_range,
            }
        })
    }
}

impl Render for ChangeAlarmRangesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = optional_section_header(
            "Change alarm ranges",
            "change-alarm-ranges",
            self.active,
            cx,
            |this: &mut Self| &mut this.active,
        );
        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(field("Name", "Optional", &self.name, cx))
                        .child(field(
                            "Short description",
                            "Optional",
                            &self.short_description,
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(select_field("Range form", &self.range_form))
                        .child(select_field("Change type", &self.change_type))
                        .child(select_field("Change basis", &self.change_basis)),
                )
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(field(
                            "Span in samples",
                            "Defaults to 1",
                            &self.span_samples,
                            cx,
                        ))
                        .child(field(
                            "Span in seconds",
                            "Defaults to 0",
                            &self.span_seconds,
                            cx,
                        )),
                )
                .child(range_details_field(&self.details, cx))
                .child(self.ancillary_data_set.render(cx));
        }
        form
    }
}

impl AlarmMultiRangesForm {
    fn new(
        ranges: Option<&xtce::AlarmMultiRangesType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let name = ranges
            .and_then(|value| value.name.as_deref())
            .unwrap_or_default();
        let short_description = ranges
            .and_then(|value| value.short_description.as_deref())
            .unwrap_or_default();
        let details = ranges
            .map(multi_range_details)
            .unwrap_or_else(default_multi_range_details);
        let ancillary_data_set = AncillaryDataSetForm::new(
            ranges.and_then(|value| value.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        cx.new(move |cx| Self {
            active: ranges.is_some(),
            name: input(name, false, window, cx),
            short_description: input(short_description, false, window, cx),
            ancillary_data_set,
            details: input(&details, true, window, cx),
        })
    }

    fn value(&self, cx: &App) -> Option<xtce::AlarmMultiRangesType> {
        let ranges = parse_multi_ranges(&value(&self.details, cx));
        (self.active && !ranges.is_empty()).then(|| xtce::AlarmMultiRangesType {
            name: optional_text(value(&self.name, cx)),
            short_description: optional_text(value(&self.short_description, cx)),
            ancillary_data_set: self.ancillary_data_set.value(cx),
            range: ranges,
        })
    }
}

impl Render for AlarmMultiRangesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = optional_section_header(
            "Multiple alarm ranges",
            "alarm-multi-ranges",
            self.active,
            cx,
            |this: &mut Self| &mut this.active,
        );
        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(field("Name", "Optional", &self.name, cx))
                        .child(field(
                            "Short description",
                            "Optional",
                            &self.short_description,
                            cx,
                        )),
                )
                .child(field(
                    "Ranges",
                    "One per line: level | range form | minimum | inclusive/exclusive | maximum | inclusive/exclusive",
                    &self.details,
                    cx,
                ))
                .child(self.ancillary_data_set.render(cx));
        }
        form
    }
}

impl TimeChangeAlarmRangesForm {
    fn new(
        ranges: Option<&xtce::TimeAlarmRangesType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let name = ranges
            .and_then(|value| value.name.as_deref())
            .unwrap_or_default();
        let short_description = ranges
            .and_then(|value| value.short_description.as_deref())
            .unwrap_or_default();
        let range_form = ranges
            .map(|value| RangeFormChoice::from_xtce(&value.range_form))
            .unwrap_or(RangeFormChoice::Outside);
        let time_units = ranges
            .map(|value| TimeUnitsChoice::from_xtce(&value.time_units))
            .unwrap_or(TimeUnitsChoice::Seconds);
        let details = ranges
            .map(|value| range_details(Some(time_range_values(value))))
            .unwrap_or_else(default_range_details);
        let ancillary_data_set = AncillaryDataSetForm::new(
            ranges.and_then(|value| value.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        cx.new(move |cx| Self {
            active: ranges.is_some(),
            name: input(name, false, window, cx),
            short_description: input(short_description, false, window, cx),
            range_form: select(RangeFormChoice::VARIANTS, range_form, window, cx),
            time_units: select(TimeUnitsChoice::VARIANTS, time_units, window, cx),
            ancillary_data_set,
            details: input(&details, true, window, cx),
        })
    }

    fn value(&self, cx: &App) -> Option<xtce::TimeAlarmRangesType> {
        self.active.then(|| {
            let (watch_range, warning_range, distress_range, critical_range, severe_range) =
                split_ranges(parse_alarm_ranges(&value(&self.details, cx)));
            xtce::TimeAlarmRangesType {
                name: optional_text(value(&self.name, cx)),
                short_description: optional_text(value(&self.short_description, cx)),
                range_form: selected_value(&self.range_form, RangeFormChoice::Outside, cx)
                    .to_xtce(),
                time_units: selected_value(&self.time_units, TimeUnitsChoice::Seconds, cx)
                    .to_xtce(),
                ancillary_data_set: self.ancillary_data_set.value(cx),
                watch_range,
                warning_range,
                distress_range,
                critical_range,
                severe_range,
            }
        })
    }
}

impl Render for TimeChangeAlarmRangesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = optional_section_header(
            "Change per second alarm ranges",
            "time-change-alarm-ranges",
            self.active,
            cx,
            |this: &mut Self| &mut this.active,
        );
        let mut form = v_flex().w_full().gap_3().child(header);
        if self.active {
            form = form
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(field("Name", "Optional", &self.name, cx))
                        .child(field(
                            "Short description",
                            "Optional",
                            &self.short_description,
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(select_field("Range form", &self.range_form))
                        .child(select_field("Time units", &self.time_units)),
                )
                .child(range_details_field(&self.details, cx))
                .child(self.ancillary_data_set.render(cx));
        }
        form
    }
}

impl AlarmEditor {
    fn edit(&self, cx: &App) -> AlarmEdit {
        let static_ranges = self.static_alarm_ranges.read(cx).edit(cx);
        let uses_static_ranges = matches!(self.kind, AlarmKind::Numeric | AlarmKind::Time);
        AlarmEdit {
            name: value(&self.name, cx),
            short_description: value(&self.short_description, cx),
            min_violations: parse_or(&value(&self.min_violations, cx), 1),
            min_conformance: parse_or(&value(&self.min_conformance, cx), 1),
            disabled: selected_value(&self.disabled, BooleanChoice::False, cx)
                == BooleanChoice::True,
            default_alarm_level: selected_value(
                &self.default_alarm_level,
                ConcernLevelChoice::Normal,
                cx,
            )
            .to_xtce(),
            range_form: static_ranges.range_form,
            time_units: static_ranges.time_units,
            details: if uses_static_ranges {
                static_ranges.details
            } else {
                value(&self.details, cx)
            },
            static_ranges_active: uses_static_ranges && static_ranges.active,
            ancillary_data_set: self.ancillary_data_set.value(cx),
            alarm_conditions: self.alarm_conditions.read(cx).value(cx),
            custom_alarm: self.custom_alarm.read(cx).value(cx),
            static_range_name: static_ranges.name,
            static_range_short_description: static_ranges.short_description,
            static_range_ancillary_data_set: static_ranges.ancillary_data_set,
            change_alarm_ranges: self.change_alarm_ranges.read(cx).value(cx),
            alarm_multi_ranges: self.alarm_multi_ranges.read(cx).value(cx),
            change_per_second_alarm_ranges: self.change_per_second_alarm_ranges.read(cx).value(cx),
        }
    }
}

impl Render for AlarmEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut form = v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_start()
                    .child(field("Name", "Optional", &self.name, cx))
                    .child(select_field("Disabled", &self.disabled)),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_start()
                    .child(field(
                        "Minimum violations",
                        "Defaults to 1",
                        &self.min_violations,
                        cx,
                    ))
                    .child(field(
                        "Minimum conformance",
                        "Defaults to 1",
                        &self.min_conformance,
                        cx,
                    )),
            );
        if matches!(self.kind, AlarmKind::String | AlarmKind::Enumerated) {
            form = form.child(select_field(
                "Default alarm level",
                &self.default_alarm_level,
            ));
        }
        if self.kind.has_details() {
            form = form.child(field(
                self.kind.details_label(),
                self.kind.details_hint(),
                &self.details,
                cx,
            ));
        }
        if matches!(self.kind, AlarmKind::Numeric | AlarmKind::Time) {
            form = form.child(self.static_alarm_ranges.clone());
        }
        let mut form = form
            .child(field(
            "Short description",
            "Optional",
            &self.short_description,
            cx,
        ))
        .child(self.ancillary_data_set.render(cx))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "XTCE allows either Alarm conditions or Custom alarm. If both are enabled, Custom alarm is saved.",
                ),
        )
        .child(self.alarm_conditions.clone())
        .child(self.custom_alarm.clone());
        if self.kind == AlarmKind::Numeric {
            form = form
                .child(self.change_alarm_ranges.clone())
                .child(self.alarm_multi_ranges.clone());
        }
        if self.kind == AlarmKind::Time {
            form = form.child(self.change_per_second_alarm_ranges.clone());
        }
        form
    }
}

impl AlarmKind {
    fn has_details(self) -> bool {
        matches!(self, Self::String | Self::Enumerated)
    }

    fn details_label(self) -> &'static str {
        match self {
            Self::String => "String alarm list",
            Self::Enumerated => "Enumeration alarm list",
            _ => "Alarm details",
        }
    }

    fn details_hint(self) -> &'static str {
        match self {
            Self::String => "One per line: level | match pattern",
            Self::Enumerated => "One per line: level | enumeration label",
            _ => "",
        }
    }
}

#[derive(Clone, Copy)]
enum AlarmSource<'a> {
    Default(AlarmRef<'a>),
    Context(ContextAlarmRef<'a>),
}

#[derive(Clone, Copy)]
enum AlarmRangeRef<'a> {
    Numeric(&'a xtce::AlarmRangesType),
    Time(&'a xtce::TimeAlarmRangesType),
}

impl<'a> AlarmRangeRef<'a> {
    fn name(self) -> Option<&'a str> {
        match self {
            Self::Numeric(value) => value.name.as_deref(),
            Self::Time(value) => value.name.as_deref(),
        }
    }

    fn short_description(self) -> Option<&'a str> {
        match self {
            Self::Numeric(value) => value.short_description.as_deref(),
            Self::Time(value) => value.short_description.as_deref(),
        }
    }

    fn ancillary_data_set(self) -> Option<&'a xtce::AncillaryDataSetType> {
        match self {
            Self::Numeric(value) => value.ancillary_data_set.as_ref(),
            Self::Time(value) => value.ancillary_data_set.as_ref(),
        }
    }

    fn range_form(self) -> RangeFormChoice {
        match self {
            Self::Numeric(value) => RangeFormChoice::from_xtce(&value.range_form),
            Self::Time(value) => RangeFormChoice::from_xtce(&value.range_form),
        }
    }

    fn time_units(self) -> TimeUnitsChoice {
        match self {
            Self::Numeric(_) => TimeUnitsChoice::Seconds,
            Self::Time(value) => TimeUnitsChoice::from_xtce(&value.time_units),
        }
    }

    fn ranges(self) -> [Option<&'a xtce::FloatRangeType>; 5] {
        match self {
            Self::Numeric(value) => [
                value.watch_range.as_ref(),
                value.warning_range.as_ref(),
                value.distress_range.as_ref(),
                value.critical_range.as_ref(),
                value.severe_range.as_ref(),
            ],
            Self::Time(value) => [
                value.watch_range.as_ref(),
                value.warning_range.as_ref(),
                value.distress_range.as_ref(),
                value.critical_range.as_ref(),
                value.severe_range.as_ref(),
            ],
        }
    }
}

fn static_alarm_range_rows(
    ranges: Option<AlarmRangeRef<'_>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Vec<Entity<StaticAlarmRangeRow>> {
    let values = ranges
        .map(AlarmRangeRef::ranges)
        .unwrap_or([None, None, None, None, None]);
    let mut rows = Vec::new();
    for (level, range) in ["Watch", "Warning", "Distress", "Critical", "Severe"]
        .into_iter()
        .zip(values)
    {
        let (minimum, minimum_boundary) = range.map(range_minimum).unwrap_or_default();
        let (maximum, maximum_boundary) = range.map(range_maximum).unwrap_or_default();
        let row_window = &mut *window;
        rows.push(cx.new(move |cx| StaticAlarmRangeRow {
            level,
            active: range.is_some(),
            minimum: input(&minimum, false, row_window, cx),
            minimum_boundary: select(
                BoundaryChoice::VARIANTS,
                BoundaryChoice::from_text(minimum_boundary),
                row_window,
                cx,
            ),
            maximum: input(&maximum, false, row_window, cx),
            maximum_boundary: select(
                BoundaryChoice::VARIANTS,
                BoundaryChoice::from_text(maximum_boundary),
                row_window,
                cx,
            ),
        }));
    }
    rows
}

struct AlarmValues {
    name: String,
    short_description: String,
    min_violations: String,
    min_conformance: String,
    disabled: BooleanChoice,
    default_alarm_level: ConcernLevelChoice,
    details: String,
}

impl AlarmValues {
    fn from_source(kind: AlarmKind, source: Option<AlarmSource<'_>>) -> Self {
        let Some(source) = source else {
            return Self::defaults(kind);
        };
        match source {
            AlarmSource::Default(value) => Self::from_default(kind, value),
            AlarmSource::Context(value) => Self::from_context(kind, value),
        }
    }

    fn from_default(kind: AlarmKind, value: AlarmRef<'_>) -> Self {
        match value {
            AlarmRef::String(value) => Self::common(
                kind,
                value.name.as_deref(),
                value.short_description.as_deref(),
                value.min_violations,
                value.min_conformance,
                value.disabled,
                Some(&value.default_alarm_level),
                string_details(value.content.iter().find_map(|item| match item {
                    xtce::StringAlarmTypeContent::StringAlarmList(value) => Some(value),
                    _ => None,
                })),
                None,
                None,
            ),
            AlarmRef::Enumerated(value) => Self::common(
                kind,
                value.name.as_deref(),
                value.short_description.as_deref(),
                value.min_violations,
                value.min_conformance,
                value.disabled,
                Some(&value.default_alarm_level),
                enumeration_details(value.content.iter().find_map(|item| match item {
                    xtce::EnumerationAlarmTypeContent::EnumerationAlarmList(value) => Some(value),
                    _ => None,
                })),
                None,
                None,
            ),
            AlarmRef::Numeric(value) => {
                let ranges = value.content.iter().find_map(|item| match item {
                    xtce::NumericAlarmTypeContent::StaticAlarmRanges(value) => Some(value),
                    _ => None,
                });
                Self::common(
                    kind,
                    value.name.as_deref(),
                    value.short_description.as_deref(),
                    value.min_violations,
                    value.min_conformance,
                    value.disabled,
                    None,
                    range_details(ranges.map(range_values)),
                    ranges.map(|value| &value.range_form),
                    None,
                )
            }
            AlarmRef::Binary(value) => Self::basic(
                kind,
                value.name.as_deref(),
                value.short_description.as_deref(),
                value.min_violations,
                value.min_conformance,
                value.disabled,
            ),
            AlarmRef::Boolean(value) => Self::basic(
                kind,
                value.name.as_deref(),
                value.short_description.as_deref(),
                value.min_violations,
                value.min_conformance,
                value.disabled,
            ),
            AlarmRef::Time(value) => {
                let ranges = value.content.iter().find_map(|item| match item {
                    xtce::TimeAlarmTypeContent::StaticAlarmRanges(value) => Some(value),
                    _ => None,
                });
                Self::common(
                    kind,
                    value.name.as_deref(),
                    value.short_description.as_deref(),
                    value.min_violations,
                    value.min_conformance,
                    value.disabled,
                    None,
                    range_details(ranges.map(time_range_values)),
                    ranges.map(|value| &value.range_form),
                    ranges.map(|value| &value.time_units),
                )
            }
        }
    }

    fn from_context(kind: AlarmKind, value: ContextAlarmRef<'_>) -> Self {
        macro_rules! context {
            ($value:expr, $level:expr, $details:expr, $range:expr, $units:expr) => {
                Self::common(
                    kind,
                    $value.name.as_deref(),
                    $value.short_description.as_deref(),
                    $value.min_violations,
                    $value.min_conformance,
                    $value.disabled,
                    $level,
                    $details,
                    $range,
                    $units,
                )
            };
        }
        match value {
            ContextAlarmRef::String(value) => context!(
                value,
                Some(&value.default_alarm_level),
                string_details(value.content.iter().find_map(|item| match item {
                    xtce::StringContextAlarmTypeContent::StringAlarmList(value) => Some(value),
                    _ => None,
                })),
                None,
                None
            ),
            ContextAlarmRef::Enumerated(value) => context!(
                value,
                Some(&value.default_alarm_level),
                enumeration_details(value.content.iter().find_map(|item| match item {
                    xtce::EnumerationContextAlarmTypeContent::EnumerationAlarmList(value) =>
                        Some(value),
                    _ => None,
                })),
                None,
                None
            ),
            ContextAlarmRef::Numeric(value) => {
                let ranges = value.content.iter().find_map(|item| match item {
                    xtce::NumericContextAlarmTypeContent::StaticAlarmRanges(value) => Some(value),
                    _ => None,
                });
                context!(
                    value,
                    None,
                    range_details(ranges.map(range_values)),
                    ranges.map(|value| &value.range_form),
                    None
                )
            }
            ContextAlarmRef::Binary(value) => context!(value, None, String::new(), None, None),
            ContextAlarmRef::Boolean(value) => context!(value, None, String::new(), None, None),
            ContextAlarmRef::Time(value) => {
                let ranges = value.content.iter().find_map(|item| match item {
                    xtce::TimeContextAlarmTypeContent::StaticAlarmRanges(value) => Some(value),
                    _ => None,
                });
                context!(
                    value,
                    None,
                    range_details(ranges.map(time_range_values)),
                    ranges.map(|value| &value.range_form),
                    ranges.map(|value| &value.time_units)
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn common(
        kind: AlarmKind,
        name: Option<&str>,
        short_description: Option<&str>,
        min_violations: i64,
        min_conformance: i64,
        disabled: bool,
        default_alarm_level: Option<&xtce::ConcernLevelsType>,
        details: String,
        _range_form: Option<&xtce::RangeFormType>,
        _time_units: Option<&xtce::TimeUnitsType>,
    ) -> Self {
        Self {
            name: name.unwrap_or_default().to_owned(),
            short_description: short_description.unwrap_or_default().to_owned(),
            min_violations: min_violations.to_string(),
            min_conformance: min_conformance.to_string(),
            disabled: bool_choice(disabled),
            default_alarm_level: default_alarm_level
                .map(ConcernLevelChoice::from_xtce)
                .unwrap_or(ConcernLevelChoice::Normal),
            details: if details.is_empty() {
                default_details(kind)
            } else {
                details
            },
        }
    }

    fn basic(
        kind: AlarmKind,
        name: Option<&str>,
        short_description: Option<&str>,
        min_violations: i64,
        min_conformance: i64,
        disabled: bool,
    ) -> Self {
        Self::common(
            kind,
            name,
            short_description,
            min_violations,
            min_conformance,
            disabled,
            None,
            String::new(),
            None,
            None,
        )
    }

    fn defaults(kind: AlarmKind) -> Self {
        Self::common(
            kind,
            None,
            None,
            1,
            1,
            false,
            None,
            String::new(),
            None,
            None,
        )
    }
}

fn alarm_editor(
    kind: AlarmKind,
    source: Option<AlarmSource<'_>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<AlarmEditor> {
    let values = AlarmValues::from_source(kind, source);
    let ancillary = source.as_ref().and_then(alarm_ancillary);
    let conditions = source.as_ref().and_then(alarm_conditions);
    let custom = source.as_ref().and_then(custom_alarm);
    let static_ranges = source.as_ref().and_then(static_alarm_ranges);
    let change_ranges = source.as_ref().and_then(change_alarm_ranges);
    let multi_ranges = source.as_ref().and_then(alarm_multi_ranges);
    let time_change_ranges = source.as_ref().and_then(change_per_second_alarm_ranges);
    let alarm_conditions = AlarmConditionsForm::new(conditions, window, cx);
    let custom_alarm = CustomAlarmForm::new(custom, window, cx);
    let static_alarm_ranges = StaticAlarmRangesForm::new(kind, static_ranges, window, cx);
    let change_alarm_ranges = ChangeAlarmRangesForm::new(change_ranges, window, cx);
    let alarm_multi_ranges = AlarmMultiRangesForm::new(multi_ranges, window, cx);
    let change_per_second_alarm_ranges =
        TimeChangeAlarmRangesForm::new(time_change_ranges, window, cx);
    cx.new(move |cx| AlarmEditor {
        kind,
        name: input(&values.name, false, window, cx),
        short_description: input(&values.short_description, false, window, cx),
        min_violations: input(&values.min_violations, false, window, cx),
        min_conformance: input(&values.min_conformance, false, window, cx),
        disabled: select(BooleanChoice::VARIANTS, values.disabled, window, cx),
        default_alarm_level: select(
            ConcernLevelChoice::VARIANTS,
            values.default_alarm_level,
            window,
            cx,
        ),
        details: input(&values.details, true, window, cx),
        ancillary_data_set: AncillaryDataSetForm::new(ancillary, window, cx),
        alarm_conditions,
        custom_alarm,
        static_alarm_ranges,
        change_alarm_ranges,
        alarm_multi_ranges,
        change_per_second_alarm_ranges,
    })
}

fn context_alarm_row(
    kind: AlarmKind,
    alarm: Option<ContextAlarmRef<'_>>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ContextAlarmRow> {
    let criteria = MessageCriteriaForm::new_context(alarm.and_then(context_match), window, cx);
    let editor = alarm_editor(kind, alarm.map(AlarmSource::Context), window, cx);
    cx.new(move |_| ContextAlarmRow { criteria, editor })
}

fn context_alarm_refs(list: Option<ContextAlarmListRef<'_>>) -> Vec<ContextAlarmRef<'_>> {
    match list {
        Some(ContextAlarmListRef::String(list)) => list
            .context_alarm
            .iter()
            .map(ContextAlarmRef::String)
            .collect(),
        Some(ContextAlarmListRef::Enumerated(list)) => list
            .context_alarm
            .iter()
            .map(ContextAlarmRef::Enumerated)
            .collect(),
        Some(ContextAlarmListRef::Numeric(list)) => list
            .context_alarm
            .iter()
            .map(ContextAlarmRef::Numeric)
            .collect(),
        Some(ContextAlarmListRef::Binary(list)) => list
            .context_alarm
            .iter()
            .map(ContextAlarmRef::Binary)
            .collect(),
        Some(ContextAlarmListRef::Boolean(list)) => list
            .context_alarm
            .iter()
            .map(ContextAlarmRef::Boolean)
            .collect(),
        Some(ContextAlarmListRef::Time(list)) => list
            .context_alarm
            .iter()
            .map(ContextAlarmRef::Time)
            .collect(),
        None => Vec::new(),
    }
}

fn context_match(alarm: ContextAlarmRef<'_>) -> Option<&xtce::ContextMatchType> {
    macro_rules! find {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::ContextMatch(value) => Some(value),
                _ => None,
            })
        };
    }
    match alarm {
        ContextAlarmRef::String(value) => find!(value, StringContextAlarmTypeContent),
        ContextAlarmRef::Enumerated(value) => find!(value, EnumerationContextAlarmTypeContent),
        ContextAlarmRef::Numeric(value) => find!(value, NumericContextAlarmTypeContent),
        ContextAlarmRef::Binary(value) => find!(value, BinaryContextAlarmTypeContent),
        ContextAlarmRef::Boolean(value) => find!(value, BooleanContextAlarmTypeContent),
        ContextAlarmRef::Time(value) => find!(value, TimeContextAlarmTypeContent),
    }
}

fn alarm_ancillary<'a>(source: &AlarmSource<'a>) -> Option<&'a xtce::AncillaryDataSetType> {
    macro_rules! find {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::AncillaryDataSet(value) => Some(value),
                _ => None,
            })
        };
    }
    match source {
        AlarmSource::Default(AlarmRef::String(value)) => find!(value, StringAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Enumerated(value)) => {
            find!(value, EnumerationAlarmTypeContent)
        }
        AlarmSource::Default(AlarmRef::Numeric(value)) => find!(value, NumericAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Binary(value)) => find!(value, BinaryAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Boolean(value)) => find!(value, BooleanAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Time(value)) => find!(value, TimeAlarmTypeContent),
        AlarmSource::Context(ContextAlarmRef::String(value)) => {
            find!(value, StringContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Enumerated(value)) => {
            find!(value, EnumerationContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Numeric(value)) => {
            find!(value, NumericContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Binary(value)) => {
            find!(value, BinaryContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Boolean(value)) => {
            find!(value, BooleanContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Time(value)) => {
            find!(value, TimeContextAlarmTypeContent)
        }
    }
}

fn alarm_conditions<'a>(source: &AlarmSource<'a>) -> Option<&'a xtce::AlarmConditionsType> {
    macro_rules! find {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::AlarmConditions(value) => Some(value),
                _ => None,
            })
        };
    }
    match source {
        AlarmSource::Default(AlarmRef::String(value)) => find!(value, StringAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Enumerated(value)) => {
            find!(value, EnumerationAlarmTypeContent)
        }
        AlarmSource::Default(AlarmRef::Numeric(value)) => find!(value, NumericAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Binary(value)) => find!(value, BinaryAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Boolean(value)) => find!(value, BooleanAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Time(value)) => find!(value, TimeAlarmTypeContent),
        AlarmSource::Context(ContextAlarmRef::String(value)) => {
            find!(value, StringContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Enumerated(value)) => {
            find!(value, EnumerationContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Numeric(value)) => {
            find!(value, NumericContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Binary(value)) => {
            find!(value, BinaryContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Boolean(value)) => {
            find!(value, BooleanContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Time(value)) => {
            find!(value, TimeContextAlarmTypeContent)
        }
    }
}

fn custom_alarm<'a>(source: &AlarmSource<'a>) -> Option<&'a xtce::CustomAlarmType> {
    macro_rules! find {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::CustomAlarm(value) => Some(value),
                _ => None,
            })
        };
    }
    match source {
        AlarmSource::Default(AlarmRef::String(value)) => find!(value, StringAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Enumerated(value)) => {
            find!(value, EnumerationAlarmTypeContent)
        }
        AlarmSource::Default(AlarmRef::Numeric(value)) => find!(value, NumericAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Binary(value)) => find!(value, BinaryAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Boolean(value)) => find!(value, BooleanAlarmTypeContent),
        AlarmSource::Default(AlarmRef::Time(value)) => find!(value, TimeAlarmTypeContent),
        AlarmSource::Context(ContextAlarmRef::String(value)) => {
            find!(value, StringContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Enumerated(value)) => {
            find!(value, EnumerationContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Numeric(value)) => {
            find!(value, NumericContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Binary(value)) => {
            find!(value, BinaryContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Boolean(value)) => {
            find!(value, BooleanContextAlarmTypeContent)
        }
        AlarmSource::Context(ContextAlarmRef::Time(value)) => {
            find!(value, TimeContextAlarmTypeContent)
        }
    }
}

fn static_alarm_ranges<'a>(source: &AlarmSource<'a>) -> Option<AlarmRangeRef<'a>> {
    match source {
        AlarmSource::Default(AlarmRef::Numeric(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::NumericAlarmTypeContent::StaticAlarmRanges(value) => {
                    Some(AlarmRangeRef::Numeric(value))
                }
                _ => None,
            })
        }
        AlarmSource::Context(ContextAlarmRef::Numeric(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::NumericContextAlarmTypeContent::StaticAlarmRanges(value) => {
                    Some(AlarmRangeRef::Numeric(value))
                }
                _ => None,
            })
        }
        AlarmSource::Default(AlarmRef::Time(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::TimeAlarmTypeContent::StaticAlarmRanges(value) => {
                    Some(AlarmRangeRef::Time(value))
                }
                _ => None,
            })
        }
        AlarmSource::Context(ContextAlarmRef::Time(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::TimeContextAlarmTypeContent::StaticAlarmRanges(value) => {
                    Some(AlarmRangeRef::Time(value))
                }
                _ => None,
            })
        }
        _ => None,
    }
}

fn change_alarm_ranges<'a>(source: &AlarmSource<'a>) -> Option<&'a xtce::ChangeAlarmRangesType> {
    match source {
        AlarmSource::Default(AlarmRef::Numeric(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::NumericAlarmTypeContent::ChangeAlarmRanges(value) => Some(value),
                _ => None,
            })
        }
        AlarmSource::Context(ContextAlarmRef::Numeric(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::NumericContextAlarmTypeContent::ChangeAlarmRanges(value) => Some(value),
                _ => None,
            })
        }
        _ => None,
    }
}

fn alarm_multi_ranges<'a>(source: &AlarmSource<'a>) -> Option<&'a xtce::AlarmMultiRangesType> {
    match source {
        AlarmSource::Default(AlarmRef::Numeric(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::NumericAlarmTypeContent::AlarmMultiRanges(value) => Some(value),
                _ => None,
            })
        }
        AlarmSource::Context(ContextAlarmRef::Numeric(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::NumericContextAlarmTypeContent::AlarmMultiRanges(value) => Some(value),
                _ => None,
            })
        }
        _ => None,
    }
}

fn change_per_second_alarm_ranges<'a>(
    source: &AlarmSource<'a>,
) -> Option<&'a xtce::TimeAlarmRangesType> {
    match source {
        AlarmSource::Default(AlarmRef::Time(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::TimeAlarmTypeContent::ChangePerSecondAlarmRanges(value) => Some(value),
                _ => None,
            })
        }
        AlarmSource::Context(ContextAlarmRef::Time(value)) => {
            value.content.iter().find_map(|item| match item {
                xtce::TimeContextAlarmTypeContent::ChangePerSecondAlarmRanges(value) => Some(value),
                _ => None,
            })
        }
        _ => None,
    }
}

fn string_details(list: Option<&xtce::StringAlarmListType>) -> String {
    list.into_iter()
        .flat_map(|list| &list.string_alarm)
        .map(|alarm| {
            format!(
                "{} | {}",
                concern_text(&alarm.alarm_level),
                alarm.match_pattern
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn enumeration_details(list: Option<&xtce::EnumerationAlarmListType>) -> String {
    list.into_iter()
        .flat_map(|list| &list.enumeration_alarm)
        .map(|alarm| {
            format!(
                "{} | {}",
                concern_text(&alarm.alarm_level),
                alarm.enumeration_label
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn range_details(ranges: Option<Vec<(&'static str, &xtce::FloatRangeType)>>) -> String {
    ranges
        .unwrap_or_default()
        .into_iter()
        .map(|(level, range)| {
            let (minimum, minimum_form) = range_minimum(range);
            let (maximum, maximum_form) = range_maximum(range);
            format!("{level} | {minimum} | {minimum_form} | {maximum} | {maximum_form}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn range_values(value: &xtce::AlarmRangesType) -> Vec<(&'static str, &xtce::FloatRangeType)> {
    alarm_ranges(
        value.watch_range.as_ref(),
        value.warning_range.as_ref(),
        value.distress_range.as_ref(),
        value.critical_range.as_ref(),
        value.severe_range.as_ref(),
    )
}

fn change_range_values(
    value: &xtce::ChangeAlarmRangesType,
) -> Vec<(&'static str, &xtce::FloatRangeType)> {
    alarm_ranges(
        value.watch_range.as_ref(),
        value.warning_range.as_ref(),
        value.distress_range.as_ref(),
        value.critical_range.as_ref(),
        value.severe_range.as_ref(),
    )
}

fn time_range_values(
    value: &xtce::TimeAlarmRangesType,
) -> Vec<(&'static str, &xtce::FloatRangeType)> {
    alarm_ranges(
        value.watch_range.as_ref(),
        value.warning_range.as_ref(),
        value.distress_range.as_ref(),
        value.critical_range.as_ref(),
        value.severe_range.as_ref(),
    )
}

fn alarm_ranges<'a>(
    watch: Option<&'a xtce::FloatRangeType>,
    warning: Option<&'a xtce::FloatRangeType>,
    distress: Option<&'a xtce::FloatRangeType>,
    critical: Option<&'a xtce::FloatRangeType>,
    severe: Option<&'a xtce::FloatRangeType>,
) -> Vec<(&'static str, &'a xtce::FloatRangeType)> {
    [
        ("watch", watch),
        ("warning", warning),
        ("distress", distress),
        ("critical", critical),
        ("severe", severe),
    ]
    .into_iter()
    .filter_map(|(level, range)| range.map(|range| (level, range)))
    .collect()
}

fn default_range_details() -> String {
    default_details(AlarmKind::Numeric)
}

fn default_multi_range_details() -> String {
    "warning | outside |  | inclusive |  | inclusive".to_owned()
}

fn multi_range_details(value: &xtce::AlarmMultiRangesType) -> String {
    value
        .range
        .iter()
        .map(|range| {
            let level = range.level.as_ref().map(concern_text).unwrap_or("");
            let range_form = match range.range_form {
                xtce::RangeFormType::Outside => "outside",
                xtce::RangeFormType::Inside => "inside",
            };
            let float_range = xtce::FloatRangeType {
                min_inclusive: range.min_inclusive,
                min_exclusive: range.min_exclusive,
                max_inclusive: range.max_inclusive,
                max_exclusive: range.max_exclusive,
            };
            let (minimum, minimum_form) = range_minimum(&float_range);
            let (maximum, maximum_form) = range_maximum(&float_range);
            format!(
                "{level} | {range_form} | {minimum} | {minimum_form} | {maximum} | {maximum_form}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_multi_ranges(value: &str) -> Vec<xtce::MultiRangeType> {
    value
        .lines()
        .filter_map(|line| {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            if columns.len() != 6 {
                return None;
            }
            let minimum = columns[2].parse().ok();
            let maximum = columns[4].parse().ok();
            let minimum_exclusive = columns[3].eq_ignore_ascii_case("exclusive");
            let maximum_exclusive = columns[5].eq_ignore_ascii_case("exclusive");
            Some(xtce::MultiRangeType {
                min_inclusive: (!minimum_exclusive).then_some(minimum).flatten(),
                min_exclusive: minimum_exclusive.then_some(minimum).flatten(),
                max_inclusive: (!maximum_exclusive).then_some(maximum).flatten(),
                max_exclusive: maximum_exclusive.then_some(maximum).flatten(),
                range_form: if columns[1].eq_ignore_ascii_case("inside") {
                    xtce::RangeFormType::Inside
                } else {
                    xtce::RangeFormType::Outside
                },
                level: concern_from_text(columns[0]),
            })
        })
        .collect()
}

fn range_minimum(range: &xtce::FloatRangeType) -> (String, &'static str) {
    if let Some(value) = range.min_inclusive {
        (value.to_string(), "inclusive")
    } else if let Some(value) = range.min_exclusive {
        (value.to_string(), "exclusive")
    } else {
        (String::new(), "inclusive")
    }
}

fn range_maximum(range: &xtce::FloatRangeType) -> (String, &'static str) {
    if let Some(value) = range.max_inclusive {
        (value.to_string(), "inclusive")
    } else if let Some(value) = range.max_exclusive {
        (value.to_string(), "exclusive")
    } else {
        (String::new(), "inclusive")
    }
}

fn default_details(kind: AlarmKind) -> String {
    match kind {
        AlarmKind::Numeric | AlarmKind::Time => [
            "watch |  | inclusive |  | inclusive",
            "warning |  | inclusive |  | inclusive",
            "distress |  | inclusive |  | inclusive",
            "critical |  | inclusive |  | inclusive",
            "severe |  | inclusive |  | inclusive",
        ]
        .join("\n"),
        _ => String::new(),
    }
}

pub(super) fn parse_string_levels(value: &str) -> Vec<xtce::StringAlarmLevelType> {
    value
        .lines()
        .filter_map(|line| {
            let (level, pattern) = line.split_once('|')?;
            Some(xtce::StringAlarmLevelType {
                alarm_level: concern_from_text(level.trim())?,
                match_pattern: pattern.trim().to_owned(),
            })
        })
        .collect()
}

pub(super) fn parse_enumeration_levels(value: &str) -> Vec<xtce::EnumerationAlarmLevelType> {
    value
        .lines()
        .filter_map(|line| {
            let (level, label) = line.split_once('|')?;
            Some(xtce::EnumerationAlarmLevelType {
                alarm_level: concern_from_text(level.trim())?,
                enumeration_label: label.trim().to_owned(),
            })
        })
        .collect()
}

pub(super) fn parse_alarm_ranges(
    value: &str,
) -> Vec<(xtce::ConcernLevelsType, xtce::FloatRangeType)> {
    value
        .lines()
        .filter_map(|line| {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            if columns.len() != 5 {
                return None;
            }
            let level = concern_from_text(columns[0])?;
            let minimum = columns[1].parse().ok();
            let maximum = columns[3].parse().ok();
            if minimum.is_none() && maximum.is_none() {
                return None;
            }
            let minimum_exclusive = columns[2].eq_ignore_ascii_case("exclusive");
            let maximum_exclusive = columns[4].eq_ignore_ascii_case("exclusive");
            Some((
                level,
                xtce::FloatRangeType {
                    min_inclusive: (!minimum_exclusive).then_some(minimum).flatten(),
                    min_exclusive: minimum_exclusive.then_some(minimum).flatten(),
                    max_inclusive: (!maximum_exclusive).then_some(maximum).flatten(),
                    max_exclusive: maximum_exclusive.then_some(maximum).flatten(),
                },
            ))
        })
        .collect()
}

macro_rules! apply_common {
    ($alarm:expr, $edit:expr) => {{
        $alarm.name = optional_text($edit.name.clone());
        $alarm.short_description = optional_text($edit.short_description.clone());
        $alarm.min_violations = $edit.min_violations;
        $alarm.min_conformance = $edit.min_conformance;
        $alarm.disabled = $edit.disabled;
    }};
}

macro_rules! apply_ancillary {
    ($content:expr, $content_type:ident, $ancillary:expr) => {{
        let current = $content
            .iter()
            .position(|item| matches!(item, xtce::$content_type::AncillaryDataSet(_)));
        match (current, $ancillary) {
            (Some(index), Some(value)) => {
                $content[index] = xtce::$content_type::AncillaryDataSet(value);
            }
            (None, Some(value)) => {
                $content.insert(0, xtce::$content_type::AncillaryDataSet(value));
            }
            (Some(index), None) => {
                $content.remove(index);
            }
            (None, None) => {}
        }
    }};
}

macro_rules! upsert_content {
    ($content:expr, $content_type:ident, $variant:ident, $value:expr) => {{
        let current = $content
            .iter()
            .position(|item| matches!(item, xtce::$content_type::$variant(_)));
        match (current, $value) {
            (Some(index), Some(value)) => {
                $content[index] = xtce::$content_type::$variant(value);
            }
            (None, Some(value)) => {
                $content.push(xtce::$content_type::$variant(value));
            }
            (Some(index), None) => {
                $content.remove(index);
            }
            (None, None) => {}
        }
    }};
}

macro_rules! apply_shared_alarm_content {
    ($alarm:expr, $content_type:ident, $edit:expr) => {{
        $alarm.content.retain(|item| {
            !matches!(
                item,
                xtce::$content_type::AlarmConditions(_) | xtce::$content_type::CustomAlarm(_)
            )
        });
        let choice = if let Some(value) = $edit.custom_alarm {
            Some(xtce::$content_type::CustomAlarm(value))
        } else {
            $edit
                .alarm_conditions
                .map(xtce::$content_type::AlarmConditions)
        };
        if let Some(choice) = choice {
            let index = usize::from(matches!(
                $alarm.content.first(),
                Some(xtce::$content_type::AncillaryDataSet(_))
            ));
            $alarm.content.insert(index, choice);
        }
    }};
}

pub(super) fn apply_string_alarm(alarm: &mut xtce::StringAlarmType, edit: AlarmEdit) {
    apply_common!(alarm, edit);
    alarm.default_alarm_level = edit.default_alarm_level;
    apply_ancillary!(
        alarm.content,
        StringAlarmTypeContent,
        edit.ancillary_data_set
    );
    let rows = parse_string_levels(&edit.details);
    upsert_content!(
        alarm.content,
        StringAlarmTypeContent,
        StringAlarmList,
        (!rows.is_empty()).then_some(xtce::StringAlarmListType { string_alarm: rows })
    );
    apply_shared_alarm_content!(alarm, StringAlarmTypeContent, edit);
}

pub(super) fn apply_enumeration_alarm(alarm: &mut xtce::EnumerationAlarmType, edit: AlarmEdit) {
    apply_common!(alarm, edit);
    alarm.default_alarm_level = edit.default_alarm_level;
    apply_ancillary!(
        alarm.content,
        EnumerationAlarmTypeContent,
        edit.ancillary_data_set
    );
    let rows = parse_enumeration_levels(&edit.details);
    upsert_content!(
        alarm.content,
        EnumerationAlarmTypeContent,
        EnumerationAlarmList,
        (!rows.is_empty()).then_some(xtce::EnumerationAlarmListType {
            enumeration_alarm: rows,
        })
    );
    apply_shared_alarm_content!(alarm, EnumerationAlarmTypeContent, edit);
}

pub(super) fn apply_numeric_alarm(alarm: &mut xtce::NumericAlarmType, mut edit: AlarmEdit) {
    apply_common!(alarm, edit);
    let ranges = static_ranges(&mut edit);
    apply_ancillary!(
        alarm.content,
        NumericAlarmTypeContent,
        edit.ancillary_data_set
    );
    upsert_content!(
        alarm.content,
        NumericAlarmTypeContent,
        StaticAlarmRanges,
        ranges
    );
    upsert_content!(
        alarm.content,
        NumericAlarmTypeContent,
        ChangeAlarmRanges,
        edit.change_alarm_ranges
    );
    upsert_content!(
        alarm.content,
        NumericAlarmTypeContent,
        AlarmMultiRanges,
        edit.alarm_multi_ranges
    );
    apply_shared_alarm_content!(alarm, NumericAlarmTypeContent, edit);
}

pub(super) fn apply_binary_alarm(alarm: &mut xtce::BinaryAlarmType, edit: AlarmEdit) {
    apply_common!(alarm, edit);
    apply_ancillary!(
        alarm.content,
        BinaryAlarmTypeContent,
        edit.ancillary_data_set
    );
    apply_shared_alarm_content!(alarm, BinaryAlarmTypeContent, edit);
}

pub(super) fn apply_boolean_alarm(alarm: &mut xtce::BooleanAlarmType, edit: AlarmEdit) {
    apply_common!(alarm, edit);
    apply_ancillary!(
        alarm.content,
        BooleanAlarmTypeContent,
        edit.ancillary_data_set
    );
    apply_shared_alarm_content!(alarm, BooleanAlarmTypeContent, edit);
}

pub(super) fn apply_time_alarm(alarm: &mut xtce::TimeAlarmType, mut edit: AlarmEdit) {
    apply_common!(alarm, edit);
    let ranges = time_static_ranges(&mut edit);
    apply_ancillary!(alarm.content, TimeAlarmTypeContent, edit.ancillary_data_set);
    upsert_content!(
        alarm.content,
        TimeAlarmTypeContent,
        StaticAlarmRanges,
        ranges
    );
    upsert_content!(
        alarm.content,
        TimeAlarmTypeContent,
        ChangePerSecondAlarmRanges,
        edit.change_per_second_alarm_ranges
    );
    apply_shared_alarm_content!(alarm, TimeAlarmTypeContent, edit);
}

pub(super) fn apply_string_context_alarm(
    alarm: &mut xtce::StringContextAlarmType,
    edit: ContextAlarmEdit,
) {
    let ContextAlarmEdit {
        context_match,
        alarm: edit,
    } = edit;
    alarm
        .content
        .retain(|item| !matches!(item, xtce::StringContextAlarmTypeContent::ContextMatch(_)));
    apply_common!(alarm, edit);
    alarm.default_alarm_level = edit.default_alarm_level;
    apply_ancillary!(
        alarm.content,
        StringContextAlarmTypeContent,
        edit.ancillary_data_set
    );
    let rows = parse_string_levels(&edit.details);
    upsert_content!(
        alarm.content,
        StringContextAlarmTypeContent,
        StringAlarmList,
        (!rows.is_empty()).then_some(xtce::StringAlarmListType { string_alarm: rows })
    );
    apply_shared_alarm_content!(alarm, StringContextAlarmTypeContent, edit);
    upsert_content!(
        alarm.content,
        StringContextAlarmTypeContent,
        ContextMatch,
        Some(context_match)
    );
}

pub(super) fn apply_enumeration_context_alarm(
    alarm: &mut xtce::EnumerationContextAlarmType,
    edit: ContextAlarmEdit,
) {
    let ContextAlarmEdit {
        context_match,
        alarm: edit,
    } = edit;
    alarm.content.retain(|item| {
        !matches!(
            item,
            xtce::EnumerationContextAlarmTypeContent::ContextMatch(_)
        )
    });
    apply_common!(alarm, edit);
    alarm.default_alarm_level = edit.default_alarm_level;
    apply_ancillary!(
        alarm.content,
        EnumerationContextAlarmTypeContent,
        edit.ancillary_data_set
    );
    let rows = parse_enumeration_levels(&edit.details);
    upsert_content!(
        alarm.content,
        EnumerationContextAlarmTypeContent,
        EnumerationAlarmList,
        (!rows.is_empty()).then_some(xtce::EnumerationAlarmListType {
            enumeration_alarm: rows,
        })
    );
    apply_shared_alarm_content!(alarm, EnumerationContextAlarmTypeContent, edit);
    upsert_content!(
        alarm.content,
        EnumerationContextAlarmTypeContent,
        ContextMatch,
        Some(context_match)
    );
}

pub(super) fn apply_numeric_context_alarm(
    alarm: &mut xtce::NumericContextAlarmType,
    edit: ContextAlarmEdit,
) {
    let ContextAlarmEdit {
        context_match,
        alarm: mut edit,
    } = edit;
    alarm
        .content
        .retain(|item| !matches!(item, xtce::NumericContextAlarmTypeContent::ContextMatch(_)));
    apply_common!(alarm, edit);
    let ranges = static_ranges(&mut edit);
    apply_ancillary!(
        alarm.content,
        NumericContextAlarmTypeContent,
        edit.ancillary_data_set
    );
    upsert_content!(
        alarm.content,
        NumericContextAlarmTypeContent,
        StaticAlarmRanges,
        ranges
    );
    upsert_content!(
        alarm.content,
        NumericContextAlarmTypeContent,
        ChangeAlarmRanges,
        edit.change_alarm_ranges
    );
    upsert_content!(
        alarm.content,
        NumericContextAlarmTypeContent,
        AlarmMultiRanges,
        edit.alarm_multi_ranges
    );
    apply_shared_alarm_content!(alarm, NumericContextAlarmTypeContent, edit);
    upsert_content!(
        alarm.content,
        NumericContextAlarmTypeContent,
        ContextMatch,
        Some(context_match)
    );
}

pub(super) fn apply_binary_context_alarm(
    alarm: &mut xtce::BinaryContextAlarmType,
    edit: ContextAlarmEdit,
) {
    let ContextAlarmEdit {
        context_match,
        alarm: edit,
    } = edit;
    alarm
        .content
        .retain(|item| !matches!(item, xtce::BinaryContextAlarmTypeContent::ContextMatch(_)));
    apply_common!(alarm, edit);
    apply_ancillary!(
        alarm.content,
        BinaryContextAlarmTypeContent,
        edit.ancillary_data_set
    );
    apply_shared_alarm_content!(alarm, BinaryContextAlarmTypeContent, edit);
    upsert_content!(
        alarm.content,
        BinaryContextAlarmTypeContent,
        ContextMatch,
        Some(context_match)
    );
}

pub(super) fn apply_boolean_context_alarm(
    alarm: &mut xtce::BooleanContextAlarmType,
    edit: ContextAlarmEdit,
) {
    let ContextAlarmEdit {
        context_match,
        alarm: edit,
    } = edit;
    alarm
        .content
        .retain(|item| !matches!(item, xtce::BooleanContextAlarmTypeContent::ContextMatch(_)));
    apply_common!(alarm, edit);
    apply_ancillary!(
        alarm.content,
        BooleanContextAlarmTypeContent,
        edit.ancillary_data_set
    );
    apply_shared_alarm_content!(alarm, BooleanContextAlarmTypeContent, edit);
    upsert_content!(
        alarm.content,
        BooleanContextAlarmTypeContent,
        ContextMatch,
        Some(context_match)
    );
}

pub(super) fn apply_time_context_alarm(
    alarm: &mut xtce::TimeContextAlarmType,
    edit: ContextAlarmEdit,
) {
    let ContextAlarmEdit {
        context_match,
        alarm: mut edit,
    } = edit;
    alarm
        .content
        .retain(|item| !matches!(item, xtce::TimeContextAlarmTypeContent::ContextMatch(_)));
    apply_common!(alarm, edit);
    let ranges = time_static_ranges(&mut edit);
    apply_ancillary!(
        alarm.content,
        TimeContextAlarmTypeContent,
        edit.ancillary_data_set
    );
    upsert_content!(
        alarm.content,
        TimeContextAlarmTypeContent,
        StaticAlarmRanges,
        ranges
    );
    upsert_content!(
        alarm.content,
        TimeContextAlarmTypeContent,
        ChangePerSecondAlarmRanges,
        edit.change_per_second_alarm_ranges
    );
    apply_shared_alarm_content!(alarm, TimeContextAlarmTypeContent, edit);
    upsert_content!(
        alarm.content,
        TimeContextAlarmTypeContent,
        ContextMatch,
        Some(context_match)
    );
}

pub(super) fn default_string_alarm() -> xtce::StringAlarmType {
    xtce::StringAlarmType {
        name: None,
        short_description: None,
        min_violations: xtce::StringAlarmType::default_min_violations(),
        min_conformance: xtce::StringAlarmType::default_min_conformance(),
        disabled: xtce::StringAlarmType::default_disabled(),
        default_alarm_level: xtce::StringAlarmType::default_default_alarm_level(),
        content: Vec::new(),
    }
}

pub(super) fn default_enumeration_alarm() -> xtce::EnumerationAlarmType {
    xtce::EnumerationAlarmType {
        name: None,
        short_description: None,
        min_violations: xtce::EnumerationAlarmType::default_min_violations(),
        min_conformance: xtce::EnumerationAlarmType::default_min_conformance(),
        disabled: xtce::EnumerationAlarmType::default_disabled(),
        default_alarm_level: xtce::EnumerationAlarmType::default_default_alarm_level(),
        content: Vec::new(),
    }
}

pub(super) fn default_numeric_alarm() -> xtce::NumericAlarmType {
    xtce::NumericAlarmType {
        name: None,
        short_description: None,
        min_violations: xtce::NumericAlarmType::default_min_violations(),
        min_conformance: xtce::NumericAlarmType::default_min_conformance(),
        disabled: xtce::NumericAlarmType::default_disabled(),
        content: Vec::new(),
    }
}

pub(super) fn default_binary_alarm() -> xtce::BinaryAlarmType {
    xtce::BinaryAlarmType {
        name: None,
        short_description: None,
        min_violations: xtce::BinaryAlarmType::default_min_violations(),
        min_conformance: xtce::BinaryAlarmType::default_min_conformance(),
        disabled: xtce::BinaryAlarmType::default_disabled(),
        content: Vec::new(),
    }
}

pub(super) fn default_boolean_alarm() -> xtce::BooleanAlarmType {
    xtce::BooleanAlarmType {
        name: None,
        short_description: None,
        min_violations: xtce::BooleanAlarmType::default_min_violations(),
        min_conformance: xtce::BooleanAlarmType::default_min_conformance(),
        disabled: xtce::BooleanAlarmType::default_disabled(),
        content: Vec::new(),
    }
}

pub(super) fn default_time_alarm() -> xtce::TimeAlarmType {
    xtce::TimeAlarmType {
        name: None,
        short_description: None,
        min_violations: xtce::TimeAlarmType::default_min_violations(),
        min_conformance: xtce::TimeAlarmType::default_min_conformance(),
        disabled: xtce::TimeAlarmType::default_disabled(),
        content: Vec::new(),
    }
}

fn static_ranges(edit: &mut AlarmEdit) -> Option<xtce::AlarmRangesType> {
    let values = parse_alarm_ranges(&edit.details);
    edit.static_ranges_active.then(|| {
        let (watch_range, warning_range, distress_range, critical_range, severe_range) =
            split_ranges(values);
        xtce::AlarmRangesType {
            name: optional_text(std::mem::take(&mut edit.static_range_name)),
            short_description: optional_text(std::mem::take(
                &mut edit.static_range_short_description,
            )),
            range_form: range_form_copy(&edit.range_form),
            ancillary_data_set: edit.static_range_ancillary_data_set.take(),
            watch_range,
            warning_range,
            distress_range,
            critical_range,
            severe_range,
        }
    })
}

fn time_static_ranges(edit: &mut AlarmEdit) -> Option<xtce::TimeAlarmRangesType> {
    let values = parse_alarm_ranges(&edit.details);
    edit.static_ranges_active.then(|| {
        let (watch_range, warning_range, distress_range, critical_range, severe_range) =
            split_ranges(values);
        xtce::TimeAlarmRangesType {
            name: optional_text(std::mem::take(&mut edit.static_range_name)),
            short_description: optional_text(std::mem::take(
                &mut edit.static_range_short_description,
            )),
            range_form: range_form_copy(&edit.range_form),
            time_units: time_units_copy(&edit.time_units),
            ancillary_data_set: edit.static_range_ancillary_data_set.take(),
            watch_range,
            warning_range,
            distress_range,
            critical_range,
            severe_range,
        }
    })
}

type SplitRanges = (
    Option<xtce::FloatRangeType>,
    Option<xtce::FloatRangeType>,
    Option<xtce::FloatRangeType>,
    Option<xtce::FloatRangeType>,
    Option<xtce::FloatRangeType>,
);

fn split_ranges(values: Vec<(xtce::ConcernLevelsType, xtce::FloatRangeType)>) -> SplitRanges {
    let mut ranges: SplitRanges = (None, None, None, None, None);
    for (level, range) in values {
        match level {
            xtce::ConcernLevelsType::Watch => ranges.0 = Some(range),
            xtce::ConcernLevelsType::Warning => ranges.1 = Some(range),
            xtce::ConcernLevelsType::Distress => ranges.2 = Some(range),
            xtce::ConcernLevelsType::Critical => ranges.3 = Some(range),
            xtce::ConcernLevelsType::Severe => ranges.4 = Some(range),
            xtce::ConcernLevelsType::Normal => {}
        }
    }
    ranges
}

fn optional_text(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn default_match_criteria() -> xtce::MatchCriteriaType {
    xtce::MatchCriteriaType::Comparison(xtce::ComparisonType {
        parameter_ref: String::new(),
        instance: xtce::ComparisonType::default_instance(),
        use_calibrated_value: xtce::ComparisonType::default_use_calibrated_value(),
        comparison_operator: "==".to_owned(),
        value: String::new(),
    })
}

fn range_form_copy(value: &xtce::RangeFormType) -> xtce::RangeFormType {
    match value {
        xtce::RangeFormType::Outside => xtce::RangeFormType::Outside,
        xtce::RangeFormType::Inside => xtce::RangeFormType::Inside,
    }
}

fn time_units_copy(value: &xtce::TimeUnitsType) -> xtce::TimeUnitsType {
    TimeUnitsChoice::from_xtce(value).to_xtce()
}

fn concern_text(value: &xtce::ConcernLevelsType) -> &'static str {
    match value {
        xtce::ConcernLevelsType::Normal => "normal",
        xtce::ConcernLevelsType::Watch => "watch",
        xtce::ConcernLevelsType::Warning => "warning",
        xtce::ConcernLevelsType::Distress => "distress",
        xtce::ConcernLevelsType::Critical => "critical",
        xtce::ConcernLevelsType::Severe => "severe",
    }
}

fn concern_from_text(value: &str) -> Option<xtce::ConcernLevelsType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(xtce::ConcernLevelsType::Normal),
        "watch" => Some(xtce::ConcernLevelsType::Watch),
        "warning" => Some(xtce::ConcernLevelsType::Warning),
        "distress" => Some(xtce::ConcernLevelsType::Distress),
        "critical" => Some(xtce::ConcernLevelsType::Critical),
        "severe" => Some(xtce::ConcernLevelsType::Severe),
        _ => None,
    }
}

impl ConcernLevelChoice {
    fn from_xtce(value: &xtce::ConcernLevelsType) -> Self {
        match value {
            xtce::ConcernLevelsType::Normal => Self::Normal,
            xtce::ConcernLevelsType::Watch => Self::Watch,
            xtce::ConcernLevelsType::Warning => Self::Warning,
            xtce::ConcernLevelsType::Distress => Self::Distress,
            xtce::ConcernLevelsType::Critical => Self::Critical,
            xtce::ConcernLevelsType::Severe => Self::Severe,
        }
    }

    fn to_xtce(self) -> xtce::ConcernLevelsType {
        concern_from_text(&self.to_string()).unwrap_or(xtce::ConcernLevelsType::Normal)
    }
}

impl RangeFormChoice {
    fn from_xtce(value: &xtce::RangeFormType) -> Self {
        match value {
            xtce::RangeFormType::Outside => Self::Outside,
            xtce::RangeFormType::Inside => Self::Inside,
        }
    }

    fn to_xtce(self) -> xtce::RangeFormType {
        match self {
            Self::Outside => xtce::RangeFormType::Outside,
            Self::Inside => xtce::RangeFormType::Inside,
        }
    }
}

impl BoundaryChoice {
    fn from_text(value: &str) -> Self {
        if value.eq_ignore_ascii_case("exclusive") {
            Self::Exclusive
        } else {
            Self::Inclusive
        }
    }
}

impl TimeUnitsChoice {
    fn from_xtce(value: &xtce::TimeUnitsType) -> Self {
        match value {
            xtce::TimeUnitsType::Seconds => Self::Seconds,
            xtce::TimeUnitsType::Milliseconds => Self::Milliseconds,
            xtce::TimeUnitsType::Microseconds => Self::Microseconds,
            xtce::TimeUnitsType::Nanoseconds => Self::Nanoseconds,
            xtce::TimeUnitsType::Picoseconds => Self::Picoseconds,
            xtce::TimeUnitsType::Minutes => Self::Minutes,
            xtce::TimeUnitsType::Hours => Self::Hours,
            xtce::TimeUnitsType::Days => Self::Days,
            xtce::TimeUnitsType::Months => Self::Months,
            xtce::TimeUnitsType::Years => Self::Years,
        }
    }

    fn to_xtce(self) -> xtce::TimeUnitsType {
        match self {
            Self::Seconds => xtce::TimeUnitsType::Seconds,
            Self::Milliseconds => xtce::TimeUnitsType::Milliseconds,
            Self::Microseconds => xtce::TimeUnitsType::Microseconds,
            Self::Nanoseconds => xtce::TimeUnitsType::Nanoseconds,
            Self::Picoseconds => xtce::TimeUnitsType::Picoseconds,
            Self::Minutes => xtce::TimeUnitsType::Minutes,
            Self::Hours => xtce::TimeUnitsType::Hours,
            Self::Days => xtce::TimeUnitsType::Days,
            Self::Months => xtce::TimeUnitsType::Months,
            Self::Years => xtce::TimeUnitsType::Years,
        }
    }
}

impl ChangeSpanChoice {
    fn from_xtce(value: &xtce::ChangeSpanType) -> Self {
        match value {
            xtce::ChangeSpanType::ChangePerSecond => Self::PerSecond,
            xtce::ChangeSpanType::ChangePerSample => Self::PerSample,
        }
    }

    fn to_xtce(self) -> xtce::ChangeSpanType {
        match self {
            Self::PerSecond => xtce::ChangeSpanType::ChangePerSecond,
            Self::PerSample => xtce::ChangeSpanType::ChangePerSample,
        }
    }
}

impl ChangeBasisChoice {
    fn from_xtce(value: &xtce::ChangeBasisType) -> Self {
        match value {
            xtce::ChangeBasisType::AbsoluteChange => Self::Absolute,
            xtce::ChangeBasisType::PercentageChange => Self::Percentage,
        }
    }

    fn to_xtce(self) -> xtce::ChangeBasisType {
        match self {
            Self::Absolute => xtce::ChangeBasisType::AbsoluteChange,
            Self::Percentage => xtce::ChangeBasisType::PercentageChange,
        }
    }
}

fn bool_choice(value: bool) -> BooleanChoice {
    if value {
        BooleanChoice::True
    } else {
        BooleanChoice::False
    }
}

fn input(
    value: &str,
    multi_line: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let input = InputState::new(window, cx).default_value(value.to_owned());
        if multi_line {
            input.auto_grow(super::MULTILINE_MIN_ROWS, super::MULTILINE_MAX_ROWS)
        } else {
            input
        }
    })
}

fn select<T>(
    options: &'static [T],
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<T>>>
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    let index = options
        .iter()
        .position(|option| option == &selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            options.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn selected_value<T>(select: &Entity<SelectState<Vec<T>>>, fallback: T, cx: &App) -> T
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(fallback)
}

fn select_field<T>(label: &'static str, select: &Entity<SelectState<Vec<T>>>) -> Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    super::select_field(label, "", select)
}

fn range_details_field(details: &Entity<InputState>, cx: &App) -> Div {
    field(
        "Ranges",
        "One per line: level | minimum | inclusive/exclusive | maximum | inclusive/exclusive",
        details,
        cx,
    )
}

fn optional_section_header<T: 'static>(
    title: &'static str,
    id: &'static str,
    active: bool,
    cx: &mut Context<T>,
    active_field: impl Fn(&mut T) -> &mut bool + Copy + 'static,
) -> Div {
    h_flex()
        .justify_between()
        .child(div().text_sm().font_medium().child(title))
        .child(if active {
            super::section_remove_button(format!("remove-{id}")).on_click(cx.listener(
                move |this, _, _, cx| {
                    *active_field(this) = false;
                    cx.notify();
                },
            ))
        } else {
            super::section_add_button(format!("add-{id}")).on_click(cx.listener(
                move |this, _, _, cx| {
                    *active_field(this) = true;
                    cx.notify();
                },
            ))
        })
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn parse_or<T: std::str::FromStr>(value: &str, fallback: T) -> T {
    value.trim().parse().unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{
        AlarmEdit, apply_numeric_alarm, default_numeric_alarm, parse_alarm_ranges,
        parse_enumeration_levels, parse_multi_ranges, parse_string_levels,
    };

    #[test]
    fn alarm_detail_rows_parse_all_supported_shapes() {
        let strings = parse_string_levels("warning | ^WARN.*\ncritical | FAILED");
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0].match_pattern, "^WARN.*");

        let enumerations = parse_enumeration_levels("watch | STANDBY\nsevere | FAILED");
        assert_eq!(enumerations.len(), 2);
        assert_eq!(enumerations[1].enumeration_label, "FAILED");

        let ranges = parse_alarm_ranges(
            "warning | 0 | inclusive | 10 | exclusive\ncritical | -5 | exclusive | 20 | inclusive",
        );
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].1.min_inclusive, Some(0.0));
        assert_eq!(ranges[0].1.max_exclusive, Some(10.0));
    }

    #[test]
    fn numeric_alarm_writes_every_range_kind_in_schema_order() {
        let mut alarm = default_numeric_alarm();
        let edit = AlarmEdit {
            name: "TemperatureAlarm".to_owned(),
            short_description: String::new(),
            min_violations: 2,
            min_conformance: 3,
            disabled: false,
            default_alarm_level: xtce::ConcernLevelsType::Normal,
            range_form: xtce::RangeFormType::Outside,
            time_units: xtce::TimeUnitsType::Seconds,
            details: "warning | 0 | inclusive | 100 | inclusive".to_owned(),
            static_ranges_active: true,
            ancillary_data_set: None,
            alarm_conditions: Some(xtce::AlarmConditionsType {
                watch_alarm: None,
                warning_alarm: None,
                distress_alarm: None,
                critical_alarm: None,
                severe_alarm: None,
            }),
            custom_alarm: None,
            static_range_name: "Static".to_owned(),
            static_range_short_description: String::new(),
            static_range_ancillary_data_set: None,
            change_alarm_ranges: Some(xtce::ChangeAlarmRangesType {
                name: Some("Rate".to_owned()),
                short_description: None,
                range_form: xtce::RangeFormType::Outside,
                change_type: xtce::ChangeSpanType::ChangePerSecond,
                change_basis: xtce::ChangeBasisType::AbsoluteChange,
                span_of_interest_in_samples: 1,
                span_of_interest_in_seconds: 1.0,
                ancillary_data_set: None,
                watch_range: None,
                warning_range: None,
                distress_range: None,
                critical_range: None,
                severe_range: None,
            }),
            alarm_multi_ranges: Some(xtce::AlarmMultiRangesType {
                name: None,
                short_description: None,
                ancillary_data_set: None,
                range: parse_multi_ranges("critical | inside | 10 | inclusive | 20 | exclusive"),
            }),
            change_per_second_alarm_ranges: None,
        };

        apply_numeric_alarm(&mut alarm, edit);

        assert!(matches!(
            alarm.content.as_slice(),
            [
                xtce::NumericAlarmTypeContent::AlarmConditions(_),
                xtce::NumericAlarmTypeContent::StaticAlarmRanges(_),
                xtce::NumericAlarmTypeContent::ChangeAlarmRanges(_),
                xtce::NumericAlarmTypeContent::AlarmMultiRanges(_)
            ]
        ));
        let xtce::NumericAlarmTypeContent::StaticAlarmRanges(ranges) = &alarm.content[1] else {
            panic!("expected static alarm ranges");
        };
        assert_eq!(ranges.name.as_deref(), Some("Static"));
    }
}
