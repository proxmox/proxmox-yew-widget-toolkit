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

    /// Leading widget placed before the title.
    pub leading: Option<Html>,

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

    /// Builder style method to set the leading widget.
    pub fn leading(mut self, leading: impl Into<VNode>) -> Self {
        self.set_leading(leading);
        self
    }

    /// Method to set the leading widget.
    pub fn set_leading(&mut self, leading: impl Into<VNode>) {
        self.leading = Some(leading.into());
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

        let actions = html!{"actions"};

        Row::new()
            .attribute("style", "z-index: 1;") // make shadow visible
            .attribute("role", "banner")
            .attribute("aria-label", props.title.clone())
            .class("pwt-navbar")
            .class("pwt-justify-content-space-between pwt-align-items-center")
            .class("pwt-border-bottom")
            .class("pwt-shadow1")
            .padding(2)
            .gap(2)
            .with_optional_child(props.leading.clone())
            .with_child(html! {
                <span class="pwt-font-headline-small pwt-text-truncate">{props.title.clone()}</span>
            })
            .with_child(actions)
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
