use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VList, VNode};

use crate::prelude::*;
use crate::state::Selection;
use crate::widget::{
    Column, MiniScroll, MiniScrollMode, Row, SelectionView, SelectionViewRenderInfo, TabBar,
    TabBarItem,
};

use pwt_macros::builder;

/// A set of layered items where only one item is displayed at a time.
///
/// TabPanels (like [Panel](crate::widget::Panel)]) may have a title and tool buttons.
///
/// Panel item are either static or dynamic. Static items are rendered
/// once before you add them. Dynamic items use a render function
/// which is called when the item gets activated the first time. After
/// that the render function is called whenever another panel gets
/// activated.
///
/// # Automatic routing.
///
/// TabPanels support fully automatic routing if you put the panel inside
/// a [NavigationContainer](crate::state::NavigationContainer) and
/// set the router flag,
///
/// ```
/// use pwt::state::NavigationContainer;
/// use pwt::widget::TabPanel;
///
/// NavigationContainer::new()
///     .with_child(
///         TabPanel::new()
///             .router(true)
///             .add_item(
///                 TabBarItem::new().key("item1").label("Item 1"),
///                 html!{"This is Item 1."}
///             )
///             .add_item(
///                 TabBarItem::new().key("item2").label("Item 2"),
///                 html!{"This is Item 2."}
///             )
///     );
/// ```
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TabPanel {
    /// The yew component key.
    pub key: Option<Key>,

    /// The content view
    view: SelectionView,

    /// The [TabBar].
    #[prop_or_default]
    pub bar: TabBar,

    /// Panel title text.
    #[builder(IntoPropValue, into_prop_value)]
    pub title: Option<AttrValue>,

    /// Tools, displayed right aligned in the header.
    #[prop_or_default]
    pub tools: Vec<VNode>,

    /// CSS class.
    #[prop_or_default]
    pub class: Classes,

    /// Use [MiniScroll] for [TabBar] to allow scrolling.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub scroll_mode: Option<MiniScrollMode>,
}

impl TabPanel {
    /// Creates a new instance.
    pub fn new() -> Self {
        let view = SelectionView::new()
            .page_cache(true)
            .class("pwt-fit");
        yew::props!(TabPanel { view })
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

    // Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property.
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
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

    /// Builder style method to enable router functionality.
    ///
    /// Save/Load state from parent NavigationContainer
    pub fn router(mut self, router: bool) -> Self {
        self.set_router(router);
        self
    }

    /// Method to enable router functionality.
    pub fn set_router(&mut self, router: bool) {
        self.bar.set_router(router);
    }

    /// Builder style method to add a static tab panel.
    pub fn with_item(mut self, item: impl Into<TabBarItem>, view: impl Into<VNode>) -> Self {
        self.add_item(item, view);
        self
    }

    /// Method to add a static tab panel.
    pub fn add_item(&mut self, item: impl Into<TabBarItem>, view: impl Into<VNode>) {
        let html = view.into();
        self.add_item_builder(item, move |_| html.clone())
    }

    /// Builder style method to add a dynamic tab panel.
    pub fn with_item_builder(
        mut self,
        item: impl Into<TabBarItem>,
        renderer: impl 'static + Fn(&SelectionViewRenderInfo) -> Html,
    ) -> Self {
        self.add_item_builder(item, renderer);
        self
    }

    /// Method to add a dynamic tab panel.
    pub fn add_item_builder(
        &mut self,
        item: impl Into<TabBarItem>,
        renderer: impl 'static + Fn(&SelectionViewRenderInfo) -> Html,
    ) {
        let mut item = item.into();

        if item.key.is_none() {
            item.key = Some(Key::from(format!(
                "__tab_panel_item{}",
                self.bar.tabs.len()
            )));
        }

        self.view.add_builder(item.key.clone().unwrap(), renderer);
        self.bar.add_item(item);
    }
}

#[doc(hidden)]
pub struct PwtTabPanel {
    selection: Selection,
}

impl Component for PwtTabPanel {
    type Message = ();
    type Properties = TabPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            selection: Selection::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut bar: Html = props.bar.clone().selection(self.selection.clone()).into();

        if let Some(scroll_mode) = props.scroll_mode {
            bar = MiniScroll::new(bar)
                .scroll_mode(scroll_mode)
                .class("pwt-flex-fill")
                .into();
        }

        let content = props.view.clone().selection(self.selection.clone());

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
