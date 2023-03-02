use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::VNode;

use crate::prelude::*;
use crate::widget::{
    Container, GestureDetector, GestureDragEvent, GestureSwipeEvent, Row, SizeObserver,
};

use pwt_macros::widget;

#[widget(pwt=crate, comp=PwtSlidable, @element)]
#[derive(Properties, Clone, PartialEq)]
pub struct Slidable {
    pub content: VNode,
    pub left_actions: Option<VNode>,
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
    pub fn left_actions(mut self, actions: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_left_actions(actions);
        self
    }

    /// Method to set the left actions pane.
    pub fn set_left_actions(&mut self, actions: impl IntoPropValue<Option<VNode>>) {
        self.left_actions = actions.into_prop_value();
    }

    /// Builder style method to set the right actions pane.
    pub fn right_actions(mut self, actions: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_right_actions(actions);
        self
    }

    /// Method to set the right actions pane.
    pub fn set_right_actions(&mut self, actions: impl IntoPropValue<Option<VNode>>) {
        self.right_actions = actions.into_prop_value();
    }

    /// Builder style method to set the dismiss callback.
    pub fn on_dismiss(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_dismiss = cb.into_event_callback();
        self
    }
}

#[doc(hidden)]
pub struct PwtSlidable {
    start_pos: f64,
    drag_start: i32,
    drag_pos: Option<i32>,
    left_size: f64,
    left_ref: NodeRef,
    left_observer: Option<SizeObserver>,
    right_size: f64,
    right_ref: NodeRef,
    right_observer: Option<SizeObserver>,
    content_size: f64,
    content_ref: NodeRef,
    content_observer: Option<SizeObserver>,
    last_action_left: bool,
    switch_back: bool,
    dismiss: bool,
}

pub enum Msg {
    Drag(GestureDragEvent),
    DragStart(GestureDragEvent),
    DragEnd(GestureDragEvent),
    Swipe(GestureSwipeEvent),
    LeftResize(f64),
    RightResize(f64),
    ContentResize(f64),
    TransitionEnd,
}

impl PwtSlidable {
    fn finalize_drag(&mut self) {
        if self.start_pos > 0f64 {
            if self.left_size > 0f64 && self.start_pos > self.left_size {
                self.start_pos = self.left_size;
            } else {
                self.start_pos = 0f64;
                self.switch_back = true;
            }
        } else if self.start_pos < 0f64 {
            if self.right_size > 0f64 && (self.start_pos) < -self.right_size {
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
            .class("pwt-w-100 pwt-h-100")
            .with_child(
                Container::new()
                    .node_ref(self.left_ref.clone())
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
            .class("pwt-w-100 pwt-h-100")
            .with_child(html! {<div style="flex: 1 1 auto;"></div>})
            .with_child(
                Container::new()
                    .node_ref(self.right_ref.clone())
                    .attribute("style", "height:100%;min-width:0;flex:0 1 auto")
                    .with_optional_child(actions),
            )
            .into()
    }
}

impl Component for PwtSlidable {
    type Message = Msg;
    type Properties = Slidable;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            start_pos: 0f64,
            drag_start: 0,
            drag_pos: None,
            left_size: 0f64,
            left_ref: NodeRef::default(),
            left_observer: None,
            right_size: 0f64,
            right_ref: NodeRef::default(),
            right_observer: None,
            content_size: 0f64,
            content_ref: NodeRef::default(),
            content_observer: None,
            last_action_left: true,
            switch_back: false,
            dismiss: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
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
            Msg::ContentResize(width) => {
                if self.start_pos == 0f64 && self.drag_pos.is_none() && !self.switch_back {
                    self.content_size = width.max(0f64);
                    log::info!("CONTENT RESIZE {width}")
                }
            }
            Msg::LeftResize(width) => {
                self.left_size = width.max(0f64);
                if self.drag_pos.is_none() {
                    // RESIZE after DRAG
                    self.finalize_drag();
                }
            }
            Msg::RightResize(width) => {
                self.right_size = width.max(0f64);
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
                            self.dismiss = true;
                        }
                    }
                }
            }
            Msg::TransitionEnd => {
                self.switch_back = false;
                if props.left_actions.is_none() && props.right_actions.is_none() {
                    if self.dismiss {
                        //log::info!("DISMISS");
                        if let Some(on_dismiss) = &props.on_dismiss {
                            on_dismiss.emit(());
                        }
                    }
                }
            }
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
                        format!("{}px", self.content_size)
                    },
                    if self.last_action_left {
                        "left"
                    } else {
                        "right"
                    },
                    if self.dismiss {
                        "height:0px;"
                    } else {
                        "height:50px;"
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
        .with_child(row)
        .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.content_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                self.content_observer = Some(SizeObserver::new(&el, move |(x, _y)| {
                    link.send_message(Msg::ContentResize(x));
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
    }
}
