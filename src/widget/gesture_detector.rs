use std::collections::HashMap;
use std::rc::Rc;

use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::Container;

#[derive(Properties, Clone, PartialEq)]
pub struct GestureDetector {
    pub key: Option<Key>,

    #[prop_or(3000)]
    pub tap_max_delay: u32,
    #[prop_or(10.0)]
    pub tap_tolerance: f64,

    /// Callback for tap events.
    pub on_tap: Option<Callback<PointerEvent>>,
    /// Callback for long-tap events.
    pub on_long_press: Option<Callback<()>>,
}

impl GestureDetector {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the on_tap callback
    pub fn on_tap(mut self, cb: impl IntoEventCallback<PointerEvent>) -> Self {
        self.on_tap = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_long_press callback
    pub fn on_long_press(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_long_press = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    PointerDown(PointerEvent),
    PointerUp(PointerEvent),
    PointerMove(PointerEvent),
    PointerCancel(PointerEvent),
    PointerLeave(PointerEvent),

    Timeout1(i32),
    TapTimeout(i32),
}

#[derive(Copy, Clone, PartialEq)]
enum DetectionState {
    Initial,
    Single,
    Double,
//    Error,
    Done,
}
struct PointerState {
    _tap_timeout: Timeout,
    got_tap_timeout: bool,
    _timeout1: Timeout,
    got_timeout1: bool,
    start_ctime: f64,
    start_x: i32,
    start_y: i32,
    ctime: f64,
    x: i32,
    y: i32,
    speed: f64,
    direction: f64,
}

#[doc(hidden)]
pub struct PwtGestureDetector {
    node_ref: NodeRef,
    state: DetectionState,
    pointers: HashMap<i32, PointerState>,
}

fn now() -> f64 {
    js_sys::Date::now() / 1000.0
}

impl PwtGestureDetector {
    fn register_pointer(&mut self, ctx: &Context<Self>, id: i32, start_x: i32, start_y: i32) {
        let props = ctx.props();

        let link = ctx.link().clone();
        let _timeout1 = Timeout::new(2000, move || link.send_message(Msg::Timeout1(id)));

        let link = ctx.link().clone();
        let _tap_timeout = Timeout::new(props.tap_max_delay, move || {
            link.send_message(Msg::TapTimeout(id))
        });

        let ctime = now();

        self.pointers.insert(
            id,
            PointerState {
                _tap_timeout,
                got_tap_timeout: false,
                _timeout1,
                got_timeout1: false,
                start_x,
                start_y,
                start_ctime: ctime,
                ctime,
                x: start_x,
                y: start_y,
                speed: 0f64,
                direction: 0f64,
            },
        );
    }

    fn unregister_pointer(&mut self, id: i32) -> Option<PointerState> {
        self.pointers.remove(&id)
    }

    fn update_pointer_position(&mut self, id: i32, x: i32, y: i32) -> Option<&PointerState> {
        if let Some(pointer_state) = self.pointers.get_mut(&id) {
            let ctime = now();
            let time_diff = ctime - pointer_state.ctime;
            if time_diff <= 0.0 {
                return None; /* do nothing */
            }

            let distance = compute_distance(pointer_state.x, pointer_state.y, x, y);
            let direction = compute_direction(pointer_state.x, pointer_state.y, x, y);

            pointer_state.ctime = ctime;
            pointer_state.x = x;
            pointer_state.y = y;

            pointer_state.speed = distance / time_diff;
            pointer_state.direction = direction;

            Some(pointer_state)
        } else {
            None
        }
    }

    fn update_initial(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::TapTimeout(_id) => { /* ignore */ }
            Msg::Timeout1(_id) => { /* ignore */ }
            Msg::PointerDown(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 0);
                self.register_pointer(ctx, event.pointer_id(), event.x(), event.y());
                self.state = DetectionState::Single;
                if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
                    let _ = el.set_pointer_capture(event.pointer_id());
                }
            }
            Msg::PointerUp(_event) => { /* ignore */ }
            Msg::PointerMove(_event) => { /* ignore */ }
            Msg::PointerCancel(_event) => { /* ignore */ }
            Msg::PointerLeave(_event) => { /* ignore */ }
        }
        true
    }

    fn update_single(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let props = ctx.props();
        match msg {
            Msg::TapTimeout(id) => {
                if let Some(pointer_state) = self.pointers.get_mut(&id) {
                    pointer_state.got_tap_timeout = true;
                }
            }
            Msg::Timeout1(id) => {
                if let Some(pointer_state) = self.pointers.get_mut(&id) {
                    pointer_state.got_timeout1 = true;
                    let distance = compute_distance(
                        pointer_state.start_x,
                        pointer_state.start_y,
                        pointer_state.x,
                        pointer_state.y,
                    );
                    if distance < props.tap_tolerance {
                        log::info!("LONG PRESS");
                        self.state = DetectionState::Done;
                        if let Some(on_long_press) = &props.on_long_press {
                            on_long_press.emit(());
                        }
                    }
                }
            }
            Msg::PointerDown(event) => {
                event.prevent_default();
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.register_pointer(ctx, event.pointer_id(), event.x(), event.y());
                self.state = DetectionState::Double;
            }
            Msg::PointerUp(event) => {
                event.prevent_default();
                log::info!("POINTERUP");
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                if let Some(pointer_state) = self.unregister_pointer(event.pointer_id()) {
                    self.state = DetectionState::Initial;
                    let distance = compute_distance(
                        pointer_state.start_x,
                        pointer_state.start_y,
                        event.x(),
                        event.y(),
                    );
                    if !pointer_state.got_tap_timeout && distance < props.tap_tolerance {
                        if let Some(on_tap) = &props.on_tap {
                            log::info!("tap {} {}", event.x(), event.y());
                            on_tap.emit(event);
                        }
                    } else {
                        let time_diff = now() - pointer_state.start_ctime;
                        let speed = distance / time_diff;
                        log::info!("DRAG END {time_diff} {speed}");
                    }
                }
            }
            Msg::PointerMove(event) => {
                event.prevent_default();
                log::info!("MOVE");
                if let Some(pointer_state) =
                    self.update_pointer_position(event.pointer_id(), event.x(), event.y())
                {
                    let distance = compute_distance(
                        pointer_state.start_x,
                        pointer_state.start_y,
                        event.x(),
                        event.y(),
                    );
                    if distance >= props.tap_tolerance || pointer_state.got_tap_timeout {
                        log::info!("DRAG TO {} {}", event.x(), event.y());
                    }
                }
            }
            Msg::PointerCancel(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                if let Some(_pointer_state) = self.unregister_pointer(event.pointer_id()) {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::PointerLeave(event) => {
                log::info!("LEAVE0");
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                if let Some(pointer_state) = self.unregister_pointer(event.pointer_id()) {
                    self.state = DetectionState::Initial;
                    //if pointer_state.speed
                    log::info!("LEAVE {}", pointer_state.speed);
                }
            }
        }
        true
    }

    fn update_error(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::TapTimeout(_id) => { /* ignore */ }
            Msg::Timeout1(_id) => { /* ignore */ }
            Msg::PointerDown(event) => {
                self.register_pointer(ctx, event.pointer_id(), event.x(), event.y());
            }
            Msg::PointerUp(event) => {
                self.unregister_pointer(event.pointer_id());
                if self.pointers.is_empty() {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::PointerMove(_event) => { /* ignore */ }
            Msg::PointerCancel(event) => {
                self.unregister_pointer(event.pointer_id());
                if self.pointers.is_empty() {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::PointerLeave(event) => {
                self.unregister_pointer(event.pointer_id());
                if self.pointers.is_empty() {
                    self.state = DetectionState::Initial;
                }
            }
        }
        true
    }
}

impl Component for PwtGestureDetector {
    type Message = Msg;
    type Properties = GestureDetector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: DetectionState::Initial,
            pointers: HashMap::new(),
            node_ref: NodeRef::default(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        //let props = ctx.props();

        match self.state {
            DetectionState::Initial => self.update_initial(ctx, msg),
            DetectionState::Single => self.update_single(ctx, msg),
            DetectionState::Double => todo!(),
            //DetectionState::Error => self.update_error(ctx, msg),
            DetectionState::Done => self.update_error(ctx, msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //let props = ctx.props();

        Container::new()
            .node_ref(self.node_ref.clone())
            .class("pwt-fit")
            .attribute("style", "touch-action:none;")
            .onpointerdown(ctx.link().callback(Msg::PointerDown))
            .onpointerup(ctx.link().callback(Msg::PointerUp))
            .onpointermove(ctx.link().callback(Msg::PointerMove))
            .onpointercancel(ctx.link().callback(Msg::PointerCancel))
            //.onpointerleave(ctx.link().callback(Msg::PointerLeave))
            //.onmouseleave(|_| {
            //    log::info!("MOUSE LEAVE");
            //})
            //.ontouchend(|_| {
            //    log::info!("TOUCH END");
            //})
            // .ontouchmove(|_| {
            //     log::info!("TOUCH MOVE");
            // })
            .with_child("TEST")
            .into()
    }
}

impl Into<VNode> for GestureDetector {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtGestureDetector>(Rc::new(self), key);
        VNode::from(comp)
    }
}

// -180...180
fn compute_direction(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    let dx = (x2 - x1) as f64;
    let dy = (y1 - y2) as f64;
    (dy.atan2(dx) * 360.0) / (2.0 * std::f64::consts::PI)
}

fn compute_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    let dx = (x2 - x1) as f64;
    let dy = (y2 - y1) as f64;

    let radius = (dx * dx + dy * dy).sqrt();
    radius
}
