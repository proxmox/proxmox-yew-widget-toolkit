use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};
use yew::html::IntoPropValue;

use pwt_macros::widget;

use crate::prelude::*;
use crate::css::{Display, FlexDirection};

/// Vertical container with flex layout.
#[widget(pwt=crate, @element, @container)]
#[derive(Default, Debug, Clone)]
pub struct Column {}

impl Column {

    /// Create a new instance.
    pub fn new() -> Self {
        Self::default()
            .class(Display::Flex)
            .class(FlexDirection::Column)
    }

    /// Builder style method to add a CSS class to set gap between children.
    ///
    /// The default CSS template defines utility classes for gaps (`pwt-gap-{gap}`).
    pub fn gap(mut self, gap: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_gap(gap);
        self
    }

    /// Method to add a CSS class to set gap between children.
    pub fn add_gap(&mut self, gap: impl IntoPropValue<Option<usize>>) {
        if let Some(gap) = gap.into_prop_value() {
            self.add_class(format!("pwt-gap-{}", gap))
        }
    }

    /// Builder style method to add a flexible spacer.
    ///
    /// A flexible spacer is a empty child with CSS `flex: 1 1 auto;`.
    pub fn with_flex_spacer(mut self) -> Self {
        self.add_flex_spacer();
        self
    }

    /// Method to add a flexible spacer.
    pub fn add_flex_spacer(&mut self) {
        self.add_child(html!{<div class="pwt-flex-fill"/>});
    }
}

impl Into<VTag> for Column {
    fn into(self) -> VTag {
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
            children.into(),
        )
    }
}
