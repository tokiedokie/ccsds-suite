use gpui::{App, Div, WeakEntity};

use super::collection_summary;
use crate::XtceEditor;

pub(super) struct ServiceSetForm;

impl ServiceSetForm {
    pub(super) fn render(
        &self,
        service_set: Option<&xtce::ServiceSetType>,
        editor: WeakEntity<XtceEditor>,
        cx: &App,
    ) -> Div {
        let rows = service_set
            .into_iter()
            .flat_map(|set| &set.service)
            .map(|service| {
                vec![
                    service.name.clone(),
                    service.short_description.clone().unwrap_or_default(),
                ]
            })
            .collect::<Vec<_>>();
        let targets = vec![None; rows.len()];
        collection_summary(
            &["Name", "Short description"],
            rows,
            targets,
            editor,
            "No services are defined.",
            cx,
        )
    }
}
