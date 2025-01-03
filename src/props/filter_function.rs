use std::rc::Rc;

use derivative::Derivative;

/// A [FilterFn] function is a callback that determine if an element
/// should be yielded.
///
/// Given an element the callback must return true or false. Only the
/// elements for which the callback returns true are yielded.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct FilterFn<T>(
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))] Rc<dyn Fn(&T) -> bool>,
);

impl<T> FilterFn<T> {
    /// Creates a new [`FilterFn`]
    pub fn new(filter: impl 'static + Fn(&T) -> bool) -> Self {
        Self(Rc::new(filter))
    }
    /// Apply the filter function
    pub fn apply(&self, item: &T) -> bool {
        (self.0)(item)
    }
}

/// Helper trait to create an optional [FilterFn] property.
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

impl<T, F: 'static + Fn(&T) -> bool> IntoFilterFn<T> for F {
    fn into_filter_fn(self) -> Option<FilterFn<T>> {
        Some(FilterFn::new(self))
    }
}

/// A [TextFilterFn] function is a callback that determine if an element
/// should be yielded. The filter query is passed as second argument.
///
/// Given an element the callback must return true or false. Only the
/// elements for which the callback returns true are yielded.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct TextFilterFn<T>(
    #[allow(clippy::type_complexity)]
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    Rc<dyn Fn(&T, &str) -> bool>,
);

impl<T> TextFilterFn<T> {
    /// Creates a new [`TextFilterFn`]
    pub fn new(filter: impl 'static + Fn(&T, &str) -> bool) -> Self {
        Self(Rc::new(filter))
    }
    /// Apply the filter function
    pub fn apply(&self, item: &T, query: &str) -> bool {
        (self.0)(item, query)
    }
}

/// Helper trait to create an optional [TextFilterFn] property.
pub trait IntoTextFilterFn<T> {
    fn into_text_filter_fn(self) -> Option<TextFilterFn<T>>;
}

impl<T> IntoTextFilterFn<T> for TextFilterFn<T> {
    fn into_text_filter_fn(self) -> Option<TextFilterFn<T>> {
        Some(self)
    }
}

impl<T> IntoTextFilterFn<T> for Option<TextFilterFn<T>> {
    fn into_text_filter_fn(self) -> Option<TextFilterFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T, &str) -> bool> IntoTextFilterFn<T> for F {
    fn into_text_filter_fn(self) -> Option<TextFilterFn<T>> {
        Some(TextFilterFn::new(self))
    }
}
