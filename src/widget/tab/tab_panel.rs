use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VList, VNode};

use crate::prelude::*;
use crate::state::{NavigationContainer, Selection};
use crate::widget::{
    Column, IntoOptionalMiniScrollMode, MiniScroll, MiniScrollMode, Row, SelectionView,
    SelectionViewRenderInfo, TabBar, TabBarItem,
};

/// A set of layered items where only one item is displayed at a time.
///
/// [TabPanel]s (like [crate::widget::Panel]s]) may have a title and tool buttons.
///
/// Panel item are either static or dynamic. Static items are rendered
/// once before you add them. Dynamic items use a render function
/// which is called when the item gets activated the first time. After
/// that the render function is called whenever another panel gets
/// activated.
#[derive(Clone, PartialEq, Properties)]
pub struct TabPanel {
    pub key: Option<Key>,

    selection: Selection,

    //#[prop_or_default]
    //pub tabs: IndexMap<Key, RenderFn<SelectionViewRenderInfo>>,
    pub view: SelectionView,

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
        let selection = Selection::new();
        let view = SelectionView::new(selection.clone()).class("pwt-fit");
        yew::props!(TabPanel {
            selection,
            view,
        })
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
        NavigationContainer::new().with_child(self)
    }

    /// Builder style method to add a static tab panel.
    pub fn with_item(
        mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        panel: impl Into<VNode>,
    ) -> Self {
        self.add_item(key, label, icon_class, panel);
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

        self.add_item_builder(key, label, icon_class, move |_| html.clone())
    }

    /// Builder style method to add a dynamic tab panel.
    pub fn with_item_builder(
        mut self,
        key: impl Into<Key>,
        label: impl Into<AttrValue>,
        icon_class: impl IntoPropValue<Option<AttrValue>>,
        renderer: impl 'static + Fn(&SelectionViewRenderInfo) -> Html,
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
        renderer: impl 'static + Fn(&SelectionViewRenderInfo) -> Html,
    ) {
        let key = key.into();
        let mut item = TabBarItem::new().key(key.clone()).label(label.into());

        if let Some(icon_class) = icon_class.into_prop_value() {
            item.set_icon_class(icon_class);
        }

        self.bar.add_item(item);
        self.view.add_builder(key, renderer);
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

#[doc(hidden)]
pub struct PwtTabPanel {}

impl Component for PwtTabPanel {
    type Message = ();
    type Properties = TabPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut bar: Html = props
            .bar
            .clone()
            .selection(props.selection.clone())
            .into();

        if let Some(scroll_mode) = props.scroll_mode {
            bar = MiniScroll::new(bar)
                .scroll_mode(scroll_mode)
                .class("pwt-flex-fill")
                .into();
        }

        let content = props.view.clone();

        let header;

        if props.title.is_some() {
            let title =
                crate::widget::panel::create_panel_title(props.title.clone(), props.tools.clone())
                    .class("pwt-pb-2");
            header = html! {<div class="pwt-panel-header">{title}{bar}</div>};
        } else {
            header = Row::new()
                .class("pwt-panel-header pwt-align-items-center pwt-gap-2")
                .with_child(bar)
                .with_optional_child(
                    (!props.tools.is_empty())
                        .then(|| VList::with_children(props.tools.clone(), None)),
                )
                .into();
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
