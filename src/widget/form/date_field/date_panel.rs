use crate::prelude::*;
use crate::widget::{Button, Container, Row, WeekStart};

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::VNode;

use super::plain_date::PlainDate;

use pwt_macros::builder;
/// A panel component for selecting dates.
#[builder]
#[derive(Properties, PartialEq, Clone)]
pub struct DatePanel {
    /// The currently selected date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<PlainDate>,

    /// Callback triggered when a date is selected.
    #[builder_cb(IntoEventCallback, into_event_callback, PlainDate)]
    #[prop_or_default]
    pub on_select: Option<Callback<PlainDate>>,
    /// The day the calendar week starts on. Defaults to [`WeekStart::Sunday`].
    #[builder]
    #[prop_or(WeekStart::Sunday)]
    pub week_start: WeekStart,

    /// Deprecated: prefer [`week_start`](Self::week_start). The first day of the
    /// week as a 0-based index (0 = Sunday). When set it overrides `week_start`.
    #[deprecated(note = "use `week_start` instead")]
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub start_day: Option<u32>,

    /// An array of days to disable, 0-based. For example, [0, 6] disables Sunday and Saturday.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub disabled_days: Vec<u32>,

    /// False to hide the footer area containing the Today button and disable the keyboard
    /// handler for spacebar that selects the current date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(true)]
    pub show_today: bool,

    /// The minimum allowed date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub min_value: Option<PlainDate>,

    /// The maximum allowed date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub max_value: Option<PlainDate>,

    /// Callback to disable specific dates. Returns true if the date should be disabled.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub disabled_dates: Option<Callback<PlainDate, bool>>,

    /// Show the week numbers.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(true)]
    pub show_week_numbers: bool,
}

impl DatePanel {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

impl From<DatePanel> for VNode {
    fn from(props: DatePanel) -> Self {
        html! { <DatePanelComp ..props /> }
    }
}

pub enum Msg {
    PrevMonth,
    NextMonth,
    SelectDate(PlainDate),
    Today,
    KeyDown(KeyboardEvent),
    ToggleViewMode,
    SelectYear(i32),
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum ViewMode {
    Day,
    Year,
}

pub struct DatePanelComp {
    view_date: PlainDate,
    focused_date: Option<PlainDate>,
    panel_ref: NodeRef,
    mode: ViewMode,
}

impl DatePanelComp {
    fn is_date_disabled(
        date: PlainDate,
        disabled_days: &[u32],
        min_value: Option<PlainDate>,
        max_value: Option<PlainDate>,
        disabled_dates: Option<&Callback<PlainDate, bool>>,
    ) -> bool {
        if disabled_days.contains(&date.week_day()) {
            return true;
        }
        if let Some(min) = min_value {
            if date < min {
                return true;
            }
        }
        if let Some(max) = max_value {
            if date > max {
                return true;
            }
        }
        if let Some(callback) = disabled_dates {
            if callback.emit(date) {
                return true;
            }
        }
        false
    }

    fn find_next_selectable_day(
        base_date: PlainDate,
        initial_offset: i32,
        disabled_days: &[u32],
        min_value: Option<PlainDate>,
        max_value: Option<PlainDate>,
        disabled_dates: Option<&Callback<PlainDate, bool>>,
    ) -> Option<PlainDate> {
        // Calculate the direction to search: +1 day or -1 day.
        // For Left/Up we search backwards (-1), for Right/Down we search forwards (+1).
        let step = if initial_offset > 0 { 1 } else { -1 };

        // Scan for the next enabled day.
        let mut candidate = base_date.add_days(initial_offset);
        let mut count = 0;

        // Limit the search to prevent infinite loops (e.g. if all future days are disabled).
        // A year scan should be sufficient for reasonable UI.
        while count < 366 {
            if !Self::is_date_disabled(
                candidate,
                disabled_days,
                min_value,
                max_value,
                disabled_dates,
            ) {
                return Some(candidate);
            }
            candidate = candidate.add_days(step);
            count += 1;
        }
        None
    }

    fn render_years(&self, ctx: &Context<Self>) -> Html {
        let view_year = self.view_date.year();
        let start_year = (view_year / 25) * 25;

        let mut grid = Container::new()
            .class("pwt-date-panel")
            .style("grid-template-columns", "repeat(5, 1fr)");

        for i in 0..25 {
            let year = start_year + i;
            let mut cell = Container::new().class("pwt-date-panel-item");

            // is_selected: matches the actual committed value of the component
            let is_selected = ctx.props().value.map(|d| d.year()) == Some(year);
            // is_focused: matches the temporary cursor during navigation
            let is_focused = self.focused_date.map(|d| d.year()) == Some(year);

            // Check min/max
            let mut disabled = false;
            if let Some(min) = ctx.props().min_value {
                if year < min.year() {
                    disabled = true;
                }
            }
            if let Some(max) = ctx.props().max_value {
                if year > max.year() {
                    disabled = true;
                }
            }

            if disabled {
                cell = cell.class("disabled");
            } else {
                if is_selected {
                    cell = cell.class("selected");
                } else if is_focused {
                    cell = cell.class("focused");
                }

                cell = cell.onclick(ctx.link().callback(move |_| Msg::SelectYear(year)));
            }

            grid = grid.with_child(cell.with_child(year.to_string()));
        }

        grid.into()
    }

    fn render_days(&self, ctx: &Context<Self>) -> Html {
        let selected = ctx.props().value;
        let view_year = self.view_date.year();
        let view_month = self.view_date.month(); // 0-11
        // `start_day` (deprecated) overrides `week_start` when a caller still sets it.
        let start_day = match ctx.props().start_day {
            Some(d) => d % 7,
            None => ctx.props().week_start.sunday_based_index(),
        };
        let show_week_numbers = ctx.props().show_week_numbers;

        // Grid generation using PlainDate
        // Start from 1st of month
        let first_of_month = PlainDate::new(view_year, view_month, 1);
        let wday = first_of_month.week_day(); // 0 (Sun) - 6 (Sat)

        // Calculate days to subtract to reach the start_day of the week (going backwards)
        // formula: (current_day - target_start_day + 7) % 7
        let days_to_subtract = (wday as i32 - start_day as i32 + 7) % 7;

        let start_date = first_of_month.add_days(-days_to_subtract);

        let next_month_start = if view_month == 11 {
            PlainDate::new(view_year + 1, 0, 1)
        } else {
            PlainDate::new(view_year, view_month + 1, 1)
        };

        let week_days_source = [
            tr!("Sun"),
            tr!("Mon"),
            tr!("Tue"),
            tr!("Wed"),
            tr!("Thu"),
            tr!("Fri"),
            tr!("Sat"),
        ];

        // Rotate week days based on start_day
        let mut week_days = Vec::new();
        for i in 0..7 {
            week_days.push(week_days_source[(start_day as usize + i) % 7].clone());
        }

        let columns = if show_week_numbers { 8 } else { 7 };
        let mut grid = Container::new()
            .class("pwt-date-panel")
            .style("grid-template-columns", format!("repeat({}, 1fr)", columns));

        if show_week_numbers {
            grid = grid.with_child(
                Container::new()
                    .class("pwt-date-panel-week-number")
                    .with_child("#"),
            );
        }

        for name in week_days.iter() {
            grid = grid.with_child(
                Container::new()
                    .class("pwt-date-panel-header")
                    .with_child(name.to_string()),
            );
        }

        let today = PlainDate::today();

        let mut curr = start_date;
        for _ in 0..6 {
            if curr >= next_month_start {
                break;
            }

            if show_week_numbers {
                // Find Thursday in this row to determine ISO week for the row
                let mut week_num = 0;
                // Check next 7 days
                for i in 0..7 {
                    let d = curr.add_days(i);
                    // 4 = Thursday (Js Date)
                    if d.week_day() == 4 {
                        week_num = d.iso_week();
                        break;
                    }
                }
                // Fallback (should shouldn't happen for 7 day window)
                if week_num == 0 {
                    week_num = curr.add_days(3).iso_week();
                }

                grid = grid.with_child(
                    Container::new()
                        .class("pwt-date-panel-week-number")
                        .with_child(week_num.to_string()),
                );
            }

            for _ in 0..7 {
                let d = curr;
                let is_other_month = d.month() != view_month;
                let d_clone = d;

                let is_selected = selected == Some(d);
                let is_focused = self.focused_date == Some(d);

                let is_disabled = Self::is_date_disabled(
                    d,
                    &ctx.props().disabled_days,
                    ctx.props().min_value,
                    ctx.props().max_value,
                    ctx.props().disabled_dates.as_ref(),
                );

                let mut cell = Container::new().class("pwt-date-panel-item");

                if is_disabled {
                    cell = cell.class("disabled");
                } else if is_selected {
                    cell = cell.class("selected");
                } else if is_focused {
                    cell = cell.class("focused");
                } else if is_other_month {
                    cell = cell.class("other-month");
                }

                if d == today && !is_selected {
                    cell = cell.class("pwt-today");
                }

                if !is_disabled {
                    cell = cell.onclick(ctx.link().callback(move |_| Msg::SelectDate(d_clone)));
                }

                grid = grid.with_child(cell.with_child(d.day().to_string()));

                curr = curr.add_days(1);
            }
        }

        grid.into()
    }
    fn handle_keydown(&mut self, ctx: &Context<Self>, e: KeyboardEvent) -> bool {
        let key = e.key();

        if key == "Enter" || key == " " {
            e.prevent_default();
            if self.mode == ViewMode::Year {
                if let Some(focus) = self.focused_date {
                    ctx.link().send_message(Msg::SelectYear(focus.year()));
                    return false;
                }
            }

            if let Some(focus) = self.focused_date {
                ctx.link().send_message(Msg::SelectDate(focus));
            } else if let Some(val) = ctx.props().value {
                ctx.link().send_message(Msg::SelectDate(val));
            }
            return false;
        }

        let (day_delta, year_delta) = match key.as_str() {
            "ArrowLeft" => (-1, -1),
            "ArrowRight" => (1, 1),
            "ArrowUp" => (-7, -5),
            "ArrowDown" => (7, 5),
            _ => return false,
        };

        // Move focus back to the panel
        crate::dom::focus::focus_node(&self.panel_ref);
        e.prevent_default();

        let current_focus = self
            .focused_date
            .or(ctx.props().value)
            .unwrap_or(self.view_date);

        match self.mode {
            ViewMode::Day => {
                if let Some(new_focus) = Self::find_next_selectable_day(
                    current_focus,
                    day_delta,
                    &ctx.props().disabled_days,
                    ctx.props().min_value,
                    ctx.props().max_value,
                    ctx.props().disabled_dates.as_ref(),
                ) {
                    self.focused_date = Some(new_focus);
                    self.view_date = new_focus;
                    true
                } else {
                    false
                }
            }
            ViewMode::Year => {
                let view_year = self.view_date.year();
                let start_year = (view_year / 25) * 25;
                let end_year = start_year + 24;
                let mut y = current_focus.year();

                if y < start_year || y > end_year {
                    y = view_year;
                }

                let mut count = 0;
                while count < 100 {
                    y += year_delta;
                    count += 1;
                    let candidate = PlainDate::new(y, current_focus.month(), 1);

                    let mut valid = true;
                    if let Some(min) = ctx.props().min_value {
                        if candidate.year() < min.year() {
                            valid = false;
                        }
                    }
                    if let Some(max) = ctx.props().max_value {
                        if candidate.year() > max.year() {
                            valid = false;
                        }
                    }

                    if valid {
                        self.focused_date = Some(candidate);
                        self.view_date = candidate;
                        return true;
                    }
                }
                false
            }
        }
    }
}

impl Component for DatePanelComp {
    type Message = Msg;
    type Properties = DatePanel;

    fn create(ctx: &Context<Self>) -> Self {
        let today = PlainDate::today();
        let view_date = ctx.props().value.unwrap_or(today);

        Self {
            view_date,
            focused_date: ctx.props().value,
            panel_ref: NodeRef::default(),
            mode: ViewMode::Day,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PrevMonth => {
                match self.mode {
                    ViewMode::Day => {
                        let mut y = self.view_date.year();
                        let mut m = self.view_date.month();
                        if m > 0 {
                            m -= 1;
                        } else {
                            y -= 1;
                            m = 11;
                        }
                        self.view_date = PlainDate::new(y, m, 1);
                    }
                    ViewMode::Year => {
                        let y = self.view_date.year() - 25;
                        self.view_date = PlainDate::new(y, self.view_date.month(), 1);
                    }
                }
                true
            }
            Msg::NextMonth => {
                match self.mode {
                    ViewMode::Day => {
                        let mut y = self.view_date.year();
                        let mut m = self.view_date.month();
                        if m < 11 {
                            m += 1;
                        } else {
                            y += 1;
                            m = 0;
                        }
                        self.view_date = PlainDate::new(y, m, 1);
                    }
                    ViewMode::Year => {
                        let y = self.view_date.year() + 25;
                        self.view_date = PlainDate::new(y, self.view_date.month(), 1);
                    }
                }
                true
            }
            Msg::SelectDate(date) => {
                self.focused_date = Some(date); // Sync focus
                self.view_date = date; // Sync view
                if let Some(on_select) = &ctx.props().on_select {
                    on_select.emit(date);
                }
                true
            }
            Msg::Today => {
                let today = PlainDate::today();
                if self.focused_date == Some(today) {
                    ctx.link().send_message(Msg::SelectDate(today));
                    false
                } else {
                    self.view_date = today;
                    self.focused_date = Some(today);
                    true
                }
            }
            Msg::ToggleViewMode => {
                self.mode = match self.mode {
                    ViewMode::Day => ViewMode::Year,
                    ViewMode::Year => ViewMode::Day,
                };
                true
            }
            Msg::SelectYear(year) => {
                let m = self.view_date.month();
                self.view_date = PlainDate::new(year, m, 1);
                self.focused_date = Some(self.view_date);

                self.mode = ViewMode::Day;
                true
            }
            Msg::KeyDown(e) => self.handle_keydown(ctx, e),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().value != _old_props.value {
            self.focused_date = ctx.props().value;
            if let Some(val) = ctx.props().value {
                self.view_date = val;
            }
            return true;
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let view_year = self.view_date.year();
        let view_month = self.view_date.month();

        // Calculate header text
        let header_text = match self.mode {
            ViewMode::Day => {
                let month_names = [
                    tr!("January"),
                    tr!("February"),
                    tr!("March"),
                    tr!("April"),
                    tr!("May"),
                    tr!("June"),
                    tr!("July"),
                    tr!("August"),
                    tr!("September"),
                    tr!("October"),
                    tr!("November"),
                    tr!("December"),
                ];
                let month_name = month_names
                    .get(view_month as usize)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                format!("{} {}", month_name, view_year)
            }
            ViewMode::Year => {
                let start_year = (view_year / 25) * 25;
                let end_year = start_year + 24;
                format!("{} - {}", start_year, end_year)
            }
        };

        // We need to stop propagation of the keydown event
        let suppress_keydown = ctx.link().batch_callback(|e: KeyboardEvent| {
            if e.key() == "Enter" || e.key() == " " {
                e.stop_propagation();
            }
            None
        });

        let header = Row::new()
            .class("pwt-font-title-medium")
            .style("justify-content", "space-between")
            .style("align-items", "center")
            .padding(1)
            .with_child(
                Button::new_icon("fa fa-angle-double-left")
                    .on_activate(ctx.link().callback(|_| Msg::PrevMonth))
                    .onkeydown(suppress_keydown.clone())
                    .class("pwt-button-text rounded"),
            )
            .with_child(
                Button::new(header_text)
                    .on_activate(ctx.link().callback(|_| Msg::ToggleViewMode))
                    .onkeydown(suppress_keydown.clone())
                    .class("pwt-button-text rounded pwt-color-primary")
                    .style("font-weight", "bold"),
            )
            .with_child(
                Button::new_icon("fa fa-angle-double-right")
                    .on_activate(ctx.link().callback(|_| Msg::NextMonth))
                    .onkeydown(suppress_keydown.clone())
                    .class("pwt-button-text rounded"),
            );

        let content = match self.mode {
            ViewMode::Day => self.render_days(ctx),
            ViewMode::Year => self.render_years(ctx),
        };

        let footer = ctx.props().show_today.then(|| {
            let (label, action) = match self.mode {
                ViewMode::Day => (tr!("Today"), ctx.link().callback(|_| Msg::Today)),
                ViewMode::Year => (
                    tr!("This Year"),
                    ctx.link()
                        .callback(|_| Msg::SelectYear(PlainDate::today().year())),
                ),
            };

            Container::new()
                .class(crate::css::TextAlign::Center)
                .padding_y(1)
                .border_top(true)
                .with_child(
                    Button::new(label)
                        .on_activate(action)
                        .onkeydown(suppress_keydown),
                )
        });

        let mode = self.mode;
        let panel = Container::new()
            .width("min-content")
            .min_width("250px")
            .class(crate::css::UserSelect::None)
            .class("selected")
            .attribute("tabindex", "0")
            .onkeydown(ctx.link().batch_callback(move |e: KeyboardEvent| {
                if e.key() == "Escape" && mode == ViewMode::Year {
                    e.stop_propagation();
                    e.prevent_default();
                    return Some(Msg::ToggleViewMode);
                }
                Some(Msg::KeyDown(e))
            }))
            .with_child(header)
            .with_child(content)
            .with_optional_child(footer);

        panel.into_html_with_ref(self.panel_ref.clone())
    }
}
