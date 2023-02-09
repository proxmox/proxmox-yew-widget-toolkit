use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super:: {Hyperlink, SvgLength, TSpan};

pub trait IntoSvgTextChild {
    fn into_svg_text_child(self) -> VNode;
}

impl<T: Into<AttrValue>> IntoSvgTextChild for T {
    fn into_svg_text_child(self) -> VNode {
        self.into().into()
    }
}

impl IntoSvgTextChild for TSpan {
    fn into_svg_text_child(self) -> VNode {
        self.into()
    }
}

impl IntoSvgTextChild for Hyperlink {
    fn into_svg_text_child(self) -> VNode {
        self.into()
    }
}

/// SVG `<text>` element.
#[widget(pwt=crate, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Text {
    #[prop_or_default]
    children: Vec<VNode>,
}

impl Text {

    /// Create a new instance.
    pub fn new(text: impl IntoSvgTextChild) -> Self {
        yew::props!(Self {}).with_child(text)
    }

    impl_svg_position_attributes!();

    /// Builder style method to set the text X offset.
    pub fn dx(mut self, x: impl Into<SvgLength>) -> Self {
        self.set_dx(x);
        self
    }

    /// Method to set the text X offset
    pub fn set_dx(&mut self, dx: impl Into<SvgLength>) {
        self.set_attribute("dx", dx.into());
    }

    /// Builder style method to set the text Y offset.
    pub fn dy(mut self, y: impl Into<SvgLength>) -> Self {
        self.set_dy(y);
        self
    }

    /// Method to set the text Y offset
    pub fn set_dy(&mut self, dy: impl Into<SvgLength>) {
        self.set_attribute("dy", dy.into());
    }

    impl_svg_presentation_attributes!();

    /// Builder style method to add a text child node.
    pub fn with_child(mut self, child: impl IntoSvgTextChild) -> Self {
        self.add_child(child);
        self
    }

    /// Method to add a text child node.
    pub fn add_child(&mut self, child: impl IntoSvgTextChild) {
        self.children.push(child.into_svg_text_child());
    }
}

impl Into<VTag> for Text {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("text"), Some(self.listeners), Some(self.children))
    }
}
