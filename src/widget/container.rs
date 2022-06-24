use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

#[widget(@element, @container)]
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

        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(
            self.listeners.listeners.into_boxed_slice()
        );

        let children = VList::with_children(self.children, None);


        VTag::__new_other(
            self.tag,
            self.std_props.node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children,
        )
    }
}
