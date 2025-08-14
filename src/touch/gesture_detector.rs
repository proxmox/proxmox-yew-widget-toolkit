use core::ops::Deref;
use std::collections::HashMap;
use std::rc::Rc;

use gloo_timers::callback::Timeout;
use gloo_utils::window;
use wasm_bindgen::JsValue;
use web_sys::{EventTarget, Touch};
use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::impl_to_html;
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder, WidgetStyleBuilder};
use crate::widget::Container;

/// An event that can happen from a [`PointerEvent`] or a [`Touch`]
///
/// For convenience, expose the most important values from the underlying events
pub enum InputEvent {
    PointerEvent(PointerEvent),
    Touch(Touch),
}

impl InputEvent {
    pub fn x(&self) -> i32 {
        match self {
            InputEvent::PointerEvent(pointer_event) => pointer_event.client_x(),
            InputEvent::Touch(touch) => touch.client_x(),
        }
    }

    pub fn y(&self) -> i32 {
        match self {
            InputEvent::PointerEvent(pointer_event) => pointer_event.client_y(),
            InputEvent::Touch(touch) => touch.client_y(),
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            InputEvent::PointerEvent(pointer_event) => pointer_event.pointer_id(),
            InputEvent::Touch(touch) => touch.identifier(),
        }
    }

    pub fn target(&self) -> Option<EventTarget> {
        match self {
            InputEvent::PointerEvent(pointer_event) => pointer_event.target(),
            InputEvent::Touch(touch) => touch.target(),
        }
    }
}

impl From<PointerEvent> for InputEvent {
    fn from(event: PointerEvent) -> Self {
        Self::PointerEvent(event)
    }
}

impl From<Touch> for InputEvent {
    fn from(touch: Touch) -> Self {
        Self::Touch(touch)
    }
}

/// Like [PointerEvent](web_sys::PointerEvent), but includes the swipe direction
pub struct GestureSwipeEvent {
    event: InputEvent,
    /// Direction angle (from -180 to +180 degree)
    pub direction: f64,
}

impl GestureSwipeEvent {
    fn new(event: InputEvent, direction: f64) -> Self {
        Self { event, direction }
    }
}

impl Deref for GestureSwipeEvent {
    type Target = InputEvent;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

/// Gesture detector.
///
/// You need to set the CSS attribute `touch-action: none;` on children to receive all events.
///
/// Detected gestures:
///
/// - tap: single tap.
/// - long press: long tab without drag.
/// - drag: pointer move while touching the surface.
/// - swipe: fired at the end of a fast drag.
///
/// # Note
///
/// We use "display: contents;", so events reports wrong relative coordiantes (offsetX and offsetY).
///
/// Nested gesture detection is currently not implemented.
///
/// Scale and rotate detection is also not implemented.
#[derive(Properties, Clone, PartialEq)]
pub struct GestureDetector {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    content: Html,

    /// The maximum delay in miliseconds between a tap start and end event.
    #[prop_or(3000)]
    pub tap_max_delay: u32,
    /// The maximum tolerated movement in pixel unless tap detection fail.
    #[prop_or(10.0)]
    pub tap_tolerance: f64,

    /// Long press delay in millisecods.
    #[prop_or(1000)]
    pub long_press_delay: u32,

    /// Minimum swipe distance in pixel.
    #[prop_or(100.0)]
    pub swipe_min_distance: f64,
    /// Maximum swipe duration in milliseconds.
    #[prop_or(0.5)]
    pub swipe_max_duration: f64,
    /// Mimimum swipe speed in pixel/second.
    #[prop_or(200.0)]
    pub swipe_min_velocity: f64,

    /// Callback for tap events.
    #[prop_or_default]
    pub on_tap: Option<Callback<InputEvent>>,
    /// Callback for long-tap events.
    #[prop_or_default]
    pub on_long_press: Option<Callback<()>>,

    /// Callback for drag-start events.
    #[prop_or_default]
    pub on_drag_start: Option<Callback<InputEvent>>,
    /// Callback for drag-start events.
    #[prop_or_default]
    pub on_drag_update: Option<Callback<InputEvent>>,
    /// Callback for drag-start events.
    #[prop_or_default]
    pub on_drag_end: Option<Callback<InputEvent>>,

    #[prop_or_default]
    pub on_swipe: Option<Callback<GestureSwipeEvent>>,
}

impl GestureDetector {
    /// Creates a new instance.
    pub fn new(content: impl Into<Html>) -> Self {
        yew::props!(Self {
            content: content.into()
        })
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the on_tap callback
    pub fn on_tap(mut self, cb: impl IntoEventCallback<InputEvent>) -> Self {
        self.on_tap = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_long_press callback
    pub fn on_long_press(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_long_press = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_drag_start callback
    pub fn on_drag_start(mut self, cb: impl IntoEventCallback<InputEvent>) -> Self {
        self.on_drag_start = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_drag_update callback
    pub fn on_drag_update(mut self, cb: impl IntoEventCallback<InputEvent>) -> Self {
        self.on_drag_update = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_drag_end callback
    pub fn on_drag_end(mut self, cb: impl IntoEventCallback<InputEvent>) -> Self {
        self.on_drag_end = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_swipe callback
    pub fn on_swipe(mut self, cb: impl IntoEventCallback<GestureSwipeEvent>) -> Self {
        self.on_swipe = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    PointerDown(PointerEvent),
    PointerUp(PointerEvent),
    PointerMove(PointerEvent),
    PointerCancel(PointerEvent),
    PointerLeave(PointerEvent),

    LongPressTimeout(i32),
    TapTimeout(i32),

    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchCancel(TouchEvent),
    TouchEnd(TouchEvent),
}

#[derive(Copy, Clone, PartialEq)]
enum DetectionState {
    Initial,
    Single,
    Drag,
    Double,
    //    Error,
    Done,
}
struct PointerState {
    _tap_timeout: Timeout,
    got_tap_timeout: bool,
    _long_press_timeout: Timeout,
    got_long_press_timeout: bool,
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
    touch_only: bool,
    node_ref: NodeRef,
    state: DetectionState,
    pointers: HashMap<i32, PointerState>,
}

fn now() -> f64 {
    js_sys::Date::now() / 1000.0
}

impl PwtGestureDetector {
    fn register_pointer_state(&mut self, ctx: &Context<Self>, id: i32, start_x: i32, start_y: i32) {
        let props = ctx.props();

        let link = ctx.link().clone();
        let _long_press_timeout = Timeout::new(props.long_press_delay, move || {
            link.send_message(Msg::LongPressTimeout(id))
        });

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
                _long_press_timeout,
                got_long_press_timeout: false,
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

    fn register_pointer(&mut self, ctx: &Context<Self>, event: &PointerEvent) {
        let id = event.pointer_id();
        let start_x = event.x();
        let start_y = event.y();

        self.register_pointer_state(ctx, id, start_x, start_y);
    }

    fn register_touches(&mut self, ctx: &Context<Self>, event: &TouchEvent) {
        for_each_changed_touch(event, |touch: Touch| {
            let id = touch.identifier();
            let x = touch.client_x();
            let y = touch.client_y();
            self.register_pointer_state(ctx, id, x, y);
        });
    }

    fn unregister_touches<F: FnMut(i32, Touch, PointerState)>(
        &mut self,
        event: &TouchEvent,
        mut func: F,
    ) {
        for_each_changed_touch(event, |touch: Touch| {
            let id = touch.identifier();
            if let Some(state) = self.pointers.remove(&id) {
                func(id, touch, state);
            }
        });
    }

    fn unregister_pointer(&mut self, id: i32) -> Option<PointerState> {
        self.pointers.remove(&id)
    }

    fn capture_pointer(&self, pointer_id: i32) {
        if let Some(el) = self.node_ref.cast::<web_sys::Element>() {
            let _ = el.set_pointer_capture(pointer_id);
        }
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
            Msg::LongPressTimeout(_id) => { /* ignore */ }
            Msg::PointerDown(event) => {
                match event.pointer_type().as_str() {
                    "mouse" | "pen" => {
                        if event.button() != 0 {
                            return false;
                        }
                    }
                    "touch" => { /* Ok */ }
                    _ => return false, // unreachable
                }
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 0);
                self.register_pointer(ctx, &event);
                self.state = DetectionState::Single;
            }
            Msg::TouchStart(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 0);
                self.register_touches(ctx, &event);
                self.state = match self.pointers.len() {
                    0 => DetectionState::Initial,
                    1 => DetectionState::Single,
                    // TODO implement more touches
                    _ => DetectionState::Double,
                };
            }
            Msg::PointerUp(_event) => { /* ignore */ }
            Msg::PointerMove(_event) => { /* ignore */ }
            Msg::PointerCancel(_event) => { /* ignore */ }
            Msg::PointerLeave(_event) => { /* ignore */ }
            Msg::TouchMove(_event) => { /* ignore */ }
            Msg::TouchCancel(_event) => { /* ignore */ }
            Msg::TouchEnd(_event) => { /* ignore */ }
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
            Msg::LongPressTimeout(id) => {
                if let Some(pointer_state) = self.pointers.get_mut(&id) {
                    pointer_state.got_long_press_timeout = true;
                    let distance = compute_distance(
                        pointer_state.start_x,
                        pointer_state.start_y,
                        pointer_state.x,
                        pointer_state.y,
                    );
                    if distance < props.tap_tolerance {
                        //log::info!("LONG PRESS");
                        // supress further (click) events on children
                        self.capture_pointer(id);

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
                self.register_pointer(ctx, &event);
                self.state = DetectionState::Double;
            }
            Msg::TouchStart(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.register_touches(ctx, &event);
                self.state = match self.pointers.len() {
                    0 => DetectionState::Initial,
                    1 => DetectionState::Single,
                    // TODO implement more touches
                    _ => DetectionState::Double,
                };
            }
            Msg::PointerUp(event) => {
                event.prevent_default();
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
                            //log::info!("tap {} {}", event.x(), event.y());
                            on_tap.emit(event.into());
                        }
                    }
                }
            }
            Msg::TouchEnd(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.unregister_touches(&event, |_id, touch, pointer_state| {
                    let distance = compute_distance(
                        pointer_state.start_x,
                        pointer_state.start_y,
                        touch.client_x(),
                        touch.client_y(),
                    );
                    if !pointer_state.got_tap_timeout && distance < props.tap_tolerance {
                        if let Some(on_tap) = &props.on_tap {
                            //log::info!("tap {} {}", event.x(), event.y());
                            on_tap.emit(touch.into());
                        }
                    }
                });
                self.state = DetectionState::Initial;
            }
            Msg::PointerMove(event) => {
                event.prevent_default();
                if let Some(pointer_state) =
                    self.update_pointer_position(event.pointer_id(), event.x(), event.y())
                {
                    let distance = compute_distance(
                        pointer_state.start_x,
                        pointer_state.start_y,
                        event.x(),
                        event.y(),
                    );
                    // Make sure it cannot be a TAP or LONG PRESS event
                    if distance >= props.tap_tolerance {
                        //log::info!("DRAG START {} {}", event.x(), event.y());
                        self.state = DetectionState::Drag;
                        self.capture_pointer(event.pointer_id());
                        if let Some(on_drag_start) = &props.on_drag_start {
                            on_drag_start.emit(event.into());
                        }
                    }
                }
            }
            Msg::TouchMove(event) => {
                for_each_changed_touch(&event, |touch| {
                    if let Some(pointer_state) = self.update_pointer_position(
                        touch.identifier(),
                        touch.client_x(),
                        touch.client_y(),
                    ) {
                        let distance = compute_distance(
                            pointer_state.start_x,
                            pointer_state.start_y,
                            touch.client_x(),
                            touch.client_y(),
                        );
                        // Make sure it cannot be a TAP or LONG PRESS event
                        if distance >= props.tap_tolerance {
                            self.state = DetectionState::Drag;
                            if let Some(on_drag_start) = &props.on_drag_start {
                                on_drag_start.emit(touch.into());
                            }
                        }
                    }
                });
            }
            Msg::PointerCancel(event) | Msg::PointerLeave(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                if let Some(_pointer_state) = self.unregister_pointer(event.pointer_id()) {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::TouchCancel(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.unregister_touches(&event, |_, _, _| {});
                self.state = DetectionState::Initial;
            }
        }
        true
    }

    fn update_drag(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let props = ctx.props();
        match msg {
            Msg::TapTimeout(_id) => { /* ignore */ }
            Msg::LongPressTimeout(_id) => { /* ignore */ }
            Msg::PointerDown(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                // Abort current drag
                self.register_pointer(ctx, &event);
                self.state = DetectionState::Double;
                //log::info!("DRAG END");
                if let Some(on_drag_end) = &props.on_drag_end {
                    on_drag_end.emit(event.into());
                }
            }
            Msg::TouchStart(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                // Abort current drags
                self.register_touches(ctx, &event);
                self.state = DetectionState::Double;
                for_each_active_touch(&event, |touch| {
                    if self.pointers.contains_key(&touch.identifier()) {
                        if let Some(on_drag_end) = &props.on_drag_end {
                            on_drag_end.emit(touch.into());
                        }
                    }
                });
            }
            Msg::PointerUp(event) => {
                event.prevent_default();
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
                    let time_diff = now() - pointer_state.start_ctime;
                    let speed = distance / time_diff;
                    //log::info!("DRAG END {time_diff} {speed}");
                    if let Some(on_drag_end) = &props.on_drag_end {
                        on_drag_end.emit(event.clone().into());
                    }

                    if let Some(on_swipe) = &props.on_swipe {
                        if distance > props.swipe_min_distance
                            && time_diff < props.swipe_max_duration
                            && speed > props.swipe_min_velocity
                        {
                            let direction = compute_direction(
                                pointer_state.start_x,
                                pointer_state.start_y,
                                event.x(),
                                event.y(),
                            );

                            let event = GestureSwipeEvent::new(event.into(), direction);
                            on_swipe.emit(event)
                        }
                    }
                }
            }
            Msg::TouchEnd(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                for_each_changed_touch(&event, |touch| {
                    if let Some(pointer_state) = self.unregister_pointer(touch.identifier()) {
                        let distance = compute_distance(
                            pointer_state.start_x,
                            pointer_state.start_y,
                            touch.client_x(),
                            touch.client_y(),
                        );
                        let time_diff = now() - pointer_state.start_ctime;
                        let speed = distance / time_diff;
                        //log::info!("DRAG END {time_diff} {speed}");
                        if let Some(on_drag_end) = &props.on_drag_end {
                            on_drag_end.emit(touch.clone().into());
                        }

                        if let Some(on_swipe) = &props.on_swipe {
                            if distance > props.swipe_min_distance
                                && time_diff < props.swipe_max_duration
                                && speed > props.swipe_min_velocity
                            {
                                let direction = compute_direction(
                                    pointer_state.start_x,
                                    pointer_state.start_y,
                                    touch.client_x(),
                                    touch.client_y(),
                                );

                                let event = GestureSwipeEvent::new(touch.into(), direction);
                                on_swipe.emit(event)
                            }
                        }
                    }
                });
                self.state = DetectionState::Initial;
            }
            Msg::PointerMove(event) => {
                event.prevent_default();
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
                        //log::info!("DRAG TO {} {}", event.x(), event.y());
                        if let Some(on_drag_update) = &props.on_drag_update {
                            on_drag_update.emit(event.into());
                        }
                    }
                }
            }
            Msg::TouchMove(event) => {
                for_each_changed_touch(&event, |touch| {
                    if let Some(pointer_state) = self.update_pointer_position(
                        touch.identifier(),
                        touch.client_x(),
                        touch.client_y(),
                    ) {
                        let distance = compute_distance(
                            pointer_state.start_x,
                            pointer_state.start_y,
                            touch.client_x(),
                            touch.client_y(),
                        );
                        if distance >= props.tap_tolerance || pointer_state.got_tap_timeout {
                            //log::info!("DRAG TO {} {}", event.x(), event.y());
                            if let Some(on_drag_update) = &props.on_drag_update {
                                on_drag_update.emit(touch.into());
                            }
                        }
                    }
                });
            }
            Msg::PointerCancel(event) | Msg::PointerLeave(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                if let Some(_pointer_state) = self.unregister_pointer(event.pointer_id()) {
                    self.state = DetectionState::Initial;
                    //log::info!("DRAG END");
                    if let Some(on_drag_end) = &props.on_drag_end {
                        on_drag_end.emit(event.into());
                    }
                }
            }
            Msg::TouchCancel(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.unregister_touches(&event, |_id, touch, _pointer_state| {
                    //log::info!("DRAG END");
                    if let Some(on_drag_end) = &props.on_drag_end {
                        on_drag_end.emit(touch.into());
                    }
                });
                self.state = DetectionState::Initial;
            }
        }
        true
    }

    // Wait until all pointers are released
    fn update_error(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::TapTimeout(_id) => { /* ignore */ }
            Msg::LongPressTimeout(_id) => { /* ignore */ }
            Msg::PointerDown(event) => {
                self.register_pointer(ctx, &event);
            }
            Msg::TouchStart(event) => {
                self.register_touches(ctx, &event);
            }
            Msg::PointerUp(event) => {
                self.unregister_pointer(event.pointer_id());
                if self.pointers.is_empty() {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::TouchEnd(event) => {
                self.unregister_touches(&event, |_, _, _| {});
                if self.pointers.is_empty() {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::PointerMove(_event) => { /* ignore */ }
            Msg::TouchMove(_event) => { /* ignore */ }
            Msg::PointerCancel(event) => {
                self.unregister_pointer(event.pointer_id());
                if self.pointers.is_empty() {
                    self.state = DetectionState::Initial;
                }
            }
            Msg::TouchCancel(event) => {
                self.unregister_touches(&event, |_, _, _| {});
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
        let touch_only = window().has_own_property(&JsValue::from_str("ontouchstart"));

        Self {
            touch_only,
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
            DetectionState::Drag => self.update_drag(ctx, msg),
            DetectionState::Double => self.update_error(ctx, msg), // todo
            //DetectionState::Error => self.update_error(ctx, msg),
            DetectionState::Done => self.update_error(ctx, msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut container = Container::new()
            .node_ref(self.node_ref.clone())
            .class("pwt-d-contents")
            .style("touch-action", "none")
            .with_child(props.content.clone());

        if self.touch_only {
            container.add_ontouchstart(ctx.link().callback(Msg::TouchStart));
            container.add_ontouchmove(ctx.link().callback(Msg::TouchMove));
            container.add_ontouchcancel(ctx.link().callback(Msg::TouchCancel));
            container.add_ontouchend(ctx.link().callback(Msg::TouchEnd));
        } else {
            container.add_onpointerdown(ctx.link().callback(Msg::PointerDown));
            container.add_onpointerup(ctx.link().callback(Msg::PointerUp));
            container.add_onpointermove(ctx.link().callback(Msg::PointerMove));
            container.add_onpointercancel(ctx.link().callback(Msg::PointerCancel));
            container.add_onpointerleave(ctx.link().callback(Msg::PointerLeave));
        }
        container.into()
    }
}

impl From<GestureDetector> for VNode {
    fn from(val: GestureDetector) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtGestureDetector>(Rc::new(val), key);
        VNode::from(comp)
    }
}

impl_to_html!(GestureDetector);

// -180...180
fn compute_direction(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    let dx = (x2 - x1) as f64;
    let dy = (y1 - y2) as f64;
    (dy.atan2(dx) * 360.0) / (2.0 * std::f64::consts::PI)
}

fn compute_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    let dx = (x2 - x1) as f64;
    let dy = (y2 - y1) as f64;

    (dx * dx + dy * dy).sqrt()
}

fn for_each_changed_touch<F: FnMut(Touch)>(event: &TouchEvent, mut func: F) {
    let touch_list = event.changed_touches();
    for i in 0..touch_list.length() {
        if let Some(touch) = touch_list.get(i) {
            func(touch);
        }
    }
}

fn for_each_active_touch<F: FnMut(Touch)>(event: &TouchEvent, mut func: F) {
    let touch_list = event.touches();
    for i in 0..touch_list.length() {
        if let Some(touch) = touch_list.get(i) {
            func(touch);
        }
    }
}
