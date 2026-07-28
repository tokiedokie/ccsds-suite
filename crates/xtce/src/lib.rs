#![allow(clippy::all)]
#![allow(missing_docs)]

use xsd_parser_types::quick_xml::{DeserializeSync, SerializeSync, SliceReader, Writer};

#[allow(warnings)]
pub mod generated {
    pub type OperatorString = String;

    include!("generated.rs");
}

pub use generated::*;
pub use xsd_parser_types::xml::Text as XmlText;

pub const XTCE_NAMESPACE: &str = "http://www.omg.org/spec/XTCE/20250214";

impl SpaceSystemType {
    /// Creates a minimal space system with the XTCE schema defaults.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_description: None,
            name: name.into(),
            system_type: Self::default_system_type(),
            asset_type: Self::default_asset_type(),
            operational_status: None,
            base: None,
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            header: None,
            telemetry_meta_data: None,
            command_meta_data: None,
            service_set: None,
            space_system: Vec::new(),
        }
    }
}

/// Decodes an XTCE 1.3 document using the generated deserializer.
pub fn from_str(xml: &str) -> Result<SpaceSystem, xsd_parser_types::quick_xml::Error> {
    let mut reader = SliceReader::new(xml);
    SpaceSystem::deserialize(&mut reader)
}

/// Encodes an XTCE 1.3 document using the generated serializer.
pub fn to_string(value: &SpaceSystem) -> Result<String, xsd_parser_types::quick_xml::Error> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    value.serialize("xtce:SpaceSystem", &mut writer)?;

    Ok(String::from_utf8(writer.into_inner())
        .expect("xsd-parser generated non-UTF-8 XML from UTF-8 input"))
}
