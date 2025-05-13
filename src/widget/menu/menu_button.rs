use wasm_bindgen::JsCast;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::dom::focus::FocusTracker;
use crate::prelude::*;
use crate::props::{BuilderFn, IntoOptionalBuilderFn};
use crate::widget::{Button, Container};

use super::{Menu, MenuControllerMsg, MenuPopper};

use pwt_macros::{builder, widget};

/// A Button that opens a [Menu].
///
/// See <https://www.w3.org/WAI/ARIA/apg/patterns/menu/>.
#[widget(pwt=crate, comp=PwtMenuButton, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct MenuButton {
    pub text: AttrValue,
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

    /// Automatically popup menu when receiving focus
    ///
    /// You can then open the menu programmatically by giving the
    /// button focus.
    #[prop_or_default]
    #[builder]
    pub autoshow_menu: bool,

    #[prop_or_default]
    #[builder]
    pub disabled: bool,

    /// Whether to show an arrow at the end of the menu.
    #[prop_or_default]
    #[builder]
    pub show_arrow: bool,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tabindex: Option<i32>,

    // Fires on menu close
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_close: Option<Callback<()>>,
}

impl MenuButton {
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
    ShowMenu,
    CloseMenu,
    FocusChange(bool),
}

#[doc(hidden)]
pub struct PwtMenuButton {
    submenu_ref: NodeRef,
    popper: MenuPopper,
    menu_controller: Callback<MenuControllerMsg>,
    show_submenu: bool,
    focus_tracker: FocusTracker,
}

impl PwtMenuButton {
    fn restore_focus(&mut self, props: &MenuButton) {
        if let Some(node) = props.std_props.node_ref.get() {
            if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
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

        let menu_controller = {
            let link = ctx.link().clone();
            Callback::from(move |msg: MenuControllerMsg| {
                match msg {
                    MenuControllerMsg::Next => { /* ignore */ }
                    MenuControllerMsg::Previous => { /* ignore */ }
                    MenuControllerMsg::Collapse => link.send_message(Msg::CloseMenu),
                }
            })
        };

        let focus_tracker = FocusTracker::new(ctx.link().callback(Msg::FocusChange));

        Self {
            submenu_ref,
            popper,
            menu_controller,
            show_submenu: false,
            focus_tracker,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.popper.update_refs(
            ctx.props().std_props.node_ref.clone(),
            self.submenu_ref.clone(),
        );

        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
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
                .node_ref(self.submenu_ref.clone())
                .class("pwt-submenu")
                .with_optional_child(menu)
        });

        let mut button = Button::new(&props.text)
            .show_arrow(props.show_arrow)
            .disabled(props.disabled)
            .attribute("aria-haspopup", "true")
            .attribute("aria-expanded", self.show_submenu.then_some("true"))
            .tabindex(props.tabindex)
            .icon_class(props.icon_class.clone());

        button.std_props = props.std_props.clone();
        button.listeners = props.listeners.clone();

        let show_submenu = self.show_submenu;
        let autoshow_menu = props.autoshow_menu;
        let button = button.onclick(ctx.link().callback(move |event: MouseEvent| {
            event.stop_propagation();
            if show_submenu && !autoshow_menu {
                Msg::CloseMenu
            } else {
                Msg::ShowMenu
            }
        }));

        Container::new()
            .style("display", "contents")
            .attribute("role", "none")
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
            .with_optional_child(submenu)
            .into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.show_submenu {
            self.popper.update();
        }
    }
}
