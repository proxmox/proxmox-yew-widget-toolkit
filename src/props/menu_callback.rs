use std::rc::Rc;

use derivative::Derivative;

/// Callback argument for menu events.
pub struct MenuEvent {
    /// MenuCheckbox sets this flag
    pub checked: bool,
}

impl MenuEvent {

    pub fn new() -> Self {
        Self {
            checked: false,
        }
    }
}

/// Menu Callback
///
/// The menu is kept open if the callback return `true`.
// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct MenuCallback(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(MenuEvent) -> bool>
);

impl MenuCallback {
    /// Creates a new [`MenuCallback`]
    pub fn new(cb: impl 'static + Fn(MenuEvent) -> bool) -> Self {
        Self(Rc::new(cb))
    }
    /// This method calls the callback’s function.
    pub fn emit(&self, event: MenuEvent) -> bool {
        (self.0)(event)
    }
}


pub trait IntoMenuCallback {
    fn into_menu_callback(self) -> Option<MenuCallback>;
}

impl IntoMenuCallback for MenuCallback {
    fn into_menu_callback(self) -> Option<MenuCallback> {
        Some(self)
    }
}

impl IntoMenuCallback for Option<MenuCallback> {
    fn into_menu_callback(self) -> Option<MenuCallback> {
        self
    }
}

impl<F: 'static + Fn(MenuEvent) -> bool> IntoMenuCallback  for F {
    fn into_menu_callback(self) -> Option<MenuCallback> {
        Some(MenuCallback::new(self))
    }
}
