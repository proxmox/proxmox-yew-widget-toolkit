use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::AnyRoute;
use yew_router::scope_ext::{RouterScopeExt, LocationHandle};
use yew_router::history::Location;
use crate::props::ContainerBuilder;


// Note: We do not use empty path segment. Instead we use '_' for empty segments.
// gloo::HashHistory constructs the location using (for "https://example.com#//menu2")
// # new URL("//memu2", "https://example.com")
// which results in total nonsense
fn normalize_segment(segment: &str) -> &str {
    if segment.is_empty() {
        "_"
    } else {
        segment
    }
}

// remove leading slash
// remove consecutive slashes
fn normalize_path(path: &str) -> String {
    let mut new = String::new();
    for seg in path.split('/') {
        if !seg.is_empty() {
            if !new.is_empty() { new.push('/'); }
            new.push_str(seg);
        }
    }
    new
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationContext {
    child_path: Option<String>,
    parent_path: Option<String>,
    path: String,
}


impl NavigationContext {

    pub fn path(&self) -> String {
        self.path.clone()
    }
    pub fn full_path(&self) -> String {
        if let Some(parent_path) = &self.parent_path {
            format!("{}/{}", normalize_segment(parent_path), normalize_segment(&self.path))
        } else {
            format!("/{}", normalize_segment(&self.path))
        }
    }

    pub fn child_context(&self) -> NavigationContext {
        match &self.child_path {
            Some(full_path) => {
                let (path, child_path) = match full_path.split_once('/') {
                    Some((path, child_path)) => (path.to_string(), Some(child_path.to_string())),
                    None => (full_path.to_string(), None),
                };
                NavigationContext {
                    child_path,
                    path,
                    parent_path: Some(self.full_path()),
                }
            }
            None => {
                NavigationContext {
                    child_path: None,
                    path: String::new(),
                    parent_path: Some(self.full_path()),
                }
            }
        }
    }
}

fn location_to_nav_ctx(loc: &Option<Location>) -> NavigationContext {

    //log::info!("LOCATION {:?}", loc);

    let full_path = match loc {
        Some(loc) => normalize_path(loc.path()),
        None => String::new(),
    };

    //log::info!("LOCATION {}", full_path);
    let (path, child_path) = match full_path.split_once('/') {
        Some((path, child_path)) => (path.to_string(), Some(child_path.to_string())),
        None => (full_path, None),
    };

    NavigationContext {
        child_path,
        path,
        parent_path: None,
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct NavigationContainer {
    active: Option<AttrValue>,
    #[prop_or_default]
    pub children: Vec<VNode>,
}

impl NavigationContainer {

    pub fn new() -> Self {
        yew::props!(Self {})
    }

}

impl ContainerBuilder for NavigationContainer {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

#[doc(hidden)]
pub struct PwtNavigationContainer {
    nav_ctx: NavigationContext,
    _parent_context_handle: Option<ContextHandle<NavigationContext>>,
    _location_context_handle: Option<LocationHandle>,
}

pub enum Msg {
    NavCtxUpdate(NavigationContext),
    LocationUpdate(Location),
}

impl Component for PwtNavigationContainer {
    type Message = Msg;
    type Properties = NavigationContainer;

    fn create(ctx: &Context<Self>) -> Self {
        let parent_context_handle;
        let location_context_handle;
        let nav_ctx;
        let on_ctx_update = ctx.link().callback(|nav_ctx| Msg::NavCtxUpdate(nav_ctx));
        if let Some((parent_nav_ctx, handle)) = ctx.link().context::<NavigationContext>(on_ctx_update) {
            nav_ctx = parent_nav_ctx.child_context();
            parent_context_handle = Some(handle);
            location_context_handle = None;
        } else {
            let on_loc_update = ctx.link().callback(|loc| Msg::LocationUpdate(loc));
            if let Some(loc_handle) = ctx.link().add_location_listener(on_loc_update) {
                nav_ctx = location_to_nav_ctx(&ctx.link().location());
                parent_context_handle = None;
                location_context_handle = Some(loc_handle);
            } else {
                nav_ctx = NavigationContext {
                    child_path: None,
                    path: String::new(),
                    parent_path: None,
                };
                parent_context_handle = None;
                location_context_handle = None;
            }
        }

        Self {
            nav_ctx,
            _parent_context_handle: parent_context_handle,
            _location_context_handle: location_context_handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NavCtxUpdate(parent_nav_ctx) => {
                self.nav_ctx = parent_nav_ctx.child_context();
                true
            }
            Msg::LocationUpdate(location) => {
                self.nav_ctx = location_to_nav_ctx(&Some(location));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        //log::info!("NAVCTX {:?}", self.nav_ctx);
        html!{
            <ContextProvider<NavigationContext> context={self.nav_ctx.clone()}>
                {props.children.clone()}
            </ContextProvider<NavigationContext>>
         }
    }
}

impl Into<VNode> for NavigationContainer {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtNavigationContainer>(Rc::new(self), None);
        VNode::from(comp)
    }
}

pub trait NavigationContextExt {
    fn nav_context(&self) -> Option<NavigationContext>;
    fn full_path(&self) -> Option<String>;
    fn child_path(&self) -> Option<String>;
    fn push_relative_route(&self, path: &str);
}

impl<COMP: Component> NavigationContextExt for yew::html::Scope<COMP> {

    fn nav_context(&self) -> Option<NavigationContext> {
        self.context::<NavigationContext>(Callback::from(|_| {}))
            .map(|(nav_ctx, _)| nav_ctx)
    }

    fn full_path(&self) -> Option<String> {
        self.nav_context()
            .map(|nav_ctx| nav_ctx.full_path())
    }

    fn child_path(&self) -> Option<String> {
        self.nav_context()
            .map(|nav_ctx| nav_ctx.child_context().path)
    }

    fn push_relative_route(&self, path: &str) {
        let path = normalize_segment(path);
        //log::info!("PUSH REL {}", path);
        if let Some((nav_ctx, _handle)) = self.context::<NavigationContext>(Callback::from(|_| {})) {
            let abs_path = match &nav_ctx.parent_path {
                Some(parent_path) => format!("{}/{}", parent_path, path),
                None => format!("/{}", path),
            };
            match self.navigator() {
                Some(navigator) => {
                    navigator.push(&AnyRoute::new(abs_path));
                }
                None => {
                    log::error!("no Navigator found");
                }
            }
        } else {
            match self.navigator() {
                Some(navigator) => {
                    let path = format!("/{}", path);
                    navigator.push(&AnyRoute::new(path));
                }
                None => {
                    log::error!("no Navigator found");
                }
            }
        }
    }

}
