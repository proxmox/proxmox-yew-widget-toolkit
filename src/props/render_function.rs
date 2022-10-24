use std::rc::Rc;

use derivative::Derivative;

use yew::Html;

use crate::state::DataNode;

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

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct BuilderFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn() -> T>
);

impl<T> BuilderFn<T> {
    /// Creates a new [`BuilderFn`]
    pub fn new(builder: impl 'static + Fn() -> T) -> Self {
        Self(Rc::new(builder))
    }
    /// Apply the builder function
    pub fn apply(&self) -> T {
        (self.0)()
    }
}

impl<T, F: 'static + Fn() -> T> From<F> for BuilderFn<T> {
    fn from(f: F) -> Self {
        BuilderFn::new(f)
    }
}

pub trait IntoOptionalBuilderFn<T> {
    fn into_optional_builder_fn(self) -> Option<BuilderFn<T>>;
}

impl<T> IntoOptionalBuilderFn<T> for Option<BuilderFn<T>> {
    fn into_optional_builder_fn(self) -> Option<BuilderFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn() -> T> IntoOptionalBuilderFn<T> for F {
    fn into_optional_builder_fn(self) -> Option<BuilderFn<T>> {
        Some(BuilderFn::new(self))
    }
}

// Note: RenderFn<dyn DataNode<T>> does not work (?Sized problems), so
// we define a separate render function.
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct RenderDataNode<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&dyn DataNode<T>) -> Html>
);

impl<T> RenderDataNode<T> {
    /// Creates a new [`RenderFn`]
    pub fn new(renderer: impl 'static + Fn(&dyn DataNode<T>) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, node: &dyn DataNode<T>) -> Html {
        (self.0)(node)
    }
}

impl<T, F: 'static + Fn(&dyn DataNode<T>) -> Html> From<F> for RenderDataNode<T> {
    fn from(f: F) -> Self {
        RenderDataNode::new(f)
    }
}
