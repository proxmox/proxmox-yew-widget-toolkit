use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::state::Theme;
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
    theme: Theme,
}

pub enum Msg {
    NextMode,
    SetTheme(Theme),
}

impl Component for PwtThemeSelector {
    type Message = Msg;
    type Properties = ThemeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            theme: Theme::System,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NextMode => {
                let theme = match self.theme {
                    Theme::System => Theme::Dark,
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::System,
                };
                return yew::Component::update(self, ctx, Msg::SetTheme(theme));
            }
            Msg::SetTheme(theme) => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                self.theme = theme;
                if let Err(err) = self.theme.store() {
                    log::error!("store theme failed: {err}");
                }

                if let Some(el) = document.get_element_by_id("__pwt-theme-loader__") {
                    let _ = el.set_attribute("href", self.theme.get_css_filename());
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onclick = ctx.link().callback(|_| Msg::NextMode);

        Button::new("")
            .class(props.class.clone())
            .onclick(onclick)
            .icon_class(match self.theme {
                Theme::System => "fa fa-fw fa-asterisk",
                Theme::Dark => "fa fa-fw fa-moon-o",
                Theme::Light => "fa fa-fw fa-sun-o",
            })
            .aria_label("Select Theme")
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let theme = Theme::load().unwrap_or_default();
            ctx.link().send_message(Msg::SetTheme(theme));
        }
    }
}

impl Into<VNode> for ThemeSelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeSelector>(Rc::new(self), None);
        VNode::from(comp)
    }
}
