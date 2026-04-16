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
use crate::props::{
    ContainerBuilder, EventSubscriber, IntoVTag, WidgetBuilder, WidgetStyleBuilder,
};
use crate::widget::Container;

/// An event that can happen from a [`PointerEvent`] or a [`Touch`]
///
/// For convenience, expose the most important values from the underlying events
#[derive(Clone, PartialEq)]
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
#[derive(Clone, PartialEq)]
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

/// Determines the phase of the Gesture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GesturePhase {
    /// The gesture just started
    Start,
    /// The gesture is already in progress and is updated
    Update,
    /// The gesture ended and this is the last update
    End,
}

/// An event that can happen when the user uses a Pinch/Zoom gesture
#[derive(Clone, PartialEq)]
pub struct GesturePinchZoomEvent {
    /// The current phase of the event
    pub phase: GesturePhase,

    /// First touch/pointer [Point] of the Pinch/Zoom event
    pub point0: PinchPoint,
    /// Second touch/pointer [Point] of the Pinch/Zoom event
    pub point1: PinchPoint,

    /// Current angle of the gesture, relative to the starting position
    pub angle: f64,

    /// Current scale of the distance between touch points relative to the starting positions
    pub scale: f64,
}

impl GesturePinchZoomEvent {
    fn new(
        phase: GesturePhase,
        point0: PinchPoint,
        point1: PinchPoint,
        angle: f64,
        scale: f64,
    ) -> Self {
        Self {
            phase,
            point0,
            point1,
            angle,
            scale,
        }
    }
}

/// An event that can happen when the user uses a drag gesture
#[derive(Clone, PartialEq)]
pub struct GestureDragEvent {
    /// The current phase of the event
    pub phase: GesturePhase,

    event: InputEvent,
}

impl GestureDragEvent {
    fn new(event: InputEvent, phase: GesturePhase) -> Self {
        Self { event, phase }
    }
}

impl Deref for GestureDragEvent {
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
/// - pinch/zoom: fired when two touches/pointers move.
///
/// # Note
///
/// We use "display: contents;", so events reports wrong relative coordiantes (offsetX and offsetY).
///
/// Nested gesture detection is currently not implemented.
///
/// It might be necessary to apply 'touch-action: none' to the content element.
///
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

    /// Callback for drag events.
    #[prop_or_default]
    pub on_drag: Option<Callback<GestureDragEvent>>,

    #[prop_or_default]
    pub on_swipe: Option<Callback<GestureSwipeEvent>>,

    /// Callback for Pinch/Zoom gesture event.
    #[prop_or_default]
    pub on_pinch_zoom: Option<Callback<GesturePinchZoomEvent>>,
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

    /// Builder style method to set the on_drag callback
    pub fn on_drag(mut self, cb: impl IntoEventCallback<GestureDragEvent>) -> Self {
        self.on_drag = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_swipe callback
    pub fn on_swipe(mut self, cb: impl IntoEventCallback<GestureSwipeEvent>) -> Self {
        self.on_swipe = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_pinch_zoom callback
    pub fn on_pinch_zoom(mut self, cb: impl IntoEventCallback<GesturePinchZoomEvent>) -> Self {
        self.on_pinch_zoom = cb.into_event_callback();
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
    Multi,
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

impl PointerState {
    fn to_pinch_point(&self, id: i32) -> PinchPoint {
        PinchPoint {
            id,
            x: self.x,
            y: self.y,
        }
    }
}

/// Represents a single pointer or touch
#[derive(Debug, Clone, PartialEq)]
pub struct PinchPoint {
    /// The numeric ID of the touch or pointer
    pub id: i32,
    /// The x coordinate in pixels
    pub x: i32,
    /// The y coordinate in pixels
    pub y: i32,
}

impl PinchPoint {
    /// calculates the distance in pixels to another [Point]
    pub fn distance(&self, other: &PinchPoint) -> f64 {
        compute_distance(self.x, self.y, other.x, other.y)
    }

    /// calculates the angle of the line to another [Point] in radians
    pub fn angle(&self, other: &PinchPoint) -> f64 {
        let x_diff = (other.x - self.x) as f64;
        let y_diff = (-other.y + self.y) as f64;

        y_diff.atan2(x_diff) + std::f64::consts::PI
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct PinchZoomInfo {
    start_angle: f64,
    current_angle: f64,
    start_distance: f64,
    current_distance: f64,
}

impl PinchZoomInfo {
    fn new(point0: PinchPoint, point1: PinchPoint) -> Self {
        let angle = point0.angle(&point1);

        // force a minimal distance of 1 pixel
        let distance = point0.distance(&point1).max(1.0);

        Self {
            start_angle: angle,
            current_angle: angle,
            start_distance: distance,
            current_distance: distance,
        }
    }

    fn update(&mut self, point0: PinchPoint, point1: PinchPoint) {
        let last_angle = self.current_angle;
        let rotations = (last_angle / std::f64::consts::TAU).round();

        let angle = point0.angle(&point1) + rotations * std::f64::consts::TAU;

        if (last_angle - angle).abs() < std::f64::consts::PI {
            self.current_angle = angle;
        } else if last_angle > angle {
            self.current_angle = angle + std::f64::consts::TAU;
        } else if last_angle < angle {
            self.current_angle = angle - std::f64::consts::TAU;
        }

        self.current_distance = point0.distance(&point1);
    }
}

#[doc(hidden)]
pub struct PwtGestureDetector {
    touch_only: bool,
    node_ref: NodeRef,
    state: DetectionState,
    pointers: HashMap<i32, PointerState>,
    pinch_zoom_info: PinchZoomInfo,
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

        if self.pointers.len() == 2 {
            self.start_pinch_zoom();
        }
    }

    fn register_touches(&mut self, ctx: &Context<Self>, event: &TouchEvent) {
        for_each_changed_touch(event, |touch: Touch| {
            let id = touch.identifier();
            let x = touch.client_x();
            let y = touch.client_y();
            self.register_pointer_state(ctx, id, x, y);
        });

        if self.pointers.len() == 2 {
            self.start_pinch_zoom();
        }
    }

    fn start_pinch_zoom(&mut self) {
        let (point0, point1) = self.get_pinch_points();
        self.pinch_zoom_info = PinchZoomInfo::new(point0, point1)
    }

    fn update_pinch_zoom(&mut self) {
        let (point0, point1) = self.get_pinch_points();
        self.pinch_zoom_info.update(point0, point1);
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

    fn get_pinch_points(&self) -> (PinchPoint, PinchPoint) {
        let mut points: Vec<_> = self
            .pointers
            .iter()
            .map(|(id, pointer)| pointer.to_pinch_point(*id))
            .collect();
        assert!(points.len() == 2);

        // sort for stable stable order
        points.sort_by_key(|p| p.id);

        (points.remove(0), points.remove(0))
    }

    fn get_angle(&self) -> f64 {
        self.pinch_zoom_info.current_angle - self.pinch_zoom_info.start_angle
    }

    fn get_scale(&self) -> f64 {
        self.pinch_zoom_info.current_distance / self.pinch_zoom_info.start_distance
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
                    2 => {
                        let (point0, point1) = self.get_pinch_points();
                        self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Start);
                        DetectionState::Double
                    }
                    _ => DetectionState::Multi,
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
                let (point0, point1) = self.get_pinch_points();
                self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Start);
            }
            Msg::TouchStart(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.register_touches(ctx, &event);
                self.state = match self.pointers.len() {
                    0 => DetectionState::Initial,
                    1 => DetectionState::Single,
                    2 => {
                        let (point0, point1) = self.get_pinch_points();
                        self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Start);
                        DetectionState::Double
                    }
                    _ => DetectionState::Multi,
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
                        if let Some(on_drag) = &props.on_drag {
                            on_drag.emit(GestureDragEvent::new(event.into(), GesturePhase::Start));
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
                            if let Some(on_drag) = &props.on_drag {
                                on_drag
                                    .emit(GestureDragEvent::new(touch.into(), GesturePhase::Start));
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
                if let Some(on_drag) = &props.on_drag {
                    on_drag.emit(GestureDragEvent::new(event.into(), GesturePhase::End));
                }
                let (point0, point1) = self.get_pinch_points();
                self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Start);
            }
            Msg::TouchStart(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                // Abort current drags
                self.register_touches(ctx, &event);
                let pointer_count = self.pointers.len();
                match pointer_count {
                    2 => {
                        self.state = DetectionState::Double;

                        let (point0, point1) = self.get_pinch_points();
                        self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Start);
                    }
                    count if count > 2 => {
                        self.state = DetectionState::Multi;
                    }
                    _ => {}
                }
                for_each_active_touch(&event, |touch| {
                    if self.pointers.contains_key(&touch.identifier()) {
                        if let Some(on_drag) = &props.on_drag {
                            on_drag.emit(GestureDragEvent::new(touch.into(), GesturePhase::End));
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
                    if let Some(on_drag) = &props.on_drag {
                        on_drag.emit(GestureDragEvent::new(
                            event.clone().into(),
                            GesturePhase::End,
                        ));
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
                        if let Some(on_drag) = &props.on_drag {
                            on_drag.emit(GestureDragEvent::new(
                                touch.clone().into(),
                                GesturePhase::End,
                            ));
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
                        if let Some(on_drag) = &props.on_drag {
                            on_drag.emit(GestureDragEvent::new(event.into(), GesturePhase::Update));
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
                            if let Some(on_drag) = &props.on_drag {
                                on_drag.emit(GestureDragEvent::new(
                                    touch.into(),
                                    GesturePhase::Update,
                                ));
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
                    if let Some(on_drag) = &props.on_drag {
                        on_drag.emit(GestureDragEvent::new(event.into(), GesturePhase::End));
                    }
                }
            }
            Msg::TouchCancel(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 1);
                self.unregister_touches(&event, |_id, touch, _pointer_state| {
                    //log::info!("DRAG END");
                    if let Some(on_drag) = &props.on_drag {
                        on_drag.emit(GestureDragEvent::new(touch.into(), GesturePhase::End));
                    }
                });
                self.state = DetectionState::Initial;
            }
        }
        true
    }

    fn call_on_pinch_zoom(
        &mut self,
        ctx: &Context<Self>,
        point0: PinchPoint,
        point1: PinchPoint,
        phase: GesturePhase,
    ) {
        if let Some(on_pinch_zoom) = &ctx.props().on_pinch_zoom {
            on_pinch_zoom.emit(GesturePinchZoomEvent::new(
                phase,
                point0,
                point1,
                self.get_angle(),
                self.get_scale(),
            ))
        }
    }

    fn update_double(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::TapTimeout(_id) => { /* ignore */ }
            Msg::LongPressTimeout(_id) => { /* ignore */ }
            Msg::PointerDown(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 2);
                let (point0, point1) = self.get_pinch_points();
                self.register_pointer(ctx, &event);
                self.state = DetectionState::Multi;
                self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::End);
            }
            Msg::TouchStart(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 2);
                let (point0, point1) = self.get_pinch_points();
                self.register_touches(ctx, &event);
                self.state = DetectionState::Multi;
                self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::End);
            }
            Msg::PointerUp(event) | Msg::PointerCancel(event) | Msg::PointerLeave(event) => {
                event.prevent_default();
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 2);
                let (point0, point1) = self.get_pinch_points();
                if self.unregister_pointer(event.pointer_id()).is_some() {
                    self.state = DetectionState::Drag;
                }
                self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::End);
            }
            Msg::TouchEnd(event) | Msg::TouchCancel(event) => {
                let pointer_count = self.pointers.len();
                assert!(pointer_count == 2);
                let (point0, point1) = self.get_pinch_points();
                let mut unregistered = 0;
                for_each_changed_touch(&event, |touch| {
                    if self.unregister_pointer(touch.identifier()).is_some() {
                        unregistered += 1;
                    }
                });
                let pointer_count = pointer_count.saturating_sub(unregistered);
                if pointer_count < 2 {
                    self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::End);
                }
                match pointer_count {
                    0 => self.state = DetectionState::Initial,
                    1 => self.state = DetectionState::Drag,
                    2 => {}
                    _more => self.state = DetectionState::Multi, // more touchpoints on removal?
                }
            }
            Msg::PointerMove(event) => {
                event.prevent_default();
                let updated = self
                    .update_pointer_position(event.pointer_id(), event.x(), event.y())
                    .is_some();

                self.update_pinch_zoom();

                if updated {
                    let (point0, point1) = self.get_pinch_points();
                    self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Update);
                }
            }
            Msg::TouchMove(event) => {
                let mut had_valid = false;
                for_each_changed_touch(&event, |touch| {
                    if self
                        .update_pointer_position(
                            touch.identifier(),
                            touch.client_x(),
                            touch.client_y(),
                        )
                        .is_some()
                    {
                        had_valid = true
                    }
                });
                self.update_pinch_zoom();
                if had_valid {
                    let (point0, point1) = self.get_pinch_points();
                    self.call_on_pinch_zoom(ctx, point0, point1, GesturePhase::Update);
                }
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
            pinch_zoom_info: PinchZoomInfo::default(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        //let props = ctx.props();

        match self.state {
            DetectionState::Initial => self.update_initial(ctx, msg),
            DetectionState::Single => self.update_single(ctx, msg),
            DetectionState::Drag => self.update_drag(ctx, msg),
            DetectionState::Double => self.update_double(ctx, msg),
            DetectionState::Multi => self.update_error(ctx, msg), // todo
            //DetectionState::Error => self.update_error(ctx, msg),
            DetectionState::Done => self.update_error(ctx, msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut container = Container::new()
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
        container.into_html_with_ref(self.node_ref.clone())
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
