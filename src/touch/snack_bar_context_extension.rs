use yew::prelude::*;

use super::{SnackBar, SnackBarController};

/// Simlify access to [SnackBarController] via [ContextProvider]
///
/// Widget like [MaterialApp](crate::touch::MaterialApp) use the [ContextProvider] to expose
/// a [SnackBarController]. This class simplifies access to that controller.
///
/// The following example shows how to emit a message from inside a button click handler.
/// ```
/// use pwt::prelude::*;
/// use pwt::touch::{SnackBar, SnackBarContextExt};
/// use pwt::widget::Button;
///
/// # fn test(ctx: &Context<pwt::widget::PwtButton>) { // fake context for testing
///
/// // assume we have access to a component context in 'ctx' ...
///
/// let button = Button::new("Show Snackbar")
///     .onclick({
///         let link = ctx.link().clone();
///         move |_| {
///             link.show_snackbar(SnackBar::new().message("This is a test message."));
///         }
///     });
/// # }
///
/// ```
pub trait SnackBarContextExt {
    /// Access the [SnackBarController] from the [ContextProvider].
    fn snackbar_controller(&self) -> Option<SnackBarController>;

    /// Convenient way to show a snackbar.
    ///
    /// # Note
    ///
    /// Only works if the context provides a [SnackBarController].
    fn show_snackbar(&self, snackbar: SnackBar) -> Option<AttrValue> {
        match self.snackbar_controller() {
            Some(controller) => Some(controller.show_snackbar(snackbar)),
            None => {
                log::error!(
                    "unable to show snackbar: context does not provide a SnackBarController."
                );
                None
            }
        }
    }
}

impl<COMP: Component> SnackBarContextExt for yew::html::Scope<COMP> {
    fn snackbar_controller(&self) -> Option<SnackBarController> {
        self.context::<SnackBarController>(Callback::from(|_| {}))
            .map(|(controller, _)| controller)
    }
}
