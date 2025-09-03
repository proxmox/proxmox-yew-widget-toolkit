use std::borrow::Cow;

use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use crate::prelude::*;

/// Vertical container with flex layout.
#[widget(pwt=crate, @element, @container)]
#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Card {}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {}).class("pwt-card")
    }
}

impl IntoVTag for Card {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(self.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(self.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children.into(),
        )
    }
}
