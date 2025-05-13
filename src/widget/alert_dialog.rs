use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::impl_yew_std_props_builder;
use crate::prelude::*;
use crate::widget::MessageBox;

/// Alert Dialog - Modal window to display error messages.
#[derive(Clone, Properties, PartialEq)]
pub struct AlertDialog {
    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Optional dialog title - defaults to "Alert".
    #[prop_or_default]
    pub title: Option<AttrValue>,

    /// The error message.
    pub message: Html,
    /// Close window callback.

    #[prop_or_default]
    pub on_close: Option<Callback<()>>,
    /// Enable/disable dragging

    #[prop_or(true)]
    pub draggable: bool,
}

impl AlertDialog {
    /// Create a new instance.
    pub fn new(message: impl Into<Html>) -> Self {
        yew::props!(AlertDialog {
            message: message.into()
        })
    }

    impl_yew_std_props_builder!();

    /// Builder style method to set the dialog title.
    pub fn title(mut self, title: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_title(title);
        self
    }

    /// Method to set the dialog title.
    pub fn set_title(&mut self, title: impl IntoPropValue<Option<AttrValue>>) {
        self.title = title.into_prop_value();
    }

    /// Builder style method to set the window close callback.
    pub fn on_close(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_close = cb.into_event_callback();
        self
    }

    /// Builder style method to enable/disable dragging
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.set_draggable(draggable);
        self
    }

    /// Enable/disable dragging
    pub fn set_draggable(&mut self, draggable: bool) {
        self.draggable = draggable;
    }
}

#[function_component(PwtAlertDialog)]
#[doc(hidden)]
pub fn pwt_alert_dialog(props: &AlertDialog) -> Html {
    let onclick = Callback::from({
        let on_close = props.on_close.clone();
        move |_| {
            if let Some(on_close) = &on_close {
                on_close.emit(());
            }
        }
    });

    let title = props.title.as_deref().unwrap_or("Alert").to_string();

    MessageBox::new(title, props.message.clone())
        .node_ref(props.node_ref.clone())
        .icon_class("fa-exclamation-triangle")
        .on_close(onclick)
        .into()
}

impl From<AlertDialog> for VNode {
    fn from(props: AlertDialog) -> Self {
        let key = props.key.clone();
        let comp = VComp::new::<PwtAlertDialog>(Rc::new(props), key);
        VNode::from(comp)
    }
}
