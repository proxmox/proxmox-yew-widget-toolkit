use std::rc::Rc;
use serde_json::Value;

use yew::Html;

use crate::state::FormState;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
pub struct RenderFn<T>(Rc<dyn Fn(&T) -> Html>);

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

impl<T> Clone for RenderFn<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for RenderFn<T> {
    fn eq(&self, other: &Self) -> bool {
        // https://github.com/rust-lang/rust-clippy/issues/6524
        // #[allow(clippy::vtable_address_comparisons)]
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// For use with KVGrid
pub struct RenderRecordFn(Rc<dyn Fn(&str, &Value, &Value) -> Html>);

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

impl Clone for RenderRecordFn {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl PartialEq for RenderRecordFn {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}


/// For use with ObjectGrid
pub struct RenderItemFn(Rc<dyn Fn(&FormState, &str, &Value, &Value) -> Html>);

impl RenderItemFn {
    /// Creates a new [`RenderFn`]
    pub fn new(renderer: impl 'static + Fn(&FormState, &str, &Value, &Value) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, form_state: &FormState, name: &str, value: &Value, record: &Value) -> Html {
        (self.0)(form_state, name, value, record)
    }
}

impl Clone for RenderItemFn {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl PartialEq for RenderItemFn {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
