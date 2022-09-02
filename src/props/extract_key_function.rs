use std::rc::Rc;

use derivative::Derivative;
use yew::virtual_dom::Key;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct ExtractKeyFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&T) -> Key>
);

impl<T> ExtractKeyFn<T> {
    /// Creates a new [`ExtractKeyFn`]
    pub fn new(get_key: impl 'static + Fn(&T) -> Key) -> Self {
        Self(Rc::new(get_key))
    }
    /// Apply the render function
    pub fn apply(&self, data: &T) -> Key {
        (self.0)(data)
    }
}

impl<T, F: 'static + Fn(&T) -> Key> From<F> for ExtractKeyFn<T> {
    fn from(f: F) -> Self {
        ExtractKeyFn::new(f)
    }
}
