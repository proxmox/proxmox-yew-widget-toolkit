use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VNode};

use pwt_macros::builder;

use crate::props::{IntoOptionalKey, WidgetBuilder};
use crate::state::Selection;
use crate::widget::SelectionView;

use super::{NavigationDrawer};

/// Navigation Panel.
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct NavigationPanel {
    #[prop_or_default]
    #[builder]
    pub node_ref: NodeRef,

    /// The yew component key.
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    view: SelectionView,

}

impl NavigationPanel {
    /// Create a new instance.
    pub fn new() -> Self {
        // fixme: move selection to state
        let view = SelectionView::new().class("pwt-fit");
        yew::props!(Self {
            view,
        })
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

    /// Builder style method to add a html class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }
}

#[doc(hidden)]
pub struct PwtNavigationPanel {
    selection: Selection,
}

impl Component for PwtNavigationPanel {
    type Message = ();
    type Properties = NavigationPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        todo!()
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        todo!()
    }
}