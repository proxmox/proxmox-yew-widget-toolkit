use std::rc::Rc;

use yew::virtual_dom::{Listener, VNode};
use yew::{
    AnimationEvent, DragEvent, Event, FocusEvent, InputEvent, KeyboardEvent, MouseEvent,
    PointerEvent, ProgressEvent, SubmitEvent, TouchEvent, TransitionEvent, WheelEvent,
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
        if self.listeners.len() != other.listeners.len() {
            return false;
        }
        if self.listeners.is_empty() {
            return true;
        }
        for i in 0..self.listeners.len() {
            match (&self.listeners[i], &other.listeners[i]) {
                (Some(left), Some(right)) => {
                    if !Rc::ptr_eq(left, right) {
                        return false;
                    }
                }
                (None, None) => { /* same, continue */ }
                _ => {
                    return false;
                }
            }
        }
        true
    }
}

macro_rules! handler {
    ($id:ident, $set_id:ident, $et:ty) => {
        /// Builder style method to set the callback
        fn $id(mut self, cb: impl ::yew::html::IntoEventCallback<$et>) -> Self {
            self.$set_id(cb);
            self
        }

        /// Method to set the callback
        fn $set_id(&mut self, cb: impl ::yew::html::IntoEventCallback<$et>) {
            if let Some(cb) = cb.into_event_callback() {
                let listener: ::std::rc::Rc<dyn ::yew::virtual_dom::Listener> =
                    Rc::new(yew::html::$id::Wrapper::new(cb));
                self.as_listeners_mut().listeners.push(Some(listener));
            }
        }
    };
}
/// Defines builder methods on [ListenersWrapper].
///
/// This trait defines builder method for all Html events.
pub trait EventSubscriber: Into<VNode> {
    /// Mutable access to the [ListenersWrapper].
    fn as_listeners_mut(&mut self) -> &mut ListenersWrapper;

    /// Set all Listeners - usefull to copy the whole set of listeners.
    fn listeners(mut self, list: &ListenersWrapper) -> Self {
        *self.as_listeners_mut() = list.clone();
        self
    }

    handler!(onauxclick, set_onauxclick, MouseEvent);
    handler!(onclick, set_onclick, MouseEvent);
    handler!(oncontextmenu, set_oncontextmenu, MouseEvent);
    handler!(ondblclick, set_ondblclick, MouseEvent);
    handler!(onmousedown, set_onmousedown, MouseEvent);
    handler!(onmouseenter, set_onmouseenter, MouseEvent);
    handler!(onmouseleave, set_onmouseleave, MouseEvent);
    handler!(onmousemove, set_onmousemove, MouseEvent);
    handler!(onmouseout, set_onmouseout, MouseEvent);
    handler!(onmouseover, set_onmouseover, MouseEvent);
    handler!(onmouseup, set_onmouseup, MouseEvent);

    handler!(onkeydown, set_onkeydown, KeyboardEvent);
    handler!(onkeyup, set_onkeyup, KeyboardEvent);
    handler!(onkeypress, set_onkeypress, KeyboardEvent);

    handler!(oninput, set_oninput, InputEvent);

    handler!(onblur, set_onblur, FocusEvent);
    handler!(onfocus, set_onfocus, FocusEvent);
    handler!(onfocusin, set_onfocusin, FocusEvent);
    handler!(onfocusout, set_onfocusout, FocusEvent);
    handler!(onsubmit, set_onsubmit, SubmitEvent);

    handler!(onabort, set_onabort, Event);
    handler!(oncancel, set_oncancel, Event);
    handler!(oncanplay, set_oncanplay, Event);
    handler!(oncanplaythrough, set_oncanplaythrough, Event);
    handler!(onchange, set_onchange, Event);
    handler!(onclose, set_onclose, Event);
    handler!(oncuechange, set_oncuechange, Event);
    handler!(ondurationchange, set_ondurationchange, Event);
    handler!(onemptied, set_onemptied, Event);
    handler!(onended, set_onended, Event);
    handler!(onerror, set_onerror, Event);
    handler!(onformdata, set_onformdata, Event);
    handler!(oninvalid, set_oninvalid, Event);
    handler!(onload, set_onload, Event);
    handler!(onloadeddata, set_onloadeddata, Event);
    handler!(onloadedmetadata, set_onloadedmetadata, Event);
    handler!(onpause, set_onpause, Event);
    handler!(onplay, set_onplay, Event);
    handler!(onplaying, set_onplaying, Event);
    handler!(onratechange, set_onratechange, Event);
    handler!(onreset, set_onreset, Event);
    handler!(onresize, set_onresize, Event);
    handler!(onscroll, set_onscroll, Event);
    handler!(
        onsecuritypolicyviolation,
        set_onsecuritypolicyviolation,
        Event
    );
    handler!(onseeked, set_onseeked, Event);
    handler!(onseeking, set_onseeking, Event);
    handler!(onselect, set_onselect, Event);
    handler!(onslotchange, set_onslotchange, Event);
    handler!(onstalled, set_onstalled, Event);
    handler!(onsuspend, set_onsuspend, Event);
    handler!(ontimeupdate, set_ontimeupdate, Event);
    handler!(ontoggle, set_ontoggle, Event);
    handler!(onvolumechange, set_onvolumechange, Event);
    handler!(onwaiting, set_onwaiting, Event);
    handler!(oncopy, set_oncopy, Event);
    handler!(oncut, set_oncut, Event);
    handler!(onpaste, set_onpaste, Event);
    handler!(onpointerlockchange, set_onpointerlockchange, Event);
    handler!(onpointerlockerror, set_onpointerlockerror, Event);
    handler!(onselectionchange, set_onselectionchange, Event);
    handler!(onselectstart, set_onselectstart, Event);
    handler!(onshow, set_onshow, Event);

    handler!(onloadstart, set_onloadstart, ProgressEvent);
    handler!(onprogress, set_onprogress, ProgressEvent);
    handler!(onloadend, set_onloadend, ProgressEvent);

    handler!(ondrag, set_ondrag, DragEvent);
    handler!(ondragend, set_ondragend, DragEvent);
    handler!(ondragenter, set_ondragenter, DragEvent);
    handler!(ondragexit, set_ondragexit, DragEvent);
    handler!(ondragleave, set_ondragleave, DragEvent);
    handler!(ondragover, set_ondragover, DragEvent);
    handler!(ondragstart, set_ondragstart, DragEvent);
    handler!(ondrop, set_ondrop, DragEvent);

    handler!(onanimationcancel, set_onanimationcancel, AnimationEvent);
    handler!(onanimationend, set_onanimationend, AnimationEvent);
    handler!(
        onanimationiteration,
        set_onanimationiteration,
        AnimationEvent
    );
    handler!(onanimationstart, set_onanimationstart, AnimationEvent);

    handler!(ongotpointercapture, set_ongotpointercapture, PointerEvent);
    handler!(onlostpointercapture, set_onlostpointercapture, PointerEvent);
    handler!(onpointercancel, set_onpointercancel, PointerEvent);
    handler!(onpointerdown, set_onpointerdown, PointerEvent);
    handler!(onpointerenter, set_onpointerenter, PointerEvent);
    handler!(onpointerleave, set_onpointerleave, PointerEvent);
    handler!(onpointermove, set_onpointermove, PointerEvent);
    handler!(onpointerout, set_onpointerout, PointerEvent);
    handler!(onpointerover, set_onpointerover, PointerEvent);
    handler!(onpointerup, set_onpointerup, PointerEvent);

    handler!(ontouchcancel, set_ontouchcancel, TouchEvent);
    handler!(ontouchend, set_ontouchend, TouchEvent);
    handler!(ontouchmove, set_ontouchmove, TouchEvent);
    handler!(ontouchstart, set_ontouchstart, TouchEvent);

    handler!(ontransitioncancel, set_ontransitioncancel, TransitionEvent);
    handler!(ontransitionend, set_ontransitionend, TransitionEvent);
    handler!(ontransitionrun, set_ontransitionrun, TransitionEvent);
    handler!(ontransitionstart, set_ontransitionstart, TransitionEvent);

    handler!(onwheel, set_onwheel, WheelEvent);
}
