use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

/// A [SubmitCallback] is an async callback ([Future]) that gets the
/// data to be submitted as parameter, returning the [Result] of the submit
/// operation.
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct SubmitCallback<T>(
    #[allow(clippy::type_complexity)]
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    Rc<dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>>,
);

impl<T> SubmitCallback<T> {
    pub fn new<F, R>(callback: F) -> Self
    where
        F: 'static + Fn(T) -> R,
        R: 'static + Future<Output = Result<(), Error>>,
    {
        Self(Rc::new(move |state: T| {
            let future = callback(state);
            Box::pin(future)
        }))
    }

    pub async fn apply(&self, form_ctx: T) -> Result<(), Error> {
        (self.0)(form_ctx).await
    }
}

/// Helper trait to create an optional [SubmitCallback] property.
pub trait IntoSubmitCallback<T> {
    fn into_submit_callback(self) -> Option<SubmitCallback<T>>;
}

impl<T> IntoSubmitCallback<T> for Option<SubmitCallback<T>> {
    fn into_submit_callback(self) -> Option<SubmitCallback<T>> {
        self
    }
}

impl<F, R, T> IntoSubmitCallback<T> for F
where
    F: 'static + Fn(T) -> R,
    R: 'static + Future<Output = Result<(), Error>>,
{
    fn into_submit_callback(self) -> Option<SubmitCallback<T>> {
        Some(SubmitCallback::new(self))
    }
}
