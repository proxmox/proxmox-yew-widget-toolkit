use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::state::Theme;
use crate::widget::form::Combobox;
use crate::{impl_class_prop_builder, impl_yew_std_props_builder};

#[derive(Clone, PartialEq, Properties)]
pub struct ThemeNameSelector {
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

impl Default for ThemeNameSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeNameSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_yew_std_props_builder!();
    impl_class_prop_builder!();
}

pub struct PwtThemeNameSelector {
    theme: String,
    available_themes: Rc<Vec<AttrValue>>,
}

pub enum Msg {
    SetThemeName(String),
}

impl Component for PwtThemeNameSelector {
    type Message = Msg;
    type Properties = ThemeNameSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        let theme = Theme::load();
        let available_themes: Vec<AttrValue> = crate::state::get_available_themes()
            .iter()
            .map(|name| AttrValue::from(*name))
            .collect();

        Self {
            theme: theme.name,
            available_themes: Rc::new(available_themes),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetThemeName(theme) => {
                if let Err(err) = Theme::store_theme_name(&theme) {
                    log::error!("store theme failed: {err}");
                }
                self.theme = theme;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Combobox::new()
            .node_ref(props.node_ref.clone())
            .class(props.class.clone())
            .on_change(ctx.link().callback(Msg::SetThemeName))
            .aria_label("Select Theme")
            .default(self.theme.clone())
            .required(true)
            .items(self.available_themes.clone())
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let theme = Theme::load();
            ctx.link().send_message(Msg::SetThemeName(theme.name));
        }
    }
}

impl From<ThemeNameSelector> for VNode {
    fn from(props: ThemeNameSelector) -> Self {
        let key = props.key.clone();
        let comp = VComp::new::<PwtThemeNameSelector>(Rc::new(props), key);
        VNode::from(comp)
    }
}
