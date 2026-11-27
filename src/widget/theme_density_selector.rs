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

        // the names are stored as they are, only their labels are translated
        let items = [
            (ThemeDensity::Preset, tr!("Preset")),
            (ThemeDensity::Compact, tr!("Compact")),
            (ThemeDensity::Medium, tr!("Medium")),
            (ThemeDensity::Relaxed, tr!("Relaxed")),
        ]
        .map(|(density, label)| (density.to_string(), label));

        Combobox::from_key_value_pairs(items)
            .class(props.class.clone())
            .required(true)
            .default(self.density.to_string())
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
