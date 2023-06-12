use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::css::JustifyContent;
use crate::prelude::*;
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::state::{NavigationContext, NavigationContextExt, Selection};
use crate::widget::Container;

use crate::widget::TabBarItem;

use pwt_macros::builder;

/// Navigation rail
///
/// # Automatic routing.
///
/// [NavigationRail] supports fully automatic routing if you put the rail inside
/// a [NavigationContainer](crate::state::NavigationContainer) and
/// set the router flag.

// Note: This is Similatr to TabBar without keyboard support.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct NavigationRail {
    /// The yew component key.
    pub key: Option<Key>,

    /// Leading widget placed above the navigation group.
    pub leading: Option<Html>,

    #[builder]
    #[prop_or(JustifyContent::Center)]
    pub group_alignment: JustifyContent,

    /// Navigation bar items.
    items: Vec<TabBarItem>,

    /// Selection object to store the currently selected tab key.
    ///
    /// The optional selction object allows you to control and observe the state from outside.
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

impl NavigationRail {
    /// Create a new instance.
    pub fn new(items: Vec<TabBarItem>) -> Self {
        yew::props!(Self { items })
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

    /// Builder style method to set the leading widget.
    pub fn leading(mut self, leading: impl Into<VNode>) -> Self {
        self.set_leading(leading);
        self
    }

    /// Method to set the leading widget.
    pub fn set_leading(&mut self, leading: impl Into<VNode>) {
        self.leading = Some(leading.into());
    }

    fn get_default_active(&self) -> Option<Key> {
        if self.default_active.is_some() {
            return self.default_active.clone();
        }

        for item in &self.items {
            if let TabBarItem { key: Some(key), .. } = item {
                return Some(key.clone());
            }
        }

        None
    }
}

pub enum Msg {
    Select(Option<Key>, bool),
    SelectionChange(Selection),
}

#[doc(hidden)]
pub struct PwtNavigationRail {
    active: Option<Key>,
    selection: Selection,
    _nav_ctx_handle: Option<ContextHandle<NavigationContext>>,
}

fn get_active_or_default(props: &NavigationRail, active: &Option<Key>) -> Option<Key> {
    if let Some(active_key) = active.as_deref() {
        if !active_key.is_empty() && active_key != "_" {
            return active.clone();
        }
    }
    props.get_default_active()
}

impl PwtNavigationRail {
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

impl Component for PwtNavigationRail {
    type Message = Msg;
    type Properties = NavigationRail;

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
            selection,
            active,
            _nav_ctx_handle,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
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
                log::info!("select {:?}", key);

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

        let children = props.items.iter().map(|item| {
            let is_active = match (&active, &item.key) {
                (Some(key1), Some(key2)) => key1 == key2,
                _ => false,
            };

            let icon_class = if is_active {
                item.active_icon_class
                    .clone()
                    .or_else(|| item.icon_class.clone())
            } else {
                item.icon_class.clone()
            };

            let icon = match icon_class {
                Some(icon_class) => {
                    let mut icon_class = Classes::from(icon_class.to_string());
                    icon_class.push("pwt-navigation-rail-icon");

                    let class = classes!(
                        "pwt-navigation-rail-icon-container",
                        is_active.then(|| "active"),
                    );
                    Some(html! {<div {class}><i class={icon_class}/></div>})
                }
                None => None,
            };
            let label = match &item.label {
                Some(label) => Some(html! {
                    <div class="pwt-navigation-rail-label">{label}</div>
                }),
                None => None,
            };

            Container::new()
                .class("pwt-navigation-rail-item")
                .with_optional_child(icon)
                .with_optional_child(label)
                .onclick(ctx.link().callback({
                    let key = item.key.clone();
                    let on_activate = item.on_activate.clone();
                    move |_| {
                        if let Some(on_activate) = &on_activate {
                            on_activate.emit(());
                        }
                        Msg::Select(key.clone(), true)
                    }
                }))
                .into()
        });

        Container::new()
            .class("pwt-navigation-rail")
            .with_optional_child(props.leading.clone())
            .with_child(
                Container::new()
                    .class("pwt-navigation-rail-tabs")
                    .class(props.group_alignment)
                    .children(children),
            )
            .into()
    }
}

impl Into<VNode> for NavigationRail {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtNavigationRail>(Rc::new(self), key);
        VNode::from(comp)
    }
}
