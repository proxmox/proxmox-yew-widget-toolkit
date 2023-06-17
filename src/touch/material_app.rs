use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::HashRouter;

use crate::prelude::IntoOptionalKey;
use crate::widget::{Container, ThemeLoader};

/// An application that uses material design gudelines.
///
/// This is just a convenient wrapper which set up a few thing:
///
/// - Provides a yew_router::HashRouter;
/// - uses [ThemeLoader] to load the material design theme (dark/light)
///
#[derive(Properties, Clone, PartialEq)]
pub struct MaterialApp {
    /// The yew component key.
    pub key: Option<Key>,

    /// The home page ("/")
    pub home: Option<VNode>,
}

impl MaterialApp {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }

    /// Builder style method to set the home page.
    pub fn home(mut self, home: impl Into<VNode>) -> Self {
        self.home = Some(home.into());
        self
    }
}

#[doc(hidden)]
pub struct PwtMaterialApp {}

impl Component for PwtMaterialApp {
    type Message = ();
    type Properties = MaterialApp;

    fn create(ctx: &Context<Self>) -> Self {
        static THEMES: &'static [&'static str] = &["Material"];
        crate::state::set_available_themes(THEMES);

        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let app: Html= match &props.home {
            None => Container::new().into(),
            Some(home) => home.clone().into(),
        };

        html! {
            <HashRouter>{
                ThemeLoader::new(app)
            }</HashRouter>
        }
    }
}

impl Into<VNode> for MaterialApp {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtMaterialApp>(Rc::new(self), key);
        VNode::from(comp)
    }
}
