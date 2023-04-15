mod navigation_drawer;
pub use navigation_drawer::{NavigationDrawer, PwtNavigationDrawer};

mod navigation_panel;
pub use navigation_panel::{NavigationPanel, PwtNavigationPanel};

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VNode};

use pwt_macros::builder;

use crate::props::IntoOptionalKey;
use crate::state::Selection;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct NavMenuItem {
    /// The yew component key.
    ///
    /// This key is used to uniquely identify entries. Items without
    /// keys are not selectable.
    key: Option<Key>,

    /// The label (text).
    label: AttrValue,

    /// Menu icon displayed on the left side.
    #[builder]
    icon_class: Option<AttrValue>,

    /// Optional submenu.
    #[builder]
    pub submenu: Option<NavMenu>,

    /// Activation callback.
    ///
    /// Emitted when the item is tapped, clicked or activated by keyboard.
    #[builder_cb]
    pub on_activate: Option<Callback<()>>,
}

impl NavMenuItem {
    /// Create a new instance.
    pub fn new(label: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            label: label.into()
        })
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

#[derive(Clone, PartialEq)]
pub enum NavMenuEntry {
    Item(NavMenuItem),
    Component(VNode),
}

#[derive(Clone, PartialEq, Properties)]
pub struct NavMenu {
    #[prop_or_default]
    children: Vec<NavMenuEntry>,
}

impl NavMenu {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn with_item(mut self, item: impl Into<NavMenuEntry>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<NavMenuEntry>) {
        self.children.push(item.into());
    }

    pub fn with_component(mut self, component: impl Into<VNode>) -> Self {
        self.add_component(component);
        self
    }

    pub fn add_component(&mut self, component: impl Into<VNode>) {
        self.children
            .push(NavMenuEntry::Component(component.into()))
    }
}