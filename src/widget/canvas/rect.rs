use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<rect>` element.
#[widget(pwt=crate, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Rect {}

impl Rect {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the rect position.
    pub fn position(mut self, cx: impl Into<SvgLength>, cy: impl Into<SvgLength>) -> Self {
        self.set_position(cx, cy);
        self
    }

    /// Method to set the rect position.
    pub fn set_position(&mut self, cx: impl Into<SvgLength>, cy: impl Into<SvgLength>) {
        self.set_attribute("x", cx.into());
        self.set_attribute("y", cy.into());
    }

    /// Builder style method to set the rect width.
    pub fn width(mut self, width: impl Into<SvgLength>) -> Self {
        self.set_width(width);
        self
    }

    /// Method to set the rect width.
    pub fn set_width(&mut self, width: impl Into<SvgLength>) {
        self.set_attribute("width", width.into());
    }

    /// Builder style method to set the rect height.
    pub fn height(mut self, height: impl Into<SvgLength>) -> Self {
        self.set_height(height);
        self
    }

    /// Method to set the rect height.
    pub fn set_height(&mut self, height: impl Into<SvgLength>) {
        self.set_attribute("height", height.into());
    }

    /// Builder style method to set the horizontal corner radius of the rect.
    pub fn rx(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_rx(r);
        self
    }

    /// Method to set the horizontal corner radius of the rect.
    pub fn set_rx(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("rx", r.into());
    }

    /// Builder style method to set the vertical corner radius of the rect.
    pub fn ry(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_ry(r);
        self
    }

    /// Method to set the vertical corner radius of the rect.
    pub fn set_ry(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("ry", r.into());
    }

    /// Builder style method to set the rect stroke width.
    pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_stroke_width(w);
        self
    }

    /// Method to set the rect stroke width.
    pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("stroke-width", w.into());
    }

    /// Builder style method to set the rect stroke color/pattern.
    pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
        self.set_stroke(stroke);
        self
    }

    /// Method to set the rect stroke color/pattern.
    pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
        self.set_attribute("stroke", stroke.into());
    }

    /// Builder style method to set the rect fill color/pattern.
    pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
        self.set_fill(fill);
        self
    }

    /// Method to set the rect fill color/pattern.
    pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
        self.set_attribute("fill", fill.into());
    }
}

impl Into<VTag> for Rect {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("rect"), Some(self.listeners), None)
    }
}
