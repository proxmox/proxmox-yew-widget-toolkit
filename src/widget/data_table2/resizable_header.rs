use std::rc::Rc;

use gloo_events::EventListener;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::UnwrapThrowExt;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};


use crate::prelude::*;
use crate::props::RenderFn;
use crate::widget::{Row, Container, SizeObserver};
use crate::widget::dropdown::focus_selected_element;


// Note about node_ref property: make it optional, and generate an
// unique one in Component::create(). That way we can clone Properies without
// generating NodeRef duplicates!

#[derive(Clone, PartialEq, Properties)]
pub struct ResizableHeader {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    /// Unique element ID
    pub id: Option<String>,
    pub tabindex: Option<i32>,

    pub content: Option<VNode>,
    pub on_resize: Option<Callback<f64>>,
    pub on_size_reset: Option<Callback<()>>,
    pub on_size_change: Option<Callback<f64>>,

    /// Function to generate the header menu.
    pub picker: Option<RenderFn<()>>,
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

    /// Builder style method to set the element id
    pub fn id(mut self, id: impl IntoPropValue<Option<String>>) -> Self {
        self.id = id.into_prop_value();
        self
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: impl IntoPropValue<Option<NodeRef>>) -> Self {
        self.node_ref = node_ref.into_prop_value();
        self
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Builder style method to set the html tabindex attribute
    pub fn tabindex(mut self, index: impl IntoPropValue<Option<i32>>) -> Self {
        self.set_tabindex(index);
        self
    }

    /// Method to set the html tabindex attribute
    pub fn set_tabindex(&mut self, index: impl IntoPropValue<Option<i32>>) {
        self.tabindex = index.into_prop_value();
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
    pub fn on_resize(mut self, cb: impl IntoEventCallback<f64>) -> Self {
        self.on_resize = cb.into_event_callback();
        self
    }

    /// Builder style method to set the size reset callback (DblClick on the resize handle)
    pub fn on_size_reset(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_size_reset = cb.into_event_callback();
        self
    }

    /// Builder style method to set the size change callback
    pub fn on_size_change(mut self, cb: impl IntoEventCallback<f64>) -> Self {
        self.on_size_change = cb.into_event_callback();
        self
    }

    /// Builder style method to set the menu render callback.
    pub fn picker(mut self, render_fn: impl Into<RenderFn<()>>) -> Self {
        self.picker = Some(render_fn.into());
        self
    }

}

pub enum Msg {
    StartResize,
    StopResize,
    MouseMove(i32),
    FocusChange(bool),
    TogglePicker,
    ShowPicker,
    HidePicker,
    PickerClosed,
}

#[doc(hidden)]
pub struct PwtResizableHeader {
    node_ref: NodeRef,
    width: f64,
    mousemove_listener: Option<EventListener>,
    mouseup_listener: Option<EventListener>,
    size_observer: Option<SizeObserver>,
    has_focus: bool,
    picker_ref: NodeRef,
    show_picker: bool,
    last_show_picker: bool,
    popper: Option<JsValue>,
    mousedown_listener: Option<EventListener>,
}

impl PwtResizableHeader {
    // focus the menu header (after closing the menu dialog)
    // just to be sure (Dialog should do this automatically)
    fn restore_focus(&mut self) {
        if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
            let _ = el.focus();
        }
    }
}

impl Component for PwtResizableHeader {
    type Message = Msg;
    type Properties =  ResizableHeader;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            width: 0.0,
            mousemove_listener: None,
            mouseup_listener: None,
            mousedown_listener: None,
            size_observer: None,
            has_focus: false,
            picker_ref: NodeRef::default(),
            show_picker: false,
            last_show_picker: false,
            popper: None,
         }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::MouseMove(x) => {
                if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                    let rect = el.get_bounding_client_rect();
                    let new_width = (x as f64) - rect.x();
                    self.width = new_width.max(0.0);
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
            Msg::TogglePicker => {
                //log::info!("TogglePicker");
                yew::Component::update(self, ctx, if self.show_picker { Msg::HidePicker } else {Msg::ShowPicker})
            }
            Msg::HidePicker => {
                // Note: close_dialog() is async, so we use the
                // onclose handler (Msg::PickerClosed) to wait for
                // the real close (else restore_focus() does not work)
                if let Some(dialog_node) = self.picker_ref.get() {
                    crate::close_dialog(dialog_node);
                }
                //log::info!("HidePicker {}", self.show_picker);
                false
            }
            Msg::PickerClosed => {
                //log::info!("PickerClosed");
                self.show_picker = false;
                self.restore_focus();
                true

            }
            Msg::ShowPicker => {
                self.show_picker = true;
                //log::info!("ShowPicker {}", self.show_picker);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Row::new()
            .class("pwt-datatable2-header-item")
            .class(self.show_picker.then(|| "focused"))
            .class(props.class.clone())
            .attribute("tabindex", props.tabindex.map(|t| t.to_string()))
            .attribute("id", props.id.clone())
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
                    .class((self.has_focus || self.show_picker).then(|| "focused"))
                    .ondblclick(|event: MouseEvent| event.stop_propagation())
                    .onclick(ctx.link().callback(|event: MouseEvent| {
                        event.stop_propagation();
                        Msg::TogglePicker
                    }))
                    .with_child(html!{<i class="fa fa-lg fa-caret-down"/>})
            )
            .with_child(
                Container::new()
                     .class("pwt-datatable2-header-resize-trigger")
                     .onmousedown(ctx.link().callback(|_| Msg::StartResize))
                     .ondblclick({
                         let on_size_reset = props.on_size_reset.clone();
                         move |event: MouseEvent| {
                             event.stop_propagation();
                             if let Some(on_size_reset) = &on_size_reset {
                                 on_size_reset.emit(());
                             }
                         }
                     })
            )
            .with_child(
                Container::new()
                    .tag("dialog")
                    .node_ref(self.picker_ref.clone())
                    .onclose(ctx.link().callback(|_| Msg::PickerClosed))
                    .oncancel(ctx.link().callback(|event: Event| {
                        event.stop_propagation();
                        event.prevent_default();
                        Msg::HidePicker
                    }))
                    .ondblclick(|event: MouseEvent| event.stop_propagation())
                    .onclick(|event: MouseEvent| event.stop_propagation())
                    .class("pwt-dropdown")
                    .attribute("data-show", self.show_picker.then(|| ""))
                    .with_optional_child(self.show_picker.then(|| {
                        if let Some(picker) = &props.picker {
                            picker.apply(&())
                        } else {
                            html!{<div class="pwt-p-2">{"No Menu configured"}</div>}
                        }
                    }))
            )
            .into()
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // always close the dialog
        if let Some(dialog_node) = self.picker_ref.get() {
            crate::close_dialog(dialog_node);
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
                let on_size_change = props.on_size_change.clone();
                //let width = el.offset_width();
                //on_size_change.as_ref().map(move |cb| cb.emit(width));
                self.size_observer = Some(SizeObserver::new(&el, move |(x, _y)| {
                    log::info!("SZ {}", x);
                    if let Some(on_size_change) = &on_size_change {
                        on_size_change.emit(x);
                    }
                }));
            }

            let opts = serde_json::json!({
                "placement": "bottom-start",
                "strategy": "fixed",
                "modifiers": [
                    {
                        "name": "preventOverflow",
                        "options": {
                            "mainAxis": true, // true by default
                            "altAxis": true, // false by default
                        },
                    },
                    {
                        "name": "flip",
                        "options": {
                            "fallbackPlacements": [], // disable fallbacks
                        },
                    },
                ],
            });

            let opts = crate::to_js_value(&opts).unwrap();

            if let Some(content_node) = self.node_ref.get() {
                if let Some(picker_node) = self.picker_ref.get() {
                    self.popper = Some(crate::create_popper(content_node, picker_node, &opts));
                }
            }

            let window = web_sys::window().unwrap();
            let picker_ref = self.picker_ref.clone();
            let link = ctx.link().clone();

            self.mousedown_listener = Some(EventListener::new(
                &window,
                "mousedown",
                move |e: &Event| {
                    let e = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();

                    if let Some(el) = picker_ref.cast::<web_sys::Element>() {
                        let x = e.client_x() as f64;
                        let y = e.client_y() as f64;

                        let rect = el.get_bounding_client_rect();
                        if x > rect.left() && x < rect.right() && y > rect.top() && y < rect.bottom() {
                            return;
                        }

                        link.send_message(Msg::HidePicker);
                    }
                },
            ));

        }

        if let Some(popper) = &self.popper {
            crate::update_popper(popper);
        }

        if self.show_picker != self.last_show_picker {
            self.last_show_picker = self.show_picker;
            if let Some(dialog_node) = self.picker_ref.get() {
                if self.show_picker {
                    crate::show_modal_dialog(dialog_node);
                    focus_selected_element(&self.picker_ref);
                } else {
                    crate::close_dialog(dialog_node);
                }
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
