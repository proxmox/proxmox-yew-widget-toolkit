use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::Button;

#[derive(Clone, PartialEq, Properties)]
pub struct ThemeSelector {
    #[prop_or_default]
    class: Classes,
}

impl ThemeSelector {

    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }
}

pub struct PwtThemeSelector {
    dark: bool,
}

pub enum Msg {
    ToggleMode,
}

impl Component for PwtThemeSelector {
    type Message = Msg;
    type Properties = ThemeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            dark: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleMode => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                self.dark = !self.dark;
                crate::store_use_dark_theme(self.dark);

                if let Some(el) = document.get_element_by_id("__pwt-theme-loader__") {
                    if self.dark {
                        let _ = el.set_attribute("href", "proxmox-yew-style-dark.css");
                    } else {
                        let _ = el.set_attribute("href", "proxmox-yew-style-light.css");
                    }
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onclick = ctx.link().callback(|_| Msg::ToggleMode);

        Button::new("")
            .class(props.class.clone())
            .onclick(onclick)
            .icon_class(if self.dark { "fa fa-sun-o" } else { "fa fa-moon-o" })
            .aria_label("Select Theme")
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {

            if let Some(dark) = crate::load_use_dark_theme() {
                if self.dark != dark {
                    ctx.link().send_message(Msg::ToggleMode);
                }
            } else {
                let window = web_sys::window().unwrap();
                if let Ok(Some(list)) = window.match_media("(prefers-color-scheme: dark)") {
                    let dark = list.matches();
                    if self.dark != dark {
                        ctx.link().send_message(Msg::ToggleMode);
                    }
                }
            }
        }
    }
}

impl Into<VNode> for ThemeSelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeSelector>(Rc::new(self), NodeRef::default(), None);
        VNode::from(comp)
    }
}
