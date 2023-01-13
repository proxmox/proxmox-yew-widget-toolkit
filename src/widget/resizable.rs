use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode, Key};

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;

#[derive(Clone, PartialEq, Properties)]
pub struct Resizable {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    pub on_resize: Option<Callback<i32>>,

    #[prop_or_default]
    pub vertical: bool,

    pub child: VNode,
}

impl Resizable {
    /// Creates a new instance
    pub fn new(child: impl Into<Html>) -> Self {
        yew::props!(Self { child: child.into() })
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.set_vertical(vertical);
        self
    }

    pub fn set_vertical(&mut self, vertical: bool) {
        self.vertical = vertical;
    }
}

pub struct PwtResizable {
    node_ref: NodeRef,
    size: i32,
    mousemove_listener: Option<EventListener>,
    mouseup_listener: Option<EventListener>,
}


pub enum Msg {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ResetSize,
    StartResize,
    StopResize,
    MouseMove(i32, i32)
}

impl Component for PwtResizable {
    type Message = Msg;
    type Properties = Resizable;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            size: 0,
            mousemove_listener: None,
            mouseup_listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::MouseMove(x, y) => {
                if let Some(el) = self.node_ref.cast::<web_sys::Element>() {

                    let rect = el.get_bounding_client_rect();

                    let new_size =  if props.vertical {
                        y - (rect.y() as i32) + 4
                    } else {
                        x - (rect.x() as i32) + 4
                    };
                    self.size = new_size.max(10);
                    if let Some(on_resize) = &props.on_resize {
                        on_resize.emit(self.size);
                    }
                }
                true
            }
            Msg::ArrowUp => {
                if !props.vertical { return false; }
                if self.size == 0 {
                    if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                        let rect = el.get_bounding_client_rect();
                        self.size = rect.y() as i32 - 11;
                    }
                }
                self.size = (self.size - 1).max(10);
                true
            }
            Msg::ArrowDown => {
                if !props.vertical { return false; }
                if self.size == 0 {
                    if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                        let rect = el.get_bounding_client_rect();
                        self.size = rect.y() as i32 - 11;
                    }
                }
                self.size = (self.size + 1).max(10);
                true
            }
            Msg::ArrowLeft => {
                if props.vertical { return false; }
                if self.size == 0 {
                    if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                        let rect = el.get_bounding_client_rect();
                        self.size = rect.x() as i32 - 15;
                    }
                }
                self.size = (self.size - 1).max(10);
                true
            }
            Msg::ArrowRight => {
                if props.vertical { return false; }
                if self.size == 0 {
                    if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                        let rect = el.get_bounding_client_rect();
                        self.size = rect.x() as i32 - 15;
                    }
                }
                self.size = (self.size + 1).max(10);
                true
            }
            Msg::ResetSize => {
                self.size = 0;
                if let Some(on_resize) = &props.on_resize {
                    on_resize.emit(self.size);
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
                    Msg::MouseMove(event.client_x(), event.client_y())
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onmousedown = ctx.link().callback(|_| Msg::StartResize);
        let ondblclick = ctx.link().callback(|_| Msg::ResetSize);

        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                let key: &str = &event.key();
                match key {
                    "Enter" => {
                        event.stop_propagation();
                        link.send_message(Msg::ResetSize);
                    }
                    "ArrowUp" => {
                        event.stop_propagation();
                        link.send_message(Msg::ArrowUp);
                    }
                    "ArrowDown" => {
                        event.stop_propagation();
                        link.send_message(Msg::ArrowDown);
                    }
                    "ArrowLeft" => {
                        event.stop_propagation();
                        link.send_message(Msg::ArrowLeft);
                    }
                    "ArrowRight" => {
                        event.stop_propagation();
                        link.send_message(Msg::ArrowRight);
                    }
                    _ => {}
                }
            }
        });

        let child_style = if self.size > 0 {
            if props.vertical {
                Some(format!("height:{}px;", self.size))
            } else {
                Some(format!("width:{}px;", self.size))
            }
        } else {
            None
        };

        let style = if props.vertical {
            "display:flex;flex-direction:column;align-items:stretch;"
        } else {
            "display:flex;flex-direction:row;align-items:stretch;"
        };

        let splitter_class =  if props.vertical { "column-split-handle" } else { "row-split-handle" };
        html! {
            <div ref={self.node_ref.clone()} style={style}>
                <div style={child_style} class="pwt-flex-fill pwt-overflow-auto">{props.child.clone()}</div>
                <div tabindex="0" style="flex: 0 0 auto;" {onkeydown} {onmousedown} {ondblclick} class={splitter_class}/>
            </div>
        }
    }
}

impl Into<VNode> for Resizable {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtResizable>(Rc::new(self), key);
        VNode::from(comp)
    }
}
