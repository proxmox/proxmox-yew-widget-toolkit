use yew::html::{IntoEventCallback, IntoPropValue};

use yew::virtual_dom::{Key, VNode};

use yew::prelude::*;

use crate::props::IntoOptionalKey;

use pwt_macros::builder;

/// TabBar item.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct TabBarItem {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Icon (CSS class).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub icon_class: Option<AttrValue>,

    /// Active Icon (CSS class).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub active_icon_class: Option<AttrValue>,

    /// Optional button label.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub label: Option<AttrValue>,

    /// Optional tooltip.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub tip: Option<VNode>,

    /// Activation callback.
    ///
    /// Emitted when the button is tapped, clicked or activated by keyboard.
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_activate: Option<Callback<()>>,

    /// Sets disabled state
    #[builder]
    pub disabled: bool,
}

impl Default for TabBarItem {
    fn default() -> Self {
        Self::new()
    }
}

impl TabBarItem {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self { disabled: false })
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
    }
}
