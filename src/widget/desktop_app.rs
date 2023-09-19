use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::Router;

use gloo_history::{AnyHistory, HashHistory};

use crate::prelude::*;
use crate::props::{IntoOptionalTextRenderFn, TextRenderFn};
use crate::state::NavigationContainer;
use crate::widget::{CatalogLoader, ThemeLoader};

use pwt_macros::builder;

/// An application that uses material design gudelines.
///
/// This is just a convenient way to set up the following things:
///
/// - Provide a yew_router::HashRouter and [NavigationContainer]
/// - uses [ThemeLoader] to load the material design theme (dark/light)
/// - uses [CatalogLoader] to load the I18N tranlation catalog.
//
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct DesktopApp {
    body: VNode,

    /// The yew component key.
    pub key: Option<Key>,

    /// Basename passed to the [Router]
    #[builder(IntoPropValue, into_prop_value)]
    pub basename: Option<AttrValue>,

    /// History used for the [Router]
    #[builder(IntoPropValue, into_prop_value)]
    pub history: Option<AnyHistory>,

    /// Convert ISO 639-1 language code to server side catalog URLs (see [CatalogLoader]).
    #[builder_cb(IntoOptionalTextRenderFn, into_optional_text_render_fn, String)]
    pub catalog_url_builder: Option<TextRenderFn<String>>,
}

impl DesktopApp {
    /// Create a new instance.
    pub fn new(body: impl Into<VNode>) -> Self {
        yew::props!(Self { body: body.into() })
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }
}

#[doc(hidden)]
pub struct PwtDesktopApp {
    history: AnyHistory,
}

impl Component for PwtDesktopApp {
    type Message = ();
    type Properties = DesktopApp;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let history = props
            .history
            .clone()
            .unwrap_or(AnyHistory::from(HashHistory::new()));

        Self { history }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let body = ThemeLoader::new(props.body.clone());
        let body = CatalogLoader::new(body).url_builder(props.catalog_url_builder.clone());
        let body = NavigationContainer::new().with_child(body);
        html! {
            <Router history={self.history.clone()} basename={props.basename.clone()}>{body}</Router>
        }
    }
}

impl Into<VNode> for DesktopApp {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDesktopApp>(Rc::new(self), key);
        VNode::from(comp)
    }
}
