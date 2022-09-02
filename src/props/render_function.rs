use std::rc::Rc;

use derivative::Derivative;

use yew::Html;

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

