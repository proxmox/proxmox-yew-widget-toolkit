use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use crate::prelude::*;

#[widget(pwt=crate, @element, @container)]
#[derive(Default, Debug, Clone)]
pub struct Column {
    gap: usize,
}

impl Column {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn gap(mut self, gap: usize) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_flex_spacer(mut self) -> Self {
        self.add_flex_spacer();
        self
    }

    pub fn add_flex_spacer(&mut self) {
        self.add_child(html!{<div class="pwt-flex-fill"/>});
    }
}

impl Into<VTag> for Column {
    fn into(mut self) -> VTag {

        self.add_class("pwt-d-flex");
        self.add_class("pwt-flex-column");

        if self.gap > 0 {
            self.add_class(format!("pwt-gap-{}", self.gap));
        }

        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(
            self.listeners.listeners.into_boxed_slice()
        );

        let children = VList::with_children(self.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            self.std_props.node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children,
        )
    }
}
