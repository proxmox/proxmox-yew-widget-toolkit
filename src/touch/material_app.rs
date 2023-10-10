use std::borrow::Cow;
use std::rc::Rc;

use derivative::Derivative;

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

use super::{PageStack, SideDialog, SideDialogController, SideDialogLocation};

// Messages sent from the [PageController].
pub enum PageControllerMsg {
    Dialog(SideDialog), // Show modal dialog
    CloseDialog,
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

    /// Show a modal dialog on top of the page stack.
    ///
    /// Used to show drawers and bottom sheets.
    pub fn show_side_dialog(&self, dialog: impl Into<SideDialog>) {
        self.state
            .write()
            .push(PageControllerMsg::Dialog(dialog.into()));
    }

    /// Close/dismiss the dialog on top of the page stack (if any).
    pub fn close_side_dialog(&self) {
        self.state.write().push(PageControllerMsg::CloseDialog);
    }

    /// Show the drawer on the left side of the body.
    ///
    /// This is just a convenient wrapper for [Self::show_side_dialog].
    pub fn show_drawer(&self, drawer: impl Into<VNode>) {
        self.show_side_dialog(
            SideDialog::new()
                .direction(SideDialogLocation::Left)
                .with_child(drawer.into()),
        );
    }

    /// Show the drawer on the right side of the body.
    ///
    /// This is just a convenient wrapper for [Self::show_side_dialog].
    pub fn show_end_drawer(&self, drawer: impl Into<VNode>) {
        self.show_side_dialog(
            SideDialog::new()
                .direction(SideDialogLocation::Right)
                .with_child(drawer.into()),
        );
    }

    /// Show a modal bottom sheet.
    ///
    /// This is just a convenient wrapper for [Self::show_side_dialog].
    pub fn show_modal_bottom_sheet(&self, bottom_sheet: impl Into<VNode>) {
        self.show_side_dialog(
            SideDialog::new()
                .direction(SideDialogLocation::Bottom)
                .with_child(bottom_sheet.into()),
        );
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
/// You need to provide a simple rendering function that translates routes into HTML pages.
/// More specifically, each route can reeturn a stack of pages.
/// Internally, this stack is passed to a [PageStack] widget that provides
/// animations when switching between pages.
///
/// First, one usually defines a routable enum to express routes using
/// static rust types. Then you define you render functions to map the
/// routes to a page stack.
///
/// ```
/// use yew_router::prelude::*;
/// use pwt::prelude::*;
/// use pwt::touch::{MaterialApp, Scaffold};
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
/// fn switch(path: &str) -> Vec<Html> {
///    let route = Route::recognize(&path).unwrap();
///    match route {
///        Route::Home => vec![
///             Scaffold::with_title("Home").into(),
///        ],
///        Route::Config => vec![
///             Scaffold::with_title("Config").into(),
///        ],
///        Route::Network => vec![
///             Scaffold::with_title("Config").into(),
///             Scaffold::with_title("Network").into(),
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
pub struct MaterialApp {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Optional Scaffold Controller.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub snackbar_controller: Option<SnackBarController>,

    /// Optional snackbar bottom offset.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub snackbar_bottom_offset: Option<u32>,

    /// Page render function.
    pub render_route: PageRenderFn,

    /// Basename passed to the [Router]
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub basename: Option<AttrValue>,

    /// History used for the [Router]
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub history: Option<AnyHistory>,
}

impl MaterialApp {
    /// Create a new instance.
    ///
    /// The 'render' functions maps from routes to html pages.
    pub fn new(render: impl Into<PageRenderFn>) -> Self {
        yew::props!(Self {
            render_route: render.into()
        })
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
    CloseDialog,
}

#[doc(hidden)]
pub struct PwtMaterialApp {
    snackbar_controller: SnackBarController,
    page_controller: PageController,
    _page_controller_observer: SharedStateObserver<Vec<PageControllerMsg>>,
    page_stack: Vec<Html>,
    dialog: Option<(SideDialogController, Html)>,
    history: AnyHistory,
    _history_listener: HistoryListener,
}

impl PwtMaterialApp {
    fn handle_page_controller_messages(&mut self, ctx: &Context<Self>) {
        let count = self.page_controller.state.read().len();
        if count == 0 {
            return;
        } // Note: avoid endless loop

        let list = self.page_controller.state.write().split_off(0);

        for msg in list.into_iter() {
            match msg {
                PageControllerMsg::Dialog(dialog) => {
                    let controller = dialog
                        .controller
                        .clone()
                        .unwrap_or(SideDialogController::new());
                    let on_close = dialog.on_close.clone();
                    let on_close = ctx.link().callback(move |_| {
                        if let Some(on_close) = &on_close {
                            on_close.emit(());
                        }
                        Msg::CloseDialog
                    });
                    self.dialog = Some((
                        controller.clone(),
                        dialog.controller(controller).on_close(on_close).into(),
                    ));
                }
                PageControllerMsg::CloseDialog => {
                    if let Some((controller, _)) = &self.dialog {
                        controller.close_dialog();
                    }
                }
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

impl Component for PwtMaterialApp {
    type Message = Msg;
    type Properties = MaterialApp;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        static THEMES: &[&str] = &["Material"];
        crate::state::set_available_themes(THEMES);

        let history = props
            .history
            .clone()
            .unwrap_or(AnyHistory::from(HashHistory::new()));

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
            dialog: None,
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
                //log::info!("HISTORY CHANGE");
                self.page_stack.clear();
                if let Some((controller, _)) = &self.dialog {
                    controller.close_dialog();
                }
                true
            }
            Msg::CloseDialog => {
                self.dialog = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let basename: Option<AttrValue> = None;
        let location = self.history.location();
        let path = strip_basename(&basename, location.path().into());

        let mut page_stack = Vec::new();

        page_stack.extend(props.render_route.apply(&path));

        page_stack.extend(self.page_stack.clone());

        let app = Container::new()
            .class("pwt-viewport")
            .with_child(PageStack::new(page_stack))
            .with_child(
                SnackBarManager::new()
                    .controller(self.snackbar_controller.clone())
                    .bottom_offset(props.snackbar_bottom_offset),
            )
            .with_optional_child(self.dialog.as_ref().map(|(_, dialog)| dialog.clone()));

        html! {
            <Router history={self.history.clone()} basename={props.basename.clone()}>
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

/// A [PageRenderFn] function is a callback that transforms a path into
/// a stack of [Html] pages.
///
/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct PageRenderFn(
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))] Rc<dyn Fn(&str) -> Vec<Html>>,
);

impl PageRenderFn {
    /// Creates a new [`PageRenderFn`]
    pub fn new(renderer: impl Into<Self>) -> Self {
        renderer.into()
    }
    /// Apply the render function
    pub fn apply(&self, path: &str) -> Vec<Html> {
        (self.0)(path)
    }
}

impl<F: 'static + Fn(&str) -> Vec<Html>> From<F> for PageRenderFn {
    fn from(renderer: F) -> Self {
        PageRenderFn(Rc::new(renderer))
    }
}

// copied from yew_router::Router, because its not acessible from outside
fn strip_basename<'a>(basename: &Option<AttrValue>, path: Cow<'a, str>) -> Cow<'a, str> {
    match basename.as_deref() {
        Some(m) => {
            let mut path = path
                .strip_prefix(m)
                .map(|m| Cow::from(m.to_owned()))
                .unwrap_or(path);

            if !path.starts_with('/') {
                path = format!("/{m}").into();
            }

            path
        }
        None => path,
    }
}
