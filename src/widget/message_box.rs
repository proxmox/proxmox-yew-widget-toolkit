use std::rc::Rc;

use pwt_macros::builder;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::{Button, Dialog, Row, Toolbar};

/// Message Box - Modal window to display various messages.
#[builder]
#[derive(Clone, Properties, PartialEq)]
pub struct MessageBox {
    /// Dialog title
    #[builder]
    pub title: AttrValue,
    /// The error message.
    #[builder]
    pub message: Html,
    /// Close window callback.
    #[builder_cb(Into, into, Option<Callback<bool>>)]
    pub on_close: Option<Callback<bool>>,
    /// Enable/disable dragging
    #[prop_or(true)]
    #[builder]
    pub draggable: bool,
    /// Button Style, defaults to a single 'Continue' button
    #[prop_or_default]
    #[builder]
    pub buttons: MessageBoxButtons,

    /// Icon Class
    #[prop_or_default]
    #[builder(Into, into)]
    pub icon_class: Classes,
}

#[derive(PartialEq, Clone)]
pub enum MessageBoxButtons {
    Single(Option<String>),
    YesNo,
    CancelOk,
}

impl Default for MessageBoxButtons {
    fn default() -> Self {
        MessageBoxButtons::Single(None)
    }
}

impl MessageBox {
    /// Create a new instance.
    pub fn new(title: impl Into<AttrValue>, message: impl Into<Html>) -> Self {
        yew::props!(MessageBox {
            title: title.into(),
            message: message.into()
        })
    }
}

pub(crate) fn message(text: impl Into<Html>, class: &str, icon_class: impl Into<Classes>) -> Html {
    let icon_class = classes!("fa-lg", "fa", "fa-align-center", icon_class,);

    Row::new()
        .padding(2)
        .class(class.to_owned())
        .class("pwt-align-items-center")
        .with_child(
            html! {<span class={"pwt-message-sign"} role="none"><i class={icon_class}/></span>},
        )
        .with_child(html! {<p style={"overflow-wrap: anywhere;"}>{text}</p>})
        .into()
}

#[function_component(PwtMessageBox)]
#[doc(hidden)]
pub fn pwt_message_box(props: &MessageBox) -> Html {
    let onclick_success = Callback::from({
        let on_close = props.on_close.clone();
        move |_| {
            if let Some(on_close) = &on_close {
                on_close.emit(true);
            }
        }
    });
    let onclick_failure = Callback::from({
        let on_close = props.on_close.clone();
        move |_| {
            if let Some(on_close) = &on_close {
                on_close.emit(false);
            }
        }
    });
    let on_close = Callback::from({
        let on_close = props.on_close.clone();
        move |_| {
            if let Some(on_close) = &on_close {
                on_close.emit(false);
            }
        }
    });

    let buttons = match &props.buttons {
        MessageBoxButtons::Single(ref text) => {
            let text = text.as_deref().map(String::from).unwrap_or(tr!("Continue"));
            vec![Button::new(text).onclick(onclick_success)]
        }
        MessageBoxButtons::YesNo => {
            vec![
                Button::new(tr!("Yes")).onclick(onclick_success),
                Button::new(tr!("No")).onclick(onclick_failure),
            ]
        }
        MessageBoxButtons::CancelOk => {
            vec![
                Button::new(tr!("Cancel")).onclick(onclick_failure),
                Button::new(tr!("Ok")).onclick(onclick_success),
            ]
        }
    };

    let mut bbar = Toolbar::new().with_flex_spacer();
    for button in buttons {
        bbar.add_child(button);
    }
    bbar.add_flex_spacer();

    Dialog::new(props.title.clone())
        .style("min-width: 300px; max-width:600px;")
        .draggable(props.draggable)
        .on_close(on_close)
        .with_child(message(props.message.clone(), "", props.icon_class.clone()))
        .with_child(bbar)
        .into()
}

impl From<MessageBox> for VNode {
    fn from(val: MessageBox) -> Self {
        let comp = VComp::new::<PwtMessageBox>(Rc::new(val), None);
        VNode::from(comp)
    }
}
