use std::borrow::Cow;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::{builder, widget};

use crate::props::{
    ContainerBuilder, EventSubscriber, IntoOptionalRenderFn, RenderFn, WidgetBuilder,
    WidgetStyleBuilder,
};
use crate::tr;
use crate::widget::{Container, Tooltip, WeekStart};

/// Which date window a [`CalendarGrid`] spans around its anchor date.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum CalendarGridView {
    /// The week containing the anchor (7 cells), starting on the grid's [`WeekStart`].
    Week,
    /// The anchor's month, expanded to full weeks (always 42 cells, so the
    /// grid height never jumps when paging between months).
    #[default]
    Month,
}

/// A multi-day bar drawn across the calendar cells its date range covers.
///
/// The grid lays each bar out per day as a segment of [`render_span`](CalendarGrid::render_span)
/// (or the built-in default markup), breaking it at week-row boundaries into one segment per row
/// and fusing the segments within a row into one continuous bar. The dates are inclusive ISO
/// `YYYY-MM-DD` strings, compared lexically; a range whose start or end falls outside the visible
/// window simply renders the covered part, with the off-window edge reading as a continuation (no
/// start marker / squared end).
///
/// # Styling without a `render_span` hook
///
/// The built-in default markup covers the common case with no hook: give each bar a
/// [`color`](Self::color) and an optional [`icon`](Self::icon), then pass the list via
/// [`span_bars`](CalendarGrid::span_bars) and, if wanted,
/// [`selected_span`](CalendarGrid::selected_span) plus
/// [`on_span_click`](CalendarGrid::on_span_click). Reach for
/// [`render_span`](CalendarGrid::render_span) only to fully replace a segment's markup.
///
/// # Class contract
///
/// The default markup emits these stable classes and custom property; a theme styles them (pwt
/// ships no CSS of its own):
///
/// - `pwt-calendar-span`: on every segment.
/// - `pwt-calendar-span-start`: the segment on the bar's true first day.
/// - `pwt-calendar-span-end`: the segment on the bar's true last day.
/// - `pwt-calendar-span-start-marker`: the true start while
///   [`span_start_marker`](CalendarGrid::span_start_marker) is on, to accent the origin.
/// - `pwt-calendar-span-continues-left`: the span carries on into the previous cell; square this
///   edge and bleed it left so the two fuse.
/// - `pwt-calendar-span-continues-right`: the span carries on into the next cell; square this edge
///   and bleed it right so the two fuse.
/// - `pwt-calendar-span-selected`: the segment whose bar matches
///   [`selected_span`](CalendarGrid::selected_span).
/// - `pwt-calendar-span-tip`: the tooltip wrapper around a segment once
///   [`render_span_tooltip`](CalendarGrid::render_span_tooltip) is set; it, not the segment, is
///   then the lane box's direct child, so flex-item rules belong on it.
/// - `pwt-calendar-span-lanes`: the per-cell scroll box wrapping a day's non-pinned segments.
/// - `pwt-calendar-span-pinned`: the per-cell fixed zone above the scroll box, wrapping the day's
///   [`pinned`](Self::pinned) segments and their lane spacers.
/// - `pwt-calendar-span-spacer`: an empty bar-height lane holder in the pinned zone, keeping a
///   pinned bar at the same height across the days of its row it does not cover.
/// - `pwt-calendar-span-icon`: the leading [`icon`](Self::icon) holder, present only on a row's
///   labelled segment.
/// - `pwt-calendar-span-label`: the label holder inside a segment.
/// - `--pwt-calendar-span-color`: custom property carrying [`color`](Self::color); derive the fill,
///   border, and text color from it.
#[derive(Clone, PartialEq)]
pub struct CalendarSpanBar {
    /// First covered day, inclusive ISO `YYYY-MM-DD`.
    pub start: String,
    /// Last covered day, inclusive ISO `YYYY-MM-DD`.
    pub end: String,
    /// Text drawn on the bar's first segment of each week row.
    pub label: String,
    /// Optional caller token (event vs leave, severity, ...). The grid does not interpret it; it is
    /// handed back on every [`CalendarSpanSegment`] so a [`render_span`](CalendarGrid::render_span)
    /// hook can branch on it, and rides [`CalendarSpanClick::kind`].
    pub kind: Option<String>,
    /// Optional caller identity for the bar, handed back on every segment. The grid uses it to
    /// match [`selected_span`](CalendarGrid::selected_span) and rides it on
    /// [`CalendarSpanClick::id`]; a hook uses it to tie the segments of one run together across
    /// cells, which the label cannot do when two runs happen to share it.
    pub id: Option<String>,
    /// Extra CSS class(es) applied to every segment of this bar. Themable alongside the built-in
    /// `pwt-calendar-span*` classes.
    pub class: Classes,
    /// Per-bar color fed to the default markup as the `--pwt-calendar-span-color` custom property;
    /// a theme derives fill, border, and text from it. Ignored when
    /// [`render_span`](CalendarGrid::render_span) takes over.
    pub color: Option<AttrValue>,
    /// Leading icon class(es) (Font Awesome-style) shown once per week row on the labelled segment,
    /// so it is not repeated on every covered day. Ignored when
    /// [`render_span`](CalendarGrid::render_span) takes over.
    pub icon: Option<Classes>,
    /// Pin this bar to a fixed zone above the scrolling lanes, so it stays visible while a busy
    /// day's other bars scroll. Pinned bars keep a stable lane across a week row (an equal-height
    /// spacer holds a lane open on the days a pinned bar does not cover), so a multi-day pinned bar
    /// sits at the same height in every cell it spans. The pinned zone does not scroll, so pin only
    /// a few bars per day: enough overlapping pins to exceed the cell height are clipped, not
    /// scrolled.
    pub pinned: bool,
    /// Stacking rank among [`pinned`](Self::pinned) bars: ranks the greedy lane assignment, so a
    /// higher priority generally sits nearer the top of the pinned zone; where chains of
    /// overlapping pins free a higher lane mid-run, a lower-priority bar can still take it - a
    /// bar's lane holds across its whole run, which rules out exact per-day ordering. Ties fall
    /// back to start then end date. Default `0`. Ignored for non-pinned bars, which stack by date.
    pub priority: i32,
}

impl CalendarSpanBar {
    /// A bar covering `start..=end` (inclusive ISO dates) carrying `label`.
    pub fn new(start: impl Into<String>, end: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
            label: label.into(),
            kind: None,
            id: None,
            class: Classes::new(),
            color: None,
            icon: None,
            pinned: false,
            priority: 0,
        }
    }

    /// Set the caller token handed back on every segment.
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Set the caller identity handed back on every segment.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Add extra CSS class(es) applied to every segment of this bar.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.class.push(class.into());
        self
    }

    /// Set the per-bar color emitted as the `--pwt-calendar-span-color` custom property.
    pub fn color(mut self, color: impl Into<AttrValue>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the leading icon class(es) shown on the bar's labelled segment.
    pub fn icon(mut self, icon: impl Into<Classes>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Pin the bar above the scrolling lanes (see [`pinned`](Self::pinned)).
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    /// Set the pinned-stacking rank (see [`priority`](Self::priority)).
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// One per-day segment of a [`CalendarSpanBar`], handed to the
/// [`render_span`](CalendarGrid::render_span) hook. Carries the originating bar plus the geometry
/// the hook needs to round/square and label correctly without re-deriving it from the dates.
#[derive(Clone, PartialEq)]
pub struct CalendarSpanSegment {
    /// The bar this segment belongs to.
    pub bar: CalendarSpanBar,
    /// The cell's day as an ISO `YYYY-MM-DD` date.
    pub date: String,
    /// This day is the bar's true start (`date == bar.start`).
    pub is_start: bool,
    /// This day is the bar's true end (`date == bar.end`).
    pub is_end: bool,
    /// A bar segment continues into the previous cell (same week row): square this edge and bleed
    /// it left so the two segments fuse. False on a week-row's first cell even mid-span, where the
    /// row break squares the edge but there is no neighbour to bleed into.
    pub continues_left: bool,
    /// A bar segment continues into the next cell (same week row): square this edge and bleed it
    /// right. False on a week-row's last cell even mid-span.
    pub continues_right: bool,
    /// The visible window's earliest covered day for this bar lands in a week row that does not
    /// contain the true start: this segment is a continuation, so it must not show the start marker
    /// even though it is the leftmost piece of the bar in this row.
    pub row_continues_from_prev: bool,
    /// As [`row_continues_from_prev`](Self::row_continues_from_prev) but for the trailing edge: the
    /// row's last covered cell is not the true end, so the right edge stays squared.
    pub row_continues_to_next: bool,
    /// Label drawn for this segment: the bar's label on the first covered cell of each week row, an
    /// empty string on the continuation cells, so a multi-day bar is titled once per row.
    pub label: String,
    /// The grid's [`span_start_marker`](CalendarGrid::span_start_marker) setting, resolved for this
    /// segment, so a `render_span` hook honors it instead of hard-coding the marker.
    pub show_start_marker: bool,
}

impl CalendarSpanSegment {
    /// The geometry and per-bar classes for this segment. A [`render_span`](CalendarGrid::render_span)
    /// hook styles from the same vocabulary as the built-in markup instead of re-deriving it, and so
    /// picks up any edge class the grid grows later.
    pub fn class(&self) -> Classes {
        let mut class = Classes::from("pwt-calendar-span");
        if self.is_start {
            class.push("pwt-calendar-span-start");
            if self.show_start_marker {
                class.push("pwt-calendar-span-start-marker");
            }
        }
        if self.is_end {
            class.push("pwt-calendar-span-end");
        }
        if self.continues_left {
            class.push("pwt-calendar-span-continues-left");
        }
        if self.continues_right {
            class.push("pwt-calendar-span-continues-right");
        }
        class.push(self.bar.class.clone());
        class
    }
}

/// The day and originating bar of a click on a span segment, handed to
/// [`on_span_click`](CalendarGrid::on_span_click). The `id`/`kind` come from the
/// [`CalendarSpanBar`] so the handler knows which run was hit; `date` is the clicked cell's day.
#[derive(Clone, PartialEq)]
pub struct CalendarSpanClick {
    /// The bar's [`id`](CalendarSpanBar::id), if any.
    pub id: Option<String>,
    /// The bar's [`kind`](CalendarSpanBar::kind), if any.
    pub kind: Option<String>,
    /// The clicked cell's day as an ISO `YYYY-MM-DD` date.
    pub date: String,
}

/// Per-day context handed to the [`CalendarGrid`] render hooks and callbacks.
#[derive(Clone, PartialEq)]
pub struct CalendarGridDay {
    /// The day as an ISO `YYYY-MM-DD` date.
    pub date: String,
    /// Day of month, 1-31.
    pub day: u32,
    /// Weekday with Monday = 0 .. Sunday = 6.
    pub weekday: u32,
    /// Whether the day belongs to the anchor's month (always true in week view).
    pub in_anchor_month: bool,
    /// Whether the day equals the `today` property.
    pub is_today: bool,
    /// Saturday or Sunday.
    pub is_weekend: bool,
}

/// A 7-column week / month calendar grid.
///
/// Pure presentation: the widget derives the visible date window from an anchor date, renders the
/// localized weekday header and one cell per day with the usual state classes
/// (`pwt-calendar-day-outside`, `-today`, `-weekend`), and leaves cell content to the
/// [`render_day`](Self::render_day) hook. Data loading, selection state, and any overlays stay in
/// the application; use [`visible_range`](Self::visible_range) to fetch data for exactly the
/// rendered window.
///
/// The widget intentionally takes `today` as a property: the toolkit does not know the
/// application's civil timezone, so the caller decides which date carries the today ring.
///
/// ```rust
/// # use pwt::prelude::*;
/// # use pwt::widget::{CalendarGrid, CalendarGridDay, CalendarGridView};
/// CalendarGrid::new("2026-06-10")
///     .view(CalendarGridView::Month)
///     .today("2026-06-10")
///     .render_day(|day: &CalendarGridDay| {
///         html! { <span>{ format!("content for {}", day.date) }</span> }
///     })
/// # ;
/// ```
///
/// Multi-day span bars with the built-in styling need no [`render_span`](Self::render_span) hook:
/// give each bar a color and icon, then wire selection and clicks.
///
/// ```rust,no_run
/// # use pwt::prelude::*;
/// # use pwt::widget::{CalendarGrid, CalendarSpanBar, CalendarSpanClick};
/// CalendarGrid::new("2026-06-10")
///     .span_bars([
///         CalendarSpanBar::new("2026-06-08", "2026-06-12", "Alice - vacation")
///             .id("leave-42")
///             .color("#2e7d32")
///             .icon("fa fa-plane"),
///         CalendarSpanBar::new("2026-06-10", "2026-06-10", "All-hands")
///             .id("event-7")
///             .color("#1565c0")
///             .icon("fa fa-users"),
///     ])
///     .selected_span("leave-42")
///     .on_span_click(|click: CalendarSpanClick| {
///         let _ = (click.id, click.kind, click.date);
///     })
/// # ;
/// ```
#[widget(pwt=crate, @element)]
#[builder]
#[derive(Default, Clone, PartialEq, Properties)]
pub struct CalendarGrid {
    /// Anchor date (`YYYY-MM-DD`) the visible window derives from.
    pub anchor: AttrValue,

    /// Window mode, [`CalendarGridView::Month`] by default.
    #[builder]
    #[prop_or_default]
    pub view: CalendarGridView,

    /// The day each week starts on, [`WeekStart::Monday`] by default.
    #[builder]
    #[prop_or_default]
    pub week_start: WeekStart,

    /// The application's current civil date (`YYYY-MM-DD`); that cell gets
    /// `pwt-calendar-day-today`.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub today: Option<AttrValue>,

    /// Renders a cell's body below the day-number row.
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, CalendarGridDay)]
    #[prop_or_default]
    pub render_day: Option<RenderFn<CalendarGridDay>>,

    /// Replaces the default day-number row at the top of a cell.
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, CalendarGridDay)]
    #[prop_or_default]
    pub render_day_header: Option<RenderFn<CalendarGridDay>>,

    /// Extra classes for a cell (holidays, highlights, ...).
    #[builder_cb(
        IntoOptionalRenderFn,
        into_optional_render_fn,
        CalendarGridDay,
        Classes
    )]
    #[prop_or_default]
    pub day_class: Option<RenderFn<CalendarGridDay, Classes>>,

    /// Fires when a day cell is clicked. A click on a span bar reaches here too, by bubbling (see
    /// [`on_span_click`](Self::on_span_click)); a bar activated by keyboard does not. A consumer
    /// wanting the two fully decoupled must stop propagation in its own
    /// [`render_span`](Self::render_span) markup.
    #[builder_cb(IntoEventCallback, into_event_callback, CalendarGridDay)]
    #[prop_or_default]
    pub on_day_click: Option<Callback<CalendarGridDay>>,

    #[builder_cb(IntoEventCallback, into_event_callback, CalendarGridDay)]
    #[prop_or_default]
    pub on_day_dblclick: Option<Callback<CalendarGridDay>>,

    /// Let the grid grow into the available height, distributing it across
    /// the week rows (dashboards); unset, rows take their natural height
    /// (mobile pages).
    #[builder]
    #[prop_or_default]
    pub fill_height: bool,

    /// Multi-day bars drawn across the cells their date range covers. Empty by default; a grid
    /// with no bars renders the plain day grid. Add via [`with_span_bar`](Self::with_span_bar) /
    /// [`add_span_bar`](Self::add_span_bar) or set the whole list with
    /// [`span_bars`](Self::span_bars).
    #[prop_or_default]
    pub span_bars: Vec<CalendarSpanBar>,

    /// Draw a thicker edge on a span's true start segment, so a bar that began before the visible
    /// window reads as a continuation rather than a fresh start. On by default.
    #[builder]
    #[prop_or(true)]
    pub span_start_marker: bool,

    /// Renders one per-day segment of a span bar, replacing the built-in default markup. Use it to
    /// add the application's own colors, tooltips, or click handling; the [`CalendarSpanSegment`]
    /// carries the geometry (start/end, continuation edges) so the hook can style without
    /// re-deriving it. Without a hook the grid emits the default `pwt-calendar-span*` markup.
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, CalendarSpanSegment)]
    #[prop_or_default]
    pub render_span: Option<RenderFn<CalendarSpanSegment>>,

    /// The [`id`](CalendarSpanBar::id) of the currently selected span. The default markup adds
    /// `pwt-calendar-span-selected` (and reflects `aria-pressed`) on every segment of the matching
    /// bar. Ignored when [`render_span`](Self::render_span) takes over.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub selected_span: Option<AttrValue>,

    /// Fires when any of a span's covered days is clicked, or its labelled (row-leading) segment is
    /// activated by keyboard - so a multi-day bar is grabbable along its whole length by mouse but
    /// tabs and keyboard-activates as one control per row. `date` is the clicked cell's day. Without
    /// this callback bars stay presentational; not called when [`render_span`](Self::render_span)
    /// takes over. A mouse click also bubbles to [`on_day_click`](Self::on_day_click) (the segment
    /// does not stop propagation); keyboard activation fires only this callback.
    #[builder_cb(IntoEventCallback, into_event_callback, CalendarSpanClick)]
    #[prop_or_default]
    pub on_span_click: Option<Callback<CalendarSpanClick>>,

    /// Renders rich hover content for a bar. When set, the default markup wraps every covered cell
    /// of the bar in a [`Tooltip`] carrying this content, so hovering or focusing anywhere along the
    /// bar shows the same popover (approval status, dates, ...) and the plain `title` is dropped, so
    /// the two never stack into a doubled-up hover. Ignored when [`render_span`](Self::render_span)
    /// takes over (a full hook builds its own tooltip).
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, CalendarSpanBar)]
    #[prop_or_default]
    pub render_span_tooltip: Option<RenderFn<CalendarSpanBar>>,
}

impl CalendarGrid {
    /// Create a new grid anchored at an ISO `YYYY-MM-DD` date.
    pub fn new(anchor: impl Into<AttrValue>) -> Self {
        yew::props! { Self { anchor: anchor.into() } }
    }

    /// Replace the whole set of span bars.
    pub fn span_bars(mut self, bars: impl IntoIterator<Item = CalendarSpanBar>) -> Self {
        self.span_bars = bars.into_iter().collect();
        self
    }

    /// Builder-style: add one span bar.
    pub fn with_span_bar(mut self, bar: CalendarSpanBar) -> Self {
        self.add_span_bar(bar);
        self
    }

    /// Add one span bar.
    pub fn add_span_bar(&mut self, bar: CalendarSpanBar) {
        self.span_bars.push(bar);
    }

    /// Render one segment: the application hook when set, else the built-in `pwt-calendar-span*`
    /// markup. The default markup carries the geometry classes (so a theme rounds the true
    /// start/end, squares the continuation edges, and accents the true start), the per-bar color
    /// and icon, the selected state, and the click/keyboard/aria wiring when
    /// [`on_span_click`](Self::on_span_click) is set. See [`CalendarSpanBar`] for the class
    /// contract.
    fn render_span_segment(&self, seg: CalendarSpanSegment) -> Html {
        if let Some(render) = &self.render_span {
            return render.apply(&seg);
        }

        // Only the row-leading cell carries the label; the covered days repeat neither text nor
        // icon and stay presentational so the bar reads (and tabs) as one control per row.
        let labelled = !seg.label.is_empty();
        let selected = match (self.selected_span.as_deref(), seg.bar.id.as_deref()) {
            (Some(sel), Some(id)) => sel == id,
            _ => false,
        };

        let mut class = seg.class();
        if selected {
            class.push("pwt-calendar-span-selected");
        }

        let mut segment = Container::from_tag("div").class(class);
        // Tie every covered cell of a bar to its id, so a hover on any one can light the whole run
        // (the consumer wires the cross-cell highlight in CSS keyed on this attribute).
        if let Some(id) = &seg.bar.id {
            segment = segment.attribute("data-span-id", id.clone());
        }
        // A rich render_span_tooltip covers the bar on every cell (below), so the native title would
        // only stack a second, differently-timed hover on top; keep the title solely as the fallback
        // for when no rich tip is set.
        if self.render_span_tooltip.is_none() {
            segment = segment.attribute("title", seg.bar.label.clone());
        }

        if let Some(color) = &seg.bar.color {
            segment = segment.style("--pwt-calendar-span-color", color.clone());
        }

        if labelled {
            if let Some(icon) = &seg.bar.icon {
                segment = segment.with_child(
                    Container::from_tag("span")
                        .class("pwt-calendar-span-icon")
                        .with_child(html! { <i class={icon.clone()} /> }),
                );
            }
        }

        // A non-breaking space keeps a continuation cell at bar height so the row does not
        // collapse.
        let text = if labelled {
            seg.label.clone()
        } else {
            "\u{00a0}".to_string()
        };
        segment = segment.with_child(
            Container::from_tag("span")
                .class("pwt-calendar-span-label")
                .with_child(html! { { text } }),
        );

        if let Some(on_click) = &self.on_span_click {
            let click = CalendarSpanClick {
                id: seg.bar.id.clone(),
                kind: seg.bar.kind.clone(),
                date: seg.date.clone(),
            };
            // Every covered day is mouse-clickable, so a multi-day bar can be grabbed anywhere
            // along its length, not only on its labelled cell; `date` is that cell's day.
            let onclick = {
                let on_click = on_click.clone();
                let click = click.clone();
                move |_: MouseEvent| on_click.emit(click.clone())
            };
            segment = segment.onclick(onclick);
            if labelled {
                // The labelled cell is the run's single keyboard control and tab stop.
                let onkeydown = {
                    let on_click = on_click.clone();
                    move |event: KeyboardEvent| match event.key().as_str() {
                        "Enter" | " " => {
                            event.prevent_default();
                            on_click.emit(click.clone());
                        }
                        _ => {}
                    }
                };
                segment = segment
                    .attribute("role", "button")
                    .attribute("tabindex", "0")
                    .attribute("aria-pressed", if selected { "true" } else { "false" })
                    .attribute("aria-label", seg.label.clone())
                    .onkeydown(onkeydown);
            } else {
                // A covered day stays out of the tab order and the a11y tree, so the bar reads (and
                // tabs) as one control per row even though every day of it is clickable.
                segment = segment
                    .attribute("role", "presentation")
                    .attribute("tabindex", "-1")
                    .attribute("aria-hidden", "true");
            }
        }

        // A rich hover / focus popover on every covered cell when the caller supplied one, so the
        // bar answers a hover the same anywhere along its run rather than the plain title on the
        // continuation cells and the rich tip only on the labelled one.
        if let Some(render) = &self.render_span_tooltip {
            return Tooltip::new(segment)
                .class("pwt-calendar-span-tip")
                .rich_tip(render.apply(&seg.bar))
                .into();
        }

        segment.into()
    }

    /// First and last visible date (inclusive) for a view around an anchor,
    /// for fetching data covering exactly the rendered window. `None` when
    /// the anchor is not a valid ISO date.
    pub fn visible_range(
        view: CalendarGridView,
        anchor: &str,
        week_start: WeekStart,
    ) -> Option<(String, String)> {
        let (start, count) = window(view, anchor, week_start)?;
        Some((format_days(start), format_days(start + count - 1)))
    }
}

/// First visible day count and cell count for a view, or `None` when the anchor is not a valid ISO
/// date. Shared by [`CalendarGrid::visible_range`] and the renderer so both span the same window.
fn window(view: CalendarGridView, anchor: &str, week_start: WeekStart) -> Option<(i64, i64)> {
    let (y, m, d) = parse_iso_date(anchor)?;
    let start = week_start.sunday_based_index();
    match view {
        CalendarGridView::Week => {
            let days = days_from_civil(y, m, d);
            Some((days - week_offset(days, start), 7))
        }
        CalendarGridView::Month => {
            let first = days_from_civil(y, m, 1);
            Some((first - week_offset(first, start), 42))
        }
    }
}

/// Day count of the civil date since 1970-01-01 (Howard Hinnant's algorithm),
/// negative for earlier dates.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = ((m + 9) % 12) as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Inverse of [`days_from_civil`].
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

/// Weekday of a day count with Monday = 0 (1970-01-01 was a Thursday).
fn weekday_monday0(days: i64) -> u32 {
    (days + 3).rem_euclid(7) as u32
}

/// Days from `days` back to the most recent week start, with `start` in the Sunday = 0 space.
fn week_offset(days: i64, start: u32) -> i64 {
    // 1970-01-01 (day 0) was a Thursday, index 4 in the Sunday = 0 space.
    let weekday = (days + 4).rem_euclid(7);
    (weekday - start as i64).rem_euclid(7)
}

/// The week start expressed in the Monday = 0 space used for header labels.
fn week_start_monday0(week_start: WeekStart) -> u32 {
    (week_start.sunday_based_index() + 6) % 7
}

/// Number of days in a Gregorian month.
fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    }
}

fn parse_iso_date(s: &str) -> Option<(i64, u32, u32)> {
    let mut parts = s.splitn(3, '-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
        return None;
    }
    Some((y, m, d))
}

fn format_days(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

fn weekday_label(weekday_monday0: u32) -> String {
    match weekday_monday0 {
        0 => tr!("Mon"),
        1 => tr!("Tue"),
        2 => tr!("Wed"),
        3 => tr!("Thu"),
        4 => tr!("Fri"),
        5 => tr!("Sat"),
        _ => tr!("Sun"),
    }
}

/// The geometry of `bar` on the cell for `date` at column `col` (0..6) within its week row, or
/// `None` when the bar does not cover the day. ISO dates compare lexically, so no date arithmetic
/// is needed.
fn span_segment(
    bar: &CalendarSpanBar,
    date: &str,
    col: u32,
    show_start_marker: bool,
) -> Option<CalendarSpanSegment> {
    if date < bar.start.as_str() || date > bar.end.as_str() {
        return None;
    }
    let is_start = date == bar.start;
    let is_end = date == bar.end;
    // A cell joins its left/right neighbour only mid-span and away from a week-row edge; the row
    // break at col 0 / col 6 squares the edge even when the span continues into the next row.
    let continues_left = !is_start && col != 0;
    let continues_right = !is_end && col != 6;
    // Leftmost cell of the bar in this row but not the true start (a row-break continuation): the
    // start marker stays off so the segment reads as carried over from the previous row.
    let row_continues_from_prev = !is_start && !continues_left;
    let row_continues_to_next = !is_end && !continues_right;
    // Title once per week row: on the bar's true start and on a row's leading continuation cell.
    let label = if is_start || row_continues_from_prev {
        bar.label.clone()
    } else {
        String::new()
    };
    Some(CalendarSpanSegment {
        bar: bar.clone(),
        date: date.to_string(),
        is_start,
        is_end,
        continues_left,
        continues_right,
        row_continues_from_prev,
        row_continues_to_next,
        label,
        show_start_marker,
    })
}

impl crate::props::IntoVTag for CalendarGrid {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        let header_start = week_start_monday0(self.week_start);
        let header: Html = (0..7u32)
            .fold(
                Container::from_tag("div").class("pwt-calendar-grid-header"),
                |grid, i| {
                    grid.with_child(
                        Container::from_tag("div")
                            .class("pwt-calendar-weekday")
                            .with_child(html! { { weekday_label((header_start + i) % 7) } }),
                    )
                },
            )
            .into();

        // An unparsable anchor renders the header over an empty grid rather than panicking deep
        // inside the view.
        let (start, count) = window(self.view, &self.anchor, self.week_start).unwrap_or((0, 0));

        let anchor_month: String = self.anchor.chars().take(7).collect();
        let today = self.today.as_deref().unwrap_or("");

        let mut grid = Container::from_tag("div").class("pwt-calendar-grid");

        // Start order, so an earlier bar never drops below a later one (no crossing). A bar still
        // compacts upward as the ones above it end.
        let mut sorted_bars: Vec<&CalendarSpanBar> = self.span_bars.iter().collect();
        sorted_bars.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));

        // Pinned bars ride a fixed zone above the scrolling lanes. Each keeps one lane for its
        // whole run (greedy interval colouring over the sorted set), so a multi-day pinned bar
        // sits at the same height in every cell it spans; a week row reserves its busiest cell's
        // lane count and holds the gaps open with spacers, so the zone height (and the scroll box
        // below it) stays uniform across the row.
        // Higher priority stacks nearer the top of the pinned zone (a lower lane); the greedy
        // colouring below assigns lanes in this order, so priority wins over the date order flow
        // bars keep.
        let mut pinned_bars: Vec<&CalendarSpanBar> =
            sorted_bars.iter().copied().filter(|b| b.pinned).collect();
        pinned_bars.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.start.cmp(&b.start))
                .then_with(|| a.end.cmp(&b.end))
        });
        let flow_bars: Vec<&CalendarSpanBar> =
            sorted_bars.iter().copied().filter(|b| !b.pinned).collect();
        let mut lane_end: Vec<&str> = Vec::new();
        let pinned_lane: Vec<usize> = pinned_bars
            .iter()
            .map(|bar| {
                let lane = lane_end
                    .iter()
                    .position(|end| *end < bar.start.as_str())
                    .unwrap_or(lane_end.len());
                if lane == lane_end.len() {
                    lane_end.push(bar.end.as_str());
                } else {
                    lane_end[lane] = bar.end.as_str();
                }
                lane
            })
            .collect();
        let num_rows = ((count + 6) / 7) as usize;
        let mut row_pinned_count = vec![0usize; num_rows];
        for (i, bar) in pinned_bars.iter().enumerate() {
            for offset in 0..count {
                let (yy, mm, dd) = civil_from_days(start + offset);
                let date = format!("{yy:04}-{mm:02}-{dd:02}");
                if bar.start.as_str() <= date.as_str() && date.as_str() <= bar.end.as_str() {
                    let row = (offset / 7) as usize;
                    row_pinned_count[row] = row_pinned_count[row].max(pinned_lane[i] + 1);
                }
            }
        }

        for offset in 0..count {
            let days = start + offset;
            let (y, m, d) = civil_from_days(days);
            let date = format!("{y:04}-{m:02}-{d:02}");
            let weekday = weekday_monday0(days);
            let info = CalendarGridDay {
                day: d,
                weekday,
                // A week view's anchor is that week's first day, so a week straddling a month
                // boundary would otherwise mark the days on the other side as outside the month.
                in_anchor_month: matches!(self.view, CalendarGridView::Week)
                    || date.starts_with(&anchor_month),
                is_today: date == today,
                is_weekend: weekday >= 5,
                date,
            };

            let mut cell = Container::from_tag("div").class("pwt-calendar-day");
            if !info.in_anchor_month {
                cell = cell.class("pwt-calendar-day-outside");
            }
            if info.is_today {
                cell = cell.class("pwt-calendar-day-today");
            }
            if info.is_weekend {
                cell = cell.class("pwt-calendar-day-weekend");
            }
            if let Some(class_fn) = &self.day_class {
                cell = cell.class(class_fn.apply(&info));
            }
            if let Some(on_click) = &self.on_day_click {
                let on_click = on_click.clone();
                let click_info = info.clone();
                cell = cell.onclick(move |_| on_click.emit(click_info.clone()));
            }
            if let Some(on_dblclick) = &self.on_day_dblclick {
                let on_dblclick = on_dblclick.clone();
                let dbl_info = info.clone();
                cell = cell.ondblclick(move |_| on_dblclick.emit(dbl_info.clone()));
            }

            let day_header: Html = match &self.render_day_header {
                Some(render) => render.apply(&info),
                None => Container::from_tag("span")
                    .class("pwt-calendar-daynum")
                    .with_child(html! { { info.day.to_string() } })
                    .into(),
            };
            cell = cell.with_child(day_header);

            // The cell body sits above the span bars, so a caller can keep its own content in view
            // while the bars scroll. Bars live in their own scroll box: a day with dozens of them
            // scrolls that cell rather than growing the row. The column index within the week row
            // (cells flow 7 per row) decides the row-break edges. An uncovered day gets no box.
            if let Some(render) = &self.render_day {
                cell = cell.with_child(render.apply(&info));
            }

            let col = (offset % 7) as u32;

            // The pinned zone, above the scroll box: one row of the reserved lanes, each holding
            // its pinned bar's segment for this day or an equal-height spacer.
            let reserved = row_pinned_count
                .get((offset / 7) as usize)
                .copied()
                .unwrap_or(0);
            if reserved > 0 {
                let mut slots: Vec<Option<CalendarSpanSegment>> =
                    (0..reserved).map(|_| None).collect();
                for (i, bar) in pinned_bars.iter().enumerate() {
                    if let Some(seg) = span_segment(bar, &info.date, col, self.span_start_marker) {
                        // `reserved` is this row's max pinned lane + 1, so a bar covering a day in
                        // the row always has `pinned_lane[i] < reserved`; the guard is a bounds
                        // safety net, never a real drop.
                        if pinned_lane[i] < reserved {
                            slots[pinned_lane[i]] = Some(seg);
                        }
                    }
                }
                // A spacer must occupy exactly a bar's box (`--pwt-calendar-span-height` plus the
                // `.pwt-calendar-span` vertical margin) or a pinned lane would drift between a
                // covered and an uncovered cell; the two are tied in the theme's calendar partial.
                let mut zone = Container::from_tag("div").class("pwt-calendar-span-pinned");
                for slot in slots {
                    zone = zone.with_child(match slot {
                        Some(seg) => self.render_span_segment(seg),
                        None => Container::from_tag("div")
                            .class("pwt-calendar-span-spacer")
                            .into(),
                    });
                }
                cell = cell.with_child(zone);
            }

            // The scrolling zone: a day with dozens of bars scrolls this box rather than growing.
            if !flow_bars.is_empty() {
                let segments: Vec<CalendarSpanSegment> = flow_bars
                    .iter()
                    .filter_map(|bar| span_segment(bar, &info.date, col, self.span_start_marker))
                    .collect();
                if !segments.is_empty() {
                    let mut lanes = Container::from_tag("div").class("pwt-calendar-span-lanes");
                    for seg in segments {
                        lanes = lanes.with_child(self.render_span_segment(seg));
                    }
                    cell = cell.with_child(lanes);
                }
            }

            grid = grid.with_child(cell);
        }

        let children = vec![header, grid.into()];

        let mut this = self;
        if this.fill_height {
            this.add_class("pwt-calendar-fill");
        }

        this.std_props.into_vtag(
            Cow::Borrowed("div"),
            node_ref,
            Some("pwt-calendar"),
            None,
            Some(children),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use yew::virtual_dom::VNode;

    use super::*;

    /// Flatten a rendered segment's html attributes (including `class` and `style`) into a map.
    fn segment_attrs(html: &Html) -> HashMap<String, String> {
        match html {
            VNode::VTag(tag) => tag
                .attributes
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            other => panic!("span segment is not a VTag: {other:?}"),
        }
    }

    #[test]
    fn default_segment_wires_button_and_selection() {
        // A labelled (row-leading) segment of a selected bar, under on_span_click, is a button
        // reflecting the selection; a covered-day continuation cell stays presentational.
        let bar = CalendarSpanBar::new("2026-06-08", "2026-06-10", "Sprint")
            .id("run-1")
            .color("#2e7d32")
            .icon("fa fa-plane");
        let grid = CalendarGrid::new("2026-06-08")
            .selected_span("run-1")
            .on_span_click(|_: CalendarSpanClick| {});

        let start = span_segment(&bar, "2026-06-08", 0, true).unwrap();
        let attrs = segment_attrs(&grid.render_span_segment(start));
        assert_eq!(attrs.get("role").map(String::as_str), Some("button"));
        assert_eq!(attrs.get("tabindex").map(String::as_str), Some("0"));
        assert_eq!(attrs.get("aria-pressed").map(String::as_str), Some("true"));
        assert_eq!(attrs.get("aria-label").map(String::as_str), Some("Sprint"));
        assert!(attrs["class"].contains("pwt-calendar-span-selected"));
        assert!(attrs["style"].contains("--pwt-calendar-span-color"));

        let mid = span_segment(&bar, "2026-06-09", 1, true).unwrap();
        let mid_attrs = segment_attrs(&grid.render_span_segment(mid));
        assert_eq!(
            mid_attrs.get("role").map(String::as_str),
            Some("presentation")
        );
        assert_eq!(mid_attrs.get("tabindex").map(String::as_str), Some("-1"));
        assert_eq!(
            mid_attrs.get("aria-hidden").map(String::as_str),
            Some("true")
        );
        assert!(!mid_attrs.contains_key("aria-pressed"));
    }

    #[test]
    fn default_segment_stays_presentational_without_click() {
        // No on_span_click and no matching selection: bars carry geometry only, no role/tabindex.
        let bar = CalendarSpanBar::new("2026-06-08", "2026-06-10", "Sprint").id("run-1");
        let grid = CalendarGrid::new("2026-06-08");
        let start = span_segment(&bar, "2026-06-08", 0, true).unwrap();
        let attrs = segment_attrs(&grid.render_span_segment(start));
        assert!(!attrs.contains_key("role"));
        assert!(!attrs.contains_key("tabindex"));
        assert!(!attrs["class"].contains("pwt-calendar-span-selected"));
    }

    #[test]
    fn span_segment_geometry_within_a_week_row() {
        // A Mon..Wed bar in a Monday-start row (cols 0,1,2): rounded start, squared middle, rounded
        // end, fusing into one bar.
        let bar = CalendarSpanBar::new("2026-06-08", "2026-06-10", "Sprint");
        let start = span_segment(&bar, "2026-06-08", 0, true).unwrap();
        assert!(start.is_start && !start.is_end);
        assert!(!start.continues_left && start.continues_right);
        assert_eq!(start.label, "Sprint");

        let mid = span_segment(&bar, "2026-06-09", 1, true).unwrap();
        assert!(!mid.is_start && !mid.is_end);
        assert!(mid.continues_left && mid.continues_right);
        assert_eq!(mid.label, ""); // titled only once per row

        let end = span_segment(&bar, "2026-06-10", 2, true).unwrap();
        assert!(end.is_end && end.continues_left && !end.continues_right);

        // Days outside the range yield no segment.
        assert!(span_segment(&bar, "2026-06-07", 6, true).is_none());
        assert!(span_segment(&bar, "2026-06-11", 3, true).is_none());
    }

    #[test]
    fn span_segment_breaks_at_week_row_boundaries() {
        // A bar crossing a week boundary: the row's last cell (col 6) squares its right edge with
        // no bleed, and the next row's first cell (col 0) is a continuation - no start marker, but
        // re-titled so the second row's bar is still labelled.
        let bar = CalendarSpanBar::new("2026-06-13", "2026-06-15", "Trip");
        let sat = span_segment(&bar, "2026-06-13", 5, true).unwrap();
        assert!(sat.is_start && sat.continues_right);
        let sun = span_segment(&bar, "2026-06-14", 6, true).unwrap();
        assert!(!sun.continues_right && sun.continues_left);
        assert!(sun.row_continues_to_next);
        let mon = span_segment(&bar, "2026-06-15", 0, true).unwrap();
        assert!(!mon.is_start && !mon.continues_left); // row-break continuation, edge squared
        assert!(mon.row_continues_from_prev);
        assert_eq!(mon.label, "Trip"); // re-titled on the new row's leading cell
    }

    #[test]
    fn span_segment_month_boundary_continuation_has_no_marker() {
        // A bar whose true start is off the visible window: its first visible cell must read as a
        // continuation (no start marker) even at col 0.
        let bar = CalendarSpanBar::new("2026-05-28", "2026-06-03", "Leave");
        let first_visible = span_segment(&bar, "2026-06-01", 0, true).unwrap();
        assert!(!first_visible.is_start);
        assert!(first_visible.row_continues_from_prev);
        assert!(!first_visible.continues_left);
    }

    #[test]
    fn civil_roundtrip_and_weekday() {
        // 1970-01-01 is day 0, a Thursday (Monday = 0 -> 3).
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(weekday_monday0(0), 3);
        // 2026-06-10 is a Wednesday.
        let days = days_from_civil(2026, 6, 10);
        assert_eq!(weekday_monday0(days), 2);
        assert_eq!(civil_from_days(days), (2026, 6, 10));
        // Leap-day round trip.
        let leap = days_from_civil(2024, 2, 29);
        assert_eq!(civil_from_days(leap), (2024, 2, 29));
    }

    #[test]
    fn week_window_follows_week_start() {
        // 2026-06-10 is a Wednesday; the Monday-start week runs Mon..Sun.
        let (from, to) =
            CalendarGrid::visible_range(CalendarGridView::Week, "2026-06-10", WeekStart::Monday)
                .unwrap();
        assert_eq!((from.as_str(), to.as_str()), ("2026-06-08", "2026-06-14"));
        // A Monday anchors its own Monday-start week.
        let (from, to) =
            CalendarGrid::visible_range(CalendarGridView::Week, "2026-06-08", WeekStart::Monday)
                .unwrap();
        assert_eq!((from.as_str(), to.as_str()), ("2026-06-08", "2026-06-14"));
        // A Sunday-start week around the same Wednesday backs up to the prior Sunday.
        let (from, to) =
            CalendarGrid::visible_range(CalendarGridView::Week, "2026-06-10", WeekStart::Sunday)
                .unwrap();
        assert_eq!((from.as_str(), to.as_str()), ("2026-06-07", "2026-06-13"));
    }

    #[test]
    fn month_window_is_42_cells_from_leading_week_start() {
        // June 2026 starts on a Monday: a Monday-start window is exactly the 1st plus 41 days.
        let (from, to) =
            CalendarGrid::visible_range(CalendarGridView::Month, "2026-06-10", WeekStart::Monday)
                .unwrap();
        assert_eq!((from.as_str(), to.as_str()), ("2026-06-01", "2026-07-12"));
        // August 2026 starts on a Saturday: the window backs up to Monday July 27.
        let (from, to) =
            CalendarGrid::visible_range(CalendarGridView::Month, "2026-08-15", WeekStart::Monday)
                .unwrap();
        assert_eq!((from.as_str(), to.as_str()), ("2026-07-27", "2026-09-06"));
    }

    #[test]
    fn invalid_anchor_yields_no_range() {
        let bad = |anchor| {
            CalendarGrid::visible_range(CalendarGridView::Month, anchor, WeekStart::Monday)
                .is_none()
        };
        assert!(bad("garbage"));
        assert!(bad("2026-13-01"));
        // Impossible in-month dates are rejected, not silently rolled forward.
        assert!(bad("2026-02-31"));
        assert!(bad("2026-04-31"));
        assert!(bad("2026-02-29")); // 2026 is not a leap year
        // The leap day itself stays valid.
        assert!(!bad("2024-02-29"));
    }
}
