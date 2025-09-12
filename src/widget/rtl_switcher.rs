use anyhow::format_err;

use pwt_macros::widget;
use yew::{Classes, Component, Properties};

use super::form::Checkbox;
use crate::{
    props::{FieldBuilder, WidgetBuilder},
    state::TextDirection,
};

/// A checkbox to switch between Left-to-Right and Right-to-Left layouts
#[widget(pwt=crate, comp=PwtRtlSwitcher, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct RtlSwitcher {
    #[prop_or_default]
    class: Classes,
}

impl RtlSwitcher {
    /// Creates a new [`RtlSwitcher`].
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

impl Default for RtlSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PwtRtlSwitcher {
    rtl: bool,
}

pub enum Msg {
    ToggleRtl,
}

impl Component for PwtRtlSwitcher {
    type Message = Msg;
    type Properties = RtlSwitcher;

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleRtl => {
                let direction = if self.rtl {
                    TextDirection::Ltr
                } else {
                    TextDirection::Rtl
                };
                match set_text_direction(direction) {
                    Err(err) => {
                        log::error!("{err}");
                        false
                    }
                    Ok(()) => {
                        self.rtl = !self.rtl;
                        true
                    }
                }
            }
        }
    }

    fn create(_ctx: &yew::Context<Self>) -> Self {
        let elements = gloo_utils::document().get_elements_by_tag_name("html");
        let rtl = elements
            .get_with_index(0)
            .and_then(|html| html.get_attribute("dir"))
            .map(|dir| &dir == "rtl")
            .unwrap_or(false);

        Self { rtl }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let onclick = ctx.link().callback(|_| Msg::ToggleRtl);
        Checkbox::new()
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .checked(self.rtl)
            .on_change(onclick)
            .into()
    }
}

/// Sets the global text direction on the root HTML element (if possible).
/// Otherwise returns an error.
pub fn set_text_direction(direction: TextDirection) -> Result<(), anyhow::Error> {
    let elements = gloo_utils::document().get_elements_by_tag_name("html");
    let html = elements
        .get_with_index(0)
        .ok_or_else(|| format_err!("no html element found"))?;
    match direction {
        TextDirection::Ltr => {
            html.remove_attribute("dir")
                .map_err(|err| format_err!("could not remove dir attribute: {err:?}"))?;
        }
        TextDirection::Rtl => {
            html.set_attribute("dir", "rtl")
                .map_err(|err| format_err!("could not set dir attribute: {err:?}"))?;
        }
    }
    Ok(())
}
