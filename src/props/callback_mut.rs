use std::rc::Rc;

use derivative::Derivative;

use yew::BaseComponent;
use yew::html::Scope;

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

impl<T: 'static> IntoEventCallbackMut<T> for CallbackMut<T> {
    fn into_event_cb_mut(self) -> Option<CallbackMut<T>> {
        Some(self)
    }
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

/// Extension trait which adds callback_mut() helper to [Scope].
pub trait CallbackMutScopeExt<COMP: BaseComponent> {
    fn callback_mut<F, T, M>(&self, function: F) -> CallbackMut<T>
    where M: Into<COMP::Message>,
          F: Fn(&mut T) -> M + 'static;
}

impl<COMP: BaseComponent> CallbackMutScopeExt<COMP> for Scope<COMP> {
    fn callback_mut<F, T, M>(&self, function: F) -> CallbackMut<T>
    where M: Into<COMP::Message>,
          F: Fn(&mut T) -> M + 'static,
    {
        let scope = self.clone();
        let closure = move |input: &mut T| {
            let output = function(input);
            scope.send_message(output);
        };
        CallbackMut::from(closure)
    }
}
