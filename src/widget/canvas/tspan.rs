use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

pub trait IntoSvgTSpanChild {
    fn into_svg_tspan_child(self) -> VNode;
}

impl<T: Into<AttrValue>> IntoSvgTSpanChild for T {
    fn into_svg_tspan_child(self) -> VNode {
        self.into().into()
    }
}

/// SVG `<tspan>` element.
#[widget(pwt=crate, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct TSpan {
    #[prop_or_default]
    children: Vec<VNode>,
}

impl TSpan {

    /// Create a new instance.
    pub fn new(text: impl IntoSvgTSpanChild) -> Self {
        yew::props!(Self {}).with_child(text)
    }

    /// Builder style method to set the tspan position.
    pub fn position(mut self, x: impl Into<SvgLength>, y: impl Into<SvgLength>) -> Self {
        self.set_position(x, y);
        self
    }

    /// Method to set the tspan position.
    pub fn set_position(&mut self, x: impl Into<SvgLength>, y: impl Into<SvgLength>) {
        self.set_attribute("x", x.into());
        self.set_attribute("y", y.into());
    }

    /// Builder style method to set the tspan X position.
    pub fn x(mut self, x: impl Into<SvgLength>) -> Self {
        self.set_x(x);
        self
    }

    /// Method to set the tspan X position.
    pub fn set_x(&mut self, x: impl Into<SvgLength>) {
        self.set_attribute("x", x.into());
    }

    /// Builder style method to set the tspan X offset.
    pub fn dx(mut self, x: impl Into<SvgLength>) -> Self {
        self.set_dx(x);
        self
    }

    /// Method to set the tspan X offset
    pub fn set_dx(&mut self, dx: impl Into<SvgLength>) {
        self.set_attribute("dx", dx.into());
    }


    /// Builder style method to set the tspan Y position.
    pub fn y(mut self, y: impl Into<SvgLength>) -> Self {
        self.set_y(y);
        self
    }

    /// Method to set the tspan Y position.
    pub fn set_y(&mut self, y: impl Into<SvgLength>) {
        self.set_attribute("y", y.into());
    }

    /// Builder style method to set the tspan Y offset.
    pub fn dy(mut self, y: impl Into<SvgLength>) -> Self {
        self.set_dy(y);
        self
    }

    /// Method to set the tspan Y offset
    pub fn set_dy(&mut self, dy: impl Into<SvgLength>) {
        self.set_attribute("dy", dy.into());
    }


    /// Builder style method to set the tspan stroke width.
    pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_stroke_width(w);
        self
    }

    /// Method to set the tspan stroke width.
    pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("stroke-width", w.into());
    }

    /// Builder style method to set the tspan stroke color/pattern.
    pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
        self.set_stroke(stroke);
        self
    }

    /// Method to set the tspan stroke color/pattern.
    pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
        self.set_attribute("stroke", stroke.into());
    }

    /// Builder style method to set the tspan fill color/pattern.
    pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
        self.set_fill(fill);
        self
    }

    /// Method to set the tspan fill color/pattern.
    pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
        self.set_attribute("fill", fill.into());
    }

    /// Builder style method to add a tspan child node.
    pub fn with_child(mut self, child: impl IntoSvgTSpanChild) -> Self {
        self.add_child(child);
        self
    }

    /// Method to add a tspan child node.
    pub fn add_child(&mut self, child: impl IntoSvgTSpanChild) {
        self.children.push(child.into_svg_tspan_child());
    }

}

impl Into<VTag> for TSpan {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("tspan"), Some(self.listeners), Some(self.children))
    }
}
