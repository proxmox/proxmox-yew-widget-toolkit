use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<ellipse>` element.
#[widget(pwt=crate, @element, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Ellipse {
    children: Option<Vec<VNode>>,
}

impl Ellipse {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the ellipse position.
    pub fn position(mut self, cx: impl Into<SvgLength>, cy: impl Into<SvgLength>) -> Self {
        self.set_position(cx, cy);
        self
    }

    /// Method to set the ellipse position.
    pub fn set_position(&mut self, cx: impl Into<SvgLength>, cy: impl Into<SvgLength>) {
        self.set_attribute("cx", cx.into());
        self.set_attribute("cy", cy.into());
    }

    /// Builder style method to set the ellipse x-axis radius.
    pub fn rx(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_rx(r);
        self
    }

    /// Method to set the ellipse x-axis radius.
    pub fn set_rx(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("rx", r.into());
    }

    /// Builder style method to set the ellipse y-axis radius.
    pub fn ry(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_ry(r);
        self
    }

    /// Method to set the ellipse y-axis radius.
    pub fn set_ry(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("ry", r.into());
    }

    impl_svg_animation_attributes!();
    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Ellipse {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("ellipse"), Some(self.listeners), None)
    }
}
