use std::collections::HashSet;

use indexmap::IndexMap;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::Key;

use crate::props::{ContainerBuilder, IntoOptionalRenderFn, RenderFn};
use crate::state::{Selection, SelectionObserver};
use crate::widget::Container;

use pwt_macros::{builder, widget};

/// Infos passed to the [SelectionView] render function.
pub struct SelectionViewRenderInfo {
    /// The key of the item to render
    pub key: Key,

    /// Set if this item is visible/active.
    ///
    /// So that the item can react on visibility changes.
    pub visible: bool,
}

/// A Container that listens to changes from a [Selection].
#[widget(pwt=crate, comp=PwtSelectionView, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct SelectionView {
    /// The Selection object.
    ///
    /// This object listens to selection changes and redraws the content
    /// whenever the selection changes. Setting a selection object is mandatory.
    #[builder(IntoPropValue, into_prop_value)]
    pub selection: Option<Selection>,

    /// Selection specific render functions.
    ///
    /// You can specify a render function for any key.
    #[prop_or_default]
    pub builders: IndexMap<Key, RenderFn<SelectionViewRenderInfo>>,

    /// The default render function.
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, SelectionViewRenderInfo)]
    pub renderer: Option<RenderFn<SelectionViewRenderInfo>>,

    /// Enable the page cache.
    ///
    /// If enabled, pages gets cached and will be rendered to html so that the keep
    /// there state. Although pages below the top page will be invisible by
    /// setting CSS "display: none;".
    #[prop_or_default]
    #[builder]
    pub page_cache: bool,
}

impl SelectionView {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to add a view builder.
    pub fn with_builder(
        mut self,
        key: impl Into<Key>,
        renderer: impl 'static + Fn(&SelectionViewRenderInfo) -> Html,
    ) -> Self {
        self.add_builder(key, renderer);
        self
    }

    /// Method to add a view builder for the specified key.
    pub fn add_builder(
        &mut self,
        key: impl Into<Key>,
        renderer: impl 'static + Fn(&SelectionViewRenderInfo) -> Html,
    ) {
        let key = key.into();
        self.builders.insert(key, RenderFn::new(renderer));
    }
}
pub enum Msg {
    SelectionChange(Selection),
}

#[doc(hidden)]
pub struct PwtSelectionView {
    active: Option<Key>,
    render_set: HashSet<Key>,
    _selection_observer: SelectionObserver,
}

impl Component for PwtSelectionView {
    type Message = Msg;
    type Properties = SelectionView;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let _selection_observer = props
            .selection
            .clone()
            .unwrap_or(Selection::new())
            .add_listener(ctx.link().callback(Msg::SelectionChange));

        Self {
            active: None,
            render_set: HashSet::new(),
            _selection_observer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SelectionChange(selection) => {
                self.active = selection.selected_key();
                if !props.page_cache {
                    self.render_set.clear();
                }
                if let Some(key) = &self.active {
                    self.render_set.insert(key.clone());
                }
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.selection != old_props.selection {
            self._selection_observer = props
                .selection
                .clone()
                .unwrap_or(Selection::new())
                .add_listener(ctx.link().callback(Msg::SelectionChange));
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = self.render_set.iter().map(|key| {
            let active = match &self.active {
                Some(active_key) => active_key == key,
                None => false,
            };

            let page = if let Some(render_fn) = props.builders.get(key) {
                render_fn.apply(&SelectionViewRenderInfo {
                    key: key.clone(),
                    visible: active,
                })
            } else if let Some(render_fn) = &props.renderer {
                render_fn.apply(&SelectionViewRenderInfo {
                    key: key.clone(),
                    visible: active,
                })
            } else {
                html! {}
            };
            if active {
                html! { <div key={key.clone()} class="pwt-fit">{page}</div>}
            } else {
                html! { <div key={key.clone()} class="pwt-d-none">{page}</div>}
            }
        });

        yew::props!(Container {
            std_props: props.std_props.clone(),
            listeners: props.listeners.clone(),
        })
        .children(content)
        .into()
    }
}
