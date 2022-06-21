use std::rc::Rc;

use yew::virtual_dom::Key;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
pub struct ExtractKeyFn<T>(Rc<dyn Fn(&T) -> Key>);

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

impl<T> Clone for ExtractKeyFn<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for ExtractKeyFn<T> {
    fn eq(&self, other: &Self) -> bool {
        // https://github.com/rust-lang/rust-clippy/issues/6524
        // #[allow(clippy::vtable_address_comparisons)]
        Rc::ptr_eq(&self.0, &other.0)
    }
}
