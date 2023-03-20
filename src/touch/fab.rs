use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{AsClassesMut, EventSubscriber, WidgetBuilder};
use crate::widget::Button;

use super::GestureSwipeEvent;

/// Favorite actions button.
#[derive(Properties, Clone, PartialEq)]
pub struct Fab {
    /// The yew component key.
    pub key: Option<Key>,

    /// Icon (CSS class).
    pub icon_class: Classes,

    /// Optional Button text (for small buttons)
    pub text: Option<AttrValue>,

    /// CSS class.
    #[prop_or_default]
    pub class: Classes,

    /// Style attribute (use this to set button position)
    pub style: Option<AttrValue>,

    /// Click callback
    pub on_click: Option<Callback<MouseEvent>>,
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

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.key = key.into_prop_value();
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

    /// Builder style method to set the html style
    pub fn style(mut self, style: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_style(style);
        self
    }

    /// Method to set the html style
    pub fn set_style(&mut self, style: impl IntoPropValue<Option<AttrValue>>) {
        self.style = style.into_prop_value();
    }

    /// Builder style method to add the "pwt-fab-small" class
    pub fn small(mut self) -> Self {
        self.add_class("pwt-fab-small");
        self
    }

    /// Builder style method to add the "pwt-fab-large" class
    pub fn large(mut self) -> Self {
        self.add_class("pwt-fab-large");
        self
    }

    /// Builder style method to set the button text
    pub fn text(mut self, text: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_text(text);
        self
    }

    /// Method to set the button text
    pub fn set_text(&mut self, text: impl IntoPropValue<Option<AttrValue>>) {
        self.text = text.into_prop_value();
    }

    /// Builder style method to set the on_click callback.
    pub fn on_click(mut self, cb: impl IntoEventCallback<MouseEvent>) -> Self {
        self.on_click = cb.into_event_callback();
        self
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

        let button = match &props.text {
            Some(text) => Button::new(text).icon_class(icon_class).class("pwt-fab-extended"),
            None => Button::new_icon(icon_class),
        };

        button
            .class(class)
            .attribute("style", props.style.clone())
            .onclick(Callback::from({
                let on_click = props.on_click.clone();
                move |event: MouseEvent| {
                    if let Some(on_click) = &on_click {
                        on_click.emit(event);
                    }
                }
            }))
            .into()
    }
}

impl Into<VNode> for Fab {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtFab>(Rc::new(self), key);
        VNode::from(comp)
    }
}
