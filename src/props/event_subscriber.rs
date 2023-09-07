use std::rc::Rc;

use yew::virtual_dom::{Listener, VNode};
use yew::{
    AnimationEvent, DragEvent, Event, FocusEvent, InputEvent,
    MouseEvent, KeyboardEvent, PointerEvent, ProgressEvent,
    SubmitEvent, TouchEvent, TransitionEvent, WheelEvent,
};

/// A list of Html event listeners
///
/// Like [yew::virtual_dom::Listeners], but uses a `Vec` to store the
/// list. This way we can dynamically add items (see [EventSubscriber]
/// trait).
#[derive(Default, Debug, Clone)]
pub struct ListenersWrapper {
    pub(crate) listeners: Vec<Option<Rc<dyn Listener>>>,
}

impl PartialEq for ListenersWrapper {
    fn eq(&self, other: &Self) -> bool {
        if self.listeners.len() != other.listeners.len() { return false; }
        if self.listeners.is_empty() { return true; }
        for i in 0..self.listeners.len() {
            match (&self.listeners[i], &other.listeners[i]) {
                (Some(left), Some(right)) => {
                    if !Rc::ptr_eq(left, right) { return false; }
                }
                (None, None) => { /* same, continue */ }
                _ => { return false; }
            }
        }
        true
    }
}

macro_rules! handler {

    ($id:ident, $et:ty) => {
        /// Builder style method to set the callback
        fn $id(mut self, cb: impl ::yew::html::IntoEventCallback<$et>) -> Self {
            if let Some(cb) = cb.into_event_callback() {
                let listener: ::std::rc::Rc<dyn ::yew::virtual_dom::Listener> =
                    Rc::new(yew::html::$id::Wrapper::new(cb));
                self.as_listeners_mut().listeners.push(Some(listener));
            }
            self
        }
    }

}

/// Defines builder methods on [ListenersWrapper].
///
/// This trait defines builder method for all Html events.
pub trait EventSubscriber: Into<VNode> {

    /// Mutable access to the [ListenersWrapper].
    fn as_listeners_mut(&mut self) -> &mut ListenersWrapper;

    /// Set all Listeners - usefull to copy the whole set of listeners.
    fn listeners(mut self, list: &ListenersWrapper) -> Self  {
        *self.as_listeners_mut() = list.clone();
        self
    }

    handler!(onauxclick, MouseEvent);
    handler!(onclick, MouseEvent);
    handler!(oncontextmenu, MouseEvent);
    handler!(ondblclick, MouseEvent);
    handler!(onmousedown, MouseEvent);
    handler!(onmouseenter, MouseEvent);
    handler!(onmouseleave, MouseEvent);
    handler!(onmousemove, MouseEvent);
    handler!(onmouseout, MouseEvent);
    handler!(onmouseover, MouseEvent);
    handler!(onmouseup, MouseEvent);

    handler!(onkeydown, KeyboardEvent);
    handler!(onkeyup, KeyboardEvent);
    handler!(onkeypress, KeyboardEvent);

    handler!(oninput, InputEvent);

    handler!(onblur, FocusEvent);
    handler!(onfocus, FocusEvent);
    handler!(onfocusin, FocusEvent);
    handler!(onfocusout, FocusEvent);
    handler!(onsubmit, SubmitEvent);

    handler!(onabort, Event);
    handler!(oncancel, Event);
    handler!(oncanplay, Event);
    handler!(oncanplaythrough, Event);
    handler!(onchange, Event);
    handler!(onclose, Event);
    handler!(oncuechange, Event);
    handler!(ondurationchange, Event);
    handler!(onemptied, Event);
    handler!(onended, Event);
    handler!(onerror, Event);
    handler!(onformdata, Event);
    handler!(oninvalid, Event);
    handler!(onload, Event);
    handler!(onloadeddata, Event);
    handler!(onloadedmetadata, Event);
    handler!(onpause, Event);
    handler!(onplay, Event);
    handler!(onplaying, Event);
    handler!(onratechange, Event);
    handler!(onreset, Event);
    handler!(onresize, Event);
    handler!(onscroll, Event);
    handler!(onsecuritypolicyviolation, Event);
    handler!(onseeked, Event);
    handler!(onseeking, Event);
    handler!(onselect, Event);
    handler!(onslotchange, Event);
    handler!(onstalled, Event);
    handler!(onsuspend, Event);
    handler!(ontimeupdate, Event);
    handler!(ontoggle, Event);
    handler!(onvolumechange, Event);
    handler!(onwaiting, Event);
    handler!(oncopy, Event);
    handler!(oncut, Event);
    handler!(onpaste, Event);
    handler!(onpointerlockchange, Event);
    handler!(onpointerlockerror, Event);
    handler!(onselectionchange, Event);
    handler!(onselectstart, Event);
    handler!(onshow, Event);

    handler!(onloadstart, ProgressEvent);
    handler!(onprogress, ProgressEvent);
    handler!(onloadend, ProgressEvent);

    handler!(ondrag, DragEvent);
    handler!(ondragend, DragEvent);
    handler!(ondragenter, DragEvent);
    handler!(ondragexit, DragEvent);
    handler!(ondragleave, DragEvent);
    handler!(ondragover, DragEvent);
    handler!(ondragstart, DragEvent);
    handler!(ondrop, DragEvent);

    handler!(onanimationcancel, AnimationEvent);
    handler!(onanimationend, AnimationEvent);
    handler!(onanimationiteration, AnimationEvent);
    handler!(onanimationstart, AnimationEvent);

    handler!(ongotpointercapture, PointerEvent);
    handler!(onlostpointercapture, PointerEvent);
    handler!(onpointercancel, PointerEvent);
    handler!(onpointerdown, PointerEvent);
    handler!(onpointerenter, PointerEvent);
    handler!(onpointerleave, PointerEvent);
    handler!(onpointermove, PointerEvent);
    handler!(onpointerout, PointerEvent);
    handler!(onpointerover, PointerEvent);
    handler!(onpointerup, PointerEvent);

    handler!(ontouchcancel, TouchEvent);
    handler!(ontouchend, TouchEvent);
    handler!(ontouchmove, TouchEvent);
    handler!(ontouchstart, TouchEvent);

    handler!(ontransitioncancel, TransitionEvent);
    handler!(ontransitionend, TransitionEvent);
    handler!(ontransitionrun, TransitionEvent);
    handler!(ontransitionstart, TransitionEvent);

    handler!(onwheel, WheelEvent);
}
