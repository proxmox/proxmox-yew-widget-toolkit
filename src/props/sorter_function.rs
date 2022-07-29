use std::rc::Rc;
use std::cmp::Ordering;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
pub struct SorterFn<T>(Rc<dyn Fn(&T, &T) -> Ordering>);

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

impl<T> Clone for SorterFn<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for SorterFn<T> {
    fn eq(&self, other: &Self) -> bool {
        // https://github.com/rust-lang/rust-clippy/issues/6524
        // #[allow(clippy::vtable_address_comparisons)]
        Rc::ptr_eq(&self.0, &other.0)
    }
}

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
