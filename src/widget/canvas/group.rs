use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG group (`<g>`) element.
#[widget(pwt=crate, @element, @container, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Group {}

impl Group {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Group {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("g"), Some(self.listeners), Some(self.children))
    }
}
