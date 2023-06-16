use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::state::ThemeObserver;
use crate::widget::dom::IntoHtmlElement;
use crate::widget::Container;

use super::{GestureDetector, GestureDragEvent, GestureSwipeEvent};

#[derive(Copy, Clone, PartialEq)]
pub enum SideDialogDirection {
    Left,
    Right,
    Top,
    Bottom,
}

use pwt_macros::builder;

/// Modal Dialog with slide in/out animations.
///
/// This widget is implemented using the relatively new Html `<dialog>`
/// tag in order to get correct focus handling.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct SideDialog {
    #[prop_or_default]
    node_ref: NodeRef,

    /// The yew component key.
    pub key: Option<Key>,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_close: Option<Callback<()>>,

    #[prop_or_default]
    pub children: Vec<VNode>,

    #[prop_or(SideDialogDirection::Left)]
    #[builder]
    pub direction: SideDialogDirection,
}

impl ContainerBuilder for SideDialog {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

impl SideDialog {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
}

pub enum Msg {
    Open,
    Close,
    Dismiss, // Slide out, then close
    SliderAnimationEnd,
    DragStart(GestureDragEvent),
    DragEnd(GestureDragEvent),
    Drag(GestureDragEvent),
    Swipe(GestureSwipeEvent),
}

#[derive(Copy, Clone, PartialEq)]
enum SliderState {
    Hidden,
    Visible,
    SlideIn,
    SlideOut,
}

#[doc(hidden)]
pub struct PwtSideDialog {
    open: bool,
    last_active: Option<web_sys::HtmlElement>, // last focused element
    slider_ref: NodeRef,
    slider_state: SliderState,
    drag_start: Option<(f64, f64)>,
    drag_delta: Option<(f64, f64)>,
}

impl PwtSideDialog {
    fn restore_focus(&mut self) {
        if let Some(el) = self.last_active.take() {
            let _ = el.focus();
        }
    }
}

impl Component for PwtSideDialog {
    type Message = Msg;
    type Properties = SideDialog;

    fn create(_ctx: &Context<Self>) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let last_active = document
            .active_element()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok());

        Self {
            open: false,
            last_active,
            slider_ref: NodeRef::default(),
            slider_state: SliderState::Hidden,
            drag_start: None,
            drag_delta: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::Open => {
                if !self.open {
                    self.slider_state = SliderState::SlideIn;

                    if let Some(dialog_node) = props.node_ref.get() {
                        crate::show_modal_dialog(dialog_node);
                        self.open = true;
                    }
                }
                true
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
                false
            }
            Msg::Dismiss => {
                if self.slider_state == SliderState::Visible {
                    self.slider_state = SliderState::SlideOut;
                    true
                } else {
                    false
                }
            }
            Msg::SliderAnimationEnd => {
                self.slider_state = match self.slider_state {
                    SliderState::SlideIn => SliderState::Visible,
                    SliderState::SlideOut => {
                        ctx.link().send_message(Msg::Close);
                        SliderState::Hidden
                    }
                    _ => self.slider_state,
                };
                true
            }
            Msg::DragStart(event) => {
                let x = event.x() as f64;
                let y = event.y() as f64;
                if x > 0.0 && y > 0.0 {
                    // prevent divide by zero
                    self.drag_start = Some((x, y));
                    self.drag_delta = Some((0.0, 0.0));
                }
                false
            }
            Msg::DragEnd(_event) => {
                let mut dismiss = false;
                let threshold = 100.0;
                if let Some((delta_x, delta_y)) = self.drag_delta {
                    dismiss = match props.direction {
                        SideDialogDirection::Left => delta_x < -threshold,
                        SideDialogDirection::Right => delta_x > threshold,
                        SideDialogDirection::Top => delta_y < -threshold,
                        SideDialogDirection::Bottom => delta_y > threshold,
                    };
                }
                self.drag_start = None;
                self.drag_delta = None;

                if dismiss {
                    ctx.link().send_message(Msg::Dismiss);
                }
                true
            }
            Msg::Drag(event) => {
                if let Some(start) = &self.drag_start {
                    let x = event.x() as f64;
                    let y = event.y() as f64;

                    self.drag_delta = Some((x - start.0, y - start.1));
                }
                true
            }
            Msg::Swipe(event) => {
                let angle = event.direction; // -180 to + 180
                let dismiss = match props.direction {
                    SideDialogDirection::Left => angle > 135.0 || angle < -135.0,
                    SideDialogDirection::Right => angle > -45.0 && angle < 45.0,
                    SideDialogDirection::Top => angle > 45.0 && angle < 135.0,
                    SideDialogDirection::Bottom => angle > -135.0 && angle < -45.0,
                };
                if dismiss {
                    ctx.link().send_message(Msg::Dismiss);
                }
                true
            }
        }
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

        let oncancel = link.callback(|event: Event| {
            event.stop_propagation();
            event.prevent_default();
            Msg::Dismiss
        });

        let slider_state_class = match self.slider_state {
            SliderState::Hidden | SliderState::SlideOut => "hidden",
            SliderState::Visible | SliderState::SlideIn => "visible",
        };

        let slider_direction_class = match props.direction {
            SideDialogDirection::Left => "pwt-side-dialog-left",
            SideDialogDirection::Right => "pwt-side-dialog-right",
            SideDialogDirection::Top => "pwt-side-dialog-top",
            SideDialogDirection::Bottom => "pwt-side-dialog-bottom",
        };

        let mut transform = None;
        if let Some((delta_x, delta_y)) = self.drag_delta {
            match props.direction {
                SideDialogDirection::Left => {
                    let delta_x = delta_x.min(0.0);
                    transform = Some(format!(
                        "transition:none;transform: translateX({}px);",
                        delta_x
                    ));
                }
                SideDialogDirection::Right => {
                    let delta_x = delta_x.max(0.0);
                    transform = Some(format!(
                        "transition:none;transform: translateX({}px);",
                        delta_x
                    ));
                }
                SideDialogDirection::Top => {
                    let delta_y = delta_y.min(0.0);
                    transform = Some(format!(
                        "transition:none;transform: translateY({}px);",
                        delta_y
                    ));
                }
                SideDialogDirection::Bottom => {
                    let delta_y = delta_y.max(0.0);
                    transform = Some(format!(
                        "transition:none;transform: translateY({}px);",
                        delta_y
                    ));
                }
            }
        }

        let dialog = Container::new()
            .node_ref(props.node_ref.clone())
            .tag("dialog")
            .class("pwt-side-dialog")
            .class(slider_state_class)
            .oncancel(oncancel)
            .with_child(
                Container::new()
                    .node_ref(self.slider_ref.clone())
                    .class("pwt-side-dialog-slider")
                    .class(slider_direction_class)
                    .class(slider_state_class)
                    .attribute("style", transform)
                    .ontransitionend(ctx.link().callback(|_| Msg::SliderAnimationEnd))
                    .children(props.children.clone()),
            );

        GestureDetector::new(dialog)
            .on_tap({
                let slider_ref = self.slider_ref.clone();
                let link = ctx.link().clone();
                move |event: PointerEvent| {
                    if let Some(element) = slider_ref.clone().into_html_element() {
                        let rect = element.get_bounding_client_rect();
                        let x = event.client_x() as f64;
                        let y = event.client_y() as f64;

                        if (rect.left() < x)
                            && (x < rect.right())
                            && (rect.top() < y)
                            && (y < rect.bottom())
                        {
                            // click inside dialog
                        } else {
                            link.send_message(Msg::Dismiss);
                        }
                    }
                }
            })
            .on_drag_start(ctx.link().callback(Msg::DragStart))
            .on_drag_end(ctx.link().callback(Msg::DragEnd))
            .on_drag_update(ctx.link().callback(Msg::Drag))
            .on_swipe(ctx.link().callback(Msg::Swipe))
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::Open);
        }
    }
}

impl Into<VNode> for SideDialog {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtSideDialog>(Rc::new(self), key);
        VNode::from(comp)
    }
}
