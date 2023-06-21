use std::rc::Rc;
use std::borrow::Cow;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use crate::props::{ContainerBuilder, EventSubscriber, IntoOptionalKey, WidgetBuilder};
use crate::widget::{Button, Container};

use pwt_macros::builder;
use pwt_macros::widget;

/// Lightweight message with an optional action button.
///
/// Snackbars provide updates on an app’s processes.
///
/// - Dismissive snackbars appear temporarily, and disappear on their own without requiring user input.
/// - Non-dismissive snackbars persist until the user takes an action or selects the close affordance.
///
/// Only one snackbar may be displayed at a time, so it is more convenient to use [SnackBarManager], which
/// automatically serializes the display of snackbars.
//#[derive(Properties, Clone, PartialEq)]
#[widget(pwt=crate, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct SnackBar {
    /// The text message.
    #[builder(IntoPropValue, into_prop_value)]
    pub message: Option<AttrValue>,

    /// The label of the action button
    #[builder(IntoPropValue, into_prop_value)]
    pub action_label: Option<AttrValue>,

    /// Callback for action button.
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_action: Option<Callback<()>>,
}

impl SnackBar {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

impl Into<VTag> for SnackBar {
    fn into(self) -> VTag {
        let attributes = self.std_props.cumulate_attributes(Some("pwt-snackbar"));

        let listeners = Listeners::Pending(self.listeners.listeners.into_boxed_slice());

        let mut children = Vec::new();
        children.push(
            Container::new()
                .class("pwt-snackbar-message")
                .with_child(self.message.clone().unwrap_or(AttrValue::Static("")))
                .into()
        );
        if let Some(action_label) = &self.action_label {
            children.push(
                Button::new(action_label.clone())
                    .class("pwt-button-filled")
                    .class("pwt-snackbar-action")
                    .class("pwt-scheme-inverse-surface")
                    .onclick({
                        let on_action = self.on_action.clone();
                        move |_| {
                            if let Some(on_action) = &on_action {
                                on_action.emit(());
                            }
                        }
                    })
                    .into()
            );
        }

        let children = VList::with_children(children, None);

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
