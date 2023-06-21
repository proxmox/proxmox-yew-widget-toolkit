use std::collections::VecDeque;
use std::rc::Rc;


use gloo_timers::callback::Timeout;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{EventSubscriber, IntoOptionalKey, WidgetBuilder};
use crate::state::{SharedState, SharedStateObserver};

use pwt_macros::builder;

use super::SnackBar;

/// Messages sent from the [SnackBarController] to the [SnackBarManager].
pub enum SnackBarControllerMsg {
    Show(SnackBar),
    Dismiss(AttrValue),
    DismissCurrent,
    DismissAll,
}

/// Snackbar controller can show and dismiss snackbars.
///
/// Each [SnackBarManager]  provides a [SnackBarController] using a [yew::ContextProvider].
#[derive(Clone, PartialEq)]
pub struct SnackBarController {
    state: SharedState<Vec<SnackBarControllerMsg>>,
}

impl SnackBarController {
    pub fn new() -> Self {
        Self { state: SharedState::new(Vec::new()) }
    }

    /// Push a new snackbar to the display queue.
    ///
    /// Returns the snackbar ID, which can be used to dismiss specific items.
    pub fn show_snackbar(&self, mut snackbar: SnackBar) -> AttrValue {

        let id = match &snackbar.id {
            Some(id) => id.clone(),
            None => {
                let id = AttrValue::from(crate::widget::get_unique_element_id());
                snackbar.id = Some(id.clone());
                id
            }
        };

        self.state.write()
            .push(SnackBarControllerMsg::Show(snackbar));

        id
    }

    /// Dismiss a specific snackbar.
    pub fn dismiss(&self, id: AttrValue) {
        self.state.write()
            .push(SnackBarControllerMsg::Dismiss(id));
    }

    /// Dismiss currently shown snackbar.
    pub fn dismiss_current(&self) {
        self.state.write()
            .push(SnackBarControllerMsg::DismissCurrent);
    }

    /// Dismiss all queued snackbars.
    pub fn dismiss_all(&self) {
        self.state.write()
            .push(SnackBarControllerMsg::DismissAll);
    }
}

/// Display snackbars one after another.
///
/// This widget can be used to serialize the display of [SnackBar]s.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct SnackBarManager {
    /// The yew component key.
    pub key: Option<Key>,

    /// Optional Scaffold Controller
    #[builder(IntoPropValue, into_prop_value)]
    pub controller: Option<SnackBarController>,

    #[builder(IntoPropValue, into_prop_value)]
    pub bottom_offset: Option<u32>,
}

impl SnackBarManager {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }
}

pub enum Msg {
    ActionButtonPress,
    CloseButtonPress,
    Controller, // Controller has new messages
    AnimationEnd(AnimationEvent),
    Timeout,
}

#[derive(Clone, PartialEq)]
enum ViewState {
    Idle,
    FadeIn(SnackBar),
    Visible(SnackBar),
    FadeOut(SnackBar),
}

#[doc(hidden)]
pub struct PwtSnackBarManager {
    controller: SnackBarController,
    _state_observer: SharedStateObserver<Vec<SnackBarControllerMsg>>,

    queue: VecDeque<SnackBar>,
    view_state: ViewState, // current visible snackbar
    timeout: Option<Timeout>,
}

impl PwtSnackBarManager {
    fn display_next(&mut self, ctx: &Context<Self>) {
        if self.view_state != ViewState::Idle { return; }

        let next_snackbar = self.queue.pop_front();

        self.view_state = match next_snackbar {
            None => ViewState::Idle,
            Some(snackbar) => {
                if snackbar.is_dismissive() {
                    let duration = snackbar.duration.unwrap_or(4000).max(1000);
                    self.timeout = Some(Timeout::new(duration, {
                      let link = ctx.link().clone();
                        move || link.send_message(Msg::Timeout)
                    }));
                }
                ViewState::FadeIn(snackbar)
            }
        };
    }

    fn dismiss_current(&mut self, opt_id: Option<&AttrValue>) {
        match &self.view_state {
            ViewState::Idle | ViewState::FadeOut(_) => { /* do nothing */ }
            ViewState::FadeIn(snackbar) | ViewState::Visible(snackbar) => {
                if opt_id.is_none() || snackbar.id.as_ref() == opt_id {
                    self.view_state = ViewState::FadeOut(snackbar.clone());
                }
            }
        };
    }

    fn handle_controller_messages(&mut self, ctx: &Context<Self>) {
        let count = self.controller.state.read().len();
        if count == 0 { return; } // Note: avoid endless loop

        let list = self.controller.state.write().split_off(0);

        for msg in list.into_iter() {
            match msg {
                SnackBarControllerMsg::Show(snackbar) => {
                    self.queue.push_back(snackbar);
                }
                SnackBarControllerMsg::Dismiss(id) => {
                    self.queue.retain(|s| s.id.as_ref() != Some(&id));
                    self.dismiss_current(Some(&id));
                }
                SnackBarControllerMsg::DismissAll => {
                    self.queue.clear();
                    self.dismiss_current(None);
                }
                SnackBarControllerMsg::DismissCurrent => {
                    self.dismiss_current(None);
                }
            }
        }
        self.display_next(ctx);
    }
}

impl Component for PwtSnackBarManager {
    type Message = Msg;
    type Properties = SnackBarManager;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let controller = props
            .controller
            .clone()
            .unwrap_or(SnackBarController::new());

        let _state_observer = controller
            .state
            .add_listener(ctx.link().callback(|_| Msg::Controller));

        let mut me = Self {
            controller,
            _state_observer,
            queue: VecDeque::new(),
            view_state: ViewState::Idle,
            timeout: None,
        };

        me.handle_controller_messages(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
           Msg::ActionButtonPress => {
                self.timeout = None;

                let snackbar = match &self.view_state {
                    ViewState::Idle | ViewState::FadeOut(_) => return false,
                    ViewState::FadeIn(snackbar) => snackbar,
                    ViewState::Visible(snackbar) => snackbar,
                };

                if let Some(on_action) = &snackbar.on_action {
                    on_action.emit(())
                }
                self.view_state = ViewState::FadeOut(snackbar.clone());

                true
            }
            Msg::CloseButtonPress => {
                self.timeout = None;

                let snackbar = match &self.view_state {
                    ViewState::Idle | ViewState::FadeOut(_) => return false,
                    ViewState::FadeIn(snackbar) => snackbar,
                    ViewState::Visible(snackbar) => snackbar,
                };

                if let Some(on_close) = &snackbar.on_close {
                    on_close.emit(())
                }
                self.view_state = ViewState::FadeOut(snackbar.clone());

                true
            }
            Msg::Controller  => {
                self.handle_controller_messages(ctx);
                true
            },
            Msg::AnimationEnd(_event) => {
                match &self.view_state {
                    ViewState::Idle | ViewState::Visible(_) => false,
                    ViewState::FadeIn(snackbar) => {
                        self.view_state = ViewState::Visible(snackbar.clone());
                        true
                    }
                    ViewState::FadeOut(_snackbar) => {
                        self.view_state = ViewState::Idle;
                        self.display_next(ctx);
                        true
                    }
                }
            }
            Msg::Timeout => {
                self.timeout = None;
                match &self.view_state {
                    ViewState::Idle | ViewState::FadeOut(_) => false,
                    ViewState::FadeIn(snackbar) | ViewState::Visible(snackbar)=> {
                        self.view_state = ViewState::FadeOut(snackbar.clone());
                        true
                    }
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let snackbar = match &self.view_state {
            ViewState::Idle => return html! {},
            ViewState::FadeIn(snackbar) => snackbar.clone().class("fade-in"),
            ViewState::FadeOut(snackbar) => snackbar.clone().class("fade-out"),
            ViewState::Visible(snackbar) => snackbar.clone().class("visible"),
        };
        let snackbar = snackbar
            .attribute("style", props.bottom_offset.map(|offset| format!("bottom: {offset}px;")))
            .on_action(ctx.link().callback(|_| Msg::ActionButtonPress))
            .on_close(ctx.link().callback(|_| Msg::CloseButtonPress))
            .onanimationend(ctx.link().callback(Msg::AnimationEnd));


        html! {
            <ContextProvider<SnackBarController> context={self.controller.clone()}>
                {snackbar}
            </ContextProvider<SnackBarController>>
        }
    }
}

impl Into<VNode> for SnackBarManager {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtSnackBarManager>(Rc::new(self), key);
        VNode::from(comp)
    }
}
