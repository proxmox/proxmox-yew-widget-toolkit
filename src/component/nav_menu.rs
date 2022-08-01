use std::rc::Rc;
use std::collections::HashMap;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::{html, Component, Html, Properties};
use yew::html::IntoPropValue;

use crate::props::{ContainerBuilder, EventSubscriber, RenderFn, WidgetBuilder};
use crate::widget::focus::focus_next_tabable;
use crate::widget::{Column, Row};

#[derive(Clone, PartialEq)]
pub struct MenuItem {
    id: AttrValue,
    text: AttrValue,
    icon_cls: Option<AttrValue>,
    content: RenderFn<AttrValue>,
}

impl MenuItem {
    pub fn new(
        id: impl IntoPropValue<AttrValue>,
        text: impl IntoPropValue<AttrValue>,
        icon_cls: impl IntoPropValue<Option<AttrValue>>,
        content: impl Fn(&AttrValue) -> Html + 'static,
    ) -> Self {
        Self {
            id: id.into_prop_value(),
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
    menu: Vec<Menu>,
    default_active: Option<AttrValue>,
    on_select: Callback<Option<AttrValue>>,
}

impl NavigationMenu {
    pub fn new() -> Self {
        Self {
            menu: Vec::new(),
            on_select: Callback::noop(),
            default_active: None,
        }
    }

    pub fn default_active(mut self, active:  impl IntoPropValue<Option<AttrValue>>) -> Self {
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

    pub fn on_select(mut self, callback: Callback<Option<AttrValue>>) -> Self {
        self.on_select = callback;
        self
    }
}

pub enum Msg {
    Select(Option<AttrValue>),
    MenuToggle(AttrValue),
    MenuClose(AttrValue),
    MenuOpen(AttrValue),
}

pub struct PwtNavigationMenu {
    active: Option<AttrValue>,
    menu_states: HashMap<AttrValue, bool>, // true = open
    menu_ref: NodeRef,
}

impl PwtNavigationMenu {
    fn render_child(
        &self,
        ctx: &yew::Context<PwtNavigationMenu>,
        item: &MenuItem,
        indent_level: usize,
        is_menu: bool,
        visible: bool,
    ) -> Html {
        let is_active = self
            .active
            .as_ref()
            .map_or(false, |active| active == &item.id);

        let class = classes!(is_active.then(|| "active"), "pwt-nav-link",);

        let onclick = ctx.link().callback({
            let key = item.id.clone();
            move |_event: MouseEvent| Msg::Select(Some(key.clone()))
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
                " " => Some(Msg::Select(Some(key.clone()))),
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
                menu.add_child(self.render_child(ctx, child, level, false, visible));
                if child.id == active {
                    content = Some(child.content.apply(&child.id));
                }
            }
            Menu::SubMenu(SubMenu { item, children }) => {
                menu.add_child(self.render_child(ctx, item, level, true, visible));
                if item.id == active {
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
}

impl Component for PwtNavigationMenu {
    type Message = Msg;
    type Properties = NavigationMenu;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            active: ctx.props().default_active.clone(),
            menu_states: HashMap::new(),
            menu_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Select(key) => {
                self.active = key.clone();
                ctx.props().on_select.emit(key);
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

        let active = self.active.as_deref().unwrap_or("");
        for item in ctx.props().menu.iter() {
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
        let comp = VComp::new::<PwtNavigationMenu>(Rc::new(self), None);
        VNode::from(comp)
    }
}
