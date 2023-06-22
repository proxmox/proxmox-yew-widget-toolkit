use yew::prelude::*;

use super::ScaffoldController;


pub trait ScaffoldContextExt {
    /// Access the [ScaffoldController] from the [ContextProvider].
    fn scaffold_controller(&self) -> Option<ScaffoldController>;

    /// Show/hide the drawer.
    fn show_drawer(&self, show: bool) {
        match self.scaffold_controller() {
            Some(controller) => controller.show_drawer(show),
            None => {
                log::error!(
                    "unable to show drawer: context does not provide a ScaffoldController."
                );
            }
        }
    }

    /// Toggle drawer between show and hide.
    fn toggle_drawer(&self) {
        match self.scaffold_controller() {
            Some(controller) => controller.toggle_drawer(),
            None => {
                log::error!(
                    "unable to toggle drawer: context does not provide a ScaffoldController."
                );
            }
        }
    }

    /// Show/hide the end drawer.
    fn show_end_drawer(&self, show: bool) {
        match self.scaffold_controller() {
            Some(controller) => controller.show_end_drawer(show),
            None => {
                log::error!(
                    "unable to show end drawer: context does not provide a ScaffoldController."
                );
            }
        }
    }

    /// Toggle end drawer between show and hide.
    fn toggle_end_drawer(&self) {
        match self.scaffold_controller() {
            Some(controller) => controller.toggle_end_drawer(),
            None => {
                log::error!(
                    "unable to toggle end drawer: context does not provide a ScaffoldController."
                );
            }
        }
    }
}


impl<COMP: Component> ScaffoldContextExt for yew::html::Scope<COMP> {
    fn scaffold_controller(&self) -> Option<ScaffoldController> {
        self.context::<ScaffoldController>(Callback::from(|_| {}))
            .map(|(controller, _)| controller)
    }
}
