use std::rc::Rc;
use std::cmp::Ordering;

use derivative::Derivative;

/// A [SorterFn] function is a callback that determines record ordering.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct SorterFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&T, &T) -> Ordering>
);

impl<T> SorterFn<T> {
    /// Creates a new [`SorterFn`]
    pub fn new(sorter: impl 'static + Fn(&T, &T) -> Ordering) -> Self {
        Self(Rc::new(sorter))
    }
    /// Apply the sorter function
    pub fn cmp(&self, a: &T, b: &T) -> Ordering {
        (self.0)(a, b)
    }
}

/// Helper trait to create an optional [SorterFn] property.
pub trait IntoSorterFn<T> {
    fn into_sorter_fn(self) -> Option<SorterFn<T>>;
}

impl<T> IntoSorterFn<T> for SorterFn<T> {
    fn into_sorter_fn(self) -> Option<SorterFn<T>> {
        Some(self)
    }
}

impl<T> IntoSorterFn<T> for Option<SorterFn<T>> {
    fn into_sorter_fn(self) -> Option<SorterFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T, &T) -> Ordering> IntoSorterFn<T>  for F {
    fn into_sorter_fn(self) -> Option<SorterFn<T>> {
        Some(SorterFn::new(self))
    }
}
