use std::rc::Rc;
use std::future::Future;
use std::pin::Pin;

use anyhow::Error;
use serde_json::Value;
use derivative::Derivative;

use super::FormContext;

#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct SubmitCallback(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(FormContext) -> Pin<Box<dyn Future<Output=Result<Value, Error>>>>>
);

impl SubmitCallback {

    pub fn new<F, R>(callback: F) -> Self
    where
        F: 'static + Fn(FormContext) -> R,
        R: 'static + Future<Output = Result<Value, Error>>,
    {
        Self(Rc::new(move |state: FormContext| {
            let future = callback(state);
            Box::pin(future)
        }))
    }

    pub async fn apply(&self, form_ctx: FormContext) -> Result<Value, Error> {
        (self.0)(form_ctx).await
    }
}

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
    R: 'static + Future<Output = Result<Value, Error>>
{
    fn into_submit_callback(self) -> Option<SubmitCallback> {
        Some(SubmitCallback::new(self))
    }
}
