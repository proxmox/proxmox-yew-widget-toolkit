use std::rc::Rc;

use anyhow::Error;

/// A [ValidateFn] function is a callback that detrermines if the
/// passed record is valid.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[allow(clippy::type_complexity)]
pub struct ValidateFn<T>(Rc<dyn Fn(&T) -> Result<(), Error>>);

/// Create a thread_local, static validation function.
///
/// The value is initialized once and gets never updated.
#[macro_export]
macro_rules! static_validation_fn {
    ($t:ty, $v:expr) => {{
        thread_local! {
            static STATIC_FN: $crate::widget::form::ValidateFn<$t> = $crate::widget::form::ValidateFn::new($v);
        }
        STATIC_FN.with($crate::widget::form::ValidateFn::clone)
    }}
}

impl<T> Clone for ValidateFn<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for ValidateFn<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> ValidateFn<T> {
    /// Creates a new [`ValidateFn`]
    pub fn new(validate: impl 'static + Fn(&T) -> Result<(), Error>) -> Self {
        Self(Rc::new(validate))
    }

    /// Apply the validation function
    pub fn apply(&self, data: &T) -> Result<(), Error> {
        (self.0)(data)
    }
}

impl<T> std::fmt::Debug for ValidateFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValidateFn({:p})", self.0)
    }
}

impl<T, F: 'static + Fn(&T) -> Result<(), Error>> From<F> for ValidateFn<T> {
    fn from(f: F) -> Self {
        ValidateFn::new(f)
    }
}

/// Helper trait to create an optional [ValidateFn] property.
pub trait IntoValidateFn<T> {
    fn into_validate_fn(self) -> Option<ValidateFn<T>>;
}

impl<T, V: Into<ValidateFn<T>>> IntoValidateFn<T> for V {
    fn into_validate_fn(self) -> Option<ValidateFn<T>> {
        Some(self.into())
    }
}
impl<T, V: Into<ValidateFn<T>>> IntoValidateFn<T> for Option<V> {
    fn into_validate_fn(self) -> Option<ValidateFn<T>> {
        self.map(|v| v.into())
    }
}
