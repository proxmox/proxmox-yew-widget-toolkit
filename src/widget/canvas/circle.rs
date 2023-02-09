use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<circle>` element.
#[widget(pwt=crate, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Circle {}

impl Circle {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the circle position.
    pub fn position(mut self, cx: impl Into<SvgLength>, cy: impl Into<SvgLength>) -> Self {
        self.set_position(cx, cy);
        self
    }

    /// Method to set the circle position.
    pub fn set_position(&mut self, cx: impl Into<SvgLength>, cy: impl Into<SvgLength>) {
        self.set_attribute("cx", cx.into());
        self.set_attribute("cy", cy.into());
    }

    /// Builder style method to set the circle radius.
    pub fn radius(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_radius(r);
        self
    }

    /// Method to set the circle radius.
    pub fn set_radius(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("r", r.into());
    }

    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Circle {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("circle"), Some(self.listeners), None)
    }
}
