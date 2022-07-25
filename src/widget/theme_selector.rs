use std::rc::Rc;

use wasm_bindgen::{prelude::*};
use wasm_bindgen::JsCast;

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
    // keep it alive
    observer_closure: Closure::<dyn Fn()>,
}

pub enum Msg {
    ToggleMode,
    SchemeChanged,
}

impl Component for PwtThemeSelector {
    type Message = Msg;
    type Properties = ThemeSelector;

    fn create(ctx: &Context<Self>) -> Self {
        let onchange = Closure::wrap({
            let link = ctx.link().clone();
            Box::new(move || {
                link.send_message(Msg::SchemeChanged);
            }) as Box<dyn Fn()>
        });

        Self {
            dark: false,
            observer_closure: onchange,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SchemeChanged => {
                //log::info!("prefers-color-scheme changes");
                let window = web_sys::window().unwrap();
                if let Ok(Some(list)) = window.match_media("(prefers-color-scheme: dark)") {
                    let dark = list.matches();
                    if self.dark != dark {
                        return yew::Component::update(self, ctx, Msg::ToggleMode);
                    }
                }
                false
            }
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
                    list.set_onchange(Some(self.observer_closure.as_ref().unchecked_ref()));
                }
            }
        }
    }
}

impl Into<VNode> for ThemeSelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeSelector>(Rc::new(self), None);
        VNode::from(comp)
    }
}
