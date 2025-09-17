use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::props::{ContainerBuilder, WidgetStyleBuilder};
use crate::widget::SizeObserver;

use pwt_macros::{builder, widget};

use super::{Container, CssBorderBuilder, ListTile};

/// List tile size Observer.
///
/// This is Used by the [List] implementation to track list tile size changes.
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

pub struct PwtListTileObserver {}

impl Component for PwtListTileObserver {
    type Message = ();
    type Properties = ListTileObserver;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let inner = Container::new()
            // We need to set height to "fit-content" in order to observe correct size
            .style("height", "fit-content")
            .style("display", "grid")
            .style("grid-template-columns", "subgrid")
            .style("grid-column", "1 / -1")
            .border_bottom(props.separator)
            .with_child(props.tile.clone());

        let resize_callback = props.resize_callback.clone();
        let tile_pos = props.tile_pos;
        let inner = SizeObserver::new(inner, move |(w, h)| {
            if let Some(resize_callback) = &resize_callback {
                resize_callback.emit((tile_pos, w, h));
            }
        });

        let mut wrapper = Container::new()
            .style("overflow", "hidden")
            .style("display", "grid")
            .style("grid-template-columns", "subgrid")
            .style("grid-column", "1 / -1")
            .with_child(inner);

        if let Some(height) = &props.force_height {
            wrapper.set_height(*height as f32);
        }
        wrapper.into()
    }
}
