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

fn get_document_root() -> Option<web_sys::Element> {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return None,
    };

    let document = match window.document() {
        Some(document) => document,
        None => return None,
    };

    document.document_element()
}

impl PwtThemeDensitySelector {
    fn set_density(&self, density: ThemeDensity) {

        let root = match get_document_root() {
            Some(root) => root,
            None => return,
        };

        let class_list = root.class_list();

        let _ = class_list.remove_3(
            "pwt-density-high",
            "pwt-density-medium",
            "pwt-density-touch",
        );

        let _ = match density {
            ThemeDensity::High => class_list.add_1("pwt-density-high"),
            ThemeDensity::Touch => class_list.add_1("pwt-density-touch"),
            ThemeDensity::Medium => class_list.add_1("pwt-density-medium"),
        };
    }
}

impl Component for PwtThemeDensitySelector {
    type Message = Msg;
    type Properties = ThemeDensitySelector;

    fn create(_ctx: &Context<Self>) -> Self {
        let theme = Theme::load();
        Self {
            density: theme.density,
            items: Rc::new(vec![
                AttrValue::from(ThemeDensity::High.to_string()),
                AttrValue::from(ThemeDensity::Medium.to_string()),
                AttrValue::from(ThemeDensity::Touch.to_string()),
            ]),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetThemeDensity(density) => {
                if let Ok(density) = ThemeDensity::try_from(density) {
                    let _ = Theme::store_theme_density(density);
                    self.set_density(density);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Combobox::new()
            .class(props.class.clone())
            .default(self.density.to_string())
            .items(self.items.clone())
            .on_change(ctx.link().callback(|density: String| {
                let density = ThemeDensity::try_from(density.as_str()).unwrap_or_default();
                Msg::SetThemeDensity(density)
            }))
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::SetThemeDensity(self.density));
        }
    }
}

impl Into<VNode> for ThemeDensitySelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeDensitySelector>(Rc::new(self), None);
        VNode::from(comp)
    }
}
