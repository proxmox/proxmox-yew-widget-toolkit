use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::dom::{element_direction_rtl, IntoHtmlElement};
use crate::prelude::*;
use crate::props::{IntoStorageLocation, StorageLocation};
use crate::state::{NavigationContext, NavigationContextExt, PersistentState, Selection};
use crate::web_sys_ext::{ResizeObserverBoxOptions, ResizeObserverOptions};
use crate::widget::focus::roving_tabindex_next;
use crate::widget::{Container, SizeObserver};

use super::TabBarItem;

use pwt_macros::builder;

/// Tab bar
///
/// The [TabPanel](super::TabPanel) combines a [TabBar] with
/// [SelectionView](crate::widget::SelectionView) to simplify usage.
///
/// # Automatic routing.
///
/// [TabBar] supports fully automatic routing if you put the bar inside
/// a [NavigationContainer](crate::state::NavigationContainer) and
/// set the router flag.
#[derive(Clone, Default, PartialEq, Properties)]
#[builder]
pub struct TabBar {
    /// The yew node ref.
    #[prop_or_default]
    node_ref: NodeRef,

    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// CSS class.
    #[prop_or_default]
    pub class: Classes,

    /// Tab bar items.
    #[prop_or_default]
    pub tabs: Vec<TabBarItem>,

    /// Store current state (selected item).
    #[prop_or_default]
    pub state_id: Option<StorageLocation>,

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

    /// The [TabBarStyle]
    #[builder]
    #[prop_or_default]
    pub style: TabBarStyle,
}

/// Tab Bar Variants
#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum TabBarStyle {
    /// Pill/Button style tabs
    Pills,
    /// Material 3 Primary style tabs
    #[default]
    MaterialPrimary,
    /// Material 3 Secondary style tabs
    MaterialSecondary,
}

impl TabBar {
    pub fn new() -> Self {
        yew::props!(TabBar {})
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
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

    /// Builder style method to set the persistent state ID.
    pub fn state_id(mut self, state_id: impl IntoStorageLocation) -> Self {
        self.set_state_id(state_id);
        self
    }

    /// Method to set the persistent state ID.
    pub fn set_state_id(&mut self, state_id: impl IntoStorageLocation) {
        self.state_id = state_id.into_storage_location();
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

    pub fn with_item(mut self, item: impl Into<TabBarItem>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<TabBarItem>) {
        self.tabs.push(item.into());
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

    fn get_default_active(&self) -> Option<Key> {
        if self.default_active.is_some() {
            return self.default_active.clone();
        }

        for item in &self.tabs {
            if let TabBarItem { key: Some(key), .. } = item {
                return Some(key.clone());
            }
        }

        None
    }
}

pub enum Msg {
    FocusIn,
    Select(Option<Key>, bool),
    SelectionChange(Selection),
    UpdateIndicator,
}

#[doc(hidden)]
pub struct PwtTabBar {
    active: Option<Key>,
    active_cache: Option<PersistentState<String>>,
    rtl: Option<bool>,
    _nav_ctx_handle: Option<ContextHandle<NavigationContext>>,
    selection: Selection,
    indicator_ref: NodeRef,
    active_ref: NodeRef,
    size_ref: NodeRef,
    active_size_observer: Option<SizeObserver>,
}

fn get_active_or_default(props: &TabBar, active: &Option<Key>) -> Option<Key> {
    if let Some(active_key) = active.as_deref() {
        if !active_key.is_empty() && active_key != "_" {
            return active.clone();
        }
    }
    props.get_default_active()
}

impl PwtTabBar {
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

    fn update_state_cache(&mut self) {
        if let Some(active_cache) = &mut self.active_cache {
            match self.active.as_deref() {
                Some(active) => {
                    active_cache.update(active.to_owned());
                }
                None => {
                    active_cache.update(String::new());
                }
            }
        }
    }
}

impl Component for PwtTabBar {
    type Message = Msg;
    type Properties = TabBar;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let mut active = props.get_default_active();

        let active_cache = props
            .state_id
            .as_ref()
            .map(|state_id| PersistentState::<String>::new(state_id.clone()));

        if let Some(active_cache) = &active_cache {
            let last_active: &str = &*active_cache;
            if !last_active.is_empty() {
                active = Some(Key::from(last_active));
            }
        }

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

                if !(active.is_some() && (path.is_empty() || path == "_")) {
                    active = get_active_or_default(props, &Some(Key::from(path)));
                } else if let Some(active) = active.as_deref() {
                    ctx.link().push_relative_route(active);
                }
            }
        }

        let selection = Self::init_selection(ctx, props.selection.clone(), &active);

        if let Some(on_select) = &props.on_select {
            on_select.emit(active.clone());
        }
        Self {
            active,
            active_cache,
            selection,
            rtl: None,
            _nav_ctx_handle,
            indicator_ref: NodeRef::default(),
            active_ref: NodeRef::default(),
            size_ref: NodeRef::default(),
            active_size_observer: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FocusIn => {
                self.rtl = element_direction_rtl(&props.node_ref);
                true
            }
            // Handle external selection changes
            Msg::SelectionChange(selection) => {
                let key = selection.selected_key();
                let key = get_active_or_default(props, &key);

                if &self.active == &key {
                    return false;
                }

                self.active = key;
                self.update_state_cache();

                if let Some(key) = &self.active {
                    if props.router {
                        ctx.link().push_relative_route(&key);
                    }
                }
                if let Some(on_select) = &props.on_select {
                    on_select.emit(self.active.clone());
                }

                true
            }
            // Handle internal selection changes
            Msg::Select(key, update_route) => {
                let key = get_active_or_default(props, &key);
                if &self.active == &key {
                    return false;
                }

                // set active to avoid Msg::SelectionChange
                self.active = key.clone();
                self.update_state_cache();

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
            Msg::UpdateIndicator => {
                let use_full_width = match ctx.props().style {
                    TabBarStyle::Pills => {
                        return false;
                    }
                    TabBarStyle::MaterialPrimary => false,
                    TabBarStyle::MaterialSecondary => true,
                };
                let indicator = self.indicator_ref.clone().into_html_element();
                let active_el = self.active_ref.clone().into_html_element();
                let size_el = self.size_ref.clone().into_html_element();
                if let (Some(indicator), Some(active), Some(size)) = (indicator, active_el, size_el)
                {
                    let style = indicator.style();
                    let is_rtl = element_direction_rtl(size.clone()).unwrap_or_default();
                    if let Some(parent) = active.parent_element() {
                        let parent_rect = parent.get_bounding_client_rect();
                        let rect = if use_full_width {
                            active.get_bounding_client_rect()
                        } else {
                            size.get_bounding_client_rect()
                        };

                        let start = if is_rtl {
                            parent_rect.right() - rect.right()
                        } else {
                            rect.left() - parent_rect.left()
                        };
                        let width = rect.width();

                        // ignore errors
                        let _ = style.set_property("inset-inline-start", &format!("{}px", start));
                        let _ = style.set_property("width", &format!("{}px", width));
                        let _ = style.remove_property("display");
                    }
                }
                false
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

        let active = get_active_or_default(props, &self.active);
        let mut any_active = false;

        let tabs = props
            .tabs
            .iter().enumerate()
            .map(|(i, panel)| {
                let is_active = if let Some(active) = &active {
                    panel.key.as_ref() == Some(active)
                } else {
                    false
                };

                let (active_ref, size_ref) = if is_active {
                    any_active = true;
                    (self.active_ref.clone(), self.size_ref.clone())
                } else {
                    (NodeRef::default(), NodeRef::default())
                };

                let disabled = panel.disabled;

                let nav_class = classes!(
                    "pwt-nav-link",
                    is_active.then_some("active"),
                    disabled.then_some("disabled")
                );

                let onclick = if disabled {
                    None
                } else {
                    Some(ctx.link().callback({
                        let key = panel.key.clone();
                        let on_activate = panel.on_activate.clone();
                        move |_| {
                            if let Some(on_activate) = &on_activate {
                                on_activate.emit(());
                            }
                            Msg::Select(key.clone(), true)
                        }
                    }))
                };
                let onkeyup = if disabled {
                    None
                } else {
                    Some(Callback::from({
                        let link = ctx.link().clone();
                        let key = panel.key.clone();
                        let on_activate = panel.on_activate.clone();
                        move |event: KeyboardEvent| {
                            if event.key() == " " {
                                if let Some(on_activate) = &on_activate {
                                    on_activate.emit(());
                                }
                                link.send_message(Msg::Select(key.clone(), true));
                            }
                        }
                    }))
                };

                let tabindex = if is_active { "0" } else { "-1" };
                let aria_disabled = if disabled { "true" } else { "false" };
                let style = format!("grid-column: {};", i + 1);

                html! {
                    <a ref={active_ref} aria-disabled={aria_disabled} {style} {onclick} {onkeyup} class={nav_class} {tabindex}>
                        <span ref={size_ref}>
                        if let Some(class) = &panel.icon_class {
                            <span class={class.to_string()} role="none"/>
                        }
                        {panel.label.as_deref().unwrap_or("")}
                        </span>
                    </a>
                }
            })
            .collect::<Html>();

        let tabs_ref = props.node_ref.clone();
        let rtl = self.rtl.unwrap_or(false);

        let (variant_class, indicator_class) = match ctx.props().style {
            TabBarStyle::Pills => ("pwt-nav-pills", classes!()),
            TabBarStyle::MaterialPrimary => (
                "pwt-tab-material",
                classes!("pwt-tab-active-indicator", "primary"),
            ),
            TabBarStyle::MaterialSecondary => (
                "pwt-tab-material",
                classes!("pwt-tab-active-indicator", "secondary"),
            ),
        };

        let indicator = any_active.then_some({
            let indicator_ref = self.indicator_ref.clone();
            let class = indicator_class;
            let style = "display:none;";
            html! {<div ref={indicator_ref} {class} {style}></div>}
        });

        Container::new()
            .node_ref(props.node_ref.clone())
            .class(variant_class)
            .class(props.class.clone())
            .with_child(tabs)
            .with_optional_child(indicator)
            .onkeydown(move |event: KeyboardEvent| {
                match event.code().as_str() {
                    "ArrowRight" => {
                        roving_tabindex_next(&tabs_ref, rtl, false);
                    }
                    "ArrowLeft" => {
                        roving_tabindex_next(&tabs_ref, !rtl, false);
                    }
                    _ => return,
                }
                event.prevent_default();
            })
            .onfocusin(ctx.link().callback(|_| Msg::FocusIn))
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let link = ctx.link().clone();
        if let Some(element) = self.active_ref.clone().into_html_element() {
            let mut options = ResizeObserverOptions::new();
            options.box_(ResizeObserverBoxOptions::BorderBox);
            self.active_size_observer = Some(SizeObserver::new_with_options(
                &element,
                move |(_, _)| {
                    link.send_message(Msg::UpdateIndicator);
                },
                options,
            ));
        } else {
            self.active_size_observer = None;
        }
    }
}

impl Into<VNode> for TabBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtTabBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}
