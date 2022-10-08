use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::Container;

use super::{Menu, MenuPopper};

#[derive(Clone, PartialEq, Properties)]
pub struct MenuItem {
    /// Menu text (html inline text)
    pub text: Html,
    /// Menu icon displayed on the left side.
    pub icon_class: Option<Classes>,
    /// Optional Submenu
    pub menu: Option<Menu>,

    #[prop_or_default]
    pub disabled: bool,

    #[prop_or_default]
    pub active: bool,

    #[prop_or_default]
    pub show_submenu: bool,

    #[prop_or_default]
    pub focus_submenu: bool,

    #[prop_or_default]
    pub(crate) inside_menubar: bool,

    /// Submenu close event
    pub on_close: Option<Callback<()>>,

    pub on_select: Option<Callback<()>>,

}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(text: impl Into<Html>) -> Self {
        yew::props!(Self {
            text: text.into()
        })
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

    pub fn active(mut self, active: bool) -> Self {
        self.set_active(active);
        self
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn show_submenu(mut self, show_submenu: bool) -> Self {
        self.set_show_submenu(show_submenu);
        self
    }

    pub fn set_show_submenu(&mut self, show_submenu: bool) {
        self.show_submenu = show_submenu;
    }

    pub fn focus_submenu(mut self, focus_submenu: bool) -> Self {
        self.set_focus_submenu(focus_submenu);
        self
    }

    pub fn set_focus_submenu(&mut self, focus_submenu: bool) {
        self.focus_submenu = focus_submenu;
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

    pub fn menu(mut self, menu: impl IntoPropValue<Option<Menu>>) -> Self {
        self.set_menu(menu);
        self
    }

    pub fn set_menu(&mut self, menu: impl IntoPropValue<Option<Menu>>) {
         self.menu = menu.into_prop_value();
    }

    pub fn on_close(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_close = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_select callback.
    pub fn on_select(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }

    pub(crate) fn inside_menubar(mut self, inside_menubar: bool) -> Self {
        self.inside_menubar = inside_menubar;
        self
    }

    pub fn has_menu(&self) -> bool {
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
        let popper = MenuPopper::new(content_ref.clone(), submenu_ref.clone(), props.inside_menubar);

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
                    on_select.emit(());
                    // fixme: close menu
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
                .node_ref(self.submenu_ref.clone())
                .class("pwt-submenu")
                .with_optional_child(show_submenu.then(|| {
                    menu.clone()
                        .menubar(false) // make sure its vertical
                        .autofocus(props.focus_submenu)
                        .on_close(props.on_close.clone())
                }))
                .into();

            submenu = Some(sub);
        }

        let icon = props.icon_class.as_ref().map(|icon_class| {
            let icon_class = classes!(
                icon_class.clone(),
                if props.inside_menubar { "pwt-menubar-item-icon" }  else { "pwt-menu-item-icon" },
            );
            html!{<i role="none" aria-hidden="true" class={icon_class}/>}
        });

        let arrow = props.menu.is_some().then(|| {
            let arrow_class = classes!(
                "fa",
                "fa-caret-right",
                if props.inside_menubar { "pwt-menubar-item-arrow" } else { "pwt-menu-item-arrow" },
            );
            html!{<i role="none" aria-hidden="true" class={arrow_class}/>}
        });

        Container::new()
            .node_ref(self.content_ref.clone())
            .class(if props.inside_menubar { "pwt-menubar-item" } else { "pwt-menu-item" })
            .attribute("tabindex", (!props.disabled).then(|| "-1"))
            .attribute("disabled", props.disabled.then(|| ""))
            .with_optional_child(icon)
            .with_child({
                let class = if props.menu.is_some() {
                    "pwt-menu-submenu-indent"
                } else {
                    "pwt-menu-item-indent"
                };
                html!{<span {class}>{props.text.clone()}</span>}
            })
            .with_optional_child(arrow)
            .with_optional_child(submenu)
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        32 => link.send_message(Msg::Select),
                        _ => {},
                    }
                }
            })
            .onclick(ctx.link().callback(|_| Msg::Select))
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let props = ctx.props();
        if props.menu.is_none() { return; }
        self.popper.update();
    }
}

impl Into<VNode> for MenuItem {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuItem>(Rc::new(self), None);
        VNode::from(comp)
    }
}
