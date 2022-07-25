use std::rc::Rc;

use anyhow::Error;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::widget::{Button, Dialog, Fa, Row, Toolbar};

#[derive(Clone, Properties, PartialEq)]
pub struct AlertDialog {
    pub title: Option<AttrValue>,
    pub message: String,
    pub onclose: Option<Callback<()>>,
}

impl AlertDialog {

    pub fn new(message: impl Into<String>) -> Self {
        yew::props!(AlertDialog { message: message.into() })
    }

    pub fn title(mut self, title: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: impl IntoPropValue<Option<AttrValue>>) {
        self.title = title.into_prop_value();
    }

    pub fn onclose(mut self, cb: impl Into<Option<Callback<()>>>) -> Self {
        self.onclose = cb.into();
        self
    }

    pub fn html(self) -> VNode {
        self.into()
    }
}

pub fn error_message(text: &str, class: &str) -> Html {
    let icon_class = "pwt-bg-color-error pwt-color-on-error pwt-p-2 pwt-shape-circle \
                      fa fa-lg fa-align-center fa-exclamation-triangle pwt-me-2";

    Row::new()
        .padding(2)
        .class(class.to_owned())
        .class("pwt-align-items-center")
        .attribute("style", "max-width:600px;")
        .with_child(html!{<span class={icon_class} aria-hidden="true"/>})
        .with_child(text)
        .into()
}

pub fn display_load_result<T>(result: &Option<Result<T, Error>>, render: impl Fn(&T) -> Html) -> Html {
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

#[function_component(PwtAlertDialog)]
pub fn pwt_alert_dialog(props: &AlertDialog) -> Html {
    let onclick = Callback::from({
        let onclose = props.onclose.clone();
        move |_| {
            if let Some(onclose) = &onclose {
                onclose.emit(());
            }
        }
    });

    let title = format!("{}", props.title.as_deref().unwrap_or("Alert"));

    Dialog::new(title.clone())
        .onclose(props.onclose.clone())
        .with_child(error_message(&props.message, "pwt-p-4"))
        .with_child(
            Toolbar::new()
                .class("emphased pwt-border-top")
                .with_flex_spacer()
                .with_child(Button::new("Continue").onclick(onclick).autofocus(true))
        )
        .into()
}

impl Into<VNode> for AlertDialog {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtAlertDialog>(Rc::new(self), None);
        VNode::from(comp)
    }
}
