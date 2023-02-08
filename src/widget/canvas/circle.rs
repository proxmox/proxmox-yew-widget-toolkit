use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG circle element.
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

    /// Builder style method to set the circle stroke width.
    pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_stroke_width(w);
        self
    }

    /// Method to set the circle stroke width.
    pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("stroke-width", w.into());
    }

    /// Builder style method to set the circle stroke color/pattern.
    pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
        self.set_stroke(stroke);
        self
    }

    /// Method to set the circle stroke color/pattern.
    pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
        self.set_attribute("stroke", stroke.into());
    }

    /// Builder style method to set the circle fill color/pattern.
    pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
        self.set_fill(fill);
        self
    }

    /// Method to set the circle fill color/pattern.
    pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
        self.set_attribute("fill", fill.into());
    }
}

impl Into<VTag> for Circle {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("circle"), Some(self.listeners), None)
    }
}
