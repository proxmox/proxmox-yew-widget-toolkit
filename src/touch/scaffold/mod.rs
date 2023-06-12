use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoPropValue;

use crate::widget::{Column, Container};
use crate::props::{ContainerBuilder, WidgetBuilder};

use super::NavigationBar;

/// Implements the basic Material Design visual layout structure.
#[derive(Properties, Clone, PartialEq)]
pub struct Scaffold {
    /// The yew component key.
    pub key: Option<Key>,

    /// The top application bar.
    pub application_bar: Option<VNode>,

    /// The primary content displayed below the application bar.
    pub body: Option<VNode>,

    /// The bottom navigation bar.
    pub navigation_bar: Option<NavigationBar>,

    /// Favorite action button.
    pub favorite_action_button: Option<VNode>,
}

impl Scaffold {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn application_bar(mut self, app_bar: impl Into<VNode>) -> Self {
        self.application_bar = Some(app_bar.into());
        self
    }

    pub fn body(mut self, body: impl Into<VNode>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn navigation_bar(mut self, nav_bar: impl IntoPropValue<Option<NavigationBar>>) -> Self {
        self.navigation_bar = nav_bar.into_prop_value();
        self
    }

    pub fn favorite_action_button(mut self, fav: impl Into<VNode>) -> Self {
        self.favorite_action_button = Some(fav.into());
        self
    }
}


#[doc(hidden)]
pub struct PwtScaffold {
}

impl Component for PwtScaffold {
    type Message = ();
    type Properties = Scaffold;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let positioned_fab = props
            .favorite_action_button
            .clone()
            .map(|fab| {
                Container::new()
                .class("pwt-position-absolute")
                .class("pwt-right-2 pwt-bottom-2")
                .with_child(fab)
            });

        let body = Column::new()
            .class("pwt-position-relative")
            .class("pwt-flex-fill")
            .with_optional_child(props.body.clone())
            .with_optional_child(positioned_fab);


        Column::new()
            .class("pwt-viewport")
            .with_optional_child(props.application_bar.clone())
            .with_child(body)
            .with_optional_child(props.navigation_bar.clone())
            .into()
    }

}

impl Into<VNode> for Scaffold {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtScaffold>(Rc::new(self), key);
        VNode::from(comp)
    }
}