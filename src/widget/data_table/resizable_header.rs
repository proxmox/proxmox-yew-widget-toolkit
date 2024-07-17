use std::rc::Rc;

use gloo_events::EventListener;
use indexmap::IndexMap;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::css::ColorScheme;
use crate::dom::element_direction_rtl;
use crate::prelude::*;
use crate::props::{BuilderFn, IntoOptionalBuilderFn};
use crate::widget::focus::FocusTracker;
use crate::widget::menu::{Menu, MenuButton};
use crate::widget::{Container, Row, SizeObserver};

// Note about node_ref property: make it optional, and generate an
// unique one in Component::create(). That way we can clone Properies without
// generating NodeRef duplicates!

#[derive(Clone, PartialEq, Properties)]
#[doc(hidden)] // only used inside this crate
pub struct ResizableHeader {
    #[prop_or_default]
    pub node_ref: Option<NodeRef>,
    #[prop_or_default]
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    /// Unique element ID
    #[prop_or_default]
    pub id: Option<String>,

    #[prop_or_default]
    pub attributes: IndexMap<AttrValue, AttrValue>,

    #[prop_or_default]
    pub content: Option<VNode>,

    /// Resizable flag.
    #[prop_or(true)]
    pub resizable: bool,

    /// Show menu flag.
    #[prop_or(true)]
    pub show_menu: bool,

    #[prop_or_default]
    pub on_resize: Option<Callback<f64>>,
    #[prop_or_default]
    pub on_size_reset: Option<Callback<()>>,
    #[prop_or_default]
    pub on_size_change: Option<Callback<f64>>,

    /// Function to generate the header menu.
    #[prop_or_default]
    pub menu_builder: Option<BuilderFn<Menu>>,
}

impl ResizableHeader {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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

    /// Builder stzyle method to set additional html attributes.
    pub fn attributes(mut self, attributes: IndexMap<AttrValue, AttrValue>) -> Self {
        self.attributes = attributes;
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

    /// Builder style method to set the resizable flag.
    pub fn resizable(mut self, hidden: bool) -> Self {
        self.set_resizable(hidden);
        self
    }

    /// Method to set the resizable flag.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    /// Builder style method to set the show_menu flag.
    pub fn show_menu(mut self, hidden: bool) -> Self {
        self.set_show_menu(hidden);
        self
    }

    /// Method to set the show_menu flag.
    pub fn set_show_menu(&mut self, show_menu: bool) {
        self.show_menu = show_menu;
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
    ShowPicker,
    HidePicker,
}

#[doc(hidden)]
pub struct PwtResizableHeader {
    rtl: Option<bool>,
    node_ref: NodeRef,
    width: f64,
    pointermove_listener: Option<EventListener>,
    pointerup_listener: Option<EventListener>,
    size_observer: Option<SizeObserver>,
    has_focus: bool,
    picker_ref: NodeRef,
    show_picker: bool,
    focus_tracker: FocusTracker,
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
    type Properties = ResizableHeader;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let focus_tracker = FocusTracker::new(ctx.link().callback(Msg::FocusChange));

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            rtl: None,
            width: 0.0,
            pointermove_listener: None,
            pointerup_listener: None,
            size_observer: None,
            has_focus: false,
            picker_ref: NodeRef::default(),
            show_picker: false,
            focus_tracker,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        if self.rtl.is_none() {
            self.rtl = element_direction_rtl(&self.node_ref);
        }

        match msg {
            Msg::MouseMove(x) => {
                if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                    let rect = el.get_bounding_client_rect();
                    let rtl = self.rtl.unwrap_or(false);
                    let new_width = if rtl {
                        rect.right() - (x as f64)
                    } else {
                        (x as f64) - rect.x()
                    };
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
                self.pointerup_listener = None;
                self.pointermove_listener = None;
                false
            }
            Msg::StartResize => {
                self.rtl = element_direction_rtl(&self.node_ref);

                let window = web_sys::window().unwrap();
                let link = ctx.link();
                let onpointermove = link.callback(|e: Event| {
                    let event = e.dyn_ref::<web_sys::PointerEvent>().unwrap_throw();
                    Msg::MouseMove(event.client_x())
                });
                let pointermove_listener = EventListener::new(&window, "pointermove", move |e| {
                    onpointermove.emit(e.clone())
                });
                self.pointermove_listener = Some(pointermove_listener);

                let onpointerup = link.callback(|_: Event| Msg::StopResize);
                let pointerup_listener =
                    EventListener::new(&window, "pointerup", move |e| onpointerup.emit(e.clone()));
                self.pointerup_listener = Some(pointerup_listener);

                false
            }
            Msg::FocusChange(has_focus) => {
                self.has_focus = has_focus;
                if has_focus {
                    self.rtl = element_direction_rtl(&self.node_ref);
                }
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

        let mut row = Row::new()
            .node_ref(self.node_ref.clone())
            .attribute("role", "none")
            .class("pwt-datatable-header-item")
            .class(self.has_focus.then(|| "focused"))
            .class(props.class.clone())
            .attribute("id", props.id.clone())
            .onfocusin(self.focus_tracker.get_focus_callback(true))
            .onfocusout(self.focus_tracker.get_focus_callback(false))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| match event.key().as_str() {
                    "ArrowDown" => {
                        event.stop_propagation();
                        link.send_message(Msg::ShowPicker);
                    }
                    _ => {}
                }
            })
            .with_child(
                Container::new()
                    .attribute("role", "none")
                    .class("pwt-datatable-header-content")
                    .with_optional_child(props.content.clone()),
            );

        for (name, value) in &props.attributes {
            row.set_attribute(name.clone(), value);
        }

        let mut anchor = Container::new().class("pwt-datatable-header-anchor");

        if props.show_menu {
            anchor.add_child(
                MenuButton::new("")
                    .node_ref(self.picker_ref.clone())
                    .tabindex(-1)
                    .autoshow_menu(true)
                    .class("pwt-datatable-header-menu-trigger pwt-button-text")
                    .class(ColorScheme::Primary)
                    .class((self.has_focus || self.show_picker).then(|| "focused"))
                    .icon_class("fa fa-lg fa-caret-down")
                    .ondblclick(|event: MouseEvent| event.stop_propagation())
                    .menu_builder(props.menu_builder.clone())
                    .on_close(ctx.link().callback(|_| Msg::HidePicker)),
            );
        }

        if props.resizable {
            anchor.add_child(
                Container::new()
                    .attribute("role", "none")
                    .class("pwt-datatable-header-resize-trigger")
                    .onpointerdown(ctx.link().callback(|_| Msg::StartResize))
                    .onclick(|event: MouseEvent| {
                        event.stop_propagation();
                        event.prevent_default();
                    })
                    .ondblclick({
                        let on_size_reset = props.on_size_reset.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            if let Some(on_size_reset) = &on_size_reset {
                                on_size_reset.emit(());
                            }
                        }
                    }),
            );
        }

        if !anchor.children.is_empty() {
            row.add_child(anchor);
        }

        row.into()
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
