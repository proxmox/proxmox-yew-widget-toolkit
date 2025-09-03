use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::{IntoVTag, WidgetBuilder};

use super::SvgLength;

/// SVG `<polygon>` element.
#[widget(pwt=crate, @element, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Polygon {
    #[prop_or_default]
    children: Option<Vec<VNode>>,
}

impl Default for Polygon {
    fn default() -> Self {
        Self::new()
    }
}

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
            if !acc.is_empty() {
                acc.push(' ');
            }
            acc.push_str(&format!("{x},{y}"));
            acc
        });
        self.set_attribute("points", points);
    }

    impl_svg_animation_attributes!();
    impl_svg_presentation_attributes!();
}

impl IntoVTag for Polygon {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        self.std_props.into_vtag(
            Cow::Borrowed("polygon"),
            node_ref,
            None::<&str>,
            Some(self.listeners),
            self.children,
        )
    }
}
