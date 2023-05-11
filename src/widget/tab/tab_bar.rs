use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::state::{NavigationContext, NavigationContextExt, Selection};
use crate::widget::dom::element_direction_rtl;
use crate::widget::focus::roving_tabindex_next;
use crate::widget::Container;

use super::TabBarItem;

use pwt_macros::builder;

/// Tab bar
///
/// The [TabPanel](super::TabPanel) combines a [TabBar] with
/// [SelectionView](crate::widget::SelectionView) to simplify usage.
///
/// # Automatic routing.
///
/// [TabBar] supports fully automatic routing if you put the bar inside
/// a [NavigationContainer](crate::state::NavigationContainer) and
/// set the router flag.
#[derive(Clone, Default, PartialEq, Properties)]
#[builder]
pub struct TabBar {
    /// The yew node ref.
    #[prop_or_default]
    node_ref: NodeRef,

    /// The yew component key.
    pub key: Option<Key>,

    /// CSS class.
    #[prop_or_default]
    pub class: Classes,

    /// Tab bar items.
    #[prop_or_default]
    pub tabs: Vec<TabBarItem>,

    /// Selection object to store the currently selected tab key.
    ///
    /// The optional selction object allows you to control and observer the state from outside.
    #[builder(IntoPropValue, into_prop_value)]
    pub selection: Option<Selection>,

    /// Selection callback.
    #[builder_cb(IntoEventCallback, into_event_callback, Option<Key>)]
    pub on_select: Option<Callback<Option<Key>>>,

    /// Default active key.
    pub default_active: Option<Key>,

    /// Enable router functionality.
    ///
    /// Save/Load state from parent NavigationContainer
    #[builder]
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

    // Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property.
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
    }

    // Builder style method to set `default_active` property.
    pub fn default_active(mut self, default_active: impl IntoOptionalKey) -> Self {
        self.set_default_active(default_active);
        self
    }

    /// Method to set the yew `default_active` property.
    pub fn set_default_active(&mut self, default_active: impl IntoOptionalKey) {
        self.default_active = default_active.into_optional_key();
    }

    pub fn with_item(mut self, item: impl Into<TabBarItem>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<TabBarItem>) {
        self.tabs.push(item.into());
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

    fn get_default_active(&self) -> Option<Key> {
        if self.default_active.is_some() {
            return self.default_active.clone();
        }

        for item in &self.tabs {
            if let TabBarItem { key: Some(key), .. } = item {
                return Some(key.clone());
            }
        }

        None
    }
}

pub enum Msg {
    FocusIn,
    Select(Option<Key>, bool),
    SelectionChange(Selection),
}

#[doc(hidden)]
pub struct PwtTabBar {
    active: Option<Key>,
    rtl: Option<bool>,
    _nav_ctx_handle: Option<ContextHandle<NavigationContext>>,
    selection: Selection,
}

fn get_active_or_default(props: &TabBar, active: &Option<Key>) -> Option<Key> {
    if let Some(active_key) = active.as_deref() {
        if !active_key.is_empty() && active_key != "_" {
            return active.clone();
        }
    }
    props.get_default_active()
}

impl PwtTabBar {
    fn init_selection(
        ctx: &Context<Self>,
        selection: Option<Selection>,
        active: &Option<Key>,
    ) -> Selection {
        let selection = match selection {
            Some(selection) => selection,
            None => Selection::new(),
        }
        .on_select(ctx.link().callback(Msg::SelectionChange));

        if let Some(active) = &active {
            selection.select(active.clone());
        } else {
            selection.clear();
        }

        selection
    }
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
                    link.send_message(Msg::Select(Some(key), false));
                }
            });
            if let Some((nav_ctx, handle)) =
                ctx.link().context::<NavigationContext>(on_nav_ctx_change)
            {
                //log::info!("INIT CTX {:?}", nav_ctx);
                _nav_ctx_handle = Some(handle);
                let path = nav_ctx.path();
                active = get_active_or_default(props, &Some(Key::from(path)));
            }
        }

        let selection = Self::init_selection(ctx, props.selection.clone(), &active);

        if let Some(on_select) = &props.on_select {
            on_select.emit(active.clone());
        }
        Self {
            active,
            selection,
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
            // Handle external selection changes
            Msg::SelectionChange(selection) => {
                let key = selection.selected_key();
                let key = get_active_or_default(props, &key);

                if &self.active == &key {
                    return false;
                }

                self.active = key;

                if let Some(key) = &self.active {
                    if props.router {
                        ctx.link().push_relative_route(&key);
                    }
                }
                if let Some(on_select) = &props.on_select {
                    on_select.emit(self.active.clone());
                }

                true
            }
            // Handle internal selection changes
            Msg::Select(key, update_route) => {
                let key = get_active_or_default(props, &key);
                if &self.active == &key {
                    return false;
                }

                // set active to avoid Msg::SelectionChange
                self.active = key.clone();

                if let Some(key) = &key {
                    self.selection.select(key.clone());
                } else {
                    self.selection.clear();
                }

                if props.router && update_route {
                    ctx.link().push_relative_route(key.as_deref().unwrap_or(""));
                }

                if let Some(on_select) = &props.on_select {
                    on_select.emit(key);
                }

                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.selection != old_props.selection {
            self.selection = Self::init_selection(ctx, props.selection.clone(), &self.active);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let active = get_active_or_default(props, &self.active);

        let pills = props
            .tabs
            .iter()
            .map(|panel| {
                let is_active = if let Some(active) = &active {
                    panel.key.as_ref() == Some(active)
                } else {
                    false
                };

                let nav_class = if is_active {
                    "pwt-nav-link active"
                } else {
                    "pwt-nav-link"
                };

                let onclick = ctx.link().callback({
                    let key = panel.key.clone();
                    let on_activate = panel.on_activate.clone();
                    move |_| {
                        if let Some(on_activate) = &on_activate {
                            on_activate.emit(());
                        }
                        Msg::Select(key.clone(), true)
                    }
                });
                let onkeyup = Callback::from({
                    let link = ctx.link().clone();
                    let key = panel.key.clone();
                    let on_activate = panel.on_activate.clone();
                    move |event: KeyboardEvent| {
                        if event.key_code() == 32 {
                            if let Some(on_activate) = &on_activate {
                                on_activate.emit(());
                            }
                            link.send_message(Msg::Select(key.clone(), true));
                        }
                    }
                });

                let tabindex = if is_active { "0" } else { "-1" };

                html! {
                    <a {onclick} {onkeyup} class={nav_class} {tabindex}>
                        if let Some(class) = &panel.icon_class {
                            <span class={class.to_string()} aria-hidden="true"/>
                        }
                        {panel.label.as_deref().unwrap_or("")}
                    </a>
                }
            })
            .collect::<Html>();

        let pills_ref = props.node_ref.clone();
        let rtl = self.rtl.unwrap_or(false);

        Container::new()
            .node_ref(props.node_ref.clone())
            .class("pwt-nav-pills")
            .class(props.class.clone())
            .with_child(pills)
            .onkeydown(move |event: KeyboardEvent| {
                match event.key_code() {
                    39 => {
                        // left
                        roving_tabindex_next(&pills_ref, rtl, false);
                    }
                    37 => {
                        // right
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
