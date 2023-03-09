use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VNode, VTag};
use yew::html::IntoPropValue;

use pwt_macros::widget;

use crate::prelude::*;
use crate::css::{Display, FlexDirection};

/// Vertical container with flex layout.
#[widget(pwt=crate, @element, @container)]
#[derive(Default, Debug, Clone)]
pub struct Card {}

impl Card {

    /// Create a new instance.
    pub fn new() -> Self {
        Self::default().class("pwt-card")
    }
}

impl Into<VTag> for Card {
    fn into(self) -> VTag {
        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(
            self.listeners.listeners.into_boxed_slice()
        );

        //let children: Vec<VNode> = Vec::new();

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
