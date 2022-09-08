use std::rc::Rc;

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};


use crate::prelude::*;
use crate::widget::{Row, Container, SizeObserver};

// Note about node_ref property: make it optional, and generate an
// unique one in Component::create(). That way we can clone Properies without
// generating NodeRef duplicates!

#[derive(Clone, PartialEq, Properties)]
pub struct ResizableHeader {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,
    pub content: Option<VNode>,
    pub on_resize: Option<Callback<i32>>,
    pub on_size_reset: Option<Callback<()>>,
    pub on_size_change: Option<Callback<i32>>,
}

impl ResizableHeader {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: impl IntoPropValue<Option<NodeRef>>) -> Self {
        self.node_ref = node_ref.into_prop_value();
        self
    }

    /// Builder style method to set the header text
    pub fn content(mut self, content: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_content(content);
        self
    }

    /// Method to set the header text
    pub fn set_content(&mut self, content: impl IntoPropValue<Option<VNode>>) {
        self.content = content.into_prop_value();
    }

    /// Builder style method to set the resize callback
    pub fn on_resize(mut self, cb: impl IntoEventCallback<i32>) -> Self {
        self.on_resize = cb.into_event_callback();
        self
    }

    /// Builder style method to set the size reset callback (DblClick on the resize handle)
    pub fn on_size_reset(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_size_reset = cb.into_event_callback();
        self
    }

    /// Builder style method to set the size change callback
    pub fn on_size_change(mut self, cb: impl IntoEventCallback<i32>) -> Self {
        self.on_size_change = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    StartResize,
    StopResize,
    MouseMove(i32),
    FocusChange(bool),
}

#[doc(hidden)]
pub struct PwtResizableHeader {
    node_ref: NodeRef,
    width: i32,
    mousemove_listener: Option<EventListener>,
    mouseup_listener: Option<EventListener>,
    size_observer: Option<SizeObserver>,
    has_focus: bool,
}

impl Component for PwtResizableHeader {
    type Message = Msg;
    type Properties =  ResizableHeader;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            width: 0,
            mousemove_listener: None,
            mouseup_listener: None,
            size_observer: None,
            has_focus: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::MouseMove(x) => {
                if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                    let rect = el.get_bounding_client_rect();
                    let new_width = x - (rect.x() as i32);
                    //log::info!("MOVE {} {} {} {}", el.client_left(), rect.x(), x, new_width);
                    self.width = new_width; //.max(40);
                    if let Some(on_resize) = &props.on_resize {
                        on_resize.emit(self.width);
                    }
                } else {
                    unreachable!();
                }
                true
            }
            Msg::StopResize => {
                self.mouseup_listener = None;
                self.mousemove_listener = None;
                false
            }
            Msg::StartResize => {
                let window = web_sys::window().unwrap();
                let link = ctx.link();
                let onmousemove = link.callback(|e: Event| {
                    let event = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
                    Msg::MouseMove(event.client_x())
                });
                let mousemove_listener = EventListener::new(
                    &window,
                    "mousemove",
                    move |e| onmousemove.emit(e.clone()),
                );
                self.mousemove_listener = Some(mousemove_listener);

                let onmouseup = link.callback(|_: Event| Msg::StopResize);
                let mouseup_listener = EventListener::new(
                    &window,
                    "mouseup",
                    move |e| onmouseup.emit(e.clone()),
                );
                self.mouseup_listener = Some(mouseup_listener);

                false
            }
            Msg::FocusChange(has_focus) => {
                self.has_focus = has_focus;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Row::new()
            .class("pwt-w-100 pwt-h-100")
            .class("pwt-datatable2-header-item")
            .attribute("tabindex", "-1")
            .node_ref(self.node_ref.clone())
            .onfocus(ctx.link().callback(|_| Msg::FocusChange(true)))
            .onblur(ctx.link().callback(|_| Msg::FocusChange(false)))
            .with_child(
                Container::new()
                    .class("pwt-align-self-center")
                    .class("pwt-text-truncate")
                    .with_optional_child(props.content.clone())
            )
            .with_child(
                Container::new()
                    .class("pwt-datatable2-header-menu-trigger")
                    .class(self.has_focus.then(|| "focused"))
                    .with_child(html!{<i class="fa fa-lg fa-caret-down"/>})
            )
            .with_child(
                Container::new()
                     .class("pwt-datatable2-header-resize-trigger")
                     .onmousedown(ctx.link().callback(|_| Msg::StartResize))
                     .ondblclick({
                         let on_size_reset = props.on_size_reset.clone();
                         move |_| {
                             if let Some(on_size_reset) = &on_size_reset {
                                 on_size_reset.emit(());
                             }
                         }
                     })
             )
        /*
            .with_child(
                Container::new()
                    .class("pwt-datatable2-header-resize-separator")
                    .attribute("style", format!("left: {}px", self.width))


            )
         */
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
                let on_size_change = props.on_size_change.clone();
                //let width = el.offset_width();
                //on_size_change.as_ref().map(move |cb| cb.emit(width));
                self.size_observer = Some(SizeObserver::new(&el, move |(x, _y)| {
                    if let Some(on_size_change) = &on_size_change {
                        on_size_change.emit(x + /* border size */ 1);
                    }
                }));
            }
        }
    }
}

impl Into<VNode> for ResizableHeader {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtResizableHeader>(Rc::new(self), key);
        VNode::from(comp)
    }

}
