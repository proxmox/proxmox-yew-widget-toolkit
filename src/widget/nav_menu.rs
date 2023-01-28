//! Navigation menus with router support.

use std::collections::HashMap;
use std::ops::Deref;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VNode};
use yew::{html, Component, Html, Properties};

use crate::props::{ContainerBuilder, EventSubscriber, RenderFn, WidgetBuilder};
use crate::state::{NavigationContainer, NavigationContext, NavigationContextExt};

use pwt_macros::widget;

use crate::widget::focus::roving_tabindex_next;
use crate::widget::{Column, Container, Pane, SplitPane};

#[derive(Clone, PartialEq)]
pub struct MenuItem {
    id: Key,
    text: AttrValue,
    icon_class: Option<AttrValue>,
    content: RenderFn<Key>,
}

impl MenuItem {
    pub fn new(
        id: impl Into<Key>,
        text: impl IntoPropValue<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        content: impl Fn(&Key) -> Html + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into_prop_value(),
            icon_class: icon_class.into_prop_value(),
            content: RenderFn::new(content),
        }
    }

    pub fn submenu(self) -> SubMenu {
        SubMenu {
            item: self,
            children: Vec::new(),
            selectable: true,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SubMenu {
    item: MenuItem,
    children: Vec<Menu>,
    selectable: bool,
}

impl From<MenuItem> for SubMenu {
    fn from(item: MenuItem) -> Self {
        item.submenu()
    }
}

impl SubMenu {
    pub fn with_item(mut self, item: impl Into<Menu>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<Menu>) {
        self.children.push(item.into());
    }

    pub fn with_component(mut self, component: impl Into<VNode>) -> Self {
        self.add_component(component);
        self
    }

    pub fn add_component(&mut self, component: impl Into<VNode>) {
        self.children.push(Menu::Component(component.into()))
    }

    pub fn unselectable(mut self) -> Self {
        self.set_selectable(false);
        self
    }

    pub fn set_selectable(&mut self, selectable: bool) {
        self.selectable = selectable;
    }
}

#[derive(Clone, PartialEq)]
pub enum Menu {
    Item(MenuItem),
    SubMenu(SubMenu),
    Component(VNode),
}

impl From<SubMenu> for Menu {
    fn from(submenu: SubMenu) -> Self {
        Menu::SubMenu(submenu)
    }
}

impl From<MenuItem> for Menu {
    fn from(item: MenuItem) -> Self {
        Menu::Item(item)
    }
}

#[widget(pwt=crate, comp=PwtNavigationMenu, @element)]
#[derive(PartialEq, Clone, Properties)]
pub struct NavigationMenu {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,
    /// ARIA label.
    pub aria_label: Option<AttrValue>,
    #[prop_or_default]
    menu: Vec<Menu>,
    default_active: Option<Key>,
    #[prop_or_default]
    router: bool,
    on_select: Option<Callback<Option<Key>>>,
}

impl NavigationMenu {
    pub fn new() -> Self {
        yew::props!(Self {})
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

    /// Builder style method to set the html aria-label attribute
    pub fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute
    pub fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.aria_label = label.into_prop_value();
    }

    pub fn navigation_container(mut self) -> NavigationContainer {
        self.router = true;
        NavigationContainer::new().with_child(self)
    }

    pub fn default_active(mut self, active: impl IntoPropValue<Option<String>>) -> Self {
        let active: Option<String> = active.into_prop_value();
        self.default_active = active.map(|active| Key::from(active));
        self
    }

    pub fn with_item(mut self, item: impl Into<Menu>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<Menu>) {
        self.menu.push(item.into());
    }

    pub fn with_component(mut self, component: impl Into<VNode>) -> Self {
        self.add_component(component);
        self
    }

    pub fn add_component(&mut self, component: impl Into<VNode>) {
        self.menu.push(Menu::Component(component.into()))
    }

    pub fn on_select(mut self, cb: impl IntoEventCallback<Option<Key>>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Select(Option<Key>, bool, bool),
    MenuClose(Key),
    MenuOpen(Key),
}

#[doc(hidden)]
pub struct PwtNavigationMenu {
    active: Option<Key>,
    menu_states: HashMap<Key, bool>, // true = open
    menu_ref: NodeRef,
    _nav_ctx_handle: Option<ContextHandle<NavigationContext>>,
}

impl PwtNavigationMenu {
    fn render_child(
        &self,
        ctx: &yew::Context<PwtNavigationMenu>,
        item: &MenuItem,
        active: &str,
        indent_level: usize,
        is_menu: bool,
        visible: bool,
    ) -> Html {
        let is_active = active == item.id.deref();

        let onclick = ctx.link().callback({
            let key = item.id.clone();
            move |_event: MouseEvent| Msg::Select(Some(key.clone()), true, true)
        });

        let onkeydown = ctx.link().batch_callback({
            let key = item.id.clone();
            move |event: KeyboardEvent| match event.key().as_str() {
                " " => Some(Msg::Select(Some(key.clone()), true, true)),
                "ArrowRight" if is_menu => Some(Msg::MenuOpen(key.clone())),
                "ArrowLeft" if is_menu => Some(Msg::MenuClose(key.clone())),
                _ => None,
            }
        });

        let open = if is_menu {
            *self.menu_states.get(&item.id).unwrap_or(&true)
        } else {
            true
        };

        let aria_expanded = if is_menu {
            Some(if open { "true" } else { "false" })
        } else {
            None
        };

        Container::new()
            .tag("a")
            .attribute("role", "link")
            .attribute("aria-expanded", aria_expanded)
            .attribute("disabled", (!visible).then(|| "true"))
            .attribute("tabindex",  if is_active { "0" } else { "-1" })
            .class("pwt-nav-link")
            .class(if visible { "pwt-d-flex" } else { "pwt-d-none" })
            .class(crate::css::AlignItems::Baseline)
            .class(is_active.then_some("active"))
            .onclick(onclick)
            .onkeydown(onkeydown)
            .with_child(
                (0..indent_level)
                    .map(|_| html! { <span class="pwt-ps-4" /> })
                    .collect::<Html>(),
            )
            .with_optional_child(item.icon_class.as_ref().and_then(|icon| {
                Some(html!{ <i class={classes!(icon.to_string(), "pwt-me-2")}/>})
            }))
            .with_child(html! {<div class="pwt-text-truncate pwt-flex-fill">{&item.text}</div>})
            .with_optional_child(is_menu.then(|| {
                Container::new()
                    .tag("i")
                    .attribute("aria-hidden", "true")
                    .class("fa fa-caret-down")
                    .class("pwt-nav-menu-item-arrow")
                    .class(open.then_some("expanded"))
                    .with_child("\u{00a0}")
            }))
            .into()
    }

    fn render_item(
        &self,
        ctx: &yew::Context<PwtNavigationMenu>,
        item: &Menu,
        menu: &mut Column,
        active: &str,
        level: usize,
        visible: bool,
    ) -> Option<(AttrValue, Html)> {
        let mut content = None;
        match item {
            Menu::Item(child) => {
                menu.add_child(self.render_child(ctx, child, active, level, false, visible));
                if child.id.deref() == active {
                    content = Some((child.text.clone(), child.content.apply(&child.id)));
                }
            }
            Menu::SubMenu(SubMenu {
                item,
                children,
                selectable: _selectable,
            }) => {
                menu.add_child(self.render_child(ctx, item, active, level, true, visible));
                if item.id.deref() == active {
                    content = Some((item.text.clone(), item.content.apply(&item.id)));
                }
                let visible = visible
                    .then(|| *self.menu_states.get(&item.id).unwrap_or(&true))
                    .unwrap_or(false);
                for sub in children.iter() {
                    if let Some(new_content) =
                        self.render_item(ctx, sub, menu, active, level + 1, visible)
                    {
                        content = Some(new_content);
                    }
                }
            }
            Menu::Component(comp) => {
                menu.add_child(comp.clone());
            }
        }
        content
    }

    fn get_active_or_default(&self, ctx: &Context<Self>) -> Option<Key> {
        let props = ctx.props();
        if let Some(active) = self.active.as_deref() {
            if !active.is_empty() && active != "_" {
                return self.active.clone();
            }
        }
        if props.default_active.is_some() {
            return props.default_active.clone();
        }
        for menu in props.menu.iter() {
            match menu {
                Menu::Item(item) => return Some(item.id.clone()),
                Menu::SubMenu(sub) => return Some(sub.item.id.clone()),
                _ => {}
            }
        }
        None
    }

    fn find_selectable_key(&mut self, ctx: &Context<Self>, desired: Key) -> Option<Key> {
        let props = ctx.props();

        fn find_first_key_recursive(menu: &[Menu]) -> Option<Key> {
            for menu in menu.iter() {
                let res = match menu {
                    Menu::Item(item) => Some(item.id.clone()),
                    Menu::SubMenu(sub) => {
                        if sub.selectable {
                            Some(sub.item.id.clone())
                        } else {
                            find_first_key_recursive(&sub.children[..])
                        }
                    }
                    _ => None,
                };
                if res.is_some() {
                    return res;
                }
            }
            None
        }

        fn find_item_recursive<'a, 'b>(menu: &'a [Menu], desired: &'b Key) -> Option<&'a Menu> {
            for menu in menu.iter() {
                match menu {
                    Menu::Item(item) => {
                        if &item.id == desired {
                            return Some(menu);
                        }
                    }
                    Menu::SubMenu(sub) => {
                        if &sub.item.id == desired {
                            return Some(menu);
                        } else {
                            let res = find_item_recursive(&sub.children[..], desired);
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

        match find_item_recursive(&props.menu, &desired) {
            Some(Menu::Item(_)) => Some(desired),
            Some(Menu::SubMenu(sub)) => {
                if sub.selectable {
                    Some(desired)
                } else {
                    self.menu_states.insert(desired, true);
                    find_first_key_recursive(&sub.children)
                }
            }
            _ => None,
        }
    }
}

impl Component for PwtNavigationMenu {
    type Message = Msg;
    type Properties = NavigationMenu;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let props = ctx.props();
        let mut active = None;
        let mut _nav_ctx_handle = None;

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
                active = Some(Key::from(path));
            }
        }

        Self {
            active,
            menu_states: HashMap::new(),
            menu_ref: NodeRef::default(),
            _nav_ctx_handle,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Select(key, update_route, toggle) => {
                if key == self.active {
                    if let Some(key) = key {
                        if toggle {
                            let entry = *self.menu_states.entry(key.clone()).or_insert_with(|| true);
                            self.menu_states.insert(key, !entry);
                        }
                    }
                    return true;
                }

                let key = if let Some(key) = key {
                    self.find_selectable_key(ctx, key)
                } else {
                    None
                };

                self.active = key.clone();

                if props.router && update_route {
                    ctx.link().push_relative_route(key.as_deref().unwrap_or(""));
                }

                if let Some(on_select) = &props.on_select {
                    on_select.emit(key);
                }
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

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let mut content: Option<Html> = None;

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
        let mut menu = Column::new()
            .class("pwt-fit")
            .node_ref(self.menu_ref.clone())
            .onkeydown(onkeydown)
            // avoid https://bugzilla.mozilla.org/show_bug.cgi?id=1069739
            .attribute("tabindex", "-1")
            .attribute("role", "navigation")
            .attribute("aria-label", props.aria_label.clone())
            // fixme: ???
            //.margin(props.std_props.margin.clone())
            //.padding(props.std_props.padding.clone())
            .class("pwt-nav-menu pwt-overflow-none");

        let active = self.get_active_or_default(ctx);
        let active = active.as_deref().unwrap_or("");

        for item in props.menu.iter() {
            if let Some((title, new_content)) =
                self.render_item(ctx, item, &mut menu, active, 0, true)
            {
                content = Some(html! {
                    <div role="main" aria-label={title} class="pwt-fit pwt-d-flex">{new_content}</div>
                })
            }
        }

        SplitPane::new()
            .node_ref(props.node_ref.clone())
            .class("pwt-flex-fill pwt-overflow-auto")
            .class(props.std_props.class.clone())
            .with_child(Pane::new(menu).size(None))
            .with_child(Pane::new(content.unwrap_or(html!{})).flex(1))
            .into()
    }
}
