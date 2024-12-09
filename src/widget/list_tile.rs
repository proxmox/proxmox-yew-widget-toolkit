use html::IntoEventCallback;
use yew::prelude::*;

use crate::props::{ContainerBuilder, ListenersWrapper, WidgetBuilder, WidgetStdProps};

use pwt_macros::{builder, widget};

use super::{Container, SizeObserver};

/// List tile. A container with grid/subgrid layout.
///
/// This is meant to be used inside [List](crate::widget::List).
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::{List, ListTile};
/// # fn create_list_tile() -> List {
///     List::new(100, |pos| {
///         ListTile::new()
///             .with_child(html!{<span>{format!("{pos}")}</span>})
///             .with_child(html!{<span>{"A simple list tile"}</span>})
///             .with_child(html!{<span>{"third column"}</span>})
///             .interactive(true)
///             .disabled(false)
///             .class(pwt::css::ColorScheme::Primary)
///     })
///     .grid_template_columns("auto 1fr auto")
/// # }
/// ```
#[widget(pwt=crate, comp=crate::widget::PwtListTile, @element, @container)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
#[builder]
pub struct ListTile {
    #[prop_or_default]
    #[builder]
    interactive: bool,

    #[prop_or_default]
    #[builder]
    disabled: bool,

    // This is used internally by the List widget.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, (f64, f64))]
    pub(crate) resize_callback: Option<Callback<(f64, f64)>>,
}

impl ListTile {
    /// Creates new ListTile instance
    pub fn new() -> Self {
        yew::props! { Self {} }
    }

    /// Creates a new instance from existing properties
    pub fn from_widget_props(
        std_props: WidgetStdProps,
        listeners: Option<ListenersWrapper>,
    ) -> Self {
        yew::props! { Self { std_props, listeners: listeners.unwrap_or_default() } }
    }
}

pub struct PwtListTile {
    node_ref: NodeRef,
    size_observer: Option<SizeObserver>,
}

impl Component for PwtListTile {
    type Message = ();
    type Properties = ListTile;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            size_observer: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Container::from_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
            .node_ref(self.node_ref.clone())
            .class("pwt-list-tile")
            .class(props.interactive.then(|| "pwt-interactive"))
            .class(props.disabled.then(|| "disabled"))
            .children(props.children.clone())
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
                if let Some(resize_callback) = &ctx.props().resize_callback {
                    let resize_callback = resize_callback.clone();
                    self.size_observer = Some(SizeObserver::new(&el, move |(x, y)| {
                        resize_callback.emit((x, y));
                    }));
                }
            }
        }
    }
}
