use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

//use pwt::prelude::*;

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
}

impl Component for PwtThemeLoader {
    type Message = ();
    type Properties = ThemeLoader;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        html!{
            <>
                <link id="__pwt-theme-loader__" href="proxmox-yew-style-light.css" rel="stylesheet"/>
                {props.body.clone()}
            </>
        }
    }
}

impl Into<VNode> for ThemeLoader {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeLoader>(Rc::new(self), NodeRef::default(), None);
        VNode::from(comp)
    }
}
