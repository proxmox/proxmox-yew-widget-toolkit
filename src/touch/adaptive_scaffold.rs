use std::rc::Rc;

use gloo_utils::window;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::MediaQueryList;
use yew::html::IntoEventCallback;
use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};

use pwt_macros::builder;

use crate::prelude::*;
use crate::props::IntoOptionalKey;
use crate::state::Selection;
use crate::widget::TabBarItem;

use super::{NavigationBar, NavigationRail, Scaffold};

/// Default media query selecting the wider layout. Mirrors Material Design's medium breakpoint and
/// matches the value PMG, PVE and PBS dashboards use to decide between rail and bar layouts.
const DEFAULT_WIDE_QUERY: &str = "(min-width: 768px)";

/// Material-style scaffold that adapts its primary navigator to the viewport width.
///
/// Below the configured viewport breakpoint a [Scaffold] with a bottom [NavigationBar] is rendered
/// (the touch-first layout used on phones and narrow windows). Above it the layout becomes a
/// [NavigationRail] anchored to the inline-start side, with the application bar and body to its
/// right.
///
/// Both navigators are populated from the same [TabBarItem] list and share the same selection /
/// router wiring, so a single declaration drives both layouts. The active layout swaps at runtime
/// when the viewport crosses the breakpoint, and the navigation state is preserved across the swap
/// because both navigators read it from the surrounding [crate::state::NavigationContainer].
///
/// Routing works the same as on the underlying widgets: set `.router(true)` and place the
/// AdaptiveScaffold under a [crate::state::NavigationContainer] (provided e.g. by
/// [crate::touch::MaterialApp]).
///
/// # Example
///
/// ```
/// use pwt::prelude::*;
/// use pwt::touch::{AdaptiveScaffold, ApplicationBar};
/// use pwt::widget::TabBarItem;
///
/// AdaptiveScaffold::new(vec![
///     TabBarItem::new()
///         .key("home")
///         .icon_class("fa fa-home")
///         .label("Home"),
///     TabBarItem::new()
///         .key("settings")
///         .icon_class("fa fa-cog")
///         .label("Settings"),
/// ])
/// .application_bar(ApplicationBar::new().title("My App"))
/// .body(html! { <div>{"content"}</div> })
/// .router(true);
/// ```
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct AdaptiveScaffold {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Top application bar. Rendered above the body in both layouts.
    #[prop_or_default]
    pub application_bar: Option<VNode>,

    /// Primary content.
    #[prop_or_default]
    pub body: Option<VNode>,

    /// Tab items used to populate either the bottom [NavigationBar]
    /// or the side [NavigationRail].
    items: Vec<TabBarItem>,

    /// Optional FAB anchored to the bottom-end corner of the body, in
    /// both layouts.
    #[prop_or_default]
    pub favorite_action_button: Option<VNode>,

    /// Optional widget rendered above the rail tabs (rail layout only,
    /// matching [NavigationRail::leading]). A common use is a small
    /// FAB or product logo.
    #[prop_or_default]
    pub rail_leading: Option<VNode>,

    /// CSS media query that selects the wider (rail) layout. Defaults
    /// to `(min-width: 768px)`.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(AttrValue::Static(DEFAULT_WIDE_QUERY))]
    pub wide_query: AttrValue,

    /// Selection forwarded to the active navigator.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub selection: Option<Selection>,

    /// Selection callback emitted when the active tab changes.
    #[builder_cb(IntoEventCallback, into_event_callback, Option<Key>)]
    #[prop_or_default]
    pub on_select: Option<Callback<Option<Key>>>,

    /// Default active key.
    #[prop_or_default]
    pub default_active: Option<Key>,

    /// Enable router-driven activation on the navigator.
    #[builder]
    #[prop_or_default]
    pub router: bool,
}

impl AdaptiveScaffold {
    /// Create a new instance from the given tab items.
    pub fn new(items: Vec<TabBarItem>) -> Self {
        yew::props!(Self { items })
    }

    /// Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }

    /// Builder style method to set the application bar.
    pub fn application_bar(mut self, app_bar: impl Into<VNode>) -> Self {
        self.application_bar = Some(app_bar.into());
        self
    }

    /// Builder style method to set the body.
    pub fn body(mut self, body: impl Into<VNode>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Builder style method to set the favorite action button.
    pub fn favorite_action_button(mut self, fab: impl Into<VNode>) -> Self {
        self.favorite_action_button = Some(fab.into());
        self
    }

    /// Builder style method to set the rail-only leading widget.
    pub fn rail_leading(mut self, leading: impl Into<VNode>) -> Self {
        self.rail_leading = Some(leading.into());
        self
    }

    /// Builder style method to set the default active key.
    pub fn default_active(mut self, default_active: impl IntoOptionalKey) -> Self {
        self.default_active = default_active.into_optional_key();
        self
    }
}

#[doc(hidden)]
pub enum Msg {
    ViewportChanged(bool),
}

#[doc(hidden)]
pub struct PwtAdaptiveScaffold {
    is_wide: bool,
    media_query: Option<MediaQueryList>,
    listener: Option<Closure<dyn Fn()>>,
}

impl PwtAdaptiveScaffold {
    fn install_query(&mut self, ctx: &Context<Self>, query: &str) {
        self.remove_query();

        let mql = match window().match_media(query) {
            Ok(Some(m)) => m,
            _ => {
                self.is_wide = false;
                return;
            }
        };

        self.is_wide = mql.matches();

        let link = ctx.link().clone();
        let mql_clone = mql.clone();
        let closure = Closure::wrap(Box::new(move || {
            link.send_message(Msg::ViewportChanged(mql_clone.matches()));
        }) as Box<dyn Fn()>);

        let _ = mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());

        self.media_query = Some(mql);
        self.listener = Some(closure);
    }

    fn remove_query(&mut self) {
        if let (Some(mql), Some(closure)) = (self.media_query.take(), self.listener.take()) {
            let _ =
                mql.remove_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
        }
    }

    fn build_rail(props: &AdaptiveScaffold) -> NavigationRail {
        let mut rail = NavigationRail::new(props.items.clone());
        if let Some(leading) = &props.rail_leading {
            rail = rail.leading(leading.clone());
        }
        if let Some(default_active) = &props.default_active {
            rail = rail.default_active(default_active.clone());
        }
        if let Some(selection) = &props.selection {
            rail = rail.selection(selection.clone());
        }
        if let Some(on_select) = &props.on_select {
            rail = rail.on_select(on_select.clone());
        }
        rail.router(props.router)
    }

    fn build_bar(props: &AdaptiveScaffold) -> NavigationBar {
        let mut bar = NavigationBar::new(props.items.clone());
        if let Some(default_active) = &props.default_active {
            bar = bar.default_active(default_active.clone());
        }
        if let Some(selection) = &props.selection {
            bar = bar.selection(selection.clone());
        }
        if let Some(on_select) = &props.on_select {
            bar = bar.on_select(on_select.clone());
        }
        bar.router(props.router)
    }
}

impl Drop for PwtAdaptiveScaffold {
    fn drop(&mut self) {
        self.remove_query();
    }
}

impl Component for PwtAdaptiveScaffold {
    type Message = Msg;
    type Properties = AdaptiveScaffold;

    fn create(ctx: &Context<Self>) -> Self {
        let mut me = Self {
            is_wide: false,
            media_query: None,
            listener: None,
        };
        me.install_query(ctx, ctx.props().wide_query.as_str());
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ViewportChanged(matches) => {
                if self.is_wide == matches {
                    return false;
                }
                self.is_wide = matches;
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if ctx.props().wide_query != old_props.wide_query {
            self.install_query(ctx, ctx.props().wide_query.as_str());
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut scaffold = Scaffold::new();
        if let Some(app_bar) = &props.application_bar {
            scaffold = scaffold.application_bar(app_bar.clone());
        }
        if let Some(body) = &props.body {
            scaffold = scaffold.body(body.clone());
        }
        if let Some(fab) = &props.favorite_action_button {
            scaffold = scaffold.favorite_action_button(fab.clone());
        }

        // Drive both form factors through Scaffold; only the navigator slot differs, and the rail
        // versus bar layout switch lives in Scaffold itself.
        scaffold = if self.is_wide {
            scaffold.navigation_rail(Self::build_rail(props))
        } else {
            scaffold.navigation_bar(Self::build_bar(props))
        };

        scaffold.into()
    }
}

impl From<AdaptiveScaffold> for VNode {
    fn from(val: AdaptiveScaffold) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtAdaptiveScaffold>(Rc::new(val), key);
        VNode::from(comp)
    }
}
