use xtce::{SystemTypeType, XTCE_NAMESPACE, from_str};

#[test]
fn applies_space_system_attribute_defaults() {
    let xml = format!(r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="Minimal" />"#);

    let document = from_str(&xml).unwrap();
    let space_system = &document;

    assert!(matches!(space_system.system_type, SystemTypeType::Unknown));
    assert_eq!(space_system.asset_type, "unknown");
}

#[test]
fn rejects_a_space_system_without_its_required_name() {
    let xml = format!(r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" />"#);

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_an_invalid_system_type() {
    let xml = format!(
        r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission" systemType="satellite" />"#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_a_wrong_root_element() {
    let xml = format!(r#"<Header xmlns="{XTCE_NAMESPACE}" validationStatus="Draft" />"#);

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_malformed_xml() {
    let xml = format!(
        r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission"><Header></SpaceSystem>"#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_empty_and_incomplete_documents() {
    let inputs = [
        "",
        "   \n\t",
        r#"<SpaceSystem name="ExampleMission""#,
        r#"<SpaceSystem name="ExampleMission">"#,
    ];

    for xml in inputs {
        assert!(from_str(xml).is_err(), "unexpectedly accepted {xml:?}");
    }
}

#[test]
fn rejects_duplicate_attributes() {
    let xml = format!(r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="First" name="Second" />"#);

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_unexpected_attributes() {
    let xml =
        format!(r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission" typo="value" />"#);

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_duplicate_singleton_children() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission">
          <TelemetryMetaData />
          <TelemetryMetaData />
        </SpaceSystem>
        "#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_a_header_without_validation_status() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission">
          <Header />
        </SpaceSystem>
        "#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_an_invalid_header_validation_status() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission">
          <Header validationStatus="Approved" />
        </SpaceSystem>
        "#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_collection_entries_missing_required_attributes() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission">
          <AliasSet>
            <Alias nameSpace="NORAD" />
          </AliasSet>
        </SpaceSystem>
        "#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn accepts_empty_header_collections_allowed_by_the_schema() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission">
          <Header validationStatus="Draft">
            <AuthorSet />
            <NoteSet />
            <HistorySet />
          </Header>
        </SpaceSystem>
        "#
    );

    let document = from_str(&xml).unwrap();
    let header = document.header.as_ref().unwrap();

    assert!(header.author_set.as_ref().unwrap().author.is_empty());
    assert!(header.note_set.as_ref().unwrap().note.is_empty());
    assert!(header.history_set.as_ref().unwrap().history.is_empty());
}

#[test]
fn rejects_unknown_and_out_of_order_children() {
    let documents = [
        format!(
            r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission"><Unknown /></SpaceSystem>"#
        ),
        format!(
            r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission"><TelemetryMetaData /><Header validationStatus="Draft" /></SpaceSystem>"#
        ),
    ];

    for xml in documents {
        assert!(from_str(&xml).is_err(), "unexpectedly accepted {xml}");
    }
}

#[test]
fn rejects_invalid_character_entities() {
    let xml =
        format!(r#"<SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission &notAnEntity;" />"#);

    assert!(from_str(&xml).is_err());
}

#[test]
fn rejects_a_child_from_an_explicitly_wrong_namespace() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" xmlns:other="https://example.com/not-xtce" name="ExampleMission">
          <other:TelemetryMetaData />
        </SpaceSystem>
        "#
    );

    assert!(from_str(&xml).is_err());
}

#[test]
fn decodes_xml_entities_in_attributes_and_text() {
    let xml = format!(
        r#"
        <SpaceSystem xmlns="{XTCE_NAMESPACE}" name="ExampleMission &amp; Friends">
          <LongDescription>A &lt; B &amp; C</LongDescription>
        </SpaceSystem>
        "#
    );

    let document = from_str(&xml).unwrap();
    let space_system = &document;

    assert_eq!(space_system.name, "ExampleMission & Friends");
    assert_eq!(space_system.long_description.as_deref(), Some("A < B & C"));
}

#[test]
fn rejects_an_xsi_nil_space_system() {
    let xml = format!(
        r#"<xtce:SpaceSystem xmlns:xtce="{XTCE_NAMESPACE}" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:nil="true" />"#
    );

    assert!(from_str(&xml).is_err());
}
