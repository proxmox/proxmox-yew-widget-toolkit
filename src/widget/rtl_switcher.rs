use pwt_macros::widget;
use yew::{
    Classes, Component, Properties,
};

use crate::prelude::EventSubscriber;

use super::form::Checkbox;

#[widget(pwt=crate, comp=PwtRtlSwitcher, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct RtlSwitcher {
    #[prop_or_default]
    class: Classes,
}

impl RtlSwitcher {
    /// Creates a new [`RtlSwitcher`].
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

impl Default for RtlSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PwtRtlSwitcher {
    rtl: bool,
}

pub enum Msg {
    ToggleRtl,
}

impl Component for PwtRtlSwitcher {
    type Message = Msg;
    type Properties = RtlSwitcher;

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleRtl => {
                let document = web_sys::window().unwrap().document().unwrap();
                let elements = document.get_elements_by_tag_name("html");
                if let Some(html) = elements.get_with_index(0) {
                    if self.rtl {
                        if let Err(err) = html.remove_attribute("dir") {
                            log::error!("could not remove dir attribute: {:?}", err);
                            return false;
                        }
                        self.rtl = false;
                    } else {
                        if let Err(err) = html.set_attribute("dir", "rtl") {
                            log::error!("could not set dir attribute: {:?}", err);
                            return false;
                        }
                        self.rtl = true;
                    }
                }
            }
        }
        true
    }

    fn create(_ctx: &yew::Context<Self>) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let elements = document.get_elements_by_tag_name("html");
        let rtl = elements
            .get_with_index(0)
            .and_then(|html| html.get_attribute("dir"))
            .map(|dir| &dir == "rtl")
            .unwrap_or(false);

        Self { rtl }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let onclick = ctx.link().callback(|_| Msg::ToggleRtl);
        Checkbox::new().checked(self.rtl).on_change(onclick).into()
    }
}
