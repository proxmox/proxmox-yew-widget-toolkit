use std::rc::Rc;

use wasm_bindgen::JsValue;
use serde_json::json;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::Container;

use super::Menu;

#[derive(Clone, PartialEq, Properties)]
pub struct MenuItem {
    pub text: AttrValue,
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

    /// Submenu close event
    pub on_close: Option<Callback<()>>,
}

impl MenuItem {
    /// Create a new menu item.
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
        self.menu = menu.into_prop_value();
        self
    }

    pub fn on_close(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_close = cb.into_event_callback();
        self
    }

    pub fn has_menu(&self) -> bool {
        self.menu.is_some()
    }

}

pub enum Msg {
}

#[doc(hidden)]
pub struct PwtMenuItem {
    content_ref: NodeRef,
    submenu_ref: NodeRef,
    popper: Option<JsValue>,
}

impl Component for PwtMenuItem {
    type Message = Msg;
    type Properties = MenuItem;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            content_ref: NodeRef::default(),
            submenu_ref: NodeRef::default(),
            popper: None,
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
                        .autofocus(props.focus_submenu)
                        .on_close(props.on_close.clone())
                }))
                .into();

            submenu = Some(sub);
        }

        let icon = props.icon_class.as_ref().map(|icon_class| {
            let icon_class = classes!(
                icon_class.clone(),
                "pwt-menu-item-icon",
            );
            html!{<i role="none" aria-hidden="true" class={icon_class}/>}
        });

        let arrow = props.menu.is_some().then(|| {
            let arrow_class = classes!(
                "fa",
                "fa-caret-right",
                "pwt-menu-item-arrow",
            );
            html!{<i role="none" aria-hidden="true" class={arrow_class}/>}
        });

        Container::new()
            .node_ref(self.content_ref.clone())
            .class("pwt-menu-item")
            .attribute("tabindex", (!props.disabled).then(|| "-1"))
            .attribute("disabled", props.disabled.then(|| ""))
            .with_child({
                let class = if props.menu.is_some() {
                    "pwt-menu-submenu-indent"
                } else {
                    "pwt-menu-item-indent"
                };
                html!{<i {class}>{&props.text}</i>}
            })
            .with_optional_child(icon)
            .with_optional_child(arrow)
            .with_optional_child(submenu)
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let props = ctx.props();
        if props.menu.is_none() { return; }

        if first_render {
            let opts = json!({
                "placement": "right-start",
                "strategy": "fixed",
                "modifiers": [
                    {
                        "name": "preventOverflow",
                        "options": {
                            "mainAxis": true, // true by default
                            "altAxis": true, // false by default
                        },
                    },
                    {
                        "name": "flip",
                        "options": {
                            "fallbackPlacements": ["bottom"],
                        },
                    },
                ],
            });

            let opts = crate::to_js_value(&opts).unwrap();

            if let Some(content_node) = self.content_ref.get() {
                if let Some(submenu_node) = self.submenu_ref.get() {
                    self.popper = Some(crate::create_popper(content_node, submenu_node, &opts));
                }
            }
        }

        if let Some(popper) = &self.popper {
            crate::update_popper(popper);
        }
    }
}

impl Into<VNode> for MenuItem {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuItem>(Rc::new(self), None);
        VNode::from(comp)
    }
}
