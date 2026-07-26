use gpui::{App, Context, Div, Window};

use super::{
    space_system_description::SpaceSystemDescriptionFields,
    space_system_identity::SpaceSystemIdentityFields,
};
use crate::XtceEditor;

pub(crate) struct SpaceSystemForm {
    identity: SpaceSystemIdentityFields,
    description: SpaceSystemDescriptionFields,
}

impl SpaceSystemForm {
    pub(crate) fn new(
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        Self {
            identity: SpaceSystemIdentityFields::new(system, window, cx),
            description: SpaceSystemDescriptionFields::new(system, window, cx),
        }
    }

    pub(crate) fn load(
        &self,
        system: &xtce::SpaceSystem,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        self.identity.load(system, window, cx);
        self.description.load(system, window, cx);
    }

    pub(crate) fn apply_to(&self, system: &mut xtce::SpaceSystem, cx: &App) {
        self.identity.apply_to(system, cx);
        self.description.apply_to(system, cx);
    }

    pub(crate) fn render_name_editor(&self) -> Div {
        self.identity.render_name_editor()
    }

    pub(crate) fn name(&self, cx: &App) -> String {
        self.identity.name(cx)
    }

    pub(crate) fn render_identity(&self, cx: &App) -> Div {
        self.identity.render(cx)
    }

    pub(crate) fn render_description(&self, cx: &App) -> Div {
        self.description.render(cx)
    }
}
