use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{AsClassesMut, WidgetBuilder};
use crate::widget::Button;

use super::GestureSwipeEvent;

/// Favorite actions button.
#[derive(Properties, Clone, PartialEq)]
pub struct Fab {
    /// The yew component key.
    pub key: Option<Key>,

    /// Icon (CSS class).
    pub icon_class: Classes,

    /// Use the small variant
    #[prop_or_default]
    pub small: bool,

    /// CSS class.
    #[prop_or_default]
    pub class: Classes,
}

impl AsClassesMut for Fab {
    fn as_classes_mut(&mut self) -> &mut yew::Classes {
        &mut self.class
    }
}

impl Fab {
    /// Create a new instance.
    pub fn new(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {
            icon_class: icon_class.into(),
        })
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Builder style method to set the small flag
    pub fn small(mut self, small: bool) -> Self {
        self.set_small(small);
        self
    }

    /// Method to set the small flag
    pub fn set_small(&mut self, small: bool) {
        self.small = small;
    }
}

#[doc(hidden)]
pub struct PwtFab {}

impl Component for PwtFab {
    type Message = ();
    type Properties = Fab;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut icon_class = props.icon_class.clone();
        icon_class.push("pwt-fab-icon");

        let mut class = props.class.clone();
        class.push("pwt-fab");
        if props.small {
            class.push("pwt-fab-small");
        }
        Button::new_icon(icon_class).class(class).into()
    }
}

impl Into<VNode> for Fab {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtFab>(Rc::new(self), key);
        VNode::from(comp)
    }
}
