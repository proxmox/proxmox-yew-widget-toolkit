use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VList, VNode};

use crate::prelude::*;
use crate::props::{IntoOptionalInlineHtml, IntoStorageLocation, StorageLocation};
use crate::state::Selection;
use crate::widget::{
    Column, MiniScroll, MiniScrollMode, SelectionView, SelectionViewRenderInfo, TabBar, TabBarItem,
    TabBarStyle,
};

use pwt_macros::{builder, widget};

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
/// # use pwt::prelude::*;
/// use pwt::state::NavigationContainer;
/// use pwt::widget::{TabPanel, TabBarItem};
///
/// NavigationContainer::new()
///     .with_child(
///         TabPanel::new()
///             .router(true)
///             .with_item(
///                 TabBarItem::new().key("item1").label("Item 1"),
///                 html!{"This is Item 1."}
///             )
///             .with_item(
///                 TabBarItem::new().key("item2").label("Item 2"),
///                 html!{"This is Item 2."}
///             )
///     );
/// ```
#[widget(pwt=crate, comp=PwtTabPanel, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TabPanel {
    /// The content view
    view: SelectionView,

    /// The [TabBar].
    #[prop_or_default]
    pub bar: TabBar,

    /// Panel title text.
    #[prop_or_default]
    pub title: Option<Html>,

    /// Tools, displayed right aligned in the header.
    #[prop_or_default]
    pub tools: Vec<VNode>,

    /// Store current state (selected item).
    #[prop_or_default]
    pub state_id: Option<StorageLocation>,

    /// Use [MiniScroll] for [TabBar] to allow scrolling.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub scroll_mode: Option<MiniScrollMode>,

    /// The [Selection] of the TabPanel. Makes it possible to control from outside
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub selection: Option<Selection>,

    /// The [TabBarStyle]
    #[prop_or_default]
    #[builder]
    pub tab_bar_style: TabBarStyle,
}

impl Default for TabPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TabPanel {
    /// Creates a new instance.
    pub fn new() -> Self {
        let view = SelectionView::new()
            .page_cache(true)
            .class("pwt-flex-fill pwt-overflow-auto");
        yew::props!(TabPanel { view })
    }

    /// Builder style method to set the persistent state ID.
    pub fn state_id(mut self, state_id: impl IntoStorageLocation) -> Self {
        self.set_state_id(state_id);
        self
    }

    /// Method to set the persistent state ID.
    pub fn set_state_id(&mut self, state_id: impl IntoStorageLocation) {
        self.state_id = state_id.into_storage_location();
    }

    /// Builder style method to set the title text.
    pub fn title(mut self, title: impl IntoOptionalInlineHtml) -> Self {
        self.set_title(title);
        self
    }

    /// Method to set the title text.
    pub fn set_title(&mut self, title: impl IntoOptionalInlineHtml) {
        self.title = title.into_optional_inline_html();
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

    fn create(ctx: &Context<Self>) -> Self {
        let selection = match ctx.props().selection.clone() {
            Some(selection) => selection,
            None => Selection::new(),
        };
        Self { selection }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut bar: Html = props
            .bar
            .clone()
            .selection(self.selection.clone())
            .style(props.tab_bar_style)
            .state_id(props.state_id.clone())
            .into();

        if let Some(scroll_mode) = props.scroll_mode {
            bar = MiniScroll::new(bar)
                .scroll_mode(scroll_mode)
                .class("pwt-flex-fill")
                .into();
        }

        let material_style = props.tab_bar_style != TabBarStyle::Pills;

        let mut class = classes!((!material_style).then_some("pwt-panel-header"));
        let (title, tools) = if props.title.is_some() {
            let title =
                crate::widget::panel::create_panel_title(props.title.clone(), props.tools.clone())
                    .padding_bottom(2)
                    .class(material_style.then_some("pwt-panel-header"));
            (Some(title), None)
        } else {
            class = classes!("pwt-d-flex", "pwt-align-items-center", "pwt-gap-2", class);
            let tools =
                (!props.tools.is_empty()).then(|| VList::with_children(props.tools.clone(), None));
            (None, tools)
        };

        Column::new()
            .with_std_props(props.as_std_props())
            .class("pwt-panel")
            .with_child(html! {<div {class}>{title}{bar}{tools}</div>})
            .with_child(props.view.clone().selection(self.selection.clone()))
            .into()
    }
}
