use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<polygon>` element.
#[widget(pwt=crate, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Polygon {}

impl Polygon {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to define the list of points.
    pub fn points(mut self, points: &[(f32, f32)]) -> Self {
        self.set_points(points);
        self
    }

    /// Method to define the list of points.
    pub fn set_points(&mut self, points: &[(f32, f32)]) {
        let points = points.iter().fold(String::new(), |mut acc, (x, y)| {
            if !acc.is_empty() { acc.push(' '); }
            acc.push_str(&format!("{x},{y}"));
            acc
        });
        self.set_attribute("points", points);
    }

    /// Builder style method to set the polygon stroke width.
    pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_stroke_width(w);
        self
    }

    /// Method to set the polygon stroke width.
    pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("stroke-width", w.into());
    }

    /// Builder style method to set the polygon stroke color/pattern.
    pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
        self.set_stroke(stroke);
        self
    }

    /// Method to set the polygon stroke color/pattern.
    pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
        self.set_attribute("stroke", stroke.into());
    }

    /// Builder style method to set the polygon fill color/pattern.
    pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
        self.set_fill(fill);
        self
    }

    /// Method to set the polygon fill color/pattern.
    pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
        self.set_attribute("fill", fill.into());
    }
}

impl Into<VTag> for Polygon {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("polygon"), Some(self.listeners), None)
    }
}
