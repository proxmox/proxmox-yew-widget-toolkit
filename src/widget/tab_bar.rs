use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::widget::focus::focus_next_tabable;

#[derive(Clone, PartialEq)]
pub struct TabBarItem {
    pub key: Key,
    pub label: String,
    pub icon_class: Option<Classes>,
}

#[derive(Clone, Default, PartialEq, Properties)]
pub struct TabBar {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub tabs: Vec<TabBarItem>,
    on_select: Option<Callback<Option<Key>>>,
}


impl TabBar {

    pub fn new() -> Self {
        yew::props!(TabBar {})
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_item(
        mut self,
        key: impl Into<Key>,
        label: impl Into<String>,
        icon_class: Option<impl Into<Classes>>,
    ) -> Self {
        self.add_item(key, label, icon_class);
        self
    }

    pub fn add_item(
        &mut self,
        key: impl Into<Key>,
        label: impl Into<String>,
        icon_class: Option<impl Into<Classes>>,
    ) {
        self.tabs.push(TabBarItem {
            key: key.into(),
            label: label.into(),
            icon_class: icon_class.map(|c| c.into()),
        });
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

    pub fn on_select(mut self, cb: impl IntoEventCallback<Option<Key>>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }

}

pub enum Msg {
    Activate(Key),
}

pub struct PwtTabBar {
    active: Option<Key>,
}

impl Component for PwtTabBar {
    type Message = Msg;
    type Properties = TabBar;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        if let Some(first) = props.tabs.get(0) {
            ctx.link().send_message(Msg::Activate(first.key.clone()));
        }

        Self {
            active: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Activate(key) => {
                self.active = Some(key);
                if let Some(on_select) = &props.on_select {
                    on_select.emit(self.active.clone());
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let pills = props.tabs.iter().map(|panel| {
            let is_active = if let Some(active) = &self.active {
                &panel.key == active
            } else {
                false
            };

            let nav_class = if is_active { "pwt-nav-link active" } else { "pwt-nav-link" };

            let onclick = ctx.link().callback({
                let key = panel.key.clone();
                move |_| Msg::Activate(key.clone())
            });
            let onkeyup = Callback::from({
                let link = ctx.link().clone();
                let key = panel.key.clone();
                move |event: KeyboardEvent| {
                    if event.key_code() == 32 {
                        link.send_message(Msg::Activate(key.clone()));
                    }
                }
            });

            let tabindex = if is_active { "0" } else { "-1" };

            html!{
                <a {onclick} {onkeyup} class={nav_class} {tabindex}>
                    if let Some(class) = &panel.icon_class {
                        <span class={classes!(class.clone(), "pwt-pe-2")}/>
                    }
                    {&panel.label}
                </a>
            }
        }).collect::<Html>();

        let tabbar_ref =  props.node_ref.clone();
        let onkeydown = Callback::from( move |event: KeyboardEvent| {
            match event.key_code() {
                39 => { // left
                    focus_next_tabable(&tabbar_ref, false, false);
                }
                37 => { // right
                    focus_next_tabable(&tabbar_ref, true, false);
                }
                _ => return,
            }
            event.prevent_default();
        });

        let class = classes!{
            "pwt-nav-pills",
            "pwt-d-flex",
            "pwt-flex-wrap",
            "pwt-column-gap-4",
            props.class.clone(),
        };

        html!{
            <div ref={props.node_ref.clone()} {onkeydown} {class}>
            {pills}
            </div>
        }
    }
}

impl Into<VNode> for TabBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtTabBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}
