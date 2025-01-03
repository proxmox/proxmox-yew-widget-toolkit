use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

/// A [SubmitValidateFn] function is a callback that detrermines if the
/// passed record is valid, and return the value to be submitted.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[allow(clippy::type_complexity)]
pub struct SubmitValidateFn<T>(Rc<dyn Fn(&T) -> Result<Value, Error>>);

impl<T> Clone for SubmitValidateFn<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for SubmitValidateFn<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> SubmitValidateFn<T> {
    /// Creates a new [`SubmitValidateFn`]
    pub fn new(validate: impl 'static + Fn(&T) -> Result<Value, Error>) -> Self {
        Self(Rc::new(validate))
    }

    /// Apply the validation function
    pub fn apply(&self, data: &T) -> Result<Value, Error> {
        (self.0)(data)
    }
}

impl<T> std::fmt::Debug for SubmitValidateFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubmitValidateFn({:p})", self.0)
    }
}

impl<T, F: 'static + Fn(&T) -> Result<Value, Error>> From<F> for SubmitValidateFn<T> {
    fn from(f: F) -> Self {
        SubmitValidateFn::new(f)
    }
}

/// Helper trait to create an optional [SubmitValidateFn] property.
pub trait IntoSubmitValidateFn<T> {
    fn into_submit_validate_fn(self) -> Option<SubmitValidateFn<T>>;
}

impl<T, V: Into<SubmitValidateFn<T>>> IntoSubmitValidateFn<T> for V {
    fn into_submit_validate_fn(self) -> Option<SubmitValidateFn<T>> {
        Some(self.into())
    }
}
impl<T, V: Into<SubmitValidateFn<T>>> IntoSubmitValidateFn<T> for Option<V> {
    fn into_submit_validate_fn(self) -> Option<SubmitValidateFn<T>> {
        self.map(|v| v.into())
    }
}
