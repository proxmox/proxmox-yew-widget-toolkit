use std::rc::Rc;

use derivative::Derivative;
use serde_json::Value;

use yew::Html;

use crate::widget::form::FormContext;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct RenderFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&T) -> Html>
);

impl<T> RenderFn<T> {
    /// Creates a new [`RenderFn`]
    pub fn new(renderer: impl 'static + Fn(&T) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, data: &T) -> Html {
        (self.0)(data)
    }
}

impl<T, F: 'static + Fn(&T) -> Html> From<F> for RenderFn<T> {
    fn from(f: F) -> Self {
        RenderFn::new(f)
    }
}


/// For use with KVGrid
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct RenderRecordFn(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&str, &Value, &Value) -> Html>
);

impl RenderRecordFn {
    /// Creates a new [`RenderFn`]
    pub fn new(renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, name: &str, value: &Value, record: &Value) -> Html {
        (self.0)(name, value, record)
    }
}


/// For use with ObjectGrid
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct RenderItemFn(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&FormContext, &str, &Value, &Value) -> Html>
);

impl RenderItemFn {
    /// Creates a new [`RenderFn`]
    pub fn new(renderer: impl 'static + Fn(&FormContext, &str, &Value, &Value) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, form_state: &FormContext, name: &str, value: &Value, record: &Value) -> Html {
        (self.0)(form_state, name, value, record)
    }
}
