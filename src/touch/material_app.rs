use std::rc::Rc;

use gloo_history::HistoryListener;
use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::history::{AnyHistory, HashHistory};
use yew_router::{history::History, Router};

use pwt_macros::builder;

use crate::prelude::*;
use crate::state::{NavigationContainer, SharedState, SharedStateObserver};
use crate::touch::{SnackBarController, SnackBarManager};
use crate::widget::{Container, ThemeLoader};

use super::PageStack;

// Messages sent from the [PageController].
pub enum PageControllerMsg {
    Push(VNode),
    Pop,
}

/// Page controller can show and dismiss pages.
///
/// Each [MaterialApp]  provides a [PageController] using a [yew::ContextProvider].
#[derive(Clone, PartialEq)]
pub struct PageController {
    state: SharedState<Vec<PageControllerMsg>>,
}

impl PageController {
    pub fn new() -> Self {
        Self {
            state: SharedState::new(Vec::new()),
        }
    }

    pub fn push_page(&self, page: impl Into<VNode>) {
        self.state
            .write()
            .push(PageControllerMsg::Push(page.into()));
    }

    pub fn pop_page(&self) {
        self.state.write().push(PageControllerMsg::Pop);
    }
}

/// An application that uses material design gudelines.
///
/// This is just a convenient way to set up the following things:
///
/// - Provide a yew_router::HashRouter and [NavigationContainer]
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

pub enum Msg {
    PageController,
    HistoryChange,
}

#[doc(hidden)]
pub struct PwtMaterialApp {
    snackbar_controller: SnackBarController,
    page_controller: PageController,
    _page_controller_observer: SharedStateObserver<Vec<PageControllerMsg>>,
    page_stack: Vec<Html>,
    history: AnyHistory,
    _history_listener: HistoryListener,
}

impl PwtMaterialApp {
    fn handle_page_controller_messages(&mut self, _ctx: &Context<Self>) {
        let count = self.page_controller.state.read().len();
        if count == 0 {
            return;
        } // Note: avoid endless loop

        let list = self.page_controller.state.write().split_off(0);

        for msg in list.into_iter() {
            match msg {
                PageControllerMsg::Push(page) => {
                    //self.history.push("/testhistory");
                    self.page_stack.push(page);
                }
                PageControllerMsg::Pop => {
                    self.page_stack.pop();
                }
            }
        }
    }
}
impl Component for PwtMaterialApp {
    type Message = Msg;
    type Properties = MaterialApp;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        static THEMES: &'static [&'static str] = &["Material"];
        crate::state::set_available_themes(THEMES);

        let history = AnyHistory::from(HashHistory::new());

        let snackbar_controller = props
            .snackbar_controller
            .clone()
            .unwrap_or(SnackBarController::new());

        let page_controller = PageController::new();

        let _page_controller_observer = page_controller
            .state
            .add_listener(ctx.link().callback(|_| Msg::PageController));

        let _history_listener = history.listen({
            let link = ctx.link().clone();
            move || link.send_message(Msg::HistoryChange)
        });

        let page_stack = Vec::new();

        let mut me = Self {
            snackbar_controller,
            page_controller,
            _page_controller_observer,
            page_stack,
            history,
            _history_listener,
        };

        me.handle_page_controller_messages(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PageController => {
                self.handle_page_controller_messages(ctx);
                true
            }
            Msg::HistoryChange => {
                log::info!("HISTORY CHANGE");
                self.page_stack.clear(); // fixme: only remove anonymous pages
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut page_stack = Vec::new();
        page_stack.push(props.home.clone().unwrap_or(Container::new().into()));
        page_stack.extend(self.page_stack.clone());

        let app = Container::new()
            .class("pwt-viewport")
            .with_child(PageStack::new(page_stack))
            .with_child(
                SnackBarManager::new()
                    .controller(self.snackbar_controller.clone())
                    .bottom_offset(props.snackbar_bottom_offset),
            );

        html! {
            <Router history={self.history.clone()}>
                <ContextProvider<SnackBarController> context={self.snackbar_controller.clone()}>
                    <ContextProvider<PageController> context={self.page_controller.clone()}>
                    { ThemeLoader::new(NavigationContainer::new().with_child(app))}
                    </ContextProvider<PageController>>
                </ContextProvider<SnackBarController>>
            </Router>
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
