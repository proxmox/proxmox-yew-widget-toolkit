use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::widget::Row;

use pwt_macros::builder;

/// Material Design application bar.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct ApplicationBar {
    /// The yew component key.
    pub key: Option<Key>,

    /// Application title.
    #[builder(IntoPropValue, into_prop_value)]
    pub title: Option<AttrValue>,
}

impl ApplicationBar {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
}

pub struct PwtApplicationBar {}

impl Component for PwtApplicationBar {
    type Message = ();
    type Properties = ApplicationBar;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();

        Row::new()
            .attribute("style", "z-index: 1;") // make shadow visible
            .attribute("role", "banner")
            .attribute("aria-label", props.title.clone())
            .class("pwt-navbar")
            .class("pwt-justify-content-space-between pwt-align-items-center")
            .class("pwt-border-bottom")
            .class("pwt-shadow1")
            .padding(2)
            .with_child(html! {
                <span class="pwt-ps-1 pwt-font-headline-small">{props.title.clone()}</span>
            })
            .into()
    }
}

impl Into<VNode> for ApplicationBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtApplicationBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}
