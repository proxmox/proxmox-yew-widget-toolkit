use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::Container;

use super::Menu;

#[derive(Clone, PartialEq, Properties)]
pub struct MenuItem {
    pub text: AttrValue,
    pub icon_class: Option<Classes>,
    /// Optional Submenu
    pub menu: Option<Menu>,

    #[prop_or_default]
    pub disabled: bool, // fixme: impl.
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(text: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            text: text.into()
        })
    }

    /// Builder style method to set the icon class.
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }
}

#[doc(hidden)]
pub struct PwtMenuItem {
}

impl Component for PwtMenuItem {
    type Message = ();
    type Properties = MenuItem;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        // fixme: show/hide submenu

        let icon = props.icon_class.as_ref().map(|icon_class| {
            let icon_class = classes!(
                icon_class.clone(),
                "pwt-menu-item-icon",
            );
            html!{<i role="none" aria-hidden="true" class={icon_class}/>}
        });

        Container::new()
            .class("pwt-menu-item")
            .attribute("tabindex", "-1")
             .with_child(html!{
                <i class="pwt-menu-item-indent">{&props.text}</i>
            })
            .with_optional_child(icon)
            .into()
    }
}

impl Into<VNode> for MenuItem {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuItem>(Rc::new(self), None);
        VNode::from(comp)
    }
}
