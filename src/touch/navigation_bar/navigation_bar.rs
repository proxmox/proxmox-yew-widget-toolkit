use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::Container;

use super::NavigationBarItem;

/// Navigation bar (bottom)
#[derive(Properties, Clone, PartialEq)]
pub struct NavigationBar {
    /// The yew component key.
    pub key: Option<Key>,

    items: Vec<NavigationBarItem>,

    // Currently active item.
    pub active_item: Option<usize>,

    pub on_select: Option<Callback<usize>>,
}

impl NavigationBar {
    /// Create a new instance.
    pub fn new(items: Vec<NavigationBarItem>) -> Self {
        yew::props!(Self { items })
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.key = key.into_prop_value();
    }

    /// Builder style method to set the active item.
    pub fn active_item(mut self, active_item: usize) -> Self {
        self.set_active_item(active_item);
        self
    }

    /// Builder style method to set the active item.
    pub fn set_active_item(&mut self, active_item: usize) {
        self.active_item = Some(active_item);
    }

    pub fn on_select(mut self, cb: impl IntoEventCallback<usize>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }
}

#[doc(hidden)]
pub struct PwtNavigationBar {}

impl Component for PwtNavigationBar {
    type Message = ();
    type Properties = NavigationBar;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let children = props.items.iter().enumerate().map(|(i, item)| {
            let is_active = match props.active_item {
                Some(pos) => pos == i,
                None => false,
            };

            let icon_class = if is_active {
                item.active_icon_class.clone().or_else(|| item.icon_class.clone())
            } else {
                item.icon_class.clone()
            };

            let icon = match icon_class {
                Some(mut icon_class) => {
                    icon_class.push("pwt-navigation-bar-icon");

                    let class = classes!(
                        "pwt-navigation-bar-icon-container",
                        is_active.then(|| "active"),
                    );
                    Some(html!{<div {class}><i class={icon_class}/></div>})
                }
                None => None,
            };
            let label = match &item.label {
                Some(label) => {
                    Some(html!{
                        <div class="pwt-navigation-bar-label">{label}</div>
                    })
                }
                None => None,
            };

            Container::new()
                .class("pwt-navigation-bar-item")
                .with_optional_child(icon)
                .with_optional_child(label)
                .onclick(Callback::from({
                    let on_select = props.on_select.clone();
                    move |_| {
                        if let Some(on_select) = &on_select {
                            on_select.emit(i);
                        }
                    }
                }))
                .into()
            });

        Container::new()
            .class("pwt-navigation-bar")
            .children(children)
            .into()
    }
}

impl Into<VNode> for NavigationBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtNavigationBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}
