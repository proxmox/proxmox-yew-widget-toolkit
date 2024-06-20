use std::collections::HashMap;
use std::rc::Rc;

use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{window, HtmlElement};

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::dom::IntoHtmlElement;
use crate::prelude::*;
use crate::props::{AsCssStylesMut, CssStyles, WidgetStyleBuilder};
use crate::widget::align::{align_to_viewport, align_to_xy, Point};
use crate::widget::{ActionIcon, Panel};

use pwt_macros::builder;

/// Modal Dialog.
///
/// This widget is implemented using the relatively new Html `<dialog>`
/// tag in order to get correct focus handling.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct Dialog {
    #[prop_or_default]
    node_ref: NodeRef,

    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Dialog Title (also used as 'arial-label')
    pub title: AttrValue,

    /// Title as Html
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub html_title: Option<Html>,

    /// Dialog close callback.
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,

    #[prop_or_default]
    pub children: Vec<VNode>,

    /// CSS style for the dialog window
    #[prop_or_default]
    pub styles: CssStyles,

    /// Determines if the dialog can be moved
    ///
    /// Makes it draggable by the title bar (exclusive the title text/tools)
    #[prop_or(true)]
    #[builder]
    pub draggable: bool,

    /// Determines if the dialog can be resized
    ///
    /// Adds a resizer on each edge and corner
    #[prop_or_default]
    #[builder]
    pub resizable: bool,

    /// Determines if the dialog should be auto centered
    ///
    /// It will be centered on every window resize
    /// This is enabled by default
    #[prop_or(true)]
    #[builder]
    pub auto_center: bool,
}

impl ContainerBuilder for Dialog {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

impl AsCssStylesMut for Dialog {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles {
        &mut self.styles
    }
}

impl WidgetStyleBuilder for Dialog {}

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
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }

    pub fn html(self) -> VNode {
        self.into()
    }
}

pub enum Msg {
    Open,
    Close,
    PointerDown(PointerEvent),
    PointerMove(PointerEvent),
    PointerUp(i32),
    ResizeStart(Point, PointerEvent),
    ResizeMove(Point, PointerEvent),
    ResizeUp(Point, i32),
    Center,
}

enum DragState {
    Idle,
    Dragging(f64, f64, EventListener, EventListener, i32),
}

#[doc(hidden)]
pub struct PwtDialog {
    open: bool,
    dragging_state: DragState,
    last_active: Option<web_sys::HtmlElement>, // last focused element
    resizer_state: HashMap<Point, DragState>,
    center_function: Option<Closure<dyn FnMut()>>,
    inner_ref: NodeRef,
}

impl PwtDialog {
    fn restore_focus(&mut self) {
        if let Some(el) = self.last_active.take() {
            let _ = el.focus();
        }
    }
}

impl Drop for PwtDialog {
    fn drop(&mut self) {
        if let Some(center_function) = self.center_function.take() {
            let window = web_sys::window().unwrap();
            window
                .remove_event_listener_with_callback(
                    "resize",
                    center_function.as_ref().unchecked_ref(),
                )
                .unwrap_throw();
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
        let last_active = document
            .active_element()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok());

        let link = ctx.link().clone();
        let center_function = ctx.props().auto_center.then_some({
            let center_function = Closure::new(move || {
                link.send_message(Msg::Center);
            });

            window
                .add_event_listener_with_callback(
                    "resize",
                    center_function.as_ref().unchecked_ref(),
                )
                .unwrap_throw();

            center_function
        });

        Self {
            open: false,
            dragging_state: DragState::Idle,
            resizer_state: HashMap::new(),
            last_active,
            center_function,
            inner_ref: Default::default(),
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
            Msg::PointerDown(event) => {
                let mut is_draggable = false;

                if let Some(target) = event.target() {
                    if let Some(target) = target.dyn_ref::<HtmlElement>() {
                        is_draggable = target.class_list().contains("pwt-draggable");
                    }
                }

                if props.draggable && is_draggable {
                    if let Some(element) = props.node_ref.clone().into_html_element() {
                        let client = element.get_bounding_client_rect();
                        let x = event.client_x() as f64 - client.x();
                        let y = event.client_y() as f64 - client.y();

                        let onmousemove = ctx.link().callback(Msg::PointerMove);
                        let onpointerup = ctx
                            .link()
                            .callback(|event: PointerEvent| Msg::PointerUp(event.pointer_id()));

                        self.dragging_state = DragState::Dragging(
                            x,
                            y,
                            EventListener::new(&window().unwrap(), "pointermove", move |event| {
                                onmousemove.emit(event.clone().dyn_into().unwrap());
                            }),
                            EventListener::new(&window().unwrap(), "pointerup", move |event| {
                                onpointerup.emit(event.clone().dyn_into().unwrap());
                            }),
                            event.pointer_id(),
                        );
                        return true;
                    }
                }
            }
            Msg::PointerMove(event) => match &self.dragging_state {
                DragState::Dragging(offset_x, offset_y, _, _, id) if *id == event.pointer_id() => {
                    let window = window().unwrap();
                    let width = window.inner_width().unwrap().as_f64().unwrap();
                    let height = window.inner_height().unwrap().as_f64().unwrap();
                    let x = (event.client_x() as f64).max(0.0).min(width) - offset_x;
                    let y = (event.client_y() as f64).max(0.0).min(height) - offset_y;
                    if let Err(err) = align_to_xy(props.node_ref.clone(), (x, y), Point::TopStart) {
                        log::error!("align_to_xy failed: {}", err.to_string());
                    }
                }
                _ => {}
            },
            Msg::PointerUp(pointer_id) => match &self.dragging_state {
                DragState::Dragging(_, _, _, _, id) if *id == pointer_id => {
                    self.dragging_state = DragState::Idle;
                    return true;
                }
                _ => {}
            },
            Msg::ResizeStart(point, event) => {
                let onpointermove = ctx
                    .link()
                    .callback(move |event| Msg::ResizeMove(point, event));
                let onpointerup = ctx
                    .link()
                    .callback(move |event: PointerEvent| Msg::ResizeUp(point, event.pointer_id()));

                let offset = if let Some(element) = self.inner_ref.clone().into_html_element() {
                    let rect = element.get_bounding_client_rect();
                    let x = match point {
                        Point::TopStart | Point::Start | Point::BottomStart => {
                            event.client_x() as f64 - rect.x()
                        }
                        Point::BottomEnd | Point::End | Point::TopEnd => {
                            rect.right() - event.client_x() as f64
                        }
                        _ => 0.0,
                    };

                    let y = match point {
                        Point::TopStart | Point::Top | Point::TopEnd => {
                            event.client_y() as f64 - rect.y()
                        }
                        Point::BottomStart | Point::BottomEnd | Point::Bottom => {
                            rect.bottom() - event.client_y() as f64
                        }
                        _ => 0.0,
                    };
                    (x, y)
                } else {
                    (0.0, 0.0)
                };

                self.resizer_state.insert(
                    point,
                    DragState::Dragging(
                        offset.0,
                        offset.1,
                        EventListener::new(&window().unwrap(), "pointermove", move |event| {
                            onpointermove.emit(event.clone().dyn_into().unwrap());
                        }),
                        EventListener::new(&window().unwrap(), "pointerup", move |event| {
                            onpointerup.emit(event.clone().dyn_into().unwrap());
                        }),
                        event.pointer_id(),
                    ),
                );
            }
            Msg::ResizeMove(point, event) => match self.resizer_state.get(&point) {
                Some(DragState::Dragging(x, y, _, _, id)) if *id == event.pointer_id() => {
                    if let Some(element) = self.inner_ref.clone().into_html_element() {
                        let rect = element.get_bounding_client_rect();
                        let old_width = rect.width();
                        let old_height = rect.height();

                        let viewport_height =
                            window().unwrap().inner_height().unwrap().as_f64().unwrap();
                        let viewport_width =
                            window().unwrap().inner_width().unwrap().as_f64().unwrap();

                        // restrict to viewport
                        let client_x = (event.client_x() as f64).clamp(5.0, viewport_width - 5.0);
                        let client_y = (event.client_y() as f64).clamp(5.0, viewport_height - 5.0);

                        let mut pos = (rect.x(), rect.y());

                        let new_width = match point {
                            Point::TopStart | Point::Start | Point::BottomStart => {
                                Some(rect.right() - client_x + x)
                            }
                            Point::TopEnd | Point::End | Point::BottomEnd => {
                                Some(client_x - pos.0 + x)
                            }
                            _ => None,
                        };

                        let new_height = match point {
                            Point::TopStart | Point::Top | Point::TopEnd => {
                                Some(rect.bottom() - client_y + y)
                            }
                            Point::BottomStart | Point::Bottom | Point::BottomEnd => {
                                Some(client_y - pos.1 + y)
                            }
                            _ => None,
                        };

                        if let Some(val) = new_width {
                            let _ = element.style().set_property("width", &format!("{val}px"));
                        }

                        if let Some(val) = new_height {
                            let _ = element.style().set_property("height", &format!("{val}px"));
                        }

                        // we set the size, but the real size could have been limited by min/man
                        // width/height etc. so get the real values, set them back to the style,
                        // and reposition only if they have changed and we were resized
                        // from top/left
                        let rect = element.get_bounding_client_rect();
                        let new_width = rect.width();
                        let new_height = rect.height();

                        let _ = element
                            .style()
                            .set_property("width", &format!("{new_width}px"));
                        let _ = element
                            .style()
                            .set_property("height", &format!("{new_height}px"));

                        pos.0 += match point {
                            Point::TopStart | Point::Start | Point::BottomStart => {
                                old_width - new_width
                            }
                            _ => 0.0,
                        };

                        pos.1 += match point {
                            Point::TopStart | Point::Top | Point::TopEnd => old_height - new_height,
                            _ => 0.0,
                        };

                        if pos.0 != rect.x() || pos.1 != rect.y() {
                            if let Err(err) =
                                align_to_xy(props.node_ref.clone(), pos, Point::TopStart)
                            {
                                log::error!("could not align dialog: {}", err.to_string())
                            }
                        }
                    }
                }
                _ => {}
            },
            Msg::ResizeUp(point, pointer_id) => match self.resizer_state.get(&point) {
                Some(DragState::Dragging(_, _, _, _, id)) if *id == pointer_id => {
                    self.resizer_state.remove(&point);
                }
                _ => {}
            },
            Msg::Center => {
                if let Err(err) =
                    align_to_viewport(props.node_ref.clone(), Point::Center, Point::Center)
                {
                    log::error!("err: {}", err.to_string());
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

        let onclose = oncancel.clone();

        let mut panel = Panel::new()
            .class("pwt-overflow-auto")
            .class("pwt-flex-fill")
            .title(
                props
                    .html_title
                    .clone()
                    .unwrap_or(html! {props.title.clone()}),
            )
            .header_class(props.draggable.then_some("pwt-draggable"))
            .border(false);

        if props.on_close.is_some() {
            panel.add_tool(
                ActionIcon::new("fa fa-close")
                    .aria_label("Close Dialog")
                    .on_activate(on_close),
            );
        };

        for child in &props.children {
            panel.add_child(child.clone());
        }

        let onpointerdown = link.callback(|event: PointerEvent| {
            event.stop_propagation();
            Msg::PointerDown(event)
        });

        let resizable = props.resizable;

        let west_down = link.callback(|e| Msg::ResizeStart(Point::Start, e));
        let east_down = link.callback(|e| Msg::ResizeStart(Point::End, e));
        let north_down = link.callback(|e| Msg::ResizeStart(Point::Top, e));
        let south_down = link.callback(|e| Msg::ResizeStart(Point::Bottom, e));

        let northwest_down = link.callback(|e| Msg::ResizeStart(Point::TopStart, e));
        let southwest_down = link.callback(|e| Msg::ResizeStart(Point::BottomStart, e));
        let northeast_down = link.callback(|e| Msg::ResizeStart(Point::TopEnd, e));
        let southeast_down = link.callback(|e| Msg::ResizeStart(Point::BottomEnd, e));

        let is_dragging = !matches!(self.dragging_state, DragState::Idle);
        let classes = classes!("pwt-dialog", is_dragging.then_some("pwt-user-select-none"));

        let style = props.styles.compile_style_attribute(None);

        html! {
            <dialog class={"pwt-outer-dialog"} {onpointerdown} aria-label={props.title.clone()} ref={props.node_ref.clone()} {oncancel} {onclose} >
                <div class={classes} {style} ref={self.inner_ref.clone()}>
                {panel}
                if resizable {
                    <div onpointerdown={west_down} class="dialog-resize-handle west"></div>
                    <div onpointerdown={east_down} class="dialog-resize-handle east"></div>
                    <div onpointerdown={north_down} class="dialog-resize-handle north"></div>
                    <div onpointerdown={south_down} class="dialog-resize-handle south"></div>
                    <div onpointerdown={northeast_down} class="dialog-resize-handle north-east"></div>
                    <div onpointerdown={northwest_down} class="dialog-resize-handle north-west"></div>
                    <div onpointerdown={southeast_down} class="dialog-resize-handle south-east"></div>
                    <div onpointerdown={southwest_down} class="dialog-resize-handle south-west"></div>
                }
                </div>
            </dialog>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
            // send the first center message in a timeout, so the browser has time to get
            // the sizes right first. The new position should be identical with
            // the automatic one, so this should not be visible anyway.
            Timeout::new(50, move || link.send_message(Msg::Center)).forget();
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
