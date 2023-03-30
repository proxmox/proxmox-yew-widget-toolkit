use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::{Button, Dialog, Row, Toolbar};

/// Alert Dialog - Modal window to display error messages.
#[derive(Clone, Properties, PartialEq)]
pub struct AlertDialog {
    /// Optional dialog title - defaults to "Alert".
    pub title: Option<AttrValue>,
    /// The error message.
    pub message: String,
    /// Close window callback.
    pub on_close: Option<Callback<()>>,
    /// Enable/disable dragging
    #[prop_or_default]
    pub draggable: bool,
}

impl AlertDialog {
    /// Create a new instance.
    pub fn new(message: impl Into<String>) -> Self {
        yew::props!(AlertDialog {
            message: message.into()
        })
    }

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

/// Creates a nicely formated error message.
pub fn error_message(text: &str, class: &str) -> Html {
    let icon_class = "pwt-bg-color-surface pwt-color-error pwt-p-2 pwt-shape-circle \
                      fa fa-lg fa-align-center fa-exclamation-triangle pwt-me-2";

    Row::new()
        .padding(2)
        .class(class.to_owned())
        .class("pwt-align-items-center")
        .attribute("style", "max-width:600px;")
        .with_child(html! {<span class={icon_class} aria-hidden="true"/>})
        .with_child(html! {<p>{text}</p>})
        .into()
}

/* we currently do not need this */
/*
pub fn display_load_result2<T>(result: &Option<Result<T, Error>>, render: impl Fn(&T) -> Html) -> Html {
    match result {
        None => html!{
            <div class="pwt-text-center pwt-p-4">
            {Fa::new("spinner").class("pwt-me-1").pulse()}
            {"Loading..."}
            </div>
        },
        Some(Ok(data)) => render(data),
        Some(Err(err)) => error_message(&format!("Error: {}", err), "pwt-p-2"),
    }
}
*/

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

    let title = format!("{}", props.title.as_deref().unwrap_or("Alert"));

    Dialog::new(title.clone())
        .draggable(props.draggable)
        .on_close(props.on_close.clone())
        .with_child(error_message(&props.message, "pwt-p-4"))
        .with_child(
            Toolbar::new()
                .class("emphased pwt-border-top")
                .with_flex_spacer()
                .with_child(Button::new("Continue").onclick(onclick).autofocus(true)),
        )
        .into()
}

impl Into<VNode> for AlertDialog {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtAlertDialog>(Rc::new(self), None);
        VNode::from(comp)
    }
}
