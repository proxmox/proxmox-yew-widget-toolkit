use std::collections::HashMap;
use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use pwt_macros::builder;

use crate::props::{ContainerBuilder, EventSubscriber, IntoOptionalKey, WidgetBuilder};
use crate::state::{NavigationContainer, NavigationContext, NavigationContextExt, Selection};

use crate::widget::focus::roving_tabindex_next;
use crate::widget::{Column, Container};

use super::{Menu, MenuEntry, MenuItem};

/// Navigation Menu Widget.
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct NavigationDrawer {
    /// The yew component key.
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    menu: Menu,

    /// Menu header.
    #[builder(IntoPropValue, into_prop_value)]
    pub header: Option<Html>,

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

    /// ARIA label.
    #[builder(IntoPropValue, into_prop_value)]
    pub aria_label: Option<AttrValue>,

    /// Enable router functionality.
    ///
    /// Save/Load state from parent NavigationContainer
    #[builder]
    #[prop_or_default]
    router: bool,
}

impl NavigationDrawer {
    /// Create a new instance.
    pub fn new(menu: Menu) -> Self {
        yew::props!(Self { menu })
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

    /// Builder style method to add a html class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Embed the [NavigationDrawer] into a [NavigationContainer]
    pub fn navigation_container(mut self) -> NavigationContainer {
        self.router = true;
        NavigationContainer::new().with_child(self)
    }

    fn get_default_active(&self) -> Option<Key> {
        if self.default_active.is_some() {
            return self.default_active.clone();
        }

        for item in &self.menu.children {
            if let MenuEntry::Item(MenuItem { key: Some(key), .. }) = item {
                return Some(key.clone());
            }
        }

        None
    }
}

pub enum Msg {
    Select(Option<Key>, bool, bool),
    SelectionChange(Selection),
    MenuToggle(Key),
    MenuClose(Key),
    MenuOpen(Key),
}

#[doc(hidden)]
pub struct PwtNavigationDrawer {
    active: Option<Key>,
    selection: Selection,
    menu_states: HashMap<Key, bool>, // true = open
    menu_ref: NodeRef,
    _nav_ctx_handle: Option<ContextHandle<NavigationContext>>,
}

impl PwtNavigationDrawer {
    fn render_single_item(
        &self,
        ctx: &yew::Context<PwtNavigationDrawer>,
        item: &MenuItem,
        active: &str,
        indent_level: usize,
        open: bool, // submenu open ?
    ) -> Html {
        let is_active = Some(active) == item.key.as_deref();
        let is_menu = item.submenu.is_some();

        let onclick = Callback::from({
            let link = ctx.link().clone();
            let key = item.key.clone();
            move |_event: MouseEvent| {
                if key.is_some() {
                    link.send_message(Msg::Select(key.clone(), true, true));
                }
            }
        });

        let ontoggle = Callback::from({
            let link = ctx.link().clone();
            let key = item.key.clone();
            move |event: MouseEvent| {
                event.stop_propagation();
                if let Some(key) = &key {
                    link.send_message(Msg::MenuToggle(key.clone()));
                }
            }
        });

        let onkeydown = ctx.link().batch_callback({
            let key = item.key.clone();
            move |event: KeyboardEvent| {
                let key = match &key {
                    Some(key) => key,
                    None => return None,
                };
                match event.key().as_str() {
                    " " => Some(Msg::Select(Some(key.clone()), true, true)),
                    "ArrowRight" if is_menu => Some(Msg::MenuOpen(key.clone())),
                    "ArrowLeft" if is_menu => Some(Msg::MenuClose(key.clone())),
                    _ => None,
                }
            }
        });

        let aria_expanded = if is_menu {
            Some(if open { "true" } else { "false" })
        } else {
            None
        };

        Container::new()
            .key(item.key.clone())
            .tag("a")
            .attribute("role", "link")
            .attribute("aria-expanded", aria_expanded)
            //.attribute("disabled", (!visible).then(|| "true"))
            .attribute("tabindex", if is_active { "0" } else { "-1" })
            .class("pwt-nav-link")
            //.class((!visible).then(|| "pwt-d-none"))
            .class(crate::css::AlignItems::Baseline)
            .class(is_active.then_some("active"))
            .onclick(onclick)
            .onkeydown(onkeydown)
            // add indentation
            .with_optional_child((indent_level > 0).then(|| {
                html!{<span style={format!("width: {}rem", (indent_level as f32) * 1.0)}/>}
            }))
            // add optional icon on the left
            .with_optional_child(item.icon_class.as_ref().and_then(|icon| {
                Some(html! { <i class={classes!(icon.to_string(), "pwt-nav-menu-icon")}/>})
            }))
            // add memu label
            .with_child(html! {<div class="pwt-text-truncate pwt-flex-fill">{&item.label}</div>})
            // add optional menu-open icon
            .with_optional_child(is_menu.then(|| {
                Container::new()
                    .tag("i")
                    .attribute("aria-hidden", "true")
                    .class("fa fa-caret-down")
                    .class("pwt-nav-menu-item-arrow")
                    .class(open.then_some("expanded"))
                    .onclick(ontoggle)
                    .with_child("\u{00a0}")
            }))
            .into()
    }

    fn render_menu_entry(
        &self,
        ctx: &yew::Context<PwtNavigationDrawer>,
        item: &MenuEntry,
        menu: &mut Column,
        active: &str,
        level: usize,
    ) {
        match item {
            MenuEntry::Item(child) => {
                let open = match &child.key {
                    Some(key) => *self.menu_states.get(key).unwrap_or(&true),
                    None => false,
                };

                menu.add_child(self.render_single_item(ctx, child, active, level, open));

                if let Some(submenu) = &child.submenu {
                    if open {
                        for sub in submenu.children.iter() {
                            self.render_menu_entry(ctx, sub, menu, active, level + 1)
                        }
                    }
                }
            }
            MenuEntry::Component(comp) => {
                menu.add_child(comp.clone());
            }
        }
    }

    fn find_selectable_key(&mut self, ctx: &Context<Self>, desired: &Key) -> Option<Key> {
        self.find_selectable_entry(ctx, desired)
            .and_then(|entry| match entry {
                MenuEntry::Item(item) => item.key.clone(),
                MenuEntry::Component(_) => None,
            })
    }

    fn find_selectable_entry<'a>(
        &'a mut self,
        ctx: &'a Context<Self>,
        desired: &Key,
    ) -> Option<&'a MenuEntry> {
        let props = ctx.props();

        fn find_first_key_recursive(menu: &[MenuEntry]) -> Option<&MenuEntry> {
            for menu in menu.iter() {
                let res = match menu {
                    MenuEntry::Item(item) => match &item.submenu {
                        None => Some(menu),
                        Some(submenu) => {
                            if item.key.is_none() || !item.selectable {
                                find_first_key_recursive(&submenu.children[..])
                            } else {
                                Some(menu)
                            }
                        }
                    },
                    _ => None,
                };
                if res.is_some() {
                    return res;
                }
            }
            None
        }

        fn find_item_recursive<'a>(
            menu: &'a [MenuEntry],
            desired: &Key,
        ) -> Option<&'a MenuEntry> {
            for menu in menu.iter() {
                match menu {
                    MenuEntry::Item(item) => {
                        if item.key.as_ref() == Some(desired) {
                            return Some(menu);
                        }

                        if let Some(submenu) = &item.submenu {
                            let res = find_item_recursive(&submenu.children[..], desired);
                            if res.is_some() {
                                return res;
                            }
                        }
                    }
                    _ => {}
                };
            }
            None
        }

        match find_item_recursive(&props.menu.children, &desired) {
            Some(entry @ MenuEntry::Item(item)) => match &item.submenu {
                None => item.selectable.then(|| entry),
                Some(submenu) => {
                    if item.selectable {
                        Some(entry)
                    } else {
                        self.menu_states.insert(desired.clone(), true);
                        find_first_key_recursive(&submenu.children)
                    }
                }
            },
            _ => None,
        }
    }

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

fn get_active_or_default(props: &NavigationDrawer, active: &Option<Key>) -> Option<Key> {
    if let Some(active_str) = active.as_deref() {
        if !active_str.is_empty() && active_str != "_" {
            return active.clone();
        }
    }
    props.get_default_active()
}

impl Component for PwtNavigationDrawer {
    type Message = Msg;
    type Properties = NavigationDrawer;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let mut _nav_ctx_handle = None;
        let mut active = props.get_default_active();

        if props.router {
            let on_nav_ctx_change = Callback::from({
                let link = ctx.link().clone();
                move |nav_ctx: NavigationContext| {
                    //log::info!("CTX CHANGE {:?}", nav_ctx);
                    let path = nav_ctx.path();
                    let key = Some(Key::from(path));
                    link.send_message(Msg::Select(key, false, false));
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
            menu_states: HashMap::new(),
            menu_ref: NodeRef::default(),
            _nav_ctx_handle,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            // Handle external selection changes
            Msg::SelectionChange(selection) => {
                let key = selection.selected_key();
                let key = get_active_or_default(props, &key);

                let key = if let Some(key) = key {
                    self.find_selectable_key(ctx, &key)
                } else {
                    None
                };

                if &self.active == &key {
                    return false;
                }

                //log::info!("SELCHANGE {:?}  {:?}", key, self.active);

                self.active = key.clone();

                if props.router {
                    ctx.link().push_relative_route(key.as_deref().unwrap_or(""));
                }

                if let Some(on_select) = &props.on_select {
                    on_select.emit(key);
                }

                true
            }
            // Handle internal selection changes
            Msg::Select(key, update_route, toggle) => {
                let key = get_active_or_default(props, &key);
                let key = if let Some(key) = key {
                    self.find_selectable_key(ctx, &key)
                } else {
                    None
                };

                if key == self.active {
                    if let Some(key) = key {
                        if toggle {
                            let entry =
                                *self.menu_states.entry(key.clone()).or_insert_with(|| true);
                            self.menu_states.insert(key, !entry);
                        }
                    }
                    return true;
                }

                // log::info!("SELECT {:?}", key);

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
            Msg::MenuToggle(key) => {
                let entry = *self.menu_states.entry(key.clone()).or_insert_with(|| true);
                self.menu_states.insert(key, !entry);
                true
            }
            Msg::MenuClose(key) => {
                self.menu_states.insert(key, false);
                true
            }
            Msg::MenuOpen(key) => {
                self.menu_states.insert(key, true);
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

        let menu_ref = self.menu_ref.clone();
        let onkeydown = Callback::from(move |event: KeyboardEvent| {
            match event.key().as_str() {
                "ArrowDown" => {
                    roving_tabindex_next(&menu_ref, false, false);
                }
                "ArrowUp" => {
                    roving_tabindex_next(&menu_ref, true, false);
                }
                _ => return,
            }
            event.prevent_default();
        });

        let mut column = Column::new()
            .node_ref(self.menu_ref.clone())
            .class("pwt-fit")
            .onkeydown(onkeydown)
            // avoid https://bugzilla.mozilla.org/show_bug.cgi?id=1069739
            .attribute("tabindex", "-1")
            .attribute("role", "navigation")
            .attribute("aria-label", props.aria_label.clone())
            .class("pwt-nav-menu pwt-overflow-none")
            .class(props.class.clone())
            .with_optional_child(props.header.clone());

        let active = get_active_or_default(props, &self.active);
        let active = active.as_deref().unwrap_or("");

        for item in props.menu.children.iter() {
            self.render_menu_entry(ctx, item, &mut column, active, 0);
        }

        column.into()
    }
}

impl Into<VNode> for NavigationDrawer {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtNavigationDrawer>(Rc::new(self), key);
        VNode::from(comp)
    }
}
