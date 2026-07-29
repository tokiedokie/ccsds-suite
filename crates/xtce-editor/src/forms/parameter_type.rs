use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    IconName, IndexPath, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    collapsible::Collapsible,
    h_flex,
    input::{Input, InputEvent, InputState},
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    aggregate_member_list::AggregateMemberListForm,
    alias_set::AliasSetForm,
    ancillary_data_set::AncillaryDataSetForm,
    array_dimension::DimensionListForm,
    data_encoding::{
        DataEncodingForm, find_data_encoding, find_data_encoding_mut, set_data_encoding_kind,
    },
    enumeration_list::EnumerationListForm,
    field, impl_select_item, optional_value,
    parameter_alarm::{
        AlarmKind, AlarmRef, ContextAlarmEdit, ContextAlarmListForm, ContextAlarmListRef,
        DefaultAlarmForm, apply_binary_alarm, apply_binary_context_alarm, apply_boolean_alarm,
        apply_boolean_context_alarm, apply_enumeration_alarm, apply_enumeration_context_alarm,
        apply_numeric_alarm, apply_numeric_context_alarm, apply_string_alarm,
        apply_string_context_alarm, apply_time_alarm, apply_time_context_alarm,
        default_binary_alarm, default_boolean_alarm, default_enumeration_alarm,
        default_numeric_alarm, default_string_alarm, default_time_alarm,
    },
    reference_time::ReferenceTimeForm,
    time_encoding::TimeEncodingForm,
    to_string::ToStringForm,
    unit_set::UnitSetForm,
    valid_range::{ValidRangeForm, ValidRangeKind, ValidRangeValue},
};
use crate::XtceEditor;

macro_rules! long_description {
    ($content:expr, $type:ident) => {
        $content
            .iter()
            .find_map(|item| match item {
                xtce::$type::LongDescription(value) => Some(value.as_str()),
                _ => None,
            })
            .unwrap_or_default()
    };
}

macro_rules! set_long_description {
    ($content:expr, $type:ident, $description:expr) => {{
        let existing = $content.iter_mut().find_map(|item| match item {
            xtce::$type::LongDescription(value) => Some(value),
            _ => None,
        });
        if let Some(existing) = existing {
            existing.clone_from($description);
        } else if !$description.is_empty() {
            $content.insert(0, xtce::$type::LongDescription($description.to_owned()));
        }
        if $description.is_empty() {
            $content.retain(|item| !matches!(item, xtce::$type::LongDescription(_)));
        }
    }};
}

fn parameter_type_alias_set(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::AliasSetType> {
    macro_rules! content_alias {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::AliasSet(value) => Some(value),
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            content_alias!(value, StringParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            content_alias!(value, EnumeratedParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            content_alias!(value, IntegerParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            content_alias!(value, BinaryParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            content_alias!(value, FloatParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            content_alias!(value, BooleanParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.alias_set.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.alias_set.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => value.alias_set.as_ref(),
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
            value.alias_set.as_ref()
        }
    }
}

fn set_parameter_type_alias_set(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    alias_set: Option<xtce::AliasSetType>,
) {
    macro_rules! set_content_alias {
        ($value:expr, $content:ident, $alias_set:expr) => {{
            let existing = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::AliasSet(_)));
            match ($alias_set, existing) {
                (Some(alias_set), Some(index)) => {
                    $value.content[index] = xtce::$content::AliasSet(alias_set);
                }
                (Some(alias_set), None) => {
                    let index = $value
                        .content
                        .iter()
                        .position(|item| !matches!(item, xtce::$content::LongDescription(_)))
                        .unwrap_or($value.content.len());
                    $value
                        .content
                        .insert(index, xtce::$content::AliasSet(alias_set));
                }
                (None, Some(index)) => {
                    $value.content.remove(index);
                }
                (None, None) => {}
            }
        }};
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            set_content_alias!(value, StringParameterTypeContent, alias_set)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            set_content_alias!(value, EnumeratedParameterTypeContent, alias_set)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            set_content_alias!(value, IntegerParameterTypeContent, alias_set)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            set_content_alias!(value, BinaryParameterTypeContent, alias_set)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            set_content_alias!(value, FloatParameterTypeContent, alias_set)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            set_content_alias!(value, BooleanParameterTypeContent, alias_set)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.alias_set = alias_set
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.alias_set = alias_set
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => value.alias_set = alias_set,
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
            value.alias_set = alias_set
        }
    }
}

fn parameter_type_ancillary_data_set(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::AncillaryDataSetType> {
    macro_rules! content_ancillary {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::AncillaryDataSet(value) => Some(value),
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            content_ancillary!(value, StringParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            content_ancillary!(value, EnumeratedParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            content_ancillary!(value, IntegerParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            content_ancillary!(value, BinaryParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            content_ancillary!(value, FloatParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            content_ancillary!(value, BooleanParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.ancillary_data_set.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.ancillary_data_set.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => {
            value.ancillary_data_set.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
            value.ancillary_data_set.as_ref()
        }
    }
}

fn set_parameter_type_ancillary_data_set(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    ancillary_data_set: Option<xtce::AncillaryDataSetType>,
) {
    macro_rules! set_content_ancillary {
        ($value:expr, $content:ident, $ancillary_data_set:expr) => {{
            let existing = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::AncillaryDataSet(_)));
            match ($ancillary_data_set, existing) {
                (Some(ancillary_data_set), Some(index)) => {
                    $value.content[index] = xtce::$content::AncillaryDataSet(ancillary_data_set);
                }
                (Some(ancillary_data_set), None) => {
                    let index = $value
                        .content
                        .iter()
                        .position(|item| {
                            !matches!(
                                item,
                                xtce::$content::LongDescription(_) | xtce::$content::AliasSet(_)
                            )
                        })
                        .unwrap_or($value.content.len());
                    $value
                        .content
                        .insert(index, xtce::$content::AncillaryDataSet(ancillary_data_set));
                }
                (None, Some(index)) => {
                    $value.content.remove(index);
                }
                (None, None) => {}
            }
        }};
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            set_content_ancillary!(value, StringParameterTypeContent, ancillary_data_set)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            set_content_ancillary!(value, EnumeratedParameterTypeContent, ancillary_data_set)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            set_content_ancillary!(value, IntegerParameterTypeContent, ancillary_data_set)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            set_content_ancillary!(value, BinaryParameterTypeContent, ancillary_data_set)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            set_content_ancillary!(value, FloatParameterTypeContent, ancillary_data_set)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            set_content_ancillary!(value, BooleanParameterTypeContent, ancillary_data_set)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.ancillary_data_set = ancillary_data_set
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.ancillary_data_set = ancillary_data_set
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => {
            value.ancillary_data_set = ancillary_data_set
        }
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
            value.ancillary_data_set = ancillary_data_set
        }
    }
}

fn parameter_type_unit_set(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::UnitSetType> {
    macro_rules! content_units {
        ($value:expr, $content:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::UnitSet(value) => Some(value),
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            content_units!(value, StringParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            content_units!(value, EnumeratedParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            content_units!(value, IntegerParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            content_units!(value, BinaryParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            content_units!(value, FloatParameterTypeContent)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            content_units!(value, BooleanParameterTypeContent)
        }
        _ => None,
    }
}

fn set_parameter_type_unit_set(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    unit_set: Option<xtce::UnitSetType>,
) {
    macro_rules! set_content_units {
        ($value:expr, $content:ident, $unit_set:expr) => {{
            let existing = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::UnitSet(_)));
            match ($unit_set, existing) {
                (Some(unit_set), Some(index)) => {
                    $value.content[index] = xtce::$content::UnitSet(unit_set);
                }
                (Some(unit_set), None) => {
                    let index = $value
                        .content
                        .iter()
                        .position(|item| {
                            !matches!(
                                item,
                                xtce::$content::LongDescription(_)
                                    | xtce::$content::AliasSet(_)
                                    | xtce::$content::AncillaryDataSet(_)
                            )
                        })
                        .unwrap_or($value.content.len());
                    $value
                        .content
                        .insert(index, xtce::$content::UnitSet(unit_set));
                }
                (None, Some(index)) => {
                    $value.content.remove(index);
                }
                (None, None) => {}
            }
        }};
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            set_content_units!(value, StringParameterTypeContent, unit_set)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            set_content_units!(value, EnumeratedParameterTypeContent, unit_set)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            set_content_units!(value, IntegerParameterTypeContent, unit_set)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            set_content_units!(value, BinaryParameterTypeContent, unit_set)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            set_content_units!(value, FloatParameterTypeContent, unit_set)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            set_content_units!(value, BooleanParameterTypeContent, unit_set)
        }
        _ => {}
    }
}

fn string_size_range(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::IntegerRangeType> {
    let xtce::ParameterTypeSetTypeContent::StringParameterType(value) = parameter_type else {
        return None;
    };
    value.content.iter().find_map(|item| match item {
        xtce::StringParameterTypeContent::SizeRangeInCharacters(value) => Some(value),
        _ => None,
    })
}

fn set_string_size_range(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    size_range: Option<xtce::IntegerRangeType>,
) {
    let xtce::ParameterTypeSetTypeContent::StringParameterType(value) = parameter_type else {
        return;
    };
    let existing = value.content.iter().position(|item| {
        matches!(
            item,
            xtce::StringParameterTypeContent::SizeRangeInCharacters(_)
        )
    });
    match (size_range, existing) {
        (Some(size_range), Some(index)) => {
            value.content[index] =
                xtce::StringParameterTypeContent::SizeRangeInCharacters(size_range);
        }
        (Some(size_range), None) => {
            let index = value
                .content
                .iter()
                .position(|item| {
                    matches!(
                        item,
                        xtce::StringParameterTypeContent::DefaultAlarm(_)
                            | xtce::StringParameterTypeContent::ContextAlarmList(_)
                    )
                })
                .unwrap_or(value.content.len());
            value.content.insert(
                index,
                xtce::StringParameterTypeContent::SizeRangeInCharacters(size_range),
            );
        }
        (None, Some(index)) => {
            value.content.remove(index);
        }
        (None, None) => {}
    }
}

fn parameter_type_to_string(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::ToStringType> {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            value.content.iter().find_map(|item| match item {
                xtce::IntegerParameterTypeContent::ToString(value) => Some(value),
                _ => None,
            })
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            value.content.iter().find_map(|item| match item {
                xtce::FloatParameterTypeContent::ToString(value) => Some(value),
                _ => None,
            })
        }
        _ => None,
    }
}

fn set_parameter_type_to_string(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    to_string: Option<xtce::ToStringType>,
) {
    macro_rules! set_to_string {
        ($value:expr, $content:ident, $to_string:expr) => {{
            let existing = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::ToString(_)));
            match ($to_string, existing) {
                (Some(to_string), Some(index)) => {
                    $value.content[index] = xtce::$content::ToString(to_string);
                }
                (Some(to_string), None) => {
                    let index = $value
                        .content
                        .iter()
                        .position(|item| {
                            matches!(
                                item,
                                xtce::$content::ValidRange(_)
                                    | xtce::$content::DefaultAlarm(_)
                                    | xtce::$content::ContextAlarmList(_)
                            )
                        })
                        .unwrap_or($value.content.len());
                    $value
                        .content
                        .insert(index, xtce::$content::ToString(to_string));
                }
                (None, Some(index)) => {
                    $value.content.remove(index);
                }
                (None, None) => {}
            }
        }};
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            set_to_string!(value, IntegerParameterTypeContent, to_string)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            set_to_string!(value, FloatParameterTypeContent, to_string)
        }
        _ => {}
    }
}

fn integer_valid_range(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::IntegerDataTypeValidRangeElementType> {
    let xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) = parameter_type else {
        return None;
    };
    value.content.iter().find_map(|item| match item {
        xtce::IntegerParameterTypeContent::ValidRange(value) => Some(value),
        _ => None,
    })
}

fn float_valid_range(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::FloatDataTypeValidRangeElementType> {
    let xtce::ParameterTypeSetTypeContent::FloatParameterType(value) = parameter_type else {
        return None;
    };
    value.content.iter().find_map(|item| match item {
        xtce::FloatParameterTypeContent::ValidRange(value) => Some(value),
        _ => None,
    })
}

fn set_parameter_type_valid_range(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    valid_range: Option<ValidRangeValue>,
) {
    macro_rules! set_valid_range {
        ($value:expr, $content:ident, $range:expr) => {{
            let existing = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::ValidRange(_)));
            match ($range, existing) {
                (Some(range), Some(index)) => {
                    $value.content[index] = xtce::$content::ValidRange(range);
                }
                (Some(range), None) => {
                    let index = $value
                        .content
                        .iter()
                        .position(|item| {
                            matches!(
                                item,
                                xtce::$content::DefaultAlarm(_)
                                    | xtce::$content::ContextAlarmList(_)
                            )
                        })
                        .unwrap_or($value.content.len());
                    $value
                        .content
                        .insert(index, xtce::$content::ValidRange(range));
                }
                (None, Some(index)) => {
                    $value.content.remove(index);
                }
                (None, None) => {}
            }
        }};
    }
    match (parameter_type, valid_range) {
        (
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value),
            Some(ValidRangeValue::Integer(range)),
        ) => set_valid_range!(value, IntegerParameterTypeContent, Some(range)),
        (
            xtce::ParameterTypeSetTypeContent::FloatParameterType(value),
            Some(ValidRangeValue::Float(range)),
        ) => set_valid_range!(value, FloatParameterTypeContent, Some(range)),
        (xtce::ParameterTypeSetTypeContent::IntegerParameterType(value), None) => {
            set_valid_range!(
                value,
                IntegerParameterTypeContent,
                None::<xtce::IntegerDataTypeValidRangeElementType>
            )
        }
        (xtce::ParameterTypeSetTypeContent::FloatParameterType(value), None) => {
            set_valid_range!(
                value,
                FloatParameterTypeContent,
                None::<xtce::FloatDataTypeValidRangeElementType>
            )
        }
        _ => {}
    }
}

fn valid_range_kind(kind: ParameterTypeKind) -> ValidRangeKind {
    match kind {
        ParameterTypeKind::Integer => ValidRangeKind::Integer,
        ParameterTypeKind::Float => ValidRangeKind::Float,
        _ => ValidRangeKind::Unsupported,
    }
}

fn parameter_type_time_encoding(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::EncodingType> {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.encoding.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.encoding.as_ref()
        }
        _ => None,
    }
}

fn set_parameter_type_time_encoding(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    encoding: Option<xtce::EncodingType>,
) {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.encoding = encoding;
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.encoding = encoding;
        }
        _ => {}
    }
}

fn parameter_type_reference_time(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<&xtce::ReferenceTimeType> {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.reference_time.as_ref()
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.reference_time.as_ref()
        }
        _ => None,
    }
}

fn set_parameter_type_reference_time(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    reference_time: Option<xtce::ReferenceTimeType>,
) {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.reference_time = reference_time;
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            value.reference_time = reference_time;
        }
        _ => {}
    }
}

fn parameter_alarm_kind(kind: ParameterTypeKind) -> AlarmKind {
    match kind {
        ParameterTypeKind::String => AlarmKind::String,
        ParameterTypeKind::Enumerated => AlarmKind::Enumerated,
        ParameterTypeKind::Integer | ParameterTypeKind::Float => AlarmKind::Numeric,
        ParameterTypeKind::Binary => AlarmKind::Binary,
        ParameterTypeKind::Boolean => AlarmKind::Boolean,
        ParameterTypeKind::RelativeTime => AlarmKind::Time,
        ParameterTypeKind::AbsoluteTime
        | ParameterTypeKind::Array
        | ParameterTypeKind::Aggregate => AlarmKind::Unsupported,
    }
}

fn parameter_default_alarm(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<AlarmRef<'_>> {
    macro_rules! find {
        ($value:expr, $content:ident, $variant:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::DefaultAlarm(value) => Some(AlarmRef::$variant(value)),
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            find!(value, StringParameterTypeContent, String)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            find!(value, EnumeratedParameterTypeContent, Enumerated)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            find!(value, IntegerParameterTypeContent, Numeric)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            find!(value, BinaryParameterTypeContent, Binary)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            find!(value, FloatParameterTypeContent, Numeric)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            find!(value, BooleanParameterTypeContent, Boolean)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            value.default_alarm.as_ref().map(AlarmRef::Time)
        }
        _ => None,
    }
}

fn parameter_context_alarm_list(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Option<ContextAlarmListRef<'_>> {
    macro_rules! find {
        ($value:expr, $content:ident, $variant:ident) => {
            $value.content.iter().find_map(|item| match item {
                xtce::$content::ContextAlarmList(value) => {
                    Some(ContextAlarmListRef::$variant(value))
                }
                _ => None,
            })
        };
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            find!(value, StringParameterTypeContent, String)
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            find!(value, EnumeratedParameterTypeContent, Enumerated)
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            find!(value, IntegerParameterTypeContent, Numeric)
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            find!(value, BinaryParameterTypeContent, Binary)
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            find!(value, FloatParameterTypeContent, Numeric)
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            find!(value, BooleanParameterTypeContent, Boolean)
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => value
            .context_alarm_list
            .as_ref()
            .map(ContextAlarmListRef::Time),
        _ => None,
    }
}

fn set_parameter_default_alarm(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    edit: Option<super::parameter_alarm::AlarmEdit>,
) {
    macro_rules! set_alarm {
        ($value:expr, $content:ident, $edit:expr, $default:path, $apply:path) => {{
            let current = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::DefaultAlarm(_)));
            match (current, $edit) {
                (Some(index), Some(edit)) => {
                    let xtce::$content::DefaultAlarm(alarm) = &mut $value.content[index] else {
                        unreachable!()
                    };
                    $apply(alarm, edit);
                }
                (None, Some(edit)) => {
                    let mut alarm = $default();
                    $apply(&mut alarm, edit);
                    let index = $value
                        .content
                        .iter()
                        .position(|item| matches!(item, xtce::$content::ContextAlarmList(_)))
                        .unwrap_or($value.content.len());
                    $value
                        .content
                        .insert(index, xtce::$content::DefaultAlarm(alarm));
                }
                (Some(index), None) => {
                    $value.content.remove(index);
                }
                (None, None) => {}
            }
        }};
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => set_alarm!(
            value,
            StringParameterTypeContent,
            edit,
            default_string_alarm,
            apply_string_alarm
        ),
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => set_alarm!(
            value,
            EnumeratedParameterTypeContent,
            edit,
            default_enumeration_alarm,
            apply_enumeration_alarm
        ),
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => set_alarm!(
            value,
            IntegerParameterTypeContent,
            edit,
            default_numeric_alarm,
            apply_numeric_alarm
        ),
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => set_alarm!(
            value,
            BinaryParameterTypeContent,
            edit,
            default_binary_alarm,
            apply_binary_alarm
        ),
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => set_alarm!(
            value,
            FloatParameterTypeContent,
            edit,
            default_numeric_alarm,
            apply_numeric_alarm
        ),
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => set_alarm!(
            value,
            BooleanParameterTypeContent,
            edit,
            default_boolean_alarm,
            apply_boolean_alarm
        ),
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => match edit {
            Some(edit) => {
                let alarm = value.default_alarm.get_or_insert_with(default_time_alarm);
                apply_time_alarm(alarm, edit);
            }
            None => value.default_alarm = None,
        },
        _ => {}
    }
}

fn set_parameter_context_alarm_list(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    edits: Vec<ContextAlarmEdit>,
) {
    macro_rules! set_list {
        (
            $value:expr,
            $content:ident,
            $list_type:ident,
            $alarm_type:ident,
            $default:expr,
            $apply:path
        ) => {{
            let current = $value
                .content
                .iter()
                .position(|item| matches!(item, xtce::$content::ContextAlarmList(_)));
            let mut existing = current
                .map(|index| $value.content.remove(index))
                .and_then(|item| match item {
                    xtce::$content::ContextAlarmList(value) => {
                        Some(value.context_alarm.into_iter())
                    }
                    _ => None,
                })
                .into_iter()
                .flatten();
            let rows = edits
                .into_iter()
                .map(|edit| {
                    let mut alarm = existing.next().unwrap_or_else(|| $default);
                    $apply(&mut alarm, edit);
                    alarm
                })
                .collect::<Vec<xtce::$alarm_type>>();
            if !rows.is_empty() {
                $value
                    .content
                    .push(xtce::$content::ContextAlarmList(xtce::$list_type {
                        context_alarm: rows,
                    }));
            }
        }};
    }
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => set_list!(
            value,
            StringParameterTypeContent,
            StringContextAlarmListType,
            StringContextAlarmType,
            default_string_context_alarm(),
            apply_string_context_alarm
        ),
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => set_list!(
            value,
            EnumeratedParameterTypeContent,
            EnumerationContextAlarmListType,
            EnumerationContextAlarmType,
            default_enumeration_context_alarm(),
            apply_enumeration_context_alarm
        ),
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => set_list!(
            value,
            IntegerParameterTypeContent,
            NumericContextAlarmListType,
            NumericContextAlarmType,
            default_numeric_context_alarm(),
            apply_numeric_context_alarm
        ),
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => set_list!(
            value,
            BinaryParameterTypeContent,
            BinaryContextAlarmListType,
            BinaryContextAlarmType,
            default_binary_context_alarm(),
            apply_binary_context_alarm
        ),
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => set_list!(
            value,
            FloatParameterTypeContent,
            NumericContextAlarmListType,
            NumericContextAlarmType,
            default_numeric_context_alarm(),
            apply_numeric_context_alarm
        ),
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => set_list!(
            value,
            BooleanParameterTypeContent,
            BooleanContextAlarmListType,
            BooleanContextAlarmType,
            default_boolean_context_alarm(),
            apply_boolean_context_alarm
        ),
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            let mut existing = value
                .context_alarm_list
                .take()
                .into_iter()
                .flat_map(|list| list.context_alarm)
                .into_iter();
            let rows = edits
                .into_iter()
                .map(|edit| {
                    let mut alarm = existing.next().unwrap_or_else(default_time_context_alarm);
                    apply_time_context_alarm(&mut alarm, edit);
                    alarm
                })
                .collect::<Vec<_>>();
            value.context_alarm_list =
                (!rows.is_empty()).then_some(xtce::TimeContextAlarmListType {
                    context_alarm: rows,
                });
        }
        _ => {}
    }
}

macro_rules! default_context_alarm {
    ($name:ident, $type:ident, $level:expr) => {
        fn $name() -> xtce::$type {
            xtce::$type {
                name: None,
                short_description: None,
                min_violations: xtce::$type::default_min_violations(),
                min_conformance: xtce::$type::default_min_conformance(),
                disabled: xtce::$type::default_disabled(),
                default_alarm_level: $level,
                content: Vec::new(),
            }
        }
    };
    ($name:ident, $type:ident) => {
        fn $name() -> xtce::$type {
            xtce::$type {
                name: None,
                short_description: None,
                min_violations: xtce::$type::default_min_violations(),
                min_conformance: xtce::$type::default_min_conformance(),
                disabled: xtce::$type::default_disabled(),
                content: Vec::new(),
            }
        }
    };
}

default_context_alarm!(
    default_string_context_alarm,
    StringContextAlarmType,
    xtce::StringContextAlarmType::default_default_alarm_level()
);
default_context_alarm!(
    default_enumeration_context_alarm,
    EnumerationContextAlarmType,
    xtce::EnumerationContextAlarmType::default_default_alarm_level()
);
default_context_alarm!(default_numeric_context_alarm, NumericContextAlarmType);
default_context_alarm!(default_binary_context_alarm, BinaryContextAlarmType);
default_context_alarm!(default_boolean_context_alarm, BooleanContextAlarmType);
default_context_alarm!(default_time_context_alarm, TimeContextAlarmType);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum CharacterWidthChoice {
    Default,
    #[strum(serialize = "8")]
    _8,
    #[strum(serialize = "16")]
    _16,
    #[strum(serialize = "32")]
    _32,
}
impl_select_item!(CharacterWidthChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum FloatSizeChoice {
    #[strum(serialize = "32")]
    _32,
    #[strum(serialize = "64")]
    _64,
    #[strum(serialize = "128")]
    _128,
}
impl_select_item!(FloatSizeChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum SignedChoice {
    #[strum(serialize = "true")]
    Signed,
    #[strum(serialize = "false")]
    Unsigned,
}
impl_select_item!(SignedChoice);

pub(super) struct ParameterTypeForm {
    present: bool,
    kind: ParameterTypeKind,
    kind_select: Entity<SelectState<Vec<ParameterTypeKind>>>,
    name_input: Entity<InputState>,
    base_or_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    unit_set: Entity<UnitSetForm>,
    time_encoding: Entity<TimeEncodingForm>,
    reference_time: Entity<ReferenceTimeForm>,
    default_alarm: Entity<DefaultAlarmForm>,
    context_alarms: Entity<ContextAlarmListForm>,
    to_string: Entity<ToStringForm>,
    valid_range: Entity<ValidRangeForm>,
    size_range_min_input: Entity<InputState>,
    size_range_max_input: Entity<InputState>,
    extra_a_input: Entity<InputState>,
    extra_b_input: Entity<InputState>,
    character_width_select: Entity<SelectState<Vec<CharacterWidthChoice>>>,
    signed_select: Entity<SelectState<Vec<SignedChoice>>>,
    float_size_select: Entity<SelectState<Vec<FloatSizeChoice>>>,
    enumeration_list: Entity<EnumerationListForm>,
    aggregate_members: Entity<AggregateMemberListForm>,
    dimensions: Entity<DimensionListForm>,
    data_encoding: Entity<DataEncodingForm>,
    base_defaults_open: bool,
    documentation_open: bool,
    metadata_open: bool,
    type_options_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl ParameterTypeForm {
    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(gpui::px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn new(
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let values = ParameterTypeValues::from_type(parameter_type);
        let kind = parameter_type
            .map(kind)
            .unwrap_or(ParameterTypeKind::String);
        let name_input = input(&values.name, false, window, cx);
        let alias_set = AliasSetForm::new(
            parameter_type.and_then(parameter_type_alias_set),
            window,
            cx,
        );
        let ancillary_data_set = AncillaryDataSetForm::new(
            parameter_type.and_then(parameter_type_ancillary_data_set),
            window,
            cx,
        );
        let unit_set =
            UnitSetForm::new(parameter_type.and_then(parameter_type_unit_set), window, cx);
        let time_encoding = TimeEncodingForm::new(
            parameter_type.and_then(parameter_type_time_encoding),
            window,
            cx,
        );
        let reference_time = ReferenceTimeForm::new(
            parameter_type.and_then(parameter_type_reference_time),
            window,
            cx,
        );
        let default_alarm = DefaultAlarmForm::new(
            parameter_alarm_kind(kind),
            parameter_type.and_then(parameter_default_alarm),
            window,
            cx,
        );
        let context_alarms = ContextAlarmListForm::new(
            parameter_alarm_kind(kind),
            parameter_type.and_then(parameter_context_alarm_list),
            window,
            cx,
        );
        let to_string = ToStringForm::new(
            parameter_type.and_then(parameter_type_to_string),
            window,
            cx,
        );
        let valid_range = ValidRangeForm::new(
            valid_range_kind(kind),
            parameter_type.and_then(integer_valid_range),
            parameter_type.and_then(float_valid_range),
            window,
            cx,
        );
        let size_range = parameter_type.and_then(string_size_range);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| {
            let kind_select = cx.new(|cx| {
                SelectState::new(
                    ParameterTypeKind::VARIANTS.to_vec(),
                    Some(IndexPath::default().row(kind.option_index())),
                    window,
                    cx,
                )
            });
            let base_or_ref_input = input(&values.base_or_ref, false, window, cx);
            let initial_value_input = input(&values.initial_value, false, window, cx);
            let short_description_input = input(&values.short_description, false, window, cx);
            let long_description_input = input(&values.long_description, true, window, cx);
            let size_range_min_input = input(
                &size_range
                    .and_then(|range| range.min_inclusive)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                false,
                window,
                cx,
            );
            let size_range_max_input = input(
                &size_range
                    .and_then(|range| range.max_inclusive)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                false,
                window,
                cx,
            );
            let extra_a_input = input(&values.extra_a, false, window, cx);
            let extra_b_input = input(&values.extra_b, false, window, cx);
            let character_width_select = select(
                CharacterWidthChoice::VARIANTS,
                parse_choice(
                    if values.extra_b.is_empty() {
                        "Default"
                    } else {
                        &values.extra_b
                    },
                    CharacterWidthChoice::Default,
                ),
                window,
                cx,
            );
            let signed_select = select(
                SignedChoice::VARIANTS,
                parse_choice(&values.extra_b, SignedChoice::Signed),
                window,
                cx,
            );
            let float_size_select = select(
                FloatSizeChoice::VARIANTS,
                parse_choice(&values.extra_a, FloatSizeChoice::_32),
                window,
                cx,
            );
            let mut subscriptions = vec![name_subscription];
            let enumeration_list = EnumerationListForm::new(parameter_type, window, cx);
            let aggregate_members = AggregateMemberListForm::new(parameter_type, window, cx);
            let dimensions = DimensionListForm::new(parameter_type, window, cx);

            let kind_extra_a = extra_a_input.clone();
            let kind_extra_b = extra_b_input.clone();
            let kind_character_width = character_width_select.clone();
            let kind_signed = signed_select.clone();
            let kind_float_size = float_size_select.clone();
            let kind_enumeration_list = enumeration_list.clone();
            let kind_aggregate_members = aggregate_members.clone();
            let kind_dimensions = dimensions.clone();
            let kind_time_encoding = time_encoding.clone();
            let kind_reference_time = reference_time.clone();
            let kind_default_alarm = default_alarm.clone();
            let kind_context_alarms = context_alarms.clone();
            let kind_to_string = to_string.clone();
            let kind_valid_range = valid_range.clone();
            let kind_size_range_min = size_range_min_input.clone();
            let kind_size_range_max = size_range_max_input.clone();
            subscriptions.push(cx.subscribe_in(
                &kind_select,
                window,
                move |this: &mut ParameterTypeForm,
                      _,
                      event: &SelectEvent<Vec<ParameterTypeKind>>,
                      window,
                      cx| {
                    let SelectEvent::Confirm(Some(selected_kind)) = event else {
                        return;
                    };
                    let selected_kind = *selected_kind;
                    if this.kind == selected_kind {
                        return;
                    }
                    this.kind = selected_kind;
                    let (extra_a, extra_b) = selected_kind.default_extra_values();
                    for (input, value) in [
                        (&kind_extra_a, extra_a.to_owned()),
                        (&kind_extra_b, extra_b.to_owned()),
                    ] {
                        input.update(cx, |input, cx| input.set_value(value, window, cx));
                    }
                    sync_select(
                        &kind_character_width,
                        parse_choice(
                            if selected_kind == ParameterTypeKind::String {
                                if extra_b.is_empty() {
                                    "Default"
                                } else {
                                    extra_b
                                }
                            } else {
                                "Default"
                            },
                            CharacterWidthChoice::Default,
                        ),
                        window,
                        cx,
                    );
                    sync_select(
                        &kind_signed,
                        parse_choice(
                            if selected_kind == ParameterTypeKind::Integer {
                                extra_b
                            } else {
                                "true"
                            },
                            SignedChoice::Signed,
                        ),
                        window,
                        cx,
                    );
                    sync_select(
                        &kind_float_size,
                        parse_choice(
                            if selected_kind == ParameterTypeKind::Float {
                                extra_a
                            } else {
                                "32"
                            },
                            FloatSizeChoice::_32,
                        ),
                        window,
                        cx,
                    );
                    for input in [&kind_size_range_min, &kind_size_range_max] {
                        input.update(cx, |input, cx| {
                            input.set_value(String::new(), window, cx);
                        });
                    }
                    kind_to_string.update(cx, |form, cx| {
                        form.load(None, window, cx);
                    });
                    kind_time_encoding.update(cx, |form, cx| {
                        form.load(None, window, cx);
                    });
                    kind_reference_time.update(cx, |form, cx| {
                        form.load(None, window, cx);
                    });
                    kind_default_alarm.update(cx, |form, cx| {
                        form.load(parameter_alarm_kind(selected_kind), None, window, cx);
                    });
                    kind_context_alarms.update(cx, |form, cx| {
                        form.load(parameter_alarm_kind(selected_kind), None, window, cx);
                    });
                    kind_valid_range.update(cx, |form, cx| {
                        form.reset(valid_range_kind(selected_kind), window, cx);
                    });
                    if selected_kind == ParameterTypeKind::Enumerated {
                        kind_enumeration_list.update(cx, |form, cx| {
                            form.reset_to_default(cx);
                        });
                    }
                    if selected_kind == ParameterTypeKind::Aggregate {
                        kind_aggregate_members.update(cx, |form, cx| {
                            form.reset_to_default(cx);
                        });
                    }
                    kind_dimensions.update(cx, |form, cx| {
                        form.reset(selected_kind == ParameterTypeKind::Array, window, cx);
                    });
                    cx.notify();
                },
            ));

            Self {
                present: parameter_type.is_some(),
                kind,
                kind_select,
                name_input,
                base_or_ref_input,
                initial_value_input,
                short_description_input,
                long_description_input,
                alias_set,
                ancillary_data_set,
                unit_set,
                time_encoding,
                reference_time,
                default_alarm,
                context_alarms,
                to_string,
                valid_range,
                size_range_min_input,
                size_range_max_input,
                extra_a_input,
                extra_b_input,
                character_width_select,
                signed_select,
                float_size_select,
                enumeration_list,
                aggregate_members,
                dimensions,
                data_encoding: DataEncodingForm::new(
                    parameter_type.and_then(find_data_encoding),
                    window,
                    cx,
                ),
                base_defaults_open: false,
                documentation_open: false,
                metadata_open: false,
                type_options_open: false,
                _subscriptions: subscriptions,
            }
        })
    }

    pub(super) fn load(
        &mut self,
        parameter_type: Option<&xtce::ParameterTypeSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = ParameterTypeValues::from_type(parameter_type);
        self.base_defaults_open = false;
        self.documentation_open = false;
        self.metadata_open = false;
        self.type_options_open = false;
        self.present = parameter_type.is_some();
        let selected_kind = parameter_type
            .map(kind)
            .unwrap_or(ParameterTypeKind::String);
        self.kind = selected_kind;
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&selected_kind, window, cx);
        });
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.base_or_ref_input, values.base_or_ref),
            (&self.initial_value_input, values.initial_value),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.extra_a_input, values.extra_a),
            (&self.extra_b_input, values.extra_b),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        let extra_a = value(&self.extra_a_input, cx);
        let extra_b = value(&self.extra_b_input, cx);
        let character_width = if extra_b.is_empty() {
            "Default"
        } else {
            &extra_b
        };
        sync_select(
            &self.character_width_select,
            parse_choice(character_width, CharacterWidthChoice::Default),
            window,
            cx,
        );
        sync_select(
            &self.signed_select,
            parse_choice(&extra_b, SignedChoice::Signed),
            window,
            cx,
        );
        sync_select(
            &self.float_size_select,
            parse_choice(&extra_a, FloatSizeChoice::_32),
            window,
            cx,
        );
        self.data_encoding.update(cx, |form, cx| {
            form.load(parameter_type.and_then(find_data_encoding), window, cx);
        });
        self.enumeration_list.update(cx, |form, cx| {
            form.load(parameter_type, cx);
        });
        self.aggregate_members.update(cx, |form, cx| {
            form.load(parameter_type, cx);
        });
        self.dimensions.update(cx, |form, cx| {
            form.load(parameter_type, window, cx);
        });
        self.alias_set.load(
            parameter_type.and_then(parameter_type_alias_set),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            parameter_type.and_then(parameter_type_ancillary_data_set),
            window,
            cx,
        );
        self.unit_set.update(cx, |form, cx| {
            form.load(parameter_type.and_then(parameter_type_unit_set), window, cx);
        });
        self.time_encoding.update(cx, |form, cx| {
            form.load(
                parameter_type.and_then(parameter_type_time_encoding),
                window,
                cx,
            );
        });
        self.reference_time.update(cx, |form, cx| {
            form.load(
                parameter_type.and_then(parameter_type_reference_time),
                window,
                cx,
            );
        });
        self.default_alarm.update(cx, |form, cx| {
            form.load(
                parameter_alarm_kind(selected_kind),
                parameter_type.and_then(parameter_default_alarm),
                window,
                cx,
            );
        });
        self.context_alarms.update(cx, |form, cx| {
            form.load(
                parameter_alarm_kind(selected_kind),
                parameter_type.and_then(parameter_context_alarm_list),
                window,
                cx,
            );
        });
        self.to_string.update(cx, |form, cx| {
            form.load(
                parameter_type.and_then(parameter_type_to_string),
                window,
                cx,
            );
        });
        self.valid_range.update(cx, |form, cx| {
            form.load(
                valid_range_kind(selected_kind),
                parameter_type.and_then(integer_valid_range),
                parameter_type.and_then(float_valid_range),
                window,
                cx,
            );
        });
        let size_range = parameter_type.and_then(string_size_range);
        for (input, value) in [
            (
                &self.size_range_min_input,
                size_range
                    .and_then(|range| range.min_inclusive)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ),
            (
                &self.size_range_max_input,
                size_range
                    .and_then(|range| range.max_inclusive)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    pub(super) fn apply_to(
        &self,
        parameter_type: &mut xtce::ParameterTypeSetTypeContent,
        cx: &App,
    ) {
        replace_parameter_type_kind(parameter_type, self.kind);
        let extra_a = if self.kind == ParameterTypeKind::Float {
            selected_value(&self.float_size_select, FloatSizeChoice::_32, cx).to_string()
        } else {
            value(&self.extra_a_input, cx)
        };
        let extra_b = match self.kind {
            ParameterTypeKind::String => {
                let width = selected_value(
                    &self.character_width_select,
                    CharacterWidthChoice::Default,
                    cx,
                );
                if width == CharacterWidthChoice::Default {
                    String::new()
                } else {
                    width.to_string()
                }
            }
            ParameterTypeKind::Integer => {
                selected_value(&self.signed_select, SignedChoice::Signed, cx).to_string()
            }
            _ => value(&self.extra_b_input, cx),
        };
        ParameterTypeValues {
            name: value(&self.name_input, cx),
            base_or_ref: value(&self.base_or_ref_input, cx),
            initial_value: value(&self.initial_value_input, cx),
            short_description: value(&self.short_description_input, cx),
            long_description: value(&self.long_description_input, cx),
            extra_a,
            extra_b,
        }
        .apply_to(parameter_type);
        self.enumeration_list.read(cx).apply_to(parameter_type, cx);
        self.aggregate_members.read(cx).apply_to(parameter_type, cx);
        self.dimensions.read(cx).apply_to(parameter_type, cx);
        set_data_encoding_kind(
            parameter_type,
            self.data_encoding.read(cx).selected_kind(cx),
        );
        if let Some(encoding) = find_data_encoding_mut(parameter_type) {
            self.data_encoding.read(cx).apply_to(encoding, cx);
        }
        set_parameter_type_alias_set(parameter_type, self.alias_set.value(cx));
        set_parameter_type_ancillary_data_set(parameter_type, self.ancillary_data_set.value(cx));
        set_parameter_type_unit_set(parameter_type, self.unit_set.read(cx).to_set(cx));
        set_parameter_type_time_encoding(parameter_type, self.time_encoding.read(cx).to_value(cx));
        set_parameter_type_reference_time(
            parameter_type,
            self.reference_time.read(cx).to_value(cx),
        );
        set_parameter_default_alarm(parameter_type, self.default_alarm.read(cx).edit(cx));
        set_parameter_context_alarm_list(parameter_type, self.context_alarms.read(cx).edits(cx));
        set_parameter_type_to_string(parameter_type, self.to_string.read(cx).to_value(cx));
        set_parameter_type_valid_range(parameter_type, self.valid_range.read(cx).to_value(cx));
        set_string_size_range(
            parameter_type,
            integer_range(
                &value(&self.size_range_min_input, cx),
                &value(&self.size_range_max_input, cx),
            ),
        );
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        if !self.present {
            return v_flex();
        }
        let kind = self.kind;

        let mut form = v_flex().gap_5().child(select_field(
            "Parameter type",
            "Required",
            &self.kind_select,
            cx,
        ));
        if kind == ParameterTypeKind::Array {
            form = form.child(field(
                "Array type reference",
                "Required",
                &self.base_or_ref_input,
                cx,
            ));
        }
        match kind {
            ParameterTypeKind::String => {
                form = form.child(self.type_options(cx));
            }
            ParameterTypeKind::Integer => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field("Size in bits", "Integer", &self.extra_a_input, cx))
                        .child(select_field("Signed", "Required", &self.signed_select, cx)),
                );
            }
            ParameterTypeKind::Float => {
                form = form.child(select_field(
                    "Size in bits",
                    "Required",
                    &self.float_size_select,
                    cx,
                ));
            }
            ParameterTypeKind::Boolean => {
                form = form.child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field(
                            "One string value",
                            "Optional; defaults to True",
                            &self.extra_a_input,
                            cx,
                        ))
                        .child(field(
                            "Zero string value",
                            "Optional; defaults to False",
                            &self.extra_b_input,
                            cx,
                        )),
                );
            }
            _ => {}
        }
        match kind {
            ParameterTypeKind::Enumerated => form = form.child(self.enumeration_list.clone()),
            ParameterTypeKind::Array => {
                form = form.child(self.dimensions.clone());
            }
            ParameterTypeKind::Aggregate => form = form.child(self.aggregate_members.clone()),
            _ => {}
        }
        if kind.supports_data_encoding() {
            form = form
                .child(self.unit_set.clone())
                .child(div().text_lg().font_semibold().child("Data encoding"))
                .child(self.data_encoding.clone());
        }
        if matches!(kind, ParameterTypeKind::Integer | ParameterTypeKind::Float) {
            form = form
                .child(self.to_string.clone())
                .child(self.valid_range.clone());
        }
        if matches!(
            kind,
            ParameterTypeKind::RelativeTime | ParameterTypeKind::AbsoluteTime
        ) {
            form = form
                .child(self.time_encoding.clone())
                .child(self.reference_time.clone());
        }
        if parameter_alarm_kind(kind) != AlarmKind::Unsupported {
            form = form
                .child(self.default_alarm.clone())
                .child(self.context_alarms.clone());
        }
        if kind == ParameterTypeKind::String {
            form = form.child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .child("Size range in characters"),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Minimum",
                                "Optional; inclusive",
                                &self.size_range_min_input,
                                cx,
                            ))
                            .child(field(
                                "Maximum",
                                "Optional; inclusive",
                                &self.size_range_max_input,
                                cx,
                            )),
                    ),
            );
        }
        form.child(self.base_defaults(cx))
            .child(self.documentation(cx))
            .child(self.metadata(cx))
    }

    fn type_options(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.type_options_open)
            .child(
                Button::new("toggle-parameter-type-options")
                    .small()
                    .link()
                    .icon(if self.type_options_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Type options")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.type_options_open = !this.type_options_open;
                        cx.notify();
                    })),
            )
            .content(
                h_flex()
                    .pt_3()
                    .gap_4()
                    .items_start()
                    .child(field(
                        "Restriction pattern",
                        "Optional",
                        &self.extra_a_input,
                        cx,
                    ))
                    .child(select_field(
                        "Character width",
                        "Optional",
                        &self.character_width_select,
                        cx,
                    )),
            )
    }

    fn base_defaults(&self, cx: &mut Context<Self>) -> Collapsible {
        let content = h_flex()
            .pt_3()
            .gap_4()
            .items_start()
            .when(self.kind != ParameterTypeKind::Array, |content| {
                content.when_some(self.kind.base_field(), |content, (label, hint)| {
                    content.child(field(label, hint, &self.base_or_ref_input, cx))
                })
            })
            .child(field(
                "Initial value",
                "Optional",
                &self.initial_value_input,
                cx,
            ));
        Collapsible::new()
            .open(self.base_defaults_open)
            .child(
                Button::new("toggle-parameter-type-base-defaults")
                    .small()
                    .link()
                    .icon(if self.base_defaults_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Base and defaults")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.base_defaults_open = !this.base_defaults_open;
                        cx.notify();
                    })),
            )
            .content(content)
    }

    fn documentation(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.documentation_open)
            .child(
                Button::new("toggle-parameter-type-documentation")
                    .small()
                    .link()
                    .icon(if self.documentation_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Documentation")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.documentation_open = !this.documentation_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_4()
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
                    )),
            )
    }

    fn metadata(&self, cx: &mut Context<Self>) -> Collapsible {
        Collapsible::new()
            .open(self.metadata_open)
            .child(
                Button::new("toggle-parameter-type-metadata")
                    .small()
                    .link()
                    .icon(if self.metadata_open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label("Metadata")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.metadata_open = !this.metadata_open;
                        cx.notify();
                    })),
            )
            .content(
                v_flex()
                    .pt_3()
                    .gap_4()
                    .child(self.alias_set.render(cx))
                    .child(self.ancillary_data_set.render(cx)),
            )
    }
}

impl Render for ParameterTypeForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ParameterTypeKind {
    #[strum(serialize = "StringParameterType")]
    String,
    #[strum(serialize = "EnumeratedParameterType")]
    Enumerated,
    #[strum(serialize = "IntegerParameterType")]
    Integer,
    #[strum(serialize = "BinaryParameterType")]
    Binary,
    #[strum(serialize = "FloatParameterType")]
    Float,
    #[strum(serialize = "BooleanParameterType")]
    Boolean,
    #[strum(serialize = "RelativeTimeParameterType")]
    RelativeTime,
    #[strum(serialize = "AbsoluteTimeParameterType")]
    AbsoluteTime,
    #[strum(serialize = "ArrayParameterType")]
    Array,
    #[strum(serialize = "AggregateParameterType")]
    Aggregate,
}
impl_select_item!(ParameterTypeKind);

impl ParameterTypeKind {
    fn option_index(self) -> usize {
        match self {
            Self::String => 0,
            Self::Enumerated => 1,
            Self::Integer => 2,
            Self::Binary => 3,
            Self::Float => 4,
            Self::Boolean => 5,
            Self::RelativeTime => 6,
            Self::AbsoluteTime => 7,
            Self::Array => 8,
            Self::Aggregate => 9,
        }
    }

    fn default_extra_values(self) -> (&'static str, &'static str) {
        match self {
            Self::Integer => ("32", "true"),
            Self::Float => ("32", "Float"),
            Self::Boolean => ("True", "False"),
            _ => ("", ""),
        }
    }

    fn supports_data_encoding(self) -> bool {
        matches!(
            self,
            Self::String
                | Self::Enumerated
                | Self::Integer
                | Self::Binary
                | Self::Float
                | Self::Boolean
        )
    }

    fn base_field(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Array => Some(("Array type reference", "Required")),
            Self::Aggregate => None,
            _ => Some(("Base type", "Optional; used only for type inheritance")),
        }
    }
}

struct ParameterTypeValues {
    name: String,
    base_or_ref: String,
    initial_value: String,
    short_description: String,
    long_description: String,
    extra_a: String,
    extra_b: String,
}

impl ParameterTypeValues {
    fn from_type(parameter_type: Option<&xtce::ParameterTypeSetTypeContent>) -> Self {
        let Some(parameter_type) = parameter_type else {
            return Self::empty();
        };
        macro_rules! common_content {
            ($value:expr, $content:ident) => {
                Self::common(
                    &$value.name,
                    $value.base_type.as_deref(),
                    $value
                        .initial_value
                        .as_ref()
                        .map(ToString::to_string)
                        .as_deref(),
                    $value.short_description.as_deref(),
                    long_description!($value.content, $content),
                    "",
                    "",
                )
            };
        }
        match parameter_type {
            xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
                let mut values = common_content!(value, StringParameterTypeContent);
                values.extra_a = value.restriction_pattern.clone().unwrap_or_default();
                values.extra_b = character_width_label(value.character_width.as_ref()).to_owned();
                values
            }
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                long_description!(value.content, EnumeratedParameterTypeContent),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
                let mut values = common_content!(value, IntegerParameterTypeContent);
                values.extra_a = value.size_in_bits.to_string();
                values.extra_b = value.signed.to_string();
                values
            }
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
                common_content!(value, BinaryParameterTypeContent)
            }
            xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
                let mut values = common_content!(value, FloatParameterTypeContent);
                values.extra_a = float_size_label(&value.size_in_bits).to_owned();
                values.extra_b = "Float".to_owned();
                values
            }
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
                let mut values = common_content!(value, BooleanParameterTypeContent);
                values.extra_a.clone_from(&value.one_string_value);
                values.extra_b.clone_from(&value.zero_string_value);
                values
            }
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => Self::common(
                &value.name,
                value.base_type.as_deref(),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => Self::common(
                &value.name,
                Some(&value.array_type_ref),
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
            xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => Self::common(
                &value.name,
                None,
                value.initial_value.as_deref(),
                value.short_description.as_deref(),
                value.long_description.as_deref().unwrap_or_default(),
                "",
                "",
            ),
        }
    }

    fn common(
        name: &str,
        base_or_ref: Option<&str>,
        initial_value: Option<&str>,
        short_description: Option<&str>,
        long_description: &str,
        extra_a: &str,
        extra_b: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            base_or_ref: base_or_ref.unwrap_or_default().to_owned(),
            initial_value: initial_value.unwrap_or_default().to_owned(),
            short_description: short_description.unwrap_or_default().to_owned(),
            long_description: long_description.to_owned(),
            extra_a: extra_a.to_owned(),
            extra_b: extra_b.to_owned(),
        }
    }

    fn empty() -> Self {
        Self::common("", None, None, None, "", "", "")
    }

    fn apply_to(&self, parameter_type: &mut xtce::ParameterTypeSetTypeContent) {
        macro_rules! common {
            ($value:expr) => {{
                $value.name.clone_from(&self.name);
                $value.base_type = optional_value(self.base_or_ref.clone());
                $value.short_description = optional_value(self.short_description.clone());
            }};
        }
        macro_rules! content {
            ($value:expr, $content:ident) => {{
                common!($value);
                set_long_description!($value.content, $content, &self.long_description);
            }};
        }
        match parameter_type {
            xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
                content!(value, StringParameterTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
                value.restriction_pattern = optional_value(self.extra_a.clone());
                value.character_width = character_width_from_str(&self.extra_b);
            }
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
                content!(value, EnumeratedParameterTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
            }
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
                content!(value, IntegerParameterTypeContent);
                value.initial_value = self.initial_value.parse().ok();
                if let Ok(size) = self.extra_a.parse() {
                    value.size_in_bits = size;
                }
                if let Some(signed) = bool_from_str(&self.extra_b) {
                    value.signed = signed;
                }
            }
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
                content!(value, BinaryParameterTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
            }
            xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
                content!(value, FloatParameterTypeContent);
                value.initial_value = self.initial_value.parse().ok();
                if let Some(size) = float_size_from_str(&self.extra_a) {
                    value.size_in_bits = size;
                }
            }
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
                content!(value, BooleanParameterTypeContent);
                value.initial_value = optional_value(self.initial_value.clone());
                value.one_string_value.clone_from(&self.extra_a);
                value.zero_string_value.clone_from(&self.extra_b);
            }
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
                common!(value);
                value.initial_value = optional_value(self.initial_value.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => {
                value.name.clone_from(&self.name);
                value.array_type_ref.clone_from(&self.base_or_ref);
                value.initial_value = optional_value(self.initial_value.clone());
                value.short_description = optional_value(self.short_description.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
                value.name.clone_from(&self.name);
                value.initial_value = optional_value(self.initial_value.clone());
                value.short_description = optional_value(self.short_description.clone());
                value.long_description = optional_value(self.long_description.clone());
            }
        }
    }
}

fn replace_parameter_type_kind(
    parameter_type: &mut xtce::ParameterTypeSetTypeContent,
    selected_kind: ParameterTypeKind,
) {
    if kind(parameter_type) == selected_kind {
        return;
    }

    *parameter_type = match selected_kind {
        ParameterTypeKind::String => {
            xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                restriction_pattern: None,
                character_width: None,
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Enumerated => {
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(
                xtce::EnumeratedParameterType {
                    short_description: None,
                    name: String::new(),
                    base_type: None,
                    initial_value: None,
                    content: vec![xtce::EnumeratedParameterTypeContent::EnumerationList(
                        xtce::EnumerationListType {
                            enumeration: vec![xtce::ValueEnumerationType {
                                value: 0,
                                max_value: None,
                                label: "VALUE".to_owned(),
                                short_description: None,
                            }],
                        },
                    )],
                },
            )
        }
        ParameterTypeKind::Integer => {
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::IntegerParameterType::default_size_in_bits(),
                signed: xtce::IntegerParameterType::default_signed(),
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Binary => {
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(xtce::BinaryParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Float => {
            xtce::ParameterTypeSetTypeContent::FloatParameterType(xtce::FloatParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::FloatParameterType::default_size_in_bits(),
                content: Vec::new(),
            })
        }
        ParameterTypeKind::Boolean => {
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(xtce::BooleanParameterType {
                short_description: None,
                name: String::new(),
                base_type: None,
                initial_value: None,
                one_string_value: xtce::BooleanParameterType::default_one_string_value(),
                zero_string_value: xtce::BooleanParameterType::default_zero_string_value(),
                content: Vec::new(),
            })
        }
        ParameterTypeKind::RelativeTime => {
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(
                xtce::RelativeTimeParameterType {
                    short_description: None,
                    name: String::new(),
                    base_type: None,
                    initial_value: None,
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                    encoding: None,
                    reference_time: None,
                    default_alarm: None,
                    context_alarm_list: None,
                },
            )
        }
        ParameterTypeKind::AbsoluteTime => {
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(
                xtce::AbsoluteTimeParameterType {
                    short_description: None,
                    name: String::new(),
                    base_type: None,
                    initial_value: None,
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                    encoding: None,
                    reference_time: None,
                },
            )
        }
        ParameterTypeKind::Array => {
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(xtce::ArrayParameterType {
                short_description: None,
                name: String::new(),
                array_type_ref: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                dimension_list: xtce::DimensionListType {
                    dimension: vec![xtce::DimensionType {
                        starting_index: xtce::IntegerValueType::FixedValue(0),
                        ending_index: xtce::IntegerValueType::FixedValue(0),
                    }],
                },
            })
        }
        ParameterTypeKind::Aggregate => xtce::ParameterTypeSetTypeContent::AggregateParameterType(
            xtce::AggregateParameterType {
                short_description: None,
                name: String::new(),
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                member_list: xtce::MemberListType {
                    member: vec![xtce::MemberType {
                        short_description: None,
                        name: "member".to_owned(),
                        type_ref: "MemberType".to_owned(),
                        initial_value: None,
                        long_description: None,
                        alias_set: None,
                        ancillary_data_set: None,
                    }],
                },
            },
        ),
    };
}

fn kind(parameter_type: &xtce::ParameterTypeSetTypeContent) -> ParameterTypeKind {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(_) => ParameterTypeKind::String,
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(_) => {
            ParameterTypeKind::Enumerated
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(_) => ParameterTypeKind::Integer,
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(_) => ParameterTypeKind::Binary,
        xtce::ParameterTypeSetTypeContent::FloatParameterType(_) => ParameterTypeKind::Float,
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(_) => ParameterTypeKind::Boolean,
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(_) => {
            ParameterTypeKind::RelativeTime
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(_) => {
            ParameterTypeKind::AbsoluteTime
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(_) => ParameterTypeKind::Array,
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(_) => {
            ParameterTypeKind::Aggregate
        }
    }
}

fn character_width_label(width: Option<&xtce::CharacterWidthType>) -> &'static str {
    match width {
        Some(xtce::CharacterWidthType::_8) => "8",
        Some(xtce::CharacterWidthType::_16) => "16",
        Some(xtce::CharacterWidthType::_32) => "32",
        None => "",
    }
}

fn character_width_from_str(value: &str) -> Option<xtce::CharacterWidthType> {
    match value.trim() {
        "8" => Some(xtce::CharacterWidthType::_8),
        "16" => Some(xtce::CharacterWidthType::_16),
        "32" => Some(xtce::CharacterWidthType::_32),
        _ => None,
    }
}

fn float_size_label(size: &xtce::FloatSizeInBitsType) -> &'static str {
    match size {
        xtce::FloatSizeInBitsType::_32 => "32",
        xtce::FloatSizeInBitsType::_64 => "64",
        xtce::FloatSizeInBitsType::_128 => "128",
    }
}

fn float_size_from_str(value: &str) -> Option<xtce::FloatSizeInBitsType> {
    match value.trim() {
        "32" => Some(xtce::FloatSizeInBitsType::_32),
        "64" => Some(xtce::FloatSizeInBitsType::_64),
        "128" => Some(xtce::FloatSizeInBitsType::_128),
        _ => None,
    }
}

fn bool_from_str(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

fn integer_range(minimum: &str, maximum: &str) -> Option<xtce::IntegerRangeType> {
    let minimum = minimum.trim().parse().ok();
    let maximum = maximum.trim().parse().ok();
    if minimum.is_none() && maximum.is_none() {
        None
    } else {
        Some(xtce::IntegerRangeType {
            min_inclusive: minimum,
            max_inclusive: maximum,
        })
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

fn parse_choice<T>(label: &str, fallback: T) -> T
where
    T: std::str::FromStr,
{
    label.parse().unwrap_or(fallback)
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

fn sync_select<T>(
    select: &Entity<SelectState<Vec<T>>>,
    value: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select.update(cx, |select, cx| {
        select.set_selected_value(&value, window, cx);
    });
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

fn select_field<T>(
    label: &'static str,
    hint: &'static str,
    select: &Entity<SelectState<Vec<T>>>,
    _cx: &App,
) -> Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    super::select_field(label, hint, select)
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use crate::forms::parameter_alarm::AlarmEdit;

    use super::{
        ContextAlarmEdit, ParameterTypeKind, ParameterTypeValues, ValidRangeValue,
        float_valid_range, integer_valid_range, parameter_type_alias_set,
        parameter_type_ancillary_data_set, parameter_type_time_encoding, parameter_type_to_string,
        parameter_type_unit_set, replace_parameter_type_kind, set_parameter_context_alarm_list,
        set_parameter_default_alarm, set_parameter_type_alias_set,
        set_parameter_type_ancillary_data_set, set_parameter_type_time_encoding,
        set_parameter_type_to_string, set_parameter_type_unit_set, set_parameter_type_valid_range,
        set_string_size_range, string_size_range,
    };

    #[test]
    fn applying_boolean_type_values_preserves_child_elements() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(xtce::BooleanParameterType {
                short_description: None,
                name: "OldFlagType".to_owned(),
                base_type: None,
                initial_value: None,
                one_string_value: "True".to_owned(),
                zero_string_value: "False".to_owned(),
                content: vec![xtce::BooleanParameterTypeContent::AncillaryDataSet(
                    xtce::AncillaryDataSetType {
                        ancillary_data: Vec::new(),
                    },
                )],
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        values.name = "OperationalFlagType".to_owned();
        values.initial_value = "Enabled".to_owned();
        values.extra_a = "Enabled".to_owned();
        values.extra_b = "Disabled".to_owned();
        values.apply_to(&mut parameter_type);

        match parameter_type {
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
                assert_eq!(value.name, "OperationalFlagType");
                assert_eq!(value.initial_value.as_deref(), Some("Enabled"));
                assert_eq!(value.one_string_value, "Enabled");
                assert_eq!(value.content.len(), 1);
            }
            _ => panic!("expected a BooleanParameterType"),
        }
    }

    #[test]
    fn long_description_is_editable_for_scalar_parameter_types() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                short_description: None,
                name: "ModeType".to_owned(),
                base_type: None,
                initial_value: None,
                restriction_pattern: None,
                character_width: None,
                content: vec![
                    xtce::StringParameterTypeContent::LongDescription(
                        "Original description".to_owned(),
                    ),
                    xtce::StringParameterTypeContent::AncillaryDataSet(
                        xtce::AncillaryDataSetType {
                            ancillary_data: Vec::new(),
                        },
                    ),
                ],
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        assert_eq!(values.long_description, "Original description");

        values.long_description = "Updated description".to_owned();
        values.apply_to(&mut parameter_type);

        let xtce::ParameterTypeSetTypeContent::StringParameterType(parameter_type) = parameter_type
        else {
            panic!("expected a StringParameterType");
        };
        assert!(parameter_type.content.iter().any(|item| matches!(
            item,
            xtce::StringParameterTypeContent::LongDescription(value)
                if value == "Updated description"
        )));
        assert!(
            parameter_type
                .content
                .iter()
                .any(|item| matches!(item, xtce::StringParameterTypeContent::AncillaryDataSet(_)))
        );
    }

    #[test]
    fn alias_set_is_editable_for_scalar_parameter_types() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: vec![xtce::IntegerParameterTypeContent::LongDescription(
                    "Counter".to_owned(),
                )],
            });
        set_parameter_type_alias_set(
            &mut parameter_type,
            Some(xtce::AliasSetType {
                alias: vec![xtce::AliasType {
                    name_space: "ops".to_owned(),
                    alias: "COUNT".to_owned(),
                }],
            }),
        );

        let aliases = parameter_type_alias_set(&parameter_type).expect("alias set");
        assert_eq!(aliases.alias[0].name_space, "ops");
        assert_eq!(aliases.alias[0].alias, "COUNT");
        let xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) = &parameter_type else {
            panic!("expected an IntegerParameterType");
        };
        assert!(matches!(
            value.content[0],
            xtce::IntegerParameterTypeContent::LongDescription(_)
        ));
        assert!(matches!(
            value.content[1],
            xtce::IntegerParameterTypeContent::AliasSet(_)
        ));

        set_parameter_type_alias_set(&mut parameter_type, None);
        assert!(parameter_type_alias_set(&parameter_type).is_none());
    }

    #[test]
    fn ancillary_data_set_is_editable_for_scalar_parameter_types() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                short_description: None,
                name: "ModeType".to_owned(),
                base_type: None,
                initial_value: None,
                restriction_pattern: None,
                character_width: None,
                content: vec![xtce::StringParameterTypeContent::AliasSet(
                    xtce::AliasSetType { alias: Vec::new() },
                )],
            });
        set_parameter_type_ancillary_data_set(
            &mut parameter_type,
            Some(xtce::AncillaryDataSetType {
                ancillary_data: vec![xtce::AncillaryDataType {
                    name: "guide".to_owned(),
                    mime_type: "text/plain".to_owned(),
                    href: None,
                    content: "Mode guide".to_owned(),
                }],
            }),
        );

        let ancillary = parameter_type_ancillary_data_set(&parameter_type).expect("ancillary data");
        assert_eq!(ancillary.ancillary_data[0].name, "guide");
        let xtce::ParameterTypeSetTypeContent::StringParameterType(value) = &parameter_type else {
            panic!("expected a StringParameterType");
        };
        assert!(matches!(
            value.content[0],
            xtce::StringParameterTypeContent::AliasSet(_)
        ));
        assert!(matches!(
            value.content[1],
            xtce::StringParameterTypeContent::AncillaryDataSet(_)
        ));

        set_parameter_type_ancillary_data_set(&mut parameter_type, None);
        assert!(parameter_type_ancillary_data_set(&parameter_type).is_none());
    }

    #[test]
    fn unit_set_is_editable_in_schema_order() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::FloatParameterType(xtce::FloatParameterType {
                short_description: None,
                name: "VelocityType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::FloatParameterType::default_size_in_bits(),
                content: vec![xtce::FloatParameterTypeContent::AncillaryDataSet(
                    xtce::AncillaryDataSetType {
                        ancillary_data: Vec::new(),
                    },
                )],
            });
        set_parameter_type_unit_set(
            &mut parameter_type,
            Some(xtce::UnitSetType {
                unit: vec![xtce::UnitType {
                    power: 1.0,
                    factor: "1".to_owned(),
                    description: None,
                    form: xtce::UnitFormType::Calibrated,
                    text: Some(xtce::XmlText("m/s".to_owned())),
                }],
            }),
        );

        let units = parameter_type_unit_set(&parameter_type).expect("unit set");
        assert_eq!(
            units.unit[0].text.as_ref().map(|text| text.0.as_str()),
            Some("m/s")
        );
        let xtce::ParameterTypeSetTypeContent::FloatParameterType(value) = &parameter_type else {
            panic!("expected a FloatParameterType");
        };
        assert!(matches!(
            value.content[0],
            xtce::FloatParameterTypeContent::AncillaryDataSet(_)
        ));
        assert!(matches!(
            value.content[1],
            xtce::FloatParameterTypeContent::UnitSet(_)
        ));
    }

    #[test]
    fn string_size_range_is_editable_in_schema_order() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
                short_description: None,
                name: "IdentifierType".to_owned(),
                base_type: None,
                initial_value: None,
                restriction_pattern: None,
                character_width: None,
                content: vec![xtce::StringParameterTypeContent::DefaultAlarm(
                    xtce::StringAlarmType {
                        name: None,
                        short_description: None,
                        min_violations: xtce::StringAlarmType::default_min_violations(),
                        min_conformance: xtce::StringAlarmType::default_min_conformance(),
                        disabled: xtce::StringAlarmType::default_disabled(),
                        default_alarm_level: xtce::StringAlarmType::default_default_alarm_level(),
                        content: Vec::new(),
                    },
                )],
            });
        set_string_size_range(
            &mut parameter_type,
            Some(xtce::IntegerRangeType {
                min_inclusive: Some(2),
                max_inclusive: Some(16),
            }),
        );

        let range = string_size_range(&parameter_type).expect("size range");
        assert_eq!(range.min_inclusive, Some(2));
        assert_eq!(range.max_inclusive, Some(16));
        let xtce::ParameterTypeSetTypeContent::StringParameterType(value) = &parameter_type else {
            panic!("expected a StringParameterType");
        };
        assert!(matches!(
            value.content[0],
            xtce::StringParameterTypeContent::SizeRangeInCharacters(_)
        ));
        assert!(matches!(
            value.content[1],
            xtce::StringParameterTypeContent::DefaultAlarm(_)
        ));

        set_string_size_range(&mut parameter_type, None);
        assert!(string_size_range(&parameter_type).is_none());
    }

    #[test]
    fn to_string_is_editable_for_numeric_parameter_types() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: vec![xtce::IntegerParameterTypeContent::ValidRange(
                    xtce::IntegerDataTypeValidRangeElementType {
                        min_inclusive: Some(0),
                        max_inclusive: Some(100),
                        valid_range_applies_to_calibrated: true,
                    },
                )],
            });
        set_parameter_type_to_string(
            &mut parameter_type,
            Some(xtce::ToStringType {
                number_format: xtce::NumberFormatType {
                    number_base: xtce::RadixType::Decimal,
                    minimum_fraction_digits: 0,
                    maximum_fraction_digits: None,
                    minimum_integer_digits: 1,
                    maximum_integer_digits: None,
                    negative_suffix: String::new(),
                    positive_suffix: String::new(),
                    negative_prefix: "-".to_owned(),
                    positive_prefix: String::new(),
                    show_thousands_grouping: false,
                    notation: xtce::FloatingPointNotationType::Normal,
                },
            }),
        );

        let format = parameter_type_to_string(&parameter_type).expect("to string");
        assert_eq!(format.number_format.minimum_integer_digits, 1);
        let xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) = &parameter_type else {
            panic!("expected an IntegerParameterType");
        };
        assert!(matches!(
            value.content[0],
            xtce::IntegerParameterTypeContent::ToString(_)
        ));
        assert!(matches!(
            value.content[1],
            xtce::IntegerParameterTypeContent::ValidRange(_)
        ));

        set_parameter_type_to_string(&mut parameter_type, None);
        assert!(parameter_type_to_string(&parameter_type).is_none());
    }

    #[test]
    fn valid_range_is_editable_for_numeric_parameter_types() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::FloatParameterType(xtce::FloatParameterType {
                short_description: None,
                name: "TemperatureType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: xtce::FloatSizeInBitsType::_64,
                content: Vec::new(),
            });
        set_parameter_type_valid_range(
            &mut parameter_type,
            Some(ValidRangeValue::Float(
                xtce::FloatDataTypeValidRangeElementType {
                    min_inclusive: Some(-40.0),
                    min_exclusive: None,
                    max_inclusive: None,
                    max_exclusive: Some(125.0),
                    valid_range_applies_to_calibrated: true,
                },
            )),
        );

        let range = float_valid_range(&parameter_type).expect("valid range");
        assert_eq!(range.min_inclusive, Some(-40.0));
        assert_eq!(range.max_exclusive, Some(125.0));
        let xtce::ParameterTypeSetTypeContent::FloatParameterType(value) = &parameter_type else {
            panic!("expected a FloatParameterType");
        };
        assert!(matches!(
            value.content[0],
            xtce::FloatParameterTypeContent::ValidRange(_)
        ));

        set_parameter_type_valid_range(&mut parameter_type, None);
        assert!(float_valid_range(&parameter_type).is_none());
        assert!(integer_valid_range(&parameter_type).is_none());
    }

    #[test]
    fn time_encoding_is_editable_for_time_parameter_types() {
        let mut parameter_type = xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(
            xtce::AbsoluteTimeParameterType {
                short_description: None,
                name: "TimestampType".to_owned(),
                base_type: None,
                initial_value: None,
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                encoding: None,
                reference_time: None,
            },
        );
        set_parameter_type_time_encoding(
            &mut parameter_type,
            Some(xtce::EncodingType {
                units: xtce::TimeUnitsType::Milliseconds,
                scale: 0.25,
                offset: 10.0,
                content: xtce::EncodingTypeContent::IntegerDataEncoding(
                    xtce::IntegerDataEncodingType {
                        bit_order: xtce::IntegerDataEncodingType::default_bit_order(),
                        byte_order: xtce::IntegerDataEncodingType::default_byte_order(),
                        encoding: xtce::IntegerDataEncodingType::default_encoding(),
                        size_in_bits: 32,
                        change_threshold: None,
                        error_detect_correct: None,
                        default_calibrator: None,
                        context_calibrator_list: None,
                    },
                ),
            }),
        );

        let encoding = parameter_type_time_encoding(&parameter_type).expect("time encoding");
        assert!(matches!(encoding.units, xtce::TimeUnitsType::Milliseconds));
        assert_eq!(encoding.scale, 0.25);
        assert_eq!(encoding.offset, 10.0);

        set_parameter_type_time_encoding(&mut parameter_type, None);
        assert!(parameter_type_time_encoding(&parameter_type).is_none());
    }

    #[test]
    fn parameter_alarms_preserve_advanced_content_and_add_context_rows() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "TemperatureType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: vec![xtce::IntegerParameterTypeContent::DefaultAlarm(
                    xtce::NumericAlarmType {
                        name: None,
                        short_description: None,
                        min_violations: 1,
                        min_conformance: 1,
                        disabled: false,
                        content: vec![xtce::NumericAlarmTypeContent::AlarmConditions(
                            xtce::AlarmConditionsType {
                                watch_alarm: None,
                                warning_alarm: None,
                                distress_alarm: None,
                                critical_alarm: None,
                                severe_alarm: None,
                            },
                        )],
                    },
                )],
            });
        let mut default_edit = test_alarm_edit("warning | 0 | inclusive | 100 | exclusive");
        default_edit.alarm_conditions = Some(xtce::AlarmConditionsType {
            watch_alarm: None,
            warning_alarm: None,
            distress_alarm: None,
            critical_alarm: None,
            severe_alarm: None,
        });
        set_parameter_default_alarm(&mut parameter_type, Some(default_edit));
        set_parameter_context_alarm_list(
            &mut parameter_type,
            vec![ContextAlarmEdit {
                context_match: xtce::ContextMatchType::Comparison(xtce::ComparisonType {
                    parameter_ref: "Mode".to_owned(),
                    instance: 0,
                    use_calibrated_value: true,
                    comparison_operator: "==".to_owned(),
                    value: "SAFE".to_owned(),
                }),
                alarm: test_alarm_edit("critical | 100 | inclusive |  | inclusive"),
            }],
        );

        let xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) = parameter_type else {
            panic!("expected IntegerParameterType");
        };
        let default_alarm = value.content.iter().find_map(|item| match item {
            xtce::IntegerParameterTypeContent::DefaultAlarm(value) => Some(value),
            _ => None,
        });
        let default_alarm = default_alarm.expect("default alarm");
        assert!(
            default_alarm
                .content
                .iter()
                .any(|item| matches!(item, xtce::NumericAlarmTypeContent::AlarmConditions(_)))
        );
        assert!(
            default_alarm
                .content
                .iter()
                .any(|item| matches!(item, xtce::NumericAlarmTypeContent::StaticAlarmRanges(_)))
        );
        let contexts = value.content.iter().find_map(|item| match item {
            xtce::IntegerParameterTypeContent::ContextAlarmList(value) => Some(value),
            _ => None,
        });
        assert_eq!(contexts.expect("context alarms").context_alarm.len(), 1);
    }

    fn test_alarm_edit(details: &str) -> AlarmEdit {
        AlarmEdit {
            name: String::new(),
            short_description: String::new(),
            min_violations: 1,
            min_conformance: 1,
            disabled: false,
            default_alarm_level: xtce::ConcernLevelsType::Normal,
            range_form: xtce::RangeFormType::Outside,
            time_units: xtce::TimeUnitsType::Seconds,
            details: details.to_owned(),
            static_ranges_active: !details.is_empty(),
            ancillary_data_set: None,
            alarm_conditions: None,
            custom_alarm: None,
            static_range_name: String::new(),
            static_range_short_description: String::new(),
            static_range_ancillary_data_set: None,
            change_alarm_ranges: None,
            alarm_multi_ranges: None,
            change_per_second_alarm_ranges: None,
        }
    }

    #[test]
    fn applying_integer_type_values_updates_size_and_signedness() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        values.extra_a = "16".to_owned();
        values.extra_b = "false".to_owned();
        values.apply_to(&mut parameter_type);

        match parameter_type {
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
                assert_eq!(value.size_in_bits, 16);
                assert!(!value.signed);
            }
            _ => panic!("expected an IntegerParameterType"),
        }
    }

    #[test]
    fn changing_the_parameter_type_kind_keeps_editable_common_values() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: Some("Counter".to_owned()),
                name: "CounterType".to_owned(),
                base_type: None,
                initial_value: Some(3),
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });
        let mut values = ParameterTypeValues::from_type(Some(&parameter_type));
        values.extra_a = "Enabled".to_owned();
        values.extra_b = "Disabled".to_owned();

        replace_parameter_type_kind(&mut parameter_type, ParameterTypeKind::Boolean);
        values.apply_to(&mut parameter_type);

        let xtce::ParameterTypeSetTypeContent::BooleanParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected a BooleanParameterType");
        };
        assert_eq!(parameter_type.name, "CounterType");
        assert_eq!(parameter_type.short_description.as_deref(), Some("Counter"));
        assert_eq!(parameter_type.initial_value.as_deref(), Some("3"));
        assert_eq!(parameter_type.one_string_value, "Enabled");
        assert_eq!(parameter_type.zero_string_value, "Disabled");
    }

    #[test]
    fn selecting_aggregate_creates_its_required_member_list() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "ValueType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });

        replace_parameter_type_kind(&mut parameter_type, ParameterTypeKind::Aggregate);

        let xtce::ParameterTypeSetTypeContent::AggregateParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected an AggregateParameterType");
        };
        assert_eq!(parameter_type.member_list.member.len(), 1);
        assert_eq!(parameter_type.member_list.member[0].name, "member");
    }

    #[test]
    fn selecting_enumerated_creates_its_required_enumeration_list() {
        let mut parameter_type =
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(xtce::IntegerParameterType {
                short_description: None,
                name: "ValueType".to_owned(),
                base_type: None,
                initial_value: None,
                size_in_bits: 32,
                signed: true,
                content: Vec::new(),
            });

        replace_parameter_type_kind(&mut parameter_type, ParameterTypeKind::Enumerated);

        let xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(parameter_type) =
            parameter_type
        else {
            panic!("expected an EnumeratedParameterType");
        };
        let Some(xtce::EnumeratedParameterTypeContent::EnumerationList(list)) =
            parameter_type.content.first()
        else {
            panic!("expected an EnumerationList");
        };
        assert_eq!(list.enumeration.len(), 1);
        assert_eq!(list.enumeration[0].value, 0);
        assert_eq!(list.enumeration[0].label, "VALUE");
    }

    #[test]
    fn base_field_matches_the_selected_parameter_type() {
        assert_eq!(
            ParameterTypeKind::Integer.base_field(),
            Some(("Base type", "Optional; used only for type inheritance"))
        );
        assert_eq!(
            ParameterTypeKind::Array.base_field(),
            Some(("Array type reference", "Required"))
        );
        assert_eq!(ParameterTypeKind::Aggregate.base_field(), None);
    }
}
