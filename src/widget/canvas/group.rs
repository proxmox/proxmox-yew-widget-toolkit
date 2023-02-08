use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG group (`<g>`) element.
#[widget(pwt=crate, @element, @container)]
#[derive(Properties, Clone, PartialEq)]
pub struct Group {}

impl Group {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the group stroke width.
    pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_stroke_width(w);
        self
    }

    /// Method to set the group stroke width.
    pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("stroke-width", w.into());
    }

    /// Builder style method to set the group stroke color/pattern.
    pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
        self.set_stroke(stroke);
        self
    }

    /// Method to set the group stroke color/pattern.
    pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
        self.set_attribute("stroke", stroke.into());
    }

    /// Builder style method to set the group fill color/pattern.
    pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
        self.set_fill(fill);
        self
    }

    /// Method to set the group fill color/pattern.
    pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
        self.set_attribute("fill", fill.into());
    }
}

impl Into<VTag> for Group {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("g"), Some(self.listeners), Some(self.children))
    }
}
