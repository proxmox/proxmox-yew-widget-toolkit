use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::Container;

use pwt_macros::builder;

use super::{Menu, MenuController, MenuEvent, MenuPopper};

/// Menu item widget with optional icon and optional submenu.
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct MenuItem {
    /// Menu text (html inline text).
    ///
    /// Please set the `focusable` flag it the html contains focusable
    /// items.
    pub text: Html,

    /// Menu icon displayed on the left side.
    #[prop_or_default]
    pub icon_class: Option<Classes>,

    /// Optional submenu.
    #[prop_or_default]
    pub menu: Option<Menu>,

    /// Disabled flag.
    #[prop_or_default]
    #[builder]
    pub disabled: bool,

    /// Indicates that the `text` contains a focusable element.
    ///
    /// If set, the menu item does not add `tabindex: -1` to the
    /// container. This avoids having nested focusable elements inside
    /// a single menu item.
    #[prop_or_default]
    #[builder]
    pub focusable: bool,

    #[prop_or_default]
    #[builder]
    pub(crate) active: bool,

    #[prop_or_default]
    #[builder]
    pub(crate) show_submenu: bool,

    #[prop_or_default]
    #[builder]
    pub(crate) focus_submenu: bool,

    #[prop_or_default]
    #[builder]
    pub(crate) inside_menubar: bool,

    /// Submenu close event
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub(crate) on_close: Option<Callback<()>>,

    #[builder_cb(IntoPropValue, into_prop_value, Option<MenuController>)]
    #[prop_or_default]
    pub(crate) menu_controller: Option<MenuController>,

    /// Select callback.
    ///
    /// Emited when the user activates the entry.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, MenuEvent)]
    pub on_select: Option<Callback<MenuEvent>>,
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(text: impl Into<Html>) -> Self {
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

    /// Builder style method to set the submenu.
    pub fn menu(mut self, menu: impl IntoPropValue<Option<Menu>>) -> Self {
        self.set_menu(menu);
        self
    }

    /// Method to set the submenu.
    pub fn set_menu(&mut self, menu: impl IntoPropValue<Option<Menu>>) {
        self.menu = menu.into_prop_value();
    }

    // Methods below are used internally.

    pub(crate) fn has_menu(&self) -> bool {
        self.menu.is_some()
    }
}

pub enum Msg {
    Select,
}

#[doc(hidden)]
pub struct PwtMenuItem {
    content_ref: NodeRef,
    submenu_ref: NodeRef,
    popper: MenuPopper,
}

impl Component for PwtMenuItem {
    type Message = Msg;
    type Properties = MenuItem;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let content_ref = NodeRef::default();
        let submenu_ref = NodeRef::default();
        let popper = MenuPopper::new(
            content_ref.clone(),
            submenu_ref.clone(),
            props.inside_menubar,
        );

        Self {
            content_ref,
            submenu_ref,
            popper,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::Select => {
                if let Some(on_select) = &props.on_select {
                    let event = MenuEvent::new();
                    on_select.emit(event.clone());
                    if !event.get_keep_open() {
                        if let Some(menu_controller) = &props.menu_controller {
                            menu_controller.collapse();
                        }
                    }
                } else {
                    // Always close menus without on_select callback
                    if let Some(menu_controller) = &props.menu_controller {
                        menu_controller.collapse();
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let show_submenu = props.active && props.show_submenu;

        let mut submenu: Option<Html> = None;
        if let Some(menu) = &props.menu {
            let sub = Container::new()
                .class("pwt-submenu")
                .attribute("role", "none")
                .with_optional_child(show_submenu.then(|| {
                    menu.clone()
                        .menubar(false) // make sure its vertical
                        .menubar_child(props.inside_menubar)
                        .menu_controller(props.menu_controller.clone())
                        .autofocus(props.focus_submenu)
                        .on_close(props.on_close.clone())
                }))
                .into_html_with_ref(self.submenu_ref.clone());

            submenu = Some(sub);
        }

        let icon = props
            .icon_class
            .as_ref()
            .filter(|c| !c.is_empty())
            .map(|icon_class| {
                let widget_class = if props.inside_menubar {
                    "pwt-menubar-item-icon"
                } else {
                    "pwt-menu-item-icon"
                };
                let icon_class = classes!(icon_class.clone(), widget_class);
                html! {<i role="none" class={icon_class}/>}
            });

        let has_submenu = props.menu.is_some();

        let arrow = has_submenu.then(|| {
            let arrow_class = classes!(
                "fa",
                "fa-caret-right",
                if props.inside_menubar {
                    "pwt-menubar-item-arrow"
                } else {
                    "pwt-menu-item-arrow"
                },
            );
            html! {<i role="none" class={arrow_class}/>}
        });

        let disabled = props.disabled;
        Container::new()
            .class(if props.inside_menubar {
                "pwt-menubar-item"
            } else {
                "pwt-menu-item"
            })
            .attribute("tabindex", (!props.focusable).then_some("-1"))
            .attribute("aria-disabled", props.disabled.then_some("true"))
            .attribute("role", "menuitem")
            .attribute("aria-haspopup", has_submenu.then_some("true"))
            .attribute(
                "aria-expanded",
                has_submenu.then_some(if show_submenu { "true" } else { "false" }),
            )
            .with_optional_child(icon)
            .with_child(html! {<span class="pwt-flex-fill">{props.text.clone()}</span>})
            .with_optional_child(arrow)
            .with_optional_child(submenu)
            .onkeydown((!disabled).then_some({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| match event.key().as_str() {
                    "Enter" | " " => {
                        if !has_submenu {
                            event.stop_propagation();
                            event.prevent_default();
                            link.send_message(Msg::Select)
                        }
                    }
                    _ => {}
                }
            }))
            .onclick((!disabled).then_some({
                let link = ctx.link().clone();
                move |_| {
                    if !has_submenu {
                        link.send_message(Msg::Select)
                    };
                }
            }))
            .into_html_with_ref(self.content_ref.clone())
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let props = ctx.props();
        if props.menu.is_none() {
            return;
        }
        if props.active && props.show_submenu {
            self.popper.update();
        }
    }
}

impl From<MenuItem> for VNode {
    fn from(val: MenuItem) -> Self {
        let comp = VComp::new::<PwtMenuItem>(Rc::new(val), None);
        VNode::from(comp)
    }
}
