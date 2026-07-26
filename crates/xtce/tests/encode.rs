use xtce::{
    AliasSetType, AliasType, AuthorSetType, HeaderType, HistorySetType, NoteSetType, SpaceSystem,
    SpaceSystemType, SystemTypeType, TelemetryMetaDataType, ValidationStatusType, XTCE_NAMESPACE,
    from_str, to_string,
};

fn space_system(name: &str, system_type: SystemTypeType) -> SpaceSystem {
    let mut value = SpaceSystemType::new(name);
    value.system_type = system_type;
    value
}

#[test]
fn encodes_generated_types_with_the_xtce_namespace() {
    let mut document = space_system(
        "ExampleMission & Friends <\"Mission\">",
        SystemTypeType::Asset,
    );
    let root = &mut document;
    root.asset_type = "spacecraft".to_owned();
    root.header = Some(HeaderType {
        version: Some("0.0.0".to_owned()),
        date: None,
        classification: HeaderType::default_classification(),
        classification_instructions: None,
        validation_status: ValidationStatusType::Draft,
        author_set: None,
        note_set: None,
        history_set: None,
    });
    root.telemetry_meta_data = Some(TelemetryMetaDataType {
        parameter_type_set: None,
        parameter_set: None,
        container_set: None,
        message_set: None,
        stream_set: None,
        algorithm_set: None,
    });
    root.space_system
        .push(space_system("Payload", SystemTypeType::AssetComponent));

    let xml = to_string(&document).unwrap();

    assert!(xml.contains(&format!("xmlns:xtce=\"{XTCE_NAMESPACE}\"")));
    assert!(xml.contains("ExampleMission &amp; Friends"));
    assert!(xml.contains("&lt;"));
    assert!(xml.contains("&quot;Mission&quot;"));
    assert!(xml.contains("<xtce:Header"));
    assert!(xml.contains("<xtce:TelemetryMetaData"));
    assert!(xml.contains("<xtce:SpaceSystem name=\"Payload\""));

    let decoded = from_str(&xml).unwrap();
    assert_eq!(decoded.name, "ExampleMission & Friends <\"Mission\">");
    assert!(matches!(decoded.system_type, SystemTypeType::Asset));
    assert_eq!(decoded.asset_type, "spacecraft");
    assert!(matches!(
        decoded.header.as_ref().unwrap().validation_status,
        ValidationStatusType::Draft
    ));
    assert!(decoded.telemetry_meta_data.is_some());
    assert_eq!(decoded.space_system[0].name, "Payload");
}

#[test]
fn round_trips_a_minimal_space_system_with_schema_defaults() {
    let document = space_system("Minimal", SpaceSystemType::default_system_type());

    let xml = to_string(&document).unwrap();
    let decoded = from_str(&xml).unwrap();
    assert!(xml.contains("systemType=\"unknown\""));
    assert!(xml.contains("assetType=\"unknown\""));
    assert_eq!(decoded.name, "Minimal");
    assert!(matches!(decoded.system_type, SystemTypeType::Unknown));
    assert_eq!(decoded.asset_type, "unknown");
}

#[test]
fn round_trips_repeated_aliases_and_header_entries_in_order() {
    let mut document = space_system("Catalog", SystemTypeType::AssetGroup);
    let root = &mut document;
    root.alias_set = Some(AliasSetType {
        alias: vec![
            AliasType {
                name_space: "NORAD".to_owned(),
                alias: "25544".to_owned(),
            },
            AliasType {
                name_space: "COSPAR".to_owned(),
                alias: "1998-067A".to_owned(),
            },
        ],
    });
    root.header = Some(HeaderType {
        version: Some("1.2.3".to_owned()),
        date: Some("2026-07-25".to_owned()),
        classification: "Public".to_owned(),
        classification_instructions: Some("Redistribution permitted".to_owned()),
        validation_status: ValidationStatusType::Released,
        author_set: Some(AuthorSetType {
            author: vec!["Flight Dynamics".to_owned(), "Operations".to_owned()],
        }),
        note_set: Some(NoteSetType {
            note: vec!["First note".to_owned(), "Second note".to_owned()],
        }),
        history_set: Some(HistorySetType {
            history: vec!["Created".to_owned(), "Released".to_owned()],
        }),
    });

    let xml = to_string(&document).unwrap();
    let decoded = from_str(&xml).unwrap();
    let aliases = &decoded.alias_set.as_ref().unwrap().alias;
    let header = decoded.header.as_ref().unwrap();

    assert_eq!(aliases.len(), 2);
    assert_eq!(aliases[0].name_space, "NORAD");
    assert_eq!(aliases[0].alias, "25544");
    assert_eq!(aliases[1].name_space, "COSPAR");
    assert_eq!(aliases[1].alias, "1998-067A");
    assert_eq!(
        header.author_set.as_ref().unwrap().author,
        ["Flight Dynamics", "Operations"]
    );
    assert_eq!(
        header.note_set.as_ref().unwrap().note,
        ["First note", "Second note"]
    );
    assert_eq!(
        header.history_set.as_ref().unwrap().history,
        ["Created", "Released"]
    );
    assert!(matches!(
        header.validation_status,
        ValidationStatusType::Released
    ));
}
