use std::rc::Rc;

use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::state::{NavigationContext, NavigationContextExt};
use super::focus::roving_tabindex_next;
use super::dom::element_direction_rtl;
use super::Container;

#[derive(Clone, PartialEq)]
pub struct TabBarItem {
    pub key: Key,
    pub label: AttrValue,
    pub icon_class: Option<AttrValue>,
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

    pub default_active: Option<Key>,

    #[prop_or_default]
    router: bool,
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

    /// Builder style method to enable router functionality
    pub fn router(mut self, enable: bool) -> Self {
        self.set_router(enable);
        self
    }

    /// Method to enable router functionality.
    ///
    /// Save/Load state from parent NavigationContainer
    pub fn set_router(&mut self, enable: bool) {
        self.router = enable;
    }

    pub fn with_item(
        mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
    ) -> Self {
        self.add_item(key, label, icon_class);
        self
    }

    pub fn add_item(
        &mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
    ) {
        self.tabs.push(TabBarItem {
            key: key.into(),
            label: label.into(),
            icon_class: icon_class.into_prop_value(),
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

    fn get_default_active(&self) -> Option<Key> {
        if self.default_active.is_some() {
            return self.default_active.clone();
        }

        if let Some(first) = self.tabs.get(0) {
            return Some(first.key.clone());
        }

        None
    }
}

pub enum Msg {
    FocusIn,
    Select(Key, bool),
}

#[doc(hidden)]
pub struct PwtTabBar {
    pills_ref: NodeRef,
    active: Option<Key>,
    rtl: Option<bool>,
    _nav_ctx_handle: Option<ContextHandle<NavigationContext>>,
}

fn get_active_or_default(props: &TabBar, active: &Option<Key>) -> Option<Key> {
    if let Some(active_key) = active.as_deref() {
        if !active_key.is_empty() && active_key != "_" {
            return active.clone();
        }
    }
    props.get_default_active()
}

impl Component for PwtTabBar {
    type Message = Msg;
    type Properties = TabBar;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let mut active = props.get_default_active();

        let mut _nav_ctx_handle = None;

        if props.router {
            let on_nav_ctx_change = Callback::from({
                let link = ctx.link().clone();
                move |nav_ctx: NavigationContext| {
                    //log::info!("CTX CHANGE {:?}", nav_ctx);
                    let path = nav_ctx.path();
                    let key = Key::from(path);
                    link.send_message(Msg::Select(key, false));
                }
            });
            if let Some((nav_ctx, handle)) = ctx.link().context::<NavigationContext>(on_nav_ctx_change) {
                //log::info!("INIT CTX {:?}", nav_ctx);
                _nav_ctx_handle = Some(handle);
                let path = nav_ctx.path();
                active = get_active_or_default(props, &Some(Key::from(path)));
            }
        }

        if active.is_some() {
            if let Some(on_select) = &props.on_select {
                on_select.emit(active.clone());
            }
        }

        Self {
            pills_ref: NodeRef::default(),
            active,
            rtl: None,
            _nav_ctx_handle,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FocusIn => {
                self.rtl = element_direction_rtl(&props.node_ref);
                true
            }
            Msg::Select(key, update_route) => {
                if let Some(active) = &self.active {
                    if &key == active { return false; }
                }

                self.active = get_active_or_default(props, &Some(key.clone()));

                if props.router && update_route {
                    ctx.link().push_relative_route(&key);
                }

                if let Some(on_select) = &props.on_select {
                    on_select.emit(self.active.clone());
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let active = get_active_or_default(props, &self.active);

        let pills = props.tabs.iter().map(|panel| {
            let is_active = if let Some(active) = &active {
                &panel.key == active
            } else {
                false
            };

            let nav_class = if is_active { "pwt-nav-link active" } else { "pwt-nav-link" };

            let onclick = ctx.link().callback({
                let key = panel.key.clone();
                move |_| Msg::Select(key.clone(), true)
            });
            let onkeyup = Callback::from({
                let link = ctx.link().clone();
                let key = panel.key.clone();
                move |event: KeyboardEvent| {
                    if event.key_code() == 32 {
                        link.send_message(Msg::Select(key.clone(), true));
                    }
                }
            });

            let tabindex = if is_active { "0" } else { "-1" };

            html!{
                <a {onclick} {onkeyup} class={nav_class} {tabindex}>
                    if let Some(class) = &panel.icon_class {
                        <span class={class.to_string()} aria-hidden="true"/>
                    }
                    {&panel.label}
                </a>
            }
        }).collect::<Html>();

        let pills_ref = self.pills_ref.clone();
        let rtl = self.rtl.unwrap_or(false);

        Container::new()
            .node_ref(props.node_ref.clone())
            .class("pwt-nav-pills")
            .class(props.class.clone())
            .with_child(html!{<div ref={self.pills_ref.clone()} class="pwt-nav-pills-content">{pills}</div>})
            .onkeydown(move |event: KeyboardEvent| {
                match event.key_code() {
                    39 => { // left
                        roving_tabindex_next(&pills_ref, rtl, false);
                    }
                    37 => { // right
                        roving_tabindex_next(&pills_ref, !rtl, false);
                    }
                    _ => return,
                }
                event.prevent_default();
            })
            .onfocusin(ctx.link().callback(|_| Msg::FocusIn))
            .into()
    }
}

impl Into<VNode> for TabBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtTabBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}
