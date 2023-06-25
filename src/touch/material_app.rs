use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;

use gloo_history::HistoryListener;
use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::Routable;
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
    LastPage,
}

/// Page controller can show and dismiss pages.
///
/// Each [MaterialApp]  provides a [PageController] using a [yew::ContextProvider].
#[derive(Clone, PartialEq)]
pub struct PageController {
    state: SharedState<Vec<PageControllerMsg>>,
}

impl PageController {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            state: SharedState::new(Vec::new()),
        }
    }

    /// Push an anonymous page on top of the page stack.
    pub fn push_page(&self, page: impl Into<VNode>) {
        self.state
            .write()
            .push(PageControllerMsg::Push(page.into()));
    }

    /// Pop one anonymous page from the page stack.
    ///
    /// This does nothing if there are no anonymous pages on the stack.
    /// You may want to use [Self::last_page] to implement the "Back"
    /// opertation, because that also use the browser history.
    pub fn pop_page(&self) {
        self.state.write().push(PageControllerMsg::Pop);
    }

    /// Nagigate to the last page.
    ///
    /// If there are anonymous pages on the stack, simply pop one.
    /// Else, use the browser history to navigate back.
    pub fn last_page(&self) {
        self.state.write().push(PageControllerMsg::LastPage);
    }
}

/// An application that uses material design gudelines.
///
/// This is just a convenient way to set up the following things:
///
/// - Provide a yew_router::HashRouter and [NavigationContainer]
/// - uses [ThemeLoader] to load the material design theme (dark/light)
/// - Provides a [SnackBarController], and display snackbars using [SnackBarManager]
/// - Uses [PageStack] to dislay/animate overlapping pages.
/// - Provides a [PageController] to navigate and control the [PageStack].
///
/// You need to provide a simple rendering function that translates [routes] into
/// HTML pages. More specifically, each route can reeturn a stack of pages.
/// Internally, this stack is passed to a [PageStack] widget that provides
/// animations when switching between pages.
///
/// First, one usually defines a [Routable] enum to express routes using
/// static rust types.
///
/// ```
/// use yew_router::prelude::*;
/// use pwt::prelude::*;
/// use pwt::touch::MaterialApp;
///
/// #[derive(Clone, Copy, PartialEq, Routable)]
/// enum Route {
///    #[at("/")]
///    Home,
///    #[at("/config")]
///    Config,
///    #[at("/config/network")]
///    Network,
/// }
///
/// // Then you define you render functions to map routes to a page stack.
///
/// fn switch(route: &Route) -> Vec<Html> {
///    match route {
///        Route::Home => vec![
///             html! {"Home"},
///        ],
///        Route::Config => vec![
///             html! {"Config"},
///        ],
///        Route::Network => vec![
///             html! {"Config"},
///             html! {"Network"},
///        ],
///    }
/// }
/// #[function_component]
/// fn YourApp() -> Html {
///     MaterialApp::new(switch)
///         .into()
/// }
///
/// ```
///
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct MaterialApp<R: Routable> {

    /// The yew component key.
    pub key: Option<Key>,

    /// Optional Scaffold Controller.
    #[builder(IntoPropValue, into_prop_value)]
    pub snackbar_controller: Option<SnackBarController>,

    /// Optional snackbar bottom offset.
    #[builder(IntoPropValue, into_prop_value)]
    pub snackbar_bottom_offset: Option<u32>,

    /// Page render function.
    pub render_route: PageRenderFn<R>,
}

impl<R: Routable + 'static> MaterialApp<R> {
    /// Create a new instance.
    ///
    /// The 'render' functions maps from [Routes] to html pages.
    pub fn new(render: impl Into<PageRenderFn<R>>) -> Self {
        yew::props!(Self { render_route: render.into()})
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }
}

pub enum Msg {
    PageController,
    HistoryChange,
}

#[doc(hidden)]
pub struct PwtMaterialApp<R> {
    snackbar_controller: SnackBarController,
    page_controller: PageController,
    _page_controller_observer: SharedStateObserver<Vec<PageControllerMsg>>,
    page_stack: Vec<Html>,
    history: AnyHistory,
    _history_listener: HistoryListener,
    phantom: PhantomData<R>,
}

impl<R: Routable + 'static> PwtMaterialApp<R> {
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
                PageControllerMsg::LastPage => {
                    if self.page_stack.is_empty() {
                        self.history.back();
                    } else {
                        self.page_stack.pop();
                    }
                }
            }
        }
    }
}

impl<R: Routable + 'static> Component for PwtMaterialApp<R> {
    type Message = Msg;
    type Properties = MaterialApp<R>;

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
            phantom: PhantomData,
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

        let location = self.history.location();
        let path = location.path();
        let route = R::recognize(path);

        let mut page_stack = Vec::new();

        if let Some(route) = route {
            page_stack.extend(props.render_route.apply(&route));
        }

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

impl<R: Routable + 'static> Into<VNode> for MaterialApp<R> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtMaterialApp<R>>(Rc::new(self), key);
        VNode::from(comp)
    }
}

/// A [PageRenderFn] function is a callback that transforms a [Route] into
/// a stack of [Html] pages.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct PageRenderFn<R: Routable>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&R) -> Vec<Html>>
);

impl<R: Routable> PageRenderFn<R> {
    /// Creates a new [`PageRenderFn`]
    pub fn new(renderer: impl Into<Self>) -> Self {
        renderer.into()
    }
    /// Apply the render function
    pub fn apply(&self, route: &R) -> Vec<Html> {
        (self.0)(route)
    }
}

impl<R: Routable, F: 'static + Fn(&R) -> Vec<Html>> From<F> for PageRenderFn<R> {
    fn from(renderer: F) -> Self {
        PageRenderFn(Rc::new(renderer))
    }
}
