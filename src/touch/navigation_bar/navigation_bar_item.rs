use yew::html::{IntoEventCallback, IntoPropValue};

use yew::virtual_dom::{VNode, Key};

use yew::prelude::*;

/// Navigation bar item.
#[derive(Properties, Clone, PartialEq)]
pub struct NavigationBarItem {
    /// The yew component key.
    pub key: Option<Key>,

    /// Icon (CSS class).
    pub icon_class: Option<Classes>,

    /// Active Icon (CSS class).
    pub active_icon_class: Option<Classes>,

    /// Optional button label.
    pub label: Option<AttrValue>,

    /// Optional tooltip.
    pub tip: Option<VNode>,

    pub on_activate: Option<Callback<()>>,
}

impl NavigationBarItem {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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

    /// Builder style method to set the button label.
    pub fn label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_label(label);
        self
    }

    /// Method to set the button label.
    pub fn set_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.label = label.into_prop_value();
    }

    /// Builder style method to set the icon class
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon_class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }

    /// Builder style method to set the active icon class
    pub fn active_icon_class(mut self, active_icon_class: impl Into<Classes>) -> Self {
        self.set_active_icon_class(active_icon_class);
        self
    }

    /// Method to set the active_icon_class.
    pub fn set_active_icon_class(&mut self, active_icon_class: impl Into<Classes>) {
        self.active_icon_class = Some(active_icon_class.into());
    }

    /// Builder style method to set the tooltip
    pub fn tooltip(mut self, tip: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_tip(tip);
        self
    }

    /// Method to set the tooltip
    pub fn set_tip(&mut self, tip: impl IntoPropValue<Option<VNode>>) {
        self.tip = tip.into_prop_value();
    }

    pub fn on_activate(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_activate = cb.into_event_callback();
        self
    }
}
