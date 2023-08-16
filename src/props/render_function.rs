use std::rc::Rc;

use derivative::Derivative;

use yew::Html;

/// A [RenderFn] function is a callback that transforms data into [Html].
///
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

/// Helper trait to create an optional [RenderFn] property.
pub trait IntoOptionalRenderFn<T> {
    fn into_optional_render_fn(self) -> Option<RenderFn<T>>;
}

impl<T> IntoOptionalRenderFn<T> for Option<RenderFn<T>> {
    fn into_optional_render_fn(self) -> Option<RenderFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T) -> Html> IntoOptionalRenderFn<T> for F {
    fn into_optional_render_fn(self) -> Option<RenderFn<T>> {
        Some(RenderFn::new(self))
    }
}

/// A [TextRenderFn] function is a callback that transforms data into [String].
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct TextRenderFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&T) -> String>
);

impl<T> TextRenderFn<T> {
    /// Creates a new [TextRenderFn]
    pub fn new(renderer: impl 'static + Fn(&T) -> String) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, data: &T) -> String {
        (self.0)(data)
    }
}

impl<T, F: 'static + Fn(&T) -> String> From<F> for TextRenderFn<T> {
    fn from(f: F) -> Self {
        TextRenderFn::new(f)
    }
}

/// Helper trait to create an optional [TextRenderFn] property.
pub trait IntoOptionalTextRenderFn<T> {
    fn into_optional_text_render_fn(self) -> Option<TextRenderFn<T>>;
}

impl<T> IntoOptionalTextRenderFn<T> for Option<TextRenderFn<T>> {
    fn into_optional_text_render_fn(self) -> Option<TextRenderFn<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&T) -> String> IntoOptionalTextRenderFn<T> for F {
    fn into_optional_text_render_fn(self) -> Option<TextRenderFn<T>> {
        Some(TextRenderFn::new(self))
    }
}

/// A [BuilderFn] function is a callback that returns a generic type.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct BuilderFn<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn() -> T>
);

impl<T: Into<Html>> Into<Html> for BuilderFn<T> {
    fn into(self) -> Html {
        self.apply().into()
     }
}

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

/// Helper trait to create an optional [BuilderFn] property.
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
