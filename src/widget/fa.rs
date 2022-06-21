// Fontawesome icons

use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

#[derive(Properties, PartialEq, Clone)]
pub struct Fa {
    #[prop_or_default]
    pub class: Classes,
}

impl Fa {

    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            class: classes!("fa", format!("fa-{}", name.as_ref())),
        }
    }

    pub fn from_class(class: impl Into<Classes>) -> Self {
        Self {
            class: class.into(),
        }
    }

    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    pub fn pulse(mut self) -> Self {
        self.add_class("fa-pulse");
        self
    }  
}

#[function_component(PwtFa)]
pub fn pwt_fa(props: &Fa) -> Html {

    html!{
        <i class={props.class.clone()} role="status"/>
    }

}

impl Into<VNode> for Fa {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtFa>(Rc::new(self), NodeRef::default(), None);
        VNode::from(comp)
    }
}
