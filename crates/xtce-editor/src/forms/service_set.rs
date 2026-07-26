use gpui::{App, Div};

use super::property_rows;

pub(super) struct ServiceSetForm;

impl ServiceSetForm {
    pub(super) fn render(&self, service_set: Option<&xtce::ServiceSetType>, cx: &App) -> Div {
        let services = service_set
            .map(|set| {
                set.service
                    .iter()
                    .map(|service| service.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let count = service_set.map_or(0, |set| set.service.len());
        property_rows(
            vec![("Services", count.to_string()), ("Names", services)],
            cx,
        )
    }
}
