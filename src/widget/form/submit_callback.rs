use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

use super::FormContext;

/// A [SubmitCallback] is an async callback ([Future]) that gets the
/// [FormContext] as parameter, returning the [Result] of the submit
/// opertation.
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct SubmitCallback(
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    Rc<dyn Fn(FormContext) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>>,
);

impl SubmitCallback {
    pub fn new<F, R>(callback: F) -> Self
    where
        F: 'static + Fn(FormContext) -> R,
        R: 'static + Future<Output = Result<(), Error>>,
    {
        Self(Rc::new(move |state: FormContext| {
            let future = callback(state);
            Box::pin(future)
        }))
    }

    pub async fn apply(&self, form_ctx: FormContext) -> Result<(), Error> {
        (self.0)(form_ctx).await
    }
}

/// Helper trait to create an optional [SubmitCallback] property.
pub trait IntoSubmitCallback {
    fn into_submit_callback(self) -> Option<SubmitCallback>;
}

impl IntoSubmitCallback for Option<SubmitCallback> {
    fn into_submit_callback(self) -> Option<SubmitCallback> {
        self
    }
}

impl<F, R> IntoSubmitCallback for F
where
    F: 'static + Fn(FormContext) -> R,
    R: 'static + Future<Output = Result<(), Error>>,
{
    fn into_submit_callback(self) -> Option<SubmitCallback> {
        Some(SubmitCallback::new(self))
    }
}
