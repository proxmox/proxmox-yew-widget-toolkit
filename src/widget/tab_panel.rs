use std::rc::Rc;
use std::collections::HashSet;

use indexmap::IndexMap;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VList, VNode};
use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::props::RenderFn;
use crate::state::NavigationContainer;
use super::{Column, IntoOptionalMiniScrollMode, MiniScroll, Row, TabBar, MiniScrollMode};

/// Infos passed to the [TabPanel] render function.
pub struct TabPanelRenderInfo {
    /// The key of the item to render
    pub key: Key,
    /// Set if this item is visible/active.
    ///
    /// So that the item can react on visibility changes.
    pub visible: bool,
}

/// A set of layered items where only one item is displayed at a time.
///
/// [TabPanel]s (like [super::Panel]s]) may have a title and tool buttons.
///
/// Panel item are either static or dynamic. Static items are rendered
/// once before you add them. Dynamic items use a render function
/// which is called when the item gets activated the first time. After
/// that the render function is called whenever another panel gets
/// activated.
#[derive(Clone, PartialEq, Properties)]
pub struct TabPanel {
    pub key: Option<Key>,
    #[prop_or_default]
    pub tabs: IndexMap<Key, RenderFn<TabPanelRenderInfo>>,
    #[prop_or_default]
    pub bar: TabBar,

    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub tools: Vec<VNode>,

    #[prop_or_default]
    pub class: Classes,

    /// Use [MiniScroll] for [TabBar] to allow scrolling.
    #[prop_or_default]
    pub scroll_mode: Option<MiniScrollMode>,
}

impl TabPanel {

    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(TabPanel {})
    }

    /// Builder style method to set the `title` property
    pub fn title(mut self, title: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_title(title);
        self
    }

    /// Method to set the `title` property
    pub fn set_title(&mut self, title: impl IntoPropValue<Option<AttrValue>>) {
        self.title = title.into_prop_value();
    }

    /// Builder style method to add a tool
    pub fn tool(mut self, tool: impl Into<VNode>) -> Self {
        self.add_tool(tool);
        self
    }

    /// Method to add a tool
    pub fn add_tool(&mut self, tool: impl Into<VNode>) {
        self.tools.push(tool.into());
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
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

    /// Embed the [TabPanel] into a [NavigationContainer]
    pub fn navigation_container(mut self) -> NavigationContainer {
        self.bar.set_router(true);
        NavigationContainer::new()
            .with_child(self)
    }

    /// Builder style method to add a static tab panel.
    pub fn with_item(
        mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        panel: impl Into<VNode>,
    ) -> Self {
        self.add_item(
            key,
            label,
            icon_class,
            panel,
        );
        self
    }

    /// Method to add a static tab panel.
    pub fn add_item(
        &mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        panel: impl Into<VNode>,
    ) {
        let html = panel.into();

        self.add_item_builder(
            key,
            label,
            icon_class,
            move |_| html.clone(),
        )
    }

    /// Builder style method to add a dynamic tab panel.
    pub fn with_item_builder(
        mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        renderer: impl 'static + Fn(&TabPanelRenderInfo) -> Html,
    ) -> Self {
        self.add_item_builder(key, label, icon_class, renderer);
        self
    }

    /// Method to add a dynamic tab panel.
    pub fn add_item_builder(
        &mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        renderer: impl 'static + Fn(&TabPanelRenderInfo) -> Html,
    ) {
        let key = key.into();
        self.bar.add_item(key.clone(), label, icon_class);
        self.tabs.insert(key, RenderFn::new(renderer));
    }

    /// Builder style method to set the scroll mode.
    pub fn scroll_mode(mut self, scroll_mode: impl IntoOptionalMiniScrollMode) -> Self {
        self.set_scroll_mode(scroll_mode);
        self
    }

    /// Method to set the scroll mode.
    pub fn set_scroll_mode(&mut self, scroll_mode: impl IntoOptionalMiniScrollMode) {
        self.scroll_mode = scroll_mode.into_optional_mini_scroll_mode();
    }
}

pub enum Msg {
    Select(Option<Key>),
}

#[doc(hidden)]
pub struct PwtTabPanel {
    active: Option<Key>,
    render_set: HashSet<Key>,
}

impl Component for PwtTabPanel {
    type Message = Msg;
    type Properties = TabPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            active: None,
            render_set: HashSet::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Select(opt_key) => {
                self.active = opt_key.clone();
                if let Some(key) = opt_key {
                    self.render_set.insert(key);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut bar: Html = props.bar.clone()
            .on_select(ctx.link().callback(|key| Msg::Select(key)))
            .into();

        if let Some(scroll_mode) = props.scroll_mode {
            bar = MiniScroll::new(bar).scroll_mode(scroll_mode).into();
        }

        let content: Html = props.tabs.iter().map(|(key, render_fn)| {
            let active = match &self.active {
                Some(active_key) => active_key == key,
                None => false,
            };

            let panel_html = if self.render_set.contains(key) {
                render_fn.apply(&TabPanelRenderInfo{
                    key: key.clone(),
                    visible: active,
                })
            } else {
                html!{}
            };

            if active {
                html!{ <div key={key.clone()} class="pwt-flex-fill pwt-overflow-auto">{panel_html} </div>}
           } else {
                html!{ <div key={key.clone()} class="pwt-d-none">{panel_html}</div>}
            }
        }).collect();

        let header;

        if props.title.is_some() {
            let title = super::panel::create_panel_title(props.title.clone(), props.tools.clone())
                .class("pwt-pb-2");
            header = html!{<div class="pwt-panel-header">{title}{bar}</div>};
        } else {
            if !props.tools.is_empty() {
                header = Row::new()
                    .class("pwt-panel-header pwt-align-items-center pwt-gap-1")
                    .with_child(bar)
                    .with_flex_spacer()
                    .with_child(VList::with_children(props.tools.clone(), None))
                    .into()
            } else {
                header = bar;
            }
        };

        Column::new()
            .class("pwt-panel")
            .class(props.class.clone())
            .with_child(header)
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for TabPanel {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtTabPanel>(Rc::new(self), key);
        VNode::from(comp)
    }
}
