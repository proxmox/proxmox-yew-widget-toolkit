use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::css::JustifyContent;
use crate::dom::ViewportQuery;
use crate::prelude::*;
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::state::{NavigationContext, NavigationContextExt, Selection};
use crate::widget::Container;

use crate::widget::TabBarItem;

use pwt_macros::builder;

/// Default media query selecting the expanded layout. Mirrors Material Design's large breakpoint,
/// where the expanded navigation rail is the recommended navigator.
const DEFAULT_EXPANDED_QUERY: &str = "(min-width: 1200px)";

/// Navigation rail
///
/// # Automatic routing.
///
/// [NavigationRail] supports fully automatic routing if you put the rail inside
/// a [NavigationContainer](crate::state::NavigationContainer) and
/// set the router flag.
///
/// # Collapsed and expanded layout.
///
/// The [expanded](Self::expanded) property selects the fixed layout. Enable
/// [auto_expand](Self::auto_expand) to follow [expanded_query](Self::expanded_query) instead.

// Note: This is similar to TabBar, but uses link semantics for primary navigation.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct NavigationRail {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Leading widget placed above the navigation group.
    #[prop_or_default]
    pub leading: Option<Html>,

    /// Accessible name for the navigation landmark.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub aria_label: Option<AttrValue>,

    #[builder]
    #[prop_or(JustifyContent::Center)]
    pub group_alignment: JustifyContent,

    /// Render the expanded layout with icon and label side by side.
    #[builder]
    #[prop_or_default]
    pub expanded: bool,

    /// Select the expanded layout through [`expanded_query`](Self::expanded_query) instead of the
    /// fixed [`expanded`](Self::expanded) property.
    #[builder]
    #[prop_or_default]
    pub auto_expand: bool,

    /// CSS media query that selects the expanded (wide) variant. Defaults to
    /// `(min-width: 1200px)`, Material Design's large breakpoint.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(AttrValue::Static(DEFAULT_EXPANDED_QUERY))]
    pub expanded_query: AttrValue,

    /// Navigation bar items.
    items: Vec<TabBarItem>,

    /// Selection object to store the currently selected tab key.
    ///
    /// The optional selction object allows you to control and observe the state from outside.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub selection: Option<Selection>,

    /// Selection callback.
    #[builder_cb(IntoEventCallback, into_event_callback, Option<Key>)]
    #[prop_or_default]
    pub on_select: Option<Callback<Option<Key>>>,

    /// Default active key.
    #[prop_or_default]
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
            if let TabBarItem {
                key: Some(key),
                disabled: false,
                ..
            } = item
            {
                return Some(key.clone());
            }
        }

        None
    }
}

pub enum Msg {
    Select(Option<Key>, bool),
    SelectionChange(Selection),
    ExpandedQueryChange(bool),
}

#[doc(hidden)]
pub struct PwtNavigationRail {
    active: Option<Key>,
    selection: Selection,
    expanded: bool,
    /// Last known match state of `expanded_query`.
    query_matches: bool,
    expanded_query: Option<ViewportQuery>,
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

    fn subscribe_expanded_query(
        ctx: &Context<Self>,
        props: &NavigationRail,
    ) -> (bool, Option<ViewportQuery>) {
        if !props.auto_expand {
            return (false, None);
        }
        ViewportQuery::subscribe(
            props.expanded_query.as_str(),
            ctx.link().callback(Msg::ExpandedQueryChange),
        )
    }

    fn set_expanded(&mut self, expanded: bool) -> bool {
        if self.expanded == expanded {
            return false;
        }
        self.expanded = expanded;
        true
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

        let (query_matches, expanded_query) = Self::subscribe_expanded_query(ctx, props);

        Self {
            selection,
            active,
            expanded: if props.auto_expand {
                query_matches
            } else {
                props.expanded
            },
            query_matches,
            expanded_query,
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

                if self.active == key {
                    return false;
                }

                self.active = key;

                if let Some(key) = &self.active {
                    if props.router {
                        ctx.link().push_relative_route(key);
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
                if self.active == key {
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
            Msg::ExpandedQueryChange(matches) => {
                self.query_matches = matches;
                if props.auto_expand {
                    self.set_expanded(matches)
                } else {
                    false
                }
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.selection != old_props.selection {
            self.selection = Self::init_selection(ctx, props.selection.clone(), &self.active);
        }
        let query_changed = props.expanded_query != old_props.expanded_query;
        let auto_expand_changed = props.auto_expand != old_props.auto_expand;
        if query_changed || auto_expand_changed {
            (self.query_matches, self.expanded_query) = Self::subscribe_expanded_query(ctx, props);
        }
        if query_changed
            || auto_expand_changed
            || (!props.auto_expand && props.expanded != old_props.expanded)
        {
            self.set_expanded(if props.auto_expand {
                self.query_matches
            } else {
                props.expanded
            });
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

            let mut badge = item.badge.as_ref().map(|badge| {
                html! { <div class="pwt-navigation-rail-badge">{badge.clone()}</div> }
            });

            let icon = match icon_class {
                Some(icon_class) => {
                    let mut icon_class = Classes::from(icon_class.to_string());
                    icon_class.push("pwt-navigation-rail-icon");

                    let class = classes!(
                        "pwt-navigation-rail-icon-container",
                        is_active.then_some("active"),
                    );
                    // the collapsed rail anchors the badge to the icon corner, the expanded
                    // variant places it after the label instead
                    let corner_badge = if self.expanded { None } else { badge.take() };
                    Some(html! {<div {class}><i role="none" class={icon_class}/>{corner_badge}</div>})
                }
                None => None,
            };
            let label = item.label.as_ref().map(|label| {
                html! {
                    <div class="pwt-navigation-rail-label">{label}</div>
                }
            });

            let (onclick, onkeydown) = if item.disabled {
                (None, None)
            } else {
                let key = item.key.clone();
                let on_activate = item.on_activate.clone();
                let onclick = ctx.link().callback(move |_| {
                    if let Some(on_activate) = &on_activate {
                        on_activate.emit(());
                    }
                    Msg::Select(key.clone(), true)
                });
                let key = item.key.clone();
                let on_activate = item.on_activate.clone();
                let link = ctx.link().clone();
                let onkeydown = Callback::from(move |event: KeyboardEvent| {
                    if crate::dom::event_key(&event) == "Enter" {
                        event.prevent_default();
                        if let Some(on_activate) = &on_activate {
                            on_activate.emit(());
                        }
                        link.send_message(Msg::Select(key.clone(), true));
                    }
                });
                (Some(onclick), Some(onkeydown))
            };

            Container::new()
                .class("pwt-navigation-rail-item")
                .class(is_active.then_some("active"))
                .class(item.disabled.then_some("disabled"))
                .attribute("role", "link")
                .attribute("tabindex", if item.disabled { "-1" } else { "0" })
                .attribute("aria-current", is_active.then_some("page"))
                .attribute("aria-disabled", item.disabled.then_some("true"))
                .with_optional_child(icon)
                .with_optional_child(label)
                .with_optional_child(badge)
                .onclick(onclick)
                .onkeydown(onkeydown)
                .into()
        });

        Container::from_tag("nav")
            .attribute(
                "aria-label",
                props
                    .aria_label
                    .clone()
                    .unwrap_or_else(|| tr!("Main Navigation").into()),
            )
            .class("pwt-navigation-rail")
            .class(self.expanded.then_some("pwt-navigation-rail-expanded"))
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

impl From<NavigationRail> for VNode {
    fn from(val: NavigationRail) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtNavigationRail>(Rc::new(val), key);
        VNode::from(comp)
    }
}
