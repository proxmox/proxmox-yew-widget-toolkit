use std::rc::Rc;

use wasm_bindgen::JsCast;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::{Button, Container};

use super::{Menu, MenuPopper};

#[derive(Clone, PartialEq, Properties)]
pub struct MenuButton {
    pub text: AttrValue,
    pub icon_class: Option<Classes>,
    /// Optional Submenu
    pub menu: Option<Menu>,

    #[prop_or_default]
    pub disabled: bool,
}

impl MenuButton {

    /// Create a new menu button
    pub fn new(text: impl Into<AttrValue>) -> Self {
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
}

pub enum Msg {
    CloseMenu,
    ToggleMenu,
}

#[doc(hidden)]
pub struct PwtMenuButton {
    content_ref: NodeRef,
    submenu_ref: NodeRef,
    popper: MenuPopper,

    show_submenu: bool,
}

impl PwtMenuButton {

    fn restore_focus(&mut self) {
        if let Some(node) = self.content_ref.get() {
            if let Some(el) = node.dyn_into::<web_sys::HtmlElement>().ok() {
                let _ = el.focus();
            }
        }
    }
}

impl Component for PwtMenuButton {
    type Message = Msg;
    type Properties = MenuButton;

    fn create(_ctx: &Context<Self>) -> Self {
        let content_ref = NodeRef::default();
        let submenu_ref = NodeRef::default();
        let popper = MenuPopper::new(content_ref.clone(), submenu_ref.clone(), true);
        Self {
            content_ref,
            submenu_ref,
            popper,
            show_submenu: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CloseMenu => {
                self.show_submenu = false;
                self.restore_focus();
                true
            }
            Msg::ToggleMenu =>  {
                self.show_submenu = !self.show_submenu;
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

        Container::new()
            .attribute("style", "z-index: 1;")
            .with_child(
                Button::new(&props.text)
                    .node_ref(self.content_ref.clone())
                    .onclick(ctx.link().callback(|_| Msg::ToggleMenu))
            )
            .with_optional_child(submenu)
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let props = ctx.props();
        if props.menu.is_none() { return; }
        self.popper.update();
    }
}

impl Into<VNode> for MenuButton {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuButton>(Rc::new(self), None);
        VNode::from(comp)
    }
}
