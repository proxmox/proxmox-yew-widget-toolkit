use std::borrow::Cow;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use crate::widget::prelude::*;

#[widget(@element, @container)]
#[derive(Default, Debug, Clone)]
pub struct Row {
    gap: usize,
}

impl Row {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn gap(mut self, gap: usize) -> Self {
        self.gap = gap;
        self
    }
}

impl Into<VTag> for Row {
    fn into(mut self) -> VTag {

        self.add_class("pwt-d-flex");

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
