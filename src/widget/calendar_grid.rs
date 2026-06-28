use std::borrow::Cow;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::{builder, widget};

use crate::props::{
    ContainerBuilder, EventSubscriber, IntoOptionalRenderFn, RenderFn, WidgetBuilder,
};
use crate::tr;
use crate::widget::{Container, WeekStart};

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
}

impl CalendarGrid {
    /// Create a new grid anchored at an ISO `YYYY-MM-DD` date.
    pub fn new(anchor: impl Into<AttrValue>) -> Self {
        yew::props! { Self { anchor: anchor.into() } }
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

        for offset in 0..count {
            let days = start + offset;
            let (y, m, d) = civil_from_days(days);
            let date = format!("{y:04}-{m:02}-{d:02}");
            let weekday = weekday_monday0(days);
            let info = CalendarGridDay {
                day: d,
                weekday,
                in_anchor_month: date.starts_with(&anchor_month),
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

            if let Some(render) = &self.render_day {
                cell = cell.with_child(render.apply(&info));
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
    use super::*;

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
