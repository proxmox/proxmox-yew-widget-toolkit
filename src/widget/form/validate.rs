use std::rc::Rc;

use anyhow::Error;

/// A [ValidateFn] function is a callback that detrermines if the
/// passed record is valid.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
pub struct ValidateFn<T>(Rc<dyn Fn(&T) -> Result<(), Error>>);

/// Create a thread_local, static validation function.
///
/// The value is initialized once and gets never updated.
#[macro_export]
macro_rules! static_validation_fn {
    ($t:ty, $v:expr) => {{
        thread_local! {
            static STATIC_FN: std::cell::OnceCell<$crate::widget::form::ValidateFn<$t>> = std::cell::OnceCell::new();
        }
        STATIC_FN.with(|cell| cell.get_or_init(|| $crate::widget::form::ValidateFn::new($v)).clone())
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

impl<T> IntoValidateFn<T> for ValidateFn<T> {
    fn into_validate_fn(self) -> Option<ValidateFn<T>> {
        Some(self)
    }
}
impl<T> IntoValidateFn<T> for Option<ValidateFn<T>> {
    fn into_validate_fn(self) -> Option<ValidateFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T)-> Result<(), Error>> IntoValidateFn<T> for F {
    fn into_validate_fn(self) -> Option<ValidateFn<T>> {
        Some(ValidateFn::new(self))
    }
}
