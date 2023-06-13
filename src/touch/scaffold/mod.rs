use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoPropValue;

use crate::widget::{Column, Container};
use crate::props::{ContainerBuilder, WidgetBuilder};

use super::NavigationBar;

use pwt_macros::builder;

/// Implements the basic Material Design visual layout structure.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct Scaffold {
    /// The yew component key.
    pub key: Option<Key>,

    /// The top application bar.
    pub application_bar: Option<VNode>,

    /// The primary content displayed below the application bar.
    pub body: Option<VNode>,

    /// The bottom navigation bar.
    #[builder(IntoPropValue, into_prop_value)]
    pub navigation_bar: Option<NavigationBar>,

    /// Favorite action button.
    pub favorite_action_button: Option<VNode>,

    /// A panel displayed to the left side of the body.
    pub drawer: Option<VNode>,
}

impl Scaffold {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the application bar.
    pub fn application_bar(mut self, app_bar: impl Into<VNode>) -> Self {
        self.application_bar = Some(app_bar.into());
        self
    }

    /// Builder style method to set the body.
    pub fn body(mut self, body: impl Into<VNode>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Builder style method to set the favorite action button.
    pub fn favorite_action_button(mut self, fav: impl Into<VNode>) -> Self {
        self.favorite_action_button = Some(fav.into());
        self
    }

    /// Builder style method to set the drawer.
    pub fn drawer(mut self, drawer: impl Into<VNode>) -> Self {
        self.drawer = Some(drawer.into());
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

        let drawer = Container::new()
            .attribute("style", "z-index:2;min-width:200px;top:0;left:0;bottom:0;")
            .class("pwt-position-absolute")
            .class("pwt-bg-color-neutral")
            .with_optional_child(props.drawer.clone());

        Column::new()
            .class("pwt-viewport")
            .class("pwt-position-relative")
            .with_optional_child(props.application_bar.clone())
            .with_child(body)
            .with_optional_child(props.navigation_bar.clone())
            .with_child(drawer)
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