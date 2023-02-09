use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

/// SVG `<a>` element (hyperlink).
///
/// This a container, which means you can create a link around any text or shape.
#[widget(pwt=crate, @element, @container, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Hyperlink {}

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

impl Into<VTag> for Hyperlink {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("a"), Some(self.listeners), Some(self.children))
    }
}
