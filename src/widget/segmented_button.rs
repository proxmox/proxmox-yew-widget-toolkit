use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use crate::props::WidgetBuilder;
use crate::widget::Button;
use pwt_macros::widget;

/// A group of related [`Button`]s rendered as a single segmented control.
///
/// Renders with `role="group"`; set [`aria_label`](Self::aria_label) so assistive
/// technology announces the buttons as one labelled control.
#[widget(pwt=crate, comp=PwtSegmentedButton, @element)]
#[derive(Properties, PartialEq, Clone)]
pub struct SegmentedButton {
    buttons: Vec<Button>,
}

impl Default for SegmentedButton {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentedButton {
    /// Create a new segmented button.
    pub fn new() -> Self {
        yew::props!(Self {
            buttons: Vec::new()
        })
        // a segmented control is semantically one group of related buttons
        .attribute("role", "group")
    }

    /// Builder style method to set the `aria-label` announced for the group.
    pub fn aria_label(self, label: impl Into<AttrValue>) -> Self {
        self.attribute("aria-label", label.into())
    }

    /// Builder style method to add a button
    pub fn with_button(mut self, button: Button) -> Self {
        self.add_button(button);
        self
    }

    /// Method to add a button
    pub fn add_button(&mut self, button: Button) {
        self.buttons.push(button);
    }
}

#[doc(hidden)]
pub struct PwtSegmentedButton;

impl Component for PwtSegmentedButton {
    type Message = ();
    type Properties = SegmentedButton;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let attributes = props
            .std_props
            .cumulate_attributes(Some(classes!("pwt-segmented-button",)));

        let buttons = props
            .buttons
            .clone()
            .into_iter()
            .map(|b| b.into())
            .collect();

        let listeners = Listeners::Pending(props.listeners.listeners.clone().into_boxed_slice());

        VTag::__new_other(
            Cow::Borrowed("div"),
            NodeRef::default(),
            props.std_props.key.clone(),
            attributes,
            listeners,
            VList::with_children(buttons, None).into(),
        )
        .into()
    }
}
