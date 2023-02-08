use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<line>` element.
#[widget(pwt=crate, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Line {}

impl Line {

    /// Create a new instance.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let mut me = yew::props!(Self {});

        me.set_attribute("x1", x1.to_string());
        me.set_attribute("y1", y1.to_string());
        me.set_attribute("x2", x2.to_string());
        me.set_attribute("y2", y2.to_string());
        
        me
    }

    /// Builder style method to set the line stroke width.
    pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_stroke_width(w);
        self
    }

    /// Method to set the line stroke width.
    pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("stroke-width", w.into());
    }

    /// Builder style method to set the line stroke color/pattern.
    pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
        self.set_stroke(stroke);
        self
    }

    /// Method to set the line stroke color/pattern.
    pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
        self.set_attribute("stroke", stroke.into());
    }

    /// Builder style method to set the line fill color/pattern.
    pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
        self.set_fill(fill);
        self
    }

    /// Method to set the line fill color/pattern.
    pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
        self.set_attribute("fill", fill.into());
    }
}

impl Into<VTag> for Line {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("line"), Some(self.listeners), None)
    }
}
