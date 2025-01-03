use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::state::{Theme, ThemeDensity};
use crate::widget::form::Combobox;

/// Combobox for selecting the theme density.
#[derive(Clone, PartialEq, Properties)]
pub struct ThemeDensitySelector {
    #[prop_or_default]
    class: Classes,
}

impl Default for ThemeDensitySelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeDensitySelector {
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

#[doc(hidden)]
pub struct PwtThemeDensitySelector {
    density: ThemeDensity,
    items: Rc<Vec<AttrValue>>,
}

pub enum Msg {
    SetThemeDensity(ThemeDensity),
}

impl Component for PwtThemeDensitySelector {
    type Message = Msg;
    type Properties = ThemeDensitySelector;

    fn create(_ctx: &Context<Self>) -> Self {
        let theme = Theme::load();

        Self {
            density: theme.density,
            items: Rc::new(vec![
                AttrValue::from(ThemeDensity::Preset.to_string()),
                AttrValue::from(ThemeDensity::Compact.to_string()),
                AttrValue::from(ThemeDensity::Medium.to_string()),
                AttrValue::from(ThemeDensity::Relaxed.to_string()),
            ]),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetThemeDensity(density) => {
                let _ = Theme::store_theme_density(density);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Combobox::new()
            .class(props.class.clone())
            .required(true)
            .default(self.density.to_string())
            .items(self.items.clone())
            .on_change(ctx.link().callback(|density: String| {
                let density = ThemeDensity::try_from(density.as_str()).unwrap_or_default();
                Msg::SetThemeDensity(density)
            }))
            .into()
    }
}

impl From<ThemeDensitySelector> for VNode {
    fn from(val: ThemeDensitySelector) -> Self {
        let comp = VComp::new::<PwtThemeDensitySelector>(Rc::new(val), None);
        VNode::from(comp)
    }
}
