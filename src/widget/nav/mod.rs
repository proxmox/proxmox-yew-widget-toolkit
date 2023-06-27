mod navigation_drawer;
pub use navigation_drawer::{NavigationDrawer, PwtNavigationDrawer};
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VNode};

use pwt_macros::builder;

use crate::props::IntoOptionalKey;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct MenuItem {
    /// The yew component key.
    ///
    /// This key is used to uniquely identify entries. Items without
    /// keys are not selectable.
    key: Option<Key>,

    /// The label (text).
    label: AttrValue,

    /// Menu icon displayed on the left side.
    #[builder(IntoPropValue, into_prop_value)]
    icon_class: Option<AttrValue>,

    /// Optional submenu.
    #[builder(IntoPropValue, into_prop_value)]
    pub submenu: Option<Menu>,

    /// Selectable flag.
    #[prop_or(true)]
    #[builder]
    pub selectable: bool,

    /// Activation callback.
    ///
    /// Emitted when the item is tapped, clicked or activated by keyboard.
    ///
    /// # Note
    ///
    /// This is not emitted when the active item is changed using the selection.
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_activate: Option<Callback<()>>,

}

impl MenuItem {
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
pub enum MenuEntry {
    Item(MenuItem),
    Component(VNode),
}

impl From<MenuItem> for MenuEntry  {
    fn from(item: MenuItem) -> Self {
        Self::Item(item)
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Menu {
    #[prop_or_default]
    pub children: Vec<MenuEntry>,
}

impl Menu {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn with_item(mut self, item: impl Into<MenuEntry>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<MenuEntry>) {
        self.children.push(item.into());
    }

    pub fn with_component(mut self, component: impl Into<VNode>) -> Self {
        self.add_component(component);
        self
    }

    pub fn add_component(&mut self, component: impl Into<VNode>) {
        self.children
            .push(MenuEntry::Component(component.into()))
    }
}