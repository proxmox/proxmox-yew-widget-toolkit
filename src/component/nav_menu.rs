use std::rc::Rc;
use std::collections::HashMap;
use std::ops::Deref;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::{html, Component, Html, Properties};
use yew::html::{IntoPropValue, IntoEventCallback};

use crate::props::{ContainerBuilder, EventSubscriber, RenderFn, WidgetBuilder};
use crate::state::{NavigationContainer, NavigationContext, NavigationContextExt};

use crate::widget::focus::focus_next_tabable;
use crate::widget::{Column, Row};

#[derive(Clone, PartialEq)]
pub struct MenuItem {
    id: Key,
    text: AttrValue,
    icon_cls: Option<AttrValue>,
    content: RenderFn<Key>,
}

impl MenuItem {
    pub fn new(
        id: impl Into<Key>,
        text: impl IntoPropValue<AttrValue>,
        icon_cls: impl IntoPropValue<Option<AttrValue>>,
        content: impl Fn(&Key) -> Html + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into_prop_value(),
            icon_cls: icon_cls.into_prop_value(),
            content: RenderFn::new(content),
        }
    }

    pub fn submenu(self) -> SubMenu {
        SubMenu {
            item: self,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SubMenu {
    item: MenuItem,
    children: Vec<Menu>,
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

#[derive(PartialEq, Clone, Properties)]
pub struct NavigationMenu {
    pub key: Option<Key>,
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

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn navigation_container(mut self) -> NavigationContainer {
        self.router = true;
        NavigationContainer::new()
            .with_child(self)
    }

    pub fn default_active(mut self, active:  impl IntoPropValue<Option<Key>>) -> Self {
        self.default_active = active.into_prop_value();
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
    Select(Option<Key>, bool),
    MenuToggle(Key),
    MenuClose(Key),
    MenuOpen(Key),
}

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

        let class = classes!(is_active.then(|| "active"), "pwt-nav-link",);

        let onclick = ctx.link().callback({
            let key = item.id.clone();
            move |_event: MouseEvent| Msg::Select(Some(key.clone()), true)
        });

        let on_expander_click = ctx.link().callback({
            let key = item.id.clone();
            move |event: MouseEvent| {
                event.stop_propagation();
                Msg::MenuToggle(key.clone())
            }
        });

        let onkeydown = ctx.link().batch_callback({
            let key = item.id.clone();
            move |event: KeyboardEvent| match event.key().as_str() {
                " " => Some(Msg::Select(Some(key.clone()), true)),
                "ArrowRight" if is_menu => Some(Msg::MenuOpen(key.clone())),
                "ArrowLeft" if is_menu => Some(Msg::MenuClose(key.clone())),
                _ => None,
            }
        });

        let tabindex = if is_active { "0" } else { "-1" };

        let open = if is_menu {
            *self.menu_states.get(&item.id).unwrap_or(&true)
        } else {
            true
        };
        let style = (!visible).then(|| "display:none").unwrap_or("");

        html! {
            <a disabled={!visible} {style} {onclick} {onkeydown} {class} {tabindex}>
            { (0..indent_level).map(|_| html!{ <span class="pwt-ps-4" /> }).collect::<Html>() }
                if let Some(icon) = &item.icon_cls {
                    <i class={classes!(icon.to_string(), "pwt-me-2")}/>
                }
            {&item.text}
            if is_menu {
                <i class={classes!{
                        "fa",
                        "fa-fw",
                        if open { "fa-caret-up" } else { "fa-caret-down" },
                        "pwt-nav-menu-expander"
                    }}
                    onclick={on_expander_click}>{"\u{00a0}"}</i>
            }
            </a>
        }
    }

    fn render_item(
        &self,
        ctx: &yew::Context<PwtNavigationMenu>,
        item: &Menu,
        menu: &mut Column,
        active: &str,
        level: usize,
        visible: bool,
    ) -> Option<Html> {
        let mut content = None;
        match item {
            Menu::Item(child) => {
                menu.add_child(self.render_child(ctx, child, active, level, false, visible));
                if child.id.deref() == active {
                    content = Some(child.content.apply(&child.id));
                }
            }
            Menu::SubMenu(SubMenu { item, children }) => {
                menu.add_child(self.render_child(ctx, item, active, level, true, visible));
                if item.id.deref() == active {
                    content = Some(item.content.apply(&item.id));
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
            return props.default_active.clone()
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
                    link.send_message(Msg::Select(key, false));
                }
            });
            if let Some((nav_ctx, handle)) = ctx.link().context::<NavigationContext>(on_nav_ctx_change) {
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
            Msg::Select(key, update_route) => {
                if key == self.active { return false; }

                self.active = key.clone();

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

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let mut content: Option<Html> = None;

        let menu_ref = self.menu_ref.clone();
        let onkeydown = Callback::from(move |event: KeyboardEvent| {
            match event.key().as_str() {
                "ArrowDown" => {
                    focus_next_tabable(&menu_ref, false, false);
                }
                "ArrowUp" => {
                    focus_next_tabable(&menu_ref, true, false);
                }
                _ => return,
            }
            event.prevent_default();
        });
        let mut menu = Column::new()
            .node_ref(self.menu_ref.clone())
            .onkeydown(onkeydown)
            .class("pwt-nav-menu pwt-overflow-auto pwt-border-right")
            .attribute("style", "min-width:200px;");


        let active = self.get_active_or_default(ctx);
        let active = active.as_deref().unwrap_or("");

        for item in props.menu.iter() {
            if let Some(new_content) = self.render_item(ctx, item, &mut menu, active, 0, true) {
                content = Some(new_content);
            }
        }

        Row::new()
            .class("pwt-flex-fill pwt-align-items-stretch pwt-overflow-auto")
            .with_child(menu)
            .with_optional_child(content)
            .into()
    }
}

impl Into<VNode> for NavigationMenu {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtNavigationMenu>(Rc::new(self), key);
        VNode::from(comp)
    }
}
