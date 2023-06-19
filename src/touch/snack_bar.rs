use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoPropValue, IntoEventCallback};

use crate::props::{IntoOptionalKey, ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Container, Button};

use pwt_macros::builder;

#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct SnackBar {
    /// The yew component key.
    pub key: Option<Key>,

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

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }
}

pub enum Msg {
    Action,
}

#[doc(hidden)]
pub struct PwtSnackBar {}

impl Component for PwtSnackBar {
    type Message = Msg;
    type Properties = SnackBar;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let action = props.action_label.as_ref().map(|label| {
            Button::new(props.action_label.clone())
                .class("pwt-button-filled")
                .class("pwt-snackbar-action")
                .class("pwt-scheme-inverse-surface")
                .onclick(ctx.link().callback(|_| Msg::Action))
        });


        Container::new()
            .class("pwt-snackbar")
            .with_child(
                Container::new()
                    .class("pwt-snackbar-message")
                    .with_child(props.message.clone().unwrap_or(AttrValue::Static("")))
            )
            .with_optional_child(action)
            .into()
    }
}

impl Into<VNode> for SnackBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtSnackBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}
