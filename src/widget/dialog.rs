use std::rc::Rc;

use web_sys::HtmlElement;
use wasm_bindgen::JsCast;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::prelude::*;
use crate::widget::{Button, Panel};

/// Modal Dialog.
///
/// This widget is implemented using the relatively new Html `<dialog>`
/// tag in order to get correct focus handling.
#[derive(Properties, Clone, PartialEq)]
pub struct Dialog {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    pub title: AttrValue,
    pub on_close: Option<Callback<()>>,

    #[prop_or_default]
    pub children: Vec<VNode>,

    pub style: Option<AttrValue>,
}

impl ContainerBuilder for Dialog {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

impl Dialog {

    pub fn new(title: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
        })
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn style(mut self, style: impl Into<AttrValue>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn on_close(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_close = cb.into_event_callback();
        self
    }

    pub fn html(self) -> VNode {
        self.into()
    }
}

pub enum Msg {
    Open,
    Close,
}

#[doc(hidden)]
pub struct PwtDialog {
    open: bool,
    last_active: Option<web_sys::HtmlElement>, // last focused element
}

impl PwtDialog {

    fn restore_focus(&mut self) {
        if let Some(el) = self.last_active.take() {
            let _ = el.focus();
        }
    }
}

impl Component for PwtDialog {
    type Message = Msg;
    type Properties = Dialog;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Open);

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let last_active = document.active_element()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok());

        Self {
            open: false,
            last_active,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::Open => {
                if !self.open {
                    if let Some(dialog_node) = props.node_ref.get() {
                        crate::show_modal_dialog(dialog_node);
                        self.open = true;
                    }
                }
            }
            Msg::Close => {
                if self.open {
                    if let Some(on_close) = &props.on_close {
                        if let Some(dialog_node) = props.node_ref.get() {
                            crate::close_dialog(dialog_node);
                        }

                        on_close.emit(());
                        self.open = false;

                        self.restore_focus();
                    }
                }
            }
        }
        false
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();
        // always close the dialog before restoring the focus
        if let Some(dialog_node) = props.node_ref.get() {
            crate::close_dialog(dialog_node);
        }
        self.restore_focus();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link().clone();

        let on_close = link.callback(|_| Msg::Close);

        let oncancel = link.callback(|event: Event| {
            event.stop_propagation();
            event.prevent_default();
            Msg::Close
        });

        let mut panel = Panel::new()
            .class("pwt-overflow-auto")
            .title(props.title.clone())
            .border(false);

        if props.on_close.is_some() {
            panel.add_tool(
                Button::new("×")
                    .aria_label("Close Dialog")
                    .class("circle")
                    .class("pwt-scheme-neutral-alt")
                    .onclick(on_close)
            );
        };

        for child in &props.children {
            panel.add_child(child.clone());
        }

        html! {
            <dialog aria-label={props.title.clone()} ref={props.node_ref.clone()} {oncancel} style={props.style.clone()}>
            {panel}
            </dialog>
        }
    }
}

impl Into<VNode> for Dialog {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDialog>(Rc::new(self), key);
        VNode::from(comp)
    }
}
