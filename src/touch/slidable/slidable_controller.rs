use yew::prelude::*;

/// Messages for the [Slidable](super::Slidable)
#[doc(hidden)]
pub enum SlidableControllerMsg {
    Collapse,
    Dismiss,
}

#[derive(Clone, PartialEq)]
pub struct SlidableController {
    callback: Callback<SlidableControllerMsg>,
}

impl SlidableController {
    pub(crate) fn new(callback: Callback<SlidableControllerMsg>) -> Self {
        Self { callback }
    }

    /// Collapse the sidable widget.
    pub fn collapse(&self) {
        self.callback.emit(SlidableControllerMsg::Collapse);
    }

    /// Dismiss the sidable widget.
    pub fn dismiss(&self) {
        self.callback.emit(SlidableControllerMsg::Dismiss);
    }
}
