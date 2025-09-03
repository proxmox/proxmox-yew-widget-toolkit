use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::{IntoVTag, WidgetBuilder};

use super::SvgLength;

/// SVG group (`<g>`) element.
#[widget(pwt=crate, @element, @container, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Group {}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Group {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_svg_container_animation_attributes!();
    impl_svg_presentation_attributes!();
}

impl IntoVTag for Group {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        self.std_props.into_vtag(
            Cow::Borrowed("g"),
            node_ref,
            None::<&str>,
            Some(self.listeners),
            Some(self.children),
        )
    }
}
