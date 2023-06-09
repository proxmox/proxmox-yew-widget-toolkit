use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::widget::Column;
use crate::props::{ContainerBuilder, WidgetBuilder};

/// Implements the basic Material Design visual layout structure.
#[derive(Properties, Clone, PartialEq)]
pub struct Scaffold {
    /// The yew component key.
    pub key: Option<Key>,

    /// The primary content displayed below the app bar.
    pub body: Option<Html>,
}

impl Scaffold {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn body(mut self, body: impl Into<VNode>) -> Self {
        self.body = Some(body.into());
        self
    }
}


#[doc(hidden)]
pub struct PwtScaffold {
}

impl Component for PwtScaffold {
    type Message = ();
    type Properties = Scaffold;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Column::new()
            .class("pwt-viewport")
            .with_optional_child(props.body.clone())
            .into()
    }

}

impl Into<VNode> for Scaffold {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtScaffold>(Rc::new(self), key);
        VNode::from(comp)
    }
}