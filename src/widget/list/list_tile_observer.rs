use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::props::{ContainerBuilder, WidgetBuilder, WidgetStyleBuilder};

use pwt_macros::{builder, widget};

use super::{Container, CssBorderBuilder, ListTile, SizeObserver};

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
#[widget(pwt=crate, comp=crate::widget::PwtListTileObserver)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
#[builder]
pub struct ListTileObserver {
    /// Add a line as separator
    #[prop_or_default]
    #[builder]
    pub separator: bool,

    /// Force the height of the widget
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub force_height: Option<u32>,

    // This is used internally by the List widget.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, (u64, f64, f64))]
    pub(crate) resize_callback: Option<Callback<(u64, f64, f64)>>,

    #[prop_or_default]
    #[builder]
    pub(crate) tile_pos: u64,

    tile: ListTile,
}

impl ListTileObserver {
    /// Creates new ListTileObserver instance
    pub fn new(tile: ListTile) -> Self {
        yew::props! { Self { tile } }
    }
}

pub struct PwtListTileObserver {
    node_ref: NodeRef,
    size_observer: Option<SizeObserver>,
}

pub enum Msg {
    SetupSizeObserver,
}

impl PwtListTileObserver {
    fn setup_size_observer(&mut self, props: &ListTileObserver) {
        if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
            if let Some(resize_callback) = &props.resize_callback {
                let resize_callback = resize_callback.clone();
                if props.force_height.is_some() {
                    //  log::info!("add size observer {}", props.tile_pos);
                }
                let tile_pos = props.tile_pos;
                let separator_height = if props.separator { 1.0 } else { 0.0 };

                self.size_observer = Some(SizeObserver::new(&el, {
                    let el = el.clone();
                    move |(w, h)| {
                        log::info!("REAL HEIGHT {}", el.scroll_height());
                        resize_callback.emit((tile_pos, w, h + separator_height));
                    }
                }));
            }
        }
    }
}
impl Component for PwtListTileObserver {
    type Message = Msg;
    type Properties = ListTileObserver;

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

        let inner = Container::new()
            .node_ref(self.node_ref.clone())
            // We need to set height to "fit-content" in order to observe correct size
            .style("height", "fit-content")
            .style("display", "grid")
            .style("grid-template-columns", "subgrid")
            .style("grid-column", "1 / -1")
            .with_child(props.tile.clone());

        let mut wrapper = Container::new()
            .with_child(inner)
            .style("overflow", "hidden")
            .style("display", "grid")
            .style("grid-template-columns", "subgrid")
            .style("grid-column", "1 / -1")
            .border_bottom(props.separator);

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
