use wasm_bindgen::JsCast;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::dom::focus::FocusTracker;
use crate::prelude::*;
use crate::props::{BuilderFn, IntoOptionalBuilderFn};
use crate::state::SharedStateObserver;
use crate::widget::menu::MenuController;
use crate::widget::{Button, Container};

use super::{Menu, MenuControllerMsg, MenuPopper};

use pwt_macros::{builder, widget};

/// Split Button
///
/// A button with an extra trigger that opens a [Menu].
///
/// A SplitButton is like a MenuButton, but with an added default action.
#[widget(pwt=crate, comp=PwtSplitButton, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct SplitButton {
    /// Button text.
    pub text: AttrValue,

    /// Icon (CSS class).
    #[prop_or_default]
    pub icon_class: Option<Classes>,

    /// Optional Submenu
    #[prop_or_default]
    pub menu: Option<Menu>,

    /// Menu Builder
    ///
    /// To create menus dynamically. If specified, the 'menu' property
    /// is ignored.
    #[prop_or_default]
    pub menu_builder: Option<BuilderFn<Menu>>,

    #[prop_or_default]
    #[builder]
    pub disabled: bool,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tabindex: Option<i32>,

    // Fires on menu close
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_close: Option<Callback<()>>,

    /// Activate callback
    #[builder_cb(IntoEventCallback, into_event_callback, MouseEvent)]
    #[prop_or_default]
    pub on_activate: Option<Callback<MouseEvent>>,

    #[builder_cb(IntoPropValue, into_prop_value, Option<MenuController>)]
    #[prop_or_default]
    pub(crate) menu_controller: Option<MenuController>,
}

impl SplitButton {
    /// Create a new menu button
    pub fn new(text: impl Into<AttrValue>) -> Self {
        yew::props!(Self { text: text.into() })
    }

    /// Builder style method to set the icon class.
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }

    /// Builder style method to set the menu.
    pub fn menu(mut self, menu: impl IntoPropValue<Option<Menu>>) -> Self {
        self.menu = menu.into_prop_value();
        self
    }

    /// Builder style method to set the menu builder.
    pub fn menu_builder(mut self, builder: impl IntoOptionalBuilderFn<Menu>) -> Self {
        self.menu_builder = builder.into_optional_builder_fn();
        self
    }
}

pub enum Msg {
    Activate(MouseEvent),
    ShowMenu,
    CloseMenu,
    FocusChange(bool),
    Controller, // MenuController has new messages
}

#[doc(hidden)]
pub struct PwtSplitButton {
    node_ref: NodeRef,
    submenu_ref: NodeRef,
    trigger_ref: NodeRef,
    popper: MenuPopper,
    menu_controller: MenuController,
    _menu_controller_observer: SharedStateObserver<Vec<MenuControllerMsg>>,
    show_submenu: bool,
    focus_tracker: FocusTracker,
}

impl PwtSplitButton {
    fn restore_focus(&mut self, _props: &SplitButton) {
        if let Some(node) = self.node_ref.get() {
            if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
        }
    }

    fn handle_controller_messages(&mut self, ctx: &Context<Self>) {
        let count = self.menu_controller.state.read().len();
        if count == 0 {
            return;
        } // Note: avoid endless loop

        let list = self.menu_controller.state.write().split_off(0);

        for msg in list.into_iter() {
            match msg {
                MenuControllerMsg::Open => {
                    ctx.link().send_message(Msg::ShowMenu);
                }
                MenuControllerMsg::Collapse => {
                    ctx.link().send_message(Msg::CloseMenu);
                }
                MenuControllerMsg::Next => { /* ignore */ }
                MenuControllerMsg::Previous => { /* ignore  */ }
            }
        }
    }
}

impl Component for PwtSplitButton {
    type Message = Msg;
    type Properties = SplitButton;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let node_ref = NodeRef::default();
        let submenu_ref = NodeRef::default();
        let trigger_ref = NodeRef::default();

        let popper = MenuPopper::new(trigger_ref.clone(), submenu_ref.clone(), true);

        let menu_controller = props
            .menu_controller
            .clone()
            .unwrap_or_else(|| MenuController::new());

        let _menu_controller_observer = menu_controller
            .state
            .add_listener(ctx.link().callback(|_| Msg::Controller));

        let focus_tracker = FocusTracker::new(ctx.link().callback(Msg::FocusChange));

        let mut me = Self {
            node_ref,
            submenu_ref,
            trigger_ref,
            popper,
            menu_controller,
            _menu_controller_observer,
            show_submenu: false,
            focus_tracker,
        };

        me.handle_controller_messages(ctx);
        me
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.popper
            .update_refs(self.trigger_ref.clone(), self.submenu_ref.clone());

        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Controller => {
                self.handle_controller_messages(ctx);
                true
            }
            Msg::ShowMenu => {
                self.show_submenu = true;
                true
            }
            Msg::CloseMenu => {
                self.show_submenu = false;
                self.restore_focus(props);
                if let Some(on_close) = &props.on_close {
                    on_close.emit(());
                }
                true
            }
            Msg::FocusChange(has_focus) => {
                if !has_focus {
                    self.show_submenu = false;
                }
                true
            }
            Msg::Activate(event) => {
                yew::Component::update(self, ctx, Msg::CloseMenu);
                if let Some(on_activate) = &props.on_activate {
                    on_activate.emit(event);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let submenu = self.show_submenu.then(|| {
            let menu = if let Some(menu_builder) = &props.menu_builder {
                Some(
                    menu_builder
                        .apply()
                        .autofocus(true)
                        .menu_controller(self.menu_controller.clone())
                        .on_close(ctx.link().callback(|_| Msg::CloseMenu)),
                )
            } else {
                props.menu.as_ref().map(|m| {
                    m.clone()
                        .autofocus(true)
                        .menu_controller(self.menu_controller.clone())
                        .on_close(ctx.link().callback(|_| Msg::CloseMenu))
                })
            };

            Container::new()
                .attribute("role", "none")
                .class("pwt-submenu")
                .with_optional_child(menu)
                .into_html_with_ref(self.submenu_ref.clone())
        });

        let mut button = Button::new(&props.text)
            .disabled(props.disabled)
            .tabindex(props.tabindex)
            .icon_class(props.icon_class.clone())
            .on_activate(ctx.link().callback(Msg::Activate));

        button.std_props = props.std_props.clone();
        button.listeners = props.listeners.clone();

        let show_submenu = self.show_submenu;
        let trigger = Button::new_icon("fa fa-chevron-down")
            .node_ref(self.trigger_ref.clone())
            .attribute("aria-haspopup", "true")
            .attribute("aria-expanded", self.show_submenu.then_some("true"))
            .disabled(props.disabled)
            .onclick(ctx.link().callback(move |event: MouseEvent| {
                event.stop_propagation();
                if show_submenu {
                    Msg::CloseMenu
                } else {
                    Msg::ShowMenu
                }
            }));

        Container::new()
            .attribute("role", "none")
            .with_std_props(&props.std_props)
            .class("pwt-segmented-button")
            .onfocusin(self.focus_tracker.get_focus_callback(true))
            .onfocusout(self.focus_tracker.get_focus_callback(false))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    match event.key().as_str() {
                        "Escape" => link.send_message(Msg::CloseMenu),
                        "ArrowDown" => link.send_message(Msg::ShowMenu),
                        _ => return,
                    }
                    event.stop_propagation();
                    event.prevent_default();
                }
            })
            .with_child(button)
            .with_child(trigger)
            .with_optional_child(submenu)
            .into_html_with_ref(self.node_ref.clone())
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.show_submenu {
            self.popper.update();
        }
    }
}
