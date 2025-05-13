use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::impl_yew_std_props_builder;
use crate::prelude::*;
use crate::widget::MessageBox;

use pwt_macros::builder;

/// Alert Dialog - Modal window to display error messages.
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct AlertDialog {
    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Optional dialog title - defaults to "Alert".
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub title: Option<AttrValue>,

    /// The error message.
    pub message: Html,

    /// Close window callback.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_close: Option<Callback<()>>,

    /// Enable/disable dragging
    #[prop_or(true)]
    #[builder]
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
