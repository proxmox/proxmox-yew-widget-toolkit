use std::rc::Rc;
use std::future::Future;
use std::pin::Pin;

use anyhow::Error;
use serde_json::Value;

use crate::widget::form::FormContext;

pub struct SubmitCallback(Rc<dyn Fn(FormContext) -> Pin<Box<dyn Future<Output=Result<Value, Error>>>>>);

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

impl Clone for SubmitCallback {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl PartialEq for SubmitCallback {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
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
