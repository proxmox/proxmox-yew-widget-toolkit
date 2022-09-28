use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::state::Theme;

#[derive(Properties, Clone, PartialEq)]
pub struct ThemeLoader {
    body: VNode,
}

impl ThemeLoader {

    pub fn new(body: impl Into<VNode>) -> Self {
        Self { body: body.into() }
    }
}

pub struct PwtThemeLoader {
    loaded: bool,
}

pub enum Msg {
    Loaded,
}

impl Component for PwtThemeLoader {
    type Message = Msg;
    type Properties = ThemeLoader;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { loaded: false}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded => {
                self.loaded = true;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let default_css = Theme::default().get_css_filename();
        let onload = ctx.link().callback(|_| Msg::Loaded);

        html! {
            <>
                <link {onload} id="__pwt-theme-loader__" href={ default_css } rel="stylesheet"/>
                {self.loaded.then(|| props.body.clone())}
            </>
        }
    }
}

impl Into<VNode> for ThemeLoader {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeLoader>(Rc::new(self), None);
        VNode::from(comp)
    }
}
