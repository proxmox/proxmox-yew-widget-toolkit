use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::state::{Theme, ThemeMode};
use crate::widget::Button;

#[derive(Clone, PartialEq, Properties)]
pub struct ThemeModeSelector {
    #[prop_or_default]
    class: Classes,
}

impl ThemeModeSelector {
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

pub struct PwtThemeModeSelector {
    theme: ThemeMode,
}

pub enum Msg {
    NextMode,
    SetThemeMode(ThemeMode),
}

impl Component for PwtThemeModeSelector {
    type Message = Msg;
    type Properties = ThemeModeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            theme: ThemeMode::System,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NextMode => {
                let theme = match self.theme {
                    ThemeMode::System => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::System,
                };
                return yew::Component::update(self, ctx, Msg::SetThemeMode(theme));
            }
            Msg::SetThemeMode(theme) => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                if let Err(err) = Theme::store_theme_mode(theme) {
                    log::error!("store theme failed: {err}");
                }
                self.theme = theme;
                let event = web_sys::Event::new("pwt-theme-changed").unwrap();
                let _ = document.dispatch_event(&event);

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
                ThemeMode::System => "fa fa-fw fa-asterisk",
                ThemeMode::Dark => "fa fa-fw fa-moon-o",
                ThemeMode::Light => "fa fa-fw fa-sun-o",
            })
            .aria_label("Select Theme Mode")
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let theme = Theme::load();
            ctx.link().send_message(Msg::SetThemeMode(theme.mode));
        }
    }
}

impl Into<VNode> for ThemeModeSelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeModeSelector>(Rc::new(self), None);
        VNode::from(comp)
    }
}
