use std::rc::Rc;
use std::collections::HashSet;

use indexmap::IndexMap;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::props::RenderFn;
use crate::state::NavigationContainer;
use super::{Column, TabBar};

#[derive(Clone, PartialEq, Properties)]
pub struct TabPanel {
    pub key: Option<Key>,
    #[prop_or_default]
    pub tabs: IndexMap<Key, RenderFn<()>>,
    #[prop_or_default]
    pub bar: TabBar,

    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub tools: Vec<VNode>,

    #[prop_or_default]
    pub class: Classes,
}

impl TabPanel {

    pub fn new() -> Self {
        yew::props!(TabPanel {})
    }

   pub fn title(mut self, title: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: impl IntoPropValue<Option<AttrValue>>) {
        self.title = title.into_prop_value();
    }

    pub fn tool(mut self, tool: impl Into<VNode>) -> Self {
        self.add_tool(tool);
        self
    }

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

    pub fn navigation_container(mut self) -> NavigationContainer {
        self.bar.set_router(true);
        NavigationContainer::new()
            .with_child(self)
    }

    pub fn with_item(
        self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        panel: impl Into<VNode>,
    ) -> Self {
        let html = panel.into();

        self.with_item_builder(
            key,
            label,
            icon_class,
            move |_| html.clone(),
        )
    }

    pub fn with_item_builder(
        mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        renderer: impl 'static + Fn(&()) -> Html,
    ) -> Self {
        let key = key.into();

        self.bar.add_item(key.clone(), label, icon_class);

        self.tabs.insert(key, RenderFn::new(renderer));

        self
    }

}

pub enum Msg {
    Select(Option<Key>),
}

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

        let bar = props.bar.clone()
            .class("pwt-scheme-neutral-container")
            .on_select(ctx.link().callback(|key| Msg::Select(key))) ;

        let content: Html = props.tabs.iter().map(|(key, render_fn)| {
            let active = match &self.active {
                Some(active_key) => active_key == key,
                None => false,
            };

            let panel_html = if self.render_set.contains(key) {
                render_fn.apply(&())
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

        if props.title.is_some() || !props.tools.is_empty() {

            let title = super::panel::create_panel_title(props.title.clone(), props.tools.clone())
                .class("pwt-pb-2");
            header = html!{<div class="pwt-panel-header">{title}{bar}</div>};
        } else {
            header = bar.class("pwt-panel-header").into();
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
