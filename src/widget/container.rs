use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

/// Wrapper for Html container elements like `<div>`.
#[widget(pwt=crate, @element, @container)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
pub struct Container {
    #[prop_or(Cow::Borrowed("div"))]
    tag: Cow<'static, str>,
}

impl Container {
    pub fn new() -> Self {
        yew::props!{ Self {
        }}
    }

    pub fn tag(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.set_tag(tag);
        self
    }

    pub fn set_tag(&mut self, tag: impl Into<Cow<'static, str>>) {
        self.tag = tag.into();
    }
}

impl Into<VTag> for Container {
    fn into(self) -> VTag {
        self.std_props.into_vtag(self.tag, None::<&str>, Some(self.listeners), Some(self.children))
    }
}
