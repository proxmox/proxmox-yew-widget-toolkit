use std::rc::Rc;

use derivative::Derivative;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct FilterFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&T) -> bool>
);

impl<T> FilterFn<T> {
    /// Creates a new [`FilterFn`]
    pub fn new(sorter: impl 'static + Fn(&T) -> bool) -> Self {
        Self(Rc::new(sorter))
    }
    /// Apply the filter function
    pub fn apply(&self, item: &T) -> bool {
        (self.0)(item)
    }
}

pub trait IntoFilterFn<T> {
    fn into_filter_fn(self) -> Option<FilterFn<T>>;
}

impl<T> IntoFilterFn<T> for FilterFn<T> {
    fn into_filter_fn(self) -> Option<FilterFn<T>> {
        Some(self)
    }
}

impl<T> IntoFilterFn<T> for Option<FilterFn<T>> {
    fn into_filter_fn(self) -> Option<FilterFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T) -> bool> IntoFilterFn<T>  for F {
    fn into_filter_fn(self) -> Option<FilterFn<T>> {
        Some(FilterFn::new(self))
    }
}
