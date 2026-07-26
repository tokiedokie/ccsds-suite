#![allow(clippy::all)]
#![allow(missing_docs)]

use xsd_parser_types::quick_xml::{DeserializeSync, SerializeSync, SliceReader, Writer};

#[allow(warnings)]
pub mod generated {
    pub type OperatorString = String;

    include!("generated.rs");
}

pub use generated::*;

pub const XTCE_NAMESPACE: &str = "http://www.omg.org/spec/XTCE/20250214";

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
