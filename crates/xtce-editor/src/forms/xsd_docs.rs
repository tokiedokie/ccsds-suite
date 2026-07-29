use std::{collections::HashMap, sync::OnceLock};

const XTCE_XSD: &str = include_str!("../../../xtce/SpaceSystem.xsd");

static DOCUMENTATION: OnceLock<HashMap<String, String>> = OnceLock::new();

pub(super) fn documentation_for(label: &str) -> Option<&'static str> {
    let key = canonical_name(label);
    let documentation = DOCUMENTATION.get_or_init(build_documentation_index);
    documentation
        .get(&key)
        .or_else(|| alias_for(&key).and_then(|alias| documentation.get(alias)))
        .map(String::as_str)
}

fn build_documentation_index() -> HashMap<String, String> {
    let document =
        roxmltree::Document::parse(XTCE_XSD).expect("the bundled XTCE XSD must be valid XML");
    let mut candidates: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for node in document.descendants().filter(|node| node.is_element()) {
        let Some(name) = node.attribute("name") else {
            continue;
        };
        let Some(description) = nearest_documentation(node) else {
            continue;
        };
        *candidates
            .entry(canonical_name(name))
            .or_default()
            .entry(description)
            .or_default() += 1;
    }

    candidates
        .into_iter()
        .filter_map(|(name, descriptions)| {
            descriptions
                .into_iter()
                .max_by(|(left_text, left_count), (right_text, right_count)| {
                    left_count
                        .cmp(right_count)
                        .then_with(|| left_text.len().cmp(&right_text.len()))
                })
                .map(|(description, _)| (name, description))
        })
        .collect()
}

fn nearest_documentation(node: roxmltree::Node<'_, '_>) -> Option<String> {
    node.ancestors().find_map(|ancestor| {
        ancestor
            .children()
            .find(|child| child.is_element() && child.tag_name().name() == "annotation")
            .and_then(|annotation| {
                annotation.descendants().find(|descendant| {
                    descendant.is_element() && descendant.tag_name().name() == "documentation"
                })
            })
            .and_then(|documentation| {
                let text = documentation
                    .descendants()
                    .filter(|descendant| descendant.is_text())
                    .filter_map(|descendant| descendant.text())
                    .flat_map(str::split_whitespace)
                    .collect::<Vec<_>>()
                    .join(" ");
                (!text.is_empty()).then_some(text)
            })
    })
}

fn canonical_name(value: &str) -> String {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut previous_was_lowercase_or_digit = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && previous_was_lowercase_or_digit && !word.is_empty()
            {
                words.push(std::mem::take(&mut word));
            }
            word.push(character.to_ascii_lowercase());
            previous_was_lowercase_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        } else if !word.is_empty() {
            words.push(std::mem::take(&mut word));
            previous_was_lowercase_or_digit = false;
        }
    }
    if !word.is_empty() {
        words.push(word);
    }

    words
        .into_iter()
        .filter(|word| word != "in")
        .map(|word| match word.as_str() {
            "ref" => "reference".to_owned(),
            "min" => "minimum".to_owned(),
            "max" => "maximum".to_owned(),
            _ => word,
        })
        .collect::<Vec<_>>()
        .concat()
}

fn alias_for(key: &str) -> Option<&'static str> {
    Some(match key {
        "algorithm" | "algorithmidentity" | "algorithmmetadata" => "simplealgorithmtype",
        "algorithmlanguage" => "language",
        "aliases" => "aliassettype",
        "aliasnamespace" => "namespace",
        "arguments" => "argumentlisttype",
        "baseargumentassignments" => "argumentassignmentlisttype",
        "argumentrestrictions" => "argumentrestrictionlisttype",
        "authors" => "authorsettype",
        "basecontainerreference" => "basecontainer",
        "basemetacommandreference" => "basemetacommand",
        "bitdirection" => "bitorder",
        "calibratedvalue" | "parametervalue" => "value",
        "changevalue" => "changevaluetype",
        "checktolockframes" => "checktolockgoodframes",
        "compareusing" => "usecalibratedvalue",
        "comparisonvalue" => "value",
        "comparisons" => "comparisonlisttype",
        "connectingstreamreference" => "streamreference",
        "containername" => "containerreference",
        "conditiontype" => "commandverifiertype",
        "contextalarms" => "numericcontextalarmlisttype",
        "contextcalibrators" => "contextcalibratorlisttype",
        "content" => "ancillarydatatype",
        "criteria" | "criteriatype" => "comparison",
        "dataencoding" => "dataencodingtype",
        "defaultinterpolationorder" => "order",
        "defaultrate" => "ratestreamtype",
        "defaultsize" | "fixedsizebits" | "resultsize" | "segmentsize" => "sizebits",
        "dimensions" => "dimensionlist",
        "externalalgorithms" => "externalalgorithmsettype",
        "errordetectioncorrection" => "errordetectcorrect",
        "framesource" => "pcmstreamtype",
        "frombinarytransform" => "frombinarytransformalgorithm",
        "implementation" => "simplealgorithmtype",
        "initialremainder" => "initialvalue",
        "inputs" => "inputsettype",
        "maximumsyncbiterrors" => "maximumbiterrorssyncpattern",
        "maximum" | "maximumboundary" => "maximuminclusive",
        "commandverifiers" => "verifiersettype",
        "metacommandelement" => "metacommand",
        "metacommandsteps" => "metacommandsteplisttype",
        "method" => "errordetectcorrect",
        "minimum" | "minimumboundary" => "minimuminclusive",
        "nextcontainerreference" => "nextcontainer",
        "notes" => "notesettype",
        "operationname" => "name",
        "operationshortdescription" => "shortdescription",
        "offsetunit" => "unit",
        "numberformatting" => "tostringtype",
        "operationmetadata" => "mathoperationcalibratortype",
        "optionalmetadata" => "namedescriptiontype",
        "outputs" => "outputsettype",
        "patternbitlocation" => "bitlocationfromstartofcontainer",
        "patternmask" => "mask",
        "percentcompletesource" => "percentcompletetype",
        "physicaladdresses" => "physicaladdressset",
        "perstreamrates" => "ratestreamwithstreamnametype",
        "progresspercentage" => "verificationprogresspercentage",
        "rangeappliesto" | "valueform" => "calibrated",
        "ranges" => "staticalarmranges",
        "referencepoint" | "referencetargettype" => "reference",
        "returnparameterreference" => "returnparmreference",
        "rpnoperation" => "mathoperationtype",
        "sizetagwidthbits" => "sizebitsofsizetag",
        "sizesource" => "dynamicvalue",
        "spansamples" => "spanofinterestsamples",
        "spanseconds" => "spanofinterestseconds",
        "stage" => "verifierenumerationtype",
        "stagespecificoptions" => "commandverifiertype",
        "startcheckalgorithm" | "stoptimealgorithm" => "checkwindowalgorithmstype",
        "startcheckingafter" => "timetostartchecking",
        "stopcheckingafter" => "timetostopchecking",
        "terminationcharacter" => "terminationchar",
        "thousandsgrouping" => "showthousandsgrouping",
        "timeencoding" => "basetimedatatype",
        "timeparameterreference" => "parameterreference",
        "tokentype" => "mathoperation",
        "tobinarytransform" => "tobinarytransformalgorithm",
        "transmissionconstraints" => "transmissionconstraintlisttype",
        "triggers" => "triggersettype",
        "unittext" => "unit",
        "valuetype" => "usecalibratedvalue",
        "valuesource" => "argumentinstancereftype",
        "verificationtrigger" => "checkwindow",
        "verificationcondition" => "commandverifiertype",
        "verifytolockframes" => "verifytolockgoodframes",
        "windowrelativeto" => "timewindowisrelativetotype",
        "windowtype" => "checkwindowtype",
        "significancewhenmatched" => "significance",
        "synchronization" => "syncstrategytype",
        "xmlbase" => "spacesystemtype",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{canonical_name, documentation_for};

    #[test]
    fn canonicalizes_editor_labels_and_xsd_names_equally() {
        assert_eq!(
            canonical_name("Parameter type reference"),
            canonical_name("parameterTypeRef")
        );
        assert_eq!(canonical_name("Size in bits"), canonical_name("sizeInBits"));
    }

    #[test]
    fn returns_direct_xsd_documentation() {
        let documentation = documentation_for("Parameter type reference").unwrap();
        assert!(documentation.contains("ParameterTypeSet"));
    }

    #[test]
    fn falls_back_to_an_enclosing_xsd_definition() {
        assert!(documentation_for("Bad frames to auto-invert").is_some());
    }

    #[test]
    fn documentation_text_is_not_collected_twice() {
        let documentation = documentation_for("Bit order").unwrap();
        assert_eq!(
            documentation,
            "Describes the bit ordering of the encoded value."
        );
    }

    #[test]
    fn resolves_editor_specific_labels_to_xsd_definitions() {
        for label in [
            "Algorithm language",
            "Argument restrictions",
            "Authors",
            "Change value",
            "Condition type",
            "Content",
            "Data encoding",
            "External algorithms",
            "Inputs",
            "Maximum sync bit errors",
            "Meta command element",
            "Meta command steps",
            "Method",
            "Notes",
            "Outputs",
            "Pattern bit location",
            "Pattern mask",
            "Percent complete source",
            "Progress percentage",
            "Reference point",
            "Reference target type",
            "Return parameter reference",
            "Size tag width in bits",
            "Span in samples",
            "Span in seconds",
            "Stage",
            "Start checking after",
            "Stop checking after",
            "Termination character",
            "Thousands grouping",
            "Token type",
            "Triggers",
            "Value type",
            "Window relative to",
            "Window type",
        ] {
            assert!(
                documentation_for(label).is_some(),
                "missing XSD documentation for {label}"
            );
        }
    }

    #[test]
    fn resolves_section_titles_to_xsd_definitions() {
        let missing = [
            "Algorithm",
            "Algorithm identity",
            "Algorithm metadata",
            "Aliases",
            "Alarm conditions",
            "Ancillary data",
            "Arguments",
            "Auto invert",
            "Base argument assignments",
            "Binary encoding",
            "Calibrator",
            "Check window",
            "Command verifiers",
            "Comparisons",
            "Condition",
            "Context alarms",
            "Context calibrators",
            "Context significance",
            "Custom alarm",
            "Data encoding",
            "Decoding algorithm",
            "Default alarm",
            "Default calibrator",
            "Default rate",
            "Dimensions",
            "Encoding algorithm",
            "Entry list",
            "Enumeration list",
            "Error detection/correction",
            "Frame source",
            "From-binary transform",
            "Implementation",
            "Input algorithm",
            "Invert algorithm",
            "Leading size",
            "Linear adjustment",
            "Match criteria",
            "Member list",
            "Number formatting",
            "Operation metadata",
            "Operator",
            "Optional metadata",
            "Parameter to set list",
            "Parameters to suspend alarms on set",
            "Per-stream rates",
            "Reference time",
            "RPN operation",
            "Synchronization",
            "Significance when matched",
            "Stage-specific options",
            "Start-check algorithm",
            "Stop-time algorithm",
            "Termination character",
            "Time encoding",
            "To-binary transform",
            "Transmission constraints",
            "Trigger set",
            "Units",
            "Valid range",
            "Verification condition",
        ]
        .into_iter()
        .filter(|label| documentation_for(label).is_none())
        .collect::<Vec<_>>();
        assert!(missing.is_empty(), "missing XSD documentation: {missing:?}");
    }
}
