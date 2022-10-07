use wasm_bindgen::JsCast;
use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::{Button, Container};

use super::{Menu, MenuPopper};

use pwt_macros::widget;

#[widget(PwtMenuButton, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct MenuButton {
    pub text: AttrValue,
    pub icon_class: Option<Classes>,
    /// Optional Submenu
    pub menu: Option<Menu>,

    /// Automatically popup menu when receiving focus
    ///
    /// You can then open the menu programmatically by giving the
    /// button focus.
    #[prop_or_default]
    pub autoshow_menu: bool,

    #[prop_or_default]
    pub disabled: bool,

    pub tabindex: Option<i32>,

    // Fires on menu close
    pub on_close: Option<Callback<()>>,
}

impl MenuButton {

    /// Create a new menu button
    pub fn new(text: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            text: text.into()
        })
    }

    /// Builder style method to set the autoshow_menu flag
    pub fn autoshow_menu(mut self, autoshow_menu: bool) -> Self {
        self.set_autoshow_menu(autoshow_menu);
        self
    }

    /// Method to set the autoshow_menu flag
    pub fn set_autoshow_menu(&mut self, autoshow_menu: bool) {
        self.autoshow_menu = autoshow_menu;
    }

    /// Builder style method to set the disabled flag
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Method to set the disabled flag
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Builder style method to set the html tabindex attribute
    pub fn tabindex(mut self, index: impl IntoPropValue<Option<i32>>) -> Self {
        self.set_tabindex(index);
        self
    }

    /// Method to set the html tabindex attribute
    pub fn set_tabindex(&mut self, index: impl IntoPropValue<Option<i32>>) {
        self.tabindex = index.into_prop_value();
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

    /// Builder style method to set the on_close callback
    pub fn on_close(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_close = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    ShowMenu,
    CloseMenu,
    FocusChange(bool),
    DelayedFocusChange(bool),
}

#[doc(hidden)]
pub struct PwtMenuButton {
    submenu_ref: NodeRef,
    popper: MenuPopper,

    show_submenu: bool,
    timeout: Option<Timeout>,

    last_has_focus: bool,
}

impl PwtMenuButton {

    fn restore_focus(&mut self, props: &MenuButton) {
        if let Some(node) = props.std_props.node_ref.get() {
            if let Some(el) = node.dyn_into::<web_sys::HtmlElement>().ok() {
                let _ = el.focus();
            }
        }
    }
}

impl Component for PwtMenuButton {
    type Message = Msg;
    type Properties = MenuButton;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let submenu_ref = NodeRef::default();
        let popper = MenuPopper::new(props.std_props.node_ref.clone(), submenu_ref.clone(), true);
        Self {
            submenu_ref,
            popper,
            show_submenu: false,
            timeout: None,
            last_has_focus: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ShowMenu =>  {
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
                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(1, move || {
                    link.send_message(Msg::DelayedFocusChange(has_focus));
                }));
                false
            }
            Msg::DelayedFocusChange(has_focus) => {
                if has_focus == self.last_has_focus { return false; }
                self.last_has_focus = has_focus;

                if has_focus {
                    if props.autoshow_menu {
                        self.show_submenu = true;
                    }
                } else {
                    self.show_submenu = false;
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let show_submenu = self.show_submenu;

        let mut submenu: Option<Html> = None;
        if let Some(menu) = &props.menu {
            let sub = Container::new()
                .node_ref(self.submenu_ref.clone())
                .class("pwt-submenu")
                .with_optional_child(show_submenu.then(|| {
                    menu.clone()
                        .autofocus(true)
                        .on_close(ctx.link().callback(|_| Msg::CloseMenu))
                }))
                .into();

            submenu = Some(sub);
        }

        let mut button = Button::new(&props.text)
            .tabindex(props.tabindex)
            .icon_class(props.icon_class.clone());

        button.std_props = props.std_props.clone();
        button.listeners = props.listeners.clone();

        let button = button.onclick(ctx.link().callback(move |event: MouseEvent| {
            event.stop_propagation();
            Msg::ShowMenu
        }));


        Container::new()
            .attribute("style", "padding: inherit;")
            .onfocusin(ctx.link().callback(|_| Msg::FocusChange(true)))
            .onfocusout(ctx.link().callback(|_| Msg::FocusChange(false)))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        27 => link.send_message(Msg::CloseMenu),
                        40 => link.send_message(Msg::ShowMenu),
                       _ => return,
                    }
                    event.stop_propagation();
                    event.prevent_default();
                }
            })
            .with_child(button)
            .with_optional_child(submenu)
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let props = ctx.props();
        if props.menu.is_none() { return; }
        self.popper.update();
    }
}
