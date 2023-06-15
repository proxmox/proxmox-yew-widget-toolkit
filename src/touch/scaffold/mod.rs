use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{ContainerBuilder, WidgetBuilder};
use crate::state::{SharedState, SharedStateObserver};
use crate::widget::{Column, Container};

use super::{NavigationBar, SideDialog};

use pwt_macros::builder;

#[derive(Copy, Clone)]
struct ScaffoldState {
    show_drawer: bool,
}

/// Scaffold controller.
#[derive(Clone, PartialEq)]
pub struct ScaffoldController {
    state: SharedState<ScaffoldState>,
}

impl ScaffoldController {
    /// Crteate a new instance.
    pub fn new() -> Self {
        Self {
            state: SharedState::new(ScaffoldState { show_drawer: false }),
        }
    }

    /// Show/hide the drawer.
    pub fn show_drawer(&self, show: bool) {
        let mut state = self.state.write();
        state.show_drawer = show;
    }

    /// Toggle between show( and hide.
    pub fn toggle_drawer(&self) {
        let mut state = self.state.write();
        state.show_drawer = !state.show_drawer;
    }
}

/// Implements the basic Material Design visual layout structure.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct Scaffold {
    /// The yew component key.
    pub key: Option<Key>,

    /// Optional Scaffold Controller
    #[builder(IntoPropValue, into_prop_value)]
    pub controller: Option<ScaffoldController>,

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
    controller: ScaffoldController,
    _state_observer: SharedStateObserver<ScaffoldState>,
}

pub enum Msg {
    Close,
    StateChange,
}

impl Component for PwtScaffold {
    type Message = Msg;
    type Properties = Scaffold;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let controller = props
            .controller
            .clone()
            .unwrap_or(ScaffoldController::new());

        let _state_observer = controller
            .state
            .add_listener(ctx.link().callback(|_| Msg::StateChange));

        Self {
            controller,
            _state_observer,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateChange => true,
            Msg::Close => {
                self.controller.show_drawer(false);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let positioned_fab = props.favorite_action_button.clone().map(|fab| {
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

        let show_drawer = self.controller.state.read().show_drawer;
        let drawer = match (show_drawer, props.drawer.clone()) {
            (true, Some(drawer)) => Some(
                SideDialog::new()
                .on_close(ctx.link().callback(|_| Msg::Close))
                //.with_child("TESTDIALOG")
                .with_child(drawer)
            ),
            _ => None,
        };

        /*
            .class("pwt-scaffold-drawer-mask")
            .class((show_drawer && props.drawer.is_some()).then(|| "visible"))
            .onclick({
                let controller = self.controller.clone();
                move |_| {
                    controller.show_drawer(false);
                }
            })

            .with_child(
                Container::new()
                    .class("pwt-scaffold-drawer")
                    .class(drawer_animation_class)
                    .onclick(|event: MouseEvent| event.stop_propagation())
                    .ontransitionend(ctx.link().callback(|_| Msg::DrawerAnimationEnd))
                    .with_optional_child(props.drawer.clone())
            );
        */

        Column::new()
            .class("pwt-viewport")
            .class("pwt-position-relative")
            .with_optional_child(props.application_bar.clone())
            .with_child(body)
            .with_optional_child(props.navigation_bar.clone())
            .with_optional_child(drawer)
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
