use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::{Hyperlink, SvgLength};

pub trait IntoSvgTSpanChild {
    fn into_svg_tspan_child(self) -> VNode;
}

impl<T: Into<AttrValue>> IntoSvgTSpanChild for T {
    fn into_svg_tspan_child(self) -> VNode {
        self.into().into()
    }
}

impl IntoSvgTSpanChild for Hyperlink {
    fn into_svg_tspan_child(self) -> VNode {
        self.into()
    }
}

/// SVG `<tspan>` element.
#[widget(pwt=crate, @element, @svg)]
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

    impl_svg_position_attributes!();

    /// Builder style method to set the tspan X offset.
    pub fn dx(mut self, x: impl Into<SvgLength>) -> Self {
        self.set_dx(x);
        self
    }

    /// Method to set the tspan X offset
    pub fn set_dx(&mut self, dx: impl Into<SvgLength>) {
        self.set_attribute("dx", dx.into());
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

    impl_svg_container_animation_attributes!();
    impl_svg_presentation_attributes!();

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

impl From<TSpan> for VTag {
    fn from(val: TSpan) -> Self {
        val.std_props.into_vtag(
            Cow::Borrowed("tspan"),
            None::<&str>,
            Some(val.listeners),
            Some(val.children),
        )
    }
}
