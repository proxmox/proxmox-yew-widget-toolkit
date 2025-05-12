use std::rc::Rc;

use yew::virtual_dom::{Key, VComp, VNode};

use crate::state::{Theme, ThemeMode};
use crate::widget::Button;
use crate::{impl_class_prop_builder, impl_yew_std_props_builder, prelude::*};

/// Round icon button to select light/dark theme.
#[derive(Clone, PartialEq, Properties)]
pub struct ThemeModeSelector {
    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// Yew `key` property
    #[prop_or_default]
    pub key: Option<Key>,

    /// CSS class
    #[prop_or_default]
    pub class: Classes,
}

impl Default for ThemeModeSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeModeSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_yew_std_props_builder!();
    impl_class_prop_builder!();
}

#[doc(hidden)]
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

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NextMode => {
                let theme = match self.theme {
                    ThemeMode::System => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::System,
                };
                yew::Component::update(self, _ctx, Msg::SetThemeMode(theme))
            }
            Msg::SetThemeMode(theme) => {
                if let Err(err) = Theme::store_theme_mode(theme) {
                    log::error!("store theme failed: {err}");
                }
                self.theme = theme;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onclick = ctx.link().callback(|_| Msg::NextMode);

        Button::new_icon(match self.theme {
            ThemeMode::System => "fa fa-fw fa-asterisk",
            ThemeMode::Dark => "fa fa-fw fa-moon-o",
            ThemeMode::Light => "fa fa-fw fa-sun-o",
        })
        .node_ref(props.node_ref.clone())
        .class(props.class.clone())
        .class("circle")
        .onclick(onclick)
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

impl From<ThemeModeSelector> for VNode {
    fn from(val: ThemeModeSelector) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtThemeModeSelector>(Rc::new(val), key);
        VNode::from(comp)
    }
}
