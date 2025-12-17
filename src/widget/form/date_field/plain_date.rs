use js_sys::Date;
use std::{fmt, str::FromStr};

/// A date without time information (Year, Month, Day).
/// Months are 0-based (0 = January, 11 = December).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlainDate {
    year: i32,
    month: u32,
    day: u32,
}

impl PlainDate {
    /// Create a new PlainDate.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Create from a timestamp (milliseconds).
    /// Uses local time extraction.
    pub fn from_timestamp(ts: f64) -> Self {
        let d = Date::new(&ts.into());
        Self {
            year: d.get_full_year() as i32,
            month: d.get_month(),
            day: d.get_date(),
        }
    }

    /// Return a new [`js_sys::Date`] with a time value of 00:00:00.
    pub fn to_date(&self) -> Date {
        Date::new_with_year_month_day(self.year as u32, self.month as i32, self.day as i32)
    }

    /// Convert to a timestamp (milliseconds).
    /// Returns the timestamp for 12:00:00 (Noon) local time on this date.
    pub fn to_timestamp(&self) -> f64 {
        let d = self.to_date();
        d.set_hours(12);
        d.get_time()
    }

    /// Convert to a timestamp (milliseconds).
    /// Returns the timestamp for 00:00:00 local time on this date.
    pub fn to_timestamp_start(&self) -> f64 {
        self.to_date().get_time()
    }

    /// Convert to a timestamp (milliseconds).
    /// Returns the timestamp for 23:59:59 local time on this date.
    pub fn to_timestamp_end(&self) -> f64 {
        let d = self.to_date();
        d.set_hours(23);
        d.set_minutes(59);
        d.set_seconds(59);
        d.get_time()
    }

    /// Get today's date.
    pub fn today() -> Self {
        let now_ts = Date::now();
        Self::from_timestamp(now_ts)
    }

    /// Add a number of days to this date.
    pub fn add_days(&self, days: i32) -> Self {
        // Use noon timestamp for safe arithmetic
        let ts = self.to_timestamp();
        let one_day_ms = 24.0 * 60.0 * 60.0 * 1000.0;
        let new_ts = ts + (days as f64 * one_day_ms);
        Self::from_timestamp(new_ts)
    }

    /// Get the year.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Get the month (0-11).
    pub fn month(&self) -> u32 {
        self.month
    }

    /// Get the day of the month (1-31).
    pub fn day(&self) -> u32 {
        self.day
    }

    /// Get the day of the week (0 = Sunday, 6 = Saturday).
    pub fn week_day(&self) -> u32 {
        let d = Date::new(&self.to_timestamp().into());
        d.get_day()
    }

    /// Get the ISO 8601 week number (1-53).
    ///
    /// The algorithm matches standard ISO week definition: the week with the year's first Thursday.
    pub fn iso_week(&self) -> u32 {
        let d = Date::new(&self.to_timestamp().into());
        // ISO week date is determined by the Thursday of the week.
        // Thursday is day 4 (Sunday=0 in JS Date) -> (day + 6) % 7 gives Mon=0..Sun=6.
        // But easier: set date to nearest Thursday.
        // Current day relative to Sunday: 0..6
        let day = d.get_day();
        // Adjust to Monday-based indexing: Mon=1,... Sun=7 (or similar logic)
        // Standard Algo:
        // 1. Find Thursday of this week.
        //    (Sunday=0, Mon=1, ... Sat=6)
        //    diff = 4 - (day || 7)  <-- treats Sunday as 7?
        //    Let's stick to standard JS Date manipulation approach often used:
        //    Target = date + (4 - (day||7)) days.

        let day_n = if day == 0 { 7 } else { day }; // Sunday is 7
        d.set_date(d.get_date() + 4 - day_n);

        // Get first day of year
        let year_start = Date::new_0();
        year_start.set_full_year(d.get_full_year());
        year_start.set_month(0);
        year_start.set_date(1);
        year_start.set_hours(0); // Ensure time is normalized

        // Calculate full weeks to nearest Thursday
        let diff_ms = d.get_time() - year_start.get_time();
        // 86400000 ms/day
        let day_diff = (diff_ms / 86400000.0).ceil();

        ((day_diff + 1.0) / 7.0).ceil() as u32
    }

    /// Format the date using the given format string.
    /// Supported tokens (submenu of PHP DateTime::format):
    /// - Y: 4-digit year (e.g. 2023)
    /// - y: 2-digit year (e.g. 23)
    /// - m: Month with leading zeros (01-12)
    /// - n: Month without leading zeros (1-12)
    /// - d: Day with leading zeros (01-31)
    /// - j: Day without leading zeros (1-31)
    /// Reference: https://www.php.net/manual/en/datetime.format.php
    pub fn format(&self, fmt: &str) -> String {
        let mut res = fmt.to_string();
        // Order matters to prevent partial replacements of longer tokens
        // Year
        res = res.replace("Y", &format!("{:04}", self.year)); // PHP: 4 digit
        res = res.replace("y", &format!("{:02}", self.year % 100)); // PHP: 2 digit

        // Month
        res = res.replace("m", &format!("{:02}", self.month + 1)); // PHP: 01-12
        res = res.replace("n", &format!("{}", self.month + 1)); // PHP: 1-12

        // Day
        res = res.replace("d", &format!("{:02}", self.day)); // PHP: 01-31
        res = res.replace("j", &format!("{}", self.day)); // PHP: 1-31

        res
    }

    fn get_current_year() -> i32 {
        #[cfg(target_arch = "wasm32")]
        {
            let d = Date::new(&Date::now().into());
            d.get_full_year() as i32
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            2025
        }
    }

    /// Parse a date string using the given format string.
    /// Supported tokens (submenu of PHP DateTime::format):
    /// - Y: 4-digit year
    /// - y: 2-digit year (pivots at 2000)
    /// - m: Month with leading zeros
    /// - n: Month without leading zeros
    /// - d: Day with leading zeros
    /// - j: Day without leading zeros
    /// Reference: https://www.php.net/manual/en/datetime.format.php
    pub fn from_format(mut input: &str, fmt: &str) -> Result<Self, String> {
        // Implementation:
        let mut year: Option<i32> = None;
        let mut month: Option<u32> = None;
        let mut day: Option<u32> = None;

        fn parse_num<NUM: FromStr>(input: &str, err: String) -> Result<NUM, String> {
            input.parse().map_err(|_| err)
        }

        fn find_num_length(input: &str) -> usize {
            let mut chars = input.chars();
            if let Some(c1) = chars.next() {
                if c1.is_ascii_digit() {
                    if ('1'..='3').contains(&c1) {
                        if let Some(c2) = chars.next() {
                            if c2.is_ascii_digit() {
                                return 2;
                            }
                        }
                    }
                    return 1;
                }
            }
            0
        }

        for f_char in fmt.chars() {
            let took = match f_char {
                'Y' => {
                    // PHP Y: 4 digit year
                    if input.len() < 4 {
                        return Err("Input ends before Year".into());
                    }
                    year = Some(parse_num(&input[..4], "Invalid year".to_string())?);
                    4
                }
                'y' => {
                    // PHP y: 2 digit year
                    if input.len() < 2 {
                        return Err("Input ends before Year".into());
                    }
                    let y: i32 = parse_num(&input[..2], "Invalid year".to_string())?;
                    year = Some(y + 2000);
                    2
                }
                'm' => {
                    // PHP m: Month with leading zeros (01-12)
                    if input.len() < 2 {
                        return Err("Input ends before Month".into());
                    }
                    month = Some(parse_num(&input[..2], "Invalid month".to_string())?);
                    2
                }
                'd' => {
                    // PHP d: Day with leading zeros (01-31)
                    if input.len() < 2 {
                        return Err("Input ends before Day".into());
                    }
                    day = Some(parse_num(&input[..2], "Invalid day".to_string())?);
                    2
                }
                'n' => {
                    // PHP n: Month without leading zeros (1-12)
                    let took = find_num_length(input);
                    if took == 0 {
                        return Err("Expected digit for Month".into());
                    }
                    month = Some(parse_num(&input[..took], "Invalid month".to_string())?);
                    took
                }
                'j' => {
                    // PHP j: Day without leading zeros (1-31)
                    let took = find_num_length(input);
                    if took == 0 {
                        return Err("Expected digit for Day".into());
                    }
                    day = Some(parse_num(&input[..took], "Invalid day".to_string())?);
                    took
                }
                _ => {
                    // Literal match
                    if let Some(c) = input.chars().next() {
                        if f_char != c {
                            return Err(format!("Expected '{}', found '{}'", f_char, c));
                        }
                    } else {
                        return Err("Input ends too soon.".to_string());
                    }
                    1
                }
            };
            input = &input[took..];
        }

        if !input.is_empty() {
            return Err("Input longer than format".into());
        }

        if year.is_none() {
            year = Some(Self::get_current_year());
        }

        match (year, month, day) {
            (Some(y), Some(m), Some(d)) => {
                if m < 1 || m > 12 || d < 1 || d > 31 {
                    Err("Invalid date values".into())
                } else {
                    Ok(PlainDate::new(y, m - 1, d))
                }
            }
            _ => Err("Missing year, month, or day".into()),
        }
    }
}

impl fmt::Display for PlainDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format as Y-m-d (ISO 8601)
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month + 1, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_date_new_and_getters() {
        let d = PlainDate::new(2023, 0, 15);
        assert_eq!(d.year(), 2023);
        assert_eq!(d.month(), 0);
        assert_eq!(d.day(), 15);
    }

    #[test]
    fn test_format_iso() {
        let d = PlainDate::new(2023, 11, 31); // Dec 31
        assert_eq!(d.format("Y-m-d"), "2023-12-31");
    }

    #[test]
    fn test_format_custom() {
        let d = PlainDate::new(2023, 0, 5); // Jan 5
        assert_eq!(d.format("d.m.Y"), "05.01.2023");
        assert_eq!(d.format("n/j/Y"), "1/5/2023");
    }

    #[test]
    fn test_from_format_iso() {
        let d = PlainDate::from_format("2023-12-31", "Y-m-d").unwrap();
        assert_eq!(d.year(), 2023);
        assert_eq!(d.month(), 11);
        assert_eq!(d.day(), 31);
    }

    #[test]
    fn test_from_format_custom() {
        let d = PlainDate::from_format("05.01.2023", "d.m.Y").unwrap();
        assert_eq!(d.year(), 2023);
        assert_eq!(d.month(), 0);
        assert_eq!(d.day(), 5);

        let d2 = PlainDate::from_format("1/5/2023", "n/j/Y").unwrap();
        assert_eq!(d2.year(), 2023);
        assert_eq!(d2.month(), 0);
        assert_eq!(d2.day(), 5);
    }

    #[test]
    fn test_from_format_with_variable_length() {
        // Test n/j logic with single/double digits
        let d = PlainDate::from_format("12/12/2023", "n/j/Y").unwrap();
        assert_eq!(d.month(), 11);
        assert_eq!(d.day(), 12);

        // Single digits
        let d2 = PlainDate::from_format("1/2/2023", "n/j/Y").unwrap();
        assert_eq!(d2.month(), 0);
        assert_eq!(d2.day(), 2);
    }

    #[test]
    fn test_invalid_dates() {
        assert!(PlainDate::from_format("2023-13-01", "Y-m-d").is_err());
        assert!(PlainDate::from_format("2023-00-01", "Y-m-d").is_err());
        assert!(PlainDate::from_format("2023-01-32", "Y-m-d").is_err());
        assert!(PlainDate::from_format("garbage", "Y-m-d").is_err());
    }
    #[test]
    fn test_php_shortcuts() {
        // n = month 1-12 (no leading zero required), j = day 1-31 (no leading zero)
        let d = PlainDate::from_format("1/2/2023", "n/j/Y").unwrap();
        assert_eq!(d.month(), 0);
        assert_eq!(d.day(), 2);

        let d2 = PlainDate::from_format("10/12/2023", "n/j/Y").unwrap();
        assert_eq!(d2.month(), 9);
        assert_eq!(d2.day(), 12);

        // y = 2 digit year
        let d3 = PlainDate::from_format("1/2/23", "n/j/y").unwrap();
        assert_eq!(d3.year(), 2023);

        // m = month 01-12 (leading zero), d = day 01-31 (leading zero)
        // strict m/d usage when available
        let d4 = PlainDate::from_format("01/02/2023", "m/d/Y").unwrap();
        assert_eq!(d4.month(), 0);
        assert_eq!(d4.day(), 2);

        assert!(PlainDate::from_format("01/02/zzzz", "m/d/Y").is_err());
        assert!(PlainDate::from_format("01/yy/2023", "m/d/Y").is_err());
        assert!(PlainDate::from_format("xx/02/zzzz", "m/d/Y").is_err());
        assert!(PlainDate::from_format("01/yy/zzzz", "m/d/Y").is_err());
        assert!(PlainDate::from_format("xx/02/zzzz", "m/d/Y").is_err());
        assert!(PlainDate::from_format("xx/yy/2023", "m/d/Y").is_err());
        assert!(PlainDate::from_format("xx/yy/zzzz", "m/d/Y").is_err());

        assert!(PlainDate::from_format("xx01/02/2023", "mm/d/Y").is_err());
        assert!(PlainDate::from_format("1022023", "nmY").is_err())
    }

    #[test]
    fn test_format_php_shortcuts() {
        let d = PlainDate::new(2023, 0, 5); // Jan 5
        assert_eq!(d.format("Y-m-d"), "2023-01-05");
        assert_eq!(d.format("y-n-j"), "23-1-5");

        let d2 = PlainDate::new(2023, 11, 23); // Dec 23
        assert_eq!(d2.format("y-n-j"), "23-12-23");
        assert_eq!(d2.format("Y-m-d"), "2023-12-23");
    }
    #[test]
    fn test_from_format_missing_year() {
        let current_year = PlainDate::get_current_year();
        let d = PlainDate::from_format("08-22", "m-d").unwrap();
        assert_eq!(d.year(), current_year);
        assert_eq!(d.month(), 7); // 0-based
        assert_eq!(d.day(), 22);
    }
}
