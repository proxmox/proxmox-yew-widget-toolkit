use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::state::{Theme, ThemeDensity};
use crate::widget::form::Combobox;
use crate::{impl_class_prop_builder, impl_yew_std_props_builder};

/// Combobox for selecting the theme density.
#[derive(Clone, PartialEq, Properties)]
pub struct ThemeDensitySelector {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// CSS class
    #[prop_or_default]
    pub class: Classes,
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

    impl_yew_std_props_builder!();
    impl_class_prop_builder!();
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
        let key = val.key.clone();
        let comp = VComp::new::<PwtThemeDensitySelector>(Rc::new(val), key);
        VNode::from(comp)
    }
}
