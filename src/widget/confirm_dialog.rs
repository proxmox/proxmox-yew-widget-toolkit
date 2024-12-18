use std::rc::Rc;

use crate::props::{ContainerBuilder, CssPaddingBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Button, Container, Dialog, Toolbar};
use html::{IntoEventCallback, IntoPropValue};
use pwt_macros::builder;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct ConfirmDialog {
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub title: AttrValue,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub confirm_text: AttrValue,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub confirm_message: AttrValue,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_confirm: Option<Callback<()>>,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_done: Option<Callback<()>>,
}

impl ConfirmDialog {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

impl Default for ConfirmDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ConfirmDialog> for VNode {
    fn from(value: ConfirmDialog) -> Self {
        VComp::new::<PwtConfirmDialog>(Rc::new(value), None).into()
    }
}

struct PwtConfirmDialog {}

impl Component for PwtConfirmDialog {
    type Message = ();
    type Properties = ConfirmDialog;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Dialog::new(props.title.clone())
            .with_child(
                Container::new()
                    .padding(4)
                    .class("pwt-d-felx pwt-flex-direction-column")
                    .with_child(props.confirm_message.clone()),
            )
            .with_child(Toolbar::new().with_flex_spacer().with_child(
                Button::new(props.confirm_text.clone()).onclick({
                    let on_confirm = props.on_confirm.clone();
                    let on_done = props.on_done.clone();

                    move |e: MouseEvent| {
                        let event = e.unchecked_into::<Event>();
                        event.prevent_default();

                        if let Some(on_confirm) = &on_confirm {
                            on_confirm.emit(());
                        }

                        if let Some(on_done) = &on_done {
                            on_done.emit(());
                        }
                    }
                }),
            ))
            .on_close({
                let on_close = props.on_close.clone();
                let on_done = props.on_done.clone();

                move |()| {
                    if let Some(on_close) = &on_close {
                        on_close.emit(());
                    }

                    if let Some(on_done) = &on_done {
                        on_done.emit(());
                    }
                }
            })
            .into()
    }
}
