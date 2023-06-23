use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<path>` element.
#[widget(pwt=crate, @element, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Path {
    children: Option<Vec<VNode>>,
}

impl Path {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to define the path to be drawn.
    pub fn d(mut self, commands: impl Into<AttrValue>) -> Self {
        self.set_d(commands);
        self
    }

    /// Method to define the path to be drawn.
    pub fn set_d(&mut self, commands: impl Into<AttrValue>) {
        self.set_attribute("d", commands.into());
    }

    impl_svg_animation_attributes!();
    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Path {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("path"), None::<&str>, Some(self.listeners), self.children)
    }
}
