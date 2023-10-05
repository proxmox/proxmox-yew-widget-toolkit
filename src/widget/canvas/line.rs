use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<line>` element.
#[widget(pwt=crate, @element, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Line {
    #[prop_or_default]
    children: Option<Vec<VNode>>,
}

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

    impl_svg_animation_attributes!();
    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Line {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("line"), None::<&str>, Some(self.listeners), self.children)
    }
}
