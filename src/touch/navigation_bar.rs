use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::Container;

use crate::widget::TabBarItem;

/// Navigation bar (bottom)
#[derive(Properties, Clone, PartialEq)]
pub struct NavigationBar {
    /// The yew component key.
    pub key: Option<Key>,

    items: Vec<TabBarItem>,

    // Currently active item.
    pub active_item: Option<Key>,
}

impl NavigationBar {
    /// Create a new instance.
    pub fn new(items: Vec<TabBarItem>) -> Self {
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
    pub fn active_item(mut self, active_item: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_active_item(active_item);
        self
    }

    /// Builder style method to set the active item.
    pub fn set_active_item(&mut self, active_item: impl IntoPropValue<Option<Key>>) {
        self.active_item = active_item.into_prop_value();
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

        let children = props.items.iter().map(|item| {
            let is_active = match (&props.active_item, &item.key) {
                (Some(key1), Some(key2)) => key1 == key2,
                _ => false,
            };

            let icon_class = if is_active {
                item.active_icon_class.clone().or_else(|| item.icon_class.clone())
            } else {
                item.icon_class.clone()
            };

            let icon = match icon_class {
                Some(icon_class) => {
                    let mut icon_class = Classes::from(icon_class.to_string());
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
                    let on_activate = item.on_activate.clone();
                    move |_| {
                        if let Some(on_activate) = &on_activate {
                            on_activate.emit(());
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
