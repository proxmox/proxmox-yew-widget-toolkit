use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
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
    density: String,
    items: Rc<Vec<AttrValue>>,
}

pub enum Msg {
    SetThemeDensity(String),
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

fn get_density() -> Option<&'static str> {
    let root = match get_document_root() {
        Some(root) => root,
        None => return None,
    };

    let class_list = root.class_list();

    if class_list.contains("pwt-density-high") {
        return Some("High");
    }
    if class_list.contains("pwt-density-medium") {
        return Some("Medium");
    }
    if class_list.contains("pwt-density-touch") {
        return Some("Touch");
    }

    let window = match web_sys::window() {
        Some(window) => window,
        None => return None,
    };

    let style = match window.get_computed_style(&root) {
        Ok(Some(style)) => style,
        _ => return None,
    };

    let spacer = match style.get_property_value("--pwt-spacer-base-width") {
        Ok(spacer) => spacer,
        _ => return None,
    };

    log::info!("SPACER WIDTH {spacer}");

    match spacer.as_str() {
        "3px" => Some("High"),
        "5px" => Some("Medium"),
        "10px" => Some("Desktop"),
        _ => None,
    }
}

impl PwtThemeDensitySelector {

    fn set_density(&self, density: String) {
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
        let _ = match density.as_str() {
            "High" => class_list.add_1("pwt-density-high"),
            "Touch" => class_list.add_1("pwt-density-touch"),
            _ => class_list.add_1("pwt-density-medium"),
        };
    }
}

impl Component for PwtThemeDensitySelector {
    type Message = Msg;
    type Properties = ThemeDensitySelector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            density: get_density().unwrap_or("Medium").into(),
            items: Rc::new(vec![
                AttrValue::Static("High"),
                AttrValue::Static("Medium"),
                AttrValue::Static("Touch"),
            ]),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetThemeDensity(density) => {
                //log::info!("SET DENSITY {density}");
                self.set_density(density);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Combobox::new()
            .class(props.class.clone())
            .default(self.density.clone())
            .items(self.items.clone())
            .on_change(ctx.link().callback(Msg::SetThemeDensity))
            .into()
    }
}

impl Into<VNode> for ThemeDensitySelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeDensitySelector>(Rc::new(self), None);
        VNode::from(comp)
    }
}
