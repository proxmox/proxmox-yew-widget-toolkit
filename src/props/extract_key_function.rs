use std::rc::Rc;

use derivative::Derivative;
use yew::virtual_dom::Key;

/// Primary Key extraction
///
/// The [super::Store] has a simplified interface for types
/// implementing this trait.
pub trait ExtractPrimaryKey {
    fn extract_key(&self) -> Key;
}

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

pub trait IntoExtractKeyFn<T> {
    fn into_extract_key_fn(self) -> Option<ExtractKeyFn<T>>;
}

impl<T> IntoExtractKeyFn<T> for ExtractKeyFn<T> {
    fn into_extract_key_fn(self) -> Option<ExtractKeyFn<T>> {
        Some(self)
    }
}

impl<T> IntoExtractKeyFn<T> for Option<ExtractKeyFn<T>> {
    fn into_extract_key_fn(self) -> Option<ExtractKeyFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T) -> Key> IntoExtractKeyFn<T>  for F {
    fn into_extract_key_fn(self) -> Option<ExtractKeyFn<T>> {
        Some(ExtractKeyFn::new(self))
    }
}
