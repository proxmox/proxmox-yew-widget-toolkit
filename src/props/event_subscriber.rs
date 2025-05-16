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
    ($id:ident, $add_id:ident, $et:ty) => {
        /// Builder style method to set the callback
        fn $id(mut self, cb: impl ::yew::html::IntoEventCallback<$et>) -> Self {
            self.$add_id(cb);
            self
        }

        /// Method to set the callback
        fn $add_id(&mut self, cb: impl ::yew::html::IntoEventCallback<$et>) {
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

    handler!(onauxclick, add_onauxclick, MouseEvent);
    handler!(onclick, add_onclick, MouseEvent);
    handler!(oncontextmenu, add_oncontextmenu, MouseEvent);
    handler!(ondblclick, add_ondblclick, MouseEvent);
    handler!(onmousedown, add_onmousedown, MouseEvent);
    handler!(onmouseenter, add_onmouseenter, MouseEvent);
    handler!(onmouseleave, add_onmouseleave, MouseEvent);
    handler!(onmousemove, add_onmousemove, MouseEvent);
    handler!(onmouseout, add_onmouseout, MouseEvent);
    handler!(onmouseover, add_onmouseover, MouseEvent);
    handler!(onmouseup, add_onmouseup, MouseEvent);

    handler!(onkeydown, add_onkeydown, KeyboardEvent);
    handler!(onkeyup, add_onkeyup, KeyboardEvent);
    handler!(onkeypress, add_onkeypress, KeyboardEvent);

    handler!(oninput, add_oninput, InputEvent);

    handler!(onblur, add_onblur, FocusEvent);
    handler!(onfocus, add_onfocus, FocusEvent);
    handler!(onfocusin, add_onfocusin, FocusEvent);
    handler!(onfocusout, add_onfocusout, FocusEvent);
    handler!(onsubmit, add_onsubmit, SubmitEvent);

    handler!(onabort, add_onabort, Event);
    handler!(oncancel, add_oncancel, Event);
    handler!(oncanplay, add_oncanplay, Event);
    handler!(oncanplaythrough, add_oncanplaythrough, Event);
    handler!(onchange, add_onchange, Event);
    handler!(onclose, add_onclose, Event);
    handler!(oncuechange, add_oncuechange, Event);
    handler!(ondurationchange, add_ondurationchange, Event);
    handler!(onemptied, add_onemptied, Event);
    handler!(onended, add_onended, Event);
    handler!(onerror, add_onerror, Event);
    handler!(onformdata, add_onformdata, Event);
    handler!(oninvalid, add_oninvalid, Event);
    handler!(onload, add_onload, Event);
    handler!(onloadeddata, add_onloadeddata, Event);
    handler!(onloadedmetadata, add_onloadedmetadata, Event);
    handler!(onpause, add_onpause, Event);
    handler!(onplay, add_onplay, Event);
    handler!(onplaying, add_onplaying, Event);
    handler!(onratechange, add_onratechange, Event);
    handler!(onreset, add_onreset, Event);
    handler!(onresize, add_onresize, Event);
    handler!(onscroll, add_onscroll, Event);
    handler!(
        onsecuritypolicyviolation,
        add_onsecuritypolicyviolation,
        Event
    );
    handler!(onseeked, add_onseeked, Event);
    handler!(onseeking, add_onseeking, Event);
    handler!(onselect, add_onselect, Event);
    handler!(onslotchange, add_onslotchange, Event);
    handler!(onstalled, add_onstalled, Event);
    handler!(onsuspend, add_onsuspend, Event);
    handler!(ontimeupdate, add_ontimeupdate, Event);
    handler!(ontoggle, add_ontoggle, Event);
    handler!(onvolumechange, add_onvolumechange, Event);
    handler!(onwaiting, add_onwaiting, Event);
    handler!(oncopy, add_oncopy, Event);
    handler!(oncut, add_oncut, Event);
    handler!(onpaste, add_onpaste, Event);
    handler!(onpointerlockchange, add_onpointerlockchange, Event);
    handler!(onpointerlockerror, add_onpointerlockerror, Event);
    handler!(onselectionchange, add_onselectionchange, Event);
    handler!(onselectstart, add_onselectstart, Event);
    handler!(onshow, add_onshow, Event);

    handler!(onloadstart, add_onloadstart, ProgressEvent);
    handler!(onprogress, add_onprogress, ProgressEvent);
    handler!(onloadend, add_onloadend, ProgressEvent);

    handler!(ondrag, add_ondrag, DragEvent);
    handler!(ondragend, add_ondragend, DragEvent);
    handler!(ondragenter, add_ondragenter, DragEvent);
    handler!(ondragexit, add_ondragexit, DragEvent);
    handler!(ondragleave, add_ondragleave, DragEvent);
    handler!(ondragover, add_ondragover, DragEvent);
    handler!(ondragstart, add_ondragstart, DragEvent);
    handler!(ondrop, add_ondrop, DragEvent);

    handler!(onanimationcancel, add_onanimationcancel, AnimationEvent);
    handler!(onanimationend, add_onanimationend, AnimationEvent);
    handler!(
        onanimationiteration,
        add_onanimationiteration,
        AnimationEvent
    );
    handler!(onanimationstart, add_onanimationstart, AnimationEvent);

    handler!(ongotpointercapture, add_ongotpointercapture, PointerEvent);
    handler!(onlostpointercapture, add_onlostpointercapture, PointerEvent);
    handler!(onpointercancel, add_onpointercancel, PointerEvent);
    handler!(onpointerdown, add_onpointerdown, PointerEvent);
    handler!(onpointerenter, add_onpointerenter, PointerEvent);
    handler!(onpointerleave, add_onpointerleave, PointerEvent);
    handler!(onpointermove, add_onpointermove, PointerEvent);
    handler!(onpointerout, add_onpointerout, PointerEvent);
    handler!(onpointerover, add_onpointerover, PointerEvent);
    handler!(onpointerup, add_onpointerup, PointerEvent);

    handler!(ontouchcancel, add_ontouchcancel, TouchEvent);
    handler!(ontouchend, add_ontouchend, TouchEvent);
    handler!(ontouchmove, add_ontouchmove, TouchEvent);
    handler!(ontouchstart, add_ontouchstart, TouchEvent);

    handler!(ontransitioncancel, add_ontransitioncancel, TransitionEvent);
    handler!(ontransitionend, add_ontransitionend, TransitionEvent);
    handler!(ontransitionrun, add_ontransitionrun, TransitionEvent);
    handler!(ontransitionstart, add_ontransitionstart, TransitionEvent);

    handler!(onwheel, add_onwheel, WheelEvent);
}
