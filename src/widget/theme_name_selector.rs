use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::state::Theme;
use crate::widget::form::Combobox;

#[derive(Clone, PartialEq, Properties)]
pub struct ThemeNameSelector {
    #[prop_or_default]
    class: Classes,
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

pub struct PwtThemeNameSelector {
    theme: String,
    combobox_ref: NodeRef,
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
            combobox_ref: NodeRef::default(),
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
            .node_ref(self.combobox_ref.clone())
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
    fn from(val: ThemeNameSelector) -> Self {
        let comp = VComp::new::<PwtThemeNameSelector>(Rc::new(val), None);
        VNode::from(comp)
    }
}
