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

impl From<Card> for VTag {
    fn from(val: Card) -> Self {
        let attributes = val.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(val.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(val.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            val.std_props.node_ref,
            val.std_props.key,
            attributes,
            listeners,
            children.into(),
        )
    }
}
