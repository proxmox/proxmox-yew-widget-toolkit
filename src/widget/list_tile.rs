use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::props::{
    ContainerBuilder, ListenersWrapper, WidgetBuilder, WidgetStdProps, WidgetStyleBuilder,
};

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

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    force_height: Option<u32>,

    // This is used internally by the List widget.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, (u64, f64, f64))]
    pub(crate) resize_callback: Option<Callback<(u64, f64, f64)>>,

    #[prop_or_default]
    #[builder]
    pub(crate) tile_pos: u64,
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

pub enum Msg {
    SetupSizeObserver,
}

impl PwtListTile {
    fn setup_size_observer(&mut self, props: &ListTile) {
        if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
            if let Some(resize_callback) = &props.resize_callback {
                let resize_callback = resize_callback.clone();
                if props.force_height.is_some() {
                    //  log::info!("add size observer {}", props.tile_pos);
                }
                let tile_pos = props.tile_pos;
                self.size_observer = Some(SizeObserver::new(&el, {
                    let el = el.clone();
                    move |(x, y)| {
                        log::info!("REAL HEIGHT {}", el.scroll_height());
                        resize_callback.emit((tile_pos, x, y));
                    }
                }));
            }
        }
    }
}
impl Component for PwtListTile {
    type Message = Msg;
    type Properties = ListTile;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            size_observer: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SetupSizeObserver => {
                if self.size_observer.is_none() {
                    self.setup_size_observer(props);
                    true
                } else {
                    false
                }
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let tile =
            Container::from_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
                .class("pwt-list-tile")
                .class(props.interactive.then(|| "pwt-interactive"))
                .attribute("aria-disabled", props.disabled.then(|| "true"))
                .class(props.disabled.then(|| "disabled"))
                .children(props.children.clone());

        let inner = Container::new()
            .node_ref(self.node_ref.clone())
            // We need to set height to "fit-content" in order to observe correct size
            .style("height", "fit-content")
            .style("display", "grid")
            .style("grid-template-columns", "subgrid")
            .style("grid-column", "1 / -1")
            .with_child(tile);

        let mut wrapper = Container::new()
            .with_child(inner)
            .style("overflow", "hidden")
            .style("display", "grid")
            .style("grid-template-columns", "subgrid")
            .style("grid-column", "1 / -1");

        if let Some(height) = &props.force_height {
            wrapper.set_height(*height as f32);
        }
        wrapper.into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let props = ctx.props();

        if self.size_observer.is_none() {
            ctx.link().send_message(Msg::SetupSizeObserver);
        }
    }
}
