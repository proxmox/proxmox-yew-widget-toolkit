use html::IntoPropValue;
use yew::virtual_dom::Key;

use crate::prelude::*;

use crate::props::{EventSubscriber, WidgetBuilder};

use crate::widget::Container;

use super::{ListTile, SizeObserver};

use pwt_macros::{builder, widget};

static VIRTUAL_SCROLL_TRIGGER: u64 = 30;

/// List with virtual scrolling (vertical).
#[widget(pwt=crate, comp=crate::widget::PwtList, @element)]
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct List {
    // Item count
    item_count: u64,
    // List item render function
    //renderer: ListRenderFn,
    renderer: Callback<u64, ListTile>,

    /// Yew key property.
    #[prop_or_default]
    pub key: Option<Key>,

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

    /// Separator, added between list items.
    #[prop_or_default]
    //#[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, u64)]
    //pub separator: Option<RenderFn<u64>>,
    #[builder(IntoPropValue, into_prop_value)]
    pub separator: Option<Callback<u64, Html>>,
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
    /// Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }
}

#[derive(Default)]
struct VirtualScrollInfo {
    start: u64,
    end: u64,
    height: f64,
    offset: f64,
}

impl VirtualScrollInfo {
    fn visible_rows(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

pub enum Msg {
    ScrollTo(i32, i32),
    TableResize(f64, f64),
    ViewportResize(f64, f64, f64),
}

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

    row_height: f64,
    scroll_info: VirtualScrollInfo,
}

impl PwtList {
    fn update_scroll_info(&mut self, props: &List) {
        let item_count = props.item_count;

        let virtual_scroll = props
            .virtual_scroll
            .unwrap_or(item_count >= VIRTUAL_SCROLL_TRIGGER);

        let mut start = if virtual_scroll {
            (self.viewport_scroll_top as f64 / self.row_height).floor() as u64
        } else {
            0
        };

        if start > 0 {
            start -= 1;
        }
        if (start & 1) == 1 {
            start -= 1;
        } // make it work with striped rows

        let max_visible_rows =
            (self.viewport_height / props.min_row_height as f64).ceil() as u64 + 5;
        let end = if virtual_scroll {
            (start + max_visible_rows).min(item_count)
        } else {
            item_count
        };

        if start > end {
            start = end.saturating_sub(max_visible_rows);
        }

        let offset = (start as f64) * self.row_height;
        let offset_end = offset + self.table_height;

        let height = offset_end + item_count.saturating_sub(end) as f64 * self.row_height;

        self.scroll_info = VirtualScrollInfo {
            start,
            end,
            offset,
            height,
        };
    }

    fn render_content(&self, props: &List) -> Html {
        let min_height = format!("{}px", props.min_row_height);

        let mut content = Container::new()
            .attribute("role", "none")
            .class("pwt-list-content")
            .node_ref(self.table_ref.clone())
            .style("display", "grid")
            .style("grid-template-columns", &props.grid_template_columns)
            .style("--pwt-list-tile-min-height", min_height)
            .style("position", "relative")
            .style("top", format!("{}px", self.scroll_info.offset));

        for pos in self.scroll_info.start..self.scroll_info.end {
            if pos != 0 {
                if let Some(separator) = &props.separator {
                    let separator = separator.emit(pos);
                    content.add_child(
                        Container::new()
                            .key(format!("sep-{pos}"))
                            .class("pwt-list-separator")
                            .with_child(separator),
                    );
                }
            }
            let mut row = props.renderer.emit(pos);
            row.set_key(format!("row-{pos}"));
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
        let props = ctx.props();
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
            row_height: props.min_row_height as f64,
            scroll_info: VirtualScrollInfo::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
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
                let height = height.max(0.0);
                if self.table_height == height {
                    return false;
                };

                self.table_height = height;
                let visible_rows = self.scroll_info.visible_rows();
                if (height > 0.0) && (visible_rows > 0) {
                    let row_height = height / visible_rows as f64;
                    if row_height > self.row_height {
                        self.row_height = row_height;
                    }
                }

                self.update_scroll_info(props);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = self.render_content(props);

        Container::from_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
            .node_ref(self.viewport_ref.clone())
            .class("pwt-list")
            .onscroll(ctx.link().batch_callback(move |event: Event| {
                let target: Option<web_sys::HtmlElement> = event.target_dyn_into();
                target.map(|el| Msg::ScrollTo(el.scroll_left(), el.scroll_top()))
            }))
            .with_child(content)
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.viewport_ref.cast::<web_sys::Element>() {
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
    }
}
