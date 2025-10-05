use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser};
use jiff::civil::{Date, DateTime, Time};
use jiff::{Timestamp, Zoned, tz::TimeZone};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while_m_n, take_while1},
    character::complete::{char, space0, space1},
    combinator::{map, map_res},
    sequence::tuple,
};

#[derive(Parser, Debug)]
#[command(
    name = "datetime",
    about = "Parse and convert datetime to different timezones"
)]
pub struct DateTimeTool {
    /// DateTime value to parse (use "now" for current time)
    datetime: String,

    /// Input timezone to use when parsing datetime without timezone info (overrides any timezone in the input)
    #[arg(short = 's', long = "source-timezone")]
    source_timezone: Option<String>,

    /// Target timezone to convert to (e.g., "America/New_York", "UTC", "Asia/Tokyo")
    #[arg(short = 't', long = "target-timezone")]
    target_timezone: Option<String>,

    /// Input format string using custom specifiers for parsing
    ///
    /// Available format specifiers:
    /// - Year4: 4-digit year (e.g., 2025)
    /// - Year2: 2-digit year (e.g., 25, interpreted as 2025)
    /// - MonthName: Full month name (e.g., January, December)
    /// - MonthName3: 3-letter month abbreviation (e.g., Jan, Dec)
    /// - MonthNum2: 2-digit month (e.g., 01, 12)
    /// - MonthNum: 1-2 digit month (e.g., 1, 12)
    /// - Day2: 2-digit day (e.g., 01, 31)
    /// - Day: 1-2 digit day (e.g., 1, 31)
    /// - WeekdayName: Full weekday name (skipped, e.g., Monday)
    /// - WeekdayName3: 3-letter weekday abbreviation (skipped, e.g., Mon)
    /// - Hour24: 2-digit hour in 24-hour format (e.g., 00, 23)
    /// - Hour12: 2-digit hour in 12-hour format (e.g., 01, 12)
    /// - Minute2: 2-digit minute (e.g., 00, 59)
    /// - Minute: 1-2 digit minute (e.g., 0, 59)
    /// - Second: 2-digit second (e.g., 00, 59)
    /// - AMPM: AM/PM indicator
    /// - TZ: Timezone offset (e.g., +05:30, -08:00)
    /// - TZName: Timezone name (skipped, e.g., UTC, EST)
    ///
    /// Example: "MonthName Day2, Year4 Hour12:Minute2 AMPM"
    #[arg(long = "parse-format")]
    parse_format: Option<String>,
}

fn parse_with_format<'a>(
    input: &'a str,
    format: &str,
    in_timezone: Option<&TimeZone>,
) -> anyhow::Result<Zoned> {
    #[derive(Default)]
    struct ParsedDateTime {
        year: Option<i16>,
        month: Option<i8>,
        day: Option<i8>,
        hour: Option<i8>,
        minute: Option<i8>,
        second: Option<i8>,
        is_pm: bool,
        tz_offset: Option<(i8, i8)>,
    }

    // Individual parser functions
    fn parse_year4(input: &str) -> IResult<&str, i16, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(4, 4, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i16>(),
        )(input)
    }

    fn parse_year2(input: &str) -> IResult<&str, i16, ()> {
        map(
            map_res(
                take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
                |s: &str| s.parse::<i16>(),
            ),
            |year| 2000 + year,
        )(input)
    }

    fn parse_month_name_short_parser(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(3, 3, |c: char| c.is_alphabetic()),
            |s: &str| parse_month_name_short(s),
        )(input)
    }

    fn parse_month_name_full(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while1::<_, _, ()>(|c: char| c.is_alphabetic()),
            |s: &str| parse_month_name(s),
        )(input)
    }

    fn parse_month_num2(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_month_num(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(1, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_day2(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_day(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(1, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_hour(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_minute2(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_minute(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(1, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_second(input: &str) -> IResult<&str, i8, ()> {
        map_res(
            take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
            |s: &str| s.parse::<i8>(),
        )(input)
    }

    fn parse_ampm(input: &str) -> IResult<&str, bool, ()> {
        map(alt::<_, _, (), _>((tag("AM"), tag("PM"))), |s| s == "PM")(input)
    }

    fn parse_tz_offset(input: &str) -> IResult<&str, (i8, i8), ()> {
        map(
            tuple::<_, _, (), _>((
                alt::<_, _, (), _>((char('+'), char('-'))),
                map_res(
                    take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
                    |s: &str| s.parse::<i8>(),
                ),
                char(':'),
                map_res(
                    take_while_m_n::<_, _, ()>(2, 2, |c: char| c.is_ascii_digit()),
                    |s: &str| s.parse::<i8>(),
                ),
            )),
            |(sign, h, _, m)| {
                let hours = if sign == '-' { -h } else { h };
                (hours, m)
            },
        )(input)
    }

    fn skip_weekday_short(input: &str) -> IResult<&str, (), ()> {
        map(
            take_while_m_n::<_, _, ()>(3, 3, |c: char| c.is_alphabetic()),
            |_| (),
        )(input)
    }

    fn skip_weekday_full(input: &str) -> IResult<&str, (), ()> {
        map(take_while1::<_, _, ()>(|c: char| c.is_alphabetic()), |_| ())(input)
    }

    fn skip_tz_name(input: &str) -> IResult<&str, (), ()> {
        map(
            take_while1::<_, _, ()>(|c: char| c.is_alphanumeric()),
            |_| (),
        )(input)
    }

    // Build parser based on format string
    let mut parsed = ParsedDateTime::default();
    let mut remaining = input;
    let mut format_str = format;

    while !format_str.is_empty() {
        // Try to match format specifiers (longest first)
        if let Some(rest) = format_str.strip_prefix("Year4") {
            let (rest_input, year) = parse_year4(remaining)?;
            parsed.year = Some(year);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Year2") {
            let (rest_input, year) = parse_year2(remaining)?;
            parsed.year = Some(year);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("MonthName3") {
            let (rest_input, month) = parse_month_name_short_parser(remaining)?;
            parsed.month = Some(month);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("MonthName") {
            let (rest_input, month) = parse_month_name_full(remaining)?;
            parsed.month = Some(month);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("MonthNum2") {
            let (rest_input, month) = parse_month_num2(remaining)?;
            parsed.month = Some(month);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("MonthNum") {
            let (rest_input, month) = parse_month_num(remaining)?;
            parsed.month = Some(month);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("WeekdayName3") {
            let (rest_input, _) = skip_weekday_short(remaining)?;
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("WeekdayName") {
            let (rest_input, _) = skip_weekday_full(remaining)?;
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Day2") {
            let (rest_input, day) = parse_day2(remaining)?;
            parsed.day = Some(day);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Day") {
            let (rest_input, day) = parse_day(remaining)?;
            parsed.day = Some(day);
            // Consume optional trailing whitespace after Day
            let (rest_input, _) = space0::<_, ()>(rest_input)?;
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Hour24") {
            let (rest_input, hour) = parse_hour(remaining)?;
            parsed.hour = Some(hour);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Hour12") {
            let (rest_input, hour) = parse_hour(remaining)?;
            parsed.hour = Some(hour);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Minute2") {
            let (rest_input, minute) = parse_minute2(remaining)?;
            parsed.minute = Some(minute);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Minute") {
            let (rest_input, minute) = parse_minute(remaining)?;
            parsed.minute = Some(minute);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("Second") {
            let (rest_input, second) = parse_second(remaining)?;
            parsed.second = Some(second);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("AMPM") {
            let (rest_input, is_pm) = parse_ampm(remaining)?;
            parsed.is_pm = is_pm;
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("TZ") {
            let (rest_input, tz_offset) = parse_tz_offset(remaining)?;
            parsed.tz_offset = Some(tz_offset);
            remaining = rest_input;
            format_str = rest;
        } else if let Some(rest) = format_str.strip_prefix("TZName") {
            let (rest_input, _) = skip_tz_name(remaining)?;
            remaining = rest_input;
            format_str = rest;
        } else if format_str.starts_with(' ') {
            let (rest_input, _) = space1::<_, ()>(remaining)?;
            remaining = rest_input;
            format_str = &format_str[1..];
        } else {
            let ch = format_str.chars().next().unwrap();
            let (rest_input, _) = char::<_, ()>(ch)(remaining)?;
            remaining = rest_input;
            format_str = &format_str[ch.len_utf8()..];
        }
    }

    // Validate we consumed all input
    if !remaining.is_empty() {
        anyhow::bail!(
            "Input does not match format - extra characters: {}",
            remaining
        );
    }

    // Extract final values
    let year = parsed.year.context("Year not found in format")?;
    let month = parsed.month.context("Month not found in format")?;
    let day = parsed.day.context("Day not found in format")?;

    let mut hour = parsed.hour.unwrap_or(0);
    if parsed.is_pm {
        hour = if hour == 12 { 12 } else { hour + 12 };
    } else if hour == 12 && parsed.hour.is_some() {
        // 12 AM is 00:00
        hour = 0;
    }

    let minute = parsed.minute.unwrap_or(0);
    let second = parsed.second.unwrap_or(0);

    // Build datetime
    let date = Date::new(year, month, day)?;
    let time = Time::new(hour, minute, second, 0)?;
    let dt = DateTime::from_parts(date, time);

    // Handle timezone
    let tz = if let Some((hours, minutes)) = parsed.tz_offset {
        let total_hours = (hours as i32 * 60 + minutes as i32 * hours.signum() as i32) / 60;
        TimeZone::fixed(jiff::tz::offset(total_hours as i8))
    } else {
        in_timezone.cloned().unwrap_or(TimeZone::UTC)
    };

    Ok(dt.to_zoned(tz)?)
}

fn parse_month_name(name: &str) -> anyhow::Result<i8> {
    match name.to_lowercase().as_str() {
        "january" => Ok(1),
        "february" => Ok(2),
        "march" => Ok(3),
        "april" => Ok(4),
        "may" => Ok(5),
        "june" => Ok(6),
        "july" => Ok(7),
        "august" => Ok(8),
        "september" => Ok(9),
        "october" => Ok(10),
        "november" => Ok(11),
        "december" => Ok(12),
        _ => anyhow::bail!("Invalid month name: {}", name),
    }
}

fn parse_month_name_short(name: &str) -> anyhow::Result<i8> {
    match name.to_lowercase().as_str() {
        "jan" => Ok(1),
        "feb" => Ok(2),
        "mar" => Ok(3),
        "apr" => Ok(4),
        "may" => Ok(5),
        "jun" => Ok(6),
        "jul" => Ok(7),
        "aug" => Ok(8),
        "sep" => Ok(9),
        "oct" => Ok(10),
        "nov" => Ok(11),
        "dec" => Ok(12),
        _ => anyhow::bail!("Invalid month abbreviation: {}", name),
    }
}

impl Tool for DateTimeTool {
    fn cli() -> Command {
        DateTimeTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        // Parse the input datetime
        let mut zoned = if self.datetime.to_lowercase() == "now" {
            Zoned::now()
        } else if let Some(ref parse_format) = self.parse_format {
            // Parse using custom format
            let in_tz = if let Some(ref in_tz_str) = self.source_timezone {
                Some(TimeZone::get(in_tz_str).context("Could not parse input timezone")?)
            } else {
                None
            };
            parse_with_format(&self.datetime, parse_format, in_tz.as_ref())?
        } else {
            // Try parsing as Zoned first
            self.datetime.parse::<Zoned>().or_else(|_| {
                // Try parsing as Timestamp (handles ISO 8601 with offset/Z but no timezone name)
                let datetime_str = self.datetime.replace('Z', "+00:00");
                datetime_str
                    .parse::<Timestamp>()
                    .map(|ts| ts.to_zoned(TimeZone::UTC))
                    .or_else(|_| -> anyhow::Result<Zoned> {
                        // If no offset, try parsing as civil datetime and use input timezone or UTC
                        use jiff::civil::DateTime;
                        let dt: DateTime =
                            self.datetime.parse().context("Could not parse datetime")?;
                        let tz = if let Some(ref in_tz_str) = self.source_timezone {
                            TimeZone::get(in_tz_str).context("Could not parse input timezone")?
                        } else {
                            TimeZone::UTC
                        };
                        Ok(dt.to_zoned(tz)?)
                    })
            })?
        };

        // Apply input timezone if specified (overrides parsed timezone) - only if not already applied during parsing
        if let Some(ref in_tz_str) = self.source_timezone {
            // Check if we already used source_timezone during parsing by checking if the datetime had no offset
            if self.parse_format.is_none()
                && (self.datetime.contains('+')
                    || self.datetime.contains('Z')
                    || self.datetime.contains('['))
            {
                let in_tz = TimeZone::get(in_tz_str).context("Could not parse input timezone")?;
                let dt = zoned.datetime();
                zoned = dt.to_zoned(in_tz)?;
            }
        }

        // Helper function to format datetime
        let format_datetime = |z: &Zoned| -> String {
            let offset = z.offset();
            let dt = z.datetime();
            let date = dt.date();
            let time = dt.time();
            let formatted_time = format!(
                "{:02}:{:02}:{:02}",
                time.hour(),
                time.minute(),
                time.second()
            );

            if offset.seconds() == 0 {
                // UTC - use Z notation
                format!("{}T{}Z", date, formatted_time)
            } else {
                // Non-UTC - use offset notation
                let offset_str = if offset.seconds() >= 0 {
                    let hours = offset.seconds() / 3600;
                    let minutes = (offset.seconds() % 3600) / 60;
                    format!("+{:02}:{:02}", hours, minutes)
                } else {
                    let hours = (-offset.seconds()) / 3600;
                    let minutes = ((-offset.seconds()) % 3600) / 60;
                    format!("-{:02}:{:02}", hours, minutes)
                };
                format!("{}T{}{}", date, formatted_time, offset_str)
            }
        };

        // Generate outputs for local, UTC, and target timezone
        let local_tz = TimeZone::system();
        let local_time = zoned.with_time_zone(local_tz);
        let utc_time = zoned.with_time_zone(TimeZone::UTC);

        let mut result = serde_json::json!({
            "local": format_datetime(&local_time),
            "utc": format_datetime(&utc_time),
        });

        // Add target timezone if specified
        if let Some(ref tz_str) = self.target_timezone {
            let tz = TimeZone::get(tz_str).context("Could not parse timezone")?;
            let target_time = zoned.with_time_zone(tz);
            result["target"] = serde_json::json!(format_datetime(&target_time));
        }

        Ok(Some(Output::JsonValue(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso8601_with_z() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00Z".to_string(),
            source_timezone: None,
            target_timezone: None,
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_iso8601_with_offset() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00+05:30".to_string(),
            source_timezone: None,
            target_timezone: None,
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_with_timezone() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00[America/New_York]".to_string(),
            source_timezone: None,
            target_timezone: None,
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_in_timezone() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00Z".to_string(),
            source_timezone: Some("America/New_York".to_string()),
            target_timezone: None,
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            let utc = val["utc"].as_str().unwrap();
            // source_timezone overrides the Z, reinterpreting 15:30 as New York time
            // New York is UTC-4 or UTC-5, so 15:30 in NY becomes 19:30 or 20:30 UTC
            assert!(utc.contains("2025-10-04") && (utc.contains("19:30") || utc.contains("20:30")));
        }
    }

    #[test]
    fn test_to_timezone_conversion() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00Z".to_string(),
            source_timezone: None,
            target_timezone: Some("Asia/Tokyo".to_string()),
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            let target = val["target"].as_str().unwrap();
            assert!(target.contains("+09:00")); // JST
        }
    }

    #[test]
    fn test_in_and_to_timezone_combined() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00".to_string(),
            source_timezone: Some("UTC".to_string()),
            target_timezone: Some("Asia/Kolkata".to_string()),
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            let target = val["target"].as_str().unwrap();
            assert!(target.contains("+05:30")); // IST
        }
    }

    #[test]
    fn test_default_iso_format_utc() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00Z".to_string(),
            source_timezone: None,
            target_timezone: None,
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            let utc = val["utc"].as_str().unwrap();
            assert!(utc.ends_with('Z'));
            assert!(utc.contains("2025-10-04T15:30:00"));
        }
    }

    #[test]
    fn test_default_iso_format_with_offset() {
        let tool = DateTimeTool {
            datetime: "2025-10-04T15:30:00+05:30".to_string(),
            source_timezone: None,
            target_timezone: None,
            parse_format: None,
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            let utc = val["utc"].as_str().unwrap();
            // UTC should end with Z
            assert!(utc.ends_with('Z'));
        }
    }

    #[test]
    fn test_parse_with_custom_format() {
        let tool = DateTimeTool {
            datetime: "04/10/2025 15:30".to_string(),
            source_timezone: Some("UTC".to_string()),
            target_timezone: None,
            parse_format: Some("Day2/MonthNum2/Year4 Hour24:Minute2".to_string()),
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            assert_eq!(val["utc"].as_str().unwrap(), "2025-10-04T15:30:00Z");
        }
    }

    #[test]
    fn test_parse_with_month_name() {
        let tool = DateTimeTool {
            datetime: "October 04, 2025 03:30 PM".to_string(),
            source_timezone: Some("UTC".to_string()),
            target_timezone: None,
            parse_format: Some("MonthName Day2, Year4 Hour12:Minute2 AMPM".to_string()),
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            assert_eq!(val["utc"].as_str().unwrap(), "2025-10-04T15:30:00Z");
        }
    }

    #[test]
    fn test_parse_with_timezone_offset() {
        let tool = DateTimeTool {
            datetime: "2025-10-04 15:30:00 +05:30".to_string(),
            source_timezone: None,
            target_timezone: Some("UTC".to_string()),
            parse_format: Some("Year4-MonthNum2-Day2 Hour24:Minute2:Second TZ".to_string()),
        };

        let result = tool.execute().unwrap();
        if let Some(Output::JsonValue(val)) = result {
            let target = val["target"].as_str().unwrap();
            // Converted to UTC, should end with Z
            assert!(target.ends_with('Z'));
        }
    }
}
