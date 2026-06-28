/// The day a calendar week starts on.
///
/// Shared by the calendar-style widgets ([`CalendarGrid`] and the [`DateField`] picker) so they
/// agree on a single first-day-of-week instead of each carrying its own numeric convention.
/// Defaults to [`Monday`](WeekStart::Monday), the ISO 8601 standard common across Europe; pass
/// [`Sunday`](WeekStart::Sunday) or [`Saturday`](WeekStart::Saturday) for locales that begin the
/// week on those days.
///
/// [`CalendarGrid`]: crate::widget::CalendarGrid
/// [`DateField`]: crate::widget::form::DateField
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WeekStart {
    /// Monday, the ISO 8601 standard.
    #[default]
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl WeekStart {
    /// The weekday index in the `Sunday = 0 .. Saturday = 6` space, matching JavaScript's
    /// `Date.getDay` and [`PlainDate::week_day`](crate::widget::form::PlainDate::week_day).
    pub fn sunday_based_index(self) -> u32 {
        match self {
            WeekStart::Sunday => 0,
            WeekStart::Monday => 1,
            WeekStart::Tuesday => 2,
            WeekStart::Wednesday => 3,
            WeekStart::Thursday => 4,
            WeekStart::Friday => 5,
            WeekStart::Saturday => 6,
        }
    }
}
