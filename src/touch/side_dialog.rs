use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{Element, EventTarget, HtmlElement};

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::dom::IntoHtmlElement;
use crate::props::{AsCssStylesMut, CssStyles};
use crate::state::{SharedState, SharedStateObserver};
use crate::touch::GestureDragEvent;
use crate::widget::Container;
use crate::{impl_yew_std_props_builder, prelude::*};

use super::{GestureDetector, GesturePhase, GestureSwipeEvent, InputEvent};

// Messages sent from the [SideDialogController].
pub enum SideDialogControllerMsg {
    Close, // Close the dialog
}

/// Side dialog controller can dismiss the dialog.
///
/// Each [SideDialog] provides a [SideDialogController] using a [yew::ContextProvider].
#[derive(Clone, PartialEq)]
pub struct SideDialogController {
    state: SharedState<Vec<SideDialogControllerMsg>>,
}

impl Default for SideDialogController {
    fn default() -> Self {
        Self::new()
    }
}

impl SideDialogController {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            state: SharedState::new(Vec::new()),
        }
    }

    /// Close the dialog.
    pub fn close_dialog(&self) {
        self.state.write().push(SideDialogControllerMsg::Close);
    }
}

/// Define the location where the dialog should be displayed.
#[derive(Copy, Clone, PartialEq)]
pub enum SideDialogLocation {
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
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Optional controller to trigger a dialog close.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub controller: Option<SideDialogController>,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,

    #[prop_or_default]
    pub children: Vec<VNode>,

    #[prop_or(SideDialogLocation::Left)]
    #[builder]
    pub location: SideDialogLocation,

    /// CSS style for the dialog window
    #[prop_or_default]
    pub styles: CssStyles,
}

impl AsCssStylesMut for SideDialog {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles {
        &mut self.styles
    }
}

impl WidgetStyleBuilder for SideDialog {}

impl ContainerBuilder for SideDialog {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

impl Default for SideDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SideDialog {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_yew_std_props_builder!();
}

pub enum Msg {
    Open,
    Close,
    Dismiss, // Slide out, then close
    SliderAnimationEnd,
    Drag(GestureDragEvent),
    Swipe(GestureSwipeEvent),
    Controller,
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
    node_ref: NodeRef,
    slider_ref: NodeRef,
    slider_state: SliderState,
    drag_start: Option<(f64, f64)>,
    drag_delta: Option<(f64, f64)>,
    controller: SideDialogController,
    _controller_observer: SharedStateObserver<Vec<SideDialogControllerMsg>>,
}

impl PwtSideDialog {
    fn restore_focus(&mut self) {
        if let Some(el) = self.last_active.take() {
            let _ = el.focus();
        }
    }
}

impl PwtSideDialog {
    fn handle_controller_messages(&mut self, ctx: &Context<Self>) {
        let count = self.controller.state.read().len();
        if count == 0 {
            // Note: avoid endless loop
            return;
        }

        let list = self.controller.state.write().split_off(0);

        for msg in list.into_iter() {
            match msg {
                SideDialogControllerMsg::Close => ctx.link().send_message(Msg::Dismiss),
            }
        }
    }
}

impl Component for PwtSideDialog {
    type Message = Msg;
    type Properties = SideDialog;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let last_active = gloo_utils::document()
            .active_element()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok());

        let controller = props.controller.clone().unwrap_or_default();

        let _controller_observer = controller
            .state
            .add_listener(ctx.link().callback(|_| Msg::Controller));

        Self {
            open: false,
            last_active,
            node_ref: NodeRef::default(),
            slider_ref: NodeRef::default(),
            slider_state: SliderState::Hidden,
            drag_start: None,
            drag_delta: None,
            controller,
            _controller_observer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::Open => {
                if !self.open {
                    self.slider_state = SliderState::SlideIn;

                    if let Some(dialog_node) = self.node_ref.get() {
                        crate::show_modal_dialog(dialog_node);
                        self.open = true;
                    }
                }
                true
            }
            Msg::Close => {
                if self.open {
                    if let Some(on_close) = &props.on_close {
                        if let Some(dialog_node) = self.node_ref.get() {
                            crate::close_dialog(dialog_node);
                        }

                        on_close.emit(());
                        self.open = false;

                        self.restore_focus();
                    }
                }
                false
            }
            Msg::Controller => {
                self.handle_controller_messages(ctx);
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
            Msg::Drag(event) => {
                if event.phase == GesturePhase::End {
                    let mut dismiss = false;
                    let threshold = 100.0;
                    if let Some((delta_x, delta_y)) = self.drag_delta {
                        dismiss = match props.location {
                            SideDialogLocation::Left => delta_x < -threshold,
                            SideDialogLocation::Right => delta_x > threshold,
                            SideDialogLocation::Top => delta_y < -threshold,
                            SideDialogLocation::Bottom => delta_y > threshold,
                        };
                    }
                    self.drag_start = None;
                    self.drag_delta = None;

                    if dismiss {
                        ctx.link().send_message(Msg::Dismiss);
                    }
                    return true;
                }

                if scrolling_element_in_range(
                    event.target(),
                    self.slider_ref.clone(),
                    props.location,
                ) {
                    // don't do anything, children is scrolling
                    return false;
                }

                let x = event.x() as f64;
                let y = event.y() as f64;

                match self.drag_start {
                    None => {
                        if x > 0.0 && y > 0.0 {
                            // prevent divide by zero
                            self.drag_start = Some((x, y));
                            self.drag_delta = Some((0.0, 0.0));
                        }
                        false
                    }
                    Some(start) => {
                        self.drag_delta = Some((x - start.0, y - start.1));
                        true
                    }
                }
            }
            Msg::Swipe(event) => {
                if scrolling_element_in_range(
                    event.target(),
                    self.slider_ref.clone(),
                    props.location,
                ) {
                    // don't do anything, children is scrolling
                    return false;
                }
                let angle = event.direction; // -180 to + 180
                let dismiss = match props.location {
                    SideDialogLocation::Left => !(-135.0..=135.0).contains(&angle),
                    SideDialogLocation::Right => angle > -45.0 && angle < 45.0,
                    SideDialogLocation::Top => angle > 45.0 && angle < 135.0,
                    SideDialogLocation::Bottom => angle > -135.0 && angle < -45.0,
                };
                if dismiss {
                    ctx.link().send_message(Msg::Dismiss);
                }
                true
            }
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // always close the dialog before restoring the focus
        if let Some(dialog_node) = self.node_ref.get() {
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

        let slider_direction_class = match props.location {
            SideDialogLocation::Left => "pwt-side-dialog-left",
            SideDialogLocation::Right => "pwt-side-dialog-right",
            SideDialogLocation::Top => "pwt-side-dialog-top",
            SideDialogLocation::Bottom => "pwt-side-dialog-bottom",
        };

        let mut transform = None;
        let mut transition = None;
        if let Some((delta_x, delta_y)) = self.drag_delta {
            transition = Some("none");
            match props.location {
                SideDialogLocation::Left => {
                    let delta_x = delta_x.min(0.0);
                    transform = Some(format!("translateX({delta_x}px)"));
                }
                SideDialogLocation::Right => {
                    let delta_x = delta_x.max(0.0);
                    transform = Some(format!("translateX({delta_x}px)",));
                }
                SideDialogLocation::Top => {
                    let delta_y = delta_y.min(0.0);
                    transform = Some(format!("translateY({delta_y}px)",));
                }
                SideDialogLocation::Bottom => {
                    let delta_y = delta_y.max(0.0);
                    transform = Some(format!("translateY({delta_y}px)",));
                }
            }
        }

        let dialog = Container::from_tag("dialog")
            .class("pwt-side-dialog")
            .class(slider_state_class)
            .oncancel(oncancel)
            .onclose(link.callback(|_| Msg::Close))
            .with_child(
                Container::new()
                    .styles(props.styles.clone())
                    .class("pwt-side-dialog-slider")
                    .class(slider_direction_class)
                    .class(slider_state_class)
                    .style("transition", transition)
                    .style("transform", transform)
                    .ontransitionend(ctx.link().callback(|_| Msg::SliderAnimationEnd))
                    .children(props.children.clone())
                    .into_html_with_ref(self.slider_ref.clone()),
            );

        let view = GestureDetector::new(dialog.into_html_with_ref(self.node_ref.clone()))
            .on_tap({
                let slider_ref = self.slider_ref.clone();
                let link = ctx.link().clone();
                move |event: InputEvent| {
                    if let Some(element) = slider_ref.clone().into_html_element() {
                        if let Some(target) = event.target() {
                            if let Ok(target) = target.dyn_into::<web_sys::Node>() {
                                if element.contains(Some(&target)) {
                                    // click target inside dialog
                                    // Note: This can be outside of element.get_bounding_client_rect(), i.e.
                                    // if we open dropdowns/menus inside a dialog.
                                    return;
                                }
                            }
                        }

                        let rect = element.get_bounding_client_rect();
                        let x = event.x() as f64;
                        let y = event.y() as f64;

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
            .on_drag(ctx.link().callback(Msg::Drag))
            .on_swipe(ctx.link().callback(Msg::Swipe));

        let controller_context = html! {
            <ContextProvider<SideDialogController> context={self.controller.clone()}>
            {view}
            </ContextProvider<SideDialogController>>
        };

        // avoid problems with nested form
        create_portal(controller_context, gloo_utils::body().into())
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::Open);
        }
    }
}

impl From<SideDialog> for VNode {
    fn from(val: SideDialog) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtSideDialog>(Rc::new(val), key);
        VNode::from(comp)
    }
}

/// Checks if any element from `target` up to, but excluding, `boundary` can scroll in the
/// direction we would close the side dialog.
///
/// Returns `false` if `target` is not a descendant of `boundary`, so a gesture starting
/// outside the dialog content (such as on the backdrop) is not suppressed by the scroll
/// state of unrelated ancestors.
fn scrolling_element_in_range(
    target: Option<EventTarget>,
    boundary: NodeRef,
    location: SideDialogLocation,
) -> bool {
    let Some(element) = target.and_then(|t| t.dyn_into::<Element>().ok()) else {
        return false;
    };

    let Some(boundary) = boundary.cast::<Element>() else {
        return false;
    };

    if !boundary.contains(element.dyn_ref::<web_sys::Node>()) {
        return false;
    }

    let mut element = Some(element);

    while let Some(el) = element {
        if el == boundary {
            break;
        }
        if let Some(html) = el.dyn_ref::<HtmlElement>()
            && check_scrolling(html, location)
        {
            return true;
        }
        element = el.parent_element();
    }

    false
}

/// Returns true if the element is in a state where it can scroll relative to the direction we
/// would like to close the side dialog, e.g. for SideDialogLocation::Bottom it means returning
/// true if the element can scroll up, etc.
fn check_scrolling(el: &HtmlElement, location: SideDialogLocation) -> bool {
    match location {
        SideDialogLocation::Bottom => {
            if el.scroll_top() > 0 {
                return true;
            }
        }
        SideDialogLocation::Top => {
            if el.scroll_top() < el.scroll_height() - el.client_height() {
                return true;
            }
        }
        SideDialogLocation::Right => {
            if el.scroll_left() > 0 {
                return true;
            }
        }
        SideDialogLocation::Left => {
            if el.scroll_left() < el.scroll_width() - el.client_width() {
                return true;
            }
        }
    }
    false
}
