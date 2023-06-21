use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::HashRouter;

use pwt_macros::builder;

use crate::prelude::*;
use crate::touch::{SnackBarController, SnackBarManager};
use crate::widget::{Container, ThemeLoader};

/// An application that uses material design gudelines.
///
/// This is just a convenient way to set up the following things:
///
/// - Provide a yew_router::HashRouter;
/// - uses [ThemeLoader] to load the material design theme (dark/light)
/// - Provides a [SnackBarController], an d display snackbars using [SnackBarManager]
///
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct MaterialApp {
    /// The yew component key.
    pub key: Option<Key>,

    /// The home page ("/")
    pub home: Option<VNode>,

    /// Optional Scaffold Controller.
    #[builder(IntoPropValue, into_prop_value)]
    pub snackbar_controller: Option<SnackBarController>,

    #[builder(IntoPropValue, into_prop_value)]
    pub snackbar_bottom_offset: Option<u32>,
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
pub struct PwtMaterialApp {
    snackbar_controller: SnackBarController,
}

impl Component for PwtMaterialApp {
    type Message = ();
    type Properties = MaterialApp;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        static THEMES: &'static [&'static str] = &["Material"];
        crate::state::set_available_themes(THEMES);

        let snackbar_controller = props
            .snackbar_controller
            .clone()
            .unwrap_or(SnackBarController::new());

        Self {
            snackbar_controller,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let app = Container::new()
            .class("pwt-viewport")
            .with_child(props.home.clone().unwrap_or(Container::new().into()))
            .with_child(
                SnackBarManager::new()
                    .controller(self.snackbar_controller.clone())
                    .bottom_offset(props.snackbar_bottom_offset)
            );

        html! {
            <HashRouter>
                <ContextProvider<SnackBarController> context={self.snackbar_controller.clone()}>
                {ThemeLoader::new(app)}
                </ContextProvider<SnackBarController>>
            </HashRouter>
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
