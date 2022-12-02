use std::rc::Rc;

use derivative::Derivative;

/// Callback wich takes a mutable argument.
///
/// Like [yew::Callback], but gets a mutable ref as argument.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct CallbackMut<T: 'static>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut T)>
);

impl<T: 'static> CallbackMut<T> {
    /// Creates a new instance.
    pub fn new(cb: impl 'static + Fn(&mut T)) -> Self {
        Self(Rc::new(cb))
    }
    /// Emit the callback.
    pub fn emit(&self, arg: &mut T) {
        (self.0)(arg);
    }
}

impl<T: 'static, F: 'static + Fn(&mut T)> From<F> for CallbackMut<T> {
    fn from(f: F) -> Self {
        CallbackMut::new(f)
    }
}

pub trait IntoEventCallbackMut<T: 'static> {
    fn into_event_cb_mut(self) -> Option<CallbackMut<T>>;
}

impl<T: 'static> IntoEventCallbackMut<T> for Option<CallbackMut<T>> {
    fn into_event_cb_mut(self) -> Option<CallbackMut<T>> {
        self
    }
}

impl<T: 'static, F: 'static + Fn(&mut T)> IntoEventCallbackMut<T> for F {
    fn into_event_cb_mut(self) -> Option<CallbackMut<T>> {
        Some(CallbackMut::new(self))
    }
}
