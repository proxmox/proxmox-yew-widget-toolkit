use gloo_timers::callback::Timeout;
use html::IntoPropValue;

use crate::prelude::*;

use crate::props::{EventSubscriber, WidgetBuilder};

use crate::widget::Container;

use super::SizeObserver;

use pwt_macros::{builder, widget};

static VIRTUAL_SCROLL_TRIGGER: u64 = 30;

mod list_tile;
pub use list_tile::ListTile;

mod list_tile_observer;
pub use list_tile_observer::ListTileObserver;
#[doc(hidden)]
pub use list_tile_observer::PwtListTileObserver;

/// List with virtual scrolling (vertical).
///
/// This [List] only renders the visible elements.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::{List, ListTile};
/// # fn create_list_tile() -> List {
///     // Create a list with 1000 items.
///     List::new(1000, |pos| {
///         // we have to return a ListTile here.
///         ListTile::new().with_child(format!("Item {pos}"))
///     })
/// # }
/// ```
///
/// The virtual scrolling algorithm can handle different tile
/// sizes even if the tile size changes dynamically.
#[widget(pwt=crate, comp=crate::widget::PwtList, @element)]
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct List {
    // Item count
    item_count: u64,
    // List item render function
    //renderer: ListRenderFn,
    renderer: Callback<u64, ListTile>,

    /// The list uses a html grid layout, and you can set the 'grid-template-columns' property.
    ///
    /// This is a convenient way to use a column layout.
    ///
    ///  ```
    /// # use pwt::prelude::*;
    /// # use pwt::widget::{List, ListTile};
    /// # fn create_list_tile() -> List {
    ///     List::new(100, |pos| {
    ///         ListTile::new()
    ///             .with_child(html!{<span>{format!("{pos}")}</span>})
    ///             .with_child(html!{<span>{"A simple list tile"}</span>})
    ///     })
    ///     // Use a two column layout.
    ///     .grid_template_columns("auto 1fr")
    /// # }
    /// ```
    ///
    /// see: <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-template-columns>
    #[prop_or(AttrValue::Static("1fr"))]
    #[builder(IntoPropValue, into_prop_value)]
    pub grid_template_columns: AttrValue,

    /// Virtual Scroll
    ///
    /// Virtual scroll is enabled by default for lists with more than 30 rows.
    #[prop_or_default]
    #[builder]
    pub virtual_scroll: Option<bool>,

    /// Minimum row height (default 22)
    ///
    /// Sets the minimum height for list rows. This is also used by
    /// the virtual scrolling algorithm to compute the maximal number
    /// of visible rows.
    #[prop_or(22)]
    #[builder]
    pub min_row_height: u64,

    /// Add a line as separator between list items.
    #[prop_or_default]
    #[builder]
    pub separator: bool,
}

impl List {
    /// Create a new instance.
    ///
    /// Note: Virtual scrolling works best if all items have the same size.
    pub fn new(item_count: u64, renderer: impl Into<Callback<u64, ListTile>>) -> Self {
        yew::props!(List {
            item_count,
            renderer: renderer.into()
        })
    }
}

#[derive(Default)]
struct VirtualScrollInfo {
    start: u64,
    end: u64,
    height: f64,
    offset: f64,
}

pub enum Msg {
    ScrollTo(i32, i32),
    TableResize(f64, f64),
    ViewportResize(f64, f64, f64),
    TileResize(u64, f64, f64),
    DelayedTileResize,
}

#[derive(Default)]
struct SizeAccumulator {
    height_list: Vec<u64>,
}

impl SizeAccumulator {
    // Returns the row height
    fn _get_row_height(&self, index: usize, min_row_height: u64) -> u64 {
        self.height_list
            .get(index)
            .map(|v| *v)
            .unwrap_or(min_row_height)
    }

    // Update the size of a row.
    //
    // Returns the size difference (new height minus previous height)
    fn update_row(&mut self, index: usize, height: u64, min_row_height: u64) -> i64 {
        if self.height_list.len() <= index {
            self.height_list.resize(index + 1, min_row_height);
        }
        let diff = height as i64 - self.height_list[index] as i64;
        self.height_list[index] = height;
        //log::info!("UPDATE ROW {} {} {}", index, height, diff);
        diff
    }

    // Find the first visible row (a row ending after offset).
    //
    // Returns the row number, and the start offset of that row.
    fn find_start_row(&mut self, offset: u64, min_row_height: u64) -> (u64, u64) {
        if self.height_list.len() <= 2 {
            return (0, 0);
        }

        let mut height = 0u64;
        for i in 0..self.height_list.len() {
            let new_height = height + self.height_list[i];
            if new_height >= offset {
                // log::info!("START OFFSET {} {}", i, height);
                return (i as u64, height);
            }
            height = new_height;
        }
        let rest = (offset - height) / min_row_height;
        // log::info!("REST OFFSET {} {}", height, rest);
        (
            self.height_list.len() as u64 + rest,
            height + rest * min_row_height,
        )
    }

    fn compute_tail_height(&mut self, row_count: usize, end: usize, min_row_height: u64) -> u64 {
        if self.height_list.len() < row_count {
            self.height_list.resize(row_count, min_row_height);
        }

        let mut height = 0u64;
        for i in ((end + 1)..row_count).rev() {
            height += self.height_list[i];
        }
        height
    }
}

#[doc(hidden)]
pub struct PwtList {
    viewport_height: f64,
    viewport_width: f64,
    viewport_ref: NodeRef,
    viewport_size_observer: Option<SizeObserver>,
    viewport_scrollbar_size: Option<f64>,
    viewport_scroll_top: usize,

    table_ref: NodeRef,
    table_size_observer: Option<SizeObserver>,
    table_height: f64,

    scroll_info: VirtualScrollInfo,

    row_heights: SizeAccumulator,

    set_scroll_top: Option<usize>,

    tile_resize_callback: Callback<(u64, f64, f64)>,
    tile_resize_timeout: Option<Timeout>,
    scroll_diff: i64,
}

impl PwtList {
    fn update_scroll_info(&mut self, props: &List) {
        let item_count = props.item_count;

        let virtual_scroll = props
            .virtual_scroll
            .unwrap_or(item_count >= VIRTUAL_SCROLL_TRIGGER);

        let (start, offset) = if virtual_scroll {
            self.row_heights
                .find_start_row(self.viewport_scroll_top as u64, props.min_row_height)
        } else {
            (0, 0)
        };

        let max_visible_rows =
            (self.viewport_height / props.min_row_height as f64).ceil() as u64 + 5;
        let end = if virtual_scroll {
            (start + max_visible_rows).min(item_count)
        } else {
            item_count
        };

        let offset_end = offset as f64 + self.table_height;

        let tail_height = self.row_heights.compute_tail_height(
            props.item_count as usize,
            end as usize,
            props.min_row_height,
        );

        let height = offset_end + tail_height as f64;

        self.scroll_info = VirtualScrollInfo {
            start,
            end,
            offset: offset as f64,
            height,
        };
    }

    fn render_content(&self, _ctx: &Context<Self>, props: &List) -> Html {
        let min_height = format!("{}px", props.min_row_height);

        let mut content = Container::new()
            .attribute("role", "none")
            .class("pwt-list-content")
            .node_ref(self.table_ref.clone())
            .attribute("data-list-start-row", self.scroll_info.start.to_string())
            .attribute("data-list-offset", self.scroll_info.offset.to_string())
            .style("display", "grid")
            .style("grid-template-columns", &props.grid_template_columns)
            .style("--pwt-list-tile-min-height", min_height)
            .style("position", "relative")
            .style("top", format!("{}px", self.scroll_info.offset));

        if self.scroll_info.end > self.scroll_info.start {
            if self.scroll_info.start > 5 {
                for index in (self.scroll_info.start - 5)..self.scroll_info.start {
                    // log::info!("ADD CACHED ROW {index}");

                    let row = ListTileObserver::new(props.renderer.emit(index))
                        .key(format!("row-{index}"))
                        .force_height(0)
                        .tile_pos(index)
                        .separator(props.separator)
                        .resize_callback(Some(self.tile_resize_callback.clone()));

                    //row.set_attribute("role", "listitem");
                    content.add_child(row);
                }
            }
        }

        for pos in self.scroll_info.start..self.scroll_info.end {
            // if we have keys, we need overflow-anchor none on the scroll container
            // see: https://github.com/facebook/react/issues/27044
            let row = ListTileObserver::new(props.renderer.emit(pos))
                .key(format!("row-{pos}"))
                .tile_pos(pos)
                .separator(props.separator)
                .resize_callback(Some(self.tile_resize_callback.clone()));

            content.add_child(row);
        }

        Container::new()
            .height(self.scroll_info.height)
            .attribute("role", "none")
            .with_child(content)
            .into()
    }
}

impl Component for PwtList {
    type Message = Msg;
    type Properties = List;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            viewport_height: 0.0,
            viewport_width: 0.0,
            viewport_size_observer: None,
            viewport_scrollbar_size: None,
            viewport_ref: NodeRef::default(),
            viewport_scroll_top: 0,

            table_ref: NodeRef::default(),
            table_size_observer: None,
            table_height: 0.0,
            scroll_info: VirtualScrollInfo::default(),

            row_heights: SizeAccumulator::default(),

            set_scroll_top: None,

            tile_resize_callback: ctx
                .link()
                .callback(|(pos, w, h)| Msg::TileResize(pos, w, h)),
            tile_resize_timeout: None,
            scroll_diff: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::DelayedTileResize => {
                self.update_scroll_info(props);
                self.tile_resize_timeout = None;
                if self.scroll_diff != 0 && self.viewport_scroll_top != 0 {
                    log::info!("TOP DIFF {}", self.scroll_diff);
                    self.set_scroll_top =
                        Some((self.viewport_scroll_top as i64 + self.scroll_diff).max(0) as usize);
                    self.scroll_diff = 0;
                }
                true
            }
            Msg::TileResize(pos, _w, h) => {
                let corr =
                    self.row_heights
                        .update_row(pos as usize, h as u64, props.min_row_height);
                log::info!("UPDATE ROW HEIGHT {pos} {h} {corr}");

                //self.update_scroll_info(props);

                if corr != 0 && pos < self.scroll_info.start {
                    log::info!("CORRECTION {pos} {corr}");
                    self.scroll_diff += corr;
                }

                let link = ctx.link().clone();
                // try to gather all update events
                self.tile_resize_timeout = Some(Timeout::new(1, move || {
                    link.send_message(Msg::DelayedTileResize);
                }));
                // avoid redraw
                false
            }
            Msg::ScrollTo(_x, y) => {
                self.viewport_scroll_top = y.max(0) as usize;
                self.update_scroll_info(props);
                props.virtual_scroll.unwrap_or(true)
            }
            Msg::ViewportResize(width, height, scrollbar_size) => {
                self.viewport_height = height.max(0.0);
                self.viewport_width = width.max(0.0);

                self.viewport_scrollbar_size = if scrollbar_size.abs() < 1.0 {
                    // on certain zoom levels, the scrollbar size calculation is not perfect...
                    None
                } else {
                    Some(scrollbar_size)
                };

                self.update_scroll_info(props);
                true
            }
            Msg::TableResize(_width, height) => {
                self.table_height = height.max(0.0);
                self.update_scroll_info(props);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = self.render_content(ctx, props);

        Container::from_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
            .node_ref(self.viewport_ref.clone())
            .class("pwt-list")
            .attribute("role", "list")
            .style("overflow-anchor", "none")
            .onscroll(ctx.link().batch_callback(move |event: Event| {
                let target: Option<web_sys::HtmlElement> = event.target_dyn_into();
                target.map(|el| Msg::ScrollTo(el.scroll_left(), el.scroll_top()))
            }))
            .with_child(content)
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let viewport_el = self.viewport_ref.cast::<web_sys::Element>();

        if first_render {
            if let Some(el) = &viewport_el {
                let link = ctx.link().clone();
                let size_observer =
                    SizeObserver::new(&el, move |(width, height, client_width, _)| {
                        link.send_message(Msg::ViewportResize(width, height, width - client_width));
                    });
                self.viewport_size_observer = Some(size_observer);
            }
            if let Some(el) = self.table_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::TableResize(width, height));
                });
                self.table_size_observer = Some(size_observer);
            }
        }

        if let Some(el) = &viewport_el {
            if let Some(top) = self.set_scroll_top.take() {
                // Note: we delay setting ScrollTop until we rendered the
                // viewport with correct height. Else, set_scroll_top can
                // fail because the viewport is smaller.
                if let Some(el) = &viewport_el {
                    el.set_scroll_top(top as i32);
                }
            }

            // Fix for missing onscroll event.
            //
            // If we hide the complete list inside a TabPanel, the scrollbar vanish and scrollTop
            // gets zero. After entering the TabPanel view again, scrollTop gets set to the previous value,
            // but chromium does not fire an onscroll event.
            if self.viewport_scroll_top == 0 {
                let top = el.scroll_top();
                if top > 0 {
                    ctx.link().send_message(Msg::ScrollTo(0, top));
                }
            }
        }
    }
}
