use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{ContainerBuilder, WidgetBuilder};
use crate::widget::{Container, Fa};

#[derive(Properties, Clone, PartialEq)]
pub struct SlidableAction {
    /// The yew component key.
    pub key: Option<Key>,

    /// The action label.
    pub label: AttrValue,

    /// An optional CSS icon class.
    pub icon_class: Option<Classes>,
}

impl SlidableAction {
    pub fn new(label: impl Into<AttrValue>) -> Self {
        yew::props!(Self { label: label.into()})
    }

    /// Builder style method to set the icon CSS class.
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon CSS class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }
}

#[doc(hidden)]
pub struct PwtSlidableAction {

}

impl Component for PwtSlidableAction {
    type Message = ();
    type Properties = SlidableAction;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let icon = props.icon_class.clone().map(|class| {
            Fa::from_class(class)
        });

        Container::new()
            .class("pwt-slidable-action")
            .with_optional_child(icon)
            .with_child(props.label.clone())
            .into()
    }
}

impl Into<VNode> for SlidableAction {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtSlidableAction>(Rc::new(self), key);
        VNode::from(comp)
    }
}
