use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<polygon>` element.
#[widget(pwt=crate, @element, @svg)]
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

    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Polygon {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("polygon"), Some(self.listeners), None)
    }
}
