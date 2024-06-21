use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use crate::props::{ListenersWrapper, WidgetStdProps};

use pwt_macros::widget;

/// Wrapper for Html container elements like `<div>`.
#[widget(pwt=crate, @element, @container)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
pub struct Container {
    #[prop_or(Cow::Borrowed("div"))]
    tag: Cow<'static, str>,
}

impl Container {
    /// Creates new Container instance
    pub fn new() -> Self {
        yew::props! { Self {} }
    }

    /// Creates a new instance from existing properties
    pub fn form_widget_props(
        std_props: WidgetStdProps,
        listeners: Option<ListenersWrapper>,
    ) -> Self {
        yew::props! { Self { std_props, listeners: listeners.unwrap_or_default() } }
    }

    /// Builder style method to set the tag of the element (default is `div`)
    pub fn tag(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.set_tag(tag);
        self
    }

    /// Method to set the tag of the element (default is `div`)
    pub fn set_tag(&mut self, tag: impl Into<Cow<'static, str>>) {
        self.tag = tag.into();
    }
}

impl Into<VTag> for Container {
    fn into(self) -> VTag {
        self.std_props.into_vtag(
            self.tag,
            None::<&str>,
            Some(self.listeners),
            Some(self.children),
        )
    }
}
