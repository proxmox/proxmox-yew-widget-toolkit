use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VNode};

use pwt_macros::builder;

use crate::props::IntoOptionalKey;
use crate::state::Selection;

use super::{NavMenu, NavMenuEntry, NavMenuItem};

/// Navigation Menu Widget.
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct NavigationDrawer {
    #[prop_or_default]
    #[builder]
    node_ref: NodeRef,

    /// The yew component key.
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    menu: NavMenu,

    /// Selection object to store the currently selected tab key.
    ///
    /// The optional selction object allows you to control and observer the state from outside.
    #[builder(IntoPropValue, into_prop_value)]
    pub selection: Option<Selection>,

    /// Selection callback.
    #[builder_cb(IntoEventCallback, into_event_callback, Option<Key>)]
    pub on_select: Option<Callback<Option<Key>>>,

    pub default_active: Option<Key>,

    /// Enable router functionality.
    ///
    /// Save/Load state from parent NavigationContainer
    #[builder]
    #[prop_or_default]
    router: bool,
}

impl NavigationDrawer {
    /// Create a new instance.
    pub fn new(menu: NavMenu) -> Self {
        yew::props!(Self { menu })
    }

    // Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property.
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
    }

    // Builder style method to set `default_active` property.
    pub fn default_active(mut self, default_active: impl IntoOptionalKey) -> Self {
        self.set_default_active(default_active);
        self
    }

    /// Method to set the yew `default_active` property.
    pub fn set_default_active(&mut self, default_active: impl IntoOptionalKey) {
        self.default_active = default_active.into_optional_key();
    }

    /// Builder style method to add a html class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    fn get_default_active(&self) -> Option<Key> {
        if self.default_active.is_some() {
            return self.default_active.clone();
        }

        for item in &self.menu.children {
            if let NavMenuEntry::Item(NavMenuItem { key: Some(key), .. }) = item {
                return Some(key.clone());
            }
        }

        None
    }
}

#[doc(hidden)]
pub struct PwtNavigationDrawer {

}

impl Component for PwtNavigationDrawer {
    type Message = ();
    type Properties = NavigationDrawer;

    fn create(_ctx: &Context<Self>) -> Self {
        todo!()
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        todo!()
    }
}