use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use crate::widget::Button;
use pwt_macros::widget;

/// List of Buttons.
#[widget(pwt=crate, comp=PwtSegmentedButton, @element)]
#[derive(Properties, PartialEq, Clone)]
pub struct SegmentedButton {
    buttons: Vec<Button>,
}

impl SegmentedButton {
    /// Create a new segmented button.
    pub fn new() -> Self {
        yew::props!(Self {
            buttons: Vec::new()
        })
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
            props.std_props.node_ref.clone(),
            props.std_props.key.clone(),
            attributes,
            listeners,
            VList::with_children(buttons, None),
        )
        .into()
    }
}
