use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::{IntoVTag, WidgetBuilder};

/// SVG `<a>` element (hyperlink).
///
/// This a container, which means you can create a link around any text or shape.
#[widget(pwt=crate, @element, @container, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Hyperlink {}

impl Default for Hyperlink {
    fn default() -> Self {
        Self::new()
    }
}

impl Hyperlink {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the href.
    pub fn href(mut self, href: impl Into<AttrValue>) -> Self {
        self.set_href(href);
        self
    }

    /// Method to set the href.
    pub fn set_href(&mut self, href: impl Into<AttrValue>) {
        self.set_attribute("href", href.into());
    }
}

impl IntoVTag for Hyperlink {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        self.std_props.into_vtag(
            Cow::Borrowed("a"),
            node_ref,
            None::<&str>,
            Some(self.listeners),
            Some(self.children),
        )
    }
}
