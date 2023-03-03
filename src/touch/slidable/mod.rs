mod slidable_controller;
pub use slidable_controller::{SlidableController, SlidableControllerMsg};

mod slidable_action_event;
pub use slidable_action_event::SlidableActionMouseEvent;

mod slidable_action;
pub use slidable_action::{PwtSlidableAction, SlidableAction};

use gloo_timers::callback::Timeout;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::VNode;

use crate::prelude::*;
use crate::widget::{Container, Row, SizeObserver};
use crate::touch::{GestureDetector, GestureDragEvent, GestureSwipeEvent};

use pwt_macros::widget;

/// Slidable widget with directional slide actions that can be dismissed.
///
/// The Slidable provides a [SlidableController] using a [yew::ContextProvider]. The
/// controller can be used to programmatically collapse or dismiss the slidable.
/// The [SlidableAction] button automaticall uses that controller.
#[widget(pwt=crate, comp=PwtSlidable, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Slidable {
    content: VNode,

    /// Widget displayed on the left side (below the slider).
    pub left_actions: Option<VNode>,

    /// Widget displayed on the right side (below the slider).
    pub right_actions: Option<VNode>,

    /// Dismiss callback.
    ///
    /// Without a callback, dismiss is disabled on slidables without actions.
    pub on_dismiss: Option<Callback<()>>,
}

impl Slidable {
    /// Create a new instance.
    pub fn new(content: impl Into<VNode>) -> Self {
        yew::props!(Self {
            content: content.into()
        })
    }

    /// Builder style method to set the left actions pane.
    pub fn left_actions<N: Into<VNode>>(mut self, actions: impl IntoPropValue<Option<N>>) -> Self {
        self.set_left_actions(actions);
        self
    }

    /// Method to set the left actions pane.
    pub fn set_left_actions<N: Into<VNode>>(&mut self, actions: impl IntoPropValue<Option<N>>) {
        self.left_actions = actions.into_prop_value().map(|p| p.into());
    }

    /// Builder style method to set the right actions pane.
    pub fn right_actions<N: Into<VNode>>(mut self, actions: impl IntoPropValue<Option<N>>) -> Self {
        self.set_right_actions(actions);
        self
    }

    /// Method to set the right actions pane.
    pub fn set_right_actions<N: Into<VNode>>(&mut self, actions: impl IntoPropValue<Option<N>>) {
        self.right_actions = actions.into_prop_value().map(|p| p.into());
    }

    /// Builder style method to set the dismiss callback.
    pub fn on_dismiss(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_dismiss = cb.into_event_callback();
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ViewState {
    Normal,
    DismissStart,
    DismissTransition,
    Dismissed,
}

#[doc(hidden)]
pub struct PwtSlidable {
    start_pos: f64,
    drag_start: i32,
    drag_pos: Option<i32>,
    left_size: f64,
    left_ref: NodeRef,
    left_action_ref: NodeRef,
    left_observer: Option<SizeObserver>,
    right_size: f64,
    right_ref: NodeRef,
    right_action_ref: NodeRef,
    right_observer: Option<SizeObserver>,
    content_width: f64,
    content_height: f64,
    content_ref: NodeRef,
    content_observer: Option<SizeObserver>,
    last_action_left: bool,
    switch_back: bool,
    view_state: ViewState,
    controller: SlidableController,
    dismiss_start_timeout: Option<Timeout>,
}

pub enum Msg {
    StartDismissTransition,
    Drag(GestureDragEvent),
    DragStart(GestureDragEvent),
    DragEnd(GestureDragEvent),
    Swipe(GestureSwipeEvent),
    LeftResize(f64),
    RightResize(f64),
    ContentResize(f64, f64),
    TransitionEnd,
    Controller(SlidableControllerMsg),
}

impl PwtSlidable {
    fn start_dismiss(&mut self) {
        self.view_state = match self.view_state {
            ViewState::Normal => ViewState::DismissStart,
            state => state,
        }
    }

    fn finalize_drag(&mut self) {
        if self.start_pos > 0f64 {
            if self.left_size > 0f64 && self.start_pos >= self.left_size {
                self.start_pos = self.left_size;
            } else {
                self.start_pos = 0f64;
                self.switch_back = true;
            }
        } else if self.start_pos < 0f64 {
            if self.right_size > 0f64 && (self.start_pos) <= -self.right_size {
                self.start_pos = -self.right_size;
            } else {
                self.start_pos = 0f64;
                self.switch_back = true;
            }
        }
    }

    fn left_container(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let actions = props.left_actions.clone();

        Row::new()
            .node_ref(self.left_ref.clone())
            .class("pwt-w-100 pwt-h-100")
            .with_child(
                Container::new()
                    .node_ref(self.left_action_ref.clone())
                    .attribute("style", "height:100%;min-width:0;flex:0 1 auto")
                    .with_optional_child(actions),
            )
            .with_child(html! {<div style="flex: 1 1 auto;"></div>})
            .into()
    }

    fn right_container(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let actions = props.right_actions.clone();

        Row::new()
            .node_ref(self.right_ref.clone())
            .class("pwt-w-100 pwt-h-100")
            .with_child(html! {<div style="flex: 1 1 auto;"></div>})
            .with_child(
                Container::new()
                    .node_ref(self.right_action_ref.clone())
                    .attribute("style", "height:100%;min-width:0;flex:0 1 auto")
                    .with_optional_child(actions),
            )
            .into()
    }
}

impl Component for PwtSlidable {
    type Message = Msg;
    type Properties = Slidable;

    fn create(ctx: &Context<Self>) -> Self {
        let controller = SlidableController::new(ctx.link().callback(Msg::Controller));

        Self {
            start_pos: 0f64,
            drag_start: 0,
            drag_pos: None,
            left_size: 0f64,
            left_ref: NodeRef::default(),
            left_action_ref: NodeRef::default(),
            left_observer: None,
            right_size: 0f64,
            right_ref: NodeRef::default(),
            right_action_ref: NodeRef::default(),
            right_observer: None,
            content_width: 0f64,
            content_height: 0f64,
            content_ref: NodeRef::default(),
            content_observer: None,
            last_action_left: true,
            switch_back: false,
            view_state: ViewState::Normal,
            controller,
            dismiss_start_timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::StartDismissTransition => {
                self.view_state = ViewState::DismissTransition;
            }
            Msg::Drag(event) => {
                self.drag_pos = Some(self.drag_start - event.x());
            }
            Msg::DragStart(event) => {
                self.drag_start = event.x();
            }
            Msg::DragEnd(_event) => {
                self.drag_start = 0;
                self.start_pos -= self.drag_pos.take().unwrap_or(0) as f64;
                self.finalize_drag();
            }
            Msg::ContentResize(width, height) => {
                if self.start_pos == 0f64 && self.drag_pos.is_none() && !self.switch_back {
                    self.content_width = width.max(0f64);
                    self.content_height = height.max(0f64);
                    //log::info!("CONTENT RESIZE {width} {height}")
                }
            }
            Msg::LeftResize(width) => {
                let left_size = self
                    .left_action_ref
                    .cast::<web_sys::HtmlElement>()
                    .unwrap()
                    .offset_width()
                    .max(0) as f64;

                if width > (left_size + 1.0) {
                    self.left_size = left_size;
                }

                if self.drag_pos.is_none() {
                    // RESIZE after DRAG
                    self.finalize_drag();
                }
            }
            Msg::RightResize(width) => {
                let right_size = self
                    .right_action_ref
                    .cast::<web_sys::HtmlElement>()
                    .unwrap()
                    .offset_width()
                    .max(0) as f64;

                if width > (right_size + 1.0) {
                    self.right_size = right_size;
                }

                if self.drag_pos.is_none() {
                    // RESIZE after DRAG
                    self.finalize_drag();
                }
            }
            Msg::Swipe(event) => {
                let direction = event.direction.abs();
                if direction < 45.0 || direction > 135.0 {
                    if props.left_actions.is_none() && props.right_actions.is_none() {
                        if props.on_dismiss.is_some() {
                            // log::info!("START DISMISS");
                            self.start_dismiss()
                        }
                    }
                }
            }
            Msg::TransitionEnd => {
                self.switch_back = false;
                if props.left_actions.is_none() && props.right_actions.is_none() {
                    if self.view_state == ViewState::DismissTransition {
                        //log::info!("DISMISS");
                        self.view_state = ViewState::Dismissed;
                        if let Some(on_dismiss) = &props.on_dismiss {
                            on_dismiss.emit(());
                        }
                    }
                }
            }
            Msg::Controller(msg) => match msg {
                SlidableControllerMsg::Dismiss => {
                    log::info!("REquest Dismiss");
                    self.start_dismiss();
                }
                SlidableControllerMsg::Collapse => {
                    log::info!("REquest Collapse");
                    if self.drag_pos.is_none() {
                        if self.start_pos != 0f64 {
                            self.start_pos = 0f64;
                            self.switch_back = true;
                        }
                    }
                }
            },
        }
        let pos = self.start_pos - (self.drag_pos.unwrap_or(0) as f64);
        if pos > 0f64 {
            self.last_action_left = true;
        } else if pos < 0f64 {
            self.last_action_left = false;
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let pos = self.start_pos - (self.drag_pos.unwrap_or(0) as f64);
        let (left, right) = if pos >= 0f64 {
            (pos, 0f64)
        } else {
            (0f64, -pos)
        };

        // no animation during drag
        let transition = if self.drag_pos.is_none() {
            "transition: width 0.1s ease-out;"
        } else {
            ""
        };

        let upper = GestureDetector::new(
            Container::new()
                .node_ref(self.content_ref.clone())
                .class("pwt-slidable-slider")
                .attribute(
                    "style",
                    "touch-action:none;width:100%;flex:0 0 auto;overflow:hidden;",
                )
                .with_child(props.content.clone()),
        )
        .on_drag_start(ctx.link().callback(Msg::DragStart))
        .on_drag_end(ctx.link().callback(Msg::DragEnd))
        .on_drag_update(ctx.link().callback(Msg::Drag))
        .on_swipe(ctx.link().callback(Msg::Swipe));

        let left_container = Container::new()
            .attribute(
                "style",
                format!(
                    "width:{left}px;flex: 0 0 auto;{};overflow:hidden;",
                    transition
                ),
            )
            .with_child(self.left_container(ctx));

        let right_container = Container::new()
            .attribute(
                "style",
                format!(
                    "width:{right}px;flex: 0 0 auto;{};overflow:hidden;",
                    transition
                ),
            )
            .with_child(self.right_container(ctx));

        let row = Row::new()
            .attribute(
                "style",
                format!(
                    "width:{};overflow:hidden;justify-content:{};transition:height 0.2s ease-out;{}",
                    if self.start_pos == 0f64 && self.drag_pos.is_none() && !self.switch_back {
                        String::from("100%")
                    } else {
                        format!("{}px", self.content_width)
                    },
                    if self.last_action_left {
                        "left"
                    } else {
                        "right"
                    },
                    match self.view_state {
                        ViewState::Normal => String::new(),
                        ViewState::DismissStart => format!("height:{}px;", self.content_height),
                        ViewState::DismissTransition | ViewState::Dismissed => String::from("height:0px;"),
                    }
                ),
            )
            .with_child(left_container)
            .with_child(upper)
            .with_child(right_container)
            .ontransitionend(ctx.link().callback(|_| Msg::TransitionEnd));

        yew::props!(Container {
            std_props: props.std_props.clone(),
            listeners: props.listeners.clone(),
        })
        .class("pwt-slidable")
        .with_child(html! {
            <ContextProvider<SlidableController> context={self.controller.clone()}>
                {row}
            </ContextProvider<SlidableController>>
        })
        .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.content_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                self.content_observer = Some(SizeObserver::new(&el, move |(x, y)| {
                    link.send_message(Msg::ContentResize(x, y));
                }));
            }
            if let Some(el) = self.left_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                self.left_observer = Some(SizeObserver::new(&el, move |(x, _y)| {
                    link.send_message(Msg::LeftResize(x));
                }));
            }
            if let Some(el) = self.right_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                self.right_observer = Some(SizeObserver::new(&el, move |(x, _y)| {
                    link.send_message(Msg::RightResize(x));
                }));
            }
        }
        if self.view_state == ViewState::DismissStart {
            // We use a timeout to make sure the browser gets the correct height before we
            // start animating the height.
            self.dismiss_start_timeout = Some(Timeout::new(1, {
                let link = ctx.link().clone();
                move || link.send_message(Msg::StartDismissTransition)
            }));
        }
    }
}
