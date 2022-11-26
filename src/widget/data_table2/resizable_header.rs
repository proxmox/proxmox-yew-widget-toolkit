use std::rc::Rc;

use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::props::{BuilderFn, IntoOptionalBuilderFn};
use crate::widget::{Menu, MenuButton, Row, Container, SizeObserver};

// Note about node_ref property: make it optional, and generate an
// unique one in Component::create(). That way we can clone Properies without
// generating NodeRef duplicates!

#[derive(Clone, PartialEq, Properties)]
#[doc(hidden)] // only used inside this crate
pub struct ResizableHeader {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    /// Unique element ID
    pub id: Option<String>,
    /// Html tabindex attribute.
    pub tabindex: Option<i32>,
    /// Optional ARIA label.
    pub aria_label: Option<AttrValue>,

    pub content: Option<VNode>,
    pub on_resize: Option<Callback<f64>>,
    pub on_size_reset: Option<Callback<()>>,
    pub on_size_change: Option<Callback<f64>>,

    /// Function to generate the header menu.
    pub menu_builder: Option<BuilderFn<Menu>>,
}

impl ResizableHeader {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /*
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
     */

    /// Builder style method to set the html aria-label attribute
    pub fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute
    pub fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.aria_label = label.into_prop_value();
    }

    /// Builder style method to set the element id
    pub fn id(mut self, id: impl IntoPropValue<Option<String>>) -> Self {
        self.id = id.into_prop_value();
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

    /// Builder style method to set the menu builder.
    pub fn menu_builder(mut self, builder: impl IntoOptionalBuilderFn<Menu>) -> Self {
        self.menu_builder = builder.into_optional_builder_fn();
        self
    }
}

pub enum Msg {
    StartResize,
    StopResize,
    MouseMove(i32),
    FocusChange(bool),
    DelayedFocusChange(bool),
    ShowPicker,
    HidePicker,
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
    timeout: Option<Timeout>,
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
            size_observer: None,
            has_focus: false,
            picker_ref: NodeRef::default(),
            show_picker: false,
            timeout: None,
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
                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(1, move || {
                    link.send_message(Msg::DelayedFocusChange(has_focus));
                }));
                false
            }
            Msg::DelayedFocusChange(has_focus) => {
                self.has_focus = has_focus;
                true
            }
            Msg::HidePicker => {
                //log::info!("HidePicker");
                self.show_picker = false;
                self.restore_focus();
                true
            }
            Msg::ShowPicker => {
                self.show_picker = true;
                //log::info!("ShowPicker {}", self.show_picker);
                if let Some(el) = self.picker_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Row::new()
            .attribute("role", "none")
            .class("pwt-datatable2-header-item")
            //.class(self.show_picker.then(|| "focused"))
            .class(self.has_focus.then(|| "focused"))
            .class(props.class.clone())
            .attribute("tabindex", props.tabindex.map(|t| t.to_string()))
            .attribute("id", props.id.clone())
            .attribute("aria-label", &props.aria_label)
            .node_ref(self.node_ref.clone())
            .onfocusin(ctx.link().callback(|_| Msg::FocusChange(true)))
            .onfocusout(ctx.link().callback(|_| Msg::FocusChange(false)))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        40 => { // arrow down
                            event.stop_propagation();
                            link.send_message(Msg::ShowPicker);
                        }
                        _ => {}
                    }
                }
            })
            .with_child(
                Container::new()
                    .attribute("role", "none")
                    .class("pwt-align-self-center")
                    .class("pwt-text-truncate")
                    .with_optional_child(props.content.clone())
            )
            .with_child(
                MenuButton::new("")
                    .node_ref(self.picker_ref.clone())
                    .tabindex(-1)
                    .autoshow_menu(true)
                    .class("pwt-datatable2-header-menu-trigger")
                    .class((self.has_focus || self.show_picker).then(|| "focused"))
                    .icon_class("fa fa-lg fa-caret-down")
                    .ondblclick(|event: MouseEvent| event.stop_propagation())
                    .menu_builder(props.menu_builder.clone())
                    .on_close(ctx.link().callback(|_| Msg::HidePicker))

            )
            .with_child(
                Container::new()
                    .attribute("role", "none")
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
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
                let on_size_change = props.on_size_change.clone();
                self.size_observer = Some(SizeObserver::new(&el, move |(x, _y)| {
                    if let Some(on_size_change) = &on_size_change {
                        on_size_change.emit(x);
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
